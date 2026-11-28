use super::*;
fn own_key() -> ProcessKey {
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            std::process::id(),
        )
    };
    assert!(!raw.is_null());
    let process = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    ProcessKey {
        pid: std::process::id(),
        created_file_time: created(&process).unwrap().to_string(),
    }
}
#[test]
fn appcontainer_sid_text_accepts_only_canonical_binary_package_sids() {
    let value = [
        1u8,2,0,0,0,0,0,15,2,0,0,0,17,0,0,0,
    ];
    assert_eq!(container_sid_text(&value).unwrap(), "S-1-15-2-17");
    for bad in [
        &value[..15],
        &[2,2,0,0,0,0,0,15,2,0,0,0,17,0,0,0][..],
        &[1,2,0,0,0,0,0,5,2,0,0,0,17,0,0,0][..],
        &[1,2,0,0,0,0,0,15,3,0,0,0,17,0,0,0][..],
        &[1,16,0,0,0,0,0,15,2,0,0,0,17,0,0,0][..],
    ] {
        assert!(container_sid_text(bad).is_err());
    }
}

#[test]
fn current_process_package_form_is_sanitized_and_stale_key_fails() {
    let own = own_key();
    let valid = inspect_package_form(&own).unwrap();
    assert!(matches!(
        valid,
        ObservedProcessPackageForm::Packaged
            | ObservedProcessPackageForm::Unpackaged
    ));
    let stale = ProcessKey {
        pid: own.pid,
        created_file_time: "1".into(),
    };
    assert!(inspect_package_form(&stale).is_err());
}

#[test]
fn current_process_family_query_agrees_with_package_form_without_exposing_names() {
    let key = own_key();
    let process = pin(&key).unwrap();
    let observed = package_family_name(&process).unwrap();
    live(&process).unwrap();
    assert_eq!(
        observed.is_some(),
        inspect_package_form(&key).unwrap() == ObservedProcessPackageForm::Packaged,
    );
    let recycled = ProcessKey {
        pid: key.pid, created_file_time: "1".to_owned(),
    };
    assert!(pin(&recycled).is_err());
}

#[test]
fn combined_native_pair_matches_separate_read_only_inspections() {
    let key = own_key();
    let (context, container) = compare_pair(&key, &key).unwrap();
    assert_eq!(context, compare(&key, &key).unwrap());
    assert_eq!(container, compare_appcontainers(&key, &key).unwrap());
    assert!(!container.distinct_container_identities);

    let stale = ProcessKey {
        pid: key.pid,
        created_file_time: "1".into(),
    };
    assert!(compare_pair(&key, &stale).is_err());
    assert!(compare_pair(&stale, &key).is_err());
}

#[test]
fn actual_windows_context_is_private_and_rejects_stale_identity() {
    let key = own_key();
    let comparison = compare(&key, &key).unwrap();
    assert_eq!(
        comparison,
        ContextComparison {
            same_windows_user: true,
            same_profile_directory: true,
            same_windows_session: true
        }
    );
    let json = serde_json::to_value(comparison).unwrap();
    assert_eq!(json.as_object().unwrap().len(), 3);
    assert!(json
        .as_object()
        .unwrap()
        .values()
        .all(|value| value.is_boolean()));
    assert!(compare(
        &ProcessKey {
            pid: key.pid,
            created_file_time: "1".into()
        },
        &key
    )
    .is_err());
    assert!(compare(
        &ProcessKey {
            pid: 0,
            created_file_time: key.created_file_time.clone()
        },
        &key
    )
    .is_err());
}

#[test]
fn token_user_only_reader_agrees_with_full_context_without_export() {
    let key = own_key();
    let process = pin(&key).unwrap();
    let token = query_token(&process).unwrap();
    let direct = user_sid_from_token(&token).unwrap();
    let context = context_from_token(&token).unwrap();
    assert_eq!(direct, context.user);
    assert!(direct.len() >= 8 && direct.len() <= 68);
    live(&process).unwrap();
}

#[test]
fn appcontainer_evidence_is_bounded_and_stale_processes_are_rejected() {
    let key = own_key();
    let comparison = compare_appcontainers(&key, &key).unwrap();
    assert!(!comparison.distinct_container_identities);
    let json = serde_json::to_value(comparison).unwrap();
    assert_eq!(json.as_object().unwrap().len(), 2);
    assert!(json.as_object().unwrap().values().all(|v| v.is_boolean()));
    assert!(compare_appcontainers(
        &key,
        &ProcessKey {
            pid: key.pid,
            created_file_time: "1".into(),
        },
    ).is_err());
}

