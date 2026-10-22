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
use std::{fs, os::windows::fs::OpenOptionsExt, path::{Path, PathBuf}, sync::atomic::{AtomicU64, Ordering}, time::{SystemTime, UNIX_EPOCH}};
use virtual_clients_core::independent_engine::SessionSlot;
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
            FILE_TRAVERSE | FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY
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
struct FixtureFileRoot {
    directory: File,
    identity: FileIdentity,
}

impl FixtureFileRoot {
    fn from_open_directory(directory: File) -> io::Result<Self> {
        let identity = inspect_file(&directory, true)?;
        Ok(Self { directory, identity })
    }

    fn verify(&self) -> io::Result<()> {
        if inspect_file(&self.directory, true)? != self.identity {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Root directory identity changed",
            ));
        }
        Ok(())
    }

    fn open_slot(&self, slot: SessionSlot, create: bool) -> io::Result<SlotFiles<'_>> {
        self.verify()?;
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
        Ok(SlotFiles { root: self, directory, identity })
    }
}

struct SlotFiles<'a> {
    root: &'a FixtureFileRoot,
    directory: File,
    identity: FileIdentity,
}

impl SlotFiles<'_> {
    fn verify(&self) -> io::Result<()> {
        self.root.verify()?;
        if inspect_file(&self.directory, true)? != self.identity ||
            self.identity.volume != self.root.identity.volume {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Slot directory identity changed",
            ));
        }
        Ok(())
    }

    /// Actual OS-backed, private-slot file create. No implicit host read or
    /// fallback, overwrite, truncate, symlink traversal, or arbitrarily large
    /// data. Partial-write failure leaves an owned object for manual review;
    /// there is deliberately no unsafe delete/rollback by path.
    fn create_canary(&self, name: &str, marker: u64) -> io::Result<()> {
        self.verify()?;
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

    fn read_canary(&self, name: &str) -> io::Result<u64> {
        self.verify()?;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disposable_root() -> (PathBuf, FixtureFileRoot) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let when = SystemTime::now().duration_since(UNIX_EPOCH)
            .unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!(
            "mivubi-native-files-{}-{when}-{}",
            std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        // Create new only, never adopt an existing profile/account root.
        fs::create_dir(&path).unwrap();
        let directory = fs::OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES | FILE_TRAVERSE |
                FILE_ADD_FILE | FILE_ADD_SUBDIRECTORY | SYNCHRONIZE)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(0x0200_0000 | 0x0020_0000)
            // FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT
            .open(&path).unwrap();
        (path, FixtureFileRoot::from_open_directory(directory).unwrap())
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
    fn real_native_file_handles_keep_two_slots_independent_and_persistent() {
        let (path, root) = disposable_root();
        let first = root.open_slot(SessionSlot::Mc02, true).unwrap();
        let second = root.open_slot(SessionSlot::Mc03, true).unwrap();
        first.create_canary("marker.bin", 0xAABB_0002).unwrap();
        assert_eq!(second.read_canary("marker.bin").unwrap_err().kind(),
            io::ErrorKind::NotFound);
        second.create_canary("marker.bin", 0xAABB_0003).unwrap();
        assert_eq!(first.read_canary("marker.bin").unwrap(), 0xAABB_0002);
        assert_eq!(second.read_canary("marker.bin").unwrap(), 0xAABB_0003);
        drop(first);
        drop(second);
        let reopened = root.open_slot(SessionSlot::Mc02, false).unwrap();
        assert_eq!(reopened.read_canary("marker.bin").unwrap(), 0xAABB_0002);
        drop(reopened);
        drop(root);
        cleanup(&path);
    }

    #[test]
    fn host_slot_collisions_and_untrusted_relative_names_are_refused() {
        let (path, root) = disposable_root();
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
        cleanup(&path);
    }
}
