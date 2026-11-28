//! Local reconnection for one retained controller, not a privileged/general broker.
//! Same logon, session, user, integrity and executable file identity at both peers.
use super::{read_message, SessionReply, SessionRequest};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, Read, Write},
    mem,
    os::windows::{
        ffi::OsStringExt,
        fs::OpenOptionsExt,
        io::{AsHandle, AsRawHandle, BorrowedHandle, FromRawHandle, OwnedHandle},
    },
    ptr,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Foundation::{
        GetLastError, LocalFree, ERROR_BROKEN_PIPE, ERROR_IO_PENDING, ERROR_PIPE_CONNECTED, FILETIME, HANDLE, INVALID_HANDLE_VALUE,
        WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        },
        *,
    },
    Storage::FileSystem::*,
    System::{Diagnostics::ToolHelp::*, Pipes::*, Threading::*, IO::{CancelIoEx, CancelSynchronousIo, GetOverlappedResult, OVERLAPPED}},
};

fn refused(reason: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, reason)
}
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
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
// Win32 pipe queries require read attributes as well as data. Writes grant data only,
// not GENERIC_WRITE (which would also grant FILE_CREATE_PIPE_INSTANCE).
const CLIENT_RIGHTS: u32 = FILE_GENERIC_READ | FILE_WRITE_DATA;

struct Allocation(*mut std::ffi::c_void);
impl Drop for Allocation {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}

fn token_data(token: HANDLE, class: TOKEN_INFORMATION_CLASS) -> io::Result<Vec<usize>> {
    let mut size = 0;
    unsafe { GetTokenInformation(token, class, ptr::null_mut(), 0, &mut size) };
    if size == 0 || size > 65536 {
        return Err(refused("Unsupported process token"));
    }
    let mut buffer = vec![0usize; (size as usize).div_ceil(mem::size_of::<usize>())];
    if unsafe { GetTokenInformation(token, class, buffer.as_mut_ptr().cast(), size, &mut size) }
        == 0
    {
        return Err(io::Error::last_os_error());
    }
    Ok(buffer)
}
unsafe fn sid_bytes(sid: PSID) -> io::Result<Vec<u8>> {
    if sid.is_null() || IsValidSid(sid) == 0 {
        return Err(refused("Invalid process token SID"));
    }
    let size = GetLengthSid(sid) as usize;
    if !(8..=68).contains(&size) {
        return Err(refused("Unsupported process token SID"));
    }
    Ok(std::slice::from_raw_parts(sid.cast::<u8>(), size).to_vec())
}

#[derive(Clone, PartialEq, Eq)]
struct Identity {
    user: Vec<u8>,
    logon: String,
    integrity: Vec<u8>,
    authentication: (u32, i32),
    session: u32,
}
impl Identity {
    fn process(process: HANDLE) -> io::Result<Self> {
        let mut token = ptr::null_mut();
        if unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let token = own(token)?;
        let user = token_data(raw(&token), TokenUser)?;
        let groups = token_data(raw(&token), TokenLogonSid)?;
        let integrity = token_data(raw(&token), TokenIntegrityLevel)?;
        let statistics = token_data(raw(&token), TokenStatistics)?;
        let session = token_data(raw(&token), TokenSessionId)?;
        unsafe {
            let groups = &*groups.as_ptr().cast::<TOKEN_GROUPS>();
            if groups.GroupCount != 1 || groups.Groups[0].Attributes & 0xc0000000 != 0xc0000000 {
                return Err(refused("Process has no unique Windows logon SID"));
            }
            let logon = sid_bytes(groups.Groups[0].Sid)?;
            let mut text = ptr::null_mut();
            if ConvertSidToStringSidW(logon.as_ptr() as _, &mut text) == 0 {
                return Err(io::Error::last_os_error());
            }
            let _allocation = Allocation(text.cast());
            let mut len = 0;
            while len < 256 && *text.add(len) != 0 {
                len += 1;
            }
            if len == 256 {
                return Err(refused("Invalid logon SID text"));
            }
            let logon = String::from_utf16(std::slice::from_raw_parts(text, len))
                .map_err(|_| refused("Invalid logon SID text"))?;
            let statistics = &*statistics.as_ptr().cast::<TOKEN_STATISTICS>();
            Ok(Self {
                user: sid_bytes((*(user.as_ptr().cast::<TOKEN_USER>())).User.Sid)?,
                logon,
                integrity: sid_bytes(
                    (*(integrity.as_ptr().cast::<TOKEN_MANDATORY_LABEL>()))
                        .Label
                        .Sid,
                )?,
                authentication: (
                    statistics.AuthenticationId.LowPart,
                    statistics.AuthenticationId.HighPart,
                ),
                session: *(session.as_ptr().cast::<u32>()),
            })
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct ImageId {
    volume: u64,
    file: [u8; 16],
}
fn image(path: &std::path::Path) -> io::Result<(File, ImageId)> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)?;
    if !crate::engine::windows_file_identity::attributes(&file).is_ok_and(|attributes|
        attributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) == 0)
    {
        return Err(refused("Controller executable is not an ordinary file"));
    }
    let id = crate::engine::windows_file_identity::file_id(&file)?;
    Ok((
        file,
        ImageId {
            volume: id.VolumeSerialNumber,
            file: id.FileId.Identifier,
        },
    ))
}

