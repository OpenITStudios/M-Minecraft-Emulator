//! Parent and recovery helper jointly retain one checkpoint lease per exact window.
//! This is a handoff/concurrency guard, not process attribution or account authority.
use super::OwnedProcess;
use std::io;

pub(super) struct RecoveryGate {
    #[cfg(windows)]
    _handle: std::os::windows::io::OwnedHandle,
}
#[cfg(windows)]
fn name(target: &OwnedProcess) -> io::Result<Vec<u16>> {
    let key = super::super::instance_identity::parse_window_identity(
        target.pid,
        Some(&target.created_file_time),
        Some(&target.window_handle),
    )
    .ok_or_else(|| io::Error::other("Invalid recovery window identity"))?;
    Ok(format!(
        "Local\\LazyTesterSurface-{}-{:016x}-{:016x}",
        key.pid, key.created_file_time, key.hwnd
    )
    .encode_utf16()
    .chain(Some(0))
    .collect())
}
pub(super) fn create(target: &OwnedProcess) -> io::Result<RecoveryGate> {
    #[cfg(windows)]
    {
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        use windows_sys::Win32::{
            Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
            System::Threading::CreateMutexW,
        };
        let name = name(target)?;
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        let handle = unsafe { OwnedHandle::from_raw_handle(handle as _) };
        if existed {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "The previous workspace is still restoring this game window",
            ));
        }
        Ok(RecoveryGate { _handle: handle })
    }
    #[cfg(not(windows))]
    {
        let _ = target;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Window recovery requires Windows",
        ))
    }
}
pub(super) fn open(target: &OwnedProcess) -> io::Result<RecoveryGate> {
    #[cfg(windows)]
    {
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        use windows_sys::Win32::System::Threading::{OpenMutexW, SYNCHRONIZATION_SYNCHRONIZE};
        let name = name(target)?;
        let handle = unsafe { OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(io::Error::other("Missing parent recovery lease"));
        }
        Ok(RecoveryGate {
            _handle: unsafe { OwnedHandle::from_raw_handle(handle as _) },
        })
    }
    #[cfg(not(windows))]
    {
        let _ = target;
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Window recovery requires Windows",
        ))
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;
    #[test]
    fn new_capture_waits_until_parent_and_helper_finish_the_same_checkpoint() {
        let target = OwnedProcess {
            pid: std::process::id(),
            created_file_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string(),
            session_id: 1,
            window_handle: "0x1".into(),
            responding: true,
        };
        // Synthetic key only: no HWND is read or modified and no game is launched.
        let parent = create(&target).unwrap();
        let helper = open(&target).unwrap();
        assert!(create(&target).is_err());
        drop(parent);
        assert!(
            create(&target).is_err(),
            "UI exit must not outrun the old recovery helper"
        );
        drop(helper);
        let replacement = create(&target).unwrap();
        drop(replacement);
    }
}
