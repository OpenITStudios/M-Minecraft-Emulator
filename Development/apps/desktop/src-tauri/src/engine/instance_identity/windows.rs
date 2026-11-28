//! Shared original-process pin, Windows token/context observation and domain routing.
use super::{
    AppContainerComparison, ContextComparison, ProcessContainment, ProcessKey,
    ObservedProcessPackageForm, ObservedPackageFamilyRelation,
    classify_package_probe, classify_package_family_pair, finish_file_signature_review,
};
use std::{io, mem::size_of, os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle}, ptr};
use std::os::windows::ffi::OsStringExt;
use windows_sys::Win32::{
    Foundation::{
        GetLastError, ERROR_INSUFFICIENT_BUFFER, HANDLE, WAIT_TIMEOUT,
    },
    Globalization::{CompareStringOrdinal, CSTR_EQUAL},
    Security::{
        GetLengthSid, GetTokenInformation, IsValidSid, TokenAppContainerSid,
        TokenIsAppContainer, TokenSessionId, TokenUser, TOKEN_APPCONTAINER_INFORMATION,
        TOKEN_QUERY, TOKEN_USER, TokenVirtualizationAllowed,
        TokenVirtualizationEnabled,
    },
    System::Threading::{
        OpenProcess,
        OpenProcessToken, WaitForSingleObject,
        PROCESS_QUERY_LIMITED_INFORMATION,
        PROCESS_SYNCHRONIZE,
    },
    UI::Shell::GetUserProfileDirectoryW,
};

// Windows-sys 0.61 does not project FreeLibrary in LibraryLoader.
// Keep this one exact kernel32 ABI local to the loaded-system-DLL owner.
// No calls from DllMain, no borrowed GetModuleHandle handles.
#[link(name = "kernel32")]
unsafe extern "system" {
    fn FreeLibrary(h_lib_module: *mut std::ffi::c_void) -> i32;
    fn LocalFree(h_mem: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
    fn GetFinalPathNameByHandleW(
        handle: HANDLE, buffer: *mut u16, capacity: u32, flags: u32,
    ) -> u32;
    fn lstrlenW(text: *const u16) -> i32;
    fn GetPackageFullName(
        process: HANDLE, name_length: *mut u32, name: *mut u16,
    ) -> i32;
    fn GetPackageFamilyName(
        process: HANDLE, name_length: *mut u32, name: *mut u16,
    ) -> i32;
}

// GetSecurityInfo returns a DWORD status (not GetLastError), with
// owner/DACL pointers borrowed from the LocalFree-owned descriptor.
// SE_FILE_OBJECT=1, OWNER_SECURITY_INFORMATION=1, DACL_SECURITY_INFORMATION=4.
// The handle must already carry READ_CONTROL; never enable privileges.
#[link(name = "advapi32")]
unsafe extern "system" {
    fn GetSecurityInfo(
        handle: HANDLE,
        object_type: i32,
        security_information: u32,
        owner_sid: *mut *mut std::ffi::c_void,
        group_sid: *mut *mut std::ffi::c_void,
        dacl: *mut *mut std::ffi::c_void,
        sacl: *mut *mut std::ffi::c_void,
        security_descriptor: *mut *mut std::ffi::c_void,
    ) -> u32;
    fn IsValidAcl(acl: *mut std::ffi::c_void) -> i32;
    fn GetAclInformation(
        acl: *mut std::ffi::c_void,
        information: *mut std::ffi::c_void,
        size: u32,
        information_class: i32,
    ) -> i32;
    fn GetAce(
        acl: *mut std::ffi::c_void,
        index: u32,
        ace: *mut *mut std::ffi::c_void,
    ) -> i32;
    fn OpenThreadToken(
        thread: HANDLE,
        desired_access: u32,
        open_as_self: i32,
        token: *mut HANDLE,
    ) -> i32;
}

mod private_root_identity;
pub(super) use self::private_root_identity::{inspect_directory_handle, inspect_directory_for_process, fixture_record_identity};
#[cfg(test)]
use self::private_root_identity::{reject_invalid_directory_acl, container_sid_text, reject_broad_directory_ace, basic_allow_for_sid, opened_dos_prefixes, opened_root_name, ancestor_directory_identity, open_relative_directory_metadata, SID_EVERYONE, SID_AUTHENTICATED, SID_ANONYMOUS, SID_USERS, SID_GUESTS};
mod package_identity;
pub(super) use self::package_identity::{inspect_package_form, compare_package_families};
#[cfg(test)]
use self::package_identity::package_family_name;
mod image_signature;
pub(super) use self::image_signature::inspect_file_authenticode;

fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn live(process: &OwnedHandle) -> io::Result<()> {
    match unsafe { WaitForSingleObject(process.as_raw_handle() as _, 0) } {
        WAIT_TIMEOUT => Ok(()),
        0 => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Observed process has exited",
        )),
        _ => Err(io::Error::last_os_error()),
    }
}
fn created(process: &OwnedHandle) -> io::Result<u64> {
    crate::engine::windows_process_time::creation_filetime(process.as_raw_handle() as _)
}
fn pin(key: &ProcessKey) -> io::Result<OwnedHandle> {
    let expected = key
        .created_file_time
        .parse::<u64>()
        .map_err(|_| invalid("Invalid process creation time"))?;
    if key.pid == 0 || expected == 0 {
        return Err(invalid("Invalid process identity"));
    }
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            key.pid,
        )
    };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let process = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    if created(&process)? != expected {
        return Err(invalid("Process creation time changed"));
    }
    live(&process)?;
    Ok(process)
}

