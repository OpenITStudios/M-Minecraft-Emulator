//! Original Windows handle-relative private file I/O for MIVUBI.
//!
//! This is a real, bounded Windows file-store primitive for future engine
//! integration, NOT a transparent file-system virtualizer. It does not
//! intercept unmodified Minecraft Win32/Nt I/O, control HKCU/WAM, or prove
//! access denial to a foreign Windows token. No Sandboxie library is used.
//!
//! The only current callers are disposable Windows tests. A future broker
//! MUST authenticate the process identity and owner token, securely obtain
//! a trusted root handle and verify effective ACL/ancestor provenance before
//! opening account-bearing paths or admitting an isolated game launch.

#![allow(dead_code)]

use std::{
    ffi::c_void,
    fs::File,
    io::{self, Read, Write},
    mem,
    os::windows::io::{AsRawHandle, FromRawHandle},
    ptr,
};
#[cfg(test)]
use std::{
    fs,
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
#[cfg(test)]
use super::independent_job::{FixtureJob, SuspendedFixture};
#[cfg(test)]
use super::independent_root_access::{self, PairAccessFacts, PairAccessReview};
#[cfg(test)]
use virtual_clients_core::independent_engine::{
    plan_private_file_access, PendingPrivateFilePlan, ProcessIdentity,
    SessionSlot, StorageRouteAction, StorageRouteDomain,
    StorageRouteRequest, StorageRouteTarget,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    },
};

const FILE_READ_DATA: u32 = 0x0001;
const FILE_WRITE_DATA: u32 = 0x0002;
const FILE_ADD_FILE: u32 = 0x0002;
const FILE_ADD_SUBDIRECTORY: u32 = 0x0004;
const FILE_TRAVERSE: u32 = 0x0020;
const FILE_READ_ATTRIBUTES: u32 = 0x0080;
const SYNCHRONIZE: u32 = 0x0010_0000;
const READ_CONTROL: u32 = 0x0002_0000;
const FILE_ATTRIBUTE_NORMAL: u32 = 0x0080;
const FILE_SHARE_READ: u32 = 0x0001;
const FILE_SHARE_WRITE: u32 = 0x0002;
const FILE_CREATE: u32 = 2;
const FILE_OPEN: u32 = 1;
const FILE_CREATED: usize = 2;
const FILE_DIRECTORY_FILE: u32 = 0x0000_0001;
const FILE_NON_DIRECTORY_FILE: u32 = 0x0000_0040;
const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x0000_0020;
const FILE_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const OBJ_CASE_INSENSITIVE: u32 = 0x0000_0040;

/// The four native structs and symbols follow winternl.h; Rust's runtime
/// cannot reinterpret the returned FILE HANDLE as an arbitrary pathname.
#[repr(C)]
struct NtUnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

#[repr(C)]
struct NtObjectAttributes {
    length: u32,
    root_directory: HANDLE,
    object_name: *mut NtUnicodeString,
    attributes: u32,
    security_descriptor: *mut c_void,
    security_quality_of_service: *mut c_void,
}

#[repr(C)]
struct NtIoStatusBlock {
    status_or_pointer: usize,
    information: usize,
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtCreateFile(
        file_handle: *mut HANDLE,
        desired_access: u32,
        object_attributes: *mut NtObjectAttributes,
        io_status_block: *mut NtIoStatusBlock,
        allocation_size: *mut i64,
        file_attributes: u32,
        share_access: u32,
        create_disposition: u32,
        create_options: u32,
        ea_buffer: *mut c_void,
        ea_length: u32,
    ) -> i32;
}

/// No paths accepted across this boundary; a single relative name only.
/// Limiting the first adapter to 7-bit safe fixture names is deliberate.
/// Minecraft Unicode filenames need an independently reviewed extension.
fn validate_component(name: &str) -> io::Result<()> {
    if name.is_empty() || name.len() > 80
        || name == "." || name == ".."
        || name.ends_with('.') || name.ends_with(' ')
        || !name.is_ascii()
        || !name.bytes().all(|b|
            b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, "Untrusted relative file component",
        ));
    }
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    let dos_device = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL"
        | "CONIN$" | "CONOUT$")
        || (stem.len() == 4 &&
            (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit());
    if dos_device {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput, "Reserved Windows device basename",
        ));
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FileIdentity {
    volume: u32,
    index: u64,
}

