//! Private first stage of the Education bootstrap experiment.
//! No Tauri command, provider admission, DLL injection or resume operation.
//! The child remains unstarted: profile/auth mediation must precede a future
//! separately reviewed resume path. Direct-start game compatibility is UNKNOWN.
#![allow(dead_code)]

use std::{
    ffi::OsStr,
    fs::{File, OpenOptions},
    io, mem,
    os::windows::{
        ffi::OsStrExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    path::Path,
    ptr,
};
use windows_sys::Win32::{
    Foundation::{FILETIME, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Storage::FileSystem::{
        FileAttributeTagInfo, FileIdInfo, GetFileInformationByHandleEx, FILE_ATTRIBUTE_DIRECTORY,
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_ATTRIBUTE_TAG_INFO, FILE_ID_INFO, FILE_SHARE_READ,
    },
    System::RemoteDesktop::ProcessIdToSessionId,
    System::Threading::{
        CreateProcessW, GetCurrentProcessId, GetProcessId, GetProcessIdOfThread, GetProcessTimes,
        IsWow64Process2, QueryFullProcessImageNameW, TerminateProcess, WaitForSingleObject,
        CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, PROCESS_INFORMATION, STARTUPINFOW,
    },
};

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

fn wide(value: &OsStr) -> io::Result<Vec<u16>> {
    let mut result: Vec<u16> = value.encode_wide().collect();
    if result.is_empty()
        || result.len() > 30_000
        || result.contains(&0)
        || result.contains(&(b'"' as u16))
    {
        return Err(invalid("Invalid bootstrap executable path"));
    }
    result.push(0);
    Ok(result)
}

fn image_id(file: &File) -> io::Result<FILE_ID_INFO> {
    let handle = file.as_raw_handle() as HANDLE;
    let mut tag: FILE_ATTRIBUTE_TAG_INFO = unsafe { mem::zeroed() };
    let mut id: FILE_ID_INFO = unsafe { mem::zeroed() };
    // SAFETY: File owns the handle and both output buffers have their exact ABI sizes.
    if unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileAttributeTagInfo,
            &mut tag as *mut _ as _,
            mem::size_of_val(&tag) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0 {
        return Err(invalid("Bootstrap image must be an ordinary file"));
    }
    if unsafe {
        GetFileInformationByHandleEx(
            handle,
            FileIdInfo,
            &mut id as *mut _ as _,
            mem::size_of_val(&id) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(id)
}

fn same_image(a: &FILE_ID_INFO, b: &FILE_ID_INFO) -> bool {
    a.VolumeSerialNumber == b.VolumeSerialNumber && a.FileId.Identifier == b.FileId.Identifier
}

fn pin_image(path: &Path) -> io::Result<File> {
    // Deny write/delete sharing while verification and process creation use this image.
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(path)?;
    image_id(&file)?;
    Ok(file)
}

/// Owns only the original handles returned by this module's CreateProcessW.
/// There is intentionally no resume, detach, raw-handle export or PID-reopen API.
pub(super) struct PendingEducationBootstrap {
    process: OwnedHandle,
    _thread: OwnedHandle,
    _image: File,
    pid: u32,
    created: u64,
}

impl PendingEducationBootstrap {
    pub(super) fn identity(&self) -> io::Result<(u32, u64)> {
        let process = self.process.as_raw_handle() as HANDLE;
        let mut created: FILETIME = unsafe { mem::zeroed() };
        let mut exited: FILETIME = unsafe { mem::zeroed() };
        let mut kernel: FILETIME = unsafe { mem::zeroed() };
        let mut user: FILETIME = unsafe { mem::zeroed() };
        // SAFETY: Query only the retained newly created process, never an existing PID.
        if unsafe { GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user) }
            == 0
        {
            return Err(io::Error::last_os_error());
        }
        let birth = ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64;
        if unsafe { GetProcessId(process) } != self.pid
            || birth == 0
            || (self.created != 0 && self.created != birth)
        {
            return Err(invalid("Bootstrap process identity mismatch"));
        }
        match unsafe { WaitForSingleObject(process, 0) } {
            WAIT_TIMEOUT => Ok((self.pid, birth)),
            WAIT_OBJECT_0 => Err(invalid("Bootstrap process exited before enrollment")),
            _ => Err(io::Error::last_os_error()),
        }
    }

    pub(super) fn cancel(&mut self) -> io::Result<()> {
        let process = self.process.as_raw_handle() as HANDLE;
        // SAFETY: This is exclusively an engine-created child whose primary thread
        // has never been resumed by this owner, not a running account-bearing game.
        match unsafe { WaitForSingleObject(process, 0) } {
            WAIT_OBJECT_0 => return Ok(()),
            WAIT_TIMEOUT => {}
            _ => return Err(io::Error::last_os_error()),
        }
        if unsafe { TerminateProcess(process, 90) } == 0
            && unsafe { WaitForSingleObject(process, 0) } != WAIT_OBJECT_0
        {
            return Err(io::Error::last_os_error());
        }
        if unsafe { WaitForSingleObject(process, 5_000) } != WAIT_OBJECT_0 {
            return Err(invalid("Unstarted bootstrap child cleanup is unconfirmed"));
        }
        Ok(())
    }
}

impl Drop for PendingEducationBootstrap {
    fn drop(&mut self) {
        if self.cancel().is_err() {
            eprintln!("Unstarted Education bootstrap child cleanup is unconfirmed");
        }
    }
}

fn interactive_session(pid: u32) -> io::Result<u32> {
    let mut session = 0;
    // SAFETY: Windows writes one u32; no process/token/account contents are read.
    if unsafe { ProcessIdToSessionId(pid, &mut session) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if session == 0 {
        return Err(invalid("Interactive bootstrap session required"));
    }
    Ok(session)
}

// Private to the registered-target entry point and this module's own-executable tests.
fn create_unstarted(path: &Path, image: File) -> io::Result<PendingEducationBootstrap> {
    if !path.is_absolute() {
        return Err(invalid("Absolute bootstrap image required"));
    }
    let expected = image_id(&image)?;
    let application = wide(path.as_os_str())?;
    let directory = wide(
        path.parent()
            .ok_or_else(|| invalid("Missing image directory"))?
            .as_os_str(),
    )?;
    let session = interactive_session(unsafe { GetCurrentProcessId() })?;
    let mut command = vec![b'"' as u16];
    command.extend_from_slice(&application[..application.len() - 1]);
    command.extend_from_slice(&[b'"' as u16, 0]);
    let mut startup: STARTUPINFOW = unsafe { mem::zeroed() };
    startup.cb = mem::size_of_val(&startup) as u32;
    let mut info: PROCESS_INFORMATION = unsafe { mem::zeroed() };
    // SAFETY: Absolute lpApplicationName; writable quoted argv[0] only; no shell,
    // arguments, inherited handles, custom token, elevation or breakaway fallback.
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            0,
            CREATE_SUSPENDED | CREATE_UNICODE_ENVIRONMENT,
            ptr::null(),
            directory.as_ptr(),
            &startup,
            &mut info,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: A successful CreateProcessW returns two newly owned valid handles.
    let mut child = PendingEducationBootstrap {
        process: unsafe { OwnedHandle::from_raw_handle(info.hProcess as _) },
        _thread: unsafe { OwnedHandle::from_raw_handle(info.hThread as _) },
        _image: image,
        pid: info.dwProcessId,
        created: 0,
    };
    child.created = child.identity()?.1;
    if unsafe { GetProcessIdOfThread(child._thread.as_raw_handle() as HANDLE) } != child.pid
        || interactive_session(child.pid)? != session
    {
        return Err(invalid("Bootstrap primary thread/session mismatch"));
    }
    let mut machine = 0;
    let mut native = 0;
    if unsafe {
        IsWow64Process2(
            child.process.as_raw_handle() as HANDLE,
            &mut machine,
            &mut native,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if machine != 0 || native != 0x8664 {
        return Err(invalid("Only native AMD64 bootstrap is supported"));
    }
    let mut actual = [0u16; 32768];
    let mut count = actual.len() as u32;
    if unsafe {
        QueryFullProcessImageNameW(
            child.process.as_raw_handle() as HANDLE,
            0,
            actual.as_mut_ptr(),
            &mut count,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    use std::{ffi::OsString, os::windows::ffi::OsStringExt};
    let actual_path = std::path::PathBuf::from(OsString::from_wide(&actual[..count as usize]));
    if !same_image(&expected, &image_id(&pin_image(&actual_path)?)?) {
        return Err(invalid(
            "Created bootstrap process image does not match pinned target",
        ));
    }
    child.identity()?;
    Ok(child)
}

/// Source-only alternative to AppsFolder for a future reviewed experiment.
/// No caller can supply a target, arguments, slot profile or DLL. Resolution
/// uses the existing registration/Start-app/signature checks with legacy
/// preferences excluded, then repeats verification while the image is locked.
/// On-disk signature/path checks are NOT loaded-PE publisher attestation.
pub(super) fn prepare_registered() -> io::Result<PendingEducationBootstrap> {
    let selector = "Resolve-ManagerEducationTarget -RegisteredOnly";
    let target = super::resolve_registered_education_target(selector)?;
    let path = Path::new(&target);
    let image = pin_image(path)?;
    let checked = super::resolve_registered_education_target(selector)?;
    if checked != target {
        return Err(invalid("Education registration changed during preparation"));
    }
    create_unstarted(path, image)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetExitCodeProcess};

    #[test]
    fn original_child_is_pinned_unstarted_and_cancellable() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let image = pin_image(&path).unwrap();
        let id = image_id(&image).unwrap();
        let mut child = create_unstarted(&path, image).unwrap();
        let identity = child.identity().unwrap();
        assert_ne!(identity.0, unsafe { GetCurrentProcessId() });
        assert!(identity.1 > 0);
        assert!(same_image(&id, &image_id(&child._image).unwrap()));
        child.cancel().unwrap();
        assert!(child.identity().is_err());
        let mut code = 0;
        assert_ne!(
            unsafe { GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code) },
            0
        );
        assert_eq!(code, 90); // The test runner never ran in the suspended child.
        assert_eq!(
            unsafe { WaitForSingleObject(GetCurrentProcess(), 0) },
            WAIT_TIMEOUT
        );
    }

    #[test]
    fn dropped_preparation_cancels_only_its_original_child() {
        use windows_sys::Win32::Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS};
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let child = create_unstarted(&path, pin_image(&path).unwrap()).unwrap();
        let current = unsafe { GetCurrentProcess() };
        let mut duplicate = ptr::null_mut();
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    current,
                    child.process.as_raw_handle() as HANDLE,
                    current,
                    &mut duplicate,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        let retained = unsafe { OwnedHandle::from_raw_handle(duplicate as _) };
        drop(child);
        assert_eq!(
            unsafe { WaitForSingleObject(retained.as_raw_handle() as HANDLE, 0) },
            WAIT_OBJECT_0
        );
    }

    #[test]
    fn malformed_executable_strings_are_refused() {
        for value in ["", "x\0y", "x\"y"] {
            assert!(wide(OsStr::new(value)).is_err());
        }
    }
}
