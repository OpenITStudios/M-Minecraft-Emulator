//! Read-only native SCM service-SID and Windows logon-SID evidence.
//! No game launch, credential access, token modification or pipe policy
//! promotion. The opt-in pilot remains disabled by default.

use std::{
    ffi::c_void, fs::File, io, mem,
    os::windows::io::{AsRawHandle, FromRawHandle}, ptr,
};
use windows_sys::Win32::Storage::FileSystem::{
    GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
};
type H = *mut c_void;
const TOKEN_QUERY: u32 = 8;
const TOKEN_DUPLICATE: u32 = 2;
const TOKEN_IMPERSONATION: u32 = 2;
const SECURITY_IMPERSONATION: u32 = 2;
const TOKEN_GROUPS: u32 = 2;
const TOKEN_USER: u32 = 1;
const TOKEN_TYPE: u32 = 8;
const TOKEN_SESSION_ID: u32 = 12;
const TOKEN_PRIMARY: u32 = 1;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const SE_GROUP_ENABLED: u32 = 4;
const SE_GROUP_DENY_ONLY: u32 = 0x10;
const SE_GROUP_LOGON_ID: u32 = 0xC000_0000;
const MAX_GROUP_BYTES: u32 = 65_536;
const SERVICE_NAME: &str = "NT SERVICE\\MIVUBIIndependentHost";
// The service SID is an ENABLED GROUP of the SCM process token, NOT
// automatically the TokenUser or NTFS object owner. The native candidate
// below deliberately uses LocalSystem as owner/group and service SID as an
// explicit allowed DACL principal.
const LOCAL_SYSTEM_SID: [u8; 12] = [1, 1, 0, 0, 0, 0, 0, 5, 18, 0, 0, 0];
const ROOT_FILE_ALL_ACCESS: u32 = 0x001F_01FF;
const ROOT_DACL_PROTECTED: u16 = 0x1000;
const ROOT_FILE_OBJECT: u32 = 1;
const ROOT_SECURITY_FLAGS: u32 = 0x0001 | 0x0002 | 0x0004;
const ROOT_ACL_SIZE_CLASS: u32 = 2;
const ROOT_ACE_ALLOWED_TYPE: u8 = 0;
const ROOT_ACE_OBJECT_AND_CONTAINER_INHERIT: u8 = 0x03;
const ROOT_LIST: u32 = 0x0001;
const ROOT_ADD_FILE: u32 = 0x0002;
const ROOT_ADD_DIRECTORY: u32 = 0x0004;
const ROOT_READ_EA: u32 = 0x0008;
const ROOT_WRITE_EA: u32 = 0x0010;
const ROOT_TRAVERSE: u32 = 0x0020;
const ROOT_DELETE_CHILD: u32 = 0x0040;
const ROOT_READ_ATTRIBUTES: u32 = 0x0080;
const ROOT_WRITE_ATTRIBUTES: u32 = 0x0100;
const ROOT_DELETE: u32 = 0x0001_0000;
const ROOT_READ_CONTROL: u32 = 0x0002_0000;
const ROOT_WRITE_DAC: u32 = 0x0004_0000;
const ROOT_WRITE_OWNER: u32 = 0x0008_0000;
const ROOT_CHECK_RIGHTS: [u32; 13] = [
    ROOT_LIST, ROOT_ADD_FILE, ROOT_ADD_DIRECTORY,
    ROOT_READ_EA, ROOT_WRITE_EA, ROOT_TRAVERSE,
    ROOT_DELETE_CHILD, ROOT_READ_ATTRIBUTES, ROOT_WRITE_ATTRIBUTES,
    ROOT_DELETE, ROOT_READ_CONTROL, ROOT_WRITE_DAC, ROOT_WRITE_OWNER,
];
const ERROR_NO_TOKEN: u32 = 1008;

// Fixed service-managed metadata root name, NOT a Minecraft path. No IPC,
// CLI or end-user filename reaches the native create helper.
const SERVICE_ROOT_CHILD: &str = "mivubi-independent-root";
// Must already be installed by a separately reviewed, privileged installer.
// This SOURCE-ONLY module can only OPEN it; never create, adopt, repair
// or re-ACL an arbitrary directory masquerading as the installer parent.
const INSTALLER_PARENT_CHILD: &str = "MIVUBIIndependentHost";
// FOLDERID_ProgramData is a machine-wide FIXED Windows Known Folder,
// NOT caller-supplied %PROGRAMDATA%, HOME, environment or IPC path.
// {62AB5D82-FDC1-4DC3-A9DD-070D1D495D97} from knownfolderid.h.
const PROGRAM_DATA_KNOWN_FOLDER: NativeGuid = NativeGuid {
    data1: 0x62AB_5D82, data2: 0xFDC1, data3: 0x4DC3,
    data4: [0xA9, 0xDD, 0x07, 0x0D, 0x1D, 0x49, 0x5D, 0x97],
};
const COM_MULTITHREADED: u32 = 0;
const NT_FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
const NT_FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const WIN32_OPEN_EXISTING: u32 = 3;
const MAX_PROGRAMDATA_UTF16: usize = 1024;
const NT_FILE_CREATE: u32 = 2;
const NT_FILE_OPEN: u32 = 1;
const NT_FILE_CREATED: usize = 2;
const NT_FILE_READ_ATTRIBUTES: u32 = 0x0080;
const NT_FILE_ADD_SUBDIRECTORY: u32 = 0x0004;
const NT_SYNCHRONIZE: u32 = 0x0010_0000;
const NT_FILE_SHARE_READ: u32 = 0x0001;
const NT_FILE_SHARE_WRITE: u32 = 0x0002;
const NT_FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
const NT_OBJ_CASE_INSENSITIVE: u32 = 0x0000_0040;
const NT_FILE_DIRECTORY_FILE: u32 = 0x0000_0001;
const NT_FILE_SYNCHRONOUS_IO_NONALERT: u32 = 0x0000_0020;
const NT_FILE_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
const NT_STATUS_OBJECT_NAME_COLLISION: u32 = 0xC000_0035;
const NT_STATUS_OBJECT_NAME_NOT_FOUND: u32 = 0xC000_0034;
const NT_STATUS_OBJECT_PATH_NOT_FOUND: u32 = 0xC000_003A;
const NT_STATUS_ACCESS_DENIED: u32 = 0xC000_0022;

#[repr(C)]
#[derive(Clone, Copy)]
struct SidAndAttributes { sid: H, attributes: u32 }
#[repr(C)]
struct TokenGroups { count: u32, groups: [SidAndAttributes; 1] }
#[repr(C)]
struct TokenUser { user: SidAndAttributes }
#[repr(C)]
struct AclSizeInformation {
    ace_count: u32, bytes_in_use: u32, bytes_free: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RootFileIdentity { volume: u32, index: u64 }

/// The COM GUID memory representation is fixed in Windows guiddef.h.
#[repr(C)]
struct NativeGuid {
    data1: u32, data2: u16, data3: u16, data4: [u8; 8],
}

/// Native user-mode ABI used only for a fixed, single-component,
/// directory-HANDLE-relative create or open. All fields match winternl.h.
#[repr(C)]
struct NtUnicodeString {
    length: u16, maximum_length: u16, buffer: *mut u16,
}
#[repr(C)]
struct NtObjectAttributes {
    length: u32,
    root_directory: H,
    object_name: *mut NtUnicodeString,
    attributes: u32,
    security_descriptor: H,
    security_quality_of_service: H,
}
#[repr(C)]
struct NtIoStatusBlock {
    status_or_pointer: usize, information: usize,
}

/// Retain ALL directory HANDLEs, verified file IDs and fixed names.
/// Dropping this source-only object closes handles but NEVER deletes
/// a newly created directory or user/world/profile data.

#[allow(dead_code)]
struct PendingCreatedServiceRoot {
    // Service-parent provenance must be rechecked through the OS-known
    // machine data anchor before every source-only root observation.
    programdata: File,
    programdata_id: RootFileIdentity,
    parent: File,
    directory: File,
    parent_id: RootFileIdentity,
    directory_id: RootFileIdentity,
}

/// A diagnostic-only sample, not a root capability or launch grant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PendingServiceRootReview { active_logons_sampled: usize }
/// ABI-equivalent Win32 GENERIC_MAPPING for file-like named-pipe rights.
/// DesiredAccess passed to AccessCheck uses only SPECIFIC rights.
#[repr(C)]
struct GenericMapping {
    generic_read: u32,
    generic_write: u32,
    generic_execute: u32,
    generic_all: u32,
}
struct Group { sid: Vec<u8>, attributes: u32 }

#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(process: H, access: u32, token: *mut H) -> i32;
    fn GetTokenInformation(token: H, class: u32, output: H, capacity: u32, size: *mut u32) -> i32;
    fn LookupAccountNameW(
        system: *const u16, name: *const u16, sid: H, sid_len: *mut u32,
        domain: *mut u16, domain_len: *mut u32, sid_use: *mut u32,
    ) -> i32;
    fn IsValidSid(sid: H) -> i32;
    fn GetLengthSid(sid: H) -> u32;
    fn EqualSid(left: H, right: H) -> i32;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        sddl: *const u16, revision: u32, descriptor: *mut H,
        descriptor_size: *mut u32,
    ) -> i32;
    fn GetSecurityDescriptorOwner(
        descriptor: H, owner: *mut H, owner_defaulted: *mut i32,
    ) -> i32;
    fn GetSecurityDescriptorGroup(
        descriptor: H, group: *mut H, group_defaulted: *mut i32,
    ) -> i32;
    fn GetSecurityDescriptorDacl(
        descriptor: H, dacl_present: *mut i32,
        dacl: *mut H, dacl_defaulted: *mut i32,
    ) -> i32;
    fn DuplicateTokenEx(
        original: H, requested_access: u32, attributes: *const c_void,
        impersonation_level: u32, token_type: u32, duplicate: *mut H,
    ) -> i32;
    fn AccessCheck(
        descriptor: H, impersonation_token: H, desired: u32,
        generic_mapping: *mut GenericMapping,
        privilege_set: H, privilege_capacity: *mut u32,
        granted_access: *mut u32, access_status: *mut i32,
    ) -> i32;
    fn GetSecurityInfo(
        handle: H, object_type: u32, information: u32,
        owner: *mut H, group: *mut H, dacl: *mut H,
        sacl: *mut H, descriptor: *mut H,
    ) -> u32;
    fn GetSecurityDescriptorControl(
        descriptor: H, control: *mut u16, revision: *mut u32,
    ) -> i32;
    fn GetAclInformation(
        acl: H, information: H, length: u32, class: u32,
    ) -> i32;
    fn GetAce(acl: H, index: u32, ace: *mut H) -> i32;
    fn OpenThreadToken(
        thread: H, desired_access: u32, open_as_self: i32, token: *mut H,
    ) -> i32;
}
#[link(name = "shell32")]
unsafe extern "system" {
    fn SHGetKnownFolderPath(
        folder: *const NativeGuid, flags: u32,
        token: H, path: *mut *mut u16,
    ) -> i32;
}
#[link(name = "ole32")]
unsafe extern "system" {
    fn CoInitializeEx(reserved: H, concurrency: u32) -> i32;
    fn CoUninitialize();
    fn CoTaskMemFree(memory: H);
}
#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtCreateFile(
        output: *mut H, desired_access: u32,
        attributes: *mut NtObjectAttributes,
        io_status: *mut NtIoStatusBlock,
        allocation: *mut i64, file_attributes: u32,
        share_access: u32, disposition: u32, options: u32,
        ea_buffer: H, ea_length: u32,
    ) -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> H;
    fn GetCurrentThread() -> H;
    fn GetLastError() -> u32;
    fn CreateFileW(
        path: *const u16, access: u32, share: u32,
        attributes: H, disposition: u32, flags: u32,
        template: H,
    ) -> H;
    fn GetFinalPathNameByHandleW(
        file: H, buffer: *mut u16, capacity: u32, flags: u32,
    ) -> u32;
    fn CloseHandle(handle: H) -> i32;
    fn LocalFree(memory: H) -> H;
}
const WTS_ACTIVE: i32 = 0;
const WTS_CONNECTED: i32 = 1;
const WTS_DISCONNECTED: i32 = 4;
const WTS_LISTEN: i32 = 6;
const MAX_WTS_SESSIONS: u32 = 128;
const MAX_ACTIVE_SESSIONS: usize = 16;

