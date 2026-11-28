//! Bootstrap-owned directory and registry handles; no independent profile lifecycle.
//! Existing product/replay owners retain all admission and mutation ordering.
use super::{invalid, wide, MAX_EDUCATION_SLOTS};
use std::{ffi::{c_void, OsStr, OsString}, fs::{File, OpenOptions}, io, mem, os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, io::AsRawHandle}, path::{Path, PathBuf}, ptr};
use windows_sys::Win32::{
    Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS, HANDLE},
    Storage::FileSystem::{GetFileInformationByHandleEx, FILE_ID_INFO, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE, FileCaseSensitiveInfo, GetFinalPathNameByHandleW, FILE_CASE_SENSITIVE_INFO, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, VOLUME_NAME_DOS},
    System::{Com::{CoCreateGuid, CoTaskMemFree}, LibraryLoader::{GetModuleHandleW, GetProcAddress}, Registry::{RegCloseKey, RegCreateKeyExW, RegOpenCurrentUser, RegOpenKeyExW, HKEY, KEY_ALL_ACCESS, KEY_READ, REG_CREATED_NEW_KEY, REG_OPTION_NON_VOLATILE}},
    UI::Shell::SHGetKnownFolderPath,
};

pub(super) struct RegistryKey(pub(super) HKEY);
impl Drop for RegistryKey {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}

pub(super) struct SlotStorage {
    pub(super) slot: u32,
    pub(super) logical: File,
    pub(super) private: File,
    pub(super) temporary: [File; 2], // Retain ordinary Local and Local/Temp without delete sharing.
    pub(super) registry: RegistryKey,
    pub(super) user_key_path: [u16; 512],
    // Fresh data persists on drop; this is not another profile/readiness database.
    pub(super) private_path: PathBuf,
    pub(super) singleton_caller_rva: u32,
}

pub(super) fn directory_id(file: &File) -> io::Result<FILE_ID_INFO> {
    // Reuse the existing raw metadata owner; keep the bootstrap-specific
    // case-sensitivity and ordinary-directory policy here.
    let attributes = crate::engine::windows_file_identity::attributes(file)?;
    let id = crate::engine::windows_file_identity::file_id(file)?;
    let mut case: FILE_CASE_SENSITIVE_INFO = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _, FileCaseSensitiveInfo,
            &mut case as *mut _ as _, mem::size_of_val(&case) as u32,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if attributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || attributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        || case.Flags != 0
    {
        return Err(invalid("Unsupported bootstrap directory"));
    }
    Ok(id)
}

