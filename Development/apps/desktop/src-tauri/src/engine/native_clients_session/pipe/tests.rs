//! Original disposable pipe fixtures. Retain pipe::tests module path.
use super::peer_endpoint::{Allocation, Identity, CLIENT_RIGHTS, wide};
use std::os::windows::io::AsHandle;
use windows_sys::Win32::{Foundation::FILETIME, Security::*};
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

#[test]
fn finite_wait_conversion_preserves_rounding_and_infinite_sentinel() {
    assert_eq!(finite_wait_millis(Duration::ZERO), 0);
    assert_eq!(finite_wait_millis(Duration::from_nanos(1)), 1);
    assert_eq!(finite_wait_millis(Duration::from_millis(1)), 1);
    assert_eq!(finite_wait_millis(Duration::from_nanos(1_000_001)), 2);
    assert_eq!(finite_wait_millis(Duration::from_millis(u64::from(u32::MAX - 1))), u32::MAX - 1);
    assert_eq!(finite_wait_millis(Duration::MAX), u32::MAX - 1);
}
