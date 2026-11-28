//! Existing native bootstrap fixtures; the module path remains independent_bootstrap::tests.
    use super::*;

    #[test]
    fn existing_registry_store_uses_explicit_current_user_root() {
        const CHILD: &str = "LAZY_TESTER_REGISTRY_ROOT_FIXTURE_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let module = module_path!().split_once("::").unwrap().1;
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &format!("{module}::existing_registry_store_uses_explicit_current_user_root"), "--nocapture"])
                .env(CHILD,"1").output().unwrap();
            assert!(output.status.success(),"{}\n{}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
            return;
        }
        use windows_sys::Win32::System::Registry::{RegOverridePredefKey,RegDeleteTreeW};
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let logical = open_directory(&parent,FILE_READ_ATTRIBUTES).unwrap();
        let store = create_private_slot(&logical,&parent,1).unwrap();
        let id = directory_id(&store.private).unwrap();
        let path = wide(OsStr::new(&format!("Software\\MivubiEducationPilot\\{}",registry_leaf(&id,1).unwrap()))).unwrap();
        let mut empty = ptr::null_mut();let mut disposition=0;
        assert_eq!(unsafe{RegCreateKeyExW(store.registry.0,wide(OsStr::new("EmptyView")).unwrap().as_ptr(),0,ptr::null(),0,KEY_ALL_ACCESS,ptr::null(),&mut empty,&mut disposition)},0);
        let empty = RegistryKey(empty);
        struct Restore;
        impl Drop for Restore {fn drop(&mut self){assert_eq!(unsafe{RegOverridePredefKey(HKEY_CURRENT_USER,ptr::null_mut())},0);}}
        assert_eq!(unsafe{RegOverridePredefKey(HKEY_CURRENT_USER,empty.0)},0);
        let restore = Restore;
        let result = open_existing_registry_store(&id,1);
        let failure = result.as_ref().err().map(ToString::to_string);
        let success = result.is_ok();drop(result);drop(restore);drop(empty);drop(store);
        assert_eq!(unsafe{RegDeleteTreeW(HKEY_CURRENT_USER,path.as_ptr())},0);
        drop(logical);assert_eq!(parent.parent(),Some(std::env::temp_dir().as_path()));std::fs::remove_dir_all(parent).unwrap();
        assert!(success,"Existing store was mistaken for missing: {failure:?}");
    }
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetExitCodeProcess};

    // Writable receipt exists only for our cooperative C++ control, never the game.
    pub(super) struct FixtureReceipt(pub(super) OwnedHandle);
    impl FixtureReceipt {
        fn new() -> Self {
            let raw = unsafe {
                CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    ptr::null(),
                    PAGE_READWRITE,
                    0,
                    2064,
                    ptr::null(),
                )
            };
            assert!(!raw.is_null());
            let owned = unsafe { OwnedHandle::from_raw_handle(raw as _) };
            assert_ne!(
                unsafe { SetHandleInformation(raw, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) },
                0
            );
            Self(owned)
        }
        pub(super) fn environment(&self, environment: Vec<u16>) -> io::Result<Vec<u16>> {
            let mut rows: Vec<Vec<u16>> = environment
                .split(|&unit| unit == 0)
                .filter(|row| !row.is_empty())
                .map(|row| row.to_vec())
                .collect();
            rows.push(
                format!(
                    "MIVUBI_BOOTSTRAP_TEST_SECTION={}",
                    self.0.as_raw_handle() as usize
                )
                .encode_utf16()
                .collect(),
            );
            rows.sort_by(|a, b| ordinal_names(environment_name(a), environment_name(b)).unwrap());
            let mut result = Vec::new();
            for row in rows {
                result.extend(row);
                result.push(0);
            }
            result.push(0);
            if result.len() > 32767 {
                return Err(invalid("Fixture environment too large"));
            }
            Ok(result)
        }
    }

    type FixtureEntry = unsafe extern "system" fn(HANDLE, HANDLE, u32, u64, HANDLE, HANDLE) -> u32;
    fn fixture_digests(path: &Path) -> PayloadDigests {
        PayloadDigests {
            enroller: payload_digest(&pin_image(&path.join("bootstrap-enroll.dll")).unwrap())
                .unwrap(),
            compatibility: payload_digest(&pin_image(&path.join("bootstrap-compat.dll")).unwrap())
                .unwrap(),
        }
    }
    pub(super) fn prepare_fixture(
        library: &BootstrapLibrary,
        child: &PendingEducationBootstrap,
        proof: &FixtureReceipt,
        expect_enrolled: bool,
    ) -> u32 {
        let symbol = if expect_enrolled {
            c"PrepareOwnedFixtureForEnrollment"
        } else {
            c"PrepareOwnedFixture"
        };
        let entry = unsafe { GetProcAddress(library.module, symbol.as_ptr().cast()) }.unwrap();
        let entry: FixtureEntry = unsafe { mem::transmute(entry) };
        unsafe {
            entry(
                child.process.as_raw_handle() as HANDLE,
                child._thread.as_raw_handle() as HANDLE,
                child.pid,
                child.created,
                child.configuration.as_ref().unwrap().as_raw_handle() as HANDLE,
                proof.0.as_raw_handle() as HANDLE,
            )
        }
    }
    fn rejected_enrollment(
        library: &BootstrapLibrary,
        child: &PendingEducationBootstrap,
        birth: u64,
        image: &File,
    ) -> u32 {
        unsafe {
            (library.entry)(
                child.process.as_raw_handle() as HANDLE,
                child._thread.as_raw_handle() as HANDLE,
                child.pid,
                birth,
                child.configuration.as_ref().unwrap().as_raw_handle() as HANDLE,
                image.as_raw_handle() as HANDLE,
                ptr::null_mut(),
            )
        }
    }

    #[test]
    fn registered_enrollment_refuses_unreviewed_payload_before_game_lookup() {
        assert!(matches!(prepare_registered_enrolled_for::<2>(), Err(error)
            if error.kind() == io::ErrorKind::Unsupported));
    }

    #[test]
    fn five_slot_policy_rejects_unreviewed_or_out_of_range_sets() {
        assert!(
            matches!(registered_slots::<5>(), Err(error) if error.kind() == io::ErrorKind::Unsupported)
        );
        assert!(registered_slots::<0>().is_err());
        assert!(registered_slots::<6>().is_err());
        let id = FILE_ID_INFO {
            VolumeSerialNumber: 7,
            FileId: windows_sys::Win32::Storage::FileSystem::FILE_ID_128 {
                Identifier: [3; 16],
            },
        };
        let leaves: Vec<_> = (1..=5)
            .map(|slot| registry_leaf(&id, slot).unwrap())
            .collect();
        for (index, leaf) in leaves.iter().enumerate() {
            assert!(leaf.ends_with(&format!("_{}", char::from(b'A' + index as u8))));
        }
        assert_eq!(
            leaves
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            5
        );
        assert!(registry_leaf(&id, 0).is_err());
        assert!(registry_leaf(&id, 6).is_err());
    }

    #[test]
    fn registered_observation_requires_replay_policy_before_storage_lookup() {
        // Test policy is deliberately absent. Refuse before Known Folders,
        // registry access or fresh profile allocation; never involve the game.
        assert!(matches!(registered_slots::<2>(), Err(error)
            if error.kind() == io::ErrorKind::Unsupported));
        assert!(matches!(registered_replay_slots::<5>(), Err(error)
            if error.kind() == io::ErrorKind::Unsupported));
        assert_eq!(registered_replay_slots::<0>().err().unwrap().kind(), io::ErrorKind::InvalidData);
        assert_eq!(registered_replay_slots::<6>().err().unwrap().kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn bootstrap_payload_rejects_mismatched_hashes_and_pins_loaded_files() {
        let exe = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let root = exe.parent().unwrap();
        let reviewed = fixture_digests(root);
        let mut wrong = reviewed;
        wrong.enroller[0] ^= 1;
        assert!(BootstrapLibrary::load(root, wrong).is_err());
        let mut wrong = reviewed;
        wrong.compatibility[0] ^= 1;
        assert!(BootstrapLibrary::load(root, wrong).is_err());
        let library = BootstrapLibrary::load(root, reviewed).unwrap();
        for leaf in ["bootstrap-enroll.dll", "bootstrap-compat.dll"] {
            assert!(OpenOptions::new()
                .write(true)
                .open(root.join(leaf))
                .is_err());
        }
        drop(library);
    }

    #[test]
    fn startup_observation_requires_config_and_never_reads_a_live_writer() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        assert!(create_fixture_observed(&path, pin_image(&path).unwrap(), None, true, None).is_err());
        let [a, _b] = test_slots();
        let mut child =
            create_fixture_observed(&path, pin_image(&path).unwrap(), Some(a), true, None).unwrap();
        assert_eq!(
            child.observation_bytes_after_exit().unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        child.cancel().unwrap();
        assert_eq!(
            child.observation_bytes_after_exit().unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }

    fn clone_store(store: &SlotStorage) -> SlotStorage {
        store.duplicate().unwrap()
    }

    #[test]
    #[ignore="Requires the explicitly built own C++ payload; no game is launched"]
    fn file_hives_reach_cpp_before_dependency_tls_and_survive_relaunch() {
        let path=PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap()).canonicalize().unwrap();
        assert_eq!(path.file_name(),Some(OsStr::new("bootstrap-ordering.exe")));
        let root=path.parent().unwrap();let library=BootstrapLibrary::load(root,fixture_digests(root)).unwrap();
        let mut stores=test_slots();
        for store in &mut stores {store.registry=product_profiles::open_hive(&store.private,store.slot,true).unwrap();}
        let proof=FixtureReceipt::new();let mut wrong=clone_store(&stores[0]);wrong.registry=clone_store(&stores[1]).registry;
        let mut rejected=create_fixture_observed(&path,pin_image(&path).unwrap(),Some(wrong),false,Some(&proof)).unwrap();
        assert_ne!(prepare_fixture(&library,&rejected,&proof,true),0);
        assert!(library.enroll(&rejected).is_err());rejected.cancel().unwrap();
        for _round in 1..=2 {for store in &stores {
            let proof=FixtureReceipt::new();
            let child=create_fixture_observed(&path,pin_image(&path).unwrap(),Some(clone_store(store)),false,Some(&proof)).unwrap();
            assert_eq!(prepare_fixture(&library,&child,&proof,true),0);
            library.enroll(&child).unwrap();finish_fixture(&child);
        }}
    }

    fn finish_fixture(child: &PendingEducationBootstrap) {
        // Only this fixed owned fixture can be resumed in the test module.
        assert_eq!(
            unsafe {
                windows_sys::Win32::System::Threading::ResumeThread(
                    child._thread.as_raw_handle() as HANDLE
                )
            },
            1
        );
        check_finished_fixture(child);
    }

    fn check_finished_fixture(child: &PendingEducationBootstrap) {
        assert_eq!(
            unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
            WAIT_OBJECT_0
        );
        let mut code = 1;
        assert_ne!(
            unsafe { GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code) },
            0
        );
        assert_eq!(code, 0);
        // Query in the unhooked parent, never through the child's translated cached key.
        use windows_sys::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, REG_DWORD};
        let name = wide(OsStr::new(&format!(
            "Software\\EducationEntry_{}_{}",
            child.pid, child.created
        )))
        .unwrap();
        let mut host = ptr::null_mut();
        assert_eq!(
            unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, name.as_ptr(), 0, KEY_READ, &mut host) },
            0
        );
        let host = RegistryKey(host);
        let mut marker = 0u32;
        let mut kind = 0;
        let mut bytes = mem::size_of_val(&marker) as u32;
        assert_eq!(
            unsafe {
                RegQueryValueExW(
                    host.0,
                    [77u16, 97, 114, 107, 101, 114, 0].as_ptr(),
                    ptr::null(),
                    &mut kind,
                    &mut marker as *mut _ as _,
                    &mut bytes,
                )
            },
            0
        );
        assert_eq!((kind, bytes, marker), (REG_DWORD, 4, 0x484f5354));
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ fixture and test-only enroller DLL"]
    fn rust_enrolls_own_fixture_before_dependency_tls_and_main() {
        let path = PathBuf::from(
            std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER")
                .expect("Set the trusted built C++ fixture path"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let root = path.parent().unwrap();
        let enroller = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        let [a, b] = test_slots();
        for store in [&a, &b] {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
        }
        // A mismatched birth is rejected before mutation; original child remains cancellable.
        let proof = FixtureReceipt::new();
        let mut rejected = create_fixture_observed(
            &path,
            pin_image(&path).unwrap(),
            Some(clone_store(&a)),
            false,
            Some(&proof),
        )
        .unwrap();
        assert_ne!(
            rejected_enrollment(&enroller, &rejected, rejected.created + 1, &rejected._image),
            0
        );
        rejected.identity().unwrap();
        rejected.cancel().unwrap();
        // The same live handles for a different executable are rejected before any canary creation.
        let self_path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let proof = FixtureReceipt::new();
        let mut wrong_image = create_fixture_observed(
            &self_path,
            pin_image(&self_path).unwrap(),
            Some(clone_store(&a)),
            false,
            Some(&proof),
        )
        .unwrap();
        assert_ne!(
            rejected_enrollment(
                &enroller,
                &wrong_image,
                wrong_image.created,
                &pin_image(&path).unwrap()
            ),
            0
        );
        wrong_image.identity().unwrap();
        wrong_image.cancel().unwrap();
        // Positive unhooked control must observe all three host registry canaries.
        let proof = FixtureReceipt::new();
        let baseline = create_fixture_observed(
            &path,
            pin_image(&path).unwrap(),
            Some(clone_store(&a)),
            false,
            Some(&proof),
        )
        .unwrap();
        assert_eq!(prepare_fixture(&enroller, &baseline, &proof, false), 0);
        finish_fixture(&baseline);
        let mut known_callsite = 0;
        let mut saved_journals = Vec::new();
        for round in 1..=2 {
            let mut first_store = clone_store(&a);
            let mut second_store = clone_store(&b);
            let host_mutex = if round == 2 {
                first_store.singleton_caller_rva = known_callsite;
                second_store.singleton_caller_rva = known_callsite;
                assert_ne!(known_callsite, 0);
                let id = directory_id(&a.logical).unwrap().FileId.Identifier;
                let part = u16::from_le_bytes([id[0], id[1]]);
                let mut bytes = [0u8; 8];
                bytes[..6].copy_from_slice(&id[2..8]);
                let tail = u64::from_le_bytes(bytes);
                let name = wide(OsStr::new(&format!(
                    "feedface-{part:04x}-4000-8000-{tail:012x}"
                )))
                .unwrap();
                let raw = unsafe {
                    windows_sys::Win32::System::Threading::CreateMutexW(
                        ptr::null(),
                        0,
                        name.as_ptr(),
                    )
                };
                assert!(!raw.is_null());
                Some(unsafe { OwnedHandle::from_raw_handle(raw as _) })
            } else {
                None
            };
            let first_proof = FixtureReceipt::new();
            let second_proof = FixtureReceipt::new();
            let first = create_fixture_observed(
                &path,
                pin_image(&path).unwrap(),
                Some(first_store),
                true,
                Some(&first_proof),
            )
            .unwrap();
            let second = create_fixture_observed(
                &path,
                pin_image(&path).unwrap(),
                Some(second_store),
                true,
                Some(&second_proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&enroller, &first, &first_proof, true), 0);
            enroller.enroll(&first).unwrap();
            assert_eq!(prepare_fixture(&enroller, &second, &second_proof, true), 0);
            enroller.enroll(&second).unwrap();
            let transaction = PreparedEducationEnrollment {
                _children: [first, second],
                _payload: BootstrapLibrary::load(root, fixture_digests(root)).unwrap(),
            }
            .resume_for_authorized_observation()
            .unwrap();
            assert!(transaction
                .attempts
                .iter()
                .all(|a| matches!(a, StartupAttempt::Resumed)));
            assert!(transaction.save_after_exit(0).is_err());
            assert!(transaction.save_after_exit(3).is_err());
            for child in &transaction.children {
                check_finished_fixture(child);
            }
            // Each process lifetime saves a fresh journal; repeated saves of one lifetime still refuse.
            assert_eq!(legacy_startup_observation::observe_until_exit(&transaction).unwrap(), 0);
            for child in &transaction.children {
                let snapshot = child.observation_bytes_after_exit().unwrap();
                let count = u32::from_le_bytes(snapshot[48..52].try_into().unwrap()) as usize;
                let callsite = (0..count)
                    .find_map(|i| {
                        let at = 64 + i * 496;
                        let word = |offset| {
                            u32::from_le_bytes(
                                snapshot[at + offset..at + offset + 4].try_into().unwrap(),
                            )
                        };
                        (word(4) == 6 && word(480) == 1).then(|| word(484))
                    })
                    .expect("Own executable CreateMutexA callsite");
                if known_callsite == 0 {
                    known_callsite = callsite;
                }
                assert_eq!(callsite, known_callsite);
                assert_eq!(u32::from_le_bytes(snapshot[48..52].try_into().unwrap()), 64);
                assert_eq!(u32::from_le_bytes(snapshot[52..56].try_into().unwrap()), 1);
                let slot = child.storage.as_ref().unwrap().slot;
                let path = child.observation_path().unwrap();
                assert_eq!(std::fs::read(&path).unwrap(), snapshot);
                saved_journals.push((path, snapshot));
                assert_eq!(
                    transaction.save_after_exit(slot).err().unwrap().kind(),
                    io::ErrorKind::AlreadyExists
                );
                assert_eq!(
                    std::fs::metadata(child.observation_path().unwrap())
                        .unwrap()
                        .len(),
                    OBSERVATION_BYTES as u64
                );
            }
            if let Some(host) = &host_mutex {
                assert_eq!(
                    unsafe { WaitForSingleObject(host.as_raw_handle() as HANDLE, 0) },
                    WAIT_OBJECT_0
                );
                assert_ne!(
                    unsafe {
                        windows_sys::Win32::System::Threading::ReleaseMutex(
                            host.as_raw_handle() as HANDLE
                        )
                    },
                    0
                );
            }
            for store in [&a, &b] {
                assert_eq!(
                    std::fs::read_to_string(store.private_path.join("Roaming/rust-enrolled.txt"))
                        .unwrap(),
                    format!("{}:{round}", store.slot)
                );
                assert_eq!(
                    std::fs::read(
                        directory_path(&store.logical)
                            .unwrap()
                            .join("Roaming/rust-enrolled.txt")
                    )
                    .unwrap(),
                    b"HOST"
                );
            }
        }
        assert_eq!(saved_journals.len(), 4);
        for (index, (path, bytes)) in saved_journals.iter().enumerate() {
            assert!(saved_journals[..index]
                .iter()
                .all(|(previous, _)| previous != path));
            assert_eq!(&std::fs::read(path).unwrap(), bytes);
        }
    }

    // Test-only duplicates of the original owned fixture handles; bounded panic cleanup
    // can terminate only this control, never a game or a PID-opened process.
    struct FixtureLifetime {
        process: OwnedHandle,
        thread: OwnedHandle,
    }
    impl FixtureLifetime {
        fn new(child: &PendingEducationBootstrap) -> Self {
            fn copy(raw: HANDLE) -> OwnedHandle {
                let mut result = ptr::null_mut();
                assert_ne!(
                    unsafe {
                        DuplicateHandle(
                            GetCurrentProcess(),
                            raw,
                            GetCurrentProcess(),
                            &mut result,
                            0,
                            0,
                            DUPLICATE_SAME_ACCESS,
                        )
                    },
                    0
                );
                unsafe { OwnedHandle::from_raw_handle(result as _) }
            }
            Self {
                process: copy(child.process.as_raw_handle() as HANDLE),
                thread: copy(child._thread.as_raw_handle() as HANDLE),
            }
        }
        fn raw(&self) -> HANDLE {
            self.process.as_raw_handle() as HANDLE
        }
    }
    impl Drop for FixtureLifetime {
        fn drop(&mut self) {
            if unsafe { WaitForSingleObject(self.raw(), 0) } != WAIT_OBJECT_0 {
                unsafe {
                    TerminateProcess(self.raw(), 91);
                    WaitForSingleObject(self.raw(), 5000);
                }
            }
        }
    }

    fn fixture_barrier(
        child: &PendingEducationBootstrap,
        proof: &FixtureReceipt,
    ) -> (OwnedHandle, OwnedHandle) {
        use windows_sys::Win32::System::Threading::CreateEventW;
        let view = unsafe {
            MapViewOfFile(
                proof.0.as_raw_handle() as HANDLE,
                FILE_MAP_WRITE,
                0,
                0,
                2064,
            )
        };
        assert!(!view.Value.is_null());
        let events = [2032usize, 2056usize].map(|offset| {
            let raw = unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) };
            assert!(!raw.is_null());
            let own = unsafe { OwnedHandle::from_raw_handle(raw as _) };
            let mut remote = ptr::null_mut();
            assert_ne!(
                unsafe {
                    DuplicateHandle(
                        GetCurrentProcess(),
                        raw,
                        child.process.as_raw_handle() as HANDLE,
                        &mut remote,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                },
                0
            );
            unsafe {
                ptr::write((view.Value as *mut u8).add(offset) as *mut HANDLE, remote);
            }
            own
        });
        assert_ne!(unsafe { UnmapViewOfFile(view) }, 0);
        let [ready, finish] = events;
        (ready, finish)
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn owned_descendant_handoff_preserves_provenance_and_reuses_slot() {
        use owned_handoff::Request;
        use windows_sys::Win32::System::Threading::SetEvent;
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let library = BootstrapLibrary::load(
            path.parent().unwrap(),
            fixture_digests(path.parent().unwrap()),
        )
        .unwrap();
        let helper_proofs = [FixtureReceipt::new(), FixtureReceipt::new()];
        let mut parents = Vec::new();
        let mut requests = Vec::new();
        let mut held = Vec::new();
        let mut barriers = Vec::new();
        for (index, store) in test_slots().into_iter().enumerate() {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
            let proof = FixtureReceipt::new();
            let mut parent = create_fixture_observed(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                false,
                Some(&proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&library, &parent, &proof, true), 0);
            library.enroll(&parent).unwrap();
            barriers.push(fixture_barrier(&parent, &proof));
            let request = Request::new(&parent, &proof, &helper_proofs[index]).unwrap();
            held.push(FixtureLifetime::new(&parent));
            parent.resume_once().unwrap();
            request.wait(&parent).unwrap();
            parents.push(parent);
            requests.push(request);
        }
        assert!(library
            .accept_owned_descendant(&parents[0], &mut requests[0], &helper_proofs[1])
            .is_err());
        requests[0].packet().created.dwLowDateTime ^= 1;
        assert!(library
            .accept_owned_descendant(&parents[0], &mut requests[0], &helper_proofs[0])
            .is_err());
        requests[0].packet().created.dwLowDateTime ^= 1;
        // Substitute a complete live peer-descendant tuple. PID/thread/birth/image all match;
        // native ancestry must still reject it without killing the peer or either sender.
        let peer = requests[1].packet();
        let peer_tuple = (peer.process, peer.thread, peer.pid, peer.created);
        let mut imported = Vec::new();
        let mut remote = Vec::new();
        for original in [peer_tuple.0, peer_tuple.1] {
            let mut local = ptr::null_mut();
            assert_ne!(
                unsafe {
                    DuplicateHandle(
                        parents[1].process.as_raw_handle() as HANDLE,
                        original,
                        GetCurrentProcess(),
                        &mut local,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                },
                0
            );
            imported.push(unsafe { OwnedHandle::from_raw_handle(local as _) });
            let mut in_sender = ptr::null_mut();
            assert_ne!(
                unsafe {
                    DuplicateHandle(
                        GetCurrentProcess(),
                        local,
                        parents[0].process.as_raw_handle() as HANDLE,
                        &mut in_sender,
                        0,
                        0,
                        DUPLICATE_SAME_ACCESS,
                    )
                },
                0
            );
            remote.push(in_sender);
        }
        let first = requests[0].packet();
        let original = (first.process, first.thread, first.pid, first.created);
        first.process = remote[0];
        first.thread = remote[1];
        first.pid = peer_tuple.2;
        first.created = peer_tuple.3;
        assert!(library
            .accept_owned_descendant(&parents[0], &mut requests[0], &helper_proofs[0])
            .is_err());
        assert!(parents.iter().all(|parent| parent.identity().is_ok()));
        assert_eq!(
            unsafe { WaitForSingleObject(imported[0].as_raw_handle() as HANDLE, 0) },
            WAIT_TIMEOUT
        );
        let first = requests[0].packet();
        first.process = original.0;
        first.thread = original.1;
        first.pid = original.2;
        first.created = original.3;
        for index in 0..2 {
            let mut helper = library
                .accept_owned_descendant(
                    &parents[index],
                    &mut requests[index],
                    &helper_proofs[index],
                )
                .unwrap();
            let lifetime = FixtureLifetime::new(&helper.child);
            assert!(library
                .accept_owned_descendant(
                    &parents[index],
                    &mut requests[index],
                    &helper_proofs[index]
                )
                .is_err());
            assert_eq!(
                unsafe { WaitForSingleObject(barriers[index].0.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            assert_eq!(
                helper.child.storage.as_ref().unwrap().slot,
                index as u32 + 1
            );
            helper.resume().unwrap();
            check_finished_fixture(&helper.child);
            assert!(parents.iter().all(|parent| parent.identity().is_ok()));
            assert_eq!(
                std::fs::read_to_string(
                    parents[index]
                        .storage
                        .as_ref()
                        .unwrap()
                        .private_path
                        .join("Roaming/rust-enrolled.txt")
                )
                .unwrap(),
                format!("{}:2", index + 1)
            );
            drop(helper);
            drop(lifetime);
        }
        for (parent, barrier) in parents.iter().zip(&barriers) {
            assert_ne!(unsafe { SetEvent(barrier.1.as_raw_handle() as HANDLE) }, 0);
            check_finished_fixture(parent);
        }
        drop(held);
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn owned_descendant_refusal_before_and_after_publication_preserves_data() {
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let library = BootstrapLibrary::load(
            path.parent().unwrap(),
            fixture_digests(path.parent().unwrap()),
        )
        .unwrap();
        for (index, store) in test_slots().into_iter().enumerate() {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
            let private = store.private_path.clone();
            let proof = FixtureReceipt::new();
            let helper_proof = FixtureReceipt::new();
            let mut parent = create_fixture_observed(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                false,
                Some(&proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&library, &parent, &proof, true), 0);
            library.enroll(&parent).unwrap();
            let mut request = owned_handoff::Request::new(&parent, &proof, &helper_proof).unwrap();
            let lifetime = FixtureLifetime::new(&parent);
            if index == 0 {
                drop(request); // Sender sees refusal before it may create a descendant.
                parent.resume_once().unwrap();
            } else {
                parent.resume_once().unwrap();
                request.wait(&parent).unwrap();
                let mut raw = ptr::null_mut();
                assert_ne!(
                    unsafe {
                        DuplicateHandle(
                            parent.process.as_raw_handle() as HANDLE,
                            request.packet().process,
                            GetCurrentProcess(),
                            &mut raw,
                            0,
                            0,
                            DUPLICATE_SAME_ACCESS,
                        )
                    },
                    0
                );
                let descendant = unsafe { OwnedHandle::from_raw_handle(raw as _) };
                drop(request); // Sender still owns and cancels this original suspended descendant.
                assert_eq!(unsafe { WaitForSingleObject(raw, 10000) }, WAIT_OBJECT_0);
                let mut code = 0;
                assert_ne!(unsafe { GetExitCodeProcess(raw, &mut code) }, 0);
                assert_eq!(code, 90);
                drop(descendant);
            }
            assert_eq!(
                unsafe { WaitForSingleObject(parent.process.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            assert_eq!(parent.exit_code_after_exit().unwrap(), 86); // Fixed fixture reports a refused handoff.
            assert_eq!(
                std::fs::read_to_string(private.join("Roaming/rust-enrolled.txt")).unwrap(),
                format!("{}:1", index + 1)
            );
            assert_eq!(
                std::fs::read(
                    directory_path(&parent.storage.as_ref().unwrap().logical)
                        .unwrap()
                        .join("Roaming/rust-enrolled.txt")
                )
                .unwrap(),
                b"HOST"
            );
            drop(lifetime);
        }
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn manager_owned_helper_transaction_keeps_slot_and_rejects_changed_bindings() {
        use windows_sys::Win32::System::Threading::SetEvent;
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let root = path.parent().unwrap();
        let library = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        let stores = test_slots();
        let mut parents = Vec::new();
        let mut held = Vec::new();
        let mut barriers = Vec::new();
        for store in stores {
            std::fs::write(
                directory_path(&store.logical)
                    .unwrap()
                    .join("Roaming/rust-enrolled.txt"),
                b"HOST",
            )
            .unwrap();
            let proof = FixtureReceipt::new();
            let mut parent = create_fixture_observed(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                false,
                Some(&proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&library, &parent, &proof, true), 0);
            library.enroll(&parent).unwrap();
            assert!(library
                .prepare_owned_helper(&parent, &FixtureReceipt::new())
                .is_err());
            let barrier = fixture_barrier(&parent, &proof);
            held.push(FixtureLifetime::new(&parent));
            parent.resume_once().unwrap();
            assert_eq!(
                unsafe { WaitForSingleObject(barrier.0.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            parents.push(parent);
            barriers.push(barrier);
        }
        for index in 0..2 {
            parents[index].created ^= 1;
            assert!(library
                .prepare_owned_helper(&parents[index], &FixtureReceipt::new())
                .is_err());
            parents[index].created ^= 1;
            let original_image = mem::replace(
                &mut parents[index]._image,
                pin_image(&std::env::current_exe().unwrap()).unwrap(),
            );
            assert!(library
                .prepare_owned_helper(&parents[index], &FixtureReceipt::new())
                .is_err());
            parents[index]._image = original_image;
            let mut foreign = parents[1 - index]
                .storage
                .as_ref()
                .unwrap()
                .duplicate()
                .unwrap();
            mem::swap(
                &mut parents[index].storage.as_mut().unwrap().registry,
                &mut foreign.registry,
            );
            assert!(library
                .prepare_owned_helper(&parents[index], &FixtureReceipt::new())
                .is_err());
            mem::swap(
                &mut parents[index].storage.as_mut().unwrap().registry,
                &mut foreign.registry,
            );
            let original_path = mem::replace(
                &mut parents[index].storage.as_mut().unwrap().private_path,
                foreign.private_path.clone(),
            );
            assert!(library
                .prepare_owned_helper(&parents[index], &FixtureReceipt::new())
                .is_err());
            parents[index].storage.as_mut().unwrap().private_path = original_path;
            let abandoned_proof = FixtureReceipt::new();
            let abandoned = library
                .prepare_owned_helper(&parents[index], &abandoned_proof)
                .unwrap();
            let abandoned_lifetime = FixtureLifetime::new(&abandoned.child);
            drop(abandoned);
            assert_eq!(
                unsafe { WaitForSingleObject(abandoned_lifetime.raw(), 10000) },
                WAIT_OBJECT_0
            );
            let mut abandoned_code = 0;
            assert_ne!(
                unsafe { GetExitCodeProcess(abandoned_lifetime.raw(), &mut abandoned_code) },
                0
            );
            assert_eq!(abandoned_code, 90);
            assert!(parents.iter().all(|parent| parent.identity().is_ok()));
            let proof = FixtureReceipt::new();
            let mut helper = library
                .prepare_owned_helper(&parents[index], &proof)
                .unwrap();
            let helper_lifetime = FixtureLifetime::new(&helper.child);
            let config = read_config(&helper.child);
            assert_ne!(
                helper.child.identity().unwrap(),
                parents[index].identity().unwrap()
            );
            assert_eq!(
                (config.slot, config.scope, config.singleton_caller_rva),
                (index as u32 + 1, 1, 0)
            );
            assert!(helper.child.observation.is_none());
            helper.child.created ^= 1;
            assert!(helper.resume().is_err());
            assert!(!helper.child.resume_attempted);
            helper.child.created ^= 1;
            let slot = helper.child.storage.as_ref().unwrap().slot;
            helper.child.storage.as_mut().unwrap().slot = 3 - slot;
            assert!(helper.resume().is_err());
            assert!(!helper.child.resume_attempted);
            helper.child.storage.as_mut().unwrap().slot = slot;
            helper.resume().unwrap();
            check_finished_fixture(&helper.child);
            assert!(helper.resume().is_err());
            assert!(parents.iter().all(|parent| parent.identity().is_ok()));
            let store = parents[index].storage.as_ref().unwrap();
            assert_eq!(
                std::fs::read_to_string(store.private_path.join("Roaming/rust-enrolled.txt"))
                    .unwrap(),
                format!("{}:2", store.slot)
            );
            assert_eq!(
                std::fs::read(
                    directory_path(&store.logical)
                        .unwrap()
                        .join("Roaming/rust-enrolled.txt")
                )
                .unwrap(),
                b"HOST"
            );
            drop(helper);
            drop(helper_lifetime);
        }
        for (parent, barrier) in parents.iter().zip(&barriers) {
            let proof = FixtureReceipt::new();
            let mut helper = library.prepare_owned_helper(parent, &proof).unwrap();
            let lifetime = FixtureLifetime::new(&helper.child);
            assert_ne!(unsafe { SetEvent(barrier.1.as_raw_handle() as HANDLE) }, 0);
            check_finished_fixture(parent);
            assert!(helper.resume().is_err());
            assert!(!helper.child.resume_attempted);
            drop(helper);
            assert_eq!(
                unsafe { WaitForSingleObject(lifetime.raw(), 10000) },
                WAIT_OBJECT_0
            );
            let mut code = 0;
            assert_ne!(unsafe { GetExitCodeProcess(lifetime.raw(), &mut code) }, 0);
            assert_eq!(code, 90);
        }
        drop(held);
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn observation_transaction_preserves_attempted_children_on_partial_resume_and_drop() {
        use windows_sys::Win32::System::Threading::{ResumeThread, SetEvent, SuspendThread};
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let root = path.parent().unwrap();
        let library = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        let [a, b] = test_slots();
        let proofs = [FixtureReceipt::new(), FixtureReceipt::new()];
        let children = [a, b]
            .into_iter()
            .zip(&proofs)
            .map(|(store, proof)| {
                std::fs::write(
                    directory_path(&store.logical)
                        .unwrap()
                        .join("Roaming/rust-enrolled.txt"),
                    b"HOST",
                )
                .unwrap();
                let child = create_fixture_observed(
                    &path,
                    pin_image(&path).unwrap(),
                    Some(store),
                    true,
                    Some(proof),
                )
                .unwrap();
                assert_eq!(prepare_fixture(&library, &child, proof, true), 0);
                library.enroll(&child).unwrap();
                child
            })
            .collect::<Vec<_>>();
        let children: [PendingEducationBootstrap; 2] = children.try_into().ok().unwrap();
        let held = children.each_ref().map(FixtureLifetime::new);
        let barriers = [
            fixture_barrier(&children[0], &proofs[0]),
            fixture_barrier(&children[1], &proofs[1]),
        ];
        // Force the second call to report "still suspended", after the first has started.
        assert_eq!(
            unsafe { SuspendThread(children[1]._thread.as_raw_handle() as HANDLE) },
            1
        );
        let mut transaction = PreparedEducationEnrollment {
            _children: children,
            _payload: library,
        }
        .resume_for_authorized_observation()
        .unwrap();
        assert!(matches!(transaction.attempts[0], StartupAttempt::Resumed));
        assert!(matches!(
            transaction.attempts[1],
            StartupAttempt::Unconfirmed(_)
        ));
        assert_eq!(
            unsafe { WaitForSingleObject(barriers[0].0.as_raw_handle() as HANDLE, 10000) },
            WAIT_OBJECT_0
        );
        assert_eq!(
            transaction.save_after_exit(1).err().unwrap().kind(),
            io::ErrorKind::WouldBlock
        );
        for child in &mut transaction.children {
            assert!(child.cancel().is_err());
            assert!(child.resume_once().is_err());
        }
        drop(transaction);
        for lifetime in &held {
            assert_eq!(
                unsafe { WaitForSingleObject(lifetime.raw(), 0) },
                WAIT_TIMEOUT
            );
        }
        // Explicit recovery/normal finish only for our controlled executable.
        assert_eq!(
            unsafe { ResumeThread(held[1].thread.as_raw_handle() as HANDLE) },
            1
        );
        assert_eq!(
            unsafe { WaitForSingleObject(barriers[1].0.as_raw_handle() as HANDLE, 10000) },
            WAIT_OBJECT_0
        );
        for (_, finish) in &barriers {
            assert_ne!(unsafe { SetEvent(finish.as_raw_handle() as HANDLE) }, 0);
        }
        for lifetime in &held {
            assert_eq!(
                unsafe { WaitForSingleObject(lifetime.raw(), 10000) },
                WAIT_OBJECT_0
            );
            let mut exit = 1;
            assert_ne!(unsafe { GetExitCodeProcess(lifetime.raw(), &mut exit) }, 0);
            assert_eq!(exit, 0);
        }
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn transaction_preflight_and_not_attempted_children_keep_unstarted_cleanup() {
        use windows_sys::Win32::System::Threading::SuspendThread;
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let root = path.parent().unwrap();
        for preflight_failure in [true, false] {
            // No execution or enrollment in these ownership failure branches.
            let mut children =
                [0, 1].map(|_| create_fixture_unstarted(&path, pin_image(&path).unwrap(), None).unwrap());
            let held = children.each_ref().map(FixtureLifetime::new);
            if preflight_failure {
                children[1].cancel().unwrap();
            } else {
                assert_eq!(
                    unsafe { SuspendThread(children[0]._thread.as_raw_handle() as HANDLE) },
                    1
                );
            }
            let prepared = PreparedEducationEnrollment {
                _children: children,
                _payload: BootstrapLibrary::load(root, fixture_digests(root)).unwrap(),
            };
            let started = prepared.resume_for_authorized_observation();
            if preflight_failure {
                assert!(started.is_err());
            } else {
                let started = started.unwrap();
                assert!(matches!(
                    started.attempts[0],
                    StartupAttempt::Unconfirmed(_)
                ));
                assert!(matches!(started.attempts[1], StartupAttempt::NotAttempted));
                drop(started);
            }
            assert_eq!(
                unsafe { WaitForSingleObject(held[1].raw(), 5000) },
                WAIT_OBJECT_0
            );
            assert_eq!(
                unsafe { WaitForSingleObject(held[0].raw(), 0) },
                if preflight_failure {
                    WAIT_OBJECT_0
                } else {
                    WAIT_TIMEOUT
                }
            );
        }
    }

    #[test]
    fn tracked_startup_restores_retry_only_before_creation_or_after_confirmed_exit() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let before_creation = UnstartedCleanup::new();
        assert!(create_fixture_tracked(Path::new("relative.exe"), pin_image(&path).unwrap(),
            None, false, None, Some(before_creation.clone()), None).is_err());
        assert!(before_creation.retry_safe());

        let missing = UnstartedCleanup::new();
        let absent = path.parent().unwrap().join(format!("absent-startup-{}.exe", std::process::id()));
        assert!(!absent.exists());
        assert!(create_fixture_tracked(&absent, pin_image(&path).unwrap(),
            None, false, None, Some(missing.clone()), None).is_err());
        assert!(missing.retry_safe());

        let cleanup = UnstartedCleanup::new();
        let child = create_fixture_tracked(&path, pin_image(&path).unwrap(),
            None, false, None, Some(cleanup.clone()), None).unwrap();
        let process = child.process.try_clone().unwrap();
        assert!(!cleanup.retry_safe());
        drop(child); // Own test executable remains suspended throughout.
        assert!(cleanup.retry_safe());
        assert_eq!(unsafe { WaitForSingleObject(process.as_raw_handle() as _, 0) }, WAIT_OBJECT_0);
    }

    #[test]
    fn unconfirmed_cancellation_and_resume_boundary_never_restore_retry() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        for attempted in [false, true] {
            let cleanup = UnstartedCleanup::new();
            let mut child = create_fixture_tracked(&path, pin_image(&path).unwrap(),
                None, false, None, Some(cleanup.clone()), None).unwrap();
            let full = child.process.try_clone().unwrap();
            if attempted {
                // Simulate only the irreversible boundary; never resume this fixture.
                child.resume_attempted = true;
            } else {
                let mut limited = ptr::null_mut();
                assert_ne!(unsafe { DuplicateHandle(GetCurrentProcess(), child.process.as_raw_handle() as _,
                    GetCurrentProcess(), &mut limited,
                    windows_sys::Win32::System::Threading::SYNCHRONIZATION_SYNCHRONIZE,
                    0, 0) }, 0);
                child.process = unsafe { OwnedHandle::from_raw_handle(limited as _) };
            }
            drop(child);
            assert!(!cleanup.retry_safe());
            assert_eq!(unsafe { WaitForSingleObject(full.as_raw_handle() as _, 0) }, WAIT_TIMEOUT);
            // Only the retained full-access handle to the own, never-resumed fixture.
            assert_ne!(unsafe { TerminateProcess(full.as_raw_handle() as _, 91) }, 0);
            assert_eq!(unsafe { WaitForSingleObject(full.as_raw_handle() as _, 5000) }, WAIT_OBJECT_0);
            assert!(!cleanup.retry_safe()); // External teardown is not a receipt.
        }
    }

    #[test]
    fn duplicated_storage_pins_keep_the_same_profile_after_failed_creation() {
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let logical_path = parent.join("logical");
        std::fs::create_dir(&logical_path).unwrap();
        for name in ["Roaming", "Local"] { std::fs::create_dir(logical_path.join(name)).unwrap(); }
        let logical = open_directory(&logical_path, FILE_READ_ATTRIBUTES).unwrap();
        let store = create_private_slot(&logical, &parent, 1).unwrap();
        let leaf = registry_leaf(&directory_id(&store.private).unwrap(), 1).unwrap();
        let sentinel = directory_path(&store.private).unwrap().join("Roaming/world-fixture.bin");
        std::fs::write(&sentinel, b"preserve").unwrap();
        let duplicate = store.duplicate().unwrap();
        assert!(same_image(&directory_id(&store.private).unwrap(), &directory_id(&duplicate.private).unwrap()));
        assert!(same_image(&directory_id(&store.logical).unwrap(), &directory_id(&duplicate.logical).unwrap()));
        assert_eq!(store.private_path, duplicate.private_path);
        assert_eq!(store.user_key_path, duplicate.user_key_path);
        for index in 0..2 {
            assert!(same_image(&directory_id(&store.temporary[index]).unwrap(), &directory_id(&duplicate.temporary[index]).unwrap()));
        }
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let cleanup = UnstartedCleanup::new();
        assert!(create_fixture_tracked(Path::new("relative.exe"), pin_image(&path).unwrap(),
            Some(duplicate), false, None, Some(cleanup.clone()), None).is_err());
        assert!(cleanup.retry_safe());
        let retry = store.duplicate().unwrap();
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"preserve");
        assert!(same_image(&directory_id(&store.private).unwrap(), &directory_id(&retry.private).unwrap()));
        drop(retry); drop(store); drop(logical);
        let key = wide(OsStr::new(&format!("Software\\MivubiEducationPilot\\{leaf}"))).unwrap();
        assert_eq!(unsafe { windows_sys::Win32::System::Registry::RegDeleteTreeW(HKEY_CURRENT_USER, key.as_ptr()) }, 0);
        assert_eq!(parent.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::remove_dir_all(parent).unwrap();
    }

    fn test_slots() -> [SlotStorage; 2] {
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let logical = parent.join("host-view");
        std::fs::create_dir(&logical).unwrap();
        for leaf in ["Roaming", "Local"] {
            std::fs::create_dir(logical.join(leaf)).unwrap();
        }
        fresh_slots(
            open_directory(&logical, FILE_READ_ATTRIBUTES).unwrap(),
            &parent,
        )
        .unwrap()
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload"]
    fn observation_reports_nonzero_child_exits_even_when_journals_save() {
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        let root = path.parent().unwrap();
        let enroller = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        let children = test_slots().map(|store| {
            // Force an existing receiver validation error after successful attach, not a game run.
            let name = wide(OsStr::new("RustEnrolledRound")).unwrap();
            let value = 2u32;
            assert_eq!(
                unsafe {
                    windows_sys::Win32::System::Registry::RegSetValueExW(
                        store.registry.0,
                        name.as_ptr(),
                        0,
                        windows_sys::Win32::System::Registry::REG_DWORD,
                        &value as *const u32 as _,
                        4,
                    )
                },
                0
            );
            let proof = FixtureReceipt::new();
            let child = create_fixture_observed(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                true,
                Some(&proof),
            )
            .unwrap();
            assert_eq!(prepare_fixture(&enroller, &child, &proof, true), 0);
            enroller.enroll(&child).unwrap();
            child
        });
        let transaction = PreparedEducationEnrollment {
            _children: children,
            _payload: enroller,
        }
        .resume_for_authorized_observation()
        .unwrap();
        for child in &transaction.children {
            assert_eq!(
                unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            assert_eq!(child.exit_code_after_exit().unwrap(), 86);
        }
        assert_eq!(legacy_startup_observation::observe_until_exit(&transaction).unwrap(), 1);
        for child in &transaction.children {
            assert_eq!(
                std::fs::metadata(child.observation_path().unwrap())
                    .unwrap()
                    .len(),
                OBSERVATION_BYTES as u64
            );
        }
    }

    #[test]
    fn five_slot_replay_preserves_pair_and_rejects_replaced_extra_slot() {
        use sha2::{Digest, Sha256};
        let pair = test_slots();
        let root = pair[0].private_path.parent().unwrap().to_path_buf();
        let parent = root.parent().unwrap();
        let mut pair_bytes = [0u8; 216];
        for (at, value) in [(0, 0x52504c31u32), (4, 1), (8, 216)] {
            pair_bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        for (i, unit) in root.file_name().unwrap().encode_wide().enumerate() {
            pair_bytes[16 + i * 2..18 + i * 2].copy_from_slice(&unit.to_le_bytes());
        }
        let root_file = open_directory(&root, FILE_READ_ATTRIBUTES).unwrap();
        for (i, file) in [&root_file, &pair[0].private, &pair[1].private]
            .iter()
            .enumerate()
        {
            let id = directory_id(file).unwrap();
            let at = 144 + i * 24;
            pair_bytes[at..at + 8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
            pair_bytes[at + 8..at + 24].copy_from_slice(&id.FileId.Identifier);
        }
        let original_paths = pair.each_ref().map(|store| store.private_path.clone());
        let mut stores = Vec::from(pair);
        for slot in 3..=5 {
            stores.push(create_private_slot(&stores[0].logical, &root, slot).unwrap());
        }
        for store in &stores {
            std::fs::write(
                store.private_path.join("Roaming/five-slot-sentinel.txt"),
                format!("slot-{}", store.slot),
            )
            .unwrap();
        }
        let bytes = five_replay_bytes(&pair_bytes, &stores).unwrap();
        assert_eq!(&bytes[16..216], &pair_bytes[16..216]);
        let reopened = reopen_slots_for::<5>(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &bytes,
            Sha256::digest(&bytes).into(),
        )
        .unwrap();
        for (index, store) in reopened.iter().enumerate() {
            assert_eq!(store.slot, index as u32 + 1);
            assert_eq!(
                std::fs::read(store.private_path.join("Roaming/five-slot-sentinel.txt")).unwrap(),
                format!("slot-{}", index + 1).as_bytes()
            );
            if index < 2 {
                assert_eq!(store.private_path, original_paths[index]);
            }
        }
        assert!(create_private_slot(&stores[0].logical, &root, 3).is_err());
        let legacy = reopen_slots(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &pair_bytes,
            Sha256::digest(pair_bytes).into(),
        )
        .unwrap();
        assert_eq!(legacy[0].private_path, original_paths[0]);
        assert_eq!(legacy[1].private_path, original_paths[1]);
        let mut replaced = bytes.clone();
        replaced[216] ^= 1;
        assert!(reopen_slots_for::<5>(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &replaced,
            Sha256::digest(&replaced).into()
        )
        .is_err());
        assert!(reopen_slots_for::<5>(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &pair_bytes,
            Sha256::digest(pair_bytes).into()
        )
        .is_err());
    }

    #[test]
    fn replay_reopens_only_the_reviewed_existing_pair() {
        use sha2::{Digest, Sha256};
        let stores = test_slots();
        let root = stores[0].private_path.parent().unwrap().to_path_buf();
        let parent = root.parent().unwrap();
        let mut bytes = [0u8; 216];
        for (at, value) in [(0, 0x52504c31u32), (4, 1), (8, 216)] {
            bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        for (i, unit) in root.file_name().unwrap().encode_wide().enumerate() {
            bytes[16 + i * 2..18 + i * 2].copy_from_slice(&unit.to_le_bytes());
        }
        let root_file = open_directory(&root, FILE_READ_ATTRIBUTES).unwrap();
        for (i, file) in [&root_file, &stores[0].private, &stores[1].private]
            .iter()
            .enumerate()
        {
            let id = directory_id(file).unwrap();
            let at = 144 + i * 24;
            bytes[at..at + 8].copy_from_slice(&id.VolumeSerialNumber.to_le_bytes());
            bytes[at + 8..at + 24].copy_from_slice(&id.FileId.Identifier);
        }
        let expected: [u8; 32] = Sha256::digest(bytes).into();
        for store in &stores {
            std::fs::write(
                store.private_path.join("Local/Temp/replay-temp-canary.bin"),
                format!("slot-{}", store.slot),
            )
            .unwrap();
        }
        assert!(reopen_slots(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &bytes,
            [0; 32]
        )
        .is_err());
        let reopened = reopen_slots(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &bytes,
            expected,
        )
        .unwrap();
        for i in 0..2 {
            assert!(same_image(
                &directory_id(&stores[i].private).unwrap(),
                &directory_id(&reopened[i].private).unwrap()
            ));
            assert_eq!(stores[i].private_path, reopened[i].private_path);
            assert!(same_image(
                &directory_id(&stores[i].temporary[1]).unwrap(),
                &directory_id(&reopened[i].temporary[1]).unwrap()
            ));
            assert_eq!(
                std::fs::read(
                    reopened[i]
                        .private_path
                        .join("Local/Temp/replay-temp-canary.bin")
                )
                .unwrap(),
                format!("slot-{}", i + 1).as_bytes()
            );
        }
        let mut replaced = bytes;
        replaced[168] ^= 1;
        let expected = Sha256::digest(replaced).into();
        assert!(reopen_slots(
            stores[0].logical.try_clone().unwrap(),
            parent,
            &replaced,
            expected
        )
        .is_err());
        // A missing directory must fail, not recreate an empty profile.
        drop(reopened);
        let local = stores[0].private_path.join("Local");
        let logical = stores[0].logical.try_clone().unwrap();
        std::fs::remove_file(local.join("Temp/replay-temp-canary.bin")).unwrap();
        assert!(std::fs::remove_dir(local.join("Temp")).is_err()); // Live storage pins protect it.
        drop(stores);
        std::fs::remove_dir(local.join("Temp")).unwrap();
        std::fs::remove_dir(&local).unwrap();
        assert!(reopen_slots(logical, parent, &bytes, Sha256::digest(bytes).into()).is_err());
        assert!(!local.exists());
    }

    fn read_config(child: &PendingEducationBootstrap) -> EducationConfig {
        let section = child.configuration.as_ref().unwrap().as_raw_handle() as HANDLE;
        let writable = unsafe {
            MapViewOfFile(
                section,
                FILE_MAP_WRITE,
                0,
                0,
                mem::size_of::<EducationConfig>(),
            )
        };
        assert!(writable.Value.is_null());
        let view = unsafe {
            MapViewOfFile(
                section,
                FILE_MAP_READ,
                0,
                0,
                mem::size_of::<EducationConfig>(),
            )
        };
        assert!(!view.Value.is_null());
        let config = unsafe { ptr::read(view.Value as *const EducationConfig) };
        assert_ne!(unsafe { UnmapViewOfFile(view) }, 0);
        config
    }

    #[test]
    fn configured_slots_publish_exact_child_bindings_and_exclude_an_ambient_handle() {
        use windows_sys::Win32::{
            Foundation::CompareObjectHandles, Security::SECURITY_ATTRIBUTES,
            System::Threading::CreateEventW,
        };
        let attributes = SECURITY_ATTRIBUTES {
            nLength: mem::size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: ptr::null_mut(),
            bInheritHandle: 1,
        };
        let ambient = unsafe { CreateEventW(&attributes, 1, 0, ptr::null()) };
        assert!(!ambient.is_null());
        let ambient = unsafe { OwnedHandle::from_raw_handle(ambient as _) };
        let [a, b] = test_slots();
        assert!(!same_image(
            &directory_id(&a.private).unwrap(),
            &directory_id(&b.private).unwrap()
        ));
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let mut first = create_fixture_unstarted(&path, pin_image(&path).unwrap(), Some(a)).unwrap();
        let mut second = create_fixture_unstarted(&path, pin_image(&path).unwrap(), Some(b)).unwrap();
        for (slot, child) in [(1, &first), (2, &second)] {
            let c = read_config(child);
            assert_eq!(
                (
                    c.pid,
                    ((c.created.dwHighDateTime as u64) << 32) | c.created.dwLowDateTime as u64
                ),
                child.identity().unwrap()
            );
            assert_eq!(c.slot, slot);
            let storage = child.storage.as_ref().unwrap();
            assert!(same_image(
                &c.private_id,
                &directory_id(&storage.private).unwrap()
            ));
            // An inherited handle keeps its value. A coincidental reused child value
            // is accepted only if it names a different kernel object.
            let mut copy = ptr::null_mut();
            if unsafe {
                DuplicateHandle(
                    child.process.as_raw_handle() as HANDLE,
                    ambient.as_raw_handle() as HANDLE,
                    GetCurrentProcess(),
                    &mut copy,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            } != 0
            {
                let copy = unsafe { OwnedHandle::from_raw_handle(copy as _) };
                assert_eq!(
                    unsafe {
                        CompareObjectHandles(
                            ambient.as_raw_handle() as HANDLE,
                            copy.as_raw_handle() as HANDLE,
                        )
                    },
                    0
                );
            }
        }
        first.cancel().unwrap();
        second.cancel().unwrap();
    }

    #[test]
    fn nested_backing_is_refused_without_reusing_or_deleting_the_logical_view() {
        let parent = fresh_root(&std::env::temp_dir()).unwrap();
        let marker = parent.join("host-marker");
        std::fs::write(&marker, b"HOST").unwrap();
        let logical = open_directory(&parent, FILE_READ_ATTRIBUTES).unwrap();
        assert!(fresh_slots(logical, &parent).is_err());
        assert_eq!(std::fs::read(marker).unwrap(), b"HOST");
    }

    #[test]
    #[ignore = "Requires the explicitly built bootstrap-ordering.exe C++ reader"]
    fn cpp_receiver_consumes_rust_configuration_for_both_slots_across_relaunch() {
        use windows_sys::Win32::System::Threading::ResumeThread;
        let path = PathBuf::from(
            std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER")
                .expect("Set the trusted built C++ fixture path"),
        )
        .canonicalize()
        .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let [a, b] = test_slots();
        for round in 1..=2 {
            let first =
                create_fixture_unstarted(&path, pin_image(&path).unwrap(), Some(clone_store(&a))).unwrap();
            let second =
                create_fixture_unstarted(&path, pin_image(&path).unwrap(), Some(clone_store(&b))).unwrap();
            // Test-only resume: fixed trusted C++ fixture, no game target or injection.
            for child in [&first, &second] {
                assert_eq!(
                    unsafe { ResumeThread(child._thread.as_raw_handle() as HANDLE) },
                    1
                );
            }
            for child in [&first, &second] {
                assert_eq!(
                    unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
                    WAIT_OBJECT_0
                );
                let mut code = 1;
                assert_ne!(
                    unsafe {
                        GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code)
                    },
                    0
                );
                assert_eq!(code, 0);
            }
            for store in [&a, &b] {
                assert_eq!(
                    std::fs::read_to_string(store.private_path.join("Roaming/rust-bridge.txt"))
                        .unwrap(),
                    format!("{}:{round}", store.slot)
                );
            }
        }
    }

    #[test]
    #[ignore = "Requires the explicitly built own C++ payload; never activates windows"]
    fn focus_lease_reaches_cpp_creation_and_drop_stops_without_killing_children() {
        use windows_sys::Win32::System::Threading::SetEvent;
        let path = PathBuf::from(std::env::var_os("MIVUBI_BOOTSTRAP_CPP_READER").unwrap())
            .canonicalize()
            .unwrap();
        assert_eq!(path.file_name(), Some(OsStr::new("bootstrap-ordering.exe")));
        let root = path.parent().unwrap();
        let library = BootstrapLibrary::load(root, fixture_digests(root)).unwrap();
        // Without a reviewed replay, registered preparation refuses before
        // looking up or allocating storage (the former fresh mode is gone).
        assert!(matches!(registered_replay_slots::<2>(), Err(error)
            if error.kind() == io::ErrorKind::Unsupported));
        // A policy request without storage refuses before process creation.
        assert!(create_fixture_configured(
            &path,
            pin_image(&path).unwrap(),
            None,
            false,
            Some(&library),
            None
        )
        .is_err());
        let proofs = [FixtureReceipt::new(), FixtureReceipt::new()];
        let mut children = Vec::new();
        let mut barriers = Vec::new();
        let mut lifetimes = Vec::new();
        for (store, proof) in test_slots().into_iter().zip(&proofs) {
            let mut child = create_fixture_configured(
                &path,
                pin_image(&path).unwrap(),
                Some(store),
                false,
                Some(&library),
                Some(proof),
            )
            .unwrap();
            assert!(child.policy.is_some());
            assert_eq!(prepare_fixture(&library, &child, proof, true), 0);
            barriers.push(fixture_barrier(&child, proof));
            lifetimes.push(FixtureLifetime::new(&child));
            library.enroll(&child).unwrap();
            child.resume_once().unwrap();
            children.push(child);
        }
        // Both hidden renderers must complete actual child-side cap validation,
        // wrapping and background pacing before the producer lease is released.
        for (child, (ready, _)) in children.iter().zip(&barriers) {
            let waits = [
                ready.as_raw_handle() as HANDLE,
                child.process.as_raw_handle() as HANDLE,
            ];
            assert_eq!(
                unsafe {
                    windows_sys::Win32::System::Threading::WaitForMultipleObjects(
                        2,
                        waits.as_ptr(),
                        0,
                        10000,
                    )
                },
                WAIT_OBJECT_0
            );
        }
        for child in &mut children {
            let lease = child.policy.take().unwrap();
            let readers = lease.reader_handles().unwrap();
            drop(lease);
            assert_eq!(
                unsafe { WaitForSingleObject(readers[2].as_raw_handle() as HANDLE, 0) },
                WAIT_OBJECT_0
            );
            assert!(child.identity().is_ok());
        }
        for (_, finish) in &barriers {
            assert_ne!(unsafe { SetEvent(finish.as_raw_handle() as HANDLE) }, 0);
        }
        for child in &children {
            assert_eq!(
                unsafe { WaitForSingleObject(child.process.as_raw_handle() as HANDLE, 10000) },
                WAIT_OBJECT_0
            );
            assert_eq!(child.exit_code_after_exit().unwrap(), 0);
        }
        drop(children);
        drop(lifetimes);
        // Last private lease stops the sole worker. Join before other owner tests.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match instance_focus_events::stop_observer() {
                Ok(report) => {
                    assert!(!report.enabled);
                    break;
                }
                Err(error)
                    if error.kind() == io::ErrorKind::WouldBlock
                        && std::time::Instant::now() < deadline =>
                {
                    std::thread::sleep(std::time::Duration::from_millis(5))
                }
                other => panic!("Focus worker cleanup failed: {other:?}"),
            }
        }
    }

    #[test]
    fn original_child_is_pinned_unstarted_and_cancellable() {
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let image = pin_image(&path).unwrap();
        let id = image_id(&image).unwrap();
        let mut child = create_fixture_unstarted(&path, image, None).unwrap();
        let identity = child.identity().unwrap();
        assert_ne!(identity.0, unsafe { GetCurrentProcessId() });
        assert!(identity.1 > 0);
        assert!(same_image(&id, &image_id(&child._image).unwrap()));
        child.cancel().unwrap();
        assert!(child.identity().is_err());
        let mut code = 0;
        assert_ne!(
            unsafe { GetExitCodeProcess(child.process.as_raw_handle() as HANDLE, &mut code) },
            0
        );
        assert_eq!(code, 90); // The test runner never ran in the suspended child.
        assert_eq!(
            unsafe { WaitForSingleObject(GetCurrentProcess(), 0) },
            WAIT_TIMEOUT
        );
    }

    #[test]
    fn dropped_preparation_cancels_only_its_original_child() {
        use windows_sys::Win32::Foundation::{DuplicateHandle, DUPLICATE_SAME_ACCESS};
        let path = std::env::current_exe().unwrap().canonicalize().unwrap();
        let child = create_fixture_unstarted(&path, pin_image(&path).unwrap(), None).unwrap();
        let current = unsafe { GetCurrentProcess() };
        let mut duplicate = ptr::null_mut();
        assert_ne!(
            unsafe {
                DuplicateHandle(
                    current,
                    child.process.as_raw_handle() as HANDLE,
                    current,
                    &mut duplicate,
                    0,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            },
            0
        );
        let retained = unsafe { OwnedHandle::from_raw_handle(duplicate as _) };
        drop(child);
        assert_eq!(
            unsafe { WaitForSingleObject(retained.as_raw_handle() as HANDLE, 0) },
            WAIT_OBJECT_0
        );
    }

    #[test]
    fn malformed_executable_strings_are_refused() {
        for value in ["", "x\0y", "x\"y"] {
            assert!(wide(OsStr::new(value)).is_err());
        }
    }
