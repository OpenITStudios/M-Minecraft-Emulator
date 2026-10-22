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
const FILE_ADD_SUBDIRECTORY: u32 = 0x0004;
const FILE_READ_EA: u32 = 0x0008;
const FILE_WRITE_EA: u32 = 0x0010;
const FILE_TRAVERSE: u32 = 0x0020;
const FILE_DELETE_CHILD: u32 = 0x0040;
const FILE_READ_ATTRIBUTES: u32 = 0x0080;
const FILE_WRITE_ATTRIBUTES: u32 = 0x0100;
const DELETE: u32 = 0x0001_0000;
const WRITE_DAC: u32 = 0x0004_0000;
const WRITE_OWNER: u32 = 0x0008_0000;
const TOKEN_USER_CLASS: u32 = 1;
const ERROR_INSUFFICIENT_BUFFER: i32 = 122;
const MAX_TOKEN_USER_BYTES: usize = 4096;
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
    fn GetTokenInformation(
        token: HANDLE, class: u32, data: *mut c_void,
        capacity: u32, length: *mut u32,
    ) -> i32;
    fn IsValidSid(sid: *mut c_void) -> i32;
    fn GetLengthSid(sid: *mut c_void) -> u32;
    fn EqualSid(left: *mut c_void, right: *mut c_void) -> i32;
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

struct OwnedDescriptor {
    allocation: *mut c_void,
    // Borrowed pointer into the same LocalFree-owned descriptor allocation.
    // Its lifetime MUST NOT outlive this object.
    owner_sid: *mut c_void,
}

impl Drop for OwnedDescriptor {
    fn drop(&mut self) {
        if !self.allocation.is_null() {
            // SAFETY: GetSecurityInfo owns this allocation via LocalFree.
            let _ = unsafe { LocalFree(self.allocation) };
        }
    }
}

// TOKEN_USER = SID_AND_ATTRIBUTES; the SID pointer references bytes in the
// GetTokenInformation-owned output buffer, not an independent allocation.
#[repr(C)]
struct SidAndAttributes {
    sid: *mut c_void,
    attributes: u32,
}
#[repr(C)]
struct TokenUserInfo {
    user: SidAndAttributes,
}

