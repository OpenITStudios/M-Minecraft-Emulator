//! Read-only private-root identity, ACL, ancestor and AppContainer folder evidence.
//! All process pins and token reads use the original Windows identity owner.
use super::{
    appcontainer_sid_from_token, invalid, live, pin, query_token, user_sid_from_token,
    GetAce, GetAclInformation, GetFinalPathNameByHandleW, GetSecurityInfo,
    IsValidAcl, LocalFree, OpenThreadToken, lstrlenW, ProcessKey,
};
use std::{io, mem::size_of, os::windows::{ffi::OsStringExt, fs::OpenOptionsExt, io::{AsRawHandle, FromRawHandle, OwnedHandle}}, ptr};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, HANDLE},
    Security::{GetLengthSid, IsValidSid, TOKEN_QUERY},
    Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_SHARE_READ, FILE_SHARE_WRITE},
    System::Threading::{GetCurrentProcess, GetCurrentThread, OpenProcessToken},
};

/// Only a minimal negative check. A non-null DACL is not a private ACL:
/// broad Everyone/Users ACEs, inherited rights, ancestor junctions and
/// the target game's actual file view remain unresolved.
pub(super) fn reject_invalid_directory_acl(owner_valid: bool, dacl_present: bool) -> io::Result<()> {
    if !owner_valid || !dacl_present {
        return Err(invalid("Private root owner/DACL is absent or invalid"));
    }
    Ok(())
}



#[link(name = "userenv")]
unsafe extern "system" {
    fn GetAppContainerFolderPath(
        appcontainer_sid: *const u16,
        folder_path: *mut *mut u16,
    ) -> i32;
}
#[link(name = "ole32")]
unsafe extern "system" {
    fn CoTaskMemFree(memory: *mut std::ffi::c_void);
}

// Convert *only* an authenticated TokenAppContainerSid from its canonical
// SID binary representation to S-1-15-2-... . No caller-supplied or
// account SID string is permitted. The OS provides the actual folder path.
pub(super) fn container_sid_text(sid: &[u8]) -> io::Result<String> {
    if sid.len() < 16 || sid[0] != 1 || !(2..=15).contains(&sid[1])
        || &sid[2..8] != [0, 0, 0, 0, 0, 15].as_slice()
        || sid.len() != 8 + 4 * usize::from(sid[1])
        || u32::from_le_bytes(sid[8..12].try_into().unwrap_or([0;4])) != 2
    {
        return Err(invalid("Not a canonical Windows AppContainer SID"));
    }
    let mut text = String::from("S-1-15");
    for chunk in sid[8..].chunks_exact(4) {
        let value = u32::from_le_bytes(chunk.try_into().unwrap_or([0; 4]));
        text.push('-');
        text.push_str(&value.to_string());
    }
    Ok(text)
}

// GetAppContainerFolderPath is CALLER-user-sensitive, not an operation
// performed on a process token. Refuse a thread impersonation token and
// a different Windows user before using it. Never report a false match
// when another user owns the running AppContainer.
fn no_thread_impersonation() -> io::Result<()> {
    const ERROR_NO_TOKEN: u32 = 1008;
    let mut raw: HANDLE = ptr::null_mut();
    let result = unsafe {
        OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut raw)
    };
    if result != 0 || !raw.is_null() {
        if !raw.is_null() {
            let _owned = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        }
        return Err(invalid("AppContainer folder lookup forbidden under impersonation"));
    }
    if unsafe { GetLastError() } != ERROR_NO_TOKEN {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn same_manager_windows_user(target_user: &[u8]) -> io::Result<bool> {
    let mut raw: HANDLE = ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if raw.is_null() { return Err(invalid("Missing token handle")); }
    let token = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    let manager = user_sid_from_token(&token)?;
    Ok(manager == target_user)
}

fn appcontainer_local_app_data_path(sid: &[u8]) -> io::Result<std::path::PathBuf> {
    use std::ffi::OsString;
    let utf16: Vec<u16> = container_sid_text(sid)?
        .encode_utf16().chain(Some(0)).collect();
    let mut raw: *mut u16 = ptr::null_mut();
    let status = unsafe { GetAppContainerFolderPath(utf16.as_ptr(), &mut raw) };
    struct TaskPath(*mut u16);
    impl Drop for TaskPath {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe { CoTaskMemFree(self.0.cast()) };
            }
        }
    }
    let owned = TaskPath(raw);
    if status < 0 || owned.0.is_null() {
        return Err(invalid("Windows AppContainer LocalAppData path unavailable"));
    }
    // The OS API returns an allocated null-terminated UTF-16 path.
    // lstrlenW is used only on OS-owned, validated successful output.
    let count = unsafe { lstrlenW(owned.0) };
    if !(4..=32760).contains(&count) {
        return Err(invalid("Invalid OS-owned AppContainer folder path size"));
    }
    let path = unsafe { std::slice::from_raw_parts(owned.0, count as usize) };
    Ok(std::path::PathBuf::from(OsString::from_wide(path)))
}