#[repr(C)]
struct WtsSessionInfo {
    id: u32,
    station_name: *mut u16,
    state: i32,
}
struct WtsMemory(*mut WtsSessionInfo);
impl Drop for WtsMemory {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: WTS-owned allocation requires WTSFreeMemory.
            unsafe { WTSFreeMemory(self.0.cast()) };
        }
    }
}
#[link(name = "wtsapi32")]
unsafe extern "system" {
    fn WTSQueryUserToken(session_id: u32, token: *mut H) -> i32;
    fn WTSEnumerateSessionsW(
        server: H, reserved: u32, version: u32,
        sessions: *mut *mut WtsSessionInfo, count: *mut u32,
    ) -> i32;
    fn WTSFreeMemory(memory: H);
}
struct Token(H);

/// The candidate descriptor is self-relative memory from LocalAlloc.
/// It contains no stored rights on a real Windows pipe.
struct CandidateDescriptor(H);
impl Drop for CandidateDescriptor {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Owned output from ConvertStringSecurityDescriptorToSecurityDescriptorW.
            let _ = unsafe { LocalFree(self.0) };
        }
    }
}
impl Drop for Token {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Only the owned OpenProcessToken handle is closed.
            unsafe { CloseHandle(self.0) };
        }
    }
}
fn denied(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason)
}
fn enabled(attrs: u32) -> bool {
    attrs & SE_GROUP_ENABLED != 0 && attrs & SE_GROUP_DENY_ONLY == 0
}
fn nt_authority_sid(sid: &[u8], n: u8, first: u32) -> bool {
    sid.len() == 8 + usize::from(n) * 4
        && sid[0] == 1 && sid[1] == n
        && sid[2..8] == [0, 0, 0, 0, 0, 5]
        && sid[8..12] == first.to_le_bytes()
}
fn service_sid(sid: &[u8]) -> bool { nt_authority_sid(sid, 6, 80) }
fn logon_sid(sid: &[u8]) -> bool { nt_authority_sid(sid, 3, 5) }

// Only the service gets GENERIC_ALL. A prospective client must never
// receive FILE_CREATE_PIPE_INSTANCE (0x0004), GENERIC_WRITE, WRITE_DAC,
// or privileges over the server endpoint.
const SCOPED_CLIENT_WRITE_DATA: u32 = 0x0002;
const SCOPED_CLIENT_SYNCHRONIZE: u32 = 0x0010_0000;
const SCOPED_CLIENT_PIPE_RIGHTS: u32 =
    SCOPED_CLIENT_WRITE_DATA | SCOPED_CLIENT_SYNCHRONIZE;
const FILE_CREATE_PIPE_INSTANCE: u32 = 0x0004;

/// Validate a canonical Windows NT SERVICE SID string obtained exclusively
/// from verified_current_service_sid(). Reject injection and lookalikes.
fn canonical_service_sid_text(text: &str) -> bool {
    let Some(tail) = text.strip_prefix("S-1-5-80-") else {
        return false;
    };
    let components: Vec<_> = tail.split('-').collect();
    components.len() == 5 && components.iter().all(|component| {
        component.parse::<u32>().is_ok_and(|n| n.to_string() == *component)
    })
}

/// Build a *candidate* minimal inbound named-pipe DACL, never apply it.
/// Both principal identities must first be obtained from Windows: the
/// enabled current SCM service token, and WTSQueryUserToken for an
/// independently observed active user logon. This helper has NO OS calls,
/// does not create an endpoint, and does NOT establish client authority.
///
/// SYSTEM/service retain full control. Only the one verified S-1-5-5-X-Y
/// logon is given FILE_WRITE_DATA and SYNCHRONIZE; it never receives
/// FILE_CREATE_PIPE_INSTANCE, BUILTIN Users or Administrators grants.
fn scoped_logon_pipe_acl_candidate(
    verified_service: &str, verified_logon_sid: &[u8],
) -> io::Result<String> {
    if !canonical_service_sid_text(verified_service) || !logon_sid(verified_logon_sid) {
        return Err(denied("Noncanonical service/logon SID cannot define pipe policy"));
    }
    if SCOPED_CLIENT_PIPE_RIGHTS & FILE_CREATE_PIPE_INSTANCE != 0 {
        return Err(denied("Client rights could create a server pipe instance"));
    }
    // SAFETY: logon_sid validates an exact native S-1-5-5-X-Y SID with
    // 20 bytes and two remaining little-endian subauthorities.
    let first = u32::from_le_bytes(verified_logon_sid[12..16].try_into()
        .map_err(|_| denied("Malformed first logon subauthority"))?);
    let second = u32::from_le_bytes(verified_logon_sid[16..20].try_into()
        .map_err(|_| denied("Malformed second logon subauthority"))?);
    let scoped_principal = format!("S-1-5-5-{first}-{second}");
    Ok(format!(
        "O:SYG:SYD:P(A;;GA;;;SY)(A;;GA;;;{verified_service})(A;;0x{SCOPED_CLIENT_PIPE_RIGHTS:08X};;;{scoped_principal})"
    ))
}

/// Convert the exact candidate to a native descriptor with explicit OWNER,
/// GROUP and non-NULL protected DACL. AccessCheck rejects descriptors that
/// omit owner/group, and a NULL DACL would grant everyone access.
fn candidate_descriptor(sddl: &str) -> io::Result<CandidateDescriptor> {
    if !sddl.starts_with("O:SYG:SYD:P(") {
        return Err(denied("A scoped descriptor must have SYSTEM owner/group and protected DACL"));
    }
    let wide: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
    let mut raw: H = ptr::null_mut();
    // SAFETY: NUL-terminated validated internal SDDL; LocalAlloc result.
    let ok = unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide.as_ptr(), 1, &mut raw, ptr::null_mut(),
        )
    };
    let guard = CandidateDescriptor(raw);
    if ok == 0 {
        return Err(io::Error::last_os_error());
    }
    if guard.0.is_null() {
        return Err(denied("No candidate Windows descriptor"));
    }
    let mut owner = ptr::null_mut();
    let mut group = ptr::null_mut();
    let mut dacl = ptr::null_mut();
    let mut defaulted = 0i32;
    let mut present = 0i32;
    // SAFETY: All returned pointer values reference guard-owned memory.
    // They are used only to assert non-NULL owner/group/protected DACL.
    let valid = unsafe {
        GetSecurityDescriptorOwner(guard.0, &mut owner, &mut defaulted) != 0
            && GetSecurityDescriptorGroup(guard.0, &mut group, &mut defaulted) != 0
            && GetSecurityDescriptorDacl(guard.0, &mut present, &mut dacl, &mut defaulted) != 0
    };
    if !valid || owner.is_null() || group.is_null() || present == 0 || dacl.is_null() {
        return Err(denied("Candidate descriptor is missing owner, group or non-NULL DACL"));
    }
    Ok(guard)
}

