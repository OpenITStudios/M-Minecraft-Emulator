//! Read-only native SCM service-SID and Windows logon-SID evidence.
//! No game launch, credential access, token modification or pipe policy
//! promotion. The opt-in pilot remains disabled by default.

use std::{ffi::c_void, io, mem, ptr};
type H = *mut c_void;
const TOKEN_QUERY: u32 = 8;
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

#[repr(C)]
#[derive(Clone, Copy)]
struct SidAndAttributes { sid: H, attributes: u32 }
#[repr(C)]
struct TokenGroups { count: u32, groups: [SidAndAttributes; 1] }
#[repr(C)]
struct TokenUser { user: SidAndAttributes }
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
        "D:P(A;;GA;;;SY)(A;;GA;;;{verified_service})(A;;0x{SCOPED_CLIENT_PIPE_RIGHTS:08X};;;{scoped_principal})"
    ))
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
    // The policy is a negative/diagnostic preflight ONLY. No ACL is
    // installed, no pipe is created, and the resulting string is dropped.
    let _unused_policy = scoped_logon_pipe_acl_candidate(
        &verified_service_sid, &peer_logon,
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
    fn scoped_pipe_candidate_allows_only_one_logon_to_write_data() {
        let service = "S-1-5-80-10-20-30-40-50";
        let mut logon = vec![1, 3, 0, 0, 0, 0, 0, 5];
        for rid in [5u32, 1234, 5678] {
            logon.extend_from_slice(&rid.to_le_bytes());
        }
        let policy = scoped_logon_pipe_acl_candidate(service, &logon).unwrap();
        assert_eq!(
            policy,
            "D:P(A;;GA;;;SY)(A;;GA;;;S-1-5-80-10-20-30-40-50)(A;;0x00100002;;;S-1-5-5-1234-5678)",
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