fn os_container_local_app_data_match(
    process_user: &[u8],
    container: &[u8],
    opened_root: super::super::PrivateRootObjectIdentity,
) -> io::Result<Option<bool>> {
    no_thread_impersonation()?;
    if !same_manager_windows_user(process_user)? {
        return Ok(None);
    }
    let path = appcontainer_local_app_data_path(container)?;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_SHARE_WRITE,
    };
    // Existing OS-returned location only. No Create, no data reads and
    // no accounts/worlds; do not follow the final reparse point.
    let folder = std::fs::OpenOptions::new()
        .access_mode(0x80 | 0x20 | 0x0002_0000) // FILE_READ_ATTRIBUTES, FILE_TRAVERSE, READ_CONTROL
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    let expected = ancestor_directory_identity(&folder)?;
    if expected != opened_root {
        return Ok(Some(false));
    }
    inspect_opened_ancestor_chain(&folder, expected)?;
    Ok(Some(true))
}

// These well-known SID byte encodings are fixed Windows SID formats:
// S-1-1-0 (Everyone), S-1-5-11 (Authenticated Users),
// S-1-5-7 (Anonymous), S-1-5-32-545 (Builtin Users) and
// S-1-5-32-546 (Builtin Guests). Domain-specific groups are UNKNOWN.
pub(super) const SID_EVERYONE: &[u8] = &[1,1,0,0,0,0,0,1,0,0,0,0];
pub(super) const SID_AUTHENTICATED: &[u8] = &[1,1,0,0,0,0,0,5,11,0,0,0];
pub(super) const SID_ANONYMOUS: &[u8] = &[1,1,0,0,0,0,0,5,7,0,0,0];
pub(super) const SID_USERS: &[u8] = &[1,2,0,0,0,0,0,5,32,0,0,0,33,2,0,0];
pub(super) const SID_GUESTS: &[u8] = &[1,2,0,0,0,0,0,5,32,0,0,0,34,2,0,0];

/// Source- and input-bounded basic ACE classifier. Never grants an
/// effective-access conclusion: deny ACE ordering, owner privileges,
/// group membership, inherited access and callback/object ACEs require
/// a separately authenticated Windows authorization evaluation.
pub(super) fn reject_broad_directory_ace(ace: &[u8]) -> io::Result<()> {
    // Simple ACCESS_ALLOWED_ACE_TYPE=0 and ACCESS_DENIED_ACE_TYPE=1.
    // Callback, conditional and object ACE layouts differ: fail closed.
    if ace.len() < 20 || !matches!(ace[0], 0 | 1) {
        return Err(invalid("Unsupported or undersized private root ACE"));
    }
    let declared = u16::from_le_bytes([ace[2], ace[3]]) as usize;
    let subauthorities = usize::from(ace[9]);
    // The SID header is 8 bytes after the eight-byte ACE header/mask.
    // A basic SID is revision 1 with no more than 15 subauthorities.
    if declared != ace.len() || ace[8] != 1 || subauthorities > 15
        || 16 + 4 * subauthorities != ace.len()
    {
        return Err(invalid("Malformed basic private root ACE SID"));
    }
    let sid = &ace[8..];
    let broad = [SID_EVERYONE, SID_AUTHENTICATED, SID_ANONYMOUS, SID_USERS, SID_GUESTS]
        .contains(&sid);
    let mask = u32::from_le_bytes([ace[4], ace[5], ace[6], ace[7]]);
    // Conservative policy: any nonzero allow ACE for these wide
    // principals is unsuitable for a private manager-owned root,
    // including read-only and inheritable ACEs for child content.
    if ace[0] == 0 && broad && mask != 0 {
        return Err(invalid("Private root DACL grants access to a broad group"));
    }
    Ok(())
}

