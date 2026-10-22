//! Source-only IPC ingress worker OWNED by the standalone SCM service.
//!
//! Compile-time feature readonly-ipc-pilot is OFF BY DEFAULT. This module
//! never installs a service, launches a game, opens credentials/profile data,
//! changes ACLs or sends replies. It temporarily impersonates the last
//! message's sender for read-only token checks, always RevertToSelf, and consumes
//! bounded, versioned READ-ONLY probes and compares OS PID/FILETIME.
//!
//! The pipe allows LOCAL SYSTEM and BUILTIN ADMINISTRATORS only, not normal
//! desktop users. The current pilot is therefore deliberately unsuitable
//! as the production Minecraft Manager channel. A future implementation
//! needs a service SID, restricted logon SID, token/session impersonation
//! checks, handles for the actual game rather than the calling desktop,
//! and signed install/rollback; DO NOT enable in release builds.

#![allow(dead_code)]

use std::{
    ffi::{c_void, OsStr},
    io,
    mem,
    os::windows::ffi::OsStrExt,
    ptr,
    sync::mpsc::{Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};
use virtual_clients_core::independent_engine::{
    decode_broker_probe, UntrustedBrokerProbe, BROKER_PROBE_BYTES,
};
use crate::service_identity::{
    verified_active_windows_logon, verified_current_service_sid,
    verified_token_logon_sid, verified_token_user_sid,
};

type NativeHandle = *mut c_void;
const INVALID_HANDLE_VALUE: NativeHandle = -1isize as NativeHandle;
const PIPE_ACCESS_INBOUND: u32 = 1;
const FILE_FLAG_FIRST_PIPE_INSTANCE: u32 = 0x0008_0000;
const FILE_FLAG_OVERLAPPED: u32 = 0x4000_0000;
const PIPE_TYPE_MESSAGE: u32 = 4;
const PIPE_READMODE_MESSAGE: u32 = 2;
const PIPE_REJECT_REMOTE_CLIENTS: u32 = 8;
const ERROR_IO_PENDING: u32 = 997;
const ERROR_PIPE_CONNECTED: u32 = 535;
const ERROR_MORE_DATA: u32 = 234;
const WAIT_OBJECT_0: u32 = 0;
const WAIT_TIMEOUT: u32 = 258;
const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
const SYNCHRONIZE: u32 = 0x0010_0000;
const CONNECTION_DEADLINE: Duration = Duration::from_millis(500);
const READ_DEADLINE: Duration = Duration::from_millis(500);
const WAKE_SLICE_MS: u32 = 50;
const TOKEN_QUERY: u32 = 0x0008;
const TOKEN_IMPERSONATION_LEVEL: u32 = 9;
const TOKEN_STATISTICS: u32 = 10;
const TOKEN_SESSION_ID: u32 = 12;
const ERROR_NO_TOKEN: u32 = 1008;
const ERROR_INSUFFICIENT_BUFFER: u32 = 122;
const TOKEN_IMPERSONATION_TYPE: u32 = 2;
const SECURITY_IMPERSONATION: u32 = 2;

#[repr(C)]
struct SecurityAttributes {
    length: u32,
    descriptor: *mut c_void,
    inherit: i32,
}

#[repr(C)]
struct Overlapped {
    internal: usize,
    internal_high: usize,
    offset: u32,
    offset_high: u32,
    event: NativeHandle,
}
impl Overlapped {
    fn new(event: NativeHandle) -> Self {
        Self { internal: 0, internal_high: 0, offset: 0, offset_high: 0, event }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Eq, PartialEq)]
struct Luid {
    low: u32,
    high: i32,
}
#[repr(C)]
struct TokenStatistics {
    token_id: Luid,
    authentication_id: Luid,
    expires: i64,
    token_type: u32,
    impersonation_level: u32,
    dynamic_charged: u32,
    dynamic_available: u32,
    group_count: u32,
    privilege_count: u32,
    modified_id: Luid,
}

#[repr(C)]
struct FileTime {
    low: u32,
    high: u32,
}
impl FileTime {
    fn value(&self) -> u64 {
        ((self.high as u64) << 32) | self.low as u64
    }
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        value: *const u16, revision: u32, descriptor: *mut *mut c_void,
        length: *mut u32,
    ) -> i32;
    fn OpenProcessToken(
        process: NativeHandle, rights: u32, result: *mut NativeHandle,
    ) -> i32;
    fn OpenThreadToken(
        thread: NativeHandle, rights: u32, open_as_self: i32,
        result: *mut NativeHandle,
    ) -> i32;
    fn GetTokenInformation(
        token: NativeHandle, info_class: u32, output: *mut c_void,
        capacity: u32, written: *mut u32,
    ) -> i32;
    fn ImpersonateNamedPipeClient(pipe: NativeHandle) -> i32;
    fn RevertToSelf() -> i32;
}
#[link(name = "kernel32")]
unsafe extern "system" {
    fn CreateNamedPipeW(
        name: *const u16, open_mode: u32, pipe_mode: u32,
        instances: u32, out_bytes: u32, in_bytes: u32,
        timeout: u32, security: *const SecurityAttributes,
    ) -> NativeHandle;
    fn CreateEventW(
        security: *const c_void, manual_reset: i32,
        initial_state: i32, name: *const u16,
    ) -> NativeHandle;
    fn ConnectNamedPipe(pipe: NativeHandle, state: *mut Overlapped) -> i32;
    fn ReadFile(
        pipe: NativeHandle, buffer: *mut c_void,
        size: u32, count: *mut u32, state: *mut Overlapped,
    ) -> i32;
    fn GetOverlappedResult(
        pipe: NativeHandle, state: *mut Overlapped,
        count: *mut u32, wait: i32,
    ) -> i32;
    fn CancelIoEx(pipe: NativeHandle, state: *mut Overlapped) -> i32;
    fn WaitForSingleObject(handle: NativeHandle, timeout: u32) -> u32;
    fn DisconnectNamedPipe(pipe: NativeHandle) -> i32;
    fn GetNamedPipeClientProcessId(pipe: NativeHandle, pid: *mut u32) -> i32;
    fn OpenProcess(rights: u32, inherit: i32, pid: u32) -> NativeHandle;
    fn GetProcessTimes(
        process: NativeHandle, creation: *mut FileTime,
        exit: *mut FileTime, kernel: *mut FileTime, user: *mut FileTime,
    ) -> i32;
    fn GetLastError() -> u32;
    fn GetCurrentThread() -> NativeHandle;
    fn CloseHandle(handle: NativeHandle) -> i32;
    fn LocalFree(memory: *mut c_void) -> *mut c_void;
}

