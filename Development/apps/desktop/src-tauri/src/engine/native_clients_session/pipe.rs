//! Local reconnection for one retained controller, not a privileged/general broker.
//! Same logon, session, user, integrity and executable file identity at both peers.
use super::{read_message, SessionReply, SessionRequest, SESSION_FRAME_LIMIT};
use std::{
    fs::File, io::{self, BufReader, Read, Write}, mem,
    os::windows::io::{AsRawHandle, BorrowedHandle, FromRawHandle, OwnedHandle},
    ptr, sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc},
    thread::{self, JoinHandle}, time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{GetLastError, ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_PIPE_CONNECTED,
        HANDLE, INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT},
    Storage::FileSystem::*,
    System::{Pipes::*, Threading::*, IO::{CancelIoEx, CancelSynchronousIo, GetOverlappedResult, OVERLAPPED}},
};

mod peer_endpoint;
pub(crate) use self::peer_endpoint::{Endpoint, McpOwnerLifetime, Peer};

fn refused(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason)
}
fn own(raw: HANDLE) -> io::Result<OwnedHandle> {
    if raw.is_null() || raw == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error());
    }
    Ok(unsafe { OwnedHandle::from_raw_handle(raw as _) })
}
fn raw(handle: &OwnedHandle) -> HANDLE {
    handle.as_raw_handle() as HANDLE
}
// The complete response was read and its authenticated native peer is
// still alive before this helper is called. The final ACK only closes the
// pipe handshake; its failure does NOT undo the already-received reply.
// Missing, malformed, or unverified replies still fail in exchange() first.
fn acknowledge_confirmed_reply(writer:&mut impl Write,reply:SessionReply)->SessionReply {
    let _=writer.write_all(b"ack\n");
    reply
}

// Serialize one complete, bounded frame before writing to the byte-mode pipe.
fn write_request(writer: &mut impl Write, request: &SessionRequest) -> io::Result<()> {
    let mut frame = serde_json::to_vec(request)?;
    if frame.len() >= SESSION_FRAME_LIMIT {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Session request exceeded its limit"));
    }
    frame.push(b'\n');
    writer.write_all(&frame)
}

pub(crate) fn exchange(
    endpoint: &Endpoint,
    mut pipe: File,
    request: &SessionRequest,
) -> io::Result<SessionReply> {
    let peer = endpoint.verify(pipe.as_raw_handle() as _, false)?;
    write_request(&mut pipe, request)?;
    // The request write, reply read and ACK are sequential on the original
    // synchronous handle. Borrow it for the read instead of duplicating it.
    let text = {
        let mut reader = BufReader::new(&mut pipe);
        read_message(&mut reader)?
            .ok_or_else(|| io::Error::other("Controller disconnected before its reply"))?
    };
    if !peer.alive() {
        return Err(refused(
            "Controller exited during the request; outcome uncertain",
        ));
    }
    let reply:SessionReply = serde_json::from_str(&text).map_err(io::Error::other)?;
    // The reply has already been fully read, structurally parsed and
    // bound to a live authenticated original controller. ACK loss must
    // not turn a confirmed native result into an ambiguous failure.
    // Unverified/missing/malformed replies still refuse above.
    Ok(acknowledge_confirmed_reply(&mut pipe,reply))
}

pub(crate) struct Envelope {
    pub request: SessionRequest,
    pub peer: Peer,
    pub reply: mpsc::SyncSender<SessionReply>,
}
pub(crate) enum Activity {
    Request(Envelope),
    Lifetime,
    AudioHint,
    Deadline,
    Disconnected,
}

// Only the server handle is overlapped. One worker, one operation in
// flight, one reusable manual-reset event. No new actor or wire fields.
const PIPE_PHASE_TIMEOUT: Duration = Duration::from_secs(5);

// Preserve ceil-to-ms and the largest finite Win32 wait. INFINITE is
// reserved for the existing explicitly unbounded connection wait.
fn finite_wait_millis(remaining: Duration) -> u32 {
    remaining.as_nanos().div_ceil(1_000_000)
        .min(u128::from(u32::MAX - 1)) as u32
}

struct ServerPipe {
    file: File,
    event: OwnedHandle,
    stop_event: Arc<OwnedHandle>,
    phase_timeout: Duration,
    deadline: Option<Instant>,
}