#[repr(C)]
struct AclSizeInformation {
    ace_count: u32,
    acl_bytes_in_use: u32,
    acl_bytes_free: u32,
}

/// The returned ACL belongs to the still-live GetSecurityInfo descriptor.
/// ACL size and iteration are capped to avoid unbounded native work.
// Syntactic ACE reference only; token group state and ACE ordering
// can make a matching entry irrelevant to effective access.
pub(super) fn basic_allow_for_sid(ace: &[u8], sid: &[u8]) -> bool {
    ace[0] == 0
        && u32::from_le_bytes([ace[4], ace[5], ace[6], ace[7]]) != 0
        && &ace[8..] == sid
}

fn inspect_directory_acl(
    acl: *mut std::ffi::c_void,
    expected_container: Option<&[u8]>,
) -> io::Result<bool> {
    if acl.is_null() || unsafe { IsValidAcl(acl) } == 0 {
        return Err(invalid("Private root DACL is missing or invalid"));
    }
    let mut info = AclSizeInformation {
        ace_count: 0,
        acl_bytes_in_use: 0,
        acl_bytes_free: 0,
    };
    // ACL_INFORMATION_CLASS::AclSizeInformation = 2.
    if unsafe {
        GetAclInformation(
            acl,
            (&mut info as *mut AclSizeInformation).cast(),
            size_of::<AclSizeInformation>() as u32,
            2,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    // Empty DACL grants nobody access and is unusable as an ordinary
    // session profile; an oversized/complex DACL must be manually reviewed.
    if !(1..=64).contains(&info.ace_count)
        || !(8..=8192).contains(&info.acl_bytes_in_use)
    {
        return Err(invalid("Private root ACL count or size unsupported"));
    }
    let mut container_allow = false;
    for index in 0..info.ace_count {
        let mut raw: *mut std::ffi::c_void = ptr::null_mut();
        if unsafe { GetAce(acl, index, &mut raw) } == 0 || raw.is_null() {
            return Err(invalid("Private root ACL entry unavailable"));
        }
        // GetAce supplies a pointer into a validated ACL allocation.
        let base = raw.cast::<u8>();
        let len = unsafe {
            u16::from_le_bytes([*base.add(2), *base.add(3)]) as usize
        };
        if !(20..=1024).contains(&len) || len > info.acl_bytes_in_use as usize {
            return Err(invalid("Private root ACL entry length invalid"));
        }
        let entry = unsafe { std::slice::from_raw_parts(base, len) };
        reject_broad_directory_ace(entry)?;
        if expected_container.is_some_and(|sid| basic_allow_for_sid(entry, sid)) {
            container_allow = true;
        }
    }
    Ok(container_allow)
}

fn inspect_directory_security(
    opened: &std::fs::File,
    expected_user: Option<&[u8]>,
    expected_container: Option<&[u8]>,
) -> io::Result<super::super::PendingRootSidRelations> {
    struct Descriptor(*mut std::ffi::c_void);
    impl Drop for Descriptor {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // GetSecurityInfo memory is always freed with LocalFree,
                // never CloseHandle/FreeLibrary or Rust's allocator.
                let _ = unsafe { LocalFree(self.0) };
            }
        }
    }
    let mut owner: *mut std::ffi::c_void = ptr::null_mut();
    let mut dacl: *mut std::ffi::c_void = ptr::null_mut();
    let mut raw_descriptor: *mut std::ffi::c_void = ptr::null_mut();
    let status = unsafe {
        GetSecurityInfo(
            opened.as_raw_handle(),
            1, // SE_FILE_OBJECT
            1 | 4, // OWNER_SECURITY_INFORMATION | DACL_SECURITY_INFORMATION
            &mut owner,
            ptr::null_mut(),
            &mut dacl,
            ptr::null_mut(),
            &mut raw_descriptor,
        )
    };
    let _descriptor = Descriptor(raw_descriptor);
    if status != 0 {
        return Err(io::Error::from_raw_os_error(status as i32));
    }
    if raw_descriptor.is_null() {
        return Err(invalid("Windows returned no private root security descriptor"));
    }
    let owner_valid = !owner.is_null() && unsafe { IsValidSid(owner) } != 0;
    reject_invalid_directory_acl(owner_valid, !dacl.is_null())?;
    let len = unsafe { GetLengthSid(owner) } as usize;
    if !(8..=68).contains(&len) {
        return Err(invalid("Private root owner SID length invalid"));
    }
    let owner_sid = unsafe { std::slice::from_raw_parts(owner.cast::<u8>(), len) };
    let allow = inspect_directory_acl(dacl, expected_container)?;
    Ok(super::super::PendingRootSidRelations {
        owner_matches_process_user: expected_user.is_some_and(|sid| sid == owner_sid),
        owner_matches_container: expected_container.is_some_and(|sid| sid == owner_sid),
        container_allow_ace_observed: allow,
        os_container_local_app_data_match: None,
    })
}