/// Exactly one owner. No pipe/process/event HANDLE crosses a protocol frame.
struct OwnedHandle(NativeHandle);
impl OwnedHandle {
    fn new(handle: NativeHandle) -> io::Result<Self> {
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self(handle))
        }
    }
}
impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != INVALID_HANDLE_VALUE {
            // SAFETY: Unique ownership of this OS handle.
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// Unconditionally leave the client's security context. Microsoft
/// documents that continuing to run if RevertToSelf fails is unsafe.
struct RevertGuard;
impl Drop for RevertGuard {
    fn drop(&mut self) {
        // SAFETY: This guard is constructed ONLY after successful
        // ImpersonateNamedPipeClient on the current service thread.
        if unsafe { RevertToSelf() } == 0 {
            std::process::abort();
        }
    }
}

/// Raw SID and logon data are private to this stack frame. They are never
/// serialized, logged or used as game/account readiness evidence.
struct LocalTokenIdentity {
    user_sid: Vec<u8>,
    windows_session: u32,
    logon_id: Luid,
}

fn token_fixed_u32(token: NativeHandle, class: u32) -> io::Result<u32> {
    let mut value = 0u32;
    let mut written = 0u32;
    // SAFETY: TokenSessionId and TokenImpersonationLevel are fixed DWORDs.
    if unsafe {
        GetTokenInformation(
            token, class, (&mut value as *mut u32).cast(),
            mem::size_of::<u32>() as u32, &mut written,
        )
    } == 0 || written != mem::size_of::<u32>() as u32 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid token DWORD metadata"));
    }
    Ok(value)
}

