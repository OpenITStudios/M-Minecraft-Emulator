//! Private first stage of the Education bootstrap experiment.
//! Production code has no Tauri command, provider admission, DLL injection or resume.
//! The cfg(test) bridge enrolls only the explicitly built own C++ fixture.
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
    storage: Option<SlotStorage>,
    configuration: Option<OwnedHandle>,
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
        // has no production resume path. Tests may resume only our C++ control executable.
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
fn create_unstarted(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
) -> io::Result<PendingEducationBootstrap> {
    create_unstarted_impl(
        path,
        image,
        storage,
        #[cfg(test)]
        None,
    )
}

fn create_unstarted_impl(
    path: &Path,
    image: File,
    storage: Option<SlotStorage>,
    #[cfg(test)] proof: Option<&tests::FixtureReceipt>,
) -> io::Result<PendingEducationBootstrap> {
    #[cfg(test)]
    if proof.is_some() && storage.is_none() {
        return Err(invalid("Fixture receipt requires configured storage"));
    }
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
    let configuration = storage
        .as_ref()
        .map(|_| Configuration::create())
        .transpose()?;
    let inherited: Vec<HANDLE> = configuration
        .as_ref()
        .map(|c| vec![c.readonly.as_raw_handle() as HANDLE])
        .unwrap_or_default();
    #[cfg(test)]
    let inherited = {
        let mut handles = inherited;
        if let Some(proof) = proof {
            handles.push(proof.0.as_raw_handle() as HANDLE);
        }
        handles
    };
    let attributes = if inherited.is_empty() {
        None
    } else {
        Some(InheritList::new(&inherited)?)
    };
    let environment = match (&storage, &configuration) {
        (Some(store), Some(config)) => Some(child_environment(
            store,
            config.readonly.as_raw_handle() as HANDLE,
        )?),
        _ => None,
    };
    #[cfg(test)]
    let environment = match (environment, proof) {
        (Some(environment), Some(proof)) => Some(proof.environment(environment)?),
        (environment, _) => environment,
    };
    let mut startup: STARTUPINFOEXW = unsafe { mem::zeroed() };
    startup.StartupInfo.cb = if configuration.is_some() {
        mem::size_of::<STARTUPINFOEXW>() as u32
    } else {
        mem::size_of::<STARTUPINFOW>() as u32
    };
    startup.lpAttributeList = attributes.as_ref().map_or(ptr::null_mut(), |a| a.list);
    let mut info: PROCESS_INFORMATION = unsafe { mem::zeroed() };
    // SAFETY: Absolute lpApplicationName; writable quoted argv[0] only; no shell,
    // arbitrary arguments, ambient handle inheritance, custom token, elevation or breakaway fallback.
    if unsafe {
        CreateProcessW(
            application.as_ptr(),
            command.as_mut_ptr(),
            ptr::null(),
            ptr::null(),
            i32::from(configuration.is_some()),
            CREATE_SUSPENDED
                | CREATE_UNICODE_ENVIRONMENT
                | if configuration.is_some() {
                    EXTENDED_STARTUPINFO_PRESENT
                } else {
                    0
                },
            environment
                .as_ref()
                .map_or(ptr::null(), |e| e.as_ptr() as _),
            directory.as_ptr(),
            &startup.StartupInfo,
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
        storage: None,
        configuration: None,
    };
    #[cfg(test)]
    if let Some(proof) = proof {
        if unsafe {
            SetHandleInformation(proof.0.as_raw_handle() as HANDLE, HANDLE_FLAG_INHERIT, 0)
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
    }
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
    if let (Some(config), Some(store)) = (configuration, storage) {
        child.configuration = Some(config.publish(&child, &store)?);
        child.storage = Some(store);
    }
    child.identity()?;
    Ok(child)
}

/// Source-only two-slot preparation. No game/DLL code can be resumed here.
/// The existing registered-image resolver remains the only target selector.
pub(super) fn prepare_registered() -> io::Result<[PendingEducationBootstrap; 2]> {
    let selector = "Resolve-ManagerEducationTarget -RegisteredOnly";
    let target = super::resolve_registered_education_target(selector)?;
    let path = Path::new(&target);
    let image = pin_image(path)?;
    if super::resolve_registered_education_target(selector)? != target {
        return Err(invalid("Education registration changed during preparation"));
    }
    let [first, second] = registered_slots()?;
    let first = create_unstarted(path, image.try_clone()?, Some(first))?;
    let second = create_unstarted(path, image, Some(second))?;
    Ok([first, second])
}

use std::{
    ffi::{c_void, OsString},
    os::windows::ffi::OsStringExt,
    path::PathBuf,
};
use windows_sys::Win32::{
    Foundation::{
        DuplicateHandle, SetHandleInformation, DUPLICATE_SAME_ACCESS, HANDLE_FLAG_INHERIT,
        INVALID_HANDLE_VALUE,
    },
    Globalization::CompareStringOrdinal,
    Storage::FileSystem::{
        FileCaseSensitiveInfo, GetFinalPathNameByHandleW, FILE_CASE_SENSITIVE_INFO,
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY,
        FILE_READ_ATTRIBUTES, FILE_SHARE_WRITE, VOLUME_NAME_DOS,
    },
    System::{
        Com::{CoCreateGuid, CoTaskMemFree},
        Environment::{FreeEnvironmentStringsW, GetEnvironmentStringsW},
        LibraryLoader::{GetModuleHandleW, GetProcAddress},
        Memory::{
            CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE,
            PAGE_READWRITE,
        },
        Registry::{
            RegCloseKey, RegCreateKeyExW, RegOpenCurrentUser, HKEY, HKEY_CURRENT_USER,
            KEY_ALL_ACCESS, KEY_READ, REG_CREATED_NEW_KEY, REG_OPTION_NON_VOLATILE,
        },
        Threading::{
            DeleteProcThreadAttributeList, GetCurrentProcess, InitializeProcThreadAttributeList,
            UpdateProcThreadAttribute, EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST,
            PROC_THREAD_ATTRIBUTE_HANDLE_LIST, STARTUPINFOEXW,
        },
    },
    UI::Shell::{
        FOLDERID_LocalAppData, FOLDERID_Profile, FOLDERID_RoamingAppData, SHGetKnownFolderPath,
    },
};

const CONFIG_ENV: &str = "MIVUBI_EDUCATION_BOOTSTRAP_SECTION";

// Must match the private education-config.h, not a public product wire type.
#[repr(C)]
struct EducationConfig {
    magic: u32,
    version: u32,
    bytes: u32,
    slot: u32,
    pid: u32,
    session: u32,
    scope: u32,
    reserved: u32,
    created: FILETIME,
    logical_root: HANDLE,
    private_root: HANDLE,
    registry_root: HKEY,
    logical_id: FILE_ID_INFO,
    private_id: FILE_ID_INFO,
    user_key_path: [u16; 512],
}
const _: () = assert!(mem::size_of::<EducationConfig>() == 1136);
const _: () = assert!(mem::offset_of!(EducationConfig, logical_id) == 64);
const _: () = assert!(mem::offset_of!(EducationConfig, user_key_path) == 112);

struct RegistryKey(HKEY);
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

struct SlotStorage {
    slot: u32,
    logical: File,
    private: File,
    registry: RegistryKey,
    user_key_path: [u16; 512],
    // Fresh data persists on drop; this is not another profile/readiness database.
    private_path: PathBuf,
}

fn directory_id(file: &File) -> io::Result<FILE_ID_INFO> {
    let raw = file.as_raw_handle() as HANDLE;
    let mut tag: FILE_ATTRIBUTE_TAG_INFO = unsafe { mem::zeroed() };
    let mut id: FILE_ID_INFO = unsafe { mem::zeroed() };
    let mut case: FILE_CASE_SENSITIVE_INFO = unsafe { mem::zeroed() };
    // SAFETY: Query fixed-size metadata on the retained directory handle.
    if unsafe {
        GetFileInformationByHandleEx(
            raw,
            FileAttributeTagInfo,
            &mut tag as *mut _ as _,
            mem::size_of_val(&tag) as u32,
        )
    } == 0
        || unsafe {
            GetFileInformationByHandleEx(
                raw,
                FileIdInfo,
                &mut id as *mut _ as _,
                mem::size_of_val(&id) as u32,
            )
        } == 0
        || unsafe {
            GetFileInformationByHandleEx(
                raw,
                FileCaseSensitiveInfo,
                &mut case as *mut _ as _,
                mem::size_of_val(&case) as u32,
            )
        } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if tag.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || tag.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || case.Flags != 0
    {
        return Err(invalid("Unsupported bootstrap directory"));
    }
    Ok(id)
}

fn open_directory(path: &Path, access: u32) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .access_mode(access)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    directory_id(&file)?;
    directory_path(&file)?;
    Ok(file)
}

fn directory_path(file: &File) -> io::Result<PathBuf> {
    let mut path = [0u16; 512];
    let count = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle() as HANDLE,
            path.as_mut_ptr(),
            512,
            VOLUME_NAME_DOS,
        )
    };
    if count < 7 || count >= 480 {
        return Err(invalid("Unsupported bootstrap directory path length"));
    }
    let value = &path[..count as usize];
    if !value.starts_with(&[92, 92, 63, 92])
        || value[5] != 58
        || value[6] != 92
        || !((65..=90).contains(&value[4]) || (97..=122).contains(&value[4]))
        || value.iter().any(|c| *c < 32 || *c > 126)
    {
        return Err(invalid(
            "Bootstrap DLL currently requires bounded ASCII directory paths",
        ));
    }
    Ok(PathBuf::from(OsString::from_wide(&value[4..])))
}