/// Inspect one already-open directory object and its bounded ACL metadata.
/// Refuse known broad grants and unsupported ACE structures, but this is
/// NOT an effective Windows permission check or proof of account ownership.

// Strict read-only inspection of the prefixes of a previously opened
// directory. Windows returned the path, not a user/Tauri input.
// A path-only check is never a race-free root access capability.
pub(super) fn opened_dos_prefixes(path: &str) -> io::Result<Vec<String>> {
    let value = path.strip_prefix(r"\\?\")
        .ok_or_else(|| invalid("Unsupported root path prefix"))?;
    let bytes = value.as_bytes();
    if bytes.len() < 4 || !bytes[0].is_ascii_alphabetic()
        || bytes[1] != b':' || bytes[2] != b'\\'
        || value.contains('/') || value.contains('\0')
    {
        return Err(invalid("Root is not an absolute local DOS drive path"));
    }
    let rest = &value[3..];
    if rest.is_empty() || rest.ends_with('\\') {
        return Err(invalid("Root has no supported directory component"));
    }
    let parts: Vec<&str> = rest.split('\\').collect();
    if parts.len() > 32 {
        return Err(invalid("Root ancestor depth exceeds safety limit"));
    }
    let mut current = format!("\\\\?\\{}:\\", &value[..1]);
    let mut prefixes = vec![current.clone()];
    for part in parts {
        if part.is_empty() || matches!(part, "." | "..")
            || part.ends_with('.') || part.ends_with(' ')
            || part.chars().any(|ch| ch.is_control()
                || matches!(ch, ':' | '*' | '?' | '"' | '<' | '>' | '|'))
        {
            return Err(invalid("Ambiguous root ancestor component"));
        }
        current.push_str(part);
        prefixes.push(current.clone());
        current.push('\\');
    }
    Ok(prefixes)
}

pub(super) fn opened_root_name(file: &std::fs::File) -> io::Result<String> {
    const FILE_NAME_OPENED: u32 = 8;
    let handle = file.as_raw_handle();
    let needed = unsafe {
        GetFinalPathNameByHandleW(handle, ptr::null_mut(), 0, FILE_NAME_OPENED)
    };
    if !(5..=32760).contains(&needed) {
        return Err(invalid("Opened root path unavailable or oversized"));
    }
    let mut name = vec![0u16; needed as usize];
    let actual = unsafe {
        GetFinalPathNameByHandleW(handle, name.as_mut_ptr(), needed, FILE_NAME_OPENED)
    };
    if actual == 0 || actual >= needed || name[..actual as usize].contains(&0) {
        return Err(invalid("Opened root name could not be pinned"));
    }
    String::from_utf16(&name[..actual as usize])
        .map_err(|_| invalid("Invalid opened root UTF-16 name"))
}

pub(super) fn ancestor_directory_identity(file: &std::fs::File)
    -> io::Result<super::super::PrivateRootObjectIdentity>
{
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(invalid("Ancestor is a file or reparse point"));
    }
    let file_index = (u64::from(info.nFileIndexHigh) << 32)
        | u64::from(info.nFileIndexLow);
    if file_index == 0 {
        return Err(invalid("Ancestor has no usable file index"));
    }
    Ok(super::super::PrivateRootObjectIdentity {
        volume: info.dwVolumeSerialNumber,
        file_index,
    })
}