fn inspect_file(handle: &File, directory: bool) -> io::Result<FileIdentity> {
    // SAFETY: The caller retains this File throughout GetFileInformationByHandle.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { mem::zeroed() };
    let valid = unsafe {
        GetFileInformationByHandle(handle.as_raw_handle() as HANDLE, &mut info)
    };
    if valid == 0 {
        return Err(io::Error::last_os_error());
    }
    let is_dir = info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY != 0;
    let reparse = info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    if reparse || is_dir != directory {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Unexpected reparse point or file/directory object",
        ));
    }
    // Hard links could alias a host file even without symlinks. Reject
    // multi-link regular files rather than trusting a private-looking path.
    if !directory && info.nNumberOfLinks != 1 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "File has multiple hard links or unverified link count",
        ));
    }
    let index = (u64::from(info.nFileIndexHigh) << 32)
        | u64::from(info.nFileIndexLow);
    if index == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Windows did not provide a usable file identity",
        ));
    }
    Ok(FileIdentity { volume: info.dwVolumeSerialNumber, index })
}

fn nt_error(status: i32) -> io::Error {
    let kind = match status as u32 {
        0xC000_0035 => io::ErrorKind::AlreadyExists, // STATUS_OBJECT_NAME_COLLISION
        0xC000_0034 | 0xC000_003A => io::ErrorKind::NotFound,
        0xC000_0022 | 0xC000_0043 | 0xC000_050B => io::ErrorKind::PermissionDenied,
        _ => io::ErrorKind::Other,
    };
    io::Error::new(kind, format!("NtCreateFile refused fixture file: 0x{:08X}", status as u32))
}

/// NtCreateFile relative to a STILL-OPEN directory HANDLE. The name is
/// exactly one checked component. Final reparse points are opened as links
/// and then rejected by inspect_file; no path from an untrusted caller
/// can traverse a parent component or alias a DOS device.
fn open_child(
    parent: &File, name: &str, directory: bool, create: bool,
) -> io::Result<File> {
    validate_component(name)?;
    let mut name_utf16: Vec<u16> = name.encode_utf16().collect();
    let length = (name_utf16.len() * mem::size_of::<u16>()) as u16;
    name_utf16.push(0);
    let mut unicode = NtUnicodeString {
        length, maximum_length: length + 2, buffer: name_utf16.as_mut_ptr(),
    };
    let mut attributes = NtObjectAttributes {
        length: mem::size_of::<NtObjectAttributes>() as u32,
        root_directory: parent.as_raw_handle() as HANDLE,
        object_name: &mut unicode,
        attributes: OBJ_CASE_INSENSITIVE,
        security_descriptor: ptr::null_mut(),
        security_quality_of_service: ptr::null_mut(),
    };
    let mut iosb = NtIoStatusBlock { status_or_pointer: 0, information: 0 };
    let mut raw: HANDLE = ptr::null_mut();
    let access = FILE_READ_ATTRIBUTES | SYNCHRONIZE |
        if directory {
            FILE_TRAVERSE | FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY | READ_CONTROL
        } else if create {
            FILE_READ_DATA | FILE_WRITE_DATA
        } else {
            FILE_READ_DATA
        };
    let flags = FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT |
        if directory { FILE_DIRECTORY_FILE } else { FILE_NON_DIRECTORY_FILE };
    // SAFETY: All native ABI structs and UTF-16 buffers are in scope.
    // NULL allocation/EAs; no named global object, inherited handle,
    // driver, service, profile, image or game process is changed.
    let status = unsafe {
        NtCreateFile(
            &mut raw, access, &mut attributes, &mut iosb, ptr::null_mut(),
            FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ | FILE_SHARE_WRITE,
            if create { FILE_CREATE } else { FILE_OPEN },
            flags, ptr::null_mut(), 0,
        )
    };
    if status != 0 || raw.is_null() ||
        (create && iosb.information != FILE_CREATED)
    {
        if !raw.is_null() {
            // SAFETY: This call alone owns the returned child HANDLE.
            unsafe { CloseHandle(raw) };
        }
        return Err(if status != 0 {
            nt_error(status)
        } else {
            io::Error::new(io::ErrorKind::InvalidData, "Unexpected native file creation result")
        });
    }
    // SAFETY: Successful NtCreateFile transfers unique handle ownership
    // to File; File::drop always closes it exactly once.
    let file = unsafe { File::from_raw_handle(raw as _) };
    inspect_file(&file, directory)?;
    Ok(file)
}

