//! Read creation FILETIME from an already-owned Windows process handle.
//! This does not establish identity, liveness, access rights, or ownership.
//! Those checks remain with the caller.

use std::io;
use windows_sys::Win32::{
    Foundation::{FILETIME, HANDLE},
    System::Threading::GetProcessTimes,
};

pub(crate) fn creation_filetime(process: HANDLE) -> io::Result<u64> {
    let mut times = [FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    }; 4];
    let [created, exited, kernel, user] = &mut times;
    if unsafe { GetProcessTimes(process, created, exited, kernel, user) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok((u64::from(created.dwHighDateTime) << 32) | u64::from(created.dwLowDateTime))
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Threading::GetCurrentProcess;

    #[test]
    fn creation_time_is_stable_on_current_process() {
        let process = unsafe { GetCurrentProcess() };
        let birth = creation_filetime(process).unwrap();
        assert_ne!(birth, 0);
        assert_eq!(creation_filetime(process).unwrap(), birth);
    }

    #[test]
    fn null_handle_is_rejected() {
        assert!(creation_filetime(std::ptr::null_mut()).is_err());
    }
}