impl ServerPipe {
    fn handle(&self) -> HANDLE { self.file.as_raw_handle() as HANDLE }

    fn begin_phase(&mut self) {
        self.deadline = Some(Instant::now() + self.phase_timeout);
    }

    fn timeout_error() -> io::Error {
        io::Error::new(io::ErrorKind::TimedOut, "Session pipe phase deadline elapsed")
    }

    fn stopped_error() -> io::Error {
        io::Error::new(io::ErrorKind::ConnectionAborted, "Session pipe stopped")
    }

    fn completed(&self, transferred: u32) -> io::Result<usize> {
        // A completion can win the OS wait even after the caller's phase
        // budget elapsed. Do not publish that late frame as timely.
        if self.deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            Err(Self::timeout_error())
        } else {
            Ok(transferred as usize)
        }
    }

    fn operation(
        &self,
        connect: bool,
        issue: impl FnOnce(HANDLE, *mut OVERLAPPED, *mut u32) -> i32,
    ) -> io::Result<usize> {
        if unsafe { WaitForSingleObject(raw(&self.stop_event), 0) } != WAIT_TIMEOUT {
            return Err(Self::stopped_error());
        }
        if self.deadline.is_some_and(|deadline| Instant::now() >= deadline) {
            return Err(Self::timeout_error());
        }
        if unsafe { ResetEvent(raw(&self.event)) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let handle = self.handle();
        // SAFETY: The worker issues exactly one operation. The overlapped
        // structure, event, transfer count and caller's read/write buffer
        // stay alive and unmoved until the operation has completed.
        let mut overlap: OVERLAPPED = unsafe { mem::zeroed() };
        overlap.hEvent = raw(&self.event);
        let mut transferred = 0u32;
        // Keep the raw pointer to the same storage throughout the pending
        // operation; do not form fresh Rust references while the OS owns it.
        let overlap_ptr = &raw mut overlap;
        let transferred_ptr = &raw mut transferred;
        if issue(handle, overlap_ptr, transferred_ptr) != 0 {
            return self.completed(transferred);
        }
        let error = unsafe { GetLastError() };
        if connect && error == ERROR_PIPE_CONNECTED {
            return Ok(0);
        }
        if error != ERROR_IO_PENDING {
            return Err(io::Error::from_raw_os_error(error as i32));
        }

        let milliseconds = self.deadline.map_or(u32::MAX, |deadline| {
            finite_wait_millis(deadline.saturating_duration_since(Instant::now()))
        });
        let handles = [raw(&self.stop_event), raw(&self.event)];
        let wake = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), 0, milliseconds) };
        if wake == WAIT_OBJECT_0 + 1 {
            // The operation event is manual-reset and exclusively ours.
            // Collect actual completion, not merely a signaled wait.
            if unsafe { GetOverlappedResult(handle, overlap_ptr, transferred_ptr, 1) } != 0 {
                return self.completed(transferred);
            }
            return Err(io::Error::last_os_error());
        }

        let failure = if wake == WAIT_OBJECT_0 { Self::stopped_error() }
            else if wake == WAIT_TIMEOUT { Self::timeout_error() }
            else { io::Error::last_os_error() };
        // Cancellation can race a successful completion. Discard this
        // connection's result regardless; never reuse its buffer before
        // the kernel completes. This completion barrier intentionally
        // retains the original worker if the OS fails to settle I/O.
        unsafe {
            CancelIoEx(handle, overlap_ptr);
            GetOverlappedResult(handle, overlap_ptr, transferred_ptr, 1);
        }
        Err(failure)
    }

    fn connect(&mut self) -> io::Result<()> {
        // Idle connection wait is event-driven, with shutdown wake only.
        self.deadline = None;
        self.operation(true, |handle, overlap, _| unsafe {
            ConnectNamedPipe(handle, overlap)
        }).map(|_| ())
    }
}

impl Read for ServerPipe {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() { return Ok(0); }
        let length = buf.len().min(u32::MAX as usize) as u32;
        match self.operation(false, |handle, overlap, transferred| unsafe {
            ReadFile(handle, buf.as_mut_ptr(), length, transferred, overlap)
        }) {
            Err(error) if error.raw_os_error() == Some(ERROR_BROKEN_PIPE as i32) => Ok(0),
            result => result,
        }
    }
}