#[test]
fn independent_source_observation_keeps_game_and_storage_unknown() {
    use virtual_clients_core::independent_engine::{
        NativePortError, ProcessIdentity, SessionSlot, StorageSurface,
    };
    let key = own_key();
    let expected = ProcessIdentity::checked(
        key.pid,
        key.created_file_time.parse::<u64>().unwrap(),
    ).unwrap();
    let result = super::super::inspect_independent_source(
        SessionSlot::Mc02,
        expected,
    ).unwrap();
    assert_eq!(result.process.identity, expected);
    assert!(matches!(
        result.process.containment,
        ProcessContainment::NoAppContainerToken
            | ProcessContainment::AppContainerTokenObserved
    ));
    assert!(!result.process.official_image_signature_observed);
    assert!(!result.process.visible_window_observed);
    assert!(result.storage.unresolved().contains(&StorageSurface::RoamingKnownFolder));
    assert!(result.storage.unresolved().contains(&StorageSurface::AuthenticationBroker));
    assert!(result.storage.unresolved().contains(&StorageSurface::NamedObjects));

    let stale = ProcessIdentity::checked(expected.pid, expected.created_file_time + 1)
        .unwrap();
    assert_eq!(
        super::super::inspect_independent_source(SessionSlot::Mc02, stale)
            .unwrap_err(),
        NativePortError::InvalidIdentity
    );
}

#[test]
fn relative_directory_open_does_not_follow_arbitrary_path_segments() {
    let root = std::env::temp_dir().join(format!(
        "mivubi-relative-root-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(
            std::time::UNIX_EPOCH,
        ).unwrap().as_nanos(),
    ));
    std::fs::create_dir(&root).unwrap();
    struct Cleanup(std::path::PathBuf);
    impl Drop for Cleanup {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _cleanup = Cleanup(root.clone());
    std::fs::create_dir(root.join("only-child")).unwrap();
    std::fs::create_dir(root.join(".private")).unwrap();
    std::fs::write(root.join("regular-file.txt"), b"synthetic").unwrap();
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };
    let parent = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&root).unwrap();
    let child = open_relative_directory_metadata(&parent, "only-child").unwrap();
    assert!(ancestor_directory_identity(&child).is_ok());
    // Dot-prefixed names are legitimate directories; only "."/".."
    // and ambiguous trailing/stream/path syntax are refused.
    let hidden = open_relative_directory_metadata(&parent, ".private").unwrap();
    assert!(ancestor_directory_identity(&hidden).is_ok());
    // NtOpenFile may return a regular file handle when not asking for
    // FILE_DIRECTORY_FILE: type-checking the resulting handle is mandatory.
    if let Ok(file) = open_relative_directory_metadata(&parent, "regular-file.txt") {
        assert!(ancestor_directory_identity(&file).is_err());
    }
    for bad in ["", ".", "..", r"..\peer", r"\absolute",
        "other/child", "file:stream", "trailing.", "trailing ",
    ] {
        assert!(open_relative_directory_metadata(&parent, bad).is_err());
    }
}

#[test]
fn opened_dos_chain_rejects_unc_and_traversal_without_any_os_call() {
    let good = opened_dos_prefixes(r"\\?\C:\data\mc02").unwrap();
    assert!(opened_dos_prefixes(r"\\?\C:\data\.private").is_ok());
    assert_eq!(good, vec![
        r"\\?\C:\".to_owned(),
        r"\\?\C:\data".to_owned(),
        r"\\?\C:\data\mc02".to_owned(),
    ]);
    for bad in [
        "", r"C:\data\mc02", r"\\server\share\mc02",
        r"\\?\UNC\server\share\mc02", r"\\?\Volume{demo}\mc02",
        r"\\?\C:\", r"\\?\C:\data\", r"\\?\C:\data\\mc02",
        r"\\?\C:\data\..\peer", r"\\?\C:\data\.\mc02",
        r"\\?\C:\data\bad.", r"\\?\C:\data\bad ",
        r"\\?\C:\data\file:stream", r"\\?\C:\data/bad",
        "\\\\?\\C:\\data\\nul\0component",
    ] {
        assert!(opened_dos_prefixes(bad).is_err(),
            "Ambiguous ancestor passed: {bad:?}");
    }
    let deep = format!("\\\\?\\C:\\{}", vec!["name"; 33].join("\\"));
    assert!(opened_dos_prefixes(&deep).is_err());
}

