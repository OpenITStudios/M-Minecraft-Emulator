//! Independent Windows one-connection, inbound-only named-pipe fixture.
//!
//! This is original Win32 code with an explicit protected DACL, restricted
//! to a fresh disposable TEST pipe owned by the current Windows user.
//! No privileged service is installed, no production endpoint is created,
//! and no Sandboxie binary/driver/DLL is loaded. It cannot launch games.
//!
//! IMPORTANT: A pipe name, same-user SID, claimed PID/FILETIME, correlation
//! ID or impersonation result is not authentication for a privileged
//! broker. The fixture only reads one 32-byte diagnostic packet, examines
//! the last-message token, and returns pending facts. A future service
//! requires a distinct service SID, per-logon access policy, bounded
//! OVERLAPPED cancellation and an independently reviewed trust model.

#![allow(dead_code)]

use std::{
    ffi::{c_void, OsString},
    io,
    mem,
    os::windows::ffi::{OsStrExt, OsStringExt},
    ptr,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use virtual_clients_core::independent_engine::{
    decode_broker_probe, BrokerProbeKind, SessionSlot, BROKER_PROBE_BYTES,
};
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE},
    System::Threading::{GetCurrentProcess, GetCurrentThread},
};
use super::independent_pipe_peer::inspect_connected_server_pipe;

const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_USER: u32 = 1;
const TOKEN_SESSION_ID: u32 = 12;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const ERROR_NO_TOKEN: u32 = 1008;
const ERROR_PIPE_CONNECTED: u32 = 535;
const PIPE_ACCESS_INBOUND: u32 = 0x0000_0001;
const FILE_FLAG_FIRST_PIPE_INSTANCE: u32 = 0x0008_0000;
const PIPE_TYPE_MESSAGE: u32 = 0x0000_0004;
const PIPE_READMODE_MESSAGE: u32 = 0x0000_0002;
const PIPE_REJECT_REMOTE_CLIENTS: u32 = 0x0000_0008;

#[repr(C)]
struct SidAndAttributes {
    sid: *mut c_void,
    attributes: u32,
}

#[repr(C)]
struct TokenUser {
    user: SidAndAttributes,
}

#[repr(C)]
struct SecurityAttributes {
    size: u32,
    security_descriptor: *mut c_void,
    inherit_handle: i32,
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(process: HANDLE, access: u32, token: *mut HANDLE) -> i32;
    fn OpenThreadToken(thread: HANDLE, access: u32, open_as_self: i32, token: *mut HANDLE) -> i32;
    fn GetTokenInformation(
        token: HANDLE, class: u32, information: *mut c_void,
        information_size: u32, required: *mut u32,
    ) -> i32;
    fn IsValidSid(sid: *mut c_void) -> i32;
    fn GetLengthSid(sid: *mut c_void) -> u32;
    fn ConvertSidToStringSidW(sid: *mut c_void, text: *mut *mut u16) -> i32;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16, revision: u32, descriptor: *mut *mut c_void,
        size: *mut u32,
    ) -> i32;
    fn ImpersonateNamedPipeClient(pipe: HANDLE) -> i32;
    fn RevertToSelf() -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
    fn lstrlenW(value: *const u16) -> i32;
    fn CreateNamedPipeW(
        name: *const u16, open_mode: u32, pipe_mode: u32,
        max_instances: u32, out_buffer: u32, in_buffer: u32,
        timeout_ms: u32, attributes: *const SecurityAttributes,
    ) -> HANDLE;
    fn GetNamedPipeClientProcessId(pipe: HANDLE, process_id: *mut u32) -> i32;
    fn ConnectNamedPipe(pipe: HANDLE, overlapped: *mut c_void) -> i32;
    fn ReadFile(
        file: HANDLE, buffer: *mut c_void, amount: u32,
        bytes_read: *mut u32, overlapped: *mut c_void,
    ) -> i32;
    fn DisconnectNamedPipe(pipe: HANDLE) -> i32;
}

struct OwnedHandle(HANDLE);
impl OwnedHandle {
    fn from_raw(value: HANDLE) -> io::Result<Self> {
        if value.is_null() || value == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(value))
        }
    }
}
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            // SAFETY: This Rust value uniquely owns the returned Windows HANDLE.
            unsafe { CloseHandle(self.0) };
        }
    }
}

struct LocalBuffer(*mut c_void);
impl Drop for LocalBuffer {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Both SDDL conversion functions allocate with LocalAlloc.
            let _ = unsafe { LocalFree(self.0) };
        }
    }
}

fn token_from_process() -> io::Result<OwnedHandle> {
    let mut raw: HANDLE = ptr::null_mut();
    // SAFETY: GetCurrentProcess returns a pseudo handle; OpenProcessToken
    // supplies a separately owned query-only token handle.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw) } == 0 {
        return Err(io::Error::last_os_error());
    }
    OwnedHandle::from_raw(raw)
}