impl Write for ServerPipe {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.is_empty() { return Ok(0); }
        let length = buf.len().min(u32::MAX as usize) as u32;
        self.operation(false, |handle, overlap, transferred| unsafe {
            WriteFile(handle, buf.as_ptr(), length, transferred, overlap)
        })
    }

    // No FlushFileBuffers: it would wait for the peer to consume the pipe.
    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

pub(crate) struct Server {
    pub requests: mpsc::Receiver<Envelope>,
    request_ready: Arc<OwnedHandle>,
    stopped: Arc<AtomicBool>,
    stop_event: Arc<OwnedHandle>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    pub(crate) fn start(endpoint: Endpoint) -> io::Result<Self> {
        Self::start_with_budget(endpoint, PIPE_PHASE_TIMEOUT)
    }

    fn start_with_budget(endpoint: Endpoint, phase_timeout: Duration) -> io::Result<Self> {
        // Bind before acknowledging startup; FIRST_PIPE_INSTANCE stays retained across reconnects.
        let file = endpoint.bind()?;
        let stop_event = Arc::new(own(unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) })?);
        let mut file = ServerPipe {
            file,
            event: own(unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) })?,
            stop_event: Arc::clone(&stop_event),
            phase_timeout,
            deadline: None,
        };
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&stopped);
        let (sender, requests) = mpsc::sync_channel(1);
        // Unnamed auto-reset event, shared only by the existing pipe worker
        // and controller. The channel remains the sole request authority.
        let request_ready = Arc::new(own(unsafe { CreateEventW(ptr::null(), 0, 0, ptr::null()) })?);
        let ready = Arc::clone(&request_ready);
        let worker = thread::Builder::new().name("lazytester-session-pipe".into()).spawn(move || {
            while !stop.load(Ordering::Acquire) {
                if file.connect().is_err() {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    unsafe {
                        DisconnectNamedPipe(file.handle());
                    }
                    thread::sleep(Duration::from_millis(20));
                    continue;
                }
                let _ = serve(&endpoint, &mut file, &sender, &stop, &ready);
                unsafe {
                    DisconnectNamedPipe(file.handle());
                }
            }
        })?;
        Ok(Self {
            requests,
            request_ready,
            stopped,
            stop_event,
            worker: Some(worker),
        })
    }

    // The actor exclusively owns the retained lifetimes and cannot close or
    // replace them during this synchronous wait. The pipe worker is also
    // waited on: a failed notification cannot strand a queued request.
    pub(crate) fn wait_for_activity<'a>(
        &self, lifetimes: impl Iterator<Item=BorrowedHandle<'a>>,
        audio_hint: Option<BorrowedHandle<'_>>, timeout: Duration,
    ) -> io::Result<Activity> {
        use mpsc::TryRecvError;
        match self.requests.try_recv() {
            Ok(envelope) => return Ok(Activity::Request(envelope)),
            Err(TryRecvError::Disconnected) => return Ok(Activity::Disconnected),
            Err(TryRecvError::Empty) => {},
        }
        let worker = self.worker.as_ref().ok_or_else(|| io::Error::other("Session pipe stopped"))?;
        // Request first, worker second, original lifetimes before the optional
        // manual-reset audio hint. No new source of request authority.
        let mut handles = [ptr::null_mut(); 9];
        handles[0] = raw(&self.request_ready);
        handles[1] = worker.as_raw_handle() as HANDLE;
        let mut count = 2;
        for handle in lifetimes {
            if count == 8 {
                return Err(io::Error::other("Controller lifetime wait exceeds its bound"));
            }
            handles[count] = handle.as_raw_handle() as HANDLE;
            count += 1;
        }
        let hint_index = audio_hint.map(|hint| {
            let index=count;
            handles[index]=hint.as_raw_handle() as HANDLE;
            count+=1;
            WAIT_OBJECT_0+index as u32
        });
        let started = Instant::now();
        loop {
            let remaining = timeout.saturating_sub(started.elapsed());
            let milliseconds = finite_wait_millis(remaining);
            let wake = unsafe { WaitForMultipleObjects(count as u32, handles.as_ptr(), 0, milliseconds) };
            if wake != WAIT_TIMEOUT && !(WAIT_OBJECT_0..WAIT_OBJECT_0 + count as u32).contains(&wake) {
                return Err(if wake == u32::MAX { io::Error::last_os_error() }
                    else { io::Error::other("Unexpected controller lifetime wait result") });
            }
            // An authenticated request wins even when another handle signaled.
            // Consume a stale auto-reset signal without manufacturing a
            // maintenance deadline or restarting the original timeout.
            match self.requests.try_recv() {
                Ok(envelope) => return Ok(Activity::Request(envelope)),
                Err(TryRecvError::Disconnected) => return Ok(Activity::Disconnected),
                Err(TryRecvError::Empty) if wake == WAIT_OBJECT_0 + 1 => return Ok(Activity::Disconnected),
                Err(TryRecvError::Empty) if wake == WAIT_TIMEOUT => return Ok(Activity::Deadline),
                Err(TryRecvError::Empty) if wake == WAIT_OBJECT_0 => continue,
                Err(TryRecvError::Empty) if hint_index==Some(wake) => return Ok(Activity::AudioHint),
                Err(TryRecvError::Empty) => return Ok(Activity::Lifetime),
            }
        }
    }
}
fn serve(
    endpoint: &Endpoint,
    file: &mut ServerPipe,
    sender: &mpsc::SyncSender<Envelope>,
    stop: &AtomicBool,
    request_ready: &OwnedHandle,
) -> io::Result<()> {
    if stop.load(Ordering::Acquire) {
        return Ok(());
    }
    let peer = endpoint.verify(file.handle(), true)?;
    file.begin_phase();
    let mut reader = BufReader::new(file);
    let text =
        read_message(&mut reader)?.ok_or_else(|| io::Error::other("Manager disconnected"))?;
    if !peer.alive() {
        return Err(refused("Manager exited before request admission"));
    }
    let request: SessionRequest = serde_json::from_str(&text).map_err(io::Error::other)?;
    // Validate at the authenticated controller ingress as well as the client.
    // A denied request must receive an ordinary error receipt, not an ambiguous disconnect.
    if let Err(error) = super::validate_request(&request) {
        return send_reply(&mut reader, Err(error.to_string()));
    }
    let (reply, response) = mpsc::sync_channel(1);
    sender
        .send(Envelope {
            request,
            peer,
            reply,
        })
        .map_err(|_| io::Error::other("Controller stopped"))?;
    // Publish before signaling. A failed signal stops this transport worker,
    // whose retained thread handle also wakes the actor. Never replay a request.
    if unsafe { SetEvent(raw(request_ready)) } == 0 {
        let error = io::Error::last_os_error();
        stop.store(true, Ordering::Release);
        return Err(error);
    }
    let reply = loop {
        if stop.load(Ordering::Acquire) {
            return Ok(());
        }
        match response.recv_timeout(Duration::from_millis(250)) {
            Ok(reply) => break reply,
            Err(mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => return Err(io::Error::other("Controller stopped")),
        }
    };
    send_reply(&mut reader, reply)
}

fn send_reply(reader: &mut BufReader<&mut ServerPipe>, reply: SessionReply) -> io::Result<()> {
    let mut bytes = serde_json::to_vec(&reply)?;
    if bytes.len() >= SESSION_FRAME_LIMIT {
        bytes = serde_json::to_vec(&SessionReply::Err(
            "Controller response exceeded its limit".into(),
        ))?;
    }
    bytes.push(b'\n');
    reader.get_mut().begin_phase();
    reader.get_mut().write_all(&bytes)?;
    reader.get_mut().begin_phase();
    if read_message(reader)?.as_deref() != Some("ack\n") {
        return Err(io::Error::other("Response was not acknowledged"));
    }
    Ok(())
}
impl Server {
    fn stop(&mut self) -> bool {
        self.stopped.store(true, Ordering::Release);
        // Wake idle connect and any pending overlapped operation. The
        // worker owns its pending buffers until cancellation completes.
        unsafe { SetEvent(raw(&self.stop_event)) };
        if let Some(worker) = self.worker.take() {
            // Best effort for synchronous authentication/file queries only.
            // Overlapped pipe cancellation is owned by the worker itself.
            // Retain its stack on a slow shutdown; never terminate a game.
            let deadline = Instant::now() + Duration::from_secs(2);
            while !worker.is_finished() && Instant::now() < deadline {
                unsafe { CancelSynchronousIo(worker.as_raw_handle() as _) };
                thread::sleep(Duration::from_millis(10));
            }
            if worker.is_finished() {
                return worker.join().is_ok();
            }
            return false;
        }
        true
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests;
