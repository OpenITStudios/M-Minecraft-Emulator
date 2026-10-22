//! Real, read-only Windows ACL access checks for engine-created test roots.
//!
//! A private-looking directory or a Rust process lease is NOT confinement.
//! This module compares the real directory security descriptor against two
//! OS process tokens with Windows AccessCheck. An owner/peer pair can be
//! rejected as shared; even peer denial is diagnostic ONLY until a trusted
//! service, root provenance, process isolation and all other broker/IPC
//! surfaces have been proven. No ACLs, tokens or accounts are modified.
//!
//! The native proof is deliberately unreachable from Tauri/Minecraft launch.

#![allow(dead_code)]

use std::{ffi::c_void, fs::File, io, mem, os::windows::io::AsRawHandle, ptr};
#[cfg(test)]
use super::independent_job::SuspendedFixture;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};

const READ_CONTROL: u32 = 0x0002_0000;
const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_DUPLICATE: u32 = 0x0002;
const FILE_LIST_DIRECTORY: u32 = 0x0001;
const FILE_ADD_FILE: u32 = 0x0002;
const OWNER_SECURITY_INFORMATION: u32 = 0x0001;
const GROUP_SECURITY_INFORMATION: u32 = 0x0002;
const DACL_SECURITY_INFORMATION: u32 = 0x0004;
const SE_FILE_OBJECT: u32 = 1;
const SECURITY_IMPERSONATION: u32 = 2;
const TOKEN_IMPERSONATION: u32 = 2;

/// ABI-equivalent GENERIC_MAPPING from winnt.h. The requested directory
/// rights below are already specific, not GENERIC_* masks.
#[repr(C)]
struct GenericMapping {
    generic_read: u32,
    generic_write: u32,
    generic_execute: u32,
    generic_all: u32,
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn GetSecurityInfo(
        handle: HANDLE, object_type: u32, security_information: u32,
        owner: *mut *mut c_void, group: *mut *mut c_void,
        dacl: *mut *mut c_void, sacl: *mut *mut c_void,
        descriptor: *mut *mut c_void,
    ) -> u32;
    fn OpenProcessToken(
        process: HANDLE, desired_access: u32, token: *mut HANDLE,
    ) -> i32;
    fn DuplicateTokenEx(
        token: HANDLE, desired_access: u32, attributes: *const c_void,
        impersonation_level: u32, token_type: u32, duplicate: *mut HANDLE,
    ) -> i32;
    fn AccessCheck(
        descriptor: *mut c_void, impersonation_token: HANDLE,
        desired_access: u32, mapping: *mut GenericMapping,
        privileges: *mut c_void, privileges_size: *mut u32,
        granted_access: *mut u32, access_status: *mut i32,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
}

struct OwnedToken(HANDLE);

impl OwnedToken {
    fn new(raw: HANDLE) -> io::Result<Self> {
        if raw.is_null() {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(raw))
        }
    }
}

impl Drop for OwnedToken {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: We own exactly one process-token/duplicate handle.
            unsafe { CloseHandle(self.0) };
        }
    }
}

struct OwnedDescriptor(*mut c_void);

impl Drop for OwnedDescriptor {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: GetSecurityInfo owns this allocation via LocalFree.
            let _ = unsafe { LocalFree(self.0) };
        }
    }
}

/// Borrowed descriptor data is valid only while its LocalFree-owned buffer
/// remains live. Never return the descriptor, raw SID, ACL or token to UI.
fn read_security_descriptor(directory: &File) -> io::Result<OwnedDescriptor> {
    // A root opened without READ_CONTROL is not an authority to inspect it.
    // The caller's OS handle rights control GetSecurityInfo.
    let mut owner: *mut c_void = ptr::null_mut();
    let mut group: *mut c_void = ptr::null_mut();
    let mut dacl: *mut c_void = ptr::null_mut();
    let mut descriptor: *mut c_void = ptr::null_mut();
    // SAFETY: The directory HANDLE remains live and each pointer is a
    // writable stack location. The returned descriptor owns owner/group/DACL.
    let code = unsafe {
        GetSecurityInfo(
            directory.as_raw_handle() as HANDLE,
            SE_FILE_OBJECT,
            OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION,
            &mut owner, &mut group, &mut dacl, ptr::null_mut(), &mut descriptor,
        )
    };
    let guard = OwnedDescriptor(descriptor);
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    // AccessCheck treats a NULL DACL as full access. It is NEVER acceptable
    // as a private root. Missing owner/group is also a hard NO-GO.
    if guard.0.is_null() || owner.is_null() || group.is_null() || dacl.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Private root lacks owner, group or non-NULL DACL",
        ));
    }
    Ok(guard)
}