// User-mode ntdll NtOpenFile, exactly the winternl.h six-argument ABI.
// RootDirectory points to an OPENED directory HANDLE, not a DOS path.
// These plain C-layout types are deliberately kept adjacent to the FFI;
// IO_STATUS_BLOCK's first union occupies one pointer-sized slot.
#[repr(C)]
struct NtUnicodeString {
    length: u16,
    maximum_length: u16,
    buffer: *mut u16,
}

#[repr(C)]
struct NtObjectAttributes {
    length: u32,
    root_directory: HANDLE,
    object_name: *mut NtUnicodeString,
    attributes: u32,
    security_descriptor: *mut std::ffi::c_void,
    security_quality_of_service: *mut std::ffi::c_void,
}

#[repr(C)]
struct NtIoStatusBlock {
    status_or_pointer: usize,
    information: usize,
}

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtOpenFile(
        file_handle: *mut HANDLE,
        desired_access: u32,
        object_attributes: *mut NtObjectAttributes,
        io_status_block: *mut NtIoStatusBlock,
        share_access: u32,
        open_options: u32,
    ) -> i32;
}

/// Open exactly ONE existing child component relative to an OS directory
/// handle. This neither creates a file nor permits read/write data I/O.
/// The parent stays open at least until the child has been inspected.
/// Final-component reparse targets are opened AS reparse objects, never
/// followed; a later metadata check rejects them.
pub(super) fn open_relative_directory_metadata(
    parent: &std::fs::File,
    component: &str,
) -> io::Result<std::fs::File> {
    use std::os::windows::io::FromRawHandle;
    const OBJ_CASE_INSENSITIVE: u32 = 0x40;
    const FILE_READ_ATTRIBUTES: u32 = 0x80;
    const FILE_TRAVERSE: u32 = 0x20;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x20;
    const FILE_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    const FILE_SHARE_READ: u32 = 1;
    const FILE_SHARE_WRITE: u32 = 2;

    // Require single, unambiguous components even if a future caller
    // bypasses the already validated opened-name parser.
    if component.is_empty() || component == "." || component == ".."
        || component.ends_with('.')
        || component.ends_with(' ') || component.chars().any(|ch|
            ch.is_control() || matches!(
                ch, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            ))
    {
        return Err(invalid("Invalid relative directory component"));
    }
    let mut wide: Vec<u16> = component.encode_utf16().collect();
    if wide.is_empty() || wide.len() > 255 {
        return Err(invalid("Relative directory name too long"));
    }
    let bytes = wide.len() * size_of::<u16>();
    let length = u16::try_from(bytes)
        .map_err(|_| invalid("Directory name exceeds NT length"))?;
    wide.push(0);
    let mut unicode = NtUnicodeString {
        length,
        maximum_length: length + 2,
        buffer: wide.as_mut_ptr(),
    };
    let mut attributes = NtObjectAttributes {
        length: size_of::<NtObjectAttributes>() as u32,
        root_directory: parent.as_raw_handle(),
        object_name: &mut unicode,
        attributes: OBJ_CASE_INSENSITIVE,
        security_descriptor: ptr::null_mut(),
        security_quality_of_service: ptr::null_mut(),
    };
    let mut iosb = NtIoStatusBlock {
        status_or_pointer: 0,
        information: 0,
    };
    let mut raw: HANDLE = ptr::null_mut();
    let status = unsafe {
        NtOpenFile(
            &mut raw,
            FILE_READ_ATTRIBUTES | FILE_TRAVERSE | SYNCHRONIZE,
            &mut attributes,
            &mut iosb,
            FILE_SHARE_READ | FILE_SHARE_WRITE, // no FILE_SHARE_DELETE
            FILE_SYNCHRONOUS_IO_NONALERT | FILE_OPEN_REPARSE_POINT,
        )
    };
    if status != 0 || raw.is_null() {
        if !raw.is_null() {
            let _ = unsafe { CloseHandle(raw) };
        }
        return Err(invalid("Failed to open child relative to a directory HANDLE"));
    }
    // Owned NT file handles are ordinary Windows file HANDLEs. File::Drop
    // uses CloseHandle; never close parent handles while children are in use.
    Ok(unsafe { std::fs::File::from_raw_handle(raw) })
}