/// The root HANDLE must already be created and authenticated by a future
/// owner. At present it comes ONLY from fresh test temporary directories.
/// Checking a handle alone cannot attest profile SID, ancestor ACL or user.
#[cfg(test)]
struct FixtureFileRoot<'owner> {
    // Both handles are retained. The child must be newly created relative
    // to THIS parent, never adopted from an arbitrary path or root HANDLE.
    parent: File,
    parent_identity: FileIdentity,
    child_name: String,
    directory: File,
    identity: FileIdentity,
    owner: &'owner SuspendedFixture,
    assigned_slot: SessionSlot,
}

#[cfg(test)]
impl<'owner> FixtureFileRoot<'owner> {
    /// Fixture only: the caller-supplied parent HANDLE is not a trusted
    /// production root. No Minecraft profile or existing directory is adopted.
    fn create_new_under_parent(
        parent: File,
        child_name: &str,
        owner: &'owner SuspendedFixture,
        assigned_slot: SessionSlot,
    ) -> io::Result<Self> {
        if assigned_slot.is_host_native() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Host MC01 cannot own an isolated file root",
            ));
        }
        owner.live_identity()?;
        let parent_identity = inspect_file(&parent, true)?;
        // FILE_CREATE, not FILE_OPEN: a pre-existing name (including a
        // symlink/junction) is a collision, never an adopted capability.
        let directory = open_child(&parent, child_name, true, true)?;
        let identity = inspect_file(&directory, true)?;
        if identity.volume != parent_identity.volume {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Fresh fixture root crossed the parent volume",
            ));
        }
        let root = Self {
            parent, parent_identity, child_name: child_name.to_owned(),
            directory, identity, owner, assigned_slot,
        };
        root.verify()?;
        Ok(root)
    }

    /// The other test process can intentionally borrow the SAME root to
    /// demonstrate why slot labels and file handles alone do not isolate
    /// same-user Windows tokens. NEVER expose this as a production factory.
    fn clone_for_fixture<'peer>(
        &self, owner: &'peer SuspendedFixture, assigned_slot: SessionSlot,
    ) -> io::Result<FixtureFileRoot<'peer>> {
        self.verify()?;
        if assigned_slot.is_host_native() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Host MC01 cannot borrow an isolated file root",
            ));
        }
        owner.live_identity()?;
        let borrowed = FixtureFileRoot {
            parent: self.parent.try_clone()?,
            parent_identity: self.parent_identity,
            child_name: self.child_name.clone(),
            directory: self.directory.try_clone()?,
            identity: self.identity,
            owner, assigned_slot,
        };
        borrowed.verify()?;
        Ok(borrowed)
    }

    fn verify(&self) -> io::Result<()> {
        self.owner.live_identity()?;
        if inspect_file(&self.parent, true)? != self.parent_identity
            || inspect_file(&self.directory, true)? != self.identity
            || self.identity.volume != self.parent_identity.volume
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Fixture parent/root handle identity changed",
            ));
        }
        // The relative name must STILL resolve to the exact retained root
        // object. A filename/path comparison could not detect substitution.
        let reopened = open_child(&self.parent, &self.child_name, true, false)?;
        if inspect_file(&reopened, true)? != self.identity
            || inspect_file(&self.parent, true)? != self.parent_identity
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Fixture parent-child HANDLE lineage changed",
            ));
        }
        self.owner.live_identity()?;
        Ok(())
    }

    /// Read the REAL Windows DACL against the live tokens of two owned
    /// fixtures. A positive peer-denial candidate is NOT a launch grant.
    fn assess_peer_acl(&self, peer: &SuspendedFixture) -> io::Result<PairAccessFacts> {
        self.verify()?;
        let facts = independent_root_access::review_fixture_pair(
            &self.directory, self.owner, peer,
        )?;
        self.verify()?;
        Ok(facts)
    }

    fn open_slot(&self, slot: SessionSlot, create: bool) -> io::Result<SlotFiles<'_, 'owner>> {
        self.verify()?;
        if slot != self.assigned_slot {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Fixture owner is bound to a different slot",
            ));
        }
        let name = match slot {
            SessionSlot::Mc01 => return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Host MC01 may not own an isolated fixture root",
            )),
            SessionSlot::Mc02 => "mc02",
            SessionSlot::Mc03 => "mc03",
            SessionSlot::Mc04 => "mc04",
        };
        let directory = open_child(&self.directory, name, true, create)?;
        let identity = inspect_file(&directory, true)?;
        if identity.volume != self.identity.volume {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Slot directory crossed Windows volume",
            ));
        }
        self.verify()?;
        Ok(SlotFiles { root: self, directory, identity, child_name: name })
    }
}