pub(crate) struct Peer {
    process: OwnedHandle,
    _image: File,
    pid:u32,
    birth: u64,
}
impl Peer {
    // Original named-pipe endpoint already verified and pinned the CLI
    // process. No extra per-pulse process query or caller-provided token.
    #[cfg(feature="mc02-native-input")]
    pub(crate) fn input_sender(&self)->(u32,u64) {
        (self.pid,self.birth)
    }
    pub(crate) fn wait_handle(&self) -> BorrowedHandle<'_> {
        self.process.as_handle()
    }
    pub(crate) fn alive(&self) -> bool {
        // Original handle remains pinned; PID reuse cannot turn it into another peer.
        (unsafe { WaitForSingleObject(raw(&self.process), 0) == WAIT_TIMEOUT })
            && crate::engine::windows_process_time::creation_filetime(raw(&self.process)).is_ok_and(|birth| birth == self.birth)
    }
}

// Bind the Windows-observed direct Node parent of the verified one-shot CLI.
// This is a process handle/birth check, not MCP-script attestation or an
// authorization to take over an abandoned Testing run.
#[derive(Debug)]
pub(crate) struct McpOwnerLifetime {
    process: OwnedHandle,
    pid: u32,
    birth: u64,
}
impl McpOwnerLifetime {
    pub(crate) fn same(&self, other: &Self) -> bool {
        self.pid == other.pid && self.birth == other.birth
    }
    pub(crate) fn status(&self) -> &'static str {
        let wait=unsafe {WaitForSingleObject(raw(&self.process),0)};
        if wait==WAIT_OBJECT_0 {return "BOUND_PARENT_EXIT_CONFIRMED";}
        if wait!=WAIT_TIMEOUT {return "BOUND_PARENT_STATUS_UNKNOWN";}
        if !crate::engine::windows_process_time::creation_filetime(raw(&self.process)).is_ok_and(|birth| birth == self.birth) {
            return "BOUND_PARENT_STATUS_UNKNOWN";
        }
        "BOUND_PARENT_RUNNING_AT_INSPECTION"
    }
}
impl Peer {
    pub(crate) fn mcp_parent(&self)->io::Result<McpOwnerLifetime> {
        if !self.alive() {return Err(refused("Testing CLI peer is no longer live"));}
        let child_pid=unsafe {GetProcessId(raw(&self.process))};
        if child_pid==0 {return Err(refused("Testing CLI identity unavailable"));}
        // ToolHelp parent attribution can be spoofed by a privileged
        // same-user caller; never treat it as a signed MCP identity.
        let snapshot=own(unsafe {CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS,0)})?;
        let mut entry:PROCESSENTRY32W=unsafe {mem::zeroed()};
        entry.dwSize=mem::size_of::<PROCESSENTRY32W>() as u32;
        if unsafe {Process32FirstW(raw(&snapshot),&mut entry)}==0 {
            return Err(io::Error::last_os_error());
        }
        let mut parent_pid=0;
        loop {
            if entry.th32ProcessID==child_pid {
                parent_pid=entry.th32ParentProcessID;
                break;
            }
            if unsafe {Process32NextW(raw(&snapshot),&mut entry)}==0 {break;}
        }
        if parent_pid==0||parent_pid==child_pid {
            return Err(refused("Testing MCP direct parent unavailable"));
        }
        let process=own(unsafe {OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZATION_SYNCHRONIZE,
            0,parent_pid)})?;
        if Identity::process(raw(&process))? !=
           Identity::process(unsafe {GetCurrentProcess()})? {
            return Err(refused("Testing MCP parent logon/session differs"));
        }
        let image_path=crate::engine::windows_process_image::query(raw(&process))?;
        let parent_path=std::ffi::OsString::from_wide(&image_path);
        if !std::path::Path::new(&parent_path).file_name()
           .and_then(|name|name.to_str())
           .is_some_and(|name|name.eq_ignore_ascii_case("node.exe")) {
            return Err(refused("Testing MCP direct parent is not Node"));
        }
        let birth = crate::engine::windows_process_time::creation_filetime(raw(&process))?;
        if birth==0||birth>self.birth {
            return Err(refused("Testing MCP parent creation is inconsistent"));
        }
        let owner=McpOwnerLifetime{process,pid:parent_pid,birth};
        if owner.status()!="BOUND_PARENT_RUNNING_AT_INSPECTION"||!self.alive() {
            return Err(refused("Testing MCP parent or CLI lifetime uncertain"));
        }
        Ok(owner)
    }
}