/// Win32 AccessCheck REQUIRES a query-capable impersonation token, not the
/// WTSQueryUserToken primary HANDLE. No impersonation is installed on the
/// service thread and no new process/privilege is created or enabled.
fn duplicate_for_access_check(primary: H) -> io::Result<Token> {
    if primary.is_null() {
        return Err(denied("Missing primary OS token for native AccessCheck"));
    }
    let mut duplicate: H = ptr::null_mut();
    // SAFETY: WTSQueryUserToken provided the OS primary token; the caller
    // retains it, and Windows independently checks its TOKEN_DUPLICATE right.
    if unsafe {
        DuplicateTokenEx(
            primary, TOKEN_QUERY, ptr::null(),
            SECURITY_IMPERSONATION, TOKEN_IMPERSONATION, &mut duplicate,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if duplicate.is_null() {
        return Err(denied("No impersonation token returned for AccessCheck"));
    }
    Ok(Token(duplicate))
}

fn native_grants(descriptor: &CandidateDescriptor, client: &Token, desired: u32)
    -> io::Result<bool>
{
    // Restrict this code to precisely the two candidate policy questions;
    // generic masks, WRITE_DAC and file mutation rights are not accepted.
    if !matches!(desired, SCOPED_CLIENT_PIPE_RIGHTS | FILE_CREATE_PIPE_INSTANCE) {
        return Err(denied("AccessCheck asked for an unreviewed pipe permission"));
    }
    checked_native_access(descriptor, client, desired)
}

/// Generic-specific RIGHT evaluation shared by the existing pipe
/// policy and the read-only directory-root review. NEVER call directly
/// on untrusted requested masks; surface-specific wrappers restrict them.
fn checked_native_access(
    descriptor: &CandidateDescriptor, client: &Token, desired: u32,
) -> io::Result<bool> {
    let mut mapping = GenericMapping {
        generic_read: 0x0012_0089,
        generic_write: 0x0012_0116,
        generic_execute: 0x0012_00A0,
        generic_all: 0x001F_01FF,
    };
    // The Win32 privilege-set output must be aligned; 4096 bytes is a
    // deliberate fixed upper bound. ERROR_INSUFFICIENT_BUFFER denies.
    let mut privileges = [0usize; 512];
    let mut length = mem::size_of_val(&privileges) as u32;
    let mut granted = 0u32;
    let mut status = 0i32;
    // SAFETY: Descriptor and query-capable impersonation token remain
    // owned for the entire read-only access evaluation.
    if unsafe {
        AccessCheck(
            descriptor.0, client.0, desired, &mut mapping,
            privileges.as_mut_ptr().cast(), &mut length,
            &mut granted, &mut status,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if length > mem::size_of_val(&privileges) as u32 {
        return Err(denied("Native privilege set exceeded fixed buffer"));
    }
    if status != 0 && (granted & desired) != desired {
        return Err(denied("Inconsistent native AccessCheck grant mask"));
    }
    Ok(status != 0)
}

/// Protected candidate for a future SERVICE-OWNED manager data root.
/// O:SY/G:SY is deliberate: the SID of a Windows service is an enabled
/// token GROUP, not its TokenUser. FA is inherited by future file/directory
/// children through OI/CI; no Users, Administrators, Everyone or interactive
/// logon grants. This function does not CREATE or modify an NTFS object.
fn service_root_acl_candidate(verified_service: &str) -> io::Result<String> {
    if !canonical_service_sid_text(verified_service) {
        return Err(denied("Unverified service SID cannot define root ACL"));
    }
    Ok(format!(
        "O:SYG:SYD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;{verified_service})"
    ))
}

fn root_access(descriptor: &CandidateDescriptor, token: &Token, right: u32)
    -> io::Result<bool>
{
    if !ROOT_CHECK_RIGHTS.contains(&right) {
        return Err(denied("Unreviewed service-root access right"));
    }
    checked_native_access(descriptor, token, right)
}

fn require_nonimpersonating_thread() -> io::Result<()> {
    let mut raw: H = ptr::null_mut();
    // SAFETY: Query the CURRENT service thread only. A root must never be
    // inspected during IPC-client impersonation or an unknown token state.
    let ok = unsafe {
        OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut raw)
    };
    let borrowed = Token(raw);
    if ok != 0 || unsafe { GetLastError() } != ERROR_NO_TOKEN {
        return Err(denied("Service-root inspector requires no thread impersonation"));
    }
    drop(borrowed);
    Ok(())
}

fn root_file_identity(root: &File) -> io::Result<RootFileIdentity> {
    // SAFETY: A retained Windows File handle is required. No pathname is
    // consulted; reject reparse points, files and unidentifiable objects.
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandle(root.as_raw_handle() as H, &mut info)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if info.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || info.dwFileAttributes & FILE_ATTRIBUTE_REPARSE_POINT != 0
    {
        return Err(denied("Service-root HANDLE is not a plain Windows directory"));
    }
    let index = (u64::from(info.nFileIndexHigh) << 32)
        | u64::from(info.nFileIndexLow);
    if index == 0 {
        return Err(denied("Windows file identity is not available"));
    }
    Ok(RootFileIdentity { volume: info.dwVolumeSerialNumber, index })
}

/// Validate the exact expected two-ACE DACL rather than trusting only one
/// sampled desktop token's AccessCheck. An arbitrary BA/WD/BU third ACE or
/// an inherited ACE is a policy mismatch even when no such user is active.
/// All returned ACE pointers and SID bytes remain inside their ACL span.
fn exact_service_root_acl(dacl: H, service_sid_bytes: &[u8]) -> io::Result<()> {
    if dacl.is_null() || !service_sid(service_sid_bytes) {
        return Err(denied("Missing DACL or non-service group SID"));
    }
    let mut info = AclSizeInformation {
        ace_count: 0, bytes_in_use: 0, bytes_free: 0,
    };
    if unsafe {
        GetAclInformation(
            dacl, (&mut info as *mut AclSizeInformation).cast(),
            mem::size_of::<AclSizeInformation>() as u32, ROOT_ACL_SIZE_CLASS,
        )
    } == 0 || info.ace_count != 2 || info.bytes_in_use < 8 {
        return Err(denied("Service-root DACL is not exactly two allow ACEs"));
    }
    // Validate the Windows ACL_HEADER's declared byte size as well as
    // AclBytesInUse before using GetAce pointers. Never let an ACE body
    // extend into memory beyond the actual returned ACL allocation.
    // SAFETY: A successful GetAclInformation above means dacl points at a
    // Win32 ACL header of at least 8 bytes.
    let acl_header = unsafe { std::slice::from_raw_parts(dacl.cast::<u8>(), 8) };
    let allocated = u16::from_le_bytes([acl_header[2], acl_header[3]]) as usize;
    if allocated < 8 || info.bytes_in_use as usize > allocated {
        return Err(denied("Service-root DACL byte usage exceeds ACL allocation"));
    }
    let beginning = dacl as usize;
    let end = beginning.checked_add(info.bytes_in_use as usize)
        .ok_or_else(|| denied("Invalid service-root ACL byte span"))?;
    let mut system_found = false;
    let mut service_found = false;
    for index in 0..2u32 {
        let mut ace: H = ptr::null_mut();
        if unsafe { GetAce(dacl, index, &mut ace) } == 0 || ace.is_null() {
            return Err(denied("Missing service-root ACE"));
        }
        let begin = ace as usize;
        if begin < beginning + 8 || begin.checked_add(16).is_none_or(|n| n > end) {
            return Err(denied("Truncated service-root ACE header"));
        }
        // SAFETY: The first 16 ACE bytes are confirmed inside the OS-owned
        // ACL. This includes the SID's minimum 8-byte header at offset 8.
        let bytes = unsafe { std::slice::from_raw_parts(ace.cast::<u8>(), 16) };
        let length = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
        let mask = u32::from_le_bytes(bytes[4..8].try_into()
            .map_err(|_| denied("Malformed ACE access mask"))?);
        let sid_len = 8 + usize::from(bytes[9]) * 4;
        if bytes[0] != ROOT_ACE_ALLOWED_TYPE
            || bytes[1] != ROOT_ACE_OBJECT_AND_CONTAINER_INHERIT
            || bytes[8] != 1 || bytes[9] > 15
            || mask != ROOT_FILE_ALL_ACCESS
            || length != 8 + sid_len
            || begin.checked_add(length).is_none_or(|n| n > end)
        {
            return Err(denied("Unexpected service-root ACE flags or size"));
        }
        // SAFETY: The full ACE/SID size was checked against the live ACL
        // buffer before reading its bytes. SID values never leave this call.
        let sid = unsafe {
            std::slice::from_raw_parts(ace.cast::<u8>().add(8), sid_len)
        };
        if sid == LOCAL_SYSTEM_SID.as_slice() && !system_found {
            system_found = true;
        } else if sid == service_sid_bytes && !service_found {
            service_found = true;
        } else {
            return Err(denied("Unknown or duplicate service-root ACL principal"));
        }
    }
    if !system_found || !service_found {
        return Err(denied("Service-root ACL lacks SYSTEM or exact service SID"));
    }
    Ok(())
}

/// Inspect one ALREADY-OPENED real directory HANDLE. The current process
/// must be the expected SCM service with its verified enabled service SID;
/// the root must have SYSTEM owner/group and an exact protected two-ACE
/// service+SYSTEM DACL, not merely a private-looking path. No filesystem
/// security descriptor is changed by this review. The trusted ancestor
/// and creation/handoff mechanism are still NOT implemented.
fn existing_root_descriptor(root: &File, service_sid: &[u8])
    -> io::Result<CandidateDescriptor>
{
    let mut owner: H = ptr::null_mut();
    let mut group: H = ptr::null_mut();
    let mut dacl: H = ptr::null_mut();
    let mut raw: H = ptr::null_mut();
    // SAFETY: Root File retains READ_CONTROL; GetSecurityInfo returns one
    // LocalFree-owned allocation whose SID/ACL pointers remain borrowed.
    let code = unsafe {
        GetSecurityInfo(
            root.as_raw_handle() as H, ROOT_FILE_OBJECT,
            ROOT_SECURITY_FLAGS,
            &mut owner, &mut group, &mut dacl, ptr::null_mut(), &mut raw,
        )
    };
    let descriptor = CandidateDescriptor(raw);
    if code != 0 {
        return Err(io::Error::from_raw_os_error(code as i32));
    }
    if descriptor.0.is_null() || owner.is_null() || group.is_null()
        || dacl.is_null()
        || unsafe { IsValidSid(owner) } == 0
        || unsafe { IsValidSid(group) } == 0
    {
        return Err(denied("Service root is missing valid owner, group or DACL"));
    }
    if unsafe {
        EqualSid(owner, LOCAL_SYSTEM_SID.as_ptr() as H)
    } == 0 || unsafe {
        EqualSid(group, LOCAL_SYSTEM_SID.as_ptr() as H)
    } == 0 {
        return Err(denied("Service-root owner and group must both be SYSTEM"));
    }
    let mut control = 0u16;
    let mut revision = 0u32;
    if unsafe {
        GetSecurityDescriptorControl(descriptor.0, &mut control, &mut revision)
    } == 0 || control & ROOT_DACL_PROTECTED == 0 {
        return Err(denied("Service-root DACL is not protected against inheritance"));
    }
    exact_service_root_acl(dacl, service_sid)?;
    Ok(descriptor)
}

/// Genuine Windows read-only root-policy review, NOT a root constructor,
/// a service/desktop handle handoff, or a production authorization gate.
/// This function is intentionally NOT CALLED by SCM startup, IPC or Tauri.
/// Revalidates exact service group membership and OS-active peer snapshots;
/// even success means ONLY one current sampled DACL/ACL observation.
#[allow(dead_code)]
fn review_existing_service_root_source(root: &File)
    -> io::Result<PendingServiceRootReview>
{
    require_nonimpersonating_thread()?;
    let service = verified_current_service_sid()?;
    let expected = expected_service_sid()?;
    // The dedicated service must run as LocalSystem in this candidate
    // ownership model. Its service SID is an enabled GROUP, not TokenUser.
    let mut raw: H = ptr::null_mut();
    if unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_DUPLICATE, &mut raw)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    let process = Token(raw);
    if process.0.is_null()
        || token_dword(process.0, TOKEN_TYPE)? != TOKEN_PRIMARY
        || verified_token_user_sid(process.0)?.as_slice() != LOCAL_SYSTEM_SID.as_slice()
    {
        return Err(denied("Service-root review requires a LocalSystem SCM process"));
    }
    let owner_token = duplicate_for_access_check(process.0)?;
    let before = root_file_identity(root)?;
    let descriptor = existing_root_descriptor(root, &expected)?;
    for right in [
        ROOT_LIST, ROOT_ADD_FILE, ROOT_ADD_DIRECTORY,
        ROOT_TRAVERSE, ROOT_READ_CONTROL,
    ] {
        if !root_access(&descriptor, &owner_token, right)? {
            return Err(denied("SCM process cannot access its candidate data root"));
        }
    }
    let active = enumerate_active_ids()?;
    for &session in &active {
        let mut raw: H = ptr::null_mut();
        let ok = unsafe { WTSQueryUserToken(session, &mut raw) };
        let other = Token(raw);
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        if other.0.is_null()
            || token_dword(other.0, TOKEN_TYPE)? != TOKEN_PRIMARY
            || token_dword(other.0, TOKEN_SESSION_ID)? != session
            || verified_token_user_sid(other.0)?.as_slice() == LOCAL_SYSTEM_SID.as_slice()
        {
            return Err(denied("Unsupported interactive peer token for root ACL review"));
        }
        let peer = duplicate_for_access_check(other.0)?;
        for right in ROOT_CHECK_RIGHTS {
            if root_access(&descriptor, &peer, right)? {
                return Err(denied("Peer has a direct sensitive right to service-root HANDLE"));
            }
        }
    }
    // Re-check only sample-specific properties. An ACL or token may change
    // AFTER this function; current findings never turn into lease/launch.
    if enumerate_active_ids()? != active
        || verified_current_service_sid()? != service
        || root_file_identity(root)? != before
    {
        return Err(denied("Native service/root/logon identity changed during review"));
    }
    let _ = existing_root_descriptor(root, &expected)?;
    require_nonimpersonating_thread()?;
    Ok(PendingServiceRootReview { active_logons_sampled: active.len() })
}

/// COM initialization is paired on this thread even if a Windows Known
/// Folder query fails. A mismatched COM apartment fails closed; no attempt
/// is made to change or impersonate an interactive user's thread.
struct ComApartment;
impl ComApartment {
    fn enter() -> io::Result<Self> {
        let result = unsafe { CoInitializeEx(ptr::null_mut(), COM_MULTITHREADED) };
        if result < 0 {
            return Err(denied("SCM Known Folder lookup requires a supported COM apartment"));
        }
        Ok(Self)
    }
}
impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}
struct KnownFolderString(*mut u16);
impl Drop for KnownFolderString {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CoTaskMemFree(self.0.cast()) };
        }
    }
}