#[cfg(test)]
struct SlotFiles<'root, 'owner> {
    root: &'root FixtureFileRoot<'owner>,
    directory: File,
    identity: FileIdentity,
    // Fixed canonical slot name, not supplied by a caller/plan. It must
    // still resolve to the exact original opened child of the root HANDLE.
    child_name: &'static str,
}

#[cfg(test)]
impl SlotFiles<'_, '_> {
    fn verify(&self) -> io::Result<()> {
        self.root.verify()?;
        if inspect_file(&self.directory, true)? != self.identity
            || self.identity.volume != self.root.identity.volume
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Slot directory identity changed",
            ));
        }
        // Holding the directory HANDLE is insufficient: the fixed slot name
        // must still resolve to that SAME file object beneath the retained
        // parent/root HANDLE. Do not use a DOS path or adopt a replacement.
        let reopened = open_child(
            &self.root.directory, self.child_name, true, false,
        )?;
        if inspect_file(&reopened, true)? != self.identity
            || inspect_file(&self.directory, true)? != self.identity
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "Slot directory parent-child HANDLE lineage changed",
            ));
        }
        self.root.verify()?;
        Ok(())
    }

    // Reuse the SINGLE canonical runtime-core storage route planner. An
    // adapter-owned request is derived from the actual fixture process,
    // retained slot and fixed LocalFiles domain. Neither a user-selected
    // path nor an untrusted PendingPrivateFilePlan can authorize Windows I/O.
    fn marker_request(&self, action: StorageRouteAction)
        -> io::Result<StorageRouteRequest>
    {
        self.verify()?;
        let process: ProcessIdentity = self.root.owner.live_identity()?;
        Ok(StorageRouteRequest {
            slot: self.root.assigned_slot,
            expected_process: process,
            observed_process: Some(process),
            domain: StorageRouteDomain::LocalFiles,
            target: StorageRouteTarget::PrivateSession(self.root.assigned_slot),
            action,
        })
    }

    fn marker_plan(
        &self, name: &str, action: StorageRouteAction,
    ) -> io::Result<PendingPrivateFilePlan> {
        let request = self.marker_request(action)?;
        let plan = plan_private_file_access(request, name)
            .map_err(|_| io::Error::new(
                io::ErrorKind::InvalidInput, "Unsafe synthetic file route or relative name",
            ))?;
        // This native fixture has only ONE retained slot-level directory;
        // the Rust planner may support nested components in future owners,
        // but it would be unsafe to silently discard any component here.
        if plan.components.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput, "Nested file plans are not supported by this fixture",
            ));
        }
        validate_component(&plan.components[0])?;
        Ok(plan)
    }

    fn checked_marker_name<'plan>(
        &self, plan: &'plan PendingPrivateFilePlan,
        action: StorageRouteAction,
    ) -> io::Result<&'plan str> {
        let request = self.marker_request(action)?;
        // Plans carry only diagnostic metadata and all their fields are
        // publicly constructible. Re-derive using trusted OS fixture facts;
        // wrong owner/slot, action, domain, or altered components fail closed.
        if plan.components.len() != 1
            || plan.slot != request.slot
            || plan.process != request.expected_process
            || plan.domain != request.domain
            || plan.action != action
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Unbound or unsupported synthetic file plan",
            ));
        }
        let name = plan.components[0].as_str();
        let canonical = self.marker_plan(name, action)?;
        if &canonical != plan {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "File plan changed after validation",
            ));
        }
        self.verify()?;
        Ok(name)
    }

    /// Actual OS-backed file create under a process/slot-bound, checked
    /// synthetic plan. Still NOT private access for an unmodified game:
    /// root ancestors/ACLs and peer denial are not service-attested.
    /// There is no host read, overwrite or cleanup-by-path rollback.
    fn create_planned_canary(
        &self, plan: &PendingPrivateFilePlan, marker: u64,
    ) -> io::Result<()> {
        let name = self.checked_marker_name(plan, StorageRouteAction::Create)?;
        let mut file = open_child(&self.directory, name, false, true)?;
        if inspect_file(&file, false)?.volume != self.identity.volume {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "File crossed Windows volume",
            ));
        }
        file.write_all(&marker.to_le_bytes())?;
        file.sync_all()?;
        self.verify()?;
        Ok(())
    }

    fn read_planned_canary(&self, plan: &PendingPrivateFilePlan) -> io::Result<u64> {
        let name = self.checked_marker_name(plan, StorageRouteAction::Read)?;
        let mut file = open_child(&self.directory, name, false, false)?;
        if inspect_file(&file, false)?.volume != self.identity.volume {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "File crossed Windows volume",
            ));
        }
        let mut bytes = [0u8; 8];
        file.read_exact(&mut bytes)?;
        let mut extra = [0u8; 1];
        if file.read(&mut extra)? != 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, "Unexpected data after bounded fixture marker",
            ));
        }
        self.verify()?;
        Ok(u64::from_le_bytes(bytes))
    }

    // Compatibility wrappers for existing disposable native Windows tests.
    // They MUST route via the same canonical plan, not a second path parser.
    fn create_canary(&self, name: &str, marker: u64) -> io::Result<()> {
        let plan = self.marker_plan(name, StorageRouteAction::Create)?;
        self.create_planned_canary(&plan, marker)
    }

    fn read_canary(&self, name: &str) -> io::Result<u64> {
        let plan = self.marker_plan(name, StorageRouteAction::Read)?;
        self.read_planned_canary(&plan)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disposable_root<'owner>(
        owner: &'owner SuspendedFixture,
        slot: SessionSlot,
    ) -> (PathBuf, FixtureFileRoot<'owner>) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let when = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap().as_nanos();
        let child_name = format!(
            "mivubi-native-files-{}-{when}-{}",
            std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed),
        );
        let parent_path = std::env::temp_dir();
        let path = parent_path.join(&child_name);
        // Only the test uses the host temp directory. Its ancestors and
        // inherited ACL are NOT a trusted future Minecraft storage root.
        let parent = fs::OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES | FILE_TRAVERSE |
                FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY | SYNCHRONIZE | READ_CONTROL)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(0x0200_0000 | 0x0020_0000)
            // FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT
            .open(&parent_path).unwrap();
        let root = FixtureFileRoot::create_new_under_parent(
            parent, &child_name, owner, slot,
        ).unwrap();
        (path, root)
    }

    fn cleanup(path: &Path) {
        for dir in ["mc02", "mc03", "mc04"] {
            // Fixed, test-created names only. No recursive account/world delete.
            let _ = fs::remove_file(path.join(dir).join("marker.bin"));
            let _ = fs::remove_file(path.join(dir).join("already.bin"));
            let _ = fs::remove_dir(path.join(dir));
        }
        let _ = fs::remove_dir(path);
    }

    #[test]
    fn real_native_file_handles_require_distinct_owned_process_capabilities() {
        let owned_a = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        let owned_b = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        assert_ne!(owned_a.live_identity().unwrap().pid, owned_b.live_identity().unwrap().pid);

        let (path, root_a) = disposable_root(&owned_a, SessionSlot::Mc02);
        // Same-user Windows process tokens normally inherit the SAME
        // directory rights. This diagnostic must never report isolation
        // merely because Rust assigned different process/slot identities.
        let facts = root_a.assess_peer_acl(&owned_b).unwrap();
        assert_eq!(facts.owner_can_list, facts.peer_can_list);
        assert_eq!(facts.owner_can_create, facts.peer_can_create);
        assert_ne!(facts.review(), PairAccessReview::PeerDeniedCandidateOnly);
        // A second owner borrows the SAME opened root handle. Its slot
        // cannot be impersonated by simply passing a different slot ID.
        let root_b = root_a.clone_for_fixture(&owned_b, SessionSlot::Mc03).unwrap();
        let first = root_a.open_slot(SessionSlot::Mc02, true).unwrap();
        let second = root_b.open_slot(SessionSlot::Mc03, true).unwrap();
        first.create_canary("marker.bin", 0xAABB_0002).unwrap();
        assert_eq!(
            second.read_canary("marker.bin").unwrap_err().kind(),
            io::ErrorKind::NotFound,
        );
        second.create_canary("marker.bin", 0xAABB_0003).unwrap();
        assert_eq!(first.read_canary("marker.bin").unwrap(), 0xAABB_0002);
        assert_eq!(second.read_canary("marker.bin").unwrap(), 0xAABB_0003);
        assert_eq!(
            root_a.open_slot(SessionSlot::Mc03, false).err().unwrap().kind(),
            io::ErrorKind::PermissionDenied,
        );
        assert_eq!(
            root_b.open_slot(SessionSlot::Mc02, false).err().unwrap().kind(),
            io::ErrorKind::PermissionDenied,
        );
        drop(first);
        drop(second);
        let reopened = root_a.open_slot(SessionSlot::Mc02, false).unwrap();
        assert_eq!(reopened.read_canary("marker.bin").unwrap(), 0xAABB_0002);
        drop(reopened);
        drop(root_b);
        drop(root_a);
        // Releasing the borrowed capability permits a normal fixture exit.
        owned_a.resume().unwrap().wait(Duration::from_secs(20)).unwrap();
        owned_b.resume().unwrap().wait(Duration::from_secs(20)).unwrap();
        cleanup(&path);
    }

    #[test]
    fn parent_bound_root_refuses_existing_child_and_rechecks_handle_lineage() {
        let owned = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        let (path, mut root) = disposable_root(&owned, SessionSlot::Mc02);
        // A fresh-only factory must NOT adopt an existing root, even when
        // the caller has a real parent HANDLE and matching process.
        assert_eq!(
            FixtureFileRoot::create_new_under_parent(
                root.parent.try_clone().unwrap(), &root.child_name,
                &owned, SessionSlot::Mc02,
            ).err().unwrap().kind(),
            io::ErrorKind::AlreadyExists,
        );
        let original_parent = root.parent_identity;
        root.parent_identity.index ^= 1;
        assert_eq!(
            root.open_slot(SessionSlot::Mc02, true).err().unwrap().kind(),
            io::ErrorKind::PermissionDenied,
        );
        root.parent_identity = original_parent;
        let original_name = std::mem::replace(
            &mut root.child_name, "absent-fixture-root".to_owned(),
        );
        assert_eq!(
            root.open_slot(SessionSlot::Mc02, false).err().unwrap().kind(),
            io::ErrorKind::NotFound,
        );
        root.child_name = original_name;
        let slot = root.open_slot(SessionSlot::Mc02, true).unwrap();
        slot.create_canary("marker.bin", 37).unwrap();
        assert_eq!(slot.read_canary("marker.bin").unwrap(), 37);
        drop(slot);
        drop(root);
        owned.resume().unwrap().wait(Duration::from_secs(20)).unwrap();
        cleanup(&path);
    }

    #[test]
    fn planned_native_markers_bind_original_slot_process_action_and_domain() {
        let owner = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        let (path, root) = disposable_root(&owner, SessionSlot::Mc02);
        let slot = root.open_slot(SessionSlot::Mc02, true).unwrap();
        let create = slot.marker_plan("marker.bin", StorageRouteAction::Create).unwrap();
        let read = slot.marker_plan("marker.bin", StorageRouteAction::Read).unwrap();
        assert_eq!(create.components, vec!["marker.bin".to_owned()]);
        assert_eq!(create.slot, SessionSlot::Mc02);
        assert_eq!(create.process, owner.live_identity().unwrap());
        assert_eq!(create.domain, StorageRouteDomain::LocalFiles);
        assert_eq!(create.action, StorageRouteAction::Create);
        slot.create_planned_canary(&create, 0xAABB_02).unwrap();
        assert_eq!(slot.read_planned_canary(&read).unwrap(), 0xAABB_02);

        // A pending plan is NOT an access ticket: mutate each publicly
        // constructible field and verify the native adapter fails closed.
        let mut wrong = create.clone();
        wrong.slot = SessionSlot::Mc03;
        assert_eq!(slot.create_planned_canary(&wrong, 1).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied);
        wrong = create.clone();
        wrong.process.created_file_time += 1;
        assert_eq!(slot.create_planned_canary(&wrong, 1).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied);
        wrong = create.clone();
        wrong.domain = StorageRouteDomain::RoamingFiles;
        assert_eq!(slot.create_planned_canary(&wrong, 1).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied);
        wrong = create.clone();
        wrong.action = StorageRouteAction::Read;
        assert_eq!(slot.create_planned_canary(&wrong, 1).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied);
        wrong = create.clone();
        wrong.components.push("child.bin".to_owned());
        assert_eq!(slot.create_planned_canary(&wrong, 1).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied);
        wrong = create.clone();
        wrong.components[0] = "../host.bin".to_owned();
        assert_eq!(slot.create_planned_canary(&wrong, 1).unwrap_err().kind(),
            io::ErrorKind::InvalidInput);
        assert_eq!(slot.read_planned_canary(&read).unwrap(), 0xAABB_02);
        assert_eq!(slot.marker_plan("folder/marker.bin", StorageRouteAction::Read)
            .unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(slot.marker_plan("CON.txt", StorageRouteAction::Create)
            .unwrap_err().kind(), io::ErrorKind::InvalidInput);

        drop(slot);
        drop(root);
        owner.resume().unwrap().wait(Duration::from_secs(20)).unwrap();
        cleanup(&path);
    }

    #[test]
    fn slot_handle_lineage_check_rejects_unrelated_child_or_changed_identity() {
        let owner = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        let (path, root) = disposable_root(&owner, SessionSlot::Mc02);
        let mut slot = root.open_slot(SessionSlot::Mc02, true).unwrap();
        let original_name = slot.child_name;
        slot.child_name = "mc03";
        assert_eq!(slot.create_canary("marker.bin", 5).unwrap_err().kind(),
            io::ErrorKind::NotFound);
        slot.child_name = original_name;
        let original_identity = slot.identity;
        slot.identity.index ^= 1;
        assert_eq!(slot.create_canary("marker.bin", 5).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied);
        slot.identity = original_identity;
        slot.create_canary("marker.bin", 11).unwrap();
        assert_eq!(slot.read_canary("marker.bin").unwrap(), 11);
        drop(slot);
        drop(root);
        owner.resume().unwrap().wait(Duration::from_secs(20)).unwrap();
        cleanup(&path);
    }

    #[test]
    fn host_slot_collisions_and_untrusted_relative_names_are_refused() {
        let owned = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        let (path, root) = disposable_root(&owned, SessionSlot::Mc04);
        assert_eq!(root.open_slot(SessionSlot::Mc01, true).err().unwrap().kind(),
            io::ErrorKind::PermissionDenied);
        let slot = root.open_slot(SessionSlot::Mc04, true).unwrap();
        for bad in ["", "..", ".", "../secret", "C:bad", "x\\y", "x/y",
                    "CON", "NUL.txt", "COM1", "bad.", "bad ", "α"]
        {
            assert_eq!(slot.create_canary(bad, 7).unwrap_err().kind(),
                io::ErrorKind::InvalidInput);
        }
        slot.create_canary("already.bin", 1).unwrap();
        assert_eq!(slot.create_canary("already.bin", 2).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists);
        assert_eq!(slot.read_canary("already.bin").unwrap(), 1);
        drop(slot);
        drop(root);
        owned.resume().unwrap().wait(Duration::from_secs(20)).unwrap();
        cleanup(&path);
    }

    #[test]
    fn effective_acl_review_rejects_same_process_and_exited_peer() {
        let owned = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        let peer = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        let (path, root) = disposable_root(&owned, SessionSlot::Mc02);
        assert_eq!(
            root.assess_peer_acl(&owned).unwrap_err().kind(),
            io::ErrorKind::InvalidInput,
        );
        // Explicitly terminate only the peer fixture; stale process identity
        // must fail before accepting any AccessCheck result.
        peer.terminate_for_test().unwrap();
        assert_eq!(
            root.assess_peer_acl(&peer).unwrap_err().kind(),
            io::ErrorKind::NotFound,
        );
        drop(root);
        drop(peer);
        owned.resume().unwrap().wait(Duration::from_secs(20)).unwrap();
        cleanup(&path);
    }

    #[test]
    fn exited_engine_owned_fixture_cannot_use_held_private_file_root() {
        let owned = FixtureJob::create().unwrap().spawn_self_test().unwrap();
        let (path, root) = disposable_root(&owned, SessionSlot::Mc02);
        let slot = root.open_slot(SessionSlot::Mc02, true).unwrap();
        slot.create_canary("marker.bin", 11).unwrap();
        owned.terminate_for_test().unwrap();
        assert_eq!(
            slot.read_canary("marker.bin").unwrap_err().kind(),
            io::ErrorKind::NotFound,
        );
        assert_eq!(
            slot.create_canary("already.bin", 22).unwrap_err().kind(),
            io::ErrorKind::NotFound,
        );
        drop(slot);
        drop(root);
        drop(owned);
        cleanup(&path);
    }
}