fn token_user_sid(token: HANDLE) -> io::Result<Vec<u8>> {
    let mut needed = 0u32;
    // SAFETY: Standard size query for TOKEN_USER; no output buffer yet.
    let first = unsafe {
        GetTokenInformation(token, TOKEN_USER, ptr::null_mut(), 0, &mut needed)
    };
    if first != 0 || unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER
        || needed < mem::size_of::<TokenUser>() as u32 || needed > 65_536
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Native token SID size unavailable",
        ));
    }
    // Align TOKEN_USER pointers by storing its bytes in usize words.
    let words = (needed as usize).div_ceil(mem::size_of::<usize>());
    let mut data = vec![0usize; words];
    // SAFETY: Correct alignment, capacity and mutable caller-owned buffer.
    if unsafe {
        GetTokenInformation(
            token, TOKEN_USER, data.as_mut_ptr().cast(),
            needed, &mut needed,
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if needed < mem::size_of::<TokenUser>() as u32 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Short token user"));
    }
    // SAFETY: TOKEN_USER is the documented layout at the head of the
    // aligned returned OS allocation. IsValidSid guards SID shape.
    let sid = unsafe { (*(data.as_ptr().cast::<TokenUser>())).user.sid };
    if sid.is_null() || unsafe { IsValidSid(sid) } == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid token user SID"));
    }
    let size = unsafe { GetLengthSid(sid) } as usize;
    if !(8..=68).contains(&size) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported SID length"));
    }
    // SAFETY: OS verified the SID pointer and returned its exact bounded
    // length. Only a private byte copy is kept; it is not exported/logged.
    Ok(unsafe { std::slice::from_raw_parts(sid.cast::<u8>(), size) }.to_vec())
}

fn token_session(token: HANDLE) -> io::Result<u32> {
    let mut session = 0u32;
    let mut used = 0u32;
    // SAFETY: Session ID output is exactly one u32.
    if unsafe {
        GetTokenInformation(
            token, TOKEN_SESSION_ID, (&mut session as *mut u32).cast(),
            mem::size_of::<u32>() as u32, &mut used,
        )
    } == 0 || used != mem::size_of::<u32>() as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Windows token session unavailable",
        ));
    }
    Ok(session)
}

fn current_sid_string() -> io::Result<String> {
    let current = token_from_process()?;
    let sid = token_user_sid(current.0)?;
    let mut text: *mut u16 = ptr::null_mut();
    // SAFETY: SID bytes are valid according to Windows; the returned UTF-16
    // allocation has exactly one LocalFree owner.
    if unsafe {
        ConvertSidToStringSidW(sid.as_ptr() as *mut c_void, &mut text)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    let _text = LocalBuffer(text.cast());
    if text.is_null() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty SID conversion"));
    }
    let len = unsafe { lstrlenW(text) };
    if !(5..=256).contains(&len) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Unsupported SID text"));
    }
    // SAFETY: lstrlenW found the terminator in the OS-provided text.
    let value = String::from_utf16(unsafe {
        std::slice::from_raw_parts(text, len as usize)
    }).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid SID UTF-16"))?;
    if !value.starts_with("S-1-") || !value.chars().all(|c|
        c.is_ascii_alphanumeric() || c == '-'
    ) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected SID text"));
    }
    Ok(value)
}