fn impersonation_token(process: HANDLE) -> io::Result<OwnedToken> {
    let mut primary: HANDLE = ptr::null_mut();
    // SAFETY: Read the token belonging to a previously verified, still-open
    // process HANDLE. No user or process ID is accepted by this function.
    if unsafe {
        OpenProcessToken(process, TOKEN_QUERY | TOKEN_DUPLICATE, &mut primary)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    let primary = OwnedToken::new(primary)?;
    let mut duplicate: HANDLE = ptr::null_mut();
    // AccessCheck requires a SECURITY_IMPERSONATION-level impersonation
    // token, never the unmodified process primary token.
    // SAFETY: OS duplicates our owned primary token with TOKEN_QUERY only.
    if unsafe {
        DuplicateTokenEx(
            primary.0, TOKEN_QUERY, ptr::null(), SECURITY_IMPERSONATION,
            TOKEN_IMPERSONATION, &mut duplicate,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    OwnedToken::new(duplicate)
}

/// Ask the actual Windows access-check algorithm about a *specific*
/// directory right. These masks contain no GENERIC_* bits.
fn allowed(descriptor: &OwnedDescriptor, token: &OwnedToken, right: u32)
    -> io::Result<bool>
{
    if !matches!(right, FILE_LIST_DIRECTORY | FILE_ADD_FILE) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, "Unreviewed native access right",
        ));
    }
    // FILE_GENERIC_READ / FILE_GENERIC_WRITE / FILE_GENERIC_EXECUTE /
    // FILE_ALL_ACCESS from WinNT.h, provided only for correct ABI mapping.
    let mut mapping = GenericMapping {
        generic_read: 0x0012_0089,
        generic_write: 0x0012_0116,
        generic_execute: 0x0012_00A0,
        generic_all: 0x001F_01FF,
    };
    // The API uses PRIVILEGE_SET as an OUT buffer; usize words provide
    // correct alignment. 4096 bytes is a deliberate fixed upper bound.
    // Excess privileges / ERROR_INSUFFICIENT_BUFFER fail closed.
    let mut privileges = [0usize; 512];
    let mut length = mem::size_of_val(&privileges) as u32;
    let mut granted = 0u32;
    let mut status = 0i32;
    // SAFETY: The security descriptor and impersonation token are retained
    // for this entire call. All output sizes/pointers are bounds checked.
    if unsafe {
        AccessCheck(
            descriptor.0, token.0, right, &mut mapping,
            privileges.as_mut_ptr().cast(), &mut length,
            &mut granted, &mut status,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if length > mem::size_of_val(&privileges) as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Privilege report exceeded fixed buffer",
        ));
    }
    if status != 0 && granted & right != right {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Inconsistent AccessCheck grant result",
        ));
    }
    Ok(status != 0)
}

/// Sanitized evidence: never exposes SID, token, ACL or private paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PairAccessFacts {
    pub owner_can_list: bool,
    pub owner_can_create: bool,
    pub peer_can_list: bool,
    pub peer_can_create: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PairAccessReview {
    OwnerCannotUseRoot,
    PeerAccessObserved,
    // Still NOT authorization: uncontrolled peers, ancestors, privileges,
    // tokens, Microsoft brokers and actual game paths remain unverified.
    PeerDeniedCandidateOnly,
}

impl PairAccessFacts {
    pub fn review(self) -> PairAccessReview {
        if !self.owner_can_list || !self.owner_can_create {
            PairAccessReview::OwnerCannotUseRoot
        } else if self.peer_can_list || self.peer_can_create {
            PairAccessReview::PeerAccessObserved
        } else {
            PairAccessReview::PeerDeniedCandidateOnly
        }
    }
}

/// Fixture integration only. Two distinct OS processes must be created by
/// the engine itself; each retained process handle is checked before and
/// after the actual DACL/impersonation-token queries.
#[cfg(test)]
pub(super) fn review_fixture_pair(
    root: &File, owner: &SuspendedFixture, peer: &SuspendedFixture,
) -> io::Result<PairAccessFacts> {
    if owner.live_identity()?.pid == peer.live_identity()?.pid {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, "Peer and owner are the same process",
        ));
    }
    owner.with_live_process(|owner_handle| {
        peer.with_live_process(|peer_handle| {
            let sd = read_security_descriptor(root)?;
            let owner_token = impersonation_token(owner_handle)?;
            let peer_token = impersonation_token(peer_handle)?;
            Ok(PairAccessFacts {
                owner_can_list: allowed(&sd, &owner_token, FILE_LIST_DIRECTORY)?,
                owner_can_create: allowed(&sd, &owner_token, FILE_ADD_FILE)?,
                peer_can_list: allowed(&sd, &peer_token, FILE_LIST_DIRECTORY)?,
                peer_can_create: allowed(&sd, &peer_token, FILE_ADD_FILE)?,
            })
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returned_facts_never_promote_unknown_owner_or_peer_access() {
        let shared = PairAccessFacts {
            owner_can_list: true, owner_can_create: true,
            peer_can_list: true, peer_can_create: true,
        };
        assert_eq!(shared.review(), PairAccessReview::PeerAccessObserved);
        let unusable = PairAccessFacts {
            owner_can_list: false, owner_can_create: true,
            peer_can_list: false, peer_can_create: false,
        };
        assert_eq!(unusable.review(), PairAccessReview::OwnerCannotUseRoot);
        let candidate = PairAccessFacts {
            owner_can_list: true, owner_can_create: true,
            peer_can_list: false, peer_can_create: false,
        };
        assert_eq!(candidate.review(), PairAccessReview::PeerDeniedCandidateOnly);
    }
}
