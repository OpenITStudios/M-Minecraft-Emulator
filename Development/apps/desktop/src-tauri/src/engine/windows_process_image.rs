//! One read-only native image-path reader for already-pinned process handles.
//! Small paths avoid the repeated 64-KiB UTF-16 allocations previously made
//! during every controller IPC/registered-image observation. Never infer a
//! process identity from this path alone: callers retain their own HANDLE,
//! FILETIME, session, signer and file-identity checks.
use std::io;
use windows_sys::Win32::{
    Foundation::{ERROR_INSUFFICIENT_BUFFER, HANDLE},
    System::Threading::QueryFullProcessImageNameW,
};

const INITIAL_CHARACTERS: usize = 512;
const MAX_CHARACTERS: usize = 32_768;

// QueryFullProcessImageNameW returns a length excluding the terminator.
// On a short buffer, only ERROR_INSUFFICIENT_BUFFER permits retrying. Never
// accept a truncated path, drop an unknown error or change the process handle.
pub(crate) fn query(process: HANDLE) -> io::Result<Vec<u16>> {
    let mut path = vec![0u16; INITIAL_CHARACTERS];
    loop {
        let mut length = path.len() as u32;
        if unsafe { QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut length) } != 0 {
            let written = length as usize;
            if written == 0 || written >= path.len() || path[..written].contains(&0) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Windows process image path is malformed or oversized",
                ));
            }
            path.truncate(written);
            return Ok(path);
        }
        let error = io::Error::last_os_error();
        if error.raw_os_error() != Some(ERROR_INSUFFICIENT_BUFFER as i32)
            || path.len() >= MAX_CHARACTERS
        {
            return Err(error);
        }
        path.resize((path.len() * 2).min(MAX_CHARACTERS), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_process_path_is_observed_on_the_original_handle() {
        use windows_sys::Win32::System::Threading::GetCurrentProcess;
        let path = query(unsafe { GetCurrentProcess() }).unwrap();
        assert!(!path.is_empty());
        assert!(!path.contains(&0));
        assert!(path.len() < MAX_CHARACTERS);
    }
}