fn token_statistics(token: NativeHandle) -> io::Result<TokenStatistics> {
    // SAFETY: TOKEN_STATISTICS is a fixed-size C POD struct.
    let mut result: TokenStatistics = unsafe { mem::zeroed() };
    let mut written = 0u32;
    if unsafe {
        GetTokenInformation(
            token, TOKEN_STATISTICS, (&mut result as *mut TokenStatistics).cast(),
            mem::size_of::<TokenStatistics>() as u32, &mut written,
        )
    } == 0 || written != mem::size_of::<TokenStatistics>() as u32 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData, "Invalid token logon statistics",
        ));
    }
    Ok(result)
}

fn token_identity(token: NativeHandle) -> io::Result<LocalTokenIdentity> {
    let user_sid = verified_token_user_sid(token)?;
    let windows_session = token_fixed_u32(token, TOKEN_SESSION_ID)?;
    let logon_id = token_statistics(token)?.authentication_id;
    Ok(LocalTokenIdentity { user_sid, windows_session, logon_id })
}

/// These are independent negative evidence bits, NOT a security grant.
/// Token equality is necessary for the pilot's same-connection claim,
/// but cannot prevent same-logon pipe HANDLE duplication or prove which
/// process sent a message with a different handle.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PendingTokenComparison {
    same_user: bool,
    same_windows_session: bool,
    same_logon_id: bool,
    same_logon_sid: bool,
    sufficient_impersonation_level: bool,
}
impl PendingTokenComparison {
    fn all_match(self) -> bool {
        self.same_user && self.same_windows_session
            && self.same_logon_id && self.same_logon_sid
            && self.sufficient_impersonation_level
    }
}

/// Compare the actual process token with the impersonated context of the
/// last 32-byte message read by this SERVICE thread (not the packet fields).
/// No file/registry/privileged call is made under impersonation.
fn inspect_last_message_token(
    pipe: NativeHandle, connected_process: NativeHandle,
) -> io::Result<PendingTokenComparison> {
    let mut process_raw = ptr::null_mut();
    // SAFETY: Query-only token from the already retained native process
    // HANDLE, before changing the service thread's security context.
    if unsafe {
        OpenProcessToken(connected_process, TOKEN_QUERY, &mut process_raw)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    let process_token = OwnedHandle::new(process_raw)?;
    let process_identity = token_identity(process_token.0)?;
    let process_logon_sid = verified_token_logon_sid(process_token.0)?;
    // Verify the connected process token against local WTSActive sessions.
    // Includes console and RDP, never an ID from the untrusted wire frame.
    verify_active_windows_logon(process_token.0)?;

    // If this worker thread already impersonates anyone, refuse to
    // proceed. RevertToSelf would otherwise erase another owner's scope.
    let mut prior_raw = ptr::null_mut();
    let opened = unsafe {
        OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut prior_raw)
    };
    let error = unsafe { GetLastError() };
    if opened != 0 || !prior_raw.is_null() {
        if !prior_raw.is_null() {
            let _prior = OwnedHandle::new(prior_raw)?;
        }
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied, "Service thread already impersonating",
        ));
    }
    if error != ERROR_NO_TOKEN {
        return Err(io::Error::from_raw_os_error(error as i32));
    }
    // SAFETY: The pipe server has consumed ONE complete and valid message.
    // Failure MUST NOT fall back to treating the service token as the client.
    if unsafe { ImpersonateNamedPipeClient(pipe) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let revert = RevertGuard;
    let inspected = (|| -> io::Result<PendingTokenComparison> {
        let mut thread_raw = ptr::null_mut();
        // SAFETY: We are impersonating only on the current thread. Use
        // OpenAsSelf to query even if the client limits token DACL rights.
        if unsafe {
            OpenThreadToken(GetCurrentThread(), TOKEN_QUERY, 1, &mut thread_raw)
        } == 0 {
            return Err(io::Error::last_os_error());
        }
        let last_message_token = OwnedHandle::new(thread_raw)?;
        let impersonation_level = token_fixed_u32(
            last_message_token.0, TOKEN_IMPERSONATION_LEVEL,
        )?;
        let message_identity = token_identity(last_message_token.0)?;
        let message_logon_sid = verified_token_logon_sid(last_message_token.0)?;
        let statistics = token_statistics(last_message_token.0)?;
        Ok(PendingTokenComparison {
            same_user: message_identity.user_sid == process_identity.user_sid,
            same_windows_session: message_identity.windows_session == process_identity.windows_session,
            same_logon_id: message_identity.logon_id == process_identity.logon_id,
            same_logon_sid: message_logon_sid == process_logon_sid,
            sufficient_impersonation_level: statistics.token_type == TOKEN_IMPERSONATION_TYPE
                && statistics.impersonation_level >= SECURITY_IMPERSONATION
                && impersonation_level >= SECURITY_IMPERSONATION,
        })
    })();
    // RAII also reverts on early ? errors inside the closure.
    drop(revert);
    inspected
}