pub(crate) struct Endpoint {
    name: Vec<u16>,
    identity: Identity,
    image: ImageId,
    _image: File,
}
impl Endpoint {
    pub(crate) fn current() -> io::Result<Self> {
        let identity = Identity::process(unsafe { GetCurrentProcess() })?;
        let (pinned, image) = image(&std::env::current_exe()?)?;
        let mut hash = Sha256::new();
        hash.update(identity.logon.as_bytes());
        hash.update(identity.session.to_le_bytes());
        hash.update(image.volume.to_le_bytes());
        hash.update(image.file);
        let digest: [u8; 32] = hash.finalize().into();
        let suffix = digest[..16]
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Ok(Self {
            name: wide(&format!(r"\\.\pipe\LazyTester-v1-{suffix}")),
            identity,
            image,
            _image: pinned,
        })
    }
    fn verify(&self, pipe: HANDLE, server_side: bool) -> io::Result<Peer> {
        let mut pid = 0;
        let get_peer = if server_side {
            GetNamedPipeClientProcessId
        } else {
            GetNamedPipeServerProcessId
        };
        if unsafe { get_peer(pipe, &mut pid) } == 0 || pid == 0 {
            return Err(refused("No Windows pipe peer"));
        }
        let process = own(unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZATION_SYNCHRONIZE,
                0,
                pid,
            )
        })?;
        if Identity::process(raw(&process))? != self.identity {
            return Err(refused(
                "Controller peer user/logon/session/integrity differs",
            ));
        }
        let path = crate::engine::windows_process_image::query(raw(&process))?;
        let (pinned, identity) = image(std::path::Path::new(
            &std::ffi::OsString::from_wide(&path),
        ))?;
        if identity != self.image {
            return Err(refused("Controller peer executable differs"));
        }
        let birth = crate::engine::windows_process_time::creation_filetime(raw(&process))?;
        let mut repeated = 0;
        if birth == 0 || unsafe { get_peer(pipe, &mut repeated) } == 0 || pid != repeated {
            return Err(refused("Controller peer identity changed"));
        }
        let peer = Peer {
            process,
            _image: pinned,
            pid,
            birth,
        };
        if !peer.alive() {
            return Err(refused("Controller peer already exited"));
        }
        Ok(peer)
    }
    pub(crate) fn connect(&self) -> io::Result<File> {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            // Exact pipe rights; generic file-open helpers may add unrelated filesystem rights.
            let handle = unsafe {
                CreateFileW(
                    self.name.as_ptr(),
                    windows_sys::Win32::Foundation::GENERIC_READ | FILE_WRITE_DATA,
                    0,
                    ptr::null(),
                    OPEN_EXISTING,
                    SECURITY_SQOS_PRESENT | SECURITY_IDENTIFICATION,
                    ptr::null_mut(),
                )
            };
            let result = if handle == INVALID_HANDLE_VALUE {
                Err(io::Error::last_os_error())
            } else {
                Ok(unsafe { File::from_raw_handle(handle as _) })
            };
            match result {
                Err(error) if error.raw_os_error() == Some(231) && Instant::now() < deadline => unsafe {
                    WaitNamedPipeW(self.name.as_ptr(), 100);
                },
                other => return other,
            }
        }
    }
    fn bind(&self) -> io::Result<File> {
        let sddl = wide(&format!(
            "D:P(A;;0x{CLIENT_RIGHTS:08x};;;{})",
            self.identity.logon
        ));
        let mut descriptor = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let _descriptor = Allocation(descriptor);
        let attributes = SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: descriptor,
            bInheritHandle: 0,
        };
        let handle = unsafe {
            CreateNamedPipeW(
                self.name.as_ptr(),
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE | FILE_FLAG_OVERLAPPED,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_REJECT_REMOTE_CLIENTS,
                1,
                65536,
                65536,
                1000,
                &attributes,
            )
        };
        if handle == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        Ok(unsafe { File::from_raw_handle(handle as _) })
    }
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
    if frame.len() >= 32768 {
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
    let mut reader = BufReader::new(pipe.try_clone()?);
    let text = read_message(&mut reader)?
        .ok_or_else(|| io::Error::other("Controller disconnected before its reply"))?;
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
            deadline.saturating_duration_since(Instant::now()).as_nanos()
                .div_ceil(1_000_000).min(u128::from(u32::MAX - 1)) as u32
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
            let milliseconds = remaining.as_nanos().div_ceil(1_000_000)
                .min(u128::from(u32::MAX - 1)) as u32;
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
    if bytes.len() >= 32768 {
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
mod tests {
    use super::super::SessionSnapshot;
    use super::*;
    use std::sync::atomic::AtomicU64;

    fn endpoints() -> (Endpoint, Endpoint) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let mut server = Endpoint::current().unwrap();
        let mut client = Endpoint::current().unwrap();
        let name = wide(&format!(
            r"\\.\pipe\LazyTester-disposable-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        server.name = name.clone();
        client.name = name;
        (server, client)
    }

    #[test]
    fn pinned_testing_parent_handle_rejects_birth_mismatch() {
        // Own-process Win32 metadata only; no Node or Minecraft started.
        let pid=unsafe {GetCurrentProcessId()};
        let open_self=||own(unsafe {OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZATION_SYNCHRONIZE,
            0,pid)}).unwrap();
        let h=open_self();
        let mut created:FILETIME=unsafe {mem::zeroed()};
        let mut exited=created;
        let mut kernel=created;
        let mut user=created;
        assert_ne!(unsafe {GetProcessTimes(raw(&h),
            &mut created,&mut exited,&mut kernel,&mut user)},0);
        let birth=(created.dwHighDateTime as u64)<<32|created.dwLowDateTime as u64;
        let valid=McpOwnerLifetime{process:h,pid,birth};
        assert_eq!(valid.status(),"BOUND_PARENT_RUNNING_AT_INSPECTION");
        let invalid=McpOwnerLifetime{process:open_self(),pid,birth:birth+1};
        assert!(!valid.same(&invalid));
        assert_eq!(invalid.status(),"BOUND_PARENT_STATUS_UNKNOWN");
    }

    #[test]
    fn native_token_logon_membership_is_usable_for_pipe_acl() {
        let context = Identity::process(unsafe { GetCurrentProcess() }).unwrap();
        let mut sid = ptr::null_mut();
        assert_ne!(
            unsafe {
                windows_sys::Win32::Security::Authorization::ConvertStringSidToSidW(
                    wide(&context.logon).as_ptr(),
                    &mut sid,
                )
            },
            0
        );
        let _sid = Allocation(sid);
        let mut logon_member = 0;
        let mut user_member = 0;
        assert_ne!(
            unsafe { CheckTokenMembership(ptr::null_mut(), sid, &mut logon_member) },
            0
        );
        assert_ne!(
            unsafe {
                CheckTokenMembership(
                    ptr::null_mut(),
                    context.user.as_ptr() as _,
                    &mut user_member,
                )
            },
            0
        );
        assert_eq!(
            (logon_member, user_member),
            (1, 1),
            "Windows membership metadata only; no SID values are emitted"
        );
    }

    #[test]
    fn native_pipe_dacl_grants_exact_client_rights() {
        use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_KERNEL_OBJECT};
        let (endpoint, _) = endpoints();
        let pipe = endpoint.bind().unwrap();
        let mut descriptor = ptr::null_mut();
        assert_eq!(
            unsafe {
                GetSecurityInfo(
                    pipe.as_raw_handle() as _,
                    SE_KERNEL_OBJECT,
                    DACL_SECURITY_INFORMATION
                        | OWNER_SECURITY_INFORMATION
                        | GROUP_SECURITY_INFORMATION,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null_mut(),
                    &mut descriptor,
                )
            },
            0
        );
        let _descriptor = Allocation(descriptor);
        let mut token = ptr::null_mut();
        let mut impersonation = ptr::null_mut();
        assert_ne!(
            unsafe {
                OpenProcessToken(
                    GetCurrentProcess(),
                    TOKEN_QUERY | TOKEN_DUPLICATE,
                    &mut token,
                )
            },
            0
        );
        let token = own(token).unwrap();
        assert_ne!(
            unsafe { DuplicateToken(raw(&token), SecurityImpersonation, &mut impersonation) },
            0
        );
        let impersonation = own(impersonation).unwrap();
        let mut mapping = GENERIC_MAPPING {
            GenericRead: FILE_GENERIC_READ,
            GenericWrite: FILE_GENERIC_WRITE,
            GenericExecute: FILE_GENERIC_EXECUTE,
            GenericAll: FILE_ALL_ACCESS,
        };
        let mut privileges = vec![0usize; 128];
        let mut size = (privileges.len() * mem::size_of::<usize>()) as u32;
        let mut granted = 0;
        let mut allowed = 0;
        assert_ne!(
            unsafe {
                AccessCheck(
                    descriptor,
                    raw(&impersonation),
                    CLIENT_RIGHTS,
                    &mut mapping,
                    privileges.as_mut_ptr().cast(),
                    &mut size,
                    &mut granted,
                    &mut allowed,
                )
            },
            0
        );
        assert_eq!(
            (allowed, granted),
            (1, CLIENT_RIGHTS),
            "DACL access decision only; no identities are emitted"
        );
        // FILE_APPEND_DATA aliases FILE_CREATE_PIPE_INSTANCE for named pipes.
        assert_ne!(
            unsafe {
                AccessCheck(
                    descriptor,
                    raw(&impersonation),
                    FILE_APPEND_DATA,
                    &mut mapping,
                    privileges.as_mut_ptr().cast(),
                    &mut size,
                    &mut granted,
                    &mut allowed,
                )
            },
            0
        );
        assert_eq!(
            (allowed, granted),
            (0, 0),
            "The client must not create another server instance"
        );
    }

    #[test]
    fn request_frame_preserves_wire_and_refuses_oversize_before_write() {
        #[derive(Default)]
        struct CountingWriter { bytes: Vec<u8>, writes: usize }
        impl Write for CountingWriter {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                self.writes += 1;
                self.bytes.extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> { Ok(()) }
        }
        let mut writer = CountingWriter::default();
        write_request(&mut writer, &SessionRequest::InspectAgentTarget { slot: 3 }).unwrap();
        assert_eq!(writer.writes, 1);
        assert_eq!(writer.bytes.as_slice(), b"{\"operation\":\"InspectAgentTarget\",\"slot\":3}\n");
        assert!(matches!(
            serde_json::from_slice::<SessionRequest>(&writer.bytes).unwrap(),
            SessionRequest::InspectAgentTarget { slot: 3 }
        ));

        // Exercise the framing limit independently of semantic request admission.
        let request = |name: String| SessionRequest::ClearProfileData {
            slot: 1, confirmation_token: "a".repeat(64),
            confirmation_name: name, selected_storage: None,
        };
        let overhead = serde_json::to_vec(&request(String::new())).unwrap().len();
        let mut boundary = CountingWriter::default();
        write_request(&mut boundary, &request("x".repeat(32767 - overhead))).unwrap();
        assert_eq!(boundary.bytes.len(), 32768);
        assert_eq!(boundary.writes, 1);
        assert!(read_message(&mut io::Cursor::new(&boundary.bytes)).unwrap().is_some());

        let mut rejected = CountingWriter::default();
        assert_eq!(write_request(&mut rejected, &request("x".repeat(32768 - overhead)))
            .unwrap_err().kind(), io::ErrorKind::InvalidInput);
        assert_eq!(rejected.writes, 0);
    }

    #[test]
    fn confirmed_native_reply_survives_ack_write_failure_without_replay() {
        // Pure transport completion fixture. No controller, pipe or
        // Minecraft process is created by this test definition.
        struct BrokenAck;
        impl Write for BrokenAck {
            fn write(&mut self,_bytes:&[u8])->io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::BrokenPipe,
                    "acknowledgement transport unavailable"))
            }
            fn flush(&mut self)->io::Result<()> {Ok(())}
        }
        let mut good=Vec::<u8>::new();
        let reply=acknowledge_confirmed_reply(
            &mut good,Ok(SessionSnapshot::stopped()));
        assert_eq!(good.as_slice(),b"ack\n");
        assert_eq!(reply.unwrap().profiles.len(),5);
        let mut broken=BrokenAck;
        let confirmed=acknowledge_confirmed_reply(
            &mut broken,Ok(SessionSnapshot::stopped()));
        assert_eq!(confirmed.unwrap().profiles.len(),5,
            "a fully received success must not become uncertain on ACK loss");
        let refused=acknowledge_confirmed_reply(
            &mut broken,Err("Native operation refused".into()));
        assert_eq!(refused.unwrap_err(),"Native operation refused",
            "an explicit native refusal must retain its original status");
        assert!(serde_json::from_str::<SessionReply>("bad partial reply").is_err(),
            "missing/malformed response is never accepted as a result");
    }

    #[test]
    fn native_wait_distinguishes_deadline_lifetime_and_queued_request() {
        let (endpoint, client) = endpoints();
        let mut server = Server::start(endpoint).unwrap();
        let lifetime = own(unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) }).unwrap();
        let hint = own(unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) }).unwrap();
        assert_ne!(unsafe { SetEvent(raw(&hint)) }, 0);
        for _ in 0..2 {
            assert!(matches!(server.wait_for_activity(std::iter::once(lifetime.as_handle()),
                Some(hint.as_handle()), Duration::ZERO).unwrap(), Activity::AudioHint));
        }
        assert_ne!(unsafe { ResetEvent(raw(&hint)) }, 0);
        assert!(matches!(server.wait_for_activity(std::iter::once(lifetime.as_handle()), None, Duration::ZERO).unwrap(),
            Activity::Deadline));
        assert_ne!(unsafe { SetEvent(raw(&server.request_ready)) }, 0);
        assert!(matches!(server.wait_for_activity(std::iter::once(lifetime.as_handle()), None, Duration::ZERO).unwrap(),
            Activity::Deadline));
        assert_ne!(unsafe { SetEvent(raw(&server.request_ready)) }, 0);
        assert_ne!(unsafe { SetEvent(raw(&lifetime)) }, 0);
        assert!(matches!(server.wait_for_activity(std::iter::once(lifetime.as_handle()), None, Duration::ZERO).unwrap(),
            Activity::Lifetime));
        let caller = thread::spawn(move || exchange(&client, client.connect().unwrap(), &SessionRequest::CloseOne { slot: 2 }));
        // Force the queue and lifetime to be ready together, including the
        // case where the notification was consumed before the actor looked.
        assert_eq!(unsafe { WaitForSingleObject(raw(&server.request_ready), 5000) }, WAIT_OBJECT_0);
        assert_ne!(unsafe { SetEvent(raw(&hint)) }, 0);
        let Activity::Request(envelope) = server.wait_for_activity(
            std::iter::once(lifetime.as_handle()), Some(hint.as_handle()), Duration::ZERO).unwrap() else {
            panic!("queued request must precede maintenance");
        };
        assert!(matches!(envelope.request, SessionRequest::CloseOne { slot: 2 }));
        envelope.reply.send(Ok(SessionSnapshot::stopped())).unwrap();
        assert!(caller.join().unwrap().unwrap().is_ok());
        assert!(matches!(server.wait_for_activity(std::iter::once(lifetime.as_handle()), Some(hint.as_handle()), Duration::ZERO).unwrap(),
            Activity::Lifetime));
        assert_eq!(unsafe { WaitForSingleObject(raw(&hint), 0) }, WAIT_OBJECT_0);
        assert!(server.stop());
        assert!(matches!(server.wait_for_activity(std::iter::empty(), None, Duration::ZERO).unwrap(),
            Activity::Disconnected));
    }

    #[test]
    fn real_pipe_reconnects_and_round_trips_five_slot_contract_without_minecraft() {
        let (endpoint, client) = endpoints();
        let mut server = Server::start(endpoint).unwrap();
        let (done, finished) = mpsc::channel();
        let actor = thread::spawn(move || {
            for _ in 0..3 {
                let Activity::Request(envelope) = server.wait_for_activity(
                    std::iter::empty(), None, Duration::from_secs(5)).unwrap() else {
                    panic!("authenticated request must wake the controller");
                };
                assert!(matches!(envelope.request, SessionRequest::Inspect));
                assert!(envelope.peer.alive());
                assert!(envelope.peer.birth > 0);
                envelope.reply.send(Ok(SessionSnapshot::stopped())).unwrap();
            }
            finished.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(
                server.stop(),
                "transport cancellation must settle before fixture cleanup"
            );
        });
        for _ in 0..3 {
            let snapshot = exchange(&client, client.connect().unwrap(), &SessionRequest::Inspect)
                .unwrap()
                .unwrap();
            assert_eq!(snapshot.profiles.len(), 5);
        }
        done.send(()).unwrap();
        actor.join().unwrap();
    }

    #[test]
    fn changed_image_or_logon_is_rejected_before_request_delivery_and_valid_peer_recovers() {
        let (endpoint, mut client) = endpoints();
        let mut server = Server::start(endpoint).unwrap();
        client.image.file[0] ^= 1;
        let connection = client.connect().unwrap();
        assert_eq!(
            exchange(&client, connection, &SessionRequest::Inspect)
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        client.image.file[0] ^= 1;
        client.identity.session ^= 1;
        let connection = client.connect().unwrap();
        assert_eq!(
            exchange(&client, connection, &SessionRequest::Inspect)
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        client.identity.session ^= 1;
        assert!(server.requests.try_recv().is_err());
        let (done, finished) = mpsc::channel();
        let actor = thread::spawn(move || {
            let envelope = server
                .requests
                .recv_timeout(Duration::from_secs(5))
                .unwrap();
            envelope.reply.send(Ok(SessionSnapshot::stopped())).unwrap();
            finished.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(server.stop());
        });
        assert!(
            exchange(&client, client.connect().unwrap(), &SessionRequest::Inspect)
                .unwrap()
                .is_ok()
        );
        done.send(()).unwrap();
        actor.join().unwrap();
    }

    #[test]
    fn stalled_request_and_missing_ack_release_connection_without_replay() {
        let (endpoint, client) = endpoints();
        let mut server = Server::start_with_budget(endpoint, Duration::from_secs(1)).unwrap();

        // Hold an authenticated client open after only a partial frame.
        let mut partial = client.connect().unwrap();
        partial.write_all(b"{").unwrap();
        let mut second = client.connect().unwrap();
        assert!(server.requests.try_recv().is_err());
        drop(partial);

        // A valid operation is admitted exactly once. Withhold its ACK
        // even after receiving the complete response; the next peer can
        // reconnect without changing or replaying the first request.
        write_request(&mut second, &SessionRequest::Inspect).unwrap();
        let envelope = server.requests.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(envelope.request, SessionRequest::Inspect));
        envelope.reply.send(Ok(SessionSnapshot::stopped())).unwrap();
        let mut unacked = BufReader::new(second);
        let response = read_message(&mut unacked).unwrap().unwrap();
        assert!(serde_json::from_str::<SessionReply>(&response).unwrap().is_ok());

        let mut third = client.connect().unwrap();
        assert!(server.requests.try_recv().is_err());
        drop(unacked);
        write_request(&mut third, &SessionRequest::Inspect).unwrap();
        let envelope = server.requests.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(matches!(envelope.request, SessionRequest::Inspect));
        envelope.reply.send(Ok(SessionSnapshot::stopped())).unwrap();
        let mut reader = BufReader::new(third.try_clone().unwrap());
        assert!(serde_json::from_str::<SessionReply>(
            &read_message(&mut reader).unwrap().unwrap()).unwrap().is_ok());
        third.write_all(b"ack\n").unwrap();
        assert!(server.stop());
    }

    #[test]
    fn first_instance_collision_and_oversized_request_never_reach_lifecycle_owner() {
        let (endpoint, client) = endpoints();
        let mut server = Server::start(endpoint).unwrap();
        assert!(client.bind().is_err());
        {
            let mut connection = client.connect().unwrap();
            connection.write_all(&vec![b'x'; 32769]).unwrap();
            let mut input = BufReader::new(connection);
            assert!(read_message(&mut input).map_or(true, |text| text.is_none()));
        }
        assert!(server.requests.try_recv().is_err());
        assert!(server.stop());
    }

    #[test]
    fn a_new_client_process_reconnects_after_the_previous_client_exits() {
        use std::{
            os::windows::process::CommandExt,
            process::{Command, Stdio},
        };
        let (endpoint, client) = endpoints();
        let mut server = Server::start(endpoint).unwrap();
        let (expected, identities) = mpsc::channel();
        let (observed_exit, exits) = mpsc::channel();
        let (done, finished) = mpsc::channel();
        let actor = thread::spawn(move || {
            for _ in 0..2 {
                let pid = identities.recv_timeout(Duration::from_secs(5)).unwrap();
                let Activity::Request(envelope) = server.wait_for_activity(
                    std::iter::empty(), None, Duration::from_secs(5)).unwrap() else {
                    panic!("verified child request did not wake the controller");
                };
                assert_eq!(unsafe { GetProcessId(raw(&envelope.peer.process)) }, pid);
                assert_ne!(pid, std::process::id());
                envelope.reply.send(Ok(SessionSnapshot::stopped())).unwrap();
                // The original verified child handle stays pinned after reply.
                assert!(matches!(server.wait_for_activity(
                    std::iter::once(envelope.peer.wait_handle()), None, Duration::from_secs(5)).unwrap(),
                    Activity::Lifetime));
                assert!(!envelope.peer.alive());
                observed_exit.send(pid).unwrap();
            }
            finished.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(server.stop());
        });
        let name = String::from_utf16(&client.name[..client.name.len() - 1]).unwrap();
        for _ in 0..2 {
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "engine::native_clients_session::pipe::tests::client_process_fixture",
                    "--nocapture",
                ])
                .env("LAZY_TESTER_PIPE_FIXTURE", &name)
                .creation_flags(0x08000000)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();
            expected.send(child.id()).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if Instant::now() >= deadline {
                    // Only this original self-test child. It cannot launch or control Minecraft.
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("Disposable pipe client timed out");
                }
                thread::sleep(Duration::from_millis(10));
            };
            assert!(status.success());
            assert_eq!(exits.recv_timeout(Duration::from_secs(5)).unwrap(), child.id());
        }
        done.send(()).unwrap();
        actor.join().unwrap();
    }

    #[test]
    fn client_process_fixture() {
        let Ok(name) = std::env::var("LAZY_TESTER_PIPE_FIXTURE") else {
            return;
        };
        assert!(name.starts_with(r"\\.\pipe\LazyTester-disposable-test-") && name.len() < 200);
        let mut endpoint = Endpoint::current().unwrap();
        endpoint.name = wide(&name);
        let snapshot = exchange(
            &endpoint,
            endpoint.connect().unwrap(),
            &SessionRequest::Inspect,
        )
        .unwrap()
        .unwrap();
        assert_eq!(snapshot.profiles.len(), 5);
    }
}