#[test]
fn regular_file_handle_is_not_a_private_directory_root() {
    // Read-only validation; never probes a host account folder.
    let executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
    let result = inspect_directory_handle(&executable);
    assert_eq!(result.unwrap_err().kind(), io::ErrorKind::InvalidData);
}

#[test]
fn incomplete_directory_acl_never_counts_as_a_checked_root() {
    for (owner, dacl) in [(false, false), (false, true), (true, false)] {
        assert_eq!(
            reject_invalid_directory_acl(owner, dacl).unwrap_err().kind(),
            io::ErrorKind::InvalidData,
        );
    }
    assert!(reject_invalid_directory_acl(true, true).is_ok());
    // Even two true inputs are NOT an ACL effectiveness or
    // account-isolation attestation, just a structural check.
}

#[test]
fn basic_dacl_ace_rejects_broad_grants_and_unsupported_types() {
    // ACL examples are synthetic only; no OS/account/registry access.
    fn ace(kind: u8, mask: u32, sid: &[u8]) -> Vec<u8> {
        let mut data = vec![kind, 0, 0, 0];
        data[2..4].copy_from_slice(&((8 + sid.len()) as u16).to_le_bytes());
        data.extend_from_slice(&mask.to_le_bytes());
        data.extend_from_slice(sid);
        data
    }
    let groups = [
        SID_EVERYONE, SID_AUTHENTICATED, SID_ANONYMOUS,
        SID_USERS, SID_GUESTS,
    ];
    for group in groups {
        assert_eq!(
            reject_broad_directory_ace(&ace(0, 1, group))
                .unwrap_err().kind(),
            io::ErrorKind::InvalidData,
        );
        // Deny does not grant access; it cannot override a separate
        // broad allow ACE elsewhere, which is independently rejected.
        assert!(reject_broad_directory_ace(&ace(1, 1, group)).is_ok());
    }
    let custom = &[1,1,0,0,0,0,0,5,21,0,0,0];
    assert!(reject_broad_directory_ace(&ace(0, 1, custom)).is_ok());
    for bad in [
        ace(5, 1, custom), // object ACE needs a different layout
        ace(9, 1, custom), // callback ACE
        ace(255, 1, custom),
    ] {
        assert_eq!(
            reject_broad_directory_ace(&bad).unwrap_err().kind(),
            io::ErrorKind::InvalidData,
        );
    }
}

#[test]
fn malformed_basic_ace_is_rejected_before_sid_interpretation() {
    let world = [1,1,0,0,0,0,0,1,0,0,0,0];
    let mut basic = vec![0,0,20,0,1,0,0,0];
    basic.extend_from_slice(&world);
    assert!(reject_broad_directory_ace(&basic).is_err());
    basic[0] = 1;
    assert!(reject_broad_directory_ace(&basic).is_ok());
    for corrupted in [
        { let mut x=basic.clone(); x[2]=19; x },
        { let mut x=basic.clone(); x[8]=2; x },
        { let mut x=basic.clone(); x[9]=16; x },
        { let mut x=basic.clone(); x.truncate(15); x },
    ] {
        assert_eq!(
            reject_broad_directory_ace(&corrupted)
                .unwrap_err().kind(),
            io::ErrorKind::InvalidData,
        );
    }
}

#[test]
fn ace_sid_correlation_never_treats_deny_or_foreign_sid_as_allow() {
    fn ace(kind: u8, mask: u32, sid: &[u8]) -> Vec<u8> {
        let mut data = vec![kind, 0, 0, 0];
        data[2..4].copy_from_slice(&((sid.len() + 8) as u16).to_le_bytes());
        data.extend_from_slice(&mask.to_le_bytes());
        data.extend_from_slice(sid);
        data
    }
    let first = &[1,2,0,0,0,0,0,15,2,0,0,0,3,0,0,0];
    let second = &[1,2,0,0,0,0,0,15,2,0,0,0,4,0,0,0];
    assert!(basic_allow_for_sid(&ace(0, 1, first), first));
    assert!(!basic_allow_for_sid(&ace(0, 1, first), second));
    assert!(!basic_allow_for_sid(&ace(1, 1, first), first));
    assert!(!basic_allow_for_sid(&ace(0, 0, first), first));
    // All of these are only syntactic claims; no effective access.
}