fn fresh_root(parent: &Path) -> io::Result<PathBuf> {
    let mut guid: windows_sys::core::GUID = unsafe { mem::zeroed() };
    if unsafe { CoCreateGuid(&mut guid) } < 0 {
        return Err(invalid("Cannot allocate fresh pilot identity"));
    }
    let suffix: String = guid.data4.iter().map(|b| format!("{b:02x}")).collect();
    let path = parent.join(format!(
        "MivubiEducationPilot_{:08x}{:04x}{:04x}{suffix}",
        guid.data1, guid.data2, guid.data3
    ));
    std::fs::create_dir(&path)?; // Never reopen, migrate, recursively clear or reseed old data.
    Ok(path)
}

fn registry_leaf(id: &FILE_ID_INFO, slot: u32) -> io::Result<String> {
    if !(1..=2).contains(&slot) {
        return Err(invalid("Only MC01 and MC02 are prepared"));
    }
    let bytes: String = id
        .FileId
        .Identifier
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(format!(
        "{:016x}_{bytes}_{}",
        id.VolumeSerialNumber,
        if slot == 1 { "A" } else { "B" }
    ))
}

fn user_registry_path() -> io::Result<[u16; 512]> {
    let mut key = ptr::null_mut();
    let status = unsafe { RegOpenCurrentUser(KEY_READ, &mut key) };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    let key = RegistryKey(key);
    type Query = unsafe extern "system" fn(HANDLE, u32, *mut c_void, u32, *mut u32) -> i32;
    let name = wide(OsStr::new("ntdll.dll"))?;
    let module = unsafe { GetModuleHandleW(name.as_ptr()) };
    if module.is_null() {
        return Err(io::Error::last_os_error());
    }
    let address = unsafe { GetProcAddress(module, c"NtQueryKey".as_ptr() as _) }
        .ok_or_else(|| invalid("Native registry metadata query unavailable"))?;
    let query: Query = unsafe { mem::transmute(address) };
    let mut buffer = [0u32; 257];
    let mut returned = 0;
    // SAFETY: Fixed loaded ntdll export, KeyNameInformation, bounded aligned output.
    if unsafe {
        query(
            key.0,
            3,
            buffer.as_mut_ptr() as _,
            mem::size_of_val(&buffer) as u32,
            &mut returned,
        )
    } < 0
        || returned < 4
        || returned as usize > mem::size_of_val(&buffer)
    {
        return Err(invalid("Current-user registry metadata unavailable"));
    }
    let bytes = buffer[0] as usize;
    if bytes % 2 != 0 || bytes >= 1024 || bytes > returned as usize - 4 {
        return Err(invalid("Invalid current-user registry path"));
    }
    let mut result = [0u16; 512];
    unsafe {
        ptr::copy_nonoverlapping(
            buffer.as_ptr().add(1) as *const u16,
            result.as_mut_ptr(),
            bytes / 2,
        );
    }
    Ok(result)
}

