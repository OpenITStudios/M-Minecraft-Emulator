//! Opt-in test-only Windows helper. This binary is NEVER part of Minecraft
//! launch/admission, Tauri, a service, or a driver. It receives exactly two
//! inherited Win32 HANDLE capabilities from the original synthetic C++
//! fixture parent: the parent-created child process and its exclusively opened
//! observation record. No path, profile name, SID or credential is accepted.
//! The original parent is the trusted provenance authority; this executable
//! can validate passed handles but cannot independently attest its caller.

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct Arguments {
    process_handle: usize,
    record_handle: usize,
    pid: u32,
    created: u64,
    reopened: bool,
}

fn parse(arguments: &[String]) -> Result<Arguments, ()> {
    if arguments.len() != 6 || arguments[0] != "--verify-inherited-fixture" {
        return Err(());
    }
    let process_handle = arguments[1].parse::<usize>().map_err(|_| ())?;
    let record_handle = arguments[2].parse::<usize>().map_err(|_| ())?;
    let pid = arguments[3].parse::<u32>().map_err(|_| ())?;
    let created = arguments[4].parse::<u64>().map_err(|_| ())?;
    let reopened = match arguments[5].as_str() {
        "first" => false,
        "reopened" => true,
        _ => return Err(()),
    };
    if process_handle == 0 || record_handle == 0 || process_handle == record_handle
        || pid == 0 || created == 0
    {
        return Err(());
    }
    Ok(Arguments { process_handle, record_handle, pid, created, reopened })
}

#[cfg(windows)]
mod native {
    use super::Arguments;
    use std::{
        fs::File,
        io::{Read, Seek},
        os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    };
    use virtual_clients_core::independent_engine::{
        decode_synthetic_fixture_paths, SyntheticFixturePhase, SYNTHETIC_FRAME_BYTES,
    };
    use windows_sys::Win32::{
        Foundation::{FILETIME, HANDLE, WAIT_TIMEOUT},
        Security::{GetTokenInformation, TokenIsAppContainer, TOKEN_QUERY},
        Storage::FileSystem::{
            GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
            FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
        },
        System::Threading::{
            GetProcessId, GetProcessTimes, OpenProcessToken, WaitForSingleObject,
        },
    };

    #[derive(Debug, Eq, PartialEq)]
    struct FileIdentity {
        volume: u32,
        index: u64,
        size: u64,
        written: u64,
        attributes: u32,
    }

    fn file_identity(file: &File) -> Result<FileIdentity, ()> {
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        let handle = file.as_raw_handle() as HANDLE;
        if unsafe { GetFileInformationByHandle(handle, &mut info) } == 0 {
            return Err(());
        }
        let size = (u64::from(info.nFileSizeHigh) << 32) | u64::from(info.nFileSizeLow);
        if size != SYNTHETIC_FRAME_BYTES as u64
            || info.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
        {
            return Err(());
        }
        Ok(FileIdentity {
            volume: info.dwVolumeSerialNumber,
            index: (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
            size,
            written: (u64::from(info.ftLastWriteTime.dwHighDateTime) << 32)
                | u64::from(info.ftLastWriteTime.dwLowDateTime),
            attributes: info.dwFileAttributes,
        })
    }

    fn verify_child(process: &OwnedHandle, expected: &Arguments) -> Result<(), ()> {
        let handle = process.as_raw_handle() as HANDLE;
        if unsafe { GetProcessId(handle) } != expected.pid
            || unsafe { WaitForSingleObject(handle, 0) } != WAIT_TIMEOUT
        {
            return Err(());
        }
        let (mut created, mut exited, mut kernel, mut user):
            (FILETIME, FILETIME, FILETIME, FILETIME) = unsafe { std::mem::zeroed() };
        if unsafe { GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user) } == 0 {
            return Err(());
        }
        let observed = (u64::from(created.dwHighDateTime) << 32)
            | u64::from(created.dwLowDateTime);
        if observed != expected.created {
            return Err(());
        }
        let mut raw = std::ptr::null_mut();
        if unsafe { OpenProcessToken(handle, TOKEN_QUERY, &mut raw) } == 0 {
            return Err(());
        }
        let token = unsafe {
            // OpenProcessToken creates a new owned handle, closed at scope end.
            OwnedHandle::from_raw_handle(raw)
        };
        let mut appcontainer = 0u32;
        let mut returned = 0u32;
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle() as HANDLE,
                TokenIsAppContainer,
                (&mut appcontainer as *mut u32).cast(),
                std::mem::size_of::<u32>() as u32,
                &mut returned,
            )
        } == 0 || appcontainer != 1
        {
            return Err(());
        }
        Ok(())
    }

    pub(super) fn run(spec: Arguments) -> Result<(), ()> {
        // The C++ parent calls CreateProcessW with an explicit inherited
        // HANDLE LIST. Both handles are duplicated with bounded rights.
        // Values are local process HANDLEs, not portable PIDs or paths.
        let process = unsafe { OwnedHandle::from_raw_handle(spec.process_handle as *mut _) };
        let mut record = unsafe { File::from_raw_handle(spec.record_handle as *mut _) };
        verify_child(&process, &spec)?;
        let before = file_identity(&record)?;
        record.rewind().map_err(|_| ())?;
        let mut frame = [0u8; SYNTHETIC_FRAME_BYTES];
        record.read_exact(&mut frame).map_err(|_| ())?;
        if file_identity(&record)? != before {
            return Err(());
        }
        let decoded = decode_synthetic_fixture_paths(&frame).map_err(|_| ())?;
        let wanted = if spec.reopened {
            SyntheticFixturePhase::ReopenedProcess
        } else {
            SyntheticFixturePhase::FirstProcess
        };
        if decoded.phase != wanted {
            return Err(());
        }
        verify_child(&process, &spec)?;
        // No serializable process/window/account proof is produced.
        Ok(())
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let spec = match parse(&arguments) {
        Ok(spec) => spec,
        Err(()) => std::process::exit(2),
    };
    #[cfg(windows)]
    {
        std::process::exit(if native::run(spec).is_ok() { 0 } else { 3 });
    }
    #[cfg(not(windows))]
    {
        let _ = spec;
        std::process::exit(4);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> Vec<String> {
        ["--verify-inherited-fixture", "123", "456", "42", "133400000000000000", "first"]
            .into_iter()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn accepts_only_bounded_handle_arguments_and_full_width_creation_time() {
        let args = parse(&valid()).unwrap();
        assert_eq!(args.process_handle, 123);
        assert_eq!(args.record_handle, 456);
        assert_eq!(args.pid, 42);
        assert_eq!(args.created, 133400000000000000);
        assert!(!args.reopened);
        let mut restarted = valid();
        restarted[5] = "reopened".into();
        assert!(parse(&restarted).unwrap().reopened);
    }

    #[test]
    fn rejects_invalid_handles_and_mixed_process_identities() {
        for (index, value) in [
            (1, "0"), (2, "0"), (2, "123"), (3, "0"), (4, "0"),
            (4, "133400000000000000.1"), (5, "minecraft"),
            (1, "-1"), (3, "-1"), (4, "18446744073709551616"),
        ] {
            let mut args = valid();
            args[index] = value.into();
            assert!(parse(&args).is_err());
        }
        let mut unexpected = valid();
        unexpected.push("extra-path".into());
        assert!(parse(&unexpected).is_err());
        let mut wrong_command = valid();
        wrong_command[0] = "--launch-minecraft".into();
        assert!(parse(&wrong_command).is_err());
    }
}
