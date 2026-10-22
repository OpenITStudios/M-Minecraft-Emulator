//! Original Windows registry application-hive and private-root primitives.
//!
//! Real OS I/O, NOT a Minecraft sandbox: RegLoadAppKeyW exposes only a
//! handle-scoped registry view; it does NOT transparently redirect HKCU.
//! The only callers are unmounted Windows fixture tests. No Microsoft account
//! hive, credential, token, game file or Sandboxie component is accessed.
//!
//! We deliberately refuse arbitrary hives and pre-existing unowned roots.
//! A production broker requires authenticated per-process/SID ownership,
//! durable lifecycle, collision-free private ACLs, and interception proof.

#![allow(dead_code)]

use std::{
    fs::{self, File, OpenOptions},
    io,
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::AsRawHandle,
    },
    path::{Path, PathBuf},
    ptr,
};
#[cfg(test)]
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use virtual_clients_core::independent_engine::SessionSlot;
use windows_sys::Win32::{
    Foundation::{ERROR_FILE_NOT_FOUND, HANDLE},
    Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
        FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_SHARE_READ, FILE_SHARE_WRITE,
    },
    System::Registry::{
        HKEY, KEY_READ, KEY_WRITE, REG_BINARY, REG_PROCESS_APPKEY,
        RegCloseKey, RegFlushKey, RegLoadAppKeyW,
        RegQueryValueExW, RegSetValueExW,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DirectoryIdentity {
    volume: u32,
    index: u64,
}

fn pinned_directory_identity(opened: &File) -> io::Result<DirectoryIdentity> {
    // SAFETY: The File holds an open Win32 HANDLE and output is a C POD.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe {
        GetFileInformationByHandle(opened.as_raw_handle() as HANDLE, &mut info)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Private fixture root must be an ordinary directory, never a reparse point",
        ));
    }
    let index = (u64::from(info.nFileIndexHigh) << 32)
        | u64::from(info.nFileIndexLow);
    if index == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Directory FILE_ID is unavailable",
        ));
    }
    Ok(DirectoryIdentity { volume: info.dwVolumeSerialNumber, index })
}

/// An opened directory capability, deliberately limited to fresh test data.
/// Parent directory chains and final target ACLs have NOT been attested:
/// this is not a trusted per-Minecraft profile root.
struct OwnedFixtureRoot {
    directory: File,
    path: PathBuf,
    identity: DirectoryIdentity,
}

impl OwnedFixtureRoot {
    fn inspect_existing(path: &Path) -> io::Result<Self> {
        if !path.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput, "Expected an absolute test directory",
            ));
        }
        // FILE_FLAG_OPEN_REPARSE_POINT opens the final link itself, not its
        // target. Holding the directory without SHARE_DELETE helps prevent
        // rename/swap while the registry hive path is being used.
        let directory = OpenOptions::new()
            .access_mode(0x80 | 0x0010_0000) // FILE_READ_ATTRIBUTES | SYNCHRONIZE
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)?;
        let identity = pinned_directory_identity(&directory)?;
        Ok(Self { directory, path: path.to_path_buf(), identity })
    }

    fn verify_stable(&self) -> io::Result<()> {
        if pinned_directory_identity(&self.directory)? != self.identity {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Opened directory identity changed",
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    fn create_fresh_test() -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?.as_nanos();
        let test_name = format!(
            "mivubi-fixture-hive-{}-{now}-{}",
            std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
        );
        // Never reuse, clear or walk account-bearing paths. If an unexpected
        // directory already exists, create_dir fails without any mutation.
        let path = std::env::temp_dir().join(test_name);
        fs::create_dir(&path)?;
        Self::inspect_existing(&path)
    }

    fn fixed_hive_path(&self, slot: SessionSlot) -> io::Result<PathBuf> {
        self.verify_stable()?;
        let name = match slot {
            SessionSlot::Mc01 => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Native MC01 cannot receive a private fixture hive",
                ));
            }
            SessionSlot::Mc02 => "fixture-02.hiv",
            SessionSlot::Mc03 => "fixture-03.hiv",
            SessionSlot::Mc04 => "fixture-04.hiv",
        };
        // No user-defined path component, no .., no alternate data stream.
        Ok(self.path.join(name))
    }
}

enum HiveOpen {
    Fresh,
    Reopen,
}

/// The only lookup key is an OS-owned HKEY obtained by RegLoadAppKeyW.
/// No registry operation uses HKEY_CURRENT_USER or HKEY_USERS.
struct NativeAppHive<'a> {
    root: &'a OwnedFixtureRoot,
    raw: HKEY,
}