/// Build a protected DACL. No NULL/default descriptor, Everyone,
/// Anonymous or remote-user grants. The current-user FULL access is
/// **fixture only**: it does not distinguish two same-user Minecrafts.
fn fixture_descriptor() -> io::Result<LocalBuffer> {
    let user_sid = current_sid_string()?;
    // SYSTEM and Administrators plus the *exact current user* may access
    // this disposable test endpoint. Production must instead use a
    // protected service SID and separate minimal client grants.
    let sddl = format!("D:P(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;{user_sid})");
    let wide: Vec<u16> = sddl.encode_utf16().chain(std::iter::once(0)).collect();
    let mut raw: *mut c_void = ptr::null_mut();
    // SAFETY: Converter returns a self-relative security descriptor
    // allocated using LocalAlloc, freed after CreateNamedPipeW returns.
    if unsafe {
        ConvertStringSecurityDescriptorToSecurityDescriptorW(
            wide.as_ptr(), 1, &mut raw, ptr::null_mut(),
        )
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if raw.is_null() {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Missing pipe descriptor"));
    }
    Ok(LocalBuffer(raw))
}

/// Single-instance inbound, message-mode, local-only, test-owned pipe.
/// Not a Windows Service: no listener loop, no elevation, no registry
/// changes, no service control manager operation and no background task.
struct FixturePipeServer {
    handle: OwnedHandle,
    name: String,
}

impl FixturePipeServer {
    fn create() -> io::Result<Self> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let now = SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?.as_nanos();
        // Nonsecret, unique fixture name. NOT an authentication nonce;
        // FIRST_PIPE_INSTANCE refuses an already registered collision.
        let name = format!(
            r"\\.\pipe\mivubi-independent-fixture-{}-{now}-{}",
            std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let wide: Vec<u16> = OsString::from(&name).encode_wide()
            .chain(std::iter::once(0)).collect();
        let descriptor = fixture_descriptor()?;
        let attrs = SecurityAttributes {
            size: mem::size_of::<SecurityAttributes>() as u32,
            security_descriptor: descriptor.0,
            inherit_handle: 0,
        };
        // SAFETY: Protected DACL is non-NULL, the name is owned and
        // bounded, and CreateNamedPipeW copies the descriptor. Exactly
        // ONE instance is permitted; remote clients are rejected by OS.
        let raw = unsafe {
            CreateNamedPipeW(
                wide.as_ptr(),
                PIPE_ACCESS_INBOUND | FILE_FLAG_FIRST_PIPE_INSTANCE,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_REJECT_REMOTE_CLIENTS,
                1, 0, BROKER_PROBE_BYTES as u32, 50, &attrs,
            )
        };
        let handle = OwnedHandle::from_raw(raw)?;
        Ok(Self { handle, name })
    }

    /// The caller must ALREADY have a live, connected fixture client that
    /// wrote exactly one message. Never use blocking synchronous reads in a
    /// privileged service; it needs overlapped I/O, deadlines/cancellation.
    fn read_preconnected_once(&self) -> io::Result<PendingFixtureMessage> {
        // Avoid blocking ConnectNamedPipe in the unconnected case. This
        // process ID is a diagnostic, not message or caller authentication.
        let mut client_pid = 0u32;
        if unsafe {
            GetNamedPipeClientProcessId(self.handle.0, &mut client_pid)
        } == 0 || client_pid == 0 {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected, "Fixture has no connected pipe client",
            ));
        }
        // A client connected before ConnectNamedPipe: ERROR_PIPE_CONNECTED
        // is the documented success-equivalent. Never wait for a new client.
        let connected = unsafe { ConnectNamedPipe(self.handle.0, ptr::null_mut()) };
        if connected == 0 && unsafe { GetLastError() } != ERROR_PIPE_CONNECTED {
            return Err(io::Error::last_os_error());
        }
        let mut frame = [0u8; BROKER_PROBE_BYTES];
        let mut count = 0u32;
        // SAFETY: Inbound-only message-mode server; exact frame buffer
        // length and no OVERLAPPED pointer. Extra-long messages fail with
        // ERROR_MORE_DATA and are NEVER decoded as a valid packet.
        let ok = unsafe {
            ReadFile(
                self.handle.0, frame.as_mut_ptr().cast(),
                BROKER_PROBE_BYTES as u32, &mut count, ptr::null_mut(),
            )
        };
        if ok == 0 || count != BROKER_PROBE_BYTES as u32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData, "Non-exact fixture pipe message",
            ));
        }
        let untrusted = decode_broker_probe(&frame).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidData, "Invalid diagnostic wire request")
        })?;
        // OS PID + full creation FILETIME + recheck using the connected
        // server-side pipe handle. Still PENDING; no authorization granted.
        let peer = inspect_connected_server_pipe(self.handle.0, untrusted)?;
        // Read the impersonated identity of the LAST consumed message,
        // not the untrusted slot/PID fields. Always revert even on errors.
        let same_fixture_user = check_last_message_user(self.handle.0)?;
        if !same_fixture_user {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied, "Different logon identity or Windows session",
            ));
        }
        Ok(PendingFixtureMessage {
            kind: untrusted.kind,
            slot: untrusted.slot,
            os_client_process_matched: peer.claimed_identity_matches_os_client,
            same_fixture_user,
        })
    }
}

impl Drop for FixturePipeServer {
    fn drop(&mut self) {
        // SAFETY: Drop owns only this disposable server pipe instance.
        unsafe { DisconnectNamedPipe(self.handle.0) };
    }
}

struct RevertGuard;
impl Drop for RevertGuard {
    fn drop(&mut self) {
        // Failure to revert impersonation in a privileged server would be
        // catastrophic. Fail the entire process instead of continuing under
        // the requester's token. The current module is fixture-only.
        if unsafe { RevertToSelf() } == 0 {
            std::process::abort();
        }
    }
}