#[test]
fn owned_disposable_directory_handle_has_stable_file_identity() {
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    };
    // This is a future opt-in synthetic-only test fixture; the path
    // is always new and never a Minecraft profile or preexisting box.
    let root = std::env::temp_dir().join(format!(
        "mivubi-disposable-root-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    std::fs::create_dir(&root).unwrap();
    struct OwnsDisposable(std::path::PathBuf);
    impl Drop for OwnsDisposable {
        fn drop(&mut self) { let _ = std::fs::remove_dir(&self.0); }
    }
    let _cleanup = OwnsDisposable(root.clone());
    let handle = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
        .open(&root).unwrap();
    let first = inspect_directory_handle(&handle).unwrap();
    let second = inspect_directory_handle(&handle).unwrap();
    assert_eq!(first, second);
    assert_ne!(first.file_index, 0);
    // The optional fixture is new/disposable; ancestor inspection
    // is expected only on supported local DOS paths.
    if opened_root_name(&handle).ok().is_some_and(|name|
        opened_dos_prefixes(&name).is_ok()
    ) {
        inspect_opened_ancestor_chain(&handle, first).unwrap();
        assert!(inspect_opened_ancestor_chain(&handle,
            super::super::PrivateRootObjectIdentity {
                volume: first.volume,
                file_index: first.file_index.wrapping_add(1),
            }).is_err());
    }
}

#[test]
fn fixture_record_identity_refuses_non_eight_byte_files() {
    // This is a normal test executable, never a synthetic observation.
    let executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
    let error = fixture_record_identity(&executable).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}

#[test]
fn stale_legacy_registry_metadata_is_rejected_and_never_reads_hkcu() {
    use virtual_clients_core::independent_engine::ProcessIdentity;
    let current = own_key();
    let stale = ProcessIdentity::checked(
        current.pid,
        current.created_file_time.parse::<u64>().unwrap() + 1,
    ).unwrap();
    assert_eq!(
        super::super::inspect_legacy_token_virtualization_source(stale).unwrap_err(),
        super::super::NativePortError::InvalidIdentity,
    );
}

#[test]
fn unbound_fixture_reader_checks_process_before_open_file_bytes() {
    use virtual_clients_core::independent_engine::ProcessIdentity;
    let key = own_key();
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            key.pid,
        )
    };
    assert!(!raw.is_null());
    let process = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    // A normal desktop test process is not a synthetic AppContainer;
    // do not attempt to interpret arbitrary files as fixture evidence.
    if appcontainer_sid(&process).unwrap().is_none() {
        let mut executable = std::fs::File::open(std::env::current_exe().unwrap()).unwrap();
        let expected = ProcessIdentity::checked(
            key.pid, key.created_file_time.parse::<u64>().unwrap()
        ).unwrap();
        assert_eq!(
            super::super::inspect_unbound_synthetic_fixture_record(
                expected, &mut executable
            ).unwrap_err(),
            super::super::FixtureRecordInspectionError::NotSyntheticContainer
        );

        let stale = ProcessIdentity::checked(
            expected.pid, expected.created_file_time + 1
        ).unwrap();
        assert_eq!(
            super::super::inspect_unbound_synthetic_fixture_record(
                stale, &mut executable
            ).unwrap_err(),
            super::super::FixtureRecordInspectionError::Process(
                super::super::NativePortError::InvalidIdentity
            )
        );
    }
}

#[test]
fn actual_child_identity_is_pinned_without_controlling_its_lifetime() {
    use std::{
        os::windows::process::CommandExt,
        process::{Child, Command, Stdio},
    };
    struct Fixture(Child);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let mut fixture = Fixture(
        Command::new(crate::engine::native_clients::system_powershell().unwrap())
            .args([
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Start-Sleep -Seconds 60",
            ])
            .creation_flags(0x08000000)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let raw = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            fixture.0.id(),
        )
    };
    assert!(!raw.is_null());
    let process = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    let child_key = ProcessKey {
        pid: fixture.0.id(),
        created_file_time: created(&process).unwrap().to_string(),
    };
    let parent_key = own_key();
    assert_eq!(
        compare(&parent_key, &child_key).unwrap(),
        ContextComparison {
            same_windows_user: true,
            same_profile_directory: true,
            same_windows_session: true,
        }
    );
    drop(pin(&child_key).unwrap());
    assert!(
        fixture.0.try_wait().unwrap().is_none(),
        "Releasing observation must not terminate a process"
    );
    fixture.0.kill().unwrap();
    fixture.0.wait().unwrap();
    // Keep the original handle alive: a dead identity must not be replaced by PID reuse.
    assert_eq!(
        compare(&parent_key, &child_key).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    assert_eq!(live(&process).unwrap_err().kind(), io::ErrorKind::NotFound);
}