// Only legacy UAC compatibility metadata, never a registry hive query.
// Pin the exact PID/FILTIME process handle so no reused PID is accepted.
pub(super) fn legacy_token_virtualization(
    key: &ProcessKey,
) -> io::Result<(bool, bool)> {
    let process = pin(key)?;
    let mut raw = ptr::null_mut();
    if unsafe { OpenProcessToken(process.as_raw_handle() as _, TOKEN_QUERY, &mut raw) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if raw.is_null() { return Err(invalid("Missing token handle")); }
    let token = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    let query = |kind| -> io::Result<bool> {
        let mut value = 0u32;
        let mut length = 0u32;
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle() as _,
                kind,
                (&mut value as *mut u32).cast(),
                size_of::<u32>() as u32,
                &mut length,
            )
        } == 0 {
            return Err(io::Error::last_os_error());
        }
        if length != size_of::<u32>() as u32 || value > 1 {
            return Err(invalid("Invalid legacy virtualization token flag"));
        }
        Ok(value != 0)
    };
    let allowed = query(TokenVirtualizationAllowed)?;
    let enabled = query(TokenVirtualizationEnabled)?;
    live(&process)?;
    if enabled && !allowed {
        return Err(invalid("Inconsistent legacy virtualization token flags"));
    }
    Ok((allowed, enabled))
}

struct Context {
    user: Vec<u8>,
    profile: Vec<u16>,
    session: u32,
}
/// One read-only token handle per process observation. The same token
/// is reused for user/profile/session and AppContainer SID in compare_pair.
fn query_token(process: &OwnedHandle) -> io::Result<OwnedHandle> {
    let mut raw = ptr::null_mut();
    if unsafe { OpenProcessToken(process.as_raw_handle() as _, TOKEN_QUERY, &mut raw) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if raw.is_null() { return Err(invalid("Missing token handle")); }
    Ok(unsafe { OwnedHandle::from_raw_handle(raw as _) })
}

// Read the user SID only. A private-root comparison does not need a
// profile path or a Windows session query.
fn user_sid_from_token(token: &OwnedHandle) -> io::Result<Vec<u8>> {
    let mut needed = 0;
    let first =
        unsafe { GetTokenInformation(token.as_raw_handle() as _, TokenUser, ptr::null_mut(), 0, &mut needed) };
    if first != 0
        || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
        || needed < size_of::<TOKEN_USER>() as u32
        || needed > 65536
    {
        return Err(invalid("Windows token user size is unavailable"));
    }
    // Token structures contain pointers and require native alignment.
    let mut buffer = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle() as _,
            TokenUser,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let sid = unsafe { (*(buffer.as_ptr().cast::<TOKEN_USER>())).User.Sid };
    if unsafe { IsValidSid(sid) } == 0 {
        return Err(invalid("Invalid Windows token user"));
    }
    let length = unsafe { GetLengthSid(sid) } as usize;
    if !(8..=68).contains(&length) {
        return Err(invalid("Invalid Windows token user length"));
    }
    Ok(unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), length) }.to_vec())
}