/// Fail closed on UNC, extended namespace, relative, dot-segment and
/// non-ASCII known-folder paths. This is intentionally narrower than all
/// possible Windows installations until Unicode/reparse chain validation
/// is independently tested. No end-user path is passed to this helper.
fn expected_known_folder_final_path(path: &str) -> io::Result<String> {
    let bytes = path.as_bytes();
    if bytes.len() < 4 || bytes.len() > 900
        || !path.is_ascii()
        || !bytes[0].is_ascii_alphabetic()
        || bytes[1] != b':' || bytes[2] != b'\\'
        || path.contains('/')
        || path.ends_with('\\')
        || path.split('\\').any(|part| {
            part.is_empty() || part == "." || part == ".."
                || part.ends_with('.') || part.ends_with(' ')
                || part.contains(':') && part.len() > 2
        })
    {
        return Err(denied("ProgramData Known Folder is not an unambiguous local drive path"));
    }
    Ok(format!("\\\\?\\{path}"))
}

/// Obtain a real nonreparse directory HANDLE from FOLDERID_ProgramData,
/// *never* by consulting environment variables or an IPC path. Compare the
/// OS-returned normalized HANDLE path to the Known Folder path; if any
/// reparse traversal, drive substitution or unsupported volume syntax
/// changes the resolved location, refuse the anchor.
fn open_known_programdata_source() -> io::Result<File> {
    require_nonimpersonating_thread()?;
    let _com = ComApartment::enter()?;
    let mut raw_path: *mut u16 = ptr::null_mut();
    let hr = unsafe {
        SHGetKnownFolderPath(
            &PROGRAM_DATA_KNOWN_FOLDER, 0, ptr::null_mut(), &mut raw_path,
        )
    };
    let guard = KnownFolderString(raw_path);
    if hr != 0 || guard.0.is_null() {
        return Err(denied("Windows ProgramData Known Folder is unavailable"));
    }
    let mut length = 0usize;
    while length < MAX_PROGRAMDATA_UTF16 {
        if unsafe { *guard.0.add(length) } == 0 {
            break;
        }
        length += 1;
    }
    if length == 0 || length == MAX_PROGRAMDATA_UTF16 {
        return Err(denied("Unbounded ProgramData Known Folder string"));
    }
    let text = String::from_utf16(unsafe {
        std::slice::from_raw_parts(guard.0, length)
    }).map_err(|_| denied("ProgramData Known Folder has invalid UTF-16"))?;
    let expected = expected_known_folder_final_path(&text)?;
    let wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
    let raw = unsafe {
        CreateFileW(
            wide.as_ptr(), NT_FILE_READ_ATTRIBUTES | ROOT_READ_CONTROL | ROOT_TRAVERSE,
            NT_FILE_SHARE_READ | NT_FILE_SHARE_WRITE, ptr::null_mut(),
            WIN32_OPEN_EXISTING,
            NT_FILE_FLAG_BACKUP_SEMANTICS | NT_FILE_FLAG_OPEN_REPARSE_POINT,
            ptr::null_mut(),
        )
    };
    if raw.is_null() || raw == (-1isize as H) {
        return Err(io::Error::last_os_error());
    }
    let folder = unsafe { File::from_raw_handle(raw) };
    root_file_identity(&folder)?;
    let mut final_name = [0u16; MAX_PROGRAMDATA_UTF16];
    let count = unsafe {
        GetFinalPathNameByHandleW(
            folder.as_raw_handle() as H,
            final_name.as_mut_ptr(), final_name.len() as u32, 0,
        )
    };
    if count == 0 || count as usize >= final_name.len() {
        return Err(denied("ProgramData resolved HANDLE path is missing or oversized"));
    }
    let resolved = String::from_utf16(&final_name[..count as usize])
        .map_err(|_| denied("ProgramData final HANDLE path has invalid UTF-16"))?;
    if !expected.eq_ignore_ascii_case(&resolved) {
        return Err(denied("ProgramData HANDLE resolves outside OS Known Folder"));
    }
    require_nonimpersonating_thread()?;
    Ok(folder)
}

/// Require a canonical installer-created service parent immediately under
/// ProgramData. The supplied parent HANDLE must refer to the SAME file
/// object as the fixed child reopened relative to the OS Known Folder
/// HANDLE. A forged path or a merely matching DACL cannot replace it.
/// The known folder's own ancestor DACLs and elevated administrators remain
/// separately unresolved; this returns no launch or storage admission.
fn verify_installer_parent_link(
    programdata: &File, parent: &File,
) -> io::Result<()> {
    let root = root_file_identity(programdata)?;
    let current = root_file_identity(parent)?;
    let reopened = relative_installer_parent(programdata)?;
    if root.volume != current.volume || root == current
        || root_file_identity(&reopened)? != current
        || root_file_identity(programdata)? != root
    {
        return Err(denied("Installer parent is not fixed child of OS ProgramData HANDLE"));
    }
    Ok(())
}

fn relative_installer_parent(programdata: &File) -> io::Result<File> {
    relative_fixed_service_child(programdata, INSTALLER_PARENT_CHILD,
        NT_FILE_OPEN, None)
}