struct Descriptor(*mut c_void);
impl Drop for Descriptor {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: Windows SDDL conversion returns LocalAlloc memory.
            let _ = unsafe { LocalFree(self.0) };
        }
    }
}

/// A service-owned pipe HANDLE; it is never inherited, duplicated to Tauri,
/// stored on disk or returned to an untrusted client.
pub(crate) struct ReadOnlyPipeWorker {
    handle: OwnedHandle,
}

impl ReadOnlyPipeWorker {
    pub(crate) fn bind() -> io::Result<Self> {
        let name: Vec<u16> = OsStr::new(r"\\.\pipe\MIVUBIIndependentHost-ReadOnly-V1")
            .encode_wide().chain(std::iter::once(0)).collect();
        // Do NOT grant generic write to a client: named-pipe
        // FILE_CREATE_PIPE_INSTANCE overlaps a generic write bit.
        // BA gets WRITE_DATA + SYNCHRONIZE + READ_CONTROL only. This
        // is an ADMIN-ONLY source pilot, NOT a user-session IPC policy.
        // Native SCM token MUST contain the Windows-resolved service SID.
        // No unverified desktop-user or caller-provided logon SID is granted.
        let service_sid = verified_current_service_sid()?;
        // The service can bind without a logged-on user. Fresh WTSActive
        // evidence is required PER MESSAGE from the actual OS peer token.
        let acl = format!(
            "D:P(A;;GA;;;SY)(A;;GA;;;{service_sid})(A;;0x00120002;;;BA)"
        );
        let sddl: Vec<u16> = acl.encode_utf16().chain(std::iter::once(0)).collect();
        let mut descriptor: *mut c_void = ptr::null_mut();
        // SAFETY: Static NUL-terminated SDDL input, one owned output.
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(), 1, &mut descriptor, ptr::null_mut(),
            )
        } == 0 {
            return Err(io::Error::last_os_error());
        }
        let guard = Descriptor(descriptor);
        if guard.0.is_null() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Empty pipe security descriptor"));
        }
        let attributes = SecurityAttributes {
            length: mem::size_of::<SecurityAttributes>() as u32,
            descriptor: guard.0,
            inherit: 0,
        };
        // SAFETY: OS allocates an entirely new first pipe instance with
        // the explicit protected DACL. No remote connections or output.
        let raw = unsafe {
            CreateNamedPipeW(
                name.as_ptr(),
                PIPE_ACCESS_INBOUND | FILE_FLAG_FIRST_PIPE_INSTANCE | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_REJECT_REMOTE_CLIENTS,
                1, 0, BROKER_PROBE_BYTES as u32, 0, &attributes,
            )
        };
        Ok(Self { handle: OwnedHandle::new(raw)? })
    }

    /// A single SCM-owned loop; no extra listener daemon or worker manager.
    /// It never returns a response and never performs a profile mutation.
    pub(crate) fn run_until_stop(
        &mut self,
        stop: &Receiver<()>,
        announce_stop: &mut dyn FnMut() -> io::Result<()>,
    ) -> io::Result<()> {
        loop {
            if wants_stop(stop, announce_stop)? {
                return Ok(());
            }
            let step = self.read_one(stop, announce_stop);
            // SAFETY: All pending OVERLAPPED operations have settled before
            // this worker disconnects or reuses the sole pipe HANDLE.
            unsafe { DisconnectNamedPipe(self.handle.0) };
            match step {
                Ok(Step::Stopped) => return Ok(()),
                Ok(Step::Ignored) => {}
                Err(error) if matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::InvalidData |
                    io::ErrorKind::PermissionDenied | io::ErrorKind::NotFound |
                    io::ErrorKind::BrokenPipe
                ) => {}
                Err(error) => return Err(error),
            }
            // Limit retry churn from malformed or repeatedly aborted client
            // messages. A stop signal is still observed on the next cycle.
            thread::sleep(Duration::from_millis(25));
        }
    }

    fn read_one(
        &self,
        stop: &Receiver<()>,
        announce_stop: &mut dyn FnMut() -> io::Result<()>,
    ) -> io::Result<Step> {
        let mut connect = PendingIo::new(self.handle.0)?;
        // SAFETY: The pipe was opened FILE_FLAG_OVERLAPPED and this stable
        // boxed OVERLAPPED/event cannot be dropped until I/O finishes.
        let started = unsafe { ConnectNamedPipe(self.handle.0, connect.ptr()) };
        if started == 0 {
            match unsafe { GetLastError() } {
                ERROR_PIPE_CONNECTED => {}
                ERROR_IO_PENDING => {
                    connect.pending = true;
                    if matches!(connect.complete(stop, announce_stop, CONNECTION_DEADLINE)?, IoCompletion::Stopped) {
                        return Ok(Step::Stopped);
                    }
                }
                value => return Err(io::Error::from_raw_os_error(value as i32)),
            }
        }
        // Buffer is created BEFORE the I/O owner, so that cancellation and
        // kernel completion occur before its memory is released.
        let mut buffer = Box::new([0u8; BROKER_PROBE_BYTES]);
        let mut read = PendingIo::new(self.handle.0)?;
        // SAFETY: Box guarantees stable storage until the pending request
        // is settled; lpNumberOfBytesRead is null in overlapped mode.
        let started = unsafe {
            ReadFile(
                self.handle.0, buffer.as_mut_ptr().cast(),
                BROKER_PROBE_BYTES as u32, ptr::null_mut(), read.ptr(),
            )
        };
        // ERROR_MORE_DATA may arrive at ReadFile OR GetOverlappedResult.
        // Both paths mean a malformed oversized message, never a service
        // fatal error and never a successful 32-byte diagnostic request.
        let received = (|| -> io::Result<Option<u32>> {
            if started != 0 {
                return read.completed_now().map(Some);
            }
            match unsafe { GetLastError() } {
                ERROR_IO_PENDING => {
                    read.pending = true;
                    match read.complete(stop, announce_stop, READ_DEADLINE)? {
                        IoCompletion::Stopped => Ok(None),
                        IoCompletion::Bytes(count) => Ok(Some(count)),
                    }
                }
                value => Err(io::Error::from_raw_os_error(value as i32)),
            }
        })();
        let received = match received {
            Ok(Some(bytes)) => bytes,
            Ok(None) => return Ok(Step::Stopped),
            Err(error) if error.raw_os_error() == Some(ERROR_MORE_DATA as i32) => {
                return Ok(Step::Ignored);
            }
            Err(error) => return Err(error),
        };
        if received != BROKER_PROBE_BYTES as u32 {
            return Ok(Step::Ignored);
        }
        // Import the single canonical parser from runtime-core. Even a
        // well-formed frame is untrusted and can never authorize an action.
        let claim = match decode_broker_probe(buffer.as_ref()) {
            Ok(value) => value,
            Err(_) => return Ok(Step::Ignored),
        };
        // A SESSION_CHANGE/Stop may have arrived just after the last
        // overlapped read completed. Never inspect a packet against a
        // session that the SCM already asked the pilot to revoke.
        if wants_stop(stop, announce_stop)? {
            return Ok(Step::Stopped);
        }
        // If connection/last-message token observations disagree, discard
        // the packet. Even complete matching evidence is NOT permission for
        // service-side operations; this pilot has no mutating opcodes.
        if !matches!(compare_native_peer(self.handle.0, claim), Ok(true)) {
            return Ok(Step::Ignored);
        }
        // Recheck before completing a diagnostic request, since the native
        // WTS/AccessCheck path can span a concurrent lock/logoff event.
        if wants_stop(stop, announce_stop)? {
            return Ok(Step::Stopped);
        }
        // Deliberately no response, credentials, root handle, or launch.
        Ok(Step::Ignored)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step { Stopped, Ignored }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IoCompletion { Stopped, Bytes(u32) }

/// SCM STOP_PENDING must precede any potentially slow native cancellation.
/// This is a service-process lifecycle signal, NOT game readiness.
fn wants_stop(
    stop: &Receiver<()>,
    announce_stop: &mut dyn FnMut() -> io::Result<()>,
) -> io::Result<bool> {
    match stop.try_recv() {
        Ok(()) | Err(TryRecvError::Disconnected) => {
            announce_stop()?;
            Ok(true)
        }
        Err(TryRecvError::Empty) => Ok(false),
    }
}

/// Per-operation stable OVERLAPPED memory with RAII cancellation. The
/// service NEVER aborts on timeout; it must not report STOPPED until
/// Windows acknowledges the completion of any pending kernel I/O.
struct PendingIo {
    pipe: NativeHandle,
    event: OwnedHandle,
    state: Box<Overlapped>,
    pending: bool,
}
impl PendingIo {
    fn new(pipe: NativeHandle) -> io::Result<Self> {
        // SAFETY: Private unnamed manual-reset event; unique HANDLE owner.
        let event = OwnedHandle::new(unsafe {
            CreateEventW(ptr::null(), 1, 0, ptr::null())
        })?;
        let state = Box::new(Overlapped::new(event.0));
        Ok(Self { pipe, event, state, pending: false })
    }
    fn ptr(&mut self) -> *mut Overlapped {
        self.state.as_mut()
    }
    fn completed_now(&mut self) -> io::Result<u32> {
        // The originating OS call reported immediate completion; no
        // kernel I/O remains pending after GetOverlappedResult.
        let mut bytes = 0;
        // SAFETY: Correct pointer to our retained OVERLAPPED and pipe.
        if unsafe { GetOverlappedResult(self.pipe, self.ptr(), &mut bytes, 0) } == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(bytes)
        }
    }
    fn complete(
        &mut self,
        stop: &Receiver<()>,
        announce_stop: &mut dyn FnMut() -> io::Result<()>,
        timeout: Duration,
    ) -> io::Result<IoCompletion> {
        let started = Instant::now();
        loop {
            if wants_stop(stop, announce_stop)? {
                self.drain();
                return Ok(IoCompletion::Stopped);
            }
            if started.elapsed() >= timeout {
                self.drain();
                return Err(io::Error::new(io::ErrorKind::TimedOut, "IPC probe deadline"));
            }
            // SAFETY: The event and OVERLAPPED both remain alive, and only
            // one operation at a time is pending on this pipe instance.
            let outcome = unsafe { WaitForSingleObject(self.event.0, WAKE_SLICE_MS) };
            if outcome == WAIT_OBJECT_0 {
                self.pending = false;
                let count = self.completed_now()?;
                return Ok(IoCompletion::Bytes(count));
            }
            if outcome != WAIT_TIMEOUT {
                self.drain();
                return Err(io::Error::last_os_error());
            }
        }
    }
    fn drain(&mut self) {
        if !self.pending {
            return;
        }
        // SAFETY: Target only our known request. CancelIoEx requests
        // cancellation but does not itself complete the operation.
        unsafe { CancelIoEx(self.pipe, self.ptr()) };
        let mut discarded = 0u32;
        // SAFETY: TRUE waits for final I/O completion; the boxed
        // OVERLAPPED and buffer remain owned and valid throughout.
        let _ = unsafe { GetOverlappedResult(self.pipe, self.ptr(), &mut discarded, 1) };
        self.pending = false;
    }
}
impl Drop for PendingIo {
    fn drop(&mut self) { self.drain(); }
}