fn context_from_token(token: &OwnedHandle) -> io::Result<Context> {
    let user = user_sid_from_token(token)?;
    let mut needed = 0u32;
    let mut session = 0u32;
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle() as _,
            TokenSessionId,
            (&mut session as *mut u32).cast(),
            size_of::<u32>() as u32,
            &mut needed,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    let mut count = 0;
    let first = unsafe { GetUserProfileDirectoryW(token.as_raw_handle() as _, ptr::null_mut(), &mut count) };
    if first != 0
        || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
        || !(2..=32768).contains(&count)
    {
        return Err(invalid("Windows profile directory is unavailable"));
    }
    let mut profile = vec![0u16; count as usize];
    if unsafe { GetUserProfileDirectoryW(token.as_raw_handle() as _, profile.as_mut_ptr(), &mut count) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let end = profile
        .iter()
        .position(|&c| c == 0)
        .ok_or_else(|| invalid("Invalid Windows profile directory"))?;
    if end == 0 {
        return Err(invalid("Empty Windows profile directory"));
    }
    profile.truncate(end);
    Ok(Context {
        user,
        profile,
        session,
    })
}
fn context(process: &OwnedHandle) -> io::Result<Context> {
    let token = query_token(process)?;
    context_from_token(&token)
}

// An independent, bounded token inspection. Reuses the existing pinned
// process handles; never reads folders, broker stores or account data.
fn appcontainer_sid_from_token(token: &OwnedHandle) -> io::Result<Option<Vec<u8>>> {
    let mut flag = 0u32;
    let mut needed = 0u32;
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle() as _,
            TokenIsAppContainer,
            (&mut flag as *mut u32).cast(),
            size_of::<u32>() as u32,
            &mut needed,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if flag == 0 {
        return Ok(None);
    }
    let first = unsafe {
        GetTokenInformation(
            token.as_raw_handle() as _,
            TokenAppContainerSid,
            ptr::null_mut(),
            0,
            &mut needed,
        )
    };
    if first != 0
        || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
        || needed < size_of::<TOKEN_APPCONTAINER_INFORMATION>() as u32
        || needed > 65536
    {
        return Err(invalid("AppContainer token identity size is unavailable"));
    }
    // Use word-aligned storage because the returned structure has a pointer.
    let mut buffer = vec![0usize; (needed as usize).div_ceil(size_of::<usize>())];
    if unsafe {
        GetTokenInformation(
            token.as_raw_handle() as _,
            TokenAppContainerSid,
            buffer.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    let sid = unsafe {
        (*(buffer.as_ptr().cast::<TOKEN_APPCONTAINER_INFORMATION>())).TokenAppContainer
    };
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err(invalid("AppContainer token SID is invalid"));
    }
    let length = unsafe { GetLengthSid(sid) } as usize;
    if !(8..=68).contains(&length) {
        return Err(invalid("AppContainer token SID length is invalid"));
    }
    Ok(Some(unsafe {
        std::slice::from_raw_parts(sid.cast::<u8>(), length)
    }.to_vec()))
}

fn appcontainer_sid(process: &OwnedHandle) -> io::Result<Option<Vec<u8>>> {
    let token = query_token(process)?;
    appcontainer_sid_from_token(&token)
}

/// Reuse the same pinned handle/FILETIME check as the existing desktop
/// observer. The handle is retained until the final liveness check.
pub(super) fn inspect_exact(expected: &ProcessKey) -> io::Result<ProcessContainment> {
    let process = pin(expected)?;
    let containment = if appcontainer_sid(&process)?.is_some() {
        ProcessContainment::AppContainerTokenObserved
    } else {
        ProcessContainment::NoAppContainerToken
    };
    live(&process)?;
    Ok(containment)
}

// Inspecting the token and profile on already-pinned handles does not
// authenticate Microsoft/Minecraft accounts. It only returns booleans.
fn compare_appcontainer_values(a: Option<&[u8]>, b: Option<&[u8]>) -> AppContainerComparison {
    let both = a.is_some() && b.is_some();
    AppContainerComparison {
        both_appcontainers: both,
        distinct_container_identities: both && a != b,
    }
}

fn compare_context_values(a: &Context, b: &Context) -> io::Result<ContextComparison> {
    let profile_equal = unsafe {
        CompareStringOrdinal(
            a.profile.as_ptr(),
            a.profile.len() as i32,
            b.profile.as_ptr(),
            b.profile.len() as i32,
            1,
        )
    };
    if profile_equal == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(ContextComparison {
        same_windows_user: a.user == b.user,
        same_profile_directory: profile_equal == CSTR_EQUAL,
        same_windows_session: a.session == b.session,
    })
}

pub(super) fn compare_appcontainers(
    left: &ProcessKey,
    right: &ProcessKey,
) -> io::Result<AppContainerComparison> {
    let left_process = pin(left)?;
    let right_process = pin(right)?;
    let a = appcontainer_sid(&left_process)?;
    let b = appcontainer_sid(&right_process)?;
    live(&left_process)?;
    live(&right_process)?;
    Ok(compare_appcontainer_values(a.as_deref(), b.as_deref()))
}

pub(super) fn compare(left: &ProcessKey, right: &ProcessKey) -> io::Result<ContextComparison> {
    let left_process = pin(left)?;
    let right_process = pin(right)?;
    let a = context(&left_process)?;
    let b = context(&right_process)?;
    let comparison = compare_context_values(&a, &b)?;
    live(&left_process)?;
    live(&right_process)?;
    Ok(comparison)
}

/// One coherent pairwise read: two pinned process HANDLEs and two
/// read-only token HANDLEs (one for each process). Profile/session/SIDs
/// are read from those exact token handles, without reopening a token
/// for the AppContainer query. Final process liveness is checked.
pub(super) fn compare_pair(
    left: &ProcessKey,
    right: &ProcessKey,
) -> io::Result<(ContextComparison, AppContainerComparison)> {
    let left_process = pin(left)?;
    let right_process = pin(right)?;
    let first_token = query_token(&left_process)?;
    let second_token = query_token(&right_process)?;
    let first = context_from_token(&first_token)?;
    let second = context_from_token(&second_token)?;
    let first_container = appcontainer_sid_from_token(&first_token)?;
    let second_container = appcontainer_sid_from_token(&second_token)?;
    let context_result = compare_context_values(&first, &second)?;
    let container_result =
        compare_appcontainer_values(first_container.as_deref(), second_container.as_deref());
    live(&left_process)?;
    live(&right_process)?;
    Ok((context_result, container_result))
}

#[cfg(test)]
mod tests;