impl NativeAppHive<'_> {
    fn open<'a>(
        root: &'a OwnedFixtureRoot,
        slot: SessionSlot,
        mode: HiveOpen,
    ) -> io::Result<NativeAppHive<'a>> {
        let path = root.fixed_hive_path(slot)?;
        // This check guards the disposable fixture from accidently opening
        // arbitrary preexisting files. A production owner must additionally
        // use handle-relative file creation and trusted ACL/SID admission.
        match (fs::symlink_metadata(&path), mode) {
            (Err(err), HiveOpen::Fresh) if err.kind() == io::ErrorKind::NotFound => {}
            (Ok(_), HiveOpen::Fresh) => return Err(io::Error::new(
                io::ErrorKind::AlreadyExists, "Refusing pre-existing hive file",
            )),
            (Ok(meta), HiveOpen::Reopen) if meta.file_type().is_file() => {
                use std::os::windows::fs::MetadataExt;
                if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                    return Err(io::Error::new(
                        io::ErrorKind::PermissionDenied, "Hive file is a reparse point",
                    ));
                }
            }
            (Err(err), _) => return Err(err),
            _ => return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Unexpected hive file object",
            )),
        }
        let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
        if wide.len() > 30_000 || wide.is_empty() || wide.contains(&0) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid hive path"));
        }
        wide.push(0);
        let mut key: HKEY = ptr::null_mut();
        // SAFETY: Freshly bounded UTF-16 buffer, live HKEY output; the call
        // does not mount a key at HKCU/HKU. REG_PROCESS_APPKEY keeps the
        // test hive scoped to this process for its loaded lifetime.
        let status = unsafe {
            RegLoadAppKeyW(
                wide.as_ptr(), &mut key,
                KEY_READ | KEY_WRITE, REG_PROCESS_APPKEY, 0,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        if key.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, "RegLoadAppKeyW returned no hive handle",
            ));
        }
        let hive = NativeAppHive { root, raw: key };
        hive.root.verify_stable()?; // Drop closes HKEY if verification fails.
        Ok(hive)
    }

    // Only a nonsecret fixed eight-byte proof value is allowed in this
    // sample. No arbitrary registry paths, strings or credentials.
    fn write_canary(&self, value: u64) -> io::Result<()> {
        self.root.verify_stable()?;
        let label: Vec<u16> = "MivubiNativeFixture".encode_utf16()
            .chain(std::iter::once(0)).collect();
        let bytes = value.to_le_bytes();
        // SAFETY: Known value name and 8-byte in-scope buffer; the OS
        // writes ONLY to this HKEY's application hive.
        let status = unsafe {
            RegSetValueExW(
                self.raw, label.as_ptr(), 0, REG_BINARY,
                bytes.as_ptr(), bytes.len() as u32,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        self.root.verify_stable()?;
        Ok(())
    }

    fn read_canary(&self) -> io::Result<Option<u64>> {
        self.root.verify_stable()?;
        let label: Vec<u16> = "MivubiNativeFixture".encode_utf16()
            .chain(std::iter::once(0)).collect();
        let mut bytes = [0u8; 8];
        let mut size = bytes.len() as u32;
        let mut kind = 0;
        // SAFETY: HKEY is retained; all output pointers remain valid and
        // length is capped to exactly eight bytes by the input capacity.
        let status = unsafe {
            RegQueryValueExW(
                self.raw, label.as_ptr(), ptr::null(),
                &mut kind, bytes.as_mut_ptr(), &mut size,
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(None);
        }
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        if kind != REG_BINARY || size != 8 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, "Unexpected fixture hive value type or size",
            ));
        }
        self.root.verify_stable()?;
        Ok(Some(u64::from_le_bytes(bytes)))
    }

    fn flush(&self) -> io::Result<()> {
        // SAFETY: HKEY is still retained; OS flushes only this hive.
        let status = unsafe { RegFlushKey(self.raw) };
        if status != 0 {
            Err(io::Error::from_raw_os_error(status as i32))
        } else {
            Ok(())
        }
    }
}

impl Drop for NativeAppHive<'_> {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            // SAFETY: HKEY is uniquely owned and must close exactly once.
            unsafe { RegCloseKey(self.raw) };
        }
    }
}

#[cfg(test)]
mod windows_tests {
    use super::*;

    /// Best-effort cleanup of only THREE hardcoded disposable fixture files.
    /// No recursive directory deletion, symlink following or account paths.
    fn cleanup_fixture(path: &Path) {
        for name in ["fixture-02.hiv", "fixture-03.hiv", "fixture-04.hiv"] {
            let _ = fs::remove_file(path.join(name));
        }
        let _ = fs::remove_dir(path);
    }

    #[test]
    fn two_real_application_hives_have_separate_values_and_persist() {
        let root = OwnedFixtureRoot::create_fresh_test().unwrap();
        let path = root.path.clone();
        let first = NativeAppHive::open(&root, SessionSlot::Mc02, HiveOpen::Fresh).unwrap();
        let second = NativeAppHive::open(&root, SessionSlot::Mc03, HiveOpen::Fresh).unwrap();

        assert_eq!(first.read_canary().unwrap(), None);
        assert_eq!(second.read_canary().unwrap(), None);
        first.write_canary(0x1234_0002).unwrap();
        assert_eq!(second.read_canary().unwrap(), None);
        second.write_canary(0x1234_0003).unwrap();
        assert_eq!(first.read_canary().unwrap(), Some(0x1234_0002));
        assert_eq!(second.read_canary().unwrap(), Some(0x1234_0003));
        first.flush().unwrap();
        second.flush().unwrap();
        drop(first);
        drop(second);

        // Windows unloads application hives after the last HKEY is closed.
        let reopened = NativeAppHive::open(&root, SessionSlot::Mc02, HiveOpen::Reopen).unwrap();
        assert_eq!(reopened.read_canary().unwrap(), Some(0x1234_0002));
        drop(reopened);
        drop(root);
        cleanup_fixture(&path);
    }

    #[test]
    fn native_host_slot_and_preexisting_hive_are_rejected() {
        let root = OwnedFixtureRoot::create_fresh_test().unwrap();
        let path = root.path.clone();
        assert_eq!(
            NativeAppHive::open(&root, SessionSlot::Mc01, HiveOpen::Fresh)
                .err().unwrap().kind(),
            io::ErrorKind::PermissionDenied
        );
        // Reject a preexisting file rather than load/overwrite potentially
        // account-bearing data. This is a new disposable test root only.
        let occupied = root.fixed_hive_path(SessionSlot::Mc04).unwrap();
        fs::write(&occupied, b"not-a-hive").unwrap();
        assert_eq!(
            NativeAppHive::open(&root, SessionSlot::Mc04, HiveOpen::Fresh)
                .err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
        assert_eq!(fs::read(&occupied).unwrap(), b"not-a-hive");
        drop(root);
        cleanup_fixture(&path);
    }
}