fn fresh_slots(logical: File, parent: &Path) -> io::Result<[SlotStorage; 2]> {
    let logical_path = directory_path(&logical)?;
    let root = fresh_root(parent)?;
    let root_handle = open_directory(&root, FILE_READ_ATTRIBUTES)?;
    let root_path = directory_path(&root_handle)?;
    let a = root_path.to_string_lossy().to_ascii_lowercase();
    let b = logical_path.to_string_lossy().to_ascii_lowercase();
    if Path::new(&a).starts_with(&b) || Path::new(&b).starts_with(&a) {
        return Err(invalid(
            "Private backing must be outside the logical AppData tree",
        ));
    }
    let user_key_path = user_registry_path()?;
    let mut slots = Vec::with_capacity(2);
    for slot in 1..=2 {
        let private_path = root.join(format!("MC{slot:02}"));
        std::fs::create_dir(&private_path)?;
        let private = open_directory(&private_path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
        // Match the DLL's canonical spelling, including expansion of Windows 8.3 aliases.
        let private_path = directory_path(&private)?;
        for leaf in ["Roaming", "Local"] {
            std::fs::create_dir(private_path.join(leaf))?;
        }
        let id = directory_id(&private)?;
        let path = wide(OsStr::new(&format!(
            "Software\\MivubiEducationPilot\\{}",
            registry_leaf(&id, slot)?
        )))?;
        let mut key = ptr::null_mut();
        let mut disposition = 0;
        let status = unsafe {
            RegCreateKeyExW(
                HKEY_CURRENT_USER,
                path.as_ptr(),
                0,
                ptr::null(),
                REG_OPTION_NON_VOLATILE,
                KEY_ALL_ACCESS,
                ptr::null(),
                &mut key,
                &mut disposition,
            )
        };
        if status != 0 {
            return Err(io::Error::from_raw_os_error(status as i32));
        }
        let registry = RegistryKey(key);
        if disposition != REG_CREATED_NEW_KEY {
            return Err(invalid("Refusing an existing pilot registry store"));
        }
        slots.push(SlotStorage {
            slot,
            logical: logical.try_clone()?,
            private,
            registry,
            user_key_path,
            private_path,
        });
    }
    slots
        .try_into()
        .map_err(|_| invalid("Missing private slot"))
}

fn known_folder(id: &windows_sys::core::GUID) -> io::Result<PathBuf> {
    let mut raw = ptr::null_mut();
    let status = unsafe { SHGetKnownFolderPath(id, 0, ptr::null_mut(), &mut raw) };
    if status < 0 || raw.is_null() {
        if !raw.is_null() {
            unsafe {
                CoTaskMemFree(raw as _);
            }
        }
        return Err(invalid("Current-user Known Folder unavailable"));
    }
    let mut count = 0;
    while count < 32768 && unsafe { *raw.add(count) } != 0 {
        count += 1;
    }
    let path = if count < 32768 {
        Ok(PathBuf::from(OsString::from_wide(unsafe {
            std::slice::from_raw_parts(raw, count)
        })))
    } else {
        Err(invalid("Invalid Known Folder pathname"))
    };
    unsafe {
        CoTaskMemFree(raw as _);
    }
    path
}

fn registered_slots() -> io::Result<[SlotStorage; 2]> {
    let roaming = known_folder(&FOLDERID_RoamingAppData)?;
    let local = known_folder(&FOLDERID_LocalAppData)?;
    if roaming.file_name() != Some(OsStr::new("Roaming"))
        || local.file_name() != Some(OsStr::new("Local"))
    {
        return Err(invalid(
            "Redirected AppData layouts require explicit mediation",
        ));
    }
    let a = open_directory(
        roaming
            .parent()
            .ok_or_else(|| invalid("Missing AppData parent"))?,
        FILE_READ_ATTRIBUTES,
    )?;
    let b = open_directory(
        local
            .parent()
            .ok_or_else(|| invalid("Missing AppData parent"))?,
        FILE_READ_ATTRIBUTES,
    )?;
    if !same_image(&directory_id(&a)?, &directory_id(&b)?) {
        return Err(invalid(
            "Split Roaming/Local roots are not supported by this DLL configuration",
        ));
    }
    fresh_slots(a, &known_folder(&FOLDERID_Profile)?)
}

struct Configuration {
    writable: OwnedHandle,
    readonly: OwnedHandle,
}
impl Configuration {
    fn create() -> io::Result<Self> {
        let raw = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                ptr::null(),
                PAGE_READWRITE,
                0,
                mem::size_of::<EducationConfig>() as u32,
                ptr::null(),
            )
        };
        if raw.is_null() {
            return Err(io::Error::last_os_error());
        }
        let writable = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        let mut reader = ptr::null_mut();
        if unsafe {
            DuplicateHandle(
                GetCurrentProcess(),
                raw,
                GetCurrentProcess(),
                &mut reader,
                FILE_MAP_READ,
                1,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(Self {
            writable,
            readonly: unsafe { OwnedHandle::from_raw_handle(reader as _) },
        })
    }

    fn publish(
        self,
        child: &PendingEducationBootstrap,
        storage: &SlotStorage,
    ) -> io::Result<OwnedHandle> {
        let identity = child.identity()?;
        let target = child.process.as_raw_handle() as HANDLE;
        let duplicate = |source: HANDLE| -> io::Result<HANDLE> {
            let mut remote = ptr::null_mut();
            if unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    source,
                    target,
                    &mut remote,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            // Valid in child only; child cancellation closes these on any later error.
            Ok(remote)
        };
        let mut config: EducationConfig = unsafe { mem::zeroed() };
        config.magic = 0x45444231;
        config.version = 1;
        config.bytes = mem::size_of_val(&config) as u32;
        config.slot = storage.slot;
        config.pid = identity.0;
        config.session = interactive_session(identity.0)?;
        config.scope = 1;
        config.created = FILETIME {
            dwLowDateTime: identity.1 as u32,
            dwHighDateTime: (identity.1 >> 32) as u32,
        };
        config.logical_root = duplicate(storage.logical.as_raw_handle() as HANDLE)?;
        config.private_root = duplicate(storage.private.as_raw_handle() as HANDLE)?;
        config.registry_root = duplicate(storage.registry.0)?;
        config.logical_id = directory_id(&storage.logical)?;
        config.private_id = directory_id(&storage.private)?;
        config.user_key_path = storage.user_key_path;
        let mapped = unsafe {
            MapViewOfFile(
                self.writable.as_raw_handle() as HANDLE,
                FILE_MAP_WRITE,
                0,
                0,
                mem::size_of::<EducationConfig>(),
            )
        };
        if mapped.Value.is_null() {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: Full pagefile-backed writable view of the checked private ABI size.
        unsafe {
            ptr::write(mapped.Value as *mut EducationConfig, config);
        }
        if unsafe { UnmapViewOfFile(mapped) } == 0 {
            return Err(io::Error::last_os_error());
        }
        if unsafe {
            SetHandleInformation(
                self.readonly.as_raw_handle() as HANDLE,
                HANDLE_FLAG_INHERIT,
                0,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        child.identity()?;
        // No parent writable view/handle survives publication.
        Ok(self.readonly)
    }
}

struct InheritList<'a> {
    _memory: Vec<usize>,
    list: LPPROC_THREAD_ATTRIBUTE_LIST,
    _handles: &'a [HANDLE],
}
impl<'a> InheritList<'a> {
    fn new(handles: &'a [HANDLE]) -> io::Result<Self> {
        let mut bytes = 0;
        unsafe {
            InitializeProcThreadAttributeList(ptr::null_mut(), 1, 0, &mut bytes);
        }
        if bytes == 0 {
            return Err(io::Error::last_os_error());
        }
        let mut memory = vec![0usize; bytes.div_ceil(mem::size_of::<usize>())];
        let list = memory.as_mut_ptr() as LPPROC_THREAD_ATTRIBUTE_LIST;
        if unsafe { InitializeProcThreadAttributeList(list, 1, 0, &mut bytes) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let result = Self {
            _memory: memory,
            list,
            _handles: handles,
        };
        if unsafe {
            UpdateProcThreadAttribute(
                list,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr() as _,
                mem::size_of_val(handles),
                ptr::null_mut(),
                ptr::null(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        Ok(result)
    }
}

impl Drop for InheritList<'_> {
    fn drop(&mut self) {
        unsafe {
            DeleteProcThreadAttributeList(self.list);
        }
    }
}

fn environment_name(row: &[u16]) -> &[u16] {
    let start = usize::from(row.first() == Some(&61)); // Preserve hidden =C: drive entries.
    let end = row[start..]
        .iter()
        .position(|c| *c == 61)
        .map_or(row.len(), |i| start + i);
    &row[..end]
}

fn ordinal_names(a: &[u16], b: &[u16]) -> io::Result<std::cmp::Ordering> {
    match unsafe { CompareStringOrdinal(a.as_ptr(), a.len() as i32, b.as_ptr(), b.len() as i32, 1) }
    {
        1 => Ok(std::cmp::Ordering::Less),
        2 => Ok(std::cmp::Ordering::Equal),
        3 => Ok(std::cmp::Ordering::Greater),
        _ => Err(io::Error::last_os_error()),
    }
}

fn child_environment(storage: &SlotStorage, section: HANDLE) -> io::Result<Vec<u16>> {
    let raw = unsafe { GetEnvironmentStringsW() };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let reserved: Vec<Vec<u16>> = [
        "APPDATA",
        "LOCALAPPDATA",
        CONFIG_ENV,
        "MIVUBI_BOOTSTRAP_TEST_SECTION",
    ]
    .iter()
    .map(|name| OsStr::new(name).encode_wide().collect())
    .collect();
    let mut rows = Vec::<Vec<u16>>::new();
    let mut cursor = raw;
    let mut units = 0usize;
    let copied = (|| {
        // Bound the copy; never log or persist inherited environment values.
        while unsafe { *cursor } != 0 {
            let mut length = 0;
            while unsafe { *cursor.add(length) } != 0 {
                length += 1;
                if units + length > 32767 {
                    return Err(invalid("Inherited environment exceeds bootstrap bound"));
                }
            }
            let row = unsafe { std::slice::from_raw_parts(cursor, length) };
            let mut overridden = false;
            for name in &reserved {
                overridden |=
                    ordinal_names(environment_name(row), name)? == std::cmp::Ordering::Equal;
            }
            if !overridden {
                rows.push(row.to_vec());
            }
            units += length + 1;
            cursor = unsafe { cursor.add(length + 1) };
        }
        Ok(())
    })();
    unsafe {
        FreeEnvironmentStringsW(raw);
    }
    copied?;
    for (key, value) in [
        (
            "APPDATA",
            storage.private_path.join("Roaming").into_os_string(),
        ),
        (
            "LOCALAPPDATA",
            storage.private_path.join("Local").into_os_string(),
        ),
        (CONFIG_ENV, OsString::from(format!("{}", section as usize))),
    ] {
        let mut entry: Vec<u16> = OsStr::new(key).encode_wide().collect();
        entry.push(61);
        entry.extend(value.encode_wide());
        rows.push(entry);
    }
    let mut compare_error = None;
    rows.sort_by(|a, b| {
        ordinal_names(environment_name(a), environment_name(b)).unwrap_or_else(|e| {
            compare_error = Some(e);
            std::cmp::Ordering::Equal
        })
    });
    if let Some(error) = compare_error {
        return Err(error);
    }
    let mut result = Vec::new();
    for row in rows {
        result.extend(row);
        result.push(0);
    }
    result.push(0);
    if result.len() > 32767 {
        return Err(invalid("Configured environment exceeds bootstrap bound"));
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetExitCodeProcess};

    // Writable receipt exists only for our cooperative C++ control, never the game.
    pub(super) struct FixtureReceipt(pub(super) OwnedHandle);
    impl FixtureReceipt {
        fn new() -> Self {
            let raw = unsafe {
                CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    ptr::null(),
                    PAGE_READWRITE,
                    0,
                    2064,
                    ptr::null(),
                )
            };
            assert!(!raw.is_null());
            let owned = unsafe { OwnedHandle::from_raw_handle(raw as _) };
            assert_ne!(
                unsafe { SetHandleInformation(raw, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) },
                0
            );
            Self(owned)
        }
        pub(super) fn environment(&self, environment: Vec<u16>) -> io::Result<Vec<u16>> {
            let mut rows: Vec<Vec<u16>> = environment
                .split(|&unit| unit == 0)
                .filter(|row| !row.is_empty())
                .map(|row| row.to_vec())
                .collect();
            rows.push(
                format!(
                    "MIVUBI_BOOTSTRAP_TEST_SECTION={}",
                    self.0.as_raw_handle() as usize
                )
                .encode_utf16()
                .collect(),
            );
            rows.sort_by(|a, b| ordinal_names(environment_name(a), environment_name(b)).unwrap());
            let mut result = Vec::new();
            for row in rows {
                result.extend(row);
                result.push(0);
            }
            result.push(0);
            if result.len() > 32767 {
                return Err(invalid("Fixture environment too large"));
            }
            Ok(result)
        }
    }

    type FixtureEntry = unsafe extern "system" fn(HANDLE, HANDLE, u32, u64, HANDLE, HANDLE) -> u32;
    struct FixtureEnroller {
        module: windows_sys::Win32::Foundation::HMODULE,
        prepare: FixtureEntry,
        enroll: FixtureEntry,
        _pins: [File; 2],
    }
    impl FixtureEnroller {
        fn load(path: &Path) -> Self {
            use windows_sys::Win32::System::LibraryLoader::{
                LoadLibraryExW, LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR, LOAD_LIBRARY_SEARCH_SYSTEM32,
            };
            assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
            let root = path.parent().unwrap();
            let library = root.join("bootstrap-enroll.dll");
            let pins = [
                pin_image(&library).unwrap(),
                pin_image(&root.join("bootstrap-compat.dll")).unwrap(),
            ];
            let name = wide(library.as_os_str()).unwrap();
            let module = unsafe {
                LoadLibraryExW(
                    name.as_ptr(),
                    ptr::null_mut(),
                    LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32,
                )
            };
            assert!(!module.is_null(), "{}", io::Error::last_os_error());
            let prepare =
                unsafe { GetProcAddress(module, c"PrepareOwnedFixture".as_ptr().cast()) }.unwrap();
            let enroll =
                unsafe { GetProcAddress(module, c"EnrollOwnedFixture".as_ptr().cast()) }.unwrap();
            Self {
                module,
                prepare: unsafe {
                    mem::transmute::<unsafe extern "system" fn() -> isize, FixtureEntry>(prepare)
                },
                enroll: unsafe {
                    mem::transmute::<unsafe extern "system" fn() -> isize, FixtureEntry>(enroll)
                },
                _pins: pins,
            }
        }
        fn call(
            &self,
            child: &PendingEducationBootstrap,
            proof: &FixtureReceipt,
            enroll: bool,
            birth: u64,
        ) -> u32 {
            child.identity().unwrap();
            let entry = if enroll { self.enroll } else { self.prepare };
            unsafe {
                entry(
                    child.process.as_raw_handle() as HANDLE,
                    child._thread.as_raw_handle() as HANDLE,
                    child.pid,
                    birth,
                    child.configuration.as_ref().unwrap().as_raw_handle() as HANDLE,
                    proof.0.as_raw_handle() as HANDLE,
                )
            }
        }
    }
    impl Drop for FixtureEnroller {
        fn drop(&mut self) {
            assert_ne!(
                unsafe { windows_sys::Win32::Foundation::FreeLibrary(self.module) },
                0
            );
        }
    }

    fn clone_store(store: &SlotStorage) -> SlotStorage {
        let mut key = ptr::null_mut();
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    store.registry.0,
                    GetCurrentProcess(),
                    &mut key,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        SlotStorage {
            slot: store.slot,
            logical: store.logical.try_clone().unwrap(),
            private: store.private.try_clone().unwrap(),
            registry: RegistryKey(key),
            user_key_path: store.user_key_path,
            private_path: store.private_path.clone(),
        }
    }

    fn finish_fixture(child: &PendingEducationBootstrap) {
        // Only this fixed owned fixture can be resumed in the test module.
        assert_eq!(
            unsafe {
                windows_sys::Win32::System::Threading::ResumeThread(
                    child._thread.as_raw_handle() as HANDLE
                )
            },
            1
        );
        assert_eq!(
            unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
            WAIT_OBJECT_0
        );
        let mut code = 1;
        assert_ne!(
            unsafe { GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code) },
            0
        );
        assert_eq!(code, 0);
        // Query in the unhooked parent, never through the child's translated cached key.
        use windows_sys::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, REG_DWORD};
        let name = wide(OsStr::new(&format!(
            "Software\\EducationEntry_{}_{}",
            child.pid, child.created
        )))
        .unwrap();
        let mut host = ptr::null_mut();
        assert_eq!(
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, name.as_ptr(), 0, KEY_READ, &mut host) },
            0
        );
        let host = RegistryKey(host);
        let mut marker = 0u32;
        let mut kind = 0;
        let mut bytes = mem::size_of_val(&marker) as u32;
        assert_eq!(
            unsafe {
                RegQueryValueExW(
                    host.0,
                    [77u16, 97, 114, 107, 101, 114, 0].as_ptr(),
                    ptr::null(),
                    &mut kind,
                    &mut marker as *mut _ as _,
                    &mut bytes,
                )
            },
            0
        );
        assert_eq!((kind, bytes, marker), (REG_DWORD, 4, 0x484f5354));
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ fixture and test-only enroller DLL"]
    fn rust_enrolls_own_fixture_before_dependency_tls_and_main() {
        let path = PathBuf::from(
            std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER")
                .expect("Set the trusted built C++ fixture path"),
        )
        .canonicalize()
        .unwrap();
        let enroller = FixtureEnroller::load(&path);
        let [a, b] = test_slots();
        for store in [&a, &b] {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
        }
        // A mismatched birth is rejected before mutation; original child remains cancellable.
        let proof = FixtureReceipt::new();
        let mut rejected = create_unstarted_impl(
            &path,
            pin_image(&path).unwrap(),
            Some(clone_store(&a)),
            Some(&proof),
        )
        .unwrap();
        assert_ne!(
            enroller.call(&rejected, &proof, true, rejected.created + 1),
            0
        );
        rejected.identity().unwrap();
        rejected.cancel().unwrap();
        // The same live handles for a different executable are rejected before any canary creation.
        let self_path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let proof = FixtureReceipt::new();
        let mut wrong_image = create_unstarted_impl(
            &self_path,
            pin_image(&self_path).unwrap(),
            Some(clone_store(&a)),
            Some(&proof),
        )
        .unwrap();
        assert_ne!(
            enroller.call(&wrong_image, &proof, true, wrong_image.created),
            0
        );
        wrong_image.identity().unwrap();
        wrong_image.cancel().unwrap();
        // Positive unhooked control must observe all three host registry canaries.
        let proof = FixtureReceipt::new();
        let baseline = create_unstarted_impl(
            &path,
            pin_image(&path).unwrap(),
            Some(clone_store(&a)),
            Some(&proof),
        )
        .unwrap();
        assert_eq!(enroller.call(&baseline, &proof, false, baseline.created), 0);
        finish_fixture(&baseline);
        for round in 1..=2 {
            let first_proof = FixtureReceipt::new();
            let second_proof = FixtureReceipt::new();
            let first = create_unstarted_impl(
                &path,
                pin_image(&path).unwrap(),
                Some(clone_store(&a)),
                Some(&first_proof),
            )
            .unwrap();
            let second = create_unstarted_impl(
                &path,
                pin_image(&path).unwrap(),
                Some(clone_store(&b)),
                Some(&second_proof),
            )
            .unwrap();
            assert_eq!(enroller.call(&first, &first_proof, true, first.created), 0);
            assert_eq!(
                enroller.call(&second, &second_proof, true, second.created),
                0
            );
            finish_fixture(&first);
            finish_fixture(&second);
            for store in [&a, &b] {
                assert_eq!(
                    std::fs::read_to_string(store.private_path.join("Roaming/rust-enrolled.txt"))
                        .unwrap(),
                    format!("{}:{round}", store.slot)
                );
                assert_eq!(
                    std::fs::read(
                        directory_path(&store.logical)
                            .unwrap()
                            .join("Roaming/rust-enrolled.txt")
                    )
                    .unwrap(),
                    b"HOST"
                );
            }
        }
    }

    fn test_slots() -> [SlotStorage; 2] {
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let logical = parent.join("host-view");
        std::fs::create_dir(&logical).unwrap();
        for leaf in ["Roaming", "Local"] {
            std::fs::create_dir(logical.join(leaf)).unwrap();
        }
        fresh_slots(
            open_directory(&logical, FILE_READ_ATTRIBUTES).unwrap(),
            &parent,
        )
        .unwrap()
    }

    fn read_config(child: &PendingEducationBootstrap) -> EducationConfig {
        let section = child.configuration.as_ref().unwrap().as_raw_handle() as HANDLE;
        let writable = unsafe {
            MapViewOfFile(
                section,
                FILE_MAP_WRITE,
                0,
                0,
                mem::size_of::<EducationConfig>(),
            )
        };
        assert!(writable.Value.is_null());
        let view = unsafe {
            MapViewOfFile(
                section,
                FILE_MAP_READ,
                0,
                0,
                mem::size_of::<EducationConfig>(),
            )
        };
        assert!(!view.Value.is_null());
        let config = unsafe { ptr::read(view.Value as *const EducationConfig) };
        assert_ne!(unsafe { UnmapViewOfFile(view) }, 0);
        config
    }

    #[test]
    fn configured_slots_publish_exact_child_bindings_and_exclude_an_ambient_handle() {
        use windows_sys::Win32::{
            Foundation::CompareObjectHandles, Security::SECURITY_ATTRIBUTES,
            System::Threading::CreateEventW,
        };
        let attributes = SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 1,
        };
        let ambient = unsafe { CreateEventW(&attributes, 1, 0, ptr::null()) };
        assert!(!ambient.is_null());
        let ambient = unsafe { OwnedHandle::from_raw_handle(ambient as _) };
        let [a, b] = test_slots();
        assert!(!same_image(
            &directory_id(&a.private).unwrap(),
            &directory_id(&b.private).unwrap()
        ));
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let mut first = create_unstarted(&path, pin_image(&path).unwrap(), Some(a)).unwrap();
        let mut second = create_unstarted(&path, pin_image(&path).unwrap(), Some(b)).unwrap();
        for (slot, child) in [(1, &first), (2, &second)] {
            let c = read_config(child);
            assert_eq!(
                (
                    c.pid,
                    ((c.created.dwHighDateTime as u64) << 32) | c.created.dwLowDateTime as u64
                ),
                child.identity().unwrap()
            );
            assert_eq!(c.slot, slot);
            let storage = child.storage.as_ref().unwrap();
            assert!(same_image(
                &c.private_id,
                &directory_id(&storage.private).unwrap()
            ));
            // An inherited handle keeps its value. A coincidental reused child value
            // is accepted only if it names a different kernel object.
            let mut copy = ptr::null_mut();
            if unsafe {
                DuplicateHandle(
                    child.process.as_raw_handle() as HANDLE,
                    ambient.as_raw_handle() as HANDLE,
                    GetCurrentProcess(),
                    &mut copy,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            } != 0
            {
                let copy = unsafe { OwnedHandle::from_raw_handle(copy as _) };
                assert_eq!(
                    unsafe {
                        CompareObjectHandles(
                            ambient.as_raw_handle() as HANDLE,
                            copy.as_raw_handle() as HANDLE,
                        )
                    },
                    0
                );
            }
        }
        first.cancel().unwrap();
        second.cancel().unwrap();
    }

    #[test]
    fn nested_backing_is_refused_without_reusing_or_deleting_the_logical_view() {
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let marker = parent.join("host-marker");
        std::fs::write(&marker, b"HOST").unwrap();
        let logical = open_directory(&parent, FILE_READ_ATTRIBUTES).unwrap();
        assert!(fresh_slots(logical, &parent).is_err());
        assert_eq!(std::fs::read(marker).unwrap(), b"HOST");
    }

    #[test]
    #[ignore = "Requires the explicitly built bootstrap-ordering.exe C++ reader"]
    fn cpp_receiver_consumes_rust_configuration_for_both_slots_across_relaunch() {
        use windows_sys::Win32::System::Threading::ResumeThread;
        let path = PathBuf::from(
            std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER")
                .expect("Set the trusted built C++ fixture path"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let [a, b] = test_slots();
        for round in 1..=2 {
            let first =
                create_unstarted(&path, pin_image(&path).unwrap(), Some(clone_store(&a))).unwrap();
            let second =
                create_unstarted(&path, pin_image(&path).unwrap(), Some(clone_store(&b))).unwrap();
            // Test-only resume: fixed trusted C++ fixture, no game target or injection.
            for child in [&first, &second] {
                assert_eq!(
                    unsafe { ResumeThread(child._thread.as_raw_handle() as HANDLE) },
                    1
                );
            }
            for child in [&first, &second] {
                assert_eq!(
                    unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
                    WAIT_OBJECT_0
                );
                let mut code = 1;
                assert_ne!(
                    unsafe {
                        GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code)
                    },
                    0
                );
                assert_eq!(code, 0);
            }
            for store in [&a, &b] {
                assert_eq!(
                    std::fs::read_to_string(store.private_path.join("Roaming/rust-bridge.txt"))
                        .unwrap(),
                    format!("{}:{round}", store.slot)
                );
            }
        }
    }

    #[test]
    fn original_child_is_pinned_unstarted_and_cancellable() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let image = pin_image(&path).unwrap();
        let id = image_id(&image).unwrap();
        let mut child = create_unstarted(&path, image, None).unwrap();
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
        let child = create_unstarted(&path, pin_image(&path).unwrap(), None).unwrap();
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
