//! Local reconnection for one retained controller, not a privileged/general broker.
//! Same logon, session, user, integrity and executable file identity at both peers.
use super::{read_message, SessionReply, SessionRequest};
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::{self, BufReader, Write},
    mem,
    os::windows::{
        ffi::OsStringExt,
        fs::OpenOptionsExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
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
        GetLastError, LocalFree, ERROR_PIPE_CONNECTED, FILETIME, HANDLE, INVALID_HANDLE_VALUE,
        WAIT_TIMEOUT,
    },
    Security::{
        Authorization::{
            ConvertSidToStringSidW, ConvertStringSecurityDescriptorToSecurityDescriptorW,
        },
        *,
    },
    Storage::FileSystem::*,
    System::{Pipes::*, Threading::*, IO::CancelSynchronousIo},
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
    let mut tag: FILE_ATTRIBUTE_TAG_INFO = unsafe { mem::zeroed() };
    let mut id: FILE_ID_INFO = unsafe { mem::zeroed() };
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _,
            FileAttributeTagInfo,
            (&mut tag as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            mem::size_of_val(&tag) as u32,
        )
    } == 0
        || tag.FileAttributes & (FILE_ATTRIBUTE_REPARSE_POINT | FILE_ATTRIBUTE_DIRECTORY) != 0
    {
        return Err(refused("Controller executable is not an ordinary file"));
    }
    if unsafe {
        GetFileInformationByHandleEx(
            file.as_raw_handle() as _,
            FileIdInfo,
            (&mut id as *mut FILE_ID_INFO).cast(),
            mem::size_of_val(&id) as u32,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
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
    birth: u64,
}
impl Peer {
    pub(crate) fn alive(&self) -> bool {
        // Original handle remains pinned; PID reuse cannot turn it into another peer.
        let mut created: FILETIME = unsafe { mem::zeroed() };
        let mut exit = created;
        let mut kernel = created;
        let mut user = created;
        unsafe {
            WaitForSingleObject(raw(&self.process), 0) == WAIT_TIMEOUT
                && GetProcessTimes(
                    raw(&self.process),
                    &mut created,
                    &mut exit,
                    &mut kernel,
                    &mut user,
                ) != 0
                && (((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64)
                    == self.birth
        }
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
        let mut created: FILETIME = unsafe { mem::zeroed() };
        let mut exit = created;
        let mut kernel = created;
        let mut user = created;
        if unsafe {
            GetProcessTimes(
                raw(&process),
                &mut created,
                &mut exit,
                &mut kernel,
                &mut user,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let birth = ((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64;
        let mut repeated = 0;
        if birth == 0 || unsafe { get_peer(pipe, &mut repeated) } == 0 || pid != repeated {
            return Err(refused("Controller peer identity changed"));
        }
        let peer = Peer {
            process,
            _image: pinned,
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
                PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE,
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

pub(crate) fn exchange(
    endpoint: &Endpoint,
    mut pipe: File,
    request: &SessionRequest,
) -> io::Result<SessionReply> {
    let peer = endpoint.verify(pipe.as_raw_handle() as _, false)?;
    serde_json::to_writer(&mut pipe, request)?;
    pipe.write_all(b"\n")?;
    let mut reader = BufReader::new(pipe.try_clone()?);
    let text = read_message(&mut reader)?
        .ok_or_else(|| io::Error::other("Controller disconnected before its reply"))?;
    if !peer.alive() {
        return Err(refused(
            "Controller exited during the request; outcome uncertain",
        ));
    }
    let reply = serde_json::from_str(&text).map_err(io::Error::other)?;
    // Acknowledgement lets the server disconnect without discarding unread response bytes.
    pipe.write_all(b"ack\n")?;
    Ok(reply)
}

pub(crate) struct Envelope {
    pub request: SessionRequest,
    pub peer: Peer,
    pub reply: mpsc::SyncSender<SessionReply>,
}
pub(crate) struct Server {
    pub requests: mpsc::Receiver<Envelope>,
    stopped: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl Server {
    pub(crate) fn start(endpoint: Endpoint) -> io::Result<Self> {
        // Bind before acknowledging startup; FIRST_PIPE_INSTANCE stays retained across reconnects.
        let mut file = endpoint.bind()?;
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = Arc::clone(&stopped);
        let (sender, requests) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            while !stop.load(Ordering::Acquire) {
                let connected =
                    unsafe { ConnectNamedPipe(file.as_raw_handle() as _, ptr::null_mut()) } != 0
                        || unsafe { GetLastError() } == ERROR_PIPE_CONNECTED;
                if !connected {
                    if stop.load(Ordering::Acquire) {
                        break;
                    }
                    unsafe {
                        DisconnectNamedPipe(file.as_raw_handle() as _);
                    }
                    thread::sleep(Duration::from_millis(20));
                    continue;
                }
                let _ = serve(&endpoint, &mut file, &sender, &stop);
                unsafe {
                    DisconnectNamedPipe(file.as_raw_handle() as _);
                }
            }
        });
        Ok(Self {
            requests,
            stopped,
            worker: Some(worker),
        })
    }
}
fn serve(
    endpoint: &Endpoint,
    file: &mut File,
    sender: &mpsc::SyncSender<Envelope>,
    stop: &AtomicBool,
) -> io::Result<()> {
    if stop.load(Ordering::Acquire) {
        return Ok(());
    }
    let peer = endpoint.verify(file.as_raw_handle() as _, true)?;
    let mut reader = BufReader::new(file.try_clone()?);
    let text =
        read_message(&mut reader)?.ok_or_else(|| io::Error::other("Manager disconnected"))?;
    if !peer.alive() {
        return Err(refused("Manager exited before request admission"));
    }
    let request = serde_json::from_str(&text).map_err(io::Error::other)?;
    let (reply, response) = mpsc::sync_channel(1);
    sender
        .send(Envelope {
            request,
            peer,
            reply,
        })
        .map_err(|_| io::Error::other("Controller stopped"))?;
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
    let mut bytes = serde_json::to_vec(&reply)?;
    if bytes.len() >= 32768 {
        bytes = serde_json::to_vec(&SessionReply::Err(
            "Controller response exceeded its limit".into(),
        ))?;
    }
    bytes.push(b'\n');
    file.write_all(&bytes)?;
    if read_message(&mut reader)?.as_deref() != Some("ack\n") {
        return Err(io::Error::other("Response was not acknowledged"));
    }
    Ok(())
}
impl Server {
    fn stop(&mut self) -> bool {
        self.stopped.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            // Only cancel this transport thread's synchronous I/O, never game threads.
            // Its own stack/buffers remain alive until it finishes, even on a slow cancel.
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
    fn real_pipe_reconnects_and_round_trips_five_slot_contract_without_minecraft() {
        let (endpoint, client) = endpoints();
        let mut server = Server::start(endpoint).unwrap();
        let (done, finished) = mpsc::channel();
        let actor = thread::spawn(move || {
            for _ in 0..3 {
                let envelope = server
                    .requests
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
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
        let (done, finished) = mpsc::channel();
        let actor = thread::spawn(move || {
            for _ in 0..2 {
                let pid = identities.recv_timeout(Duration::from_secs(5)).unwrap();
                let envelope = server
                    .requests
                    .recv_timeout(Duration::from_secs(5))
                    .unwrap();
                assert_eq!(unsafe { GetProcessId(raw(&envelope.peer.process)) }, pid);
                assert_ne!(pid, std::process::id());
                envelope.reply.send(Ok(SessionSnapshot::stopped())).unwrap();
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