pub(super) fn open_directory(path: &Path, access: u32) -> io::Result<File> {
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

pub(super) fn directory_path(file: &File) -> io::Result<PathBuf> {
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

// Also used by the existing confirmed full-clear staging transaction.
pub(super) fn fresh_root(parent: &Path) -> io::Result<PathBuf> {
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

pub(super) fn registry_leaf(id: &FILE_ID_INFO, slot: u32) -> io::Result<String> {
    if !(1..=MAX_EDUCATION_SLOTS).contains(&slot) {
        return Err(invalid("Invalid Education profile slot"));
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
        char::from(b'A' + (slot - 1) as u8)
    ))
}

pub(super) fn open_existing_registry_store(id: &FILE_ID_INFO, slot: u32) -> io::Result<RegistryKey> {
    // Match user_registry_path's explicit user hive, not a process-wide cached HKCU mapping.
    let root = current_user_registry(KEY_READ)?;
    let path = wide(OsStr::new(&format!("Software\\MivubiEducationPilot\\{}", registry_leaf(id,slot)?)))?;
    let mut key = ptr::null_mut();
    let status = unsafe { RegOpenKeyExW(root.0,path.as_ptr(),0,KEY_ALL_ACCESS,&mut key) };
    if status != 0 {
        return Err(io::Error::new(io::Error::from_raw_os_error(status as i32).kind(),format!("Minecraft - {slot}: saved registry store cannot be opened (Windows error {status})")));
    }
    Ok(RegistryKey(key))
}

fn current_user_registry(access: u32) -> io::Result<RegistryKey> {
    let mut key = ptr::null_mut();
    let status = unsafe { RegOpenCurrentUser(access, &mut key) };
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    Ok(RegistryKey(key))
}

pub(super) fn user_registry_path() -> io::Result<[u16; 512]> {
    let key = current_user_registry(KEY_READ)?;
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

// The mode is explicit: existing-profile admission must never create Temp.
#[derive(Clone, Copy)]
pub(super) enum PrivateTemporaryMode { CreateIfMissing, ExistingOnly }

pub(super) fn open_private_temporary(private: &File, mode: PrivateTemporaryMode) -> io::Result<[File; 2]> {
    let access = FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY;
    let local = open_directory(&directory_path(private)?.join("Local"), access)?;
    let path = directory_path(&local)?.join("Temp");
    if matches!(mode, PrivateTemporaryMode::CreateIfMissing) {
        match std::fs::create_dir(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    let temporary = open_directory(&path, access)?;
    Ok([local, temporary]) // Never follow a reparse, clear a cache or copy a profile.
}

pub(super) fn create_private_slot(logical: &File, root: &Path, slot: u32) -> io::Result<SlotStorage> {
    create_private_slot_at(logical, &root.join(format!("MC{slot:02}")), slot)
}

pub(super) fn create_private_slot_at(logical: &File, private_path: &Path, slot: u32) -> io::Result<SlotStorage> {
    if !(1..=MAX_EDUCATION_SLOTS).contains(&slot) {
        return Err(invalid("Invalid Education slot"));
    }
    let user_key_path = user_registry_path()?;
    std::fs::create_dir(&private_path)?;
    let private = open_directory(&private_path, FILE_READ_ATTRIBUTES | FILE_LIST_DIRECTORY)?;
    // Match the DLL's canonical spelling, including expansion of Windows 8.3 aliases.
    let private_path = directory_path(&private)?;
    for leaf in ["Roaming", "Local"] {
        std::fs::create_dir(private_path.join(leaf))?;
    }
    let id = directory_id(&private)?;
    let root = current_user_registry(KEY_ALL_ACCESS)?;
    let path = wide(OsStr::new(&format!(
        "Software\\MivubiEducationPilot\\{}",
        registry_leaf(&id, slot)?
    )))?;
    let mut key = ptr::null_mut();
    let mut disposition = 0;
    let status = unsafe {
        RegCreateKeyExW(
            root.0,
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
    Ok(SlotStorage {
        slot,
        logical: logical.try_clone()?,
        temporary: open_private_temporary(&private, PrivateTemporaryMode::CreateIfMissing)?,
        private,
        registry,
        user_key_path,
        private_path,
        singleton_caller_rva: 0,
    })
}

#[cfg(test)]
pub(super) fn fresh_slots(logical: File, parent: &Path) -> io::Result<[SlotStorage; 2]> {
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
    let mut slots = Vec::with_capacity(2);
    for slot in 1..=2 {
        slots.push(create_private_slot(&logical, &root, slot)?);
    }
    slots
        .try_into()
        .map_err(|_| invalid("Missing private slot"))
}

pub(super) fn known_folder(id: &windows_sys::core::GUID) -> io::Result<PathBuf> {
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

impl SlotStorage {
    pub(super) fn duplicate(&self) -> io::Result<Self> {
        let mut key = ptr::null_mut();
        if unsafe {
            DuplicateHandle(
                windows_sys::Win32::System::Threading::GetCurrentProcess(),
                self.registry.0,
                windows_sys::Win32::System::Threading::GetCurrentProcess(),
                &mut key,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let registry = RegistryKey(key);
        Ok(Self {
            slot: self.slot,
            logical: self.logical.try_clone()?,
            private: self.private.try_clone()?,
            temporary: [
                self.temporary[0].try_clone()?,
                self.temporary[1].try_clone()?,
            ],
            registry,
            user_key_path: self.user_key_path,
            private_path: self.private_path.clone(),
            singleton_caller_rva: self.singleton_caller_rva,
        })
    }
}
