//! Source-only IPC ingress worker OWNED by the standalone SCM service.
//!
//! Compile-time feature readonly-ipc-pilot is OFF BY DEFAULT. This module
//! never installs a service, launches a game, opens credentials/profile data,
//! impersonates clients, changes ACLs or sends replies. It only consumes
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
        let sddl: Vec<u16> = "D:P(A;;GA;;;SY)(A;;0x00120002;;;BA)"
            .encode_utf16().chain(std::iter::once(0)).collect();
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
    pub(crate) fn run_until_stop(&mut self, stop: &Receiver<()>) -> io::Result<()> {
        loop {
            if wants_stop(stop) {
                return Ok(());
            }
            let step = self.read_one(stop);
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

    fn read_one(&self, stop: &Receiver<()>) -> io::Result<Step> {
        let mut connect = PendingIo::new(self.handle.0)?;
        // SAFETY: The pipe was opened FILE_FLAG_OVERLAPPED and this stable
        // boxed OVERLAPPED/event cannot be dropped until I/O finishes.
        let started = unsafe { ConnectNamedPipe(self.handle.0, connect.ptr()) };
        if started == 0 {
            match unsafe { GetLastError() } {
                ERROR_PIPE_CONNECTED => {}
                ERROR_IO_PENDING => {
                    connect.pending = true;
                    if matches!(connect.complete(stop, CONNECTION_DEADLINE)?, IoCompletion::Stopped) {
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
        let received = if started != 0 {
            read.completed_now()?
        } else {
            match unsafe { GetLastError() } {
                ERROR_IO_PENDING => {
                    read.pending = true;
                    match read.complete(stop, READ_DEADLINE)? {
                        IoCompletion::Stopped => return Ok(Step::Stopped),
                        IoCompletion::Bytes(count) => count,
                    }
                }
                ERROR_MORE_DATA => return Ok(Step::Ignored),
                value => return Err(io::Error::from_raw_os_error(value as i32)),
            }
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
        let _ = compare_native_peer(self.handle.0, claim);
        // Deliberately no response, credentials, root handle, or launch.
        Ok(Step::Ignored)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Step { Stopped, Ignored }
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IoCompletion { Stopped, Bytes(u32) }

fn wants_stop(stop: &Receiver<()>) -> bool {
    match stop.try_recv() {
        Ok(()) | Err(TryRecvError::Disconnected) => true,
        Err(TryRecvError::Empty) => false,
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
        &mut self, stop: &Receiver<()>, timeout: Duration,
    ) -> io::Result<IoCompletion> {
        let started = Instant::now();
        loop {
            if wants_stop(stop) {
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
    Ok(current_pid == pid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use virtual_clients_core::independent_engine::{
        encode_broker_probe, BrokerProbeKind, ProcessIdentity, SessionSlot,
    };

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
