//! Read-only native SCM service-SID and Windows logon-SID evidence.
//! No game launch, credential access, token modification or pipe policy
//! promotion. The opt-in pilot remains disabled by default.

use std::{ffi::c_void, io, mem, ptr};
type H = *mut c_void;
const TOKEN_QUERY: u32 = 8;
const TOKEN_GROUPS: u32 = 2;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const SE_GROUP_ENABLED: u32 = 4;
const SE_GROUP_DENY_ONLY: u32 = 0x10;
const SE_GROUP_LOGON_ID: u32 = 0xC000_0000;
const MAX_GROUP_BYTES: u32 = 65_536;
const SERVICE_NAME: &str = "NT SERVICE\\MIVUBIIndependentHost";

#[repr(C)]
#[derive(Clone, Copy)]
struct SidAndAttributes { sid: H, attributes: u32 }
#[repr(C)]
struct TokenGroups { count: u32, groups: [SidAndAttributes; 1] }
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
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> H;
    fn GetLastError() -> u32;
    fn CloseHandle(handle: H) -> i32;
}
struct Token(H);
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
    let base = words.as_ptr() as usize;
    let end = base.checked_add(returned as usize)
        .ok_or_else(|| denied("Group address overflow"))?;
    let table = unsafe { words.as_ptr().cast::<u8>().add(offset).cast::<SidAndAttributes>() };
    let mut result = Vec::with_capacity(count);
    for index in 0..count {
        let group = unsafe { *table.add(index) };
        let start = group.sid as usize;
        if start < base || start.checked_add(8).is_none_or(|n| n > end) {
            return Err(denied("SID header outside returned buffer"));
        }
        let hdr = unsafe { std::slice::from_raw_parts(group.sid.cast::<u8>(), 8) };
        if hdr[0] != 1 || hdr[1] > 15 { return Err(denied("Invalid SID header")); }
        let len = 8 + usize::from(hdr[1]) * 4;
        if start.checked_add(len).is_none_or(|n| n > end) {
            return Err(denied("SID bytes outside returned buffer"));
        }
        // SAFETY: Both SID header and complete SID are verified in-bounds.
        if unsafe { IsValidSid(group.sid) } == 0
            || unsafe { GetLengthSid(group.sid) } as usize != len
        { return Err(denied("Invalid OS token SID")); }
        result.push(Group {
            sid: unsafe { std::slice::from_raw_parts(group.sid.cast::<u8>(), len) }.to_vec(),
            attributes: group.attributes,
        });
    }
    Ok(result)
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
    fn token_groups_array_is_pointer_aligned() {
        assert_eq!(mem::offset_of!(TokenGroups, groups), mem::size_of::<usize>());
    }
}