/// Read-only, non-authorizing PID/FILETIME correlation. A duplicated pipe
/// handle or same-user service token is NOT a privileged authorization.
fn compare_native_peer(
    pipe: NativeHandle, claim: UntrustedBrokerProbe,
) -> io::Result<bool> {
    let mut pid = 0;
    if unsafe { GetNamedPipeClientProcessId(pipe, &mut pid) } == 0 || pid == 0 {
        return Err(io::Error::last_os_error());
    }
    if pid != claim.claimed_process.pid {
        return Ok(false);
    }
    // SAFETY: Query-only Windows process handle, no termination access.
    let process = OwnedHandle::new(unsafe {
        OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZE, 0, pid)
    })?;
    let mut created = FileTime { low: 0, high: 0 };
    let mut exit = FileTime { low: 0, high: 0 };
    let mut kernel = FileTime { low: 0, high: 0 };
    let mut user = FileTime { low: 0, high: 0 };
    if unsafe {
        GetProcessTimes(process.0, &mut created, &mut exit, &mut kernel, &mut user)
    } == 0 {
        return Err(io::Error::last_os_error());
    }
    if created.value() != claim.claimed_process.created_file_time {
        return Ok(false);
    }
    if unsafe { WaitForSingleObject(process.0, 0) } != WAIT_TIMEOUT {
        return Ok(false);
    }
    let mut current_pid = 0;
    if unsafe { GetNamedPipeClientProcessId(pipe, &mut current_pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    // Token checks run only after the fixed frame and native connected PID
    // and creation time have been checked; no security claim comes from
    // the untrusted packet's correlation ID or MCxx label.
    if current_pid != pid {
        return Ok(false);
    }
    let token_evidence = inspect_last_message_token(pipe, process.0)?;
    if !token_evidence.all_match() {
        return Ok(false);
    }
    // SOURCE-ONLY read-only Authenticode review against this exact,
    // retained OS-connected client process HANDLE. No pathname, file
    // or leaf fingerprint is accepted from the untrusted request.
    let image = crate::peer_image::inspect_os_peer_image(process.0)?;
    // No reviewed MIVUBI installer signer leaf is pinned in this
    // repository. A generic Windows-trusted signature MUST NOT grant
    // even diagnostic publisher admission: the checked-in empty policy
    // deliberately keeps this opt-in pilot fail closed.
    if !image.matches_reviewed_on_disk_signer() {
        return Ok(false);
    }
    // Even a signer match is not real origin authentication until the
    // running mapped PE section is bound to the verified file object.
    // This pilot has no privileged/mutating operation or reply.
    let mut final_pid = 0u32;
    if unsafe { GetNamedPipeClientProcessId(pipe, &mut final_pid) } == 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(final_pid == pid && unsafe { WaitForSingleObject(process.0, 0) } == WAIT_TIMEOUT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use virtual_clients_core::independent_engine::{
        encode_broker_probe, BrokerProbeKind, ProcessIdentity, SessionSlot,
    };

    #[test]
    fn stop_signal_announces_pending_before_any_io_cleanup() {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        tx.send(()).unwrap();
        let mut calls = 0u32;
        assert!(wants_stop(&rx, &mut || {
            calls += 1;
            Ok(())
        }).unwrap());
        assert_eq!(calls, 1);
    }

    #[test]
    fn failed_stop_status_report_is_never_misclassified_as_clean_stop() {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        tx.send(()).unwrap();
        assert_eq!(
            wants_stop(&rx, &mut || Err(io::Error::new(
                io::ErrorKind::Other, "SCM unavailable",
            ))).unwrap_err().kind(),
            io::ErrorKind::Other,
        );
    }

    #[test]
    fn token_identity_review_requires_every_os_property() {
        let full = PendingTokenComparison {
            same_user: true,
            same_windows_session: true,
            same_logon_id: true,
            same_logon_sid: true,
            sufficient_impersonation_level: true,
        };
        assert!(full.all_match());
        for field in 0..5 {
            let mut missing = full;
            match field {
                0 => missing.same_user = false,
                1 => missing.same_windows_session = false,
                2 => missing.same_logon_id = false,
                3 => missing.same_logon_sid = false,
                _ => missing.sufficient_impersonation_level = false,
            }
            assert!(!missing.all_match());
        }
    }

    #[test]
    fn native_token_statistics_c_layout_is_fixed() {
        // Regression for the WinNT.h TOKEN_STATISTICS ABI projection.
        assert_eq!(mem::size_of::<Luid>(), 8);
        assert_eq!(mem::size_of::<TokenStatistics>(), 56);
    }

    #[test]
    fn shared_wire_decoder_keeps_service_request_non_authorizing() {
        let frame = encode_broker_probe(UntrustedBrokerProbe {
            kind: BrokerProbeKind::InspectPrivateRootAcl,
            slot: SessionSlot::Mc02,
            claimed_process: ProcessIdentity::checked(42, 123456).unwrap(),
            correlation_id: 1,
        }).unwrap();
        let result = decode_broker_probe(&frame).unwrap();
        assert_eq!(result.slot, SessionSlot::Mc02);
        assert_eq!(result.kind, BrokerProbeKind::InspectPrivateRootAcl);
        // This is not an authorized request or output; the native service
        // worker discards the message and does no filesystem/registry work.
    }
}