/// Refuse any already-impersonated server thread; enter impersonation
/// ONLY after the exact message was read. No privileged operations run
/// while the thread is impersonating the pipe client.
fn check_last_message_user(pipe: HANDLE) -> io::Result<bool> {
    let mut previous: HANDLE = ptr::null_mut();
    if unsafe {
        OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut previous)
    } != 0 {
        let _old = OwnedHandle::from_raw(previous)?;
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied, "Server thread already impersonating",
        ));
    }
    if !previous.is_null() {
        let _old = OwnedHandle::from_raw(previous)?;
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied, "Unexpected prior thread token",
        ));
    }
    if unsafe { GetLastError() } != ERROR_NO_TOKEN {
        return Err(io::Error::last_os_error());
    }
    if unsafe { ImpersonateNamedPipeClient(pipe) } == 0 {
        // No authorization or filesystem call follows a failed impersonation.
        return Err(io::Error::last_os_error());
    }
    let revert = RevertGuard;
    let comparison = (|| {
        let mut raw: HANDLE = ptr::null_mut();
        if unsafe {
            OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut raw)
        } == 0 {
            return Err(io::Error::last_os_error());
        }
        let impersonated = OwnedHandle::from_raw(raw)?;
        let current = token_from_process()?;
        let same_user = token_user_sid(impersonated.0)? == token_user_sid(current.0)?;
        let same_session = token_session(impersonated.0)? == token_session(current.0)?;
        Ok(same_user && same_session)
    })();
    drop(revert); // RevertToSelf before returning to the nonclient context.
    comparison
}

/// Sanitized, explicitly NON-authorizing fixture observations. No raw PID,
/// FILETIME, token, SID, endpoint name or message payload escapes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PendingFixtureMessage {
    kind: BrokerProbeKind,
    slot: SessionSlot,
    os_client_process_matched: bool,
    same_fixture_user: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::OpenOptions, io::Write};
    use virtual_clients_core::independent_engine::{
        encode_broker_probe, ProcessIdentity, UntrustedBrokerProbe,
    };
    use windows_sys::Win32::Foundation::FILETIME;
    use windows_sys::Win32::System::Threading::GetProcessTimes;

    fn current_probe() -> UntrustedBrokerProbe {
        // SAFETY: Query immutable creation time of the current test process.
        let mut creation: FILETIME = unsafe { mem::zeroed() };
        let mut exit: FILETIME = unsafe { mem::zeroed() };
        let mut kernel: FILETIME = unsafe { mem::zeroed() };
        let mut user: FILETIME = unsafe { mem::zeroed() };
        assert_ne!(
            unsafe {
                GetProcessTimes(
                    GetCurrentProcess(), &mut creation, &mut exit,
                    &mut kernel, &mut user,
                )
            }, 0
        );
        UntrustedBrokerProbe {
            kind: BrokerProbeKind::ObserveExactProcess,
            slot: SessionSlot::Mc02,
            claimed_process: ProcessIdentity::checked(
                std::process::id(),
                ((creation.dwHighDateTime as u64) << 32)
                    | creation.dwLowDateTime as u64,
            ).unwrap(),
            correlation_id: 1,
        }
    }

    #[test]
    fn local_read_only_pipe_fixture_verifies_exact_own_process_and_last_message() {
        let server = FixturePipeServer::create().unwrap();
        let mut client = OpenOptions::new().write(true).open(&server.name).unwrap();
        client.write_all(&encode_broker_probe(current_probe()).unwrap()).unwrap();
        let pending = server.read_preconnected_once().unwrap();
        assert_eq!(pending.kind, BrokerProbeKind::ObserveExactProcess);
        assert_eq!(pending.slot, SessionSlot::Mc02);
        assert!(pending.os_client_process_matched);
        assert!(pending.same_fixture_user);
        drop(client);
        // Drop closes the pipe; no game process, registry or service changed.
    }

    #[test]
    fn malformed_oversized_message_cannot_be_processed_as_valid_probe() {
        let server = FixturePipeServer::create().unwrap();
        let mut client = OpenOptions::new().write(true).open(&server.name).unwrap();
        let mut frame = Vec::from(encode_broker_probe(current_probe()).unwrap());
        frame.push(0); // one extra byte must cause ERROR_MORE_DATA.
        client.write_all(&frame).unwrap();
        assert_eq!(
            server.read_preconnected_once().unwrap_err().kind(),
            io::ErrorKind::InvalidData,
        );
    }

    #[test]
    fn forged_process_identity_is_rejected_before_token_promotion() {
        let server = FixturePipeServer::create().unwrap();
        let mut client = OpenOptions::new().write(true).open(&server.name).unwrap();
        let mut forged = current_probe();
        forged.claimed_process.created_file_time += 1;
        client.write_all(&encode_broker_probe(forged).unwrap()).unwrap();
        assert_eq!(
            server.read_preconnected_once().unwrap_err().kind(),
            io::ErrorKind::PermissionDenied,
        );
    }
}