// Open only the volume root by DOS name. Then open each descendant by
// ONE component relative to a HELD parent HANDLE (NtOpenFile.RootDirectory).
// Holding all parent handles without FILE_SHARE_DELETE reduces rename/swap
// races. This cannot prove that the initial drive/root belongs to a
// trusted profile owner; the result remains a pending diagnostic.
fn inspect_opened_ancestor_chain(
    opened: &std::fs::File,
    expected: super::super::PrivateRootObjectIdentity,
) -> io::Result<()> {
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };
    let original_name = opened_root_name(opened)?;
    let prefixes = opened_dos_prefixes(&original_name)?;
    // Only the volume root uses a pathname. The remaining components
    // are opened relative to the preceding validated directory HANDLE.
    // Excluding FILE_SHARE_DELETE holds each ancestor against rename.
    let drive = std::fs::OpenOptions::new()
        .access_mode(0x80 | 0x20 | 0x0010_0000)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(std::path::Path::new(&prefixes[0]))?;
    let start = ancestor_directory_identity(&drive)?;
    if start.volume != expected.volume {
        return Err(invalid("Root drive volume mismatches opened directory"));
    }
    let mut held = vec![drive];
    for prefix in prefixes.iter().skip(1) {
        let component = prefix.rsplit('\\').next()
            .ok_or_else(|| invalid("Missing root ancestor component"))?;
        let child = open_relative_directory_metadata(
            held.last().expect("root directory is always held"), component,
        )?;
        let identity = ancestor_directory_identity(&child)?;
        if identity.volume != expected.volume {
            return Err(invalid("Ancestor crossed a volume boundary"));
        }
        held.push(child); // all ancestors stay open until final checks
    }
    if ancestor_directory_identity(
        held.last().expect("final root directory must be held"),
    )? != expected
    {
        return Err(invalid("Final ancestor is not original root object"));
    }
    if ancestor_directory_identity(opened)? != expected
        || opened_root_name(opened)? != original_name
    {
        return Err(invalid("Root changed during handle-relative inspection"));
    }
    Ok(())
}