/// The ONLY permitted root name is fixed by code. In particular a
/// forged parent-relative path, DOS device, arbitrary slot or reparse name
/// cannot be smuggled through the factory's native OBJECT_ATTRIBUTES.
fn checked_service_root_name(name: &str) -> io::Result<()> {
    if name != SERVICE_ROOT_CHILD
        || name.is_empty()
        || name.len() > 64
        || !name.is_ascii()
        || !name.bytes().all(|b|
            b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    {
        return Err(denied("Unsupported service-root child name"));
    }
    Ok(())
}

fn nt_root_create_outcome(status: i32, information: usize)
    -> io::Result<()>
{
    if status != 0 {
        let kind = match status as u32 {
            NT_STATUS_OBJECT_NAME_COLLISION => io::ErrorKind::AlreadyExists,
            NT_STATUS_OBJECT_NAME_NOT_FOUND | NT_STATUS_OBJECT_PATH_NOT_FOUND =>
                io::ErrorKind::NotFound,
            NT_STATUS_ACCESS_DENIED => io::ErrorKind::PermissionDenied,
            _ => io::ErrorKind::Other,
        };
        return Err(io::Error::new(
            kind, format!("NtCreateFile refused service root (0x{:08X})", status as u32),
        ));
    }
    if information != NT_FILE_CREATED {
        // FILE_OPENED/FILE_EXISTS must never adopt a pre-existing root.
        return Err(denied("Native service-root creation did not return FILE_CREATED"));
    }
    Ok(())
}

/// One HANDLE-relative native I/O helper for ONLY the two fixed names
/// with distinct dispositions. An installer parent can ONLY be opened;
/// only the inner service root may be newly created with the exact reviewed
/// protected DACL. No caller-supplied path or slot is accepted.
fn relative_fixed_service_child(
    parent: &File, child: &str, disposition: u32,
    candidate: Option<&CandidateDescriptor>,
) -> io::Result<File> {
    if !matches!(disposition, NT_FILE_CREATE | NT_FILE_OPEN)
        || (disposition == NT_FILE_CREATE) != candidate.is_some()
        || !(child == SERVICE_ROOT_CHILD
            || child == INSTALLER_PARENT_CHILD && disposition == NT_FILE_OPEN)
    {
        return Err(denied("Unsupported native service-child name or disposition"));
    }
    if child == SERVICE_ROOT_CHILD {
        checked_service_root_name(child)?;
    }
    let mut wide: Vec<u16> = child.encode_utf16().collect();
    let byte_len = wide.len().checked_mul(mem::size_of::<u16>())
        .filter(|len| *len <= u16::MAX as usize)
        .ok_or_else(|| denied("Native service-root UTF-16 name exceeded bounds"))?;
    wide.push(0);
    let mut name = NtUnicodeString {
        length: byte_len as u16,
        maximum_length: (byte_len + 2) as u16,
        buffer: wide.as_mut_ptr(),
    };
    let mut attributes = NtObjectAttributes {
        length: mem::size_of::<NtObjectAttributes>() as u32,
        root_directory: parent.as_raw_handle() as H,
        object_name: &mut name,
        attributes: NT_OBJ_CASE_INSENSITIVE,
        security_descriptor: candidate.map_or(ptr::null_mut(), |v| v.0),
        security_quality_of_service: ptr::null_mut(),
    };
    let mut outcome = NtIoStatusBlock {
        status_or_pointer: 0, information: 0,
    };
    let mut raw: H = ptr::null_mut();
    let access = NT_FILE_READ_ATTRIBUTES | ROOT_READ_CONTROL | NT_SYNCHRONIZE
        | NT_FILE_ADD_SUBDIRECTORY;
    // Do not allow DELETE sharing while this root HANDLE is retained.
    // Root or parent substitution must still be checked after each action.
    let code = unsafe {
        NtCreateFile(
            &mut raw, access, &mut attributes, &mut outcome,
            ptr::null_mut(), NT_FILE_ATTRIBUTE_NORMAL,
            NT_FILE_SHARE_READ | NT_FILE_SHARE_WRITE,
            disposition,
            NT_FILE_DIRECTORY_FILE | NT_FILE_SYNCHRONOUS_IO_NONALERT
                | NT_FILE_OPEN_REPARSE_POINT,
            ptr::null_mut(), 0,
        )
    };
    let validity = if disposition == NT_FILE_CREATE {
        nt_root_create_outcome(code, outcome.information)
    } else if code == 0 {
        Ok(())
    } else {
        nt_root_create_outcome(code, outcome.information)
    };
    if let Err(error) = validity {
        if !raw.is_null() && raw != (-1isize as H) {
            unsafe { CloseHandle(raw) };
        }
        return Err(error);
    }
    if raw.is_null() || raw == (-1isize as H) {
        return Err(denied("Native service-root operation returned an invalid HANDLE"));
    }
    // SAFETY: A successful NtCreateFile call transfers exactly one HANDLE,
    // which std::fs::File closes when this pending object is dropped.
    let result = unsafe { File::from_raw_handle(raw) };
    root_file_identity(&result)?;
    Ok(result)
}

fn relative_service_root(
    parent: &File,
    disposition: u32,
    candidate: Option<&CandidateDescriptor>,
) -> io::Result<File> {
    relative_fixed_service_child(
        parent, SERVICE_ROOT_CHILD, disposition, candidate,
    )
}

impl PendingCreatedServiceRoot {
    /// Read-only revalidation. Even a successful result is not an approved
    /// Windows filesystem virtualization owner or a Minecraft launch grant.
    fn verify(&self) -> io::Result<PendingServiceRootReview> {
        require_nonimpersonating_thread()?;
        let fresh_os_folder = open_known_programdata_source()?;
        if root_file_identity(&fresh_os_folder)? != self.programdata_id
            || root_file_identity(&self.programdata)? != self.programdata_id
        {
            return Err(denied("Windows ProgramData Known Folder anchor changed"));
        }
        verify_installer_parent_link(&self.programdata, &self.parent)?;
        let active_before = enumerate_active_ids()?;
        let parent_report = review_existing_service_root_source(&self.parent)?;
        if root_file_identity(&self.parent)? != self.parent_id {
            return Err(denied("Service parent HANDLE identity changed"));
        }
        let reopened = relative_service_root(&self.parent, NT_FILE_OPEN, None)?;
        if root_file_identity(&reopened)? != self.directory_id
            || root_file_identity(&self.directory)? != self.directory_id
            || self.parent_id.volume != self.directory_id.volume
            || self.parent_id == self.directory_id
        {
            return Err(denied("Service child is no longer attached to original parent HANDLE"));
        }
        let child_report = review_existing_service_root_source(&self.directory)?;
        if root_file_identity(&self.parent)? != self.parent_id
            || root_file_identity(&self.programdata)? != self.programdata_id
            || enumerate_active_ids()? != active_before
            || parent_report.active_logons_sampled != active_before.len()
            || child_report.active_logons_sampled != active_before.len()
        {
            return Err(denied("Service parent or active logons changed during root verification"));
        }
        require_nonimpersonating_thread()?;
        Ok(PendingServiceRootReview {
            active_logons_sampled: active_before.len(),
        })
    }
}

/// SOURCE-ONLY fresh root factory, deliberately unreachable from SCM
/// startup, named pipe handlers, desktop IPC, installers and game launch.
/// The caller's parent HANDLE MUST match the fixed, existing installer
/// directory reopened below Windows FOLDERID_ProgramData. It is never
/// adopted by name alone; only its exact same native FILE_ID is accepted.
/// Full drive-root ancestry ACLs, installer signature and privilege scope
/// remain unproven: this cannot activate client or game access.
/// No arbitrary child names, host profiles or Minecraft paths are accepted.
///
/// On post-create failure leave the new object untouched for deliberate
/// review. Never remove a Windows directory automatically by pathname.
#[allow(dead_code)]
fn create_fresh_service_root_source(parent: File)
    -> io::Result<PendingCreatedServiceRoot>
{
    checked_service_root_name(SERVICE_ROOT_CHILD)?;
    require_nonimpersonating_thread()?;
    // The OS machine-wide anchor is obtained INSIDE this source owner,
    // not from a caller-supplied File or environment variable.
    let programdata = open_known_programdata_source()?;
    let programdata_id = root_file_identity(&programdata)?;
    verify_installer_parent_link(&programdata, &parent)?;
    let parent_id = root_file_identity(&parent)?;
    let active_before = enumerate_active_ids()?;
    review_existing_service_root_source(&parent)?;
    if enumerate_active_ids()? != active_before {
        return Err(denied("Active Windows logons changed before root creation"));
    }
    let service = verified_current_service_sid()?;
    let candidate = candidate_descriptor(&service_root_acl_candidate(&service)?)?;
    let directory = relative_service_root(&parent, NT_FILE_CREATE, Some(&candidate))?;
    let directory_id = root_file_identity(&directory)?;
    if parent_id.volume != directory_id.volume || parent_id == directory_id {
        return Err(denied("Fresh service-root child was not distinct on parent volume"));
    }
    let created = PendingCreatedServiceRoot {
        programdata, programdata_id,
        parent, directory, parent_id, directory_id,
    };
    created.verify()?;
    if enumerate_active_ids()? != active_before
        || verified_current_service_sid()? != service
        || root_file_identity(&created.parent)? != created.parent_id
    {
        return Err(denied("SCM service or active Windows logons changed during root creation"));
    }
    Ok(created)
}

/// Actually ask Windows to evaluate a candidate descriptor against the WTS
/// primary token, and optionally the other ACTIVE, OS-enumerated logons.
/// This is a validation gate only. No DACL is applied to an endpoint,
/// no authorization is emitted, and all handles/descriptors are dropped.
fn review_candidate_effective_rights(
    candidate: &str, intended: H, active: &[u32], intended_session: u32,
    intended_logon: &[u8],
) -> io::Result<()> {
    let descriptor = candidate_descriptor(candidate)?;
    let token = duplicate_for_access_check(intended)?;
    if !native_grants(&descriptor, &token, SCOPED_CLIENT_PIPE_RIGHTS)? {
        return Err(denied("Windows denies expected logon's minimal pipe write rights"));
    }
    if native_grants(&descriptor, &token, FILE_CREATE_PIPE_INSTANCE)? {
        return Err(denied("Windows grants client forbidden pipe-instance creation"));
    }
    // A second simultaneously active logon must not receive the first
    // logon's WRITE_DATA grant. The source can evaluate 0..15 peers;
    // absent peers, negative peer isolation is STILL NOT proven.
    for &session in active.iter().filter(|&&id| id != intended_session) {
        let mut raw: H = ptr::null_mut();
        // SAFETY: This ID is from the bounded local WTSActive inventory,
        // NEVER the broker frame or a target supplied by an IPC caller.
        let ok = unsafe { WTSQueryUserToken(session, &mut raw) };
        let other = Token(raw);
        if ok == 0 {
            return Err(io::Error::last_os_error());
        }
        if other.0.is_null()
            || token_dword(other.0, TOKEN_TYPE)? != TOKEN_PRIMARY
            || token_dword(other.0, TOKEN_SESSION_ID)? != session
            || verified_token_logon_sid(other.0)? == intended_logon
        {
            return Err(denied("Other active Windows session has unusable logon evidence"));
        }
        let other_token = duplicate_for_access_check(other.0)?;
        if native_grants(&descriptor, &other_token, SCOPED_CLIENT_PIPE_RIGHTS)?
            || native_grants(&descriptor, &other_token, FILE_CREATE_PIPE_INSTANCE)?
        {
            return Err(denied("Windows grants another logon the candidate pipe rights"));
        }
    }
    // A connected/disconnected/relogged session can change between
    // observations. Re-enumerate at the end and fail closed on any change.
    if enumerate_active_ids()? != active {
        return Err(denied("WTSActive session set changed during ACL review"));
    }
    Ok(())
}

/// Copy a native SID only after proving its header AND entire body are
/// inside the returned GetTokenInformation allocation. OS API output
/// pointers must never be dereferenced before these bounds checks.
/// One source owner handles TokenGroups and TokenUser with the same rule.
fn copy_bounded_sid(raw: H, words: &[usize], returned: u32) -> io::Result<Vec<u8>> {
    let capacity = words.len().checked_mul(mem::size_of::<usize>())
        .ok_or_else(|| denied("Token buffer capacity overflow"))?;
    if raw.is_null() || returned == 0 || returned as usize > capacity {
        return Err(denied("No bounded SID in token result"));
    }
    let base = words.as_ptr() as usize;
    let end = base.checked_add(returned as usize)
        .ok_or_else(|| denied("Token result address overflow"))?;
    let start = raw as usize;
    if start < base || start.checked_add(8).is_none_or(|value| value > end) {
        return Err(denied("Native SID header outside returned token buffer"));
    }
    // SAFETY: All eight SID header bytes are now inside live aligned
    // backing storage; neither raw pointer nor length came from IPC.
    let header = unsafe { std::slice::from_raw_parts(raw.cast::<u8>(), 8) };
    if header[0] != 1 || header[1] > 15 {
        return Err(denied("Unsupported native SID revision or count"));
    }
    let len = 8 + usize::from(header[1]) * 4;
    if start.checked_add(len).is_none_or(|value| value > end) {
        return Err(denied("Native SID body outside returned token buffer"));
    }
    // SAFETY: Full SID body has been checked before consulting Win32.
    if unsafe { IsValidSid(raw) } == 0
        || unsafe { GetLengthSid(raw) } as usize != len
    {
        return Err(denied("Invalid bounded Windows SID"));
    }
    Ok(unsafe { std::slice::from_raw_parts(raw.cast::<u8>(), len) }.to_vec())
}

/// Copy actual TOKEN_GROUPS with aligned native storage. The SID bytes
/// MUST be within the same returned buffer before any pointer is read.
fn groups(token: H) -> io::Result<Vec<Group>> {
    if token.is_null() { return Err(denied("Null token")); }
    let mut needed = 0u32;
    let first = unsafe {
        GetTokenInformation(token, TOKEN_GROUPS, ptr::null_mut(), 0, &mut needed)
    };
    if first != 0 || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
        || needed < mem::size_of::<TokenGroups>() as u32 || needed > MAX_GROUP_BYTES
    { return Err(denied("Unsupported TokenGroups length")); }
    let mut words = vec![0usize; (needed as usize).div_ceil(mem::size_of::<usize>())];
    let mut returned = needed;
    // SAFETY: Native output is aligned by usize words and bounded to 'needed'.
    if unsafe {
        GetTokenInformation(token, TOKEN_GROUPS, words.as_mut_ptr().cast(), needed, &mut returned)
    } == 0 { return Err(io::Error::last_os_error()); }
    let offset = mem::offset_of!(TokenGroups, groups);
    if returned > needed || (returned as usize) < offset + mem::size_of::<SidAndAttributes>() {
        return Err(denied("Truncated TokenGroups header"));
    }
    let count = unsafe { (*(words.as_ptr().cast::<TokenGroups>())).count as usize };
    if count == 0 || count > 512 || count
        .checked_mul(mem::size_of::<SidAndAttributes>())
        .and_then(|n| n.checked_add(offset))
        .is_none_or(|n| n > returned as usize)
    { return Err(denied("Truncated token groups table")); }
    let table = unsafe { words.as_ptr().cast::<u8>().add(offset).cast::<SidAndAttributes>() };
    let mut result = Vec::with_capacity(count);
    for index in 0..count {
        // SAFETY: table count and native header span were bounds checked.
        let group = unsafe { *table.add(index) };
        result.push(Group {
            sid: copy_bounded_sid(group.sid, &words, returned)?,
            attributes: group.attributes,
        });
    }
    Ok(result)
}

/// Canonical TokenUser reader shared by the native SCM identity owner
/// and the service IPC last-message inspector. All SID pointers are
/// required to lie entirely inside GetTokenInformation's output buffer.
/// Never serialize or log the returned bytes.
pub(super) fn verified_token_user_sid(token: H) -> io::Result<Vec<u8>> {
    if token.is_null() {
        return Err(denied("No native token for User SID"));
    }
    let mut needed = 0u32;
    // SAFETY: Zero-capacity size probe for TOKEN_USER only.
    let first = unsafe {
        GetTokenInformation(token, TOKEN_USER, ptr::null_mut(), 0, &mut needed)
    };
    if first != 0 || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
        || needed < mem::size_of::<TokenUser>() as u32 || needed > MAX_GROUP_BYTES
    {
        return Err(denied("Unusable native TokenUser capacity"));
    }
    let mut words = vec![0usize;
        (needed as usize).div_ceil(mem::size_of::<usize>())];
    let mut returned = needed;
    // SAFETY: Aligned storage and exact native buffer capacity, owned
    // until the bounded User SID has been copied.
    if unsafe {
        GetTokenInformation(
            token, TOKEN_USER, words.as_mut_ptr().cast(),
            needed, &mut returned,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if returned < mem::size_of::<TokenUser>() as u32 || returned > needed {
        return Err(denied("Truncated or expanded native TokenUser"));
    }
    // SAFETY: Native TOKEN_USER layout is present within the live output;
    // its SID pointer will be checked BEFORE any SID dereference.
    let sid = unsafe { (*(words.as_ptr().cast::<TokenUser>())).user.sid };
    copy_bounded_sid(sid, &words, returned)
}

/// Resolve the literal NT SERVICE account through the local Windows LSA,
/// not from a caller-supplied SID, PID, slot, or registry property.
fn expected_service_sid() -> io::Result<Vec<u8>> {
    let name: Vec<u16> = SERVICE_NAME.encode_utf16().chain(Some(0)).collect();
    let (mut sid_len, mut domain_len, mut use_type) = (0u32, 0u32, 0u32);
    let first = unsafe {
        LookupAccountNameW(
            ptr::null(), name.as_ptr(), ptr::null_mut(), &mut sid_len,
            ptr::null_mut(), &mut domain_len, &mut use_type,
        )
    };
    if first != 0 || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
        || sid_len != 32 || !(1..=256).contains(&domain_len)
    { return Err(denied("MIVUBI SCM service SID not resolvable")); }
    let mut sid_words = vec![0usize; (sid_len as usize).div_ceil(mem::size_of::<usize>())];
    let mut domain = vec![0u16; domain_len as usize];
    let original_capacity = sid_len;
    let success = unsafe {
        LookupAccountNameW(
            ptr::null(), name.as_ptr(), sid_words.as_mut_ptr().cast(), &mut sid_len,
            domain.as_mut_ptr(), &mut domain_len, &mut use_type,
        )
    };
    if success == 0 { return Err(io::Error::last_os_error()); }
    if sid_len != original_capacity { return Err(denied("Service SID changed size")); }
    let sid = unsafe {
        std::slice::from_raw_parts(sid_words.as_ptr().cast::<u8>(), sid_len as usize)
    }.to_vec();
    if !service_sid(&sid) { return Err(denied("Lookup returned a non-service SID")); }
    Ok(sid)
}

/// This MUST execute before creating the service-owned named pipe.
/// Missing/disabled/deny-only SID prevents the feature from starting.
pub(super) fn verified_current_service_sid() -> io::Result<String> {
    let expected = expected_service_sid()?;
    let mut raw = ptr::null_mut();
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if raw.is_null() { return Err(denied("No SCM process token")); }
    let token = Token(raw);
    if groups(token.0)?.iter().filter(|g| g.sid == expected && enabled(g.attributes)).count() != 1 {
        return Err(denied("SCM process lacks enabled MIVUBI service SID"));
    }
    // Only format the exact Windows-resolved S-1-5-80 SID.
    let mut result = String::from("S-1-5-80");
    for bytes in expected[12..].chunks_exact(4) {
        let n = u32::from_le_bytes(bytes.try_into().map_err(|_| denied("Invalid SID bytes"))?);
        result.push('-');
        result.push_str(&n.to_string());
    }
    Ok(result)
}

/// A unique enabled S-1-5-5-X-Y group must be marked SE_GROUP_LOGON_ID.
/// This is NOT authorization for a target desktop user's DACL.
pub(super) fn verified_token_logon_sid(token: H) -> io::Result<Vec<u8>> {
    let candidates: Vec<Group> = groups(token)?.into_iter()
        .filter(|g| g.attributes & SE_GROUP_LOGON_ID == SE_GROUP_LOGON_ID)
        .collect();
    if candidates.len() != 1 || !enabled(candidates[0].attributes)
        || !logon_sid(&candidates[0].sid)
    { return Err(denied("Missing unique enabled logon SID")); }
    Ok(candidates[0].sid.clone())
}

/// OS-active physical-console AND RDP sessions are candidates. A
/// WinStation name is never a security identity. No packet supplies IDs.
fn active_ids(records: &[(u32, i32)]) -> io::Result<Vec<u32>> {
    if records.is_empty() || records.len() > MAX_WTS_SESSIONS as usize {
        return Err(denied("Unbounded WTS session records"));
    }
    let mut seen = std::collections::HashSet::new();
    let mut active = Vec::new();
    for &(id, state) in records {
        if id == u32::MAX || !matches!(state, 0..=9) || !seen.insert(id) {
            return Err(denied("Malformed or duplicated WTS session"));
        }
        if state == WTS_ACTIVE {
            if id == 0 { return Err(denied("Service session zero cannot be interactive")); }
            active.push(id);
        }
    }
    if active.is_empty() || active.len() > MAX_ACTIVE_SESSIONS {
        return Err(denied("No bounded active WTS sessions"));
    }
    active.sort_unstable();
    Ok(active)
}

/// Enumerate only the local RD host (WTS_CURRENT_SERVER_HANDLE == NULL).
/// Buffers are always returned to the WTS allocator, not LocalFree.
/// No station names, usernames or desktop content are read.
fn enumerate_active_ids() -> io::Result<Vec<u32>> {
    let mut sessions: *mut WtsSessionInfo = ptr::null_mut();
    let mut count = 0u32;
    // SAFETY: Native API uses Reserved=0, Version=1.
    let success = unsafe {
        WTSEnumerateSessionsW(ptr::null_mut(), 0, 1, &mut sessions, &mut count)
    };
    let owned = WtsMemory(sessions);
    if success == 0 { return Err(io::Error::last_os_error()); }
    if owned.0.is_null() || count == 0 || count > MAX_WTS_SESSIONS
        || (owned.0 as usize) % mem::align_of::<WtsSessionInfo>() != 0
    {
        return Err(denied("Invalid WTS session allocation"));
    }
    // SAFETY: Windows returns count contiguous WTS_SESSION_INFOW values;
    // count and alignment are bounded. WinStation string is never accessed.
    let rows = unsafe { std::slice::from_raw_parts(owned.0, count as usize) };
    let facts: Vec<_> = rows.iter().map(|r| (r.id, r.state)).collect();
    active_ids(&facts)
}

/// Inspect a real peer process TOKEN, never a self-reported session or PID.
/// The existing last-message SID/session/AuthenticationId checks remain
/// mandatory. This function is negative evidence ONLY, not IPC admission.
/// Windows requires LocalSystem plus SeTcbPrivilege for WTSQueryUserToken.
/// Missing privilege or unsupported OS session fails closed.
pub(super) fn verify_active_windows_logon(peer_token: H) -> io::Result<()> {
    let verified_service_sid = verified_current_service_sid()?;
    let session = token_dword(peer_token, TOKEN_SESSION_ID)?;
    if session == 0 || session == u32::MAX {
        return Err(denied("Pipe peer token is not interactive"));
    }
    // A process has a primary token; a thread impersonation token must
    // not masquerade as the OS-connected process for WTS enrollment.
    if token_dword(peer_token, TOKEN_TYPE)? != TOKEN_PRIMARY {
        return Err(denied("Pipe peer lacks a primary Windows process token"));
    }
    let peer_logon = verified_token_logon_sid(peer_token)?;
    let peer_user = verified_token_user_sid(peer_token)?;
    let before = enumerate_active_ids()?;
    if before.binary_search(&session).is_err() {
        return Err(denied("Connected process session is not WTSActive"));
    }
    let mut raw = ptr::null_mut();
    // SAFETY: The session comes from a real peer OS token AND the local
    // WTSActive inventory, not a caller request.
    let success = unsafe { WTSQueryUserToken(session, &mut raw) };
    let user = Token(raw);
    if success == 0 { return Err(io::Error::last_os_error()); }
    if user.0.is_null()
        || token_dword(user.0, TOKEN_TYPE)? != TOKEN_PRIMARY
        || token_dword(user.0, TOKEN_SESSION_ID)? != session
        || verified_token_logon_sid(user.0)? != peer_logon
        || verified_token_user_sid(user.0)? != peer_user
    {
        return Err(denied("WTS primary logon differs from OS peer token"));
    }
    // Detect a changing active session set. Reusing a session ID between
    // observations is still possible; this NEVER becomes authorization.
    if enumerate_active_ids()? != before {
        return Err(denied("Active Windows session inventory changed"));
    }
    // WTSActive alone does not prevent a logoff/relogon under an unchanged
    // numerical session ID. Re-read the Windows primary logon token and
    // require the SAME user and unique logon SID before preparing policy.
    let mut second_raw = ptr::null_mut();
    let fresh = unsafe { WTSQueryUserToken(session, &mut second_raw) };
    let rechecked_user = Token(second_raw);
    if fresh == 0 {
        return Err(io::Error::last_os_error());
    }
    if rechecked_user.0.is_null()
        || token_dword(rechecked_user.0, TOKEN_TYPE)? != TOKEN_PRIMARY
        || token_dword(rechecked_user.0, TOKEN_SESSION_ID)? != session
        || verified_token_user_sid(rechecked_user.0)? != peer_user
        || verified_token_logon_sid(rechecked_user.0)? != peer_logon
    {
        return Err(denied("Interactive Windows logon changed during observation"));
    }
    // Build the never-mounted policy from the independently verified OS
    // service SID and WTS user's exact logon SID. Convert it to a native
    // descriptor and ask the real Windows AccessCheck algorithm about
    // minimal permitted rights and forbidden client server-instance rights.
    // These facts are diagnostic ONLY; no pipe DACL or grant is changed.
    let candidate = scoped_logon_pipe_acl_candidate(
        &verified_service_sid, &peer_logon,
    )?;
    review_candidate_effective_rights(
        &candidate, rechecked_user.0, &before, session, &peer_logon,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn service_sid_must_have_exact_service_authority() {
        let mut sid = vec![1, 6, 0, 0, 0, 0, 0, 5];
        for n in [80u32, 1, 2, 3, 4, 5] { sid.extend_from_slice(&n.to_le_bytes()); }
        assert!(service_sid(&sid)); assert!(!logon_sid(&sid));
        sid[8] = 81; assert!(!service_sid(&sid));
    }
    #[test]
    fn logon_sid_requires_enabled_non_deny_only_group() {
        let mut sid = vec![1, 3, 0, 0, 0, 0, 0, 5];
        for n in [5u32, 22, 33] { sid.extend_from_slice(&n.to_le_bytes()); }
        assert!(logon_sid(&sid)); assert!(!service_sid(&sid));
        assert!(enabled(SE_GROUP_LOGON_ID | SE_GROUP_ENABLED));
        assert!(!enabled(SE_GROUP_LOGON_ID | SE_GROUP_DENY_ONLY));
        assert!(!enabled(SE_GROUP_LOGON_ID));
    }
    #[test]
    fn token_sid_reader_rejects_foreign_and_truncated_native_pointers() {
        let mut words = [0usize; 8];
        let base = words.as_mut_ptr().cast::<u8>();
        let capacity = mem::size_of_val(&words) as u32;
        // Neither a null pointer nor a foreign allocation is a valid
        // TOKEN_USER/TOKEN_GROUPS SID, even if Windows reports success.
        assert!(copy_bounded_sid(ptr::null_mut(), &words, capacity).is_err());
        let foreign = ((base as usize) - 1) as H;
        assert!(copy_bounded_sid(foreign, &words, capacity).is_err());
        // A header that fits but whose complete SID extends past
        // ReturnLength must be rejected BEFORE calling IsValidSid.
        let native_sid: [u8; 12] = [1, 1, 0, 0, 0, 0, 0, 5, 18, 0, 0, 0];
        // SAFETY: offset 16 plus 12 bytes fits the 64-byte word buffer.
        let pointer = unsafe { base.add(16) };
        unsafe { ptr::copy_nonoverlapping(native_sid.as_ptr(), pointer, native_sid.len()) };
        assert!(copy_bounded_sid(pointer.cast(), &words, 27).is_err());
        assert_eq!(
            copy_bounded_sid(pointer.cast(), &words, 28).unwrap(),
            native_sid,
        );
        assert!(copy_bounded_sid(pointer.cast(), &words, capacity + 1).is_err());
    }

    #[test]
    fn native_token_user_layout_is_one_sid_and_attributes_pair() {
        assert_eq!(mem::size_of::<TokenUser>(), mem::size_of::<SidAndAttributes>());
        assert_eq!(mem::offset_of!(TokenUser, user), 0);
    }

    #[test]
    fn windows_accesscheck_probes_scoped_logon_and_refuses_pipe_instance() {
        // Independent native fixture: does not need an installed service.
        // Runs under the current interactive Windows user token, if the
        // test environment actually supplies an enabled logon SID.
        let mut raw = ptr::null_mut();
        // SAFETY: Query/duplicate the real current TEST process token only.
        if unsafe {
            OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY | TOKEN_DUPLICATE, &mut raw)
        } == 0 {
            panic!("Windows test cannot open current process token");
        }
        assert!(!raw.is_null());
        let primary = Token(raw);
        let logon = verified_token_logon_sid(primary.0).unwrap();
        let sddl = scoped_logon_pipe_acl_candidate(
            "S-1-5-80-10-20-30-40-50", &logon,
        ).unwrap();
        let desc = candidate_descriptor(&sddl).unwrap();
        let client = duplicate_for_access_check(primary.0).unwrap();
        assert!(native_grants(&desc, &client, SCOPED_CLIENT_PIPE_RIGHTS).unwrap());
        assert!(!native_grants(&desc, &client, FILE_CREATE_PIPE_INSTANCE).unwrap());
    }

    #[test]
    fn native_acl_parser_does_not_accept_ownerless_or_daclless_descriptors() {
        assert!(candidate_descriptor("D:P(A;;GA;;;SY)").is_err());
        assert!(candidate_descriptor("O:SYG:SY").is_err());
        // A valid candidate includes explicit owner/group and a protected
        // non-NULL DACL. It is never installed on a named pipe here.
        let service = "S-1-5-80-10-20-30-40-50";
        let mut logon = vec![1, 3, 0, 0, 0, 0, 0, 5];
        for rid in [5u32, 6, 7] { logon.extend_from_slice(&rid.to_le_bytes()); }
        let sddl = scoped_logon_pipe_acl_candidate(service, &logon).unwrap();
        assert!(candidate_descriptor(&sddl).is_ok());
    }

    #[test]
    fn scoped_pipe_candidate_allows_only_one_logon_to_write_data() {
        let service = "S-1-5-80-10-20-30-40-50";
        let mut logon = vec![1, 3, 0, 0, 0, 0, 0, 5];
        for rid in [5u32, 1234, 5678] {
            logon.extend_from_slice(&rid.to_le_bytes());
        }
        let policy = scoped_logon_pipe_acl_candidate(service, &logon).unwrap();
        assert_eq!(
            policy,
            "O:SYG:SYD:P(A;;GA;;;SY)(A;;GA;;;S-1-5-80-10-20-30-40-50)(A;;0x00100002;;;S-1-5-5-1234-5678)",
        );
        assert_eq!(SCOPED_CLIENT_PIPE_RIGHTS & FILE_CREATE_PIPE_INSTANCE, 0);
        assert_eq!(SCOPED_CLIENT_PIPE_RIGHTS, 0x0010_0002);
        assert!(!policy.contains(";;;BA"));
        assert!(!policy.contains(";;;BU"));
        assert!(!policy.contains(";;;AU"));
        assert!(!policy.contains(";;;WD"));
    }

    #[test]
    fn scoped_pipe_candidate_rejects_forged_or_cross_logon_sids() {
        let good_service = "S-1-5-80-10-20-30-40-50";
        let mut logon = vec![1, 3, 0, 0, 0, 0, 0, 5];
        for rid in [5u32, 1234, 5678] {
            logon.extend_from_slice(&rid.to_le_bytes());
        }
        for service in [
            "", "S-1-5-18", "S-1-5-80-10-20-30-40",
            "S-1-5-80-10-20-30-40-50-60",
            "S-1-5-80-10-20-30-40-50(A;;GA;;;WD)",
            "S-1-5-80-01-20-30-40-50",
        ] {
            assert!(scoped_logon_pipe_acl_candidate(service, &logon).is_err());
        }
        let mut forged_logon = logon.clone();
        forged_logon[8] = 6;
        assert!(scoped_logon_pipe_acl_candidate(good_service, &forged_logon).is_err());
        assert!(scoped_logon_pipe_acl_candidate(good_service, &logon[..19]).is_err());
        assert!(scoped_logon_pipe_acl_candidate(good_service, &[1, 1, 0, 0, 0, 0, 0, 5, 18, 0, 0, 0]).is_err());
        // A different verified logon produces a different, nonreusable DACL.
        let mut next_logon = logon.clone();
        next_logon[16..20].copy_from_slice(&5679u32.to_le_bytes());
        let first = scoped_logon_pipe_acl_candidate(good_service, &logon).unwrap();
        let second = scoped_logon_pipe_acl_candidate(good_service, &next_logon).unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn directory_root_candidate_has_system_owner_and_exact_service_only() {
        let expected = "S-1-5-80-10-20-30-40-50";
        assert_eq!(service_root_acl_candidate(expected).unwrap(),
            "O:SYG:SYD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;S-1-5-80-10-20-30-40-50)");
        for bad in [
            "", "S-1-5-18", "S-1-5-80-10-20-30-40",
            "S-1-5-80-10-20-30-40-50(A;;GA;;;WD)",
            "S-1-5-80-010-20-30-40-50",
        ] {
            assert!(service_root_acl_candidate(bad).is_err());
        }
        let sddl = service_root_acl_candidate(expected).unwrap();
        assert!(!sddl.contains(";;;BA"));
        assert!(!sddl.contains(";;;WD"));
        assert!(!sddl.contains(";;;AU"));
        assert!(!sddl.contains(";;;BU"));
        let descriptor = candidate_descriptor(&sddl).unwrap();
        let mut actual_service = vec![1, 6, 0, 0, 0, 0, 0, 5];
        for rid in [80u32, 10, 20, 30, 40, 50] {
            actual_service.extend_from_slice(&rid.to_le_bytes());
        }
        let mut dacl: H = ptr::null_mut();
        let mut present = 0i32;
        let mut defaulted = 0i32;
        assert_ne!(unsafe {
            GetSecurityDescriptorDacl(
                descriptor.0, &mut present, &mut dacl, &mut defaulted,
            )
        }, 0);
        assert_eq!(present, 1);
        assert!(exact_service_root_acl(dacl, &actual_service).is_ok());
    }

    #[test]
    fn root_policy_rejects_extra_or_changed_acl_principals_and_flags() {
        let service = "S-1-5-80-10-20-30-40-50";
        let mut sid = vec![1, 6, 0, 0, 0, 0, 0, 5];
        for rid in [80u32, 10, 20, 30, 40, 50] {
            sid.extend_from_slice(&rid.to_le_bytes());
        }
        for invalid in [
            format!("O:SYG:SYD:P(A;OICI;FA;;;SY)(A;OICI;FA;;;{service})(A;;FA;;;BA)"),
            format!("O:SYG:SYD:P(A;;FA;;;SY)(A;OICI;FA;;;{service})"),
            format!("O:SYG:SYD:P(A;OICI;FR;;;SY)(A;OICI;FA;;;{service})"),
            format!("O:SYG:SYD:P(A;OICI;FA;;;SY)(D;OICI;FA;;;{service})"),
        ] {
            let descriptor = candidate_descriptor(&invalid).unwrap();
            let mut dacl: H = ptr::null_mut();
            let mut present = 0i32;
            let mut defaulted = 0i32;
            assert_ne!(unsafe {
                GetSecurityDescriptorDacl(
                    descriptor.0, &mut present, &mut dacl, &mut defaulted,
                )
            }, 0);
            assert_ne!(present, 0);
            assert!(exact_service_root_acl(dacl, &sid).is_err());
        }
        assert!(root_access(
            &candidate_descriptor(&service_root_acl_candidate(service).unwrap()).unwrap(),
            &Token(ptr::null_mut()), ROOT_FILE_ALL_ACCESS,
        ).is_err());
    }

    #[test]
    fn fresh_service_root_name_is_fixed_and_cannot_accept_external_paths() {
        assert!(checked_service_root_name(SERVICE_ROOT_CHILD).is_ok());
        for forged in [
            "", "..", ".", "mc02", "mivubi-independent-root/",
            "../mivubi-independent-root", "C:\\somewhere",
            "mivubi-independent-root\\other",
            "mivubi-independent-root ", "mivubi-independent-root.",
            "MIVUBI-INDEPENDENT-ROOT",
            "NT SERVICE\\MIVUBIIndependentHost",
        ] {
            assert!(checked_service_root_name(forged).is_err(),
                "Unexpected service-root path accepted: {forged}");
        }
    }

    #[test]
    fn os_knownfolder_path_rejects_unc_relative_device_and_traversal() {
        assert_eq!(expected_known_folder_final_path("C:\\ProgramData").unwrap(),
            "\\\\?\\C:\\ProgramData");
        assert_eq!(expected_known_folder_final_path("D:\\Machine\\Data").unwrap(),
            "\\\\?\\D:\\Machine\\Data");
        for path in [
            "", "ProgramData", "C:ProgramData", "C:/ProgramData",
            "\\\\server\\share", "\\\\?\\C:\\ProgramData",
            "C:\\ProgramData\\..\\Users", "C:\\ProgramData\\.\\Other",
            "C:\\ProgramData\\", "C:\\ProgramData ",
            "C:\\ProgramData.\\Other", "C:\\Data:Stream",
            "C:\\éxtra",
        ] {
            assert!(expected_known_folder_final_path(path).is_err(),
                "Ambiguous ProgramData path accepted: {path}");
        }
    }

    #[test]
    fn fixed_installer_parent_cannot_be_created_or_reused_as_service_root() {
        assert_ne!(INSTALLER_PARENT_CHILD, SERVICE_ROOT_CHILD);
        assert_eq!(PROGRAM_DATA_KNOWN_FOLDER.data1, 0x62AB_5D82);
        assert_eq!(PROGRAM_DATA_KNOWN_FOLDER.data2, 0xFDC1);
        assert_eq!(PROGRAM_DATA_KNOWN_FOLDER.data3, 0x4DC3);
        assert_eq!(PROGRAM_DATA_KNOWN_FOLDER.data4,
            [0xA9, 0xDD, 0x07, 0x0D, 0x1D, 0x49, 0x5D, 0x97]);
        // Parent never gets a constructor: the file must ALREADY exist
        // with the SYSTEM/service ACL and exact expected native FILE_ID.
        // No installer-path, CLI or client profile API is exposed here.
        let source = include_str!("service_identity.rs");
        assert!(source.contains("relative_installer_parent(programdata: &File)"));
        assert!(source.contains("child == INSTALLER_PARENT_CHILD && disposition == NT_FILE_OPEN"));
        assert!(source.contains("verify_installer_parent_link(&programdata, &parent)?;"));
    }

    #[test]
    fn nt_create_disposition_requires_created_not_existing_objects() {
        assert!(nt_root_create_outcome(0, NT_FILE_CREATED).is_ok());
        for existing in [0usize, 1, 3, 4, 5] {
            assert_eq!(nt_root_create_outcome(0, existing).unwrap_err().kind(),
                io::ErrorKind::PermissionDenied);
        }
        assert_eq!(
            nt_root_create_outcome(NT_STATUS_OBJECT_NAME_COLLISION as i32, 0)
                .unwrap_err().kind(), io::ErrorKind::AlreadyExists,
        );
        assert_eq!(
            nt_root_create_outcome(NT_STATUS_OBJECT_NAME_NOT_FOUND as i32, 0)
                .unwrap_err().kind(), io::ErrorKind::NotFound,
        );
        assert_eq!(
            nt_root_create_outcome(NT_STATUS_ACCESS_DENIED as i32, 0)
                .unwrap_err().kind(), io::ErrorKind::PermissionDenied,
        );
    }

    #[test]
    fn active_console_and_rdp_records_are_candidates_without_names() {
        let rows = [
            (0, WTS_LISTEN), (4, WTS_ACTIVE), (7, WTS_DISCONNECTED),
            (2, WTS_ACTIVE), (6, WTS_CONNECTED),
        ];
        assert_eq!(active_ids(&rows).unwrap(), vec![2, 4]);
    }
    #[test]
    fn invalid_or_nonactive_sessions_cannot_be_admitted() {
        assert!(active_ids(&[]).is_err());
        assert!(active_ids(&[(0, WTS_ACTIVE)]).is_err());
        assert!(active_ids(&[(u32::MAX, WTS_ACTIVE)]).is_err());
        assert!(active_ids(&[(2, WTS_ACTIVE), (2, WTS_LISTEN)]).is_err());
        assert!(active_ids(&[(2, -1)]).is_err());
        assert!(active_ids(&[(3, WTS_DISCONNECTED)]).is_err());
        let many: Vec<_> = (1..=17).map(|id| (id, WTS_ACTIVE)).collect();
        assert!(active_ids(&many).is_err());
    }
    #[test]
    fn token_groups_array_is_pointer_aligned() {
        assert_eq!(mem::offset_of!(TokenGroups, groups), mem::size_of::<usize>());
    }
}