/// Read only the process TokenUser principal. No SID bytes or paths leave
/// this function, and the token is never modified or used to impersonate.
/// Check both OS output-buffer boundaries and SID validity before EqualSid.
fn token_user_matches(token: &OwnedToken, owner_sid: *mut c_void) -> io::Result<bool> {
    if owner_sid.is_null() || unsafe { IsValidSid(owner_sid) } == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Directory security owner SID is invalid",
        ));
    }
    let mut needed = 0u32;
    let first = unsafe {
        GetTokenInformation(token.0, TOKEN_USER_CLASS, ptr::null_mut(), 0, &mut needed)
    };
    if first != 0 || io::Error::last_os_error().raw_os_error()
        != Some(ERROR_INSUFFICIENT_BUFFER)
        || (needed as usize) < mem::size_of::<TokenUserInfo>()
        || (needed as usize) > MAX_TOKEN_USER_BYTES
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Unbounded or invalid Windows TokenUser preflight",
        ));
    }
    let capacity_words = (needed as usize + mem::size_of::<usize>() - 1)
        / mem::size_of::<usize>();
    let mut buffer = vec![0usize; capacity_words];
    let mut returned = 0u32;
    if unsafe {
        GetTokenInformation(
            token.0, TOKEN_USER_CLASS, buffer.as_mut_ptr().cast(),
            needed, &mut returned,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if returned < mem::size_of::<TokenUserInfo>() as u32 || returned > needed {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Windows TokenUser response changed size",
        ));
    }
    // SAFETY: usize buffer is correctly aligned for TOKEN_USER and large
    // enough for the returned fixed-size header, checked above.
    let user = unsafe { &*(buffer.as_ptr().cast::<TokenUserInfo>()) };
    let sid = user.user.sid;
    let start = buffer.as_ptr() as usize;
    let end = start + returned as usize;
    let pointer = sid as usize;
    // A SID has at least an 8-byte header. Bounds-check that header
    // BEFORE calling IsValidSid, which inspects it via a raw pointer.
    if sid.is_null() || pointer < start + mem::size_of::<TokenUserInfo>()
        || pointer.checked_add(8).is_none_or(|limit| limit > end)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Windows TokenUser SID escaped its buffer",
        ));
    }
    // SAFETY: Pointer bounds and the minimum header have been checked.
    if unsafe { IsValidSid(sid) } == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Windows TokenUser SID is invalid",
        ));
    }
    let size = unsafe { GetLengthSid(sid) } as usize;
    if size == 0 || pointer.checked_add(size).is_none_or(|limit| limit > end) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Windows TokenUser SID exceeds its buffer",
        ));
    }
    // SAFETY: Both SID pointers have been validated and are retained by
    // the token-information buffer and the directory descriptor.
    Ok(unsafe { EqualSid(sid, owner_sid) } != 0)
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
    let guard = OwnedDescriptor {
        allocation: descriptor,
        owner_sid: owner,
    };
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    // AccessCheck treats a NULL DACL as full access. It is NEVER acceptable
    // as a private root. Missing owner/group is also a hard NO-GO.
    if guard.allocation.is_null() || owner.is_null() || group.is_null()
        || dacl.is_null()
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Private root lacks owner, group or non-NULL DACL",
        ));
    }
    if unsafe { IsValidSid(owner) } == 0 || unsafe { IsValidSid(group) } == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Directory security owner/group SID is invalid",
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
    if !matches!(
        right,
        FILE_LIST_DIRECTORY | FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY
        | FILE_READ_EA | FILE_WRITE_EA | FILE_TRAVERSE | FILE_DELETE_CHILD
        | FILE_READ_ATTRIBUTES | FILE_WRITE_ATTRIBUTES
        | READ_CONTROL | DELETE | WRITE_DAC | WRITE_OWNER
    ) {
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
            descriptor.allocation, token.0, right, &mut mapping,
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

/// Sanitized evidence from a REAL OS directory security descriptor and
/// two process tokens. Booleans cannot attest the ancestor chain, service
/// identity, inherited file ACLs, token privileges or Minecraft game I/O.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct PairAccessFacts {
    pub directory_owner_matches_owner_user: bool,
    pub directory_owner_matches_peer_user: bool,
    pub owner_can_list: bool,
    pub owner_can_create: bool,
    pub peer_can_list: bool,
    pub peer_can_create: bool,
    pub peer_can_create_directory: bool,
    pub peer_can_read_ea: bool,
    pub peer_can_write_ea: bool,
    pub peer_can_traverse: bool,
    pub peer_can_read_attributes: bool,
    pub peer_can_write_attributes: bool,
    pub peer_can_read_security: bool,
    pub peer_can_delete_children: bool,
    pub peer_can_delete_directory: bool,
    pub peer_can_change_dacl: bool,
    pub peer_can_change_owner: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PairAccessReview {
    OwnerPrincipalUnverified,
    PeerSharesOwnerPrincipal,
    OwnerCannotUseRoot,
    PeerAccessObserved,
    // STILL NOT AUTHORIZATION: privileged peer access, race-free ACL
    // revalidation, all descendants, broker/WAM and game routing unproven.
    PeerDeniedCandidateOnly,
}

impl PairAccessFacts {
    pub fn review(self) -> PairAccessReview {
        if !self.directory_owner_matches_owner_user {
            PairAccessReview::OwnerPrincipalUnverified
        } else if self.directory_owner_matches_peer_user {
            // Two desktop clients under the same Windows user generally have
            // the SAME directory principal, even when PID/slot differ.
            PairAccessReview::PeerSharesOwnerPrincipal
        } else if !self.owner_can_list || !self.owner_can_create {
            PairAccessReview::OwnerCannotUseRoot
        } else if self.peer_can_list || self.peer_can_create
            || self.peer_can_create_directory || self.peer_can_traverse
            || self.peer_can_read_ea || self.peer_can_write_ea
            || self.peer_can_read_attributes || self.peer_can_write_attributes
            || self.peer_can_read_security
            || self.peer_can_delete_children || self.peer_can_delete_directory
            || self.peer_can_change_dacl || self.peer_can_change_owner
        {
            PairAccessReview::PeerAccessObserved
        } else {
            PairAccessReview::PeerDeniedCandidateOnly
        }
    }
}

/// Native Windows HANDLE-based ACL observer that can be reused by a
/// future reviewed root owner. The caller MUST pin both PROCESS HANDLEs,
/// verify full PID/creation FILETIME and liveness before AND after this
/// inspection, and independently attest the directory's ancestor chain.
/// Passing arbitrary open handles here never grants Windows access.
fn review_process_handles(
    root: &File, owner_handle: HANDLE, peer_handle: HANDLE,
) -> io::Result<PairAccessFacts> {
    let sd = read_security_descriptor(root)?;
    let owner_token = impersonation_token(owner_handle)?;
    let peer_token = impersonation_token(peer_handle)?;
    Ok(PairAccessFacts {
        directory_owner_matches_owner_user:
            token_user_matches(&owner_token, sd.owner_sid)?,
        directory_owner_matches_peer_user:
            token_user_matches(&peer_token, sd.owner_sid)?,
        owner_can_list: allowed(&sd, &owner_token, FILE_LIST_DIRECTORY)?,
        owner_can_create: allowed(&sd, &owner_token, FILE_ADD_FILE)?,
        peer_can_list: allowed(&sd, &peer_token, FILE_LIST_DIRECTORY)?,
        peer_can_create: allowed(&sd, &peer_token, FILE_ADD_FILE)?,
        peer_can_create_directory:
            allowed(&sd, &peer_token, FILE_ADD_SUBDIRECTORY)?,
        peer_can_read_ea: allowed(&sd, &peer_token, FILE_READ_EA)?,
        peer_can_write_ea: allowed(&sd, &peer_token, FILE_WRITE_EA)?,
        peer_can_traverse: allowed(&sd, &peer_token, FILE_TRAVERSE)?,
        peer_can_read_attributes:
            allowed(&sd, &peer_token, FILE_READ_ATTRIBUTES)?,
        peer_can_write_attributes:
            allowed(&sd, &peer_token, FILE_WRITE_ATTRIBUTES)?,
        peer_can_read_security: allowed(&sd, &peer_token, READ_CONTROL)?,
        peer_can_delete_children:
            allowed(&sd, &peer_token, FILE_DELETE_CHILD)?,
        peer_can_delete_directory: allowed(&sd, &peer_token, DELETE)?,
        peer_can_change_dacl: allowed(&sd, &peer_token, WRITE_DAC)?,
        peer_can_change_owner: allowed(&sd, &peer_token, WRITE_OWNER)?,
    })
}

/// Fixture integration only. Each exact process identity is rechecked before
/// and after the native HANDLE-based observation by the original Job owner.
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
            review_process_handles(root, owner_handle, peer_handle)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn isolated_candidate() -> PairAccessFacts {
        PairAccessFacts {
            directory_owner_matches_owner_user: true,
            directory_owner_matches_peer_user: false,
            owner_can_list: true, owner_can_create: true,
            peer_can_list: false, peer_can_create: false,
            peer_can_create_directory: false, peer_can_traverse: false,
            peer_can_read_ea: false, peer_can_write_ea: false,
            peer_can_read_attributes: false, peer_can_write_attributes: false,
            peer_can_read_security: false,
            peer_can_delete_children: false, peer_can_delete_directory: false,
            peer_can_change_dacl: false, peer_can_change_owner: false,
        }
    }

    #[test]
    fn candidate_requires_distinct_directory_principal_and_owner_rights() {
        let candidate = isolated_candidate();
        assert_eq!(candidate.review(), PairAccessReview::PeerDeniedCandidateOnly);
        assert_eq!(PairAccessFacts {
            directory_owner_matches_owner_user: false, ..candidate
        }.review(), PairAccessReview::OwnerPrincipalUnverified);
        assert_eq!(PairAccessFacts {
            directory_owner_matches_peer_user: true, ..candidate
        }.review(), PairAccessReview::PeerSharesOwnerPrincipal);
        assert_eq!(PairAccessFacts {
            owner_can_list: false, ..candidate
        }.review(), PairAccessReview::OwnerCannotUseRoot);
        assert_eq!(PairAccessFacts {
            owner_can_create: false, ..candidate
        }.review(), PairAccessReview::OwnerCannotUseRoot);
    }

    #[test]
    fn peer_mutation_and_traverse_rights_invalidate_denial_even_without_listing() {
        let candidate = isolated_candidate();
        for observed in [
            PairAccessFacts { peer_can_list: true, ..candidate },
            PairAccessFacts { peer_can_create: true, ..candidate },
            PairAccessFacts { peer_can_create_directory: true, ..candidate },
            PairAccessFacts { peer_can_read_ea: true, ..candidate },
            PairAccessFacts { peer_can_write_ea: true, ..candidate },
            PairAccessFacts { peer_can_traverse: true, ..candidate },
            PairAccessFacts { peer_can_read_attributes: true, ..candidate },
            PairAccessFacts { peer_can_write_attributes: true, ..candidate },
            PairAccessFacts { peer_can_read_security: true, ..candidate },
            PairAccessFacts { peer_can_delete_children: true, ..candidate },
            PairAccessFacts { peer_can_delete_directory: true, ..candidate },
            PairAccessFacts { peer_can_change_dacl: true, ..candidate },
            PairAccessFacts { peer_can_change_owner: true, ..candidate },
        ] {
            assert_eq!(observed.review(), PairAccessReview::PeerAccessObserved);
        }
    }
}