fn inspect_directory_with_sids(
    opened: &std::fs::File,
    user: Option<&[u8]>,
    container: Option<&[u8]>,
) -> io::Result<(super::super::PrivateRootObjectIdentity, super::super::PendingRootSidRelations)> {
    fn snapshot(file: &std::fs::File) -> io::Result<super::super::PrivateRootObjectIdentity> {
        let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
        if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
            return Err(io::Error::last_os_error());
        }
        // Must have been opened with FILE_FLAG_OPEN_REPARSE_POINT.
        // Checking a followed target alone cannot prove a safe ancestor.
        if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
            || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
        {
            return Err(invalid("Private root handle is not an ordinary directory"));
        }
        let index = (u64::from(info.nFileIndexHigh) << 32)
            | u64::from(info.nFileIndexLow);
        if index == 0 {
            return Err(invalid("Private root directory identity is unavailable"));
        }
        Ok(super::super::PrivateRootObjectIdentity {
            volume: info.dwVolumeSerialNumber,
            file_index: index,
        })
    }

    let before = snapshot(opened)?;
    // READ_CONTROL on the already-opened object, no path traversal and
    // no read of user account files, world data, or registry keys.
    let relations = inspect_directory_security(opened, user, container)?;
    if snapshot(opened)? != before {
        return Err(invalid("Directory changed during ACL observation"));
    }
    Ok((before, relations))
}

pub(in crate::engine::instance_identity) fn inspect_directory_handle(
    opened: &std::fs::File,
) -> io::Result<super::super::PrivateRootObjectIdentity> {
    inspect_directory_with_sids(opened, None, None).map(|(identity, _)| identity)
}

/// Pin the exact process and its read-only token while inspecting one
/// already-opened directory; never opens a profile by a caller path.
pub(in crate::engine::instance_identity) fn inspect_directory_for_process(
    opened: &std::fs::File,
    key: &ProcessKey,
) -> io::Result<(super::super::PrivateRootObjectIdentity, super::super::PendingRootSidRelations)> {
    let process = pin(key)?;
    let token = query_token(&process)?;
    // Only TokenUser and TokenAppContainerSid are needed here.
    // Avoid the unrelated GetUserProfileDirectoryW/session probes.
    let user = user_sid_from_token(&token)?;
    let container = appcontainer_sid_from_token(&token)?
        .ok_or_else(|| invalid("No AppContainer SID for scoped directory check"))?;
    let (directory, mut relations) =
        inspect_directory_with_sids(opened, Some(&user), Some(&container))?;
    inspect_opened_ancestor_chain(opened, directory)?;
    relations.os_container_local_app_data_match =
        os_container_local_app_data_match(&user, &container, directory)?;
    live(&process)?;
    Ok((directory, relations))
}

/// Exact identity and attributes of a fixture file HANDLE, not its path.
/// This is an *intra-reader stability check*, not proof that a child wrote
/// the file. The original fixture owner must enforce exclusive sharing
/// and non-following reparse-point open options before passing a handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::engine::instance_identity) struct FixtureRecordIdentity {
    volume: u32,
    file_index: u64,
    length: u64,
    modified: u64,
    attributes: u32,
}

pub(in crate::engine::instance_identity) fn fixture_record_identity(
    opened: &std::fs::File,
) -> io::Result<FixtureRecordIdentity> {
    use virtual_clients_core::independent_engine::SYNTHETIC_FRAME_BYTES;

    // Windows BY_HANDLE_FILE_INFORMATION has no uninitialized pointers;
    // zero initialization is the documented starting state for this FFI.
    let mut value: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(opened.as_raw_handle(), &mut value) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let length = (u64::from(value.nFileSizeHigh) << 32)
        | u64::from(value.nFileSizeLow);
    if length != SYNTHETIC_FRAME_BYTES as u64
        || value.dwFileAttributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT) != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not a regular 8-byte fixture record",
        ));
    }
    Ok(FixtureRecordIdentity {
        volume: value.dwVolumeSerialNumber,
        file_index: (u64::from(value.nFileIndexHigh) << 32)
            | u64::from(value.nFileIndexLow),
        length,
        modified: (u64::from(value.ftLastWriteTime.dwHighDateTime) << 32)
            | u64::from(value.ftLastWriteTime.dwLowDateTime),
        attributes: value.dwFileAttributes,
    })
}
