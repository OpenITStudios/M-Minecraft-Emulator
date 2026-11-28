//! Original-child shared configuration mapping and exact private storage binding.
//! The parent bootstrap retains publication order and cancellation responsibility.
use super::{directory_id, interactive_session, PendingEducationBootstrap, SlotStorage};
use std::{io, mem, os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle}, ptr};
use windows_sys::Win32::{
    Foundation::{DuplicateHandle, SetHandleInformation, DUPLICATE_SAME_ACCESS, FILETIME, HANDLE, HANDLE_FLAG_INHERIT, INVALID_HANDLE_VALUE},
    Storage::FileSystem::FILE_ID_INFO,
    System::{Memory::{CreateFileMappingW, MapViewOfFile, UnmapViewOfFile, FILE_MAP_READ, FILE_MAP_WRITE, PAGE_READWRITE}, Registry::HKEY, Threading::GetCurrentProcess},
};

pub(super) const CONFIG_ENV: &str = "MIVUBI_EDUCATION_BOOTSTRAP_SECTION";

// Must match the private education-config.h, not a public product wire type.
#[repr(C)]
pub(super) struct EducationConfig {
    pub(super) magic: u32,
    pub(super) version: u32,
    pub(super) bytes: u32,
    pub(super) slot: u32,
    pub(super) pid: u32,
    pub(super) session: u32,
    pub(super) scope: u32,
    pub(super) singleton_caller_rva: u32,
    pub(super) created: FILETIME,
    pub(super) logical_root: HANDLE,
    pub(super) private_root: HANDLE,
    pub(super) registry_root: HKEY,
    pub(super) logical_id: FILE_ID_INFO,
    pub(super) private_id: FILE_ID_INFO,
    pub(super) user_key_path: [u16; 512],
}
const _: () = assert!(mem::size_of::<EducationConfig>() == 1136);
const _: () = assert!(mem::offset_of!(EducationConfig, logical_id) == 64);
const _: () = assert!(mem::offset_of!(EducationConfig, user_key_path) == 112);

pub(super) struct Configuration {
    pub(super) writable: OwnedHandle,
    pub(super) readonly: OwnedHandle,
}
impl Configuration {
    pub(super) fn create() -> io::Result<Self> {
        Self::create_sized(mem::size_of::<EducationConfig>() as u32)
    }
    pub(super) fn create_sized(bytes: u32) -> io::Result<Self> {
        let raw = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                ptr::null(),
                PAGE_READWRITE,
                0,
                bytes,
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

    pub(super) fn publish(
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
        config.version = 2;
        config.bytes = mem::size_of_val(&config) as u32;
        config.slot = storage.slot;
        config.pid = identity.0;
        config.session = interactive_session(identity.0)?;
        config.scope = if storage.singleton_caller_rva == 0 {
            1
        } else {
            2
        };
        config.singleton_caller_rva = storage.singleton_caller_rva;
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
