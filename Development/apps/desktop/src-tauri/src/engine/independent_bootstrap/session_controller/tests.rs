//! Existing controller fixtures; original test module path is retained.
    use super::*;
    #[cfg(feature="mc02-native-input")]
    #[test]
    fn legacy_prepublication_refusal_is_distinct_from_unknown_native_ack() {
        // Pure error provenance check. All named original Channel
        // branches reject before cursorRevision/packet publication.
        use std::io::{Error,ErrorKind};
        for message in [
            // Native foreground/window/mode refusal before the original
            // Channel is ever called. An active game-input packet has
            // not been published by THIS failed attempt.
            "Native AI experiment is closing",
            "AI profile is not running",
            "AI profile was not opted in at launch",
            "Verified AI game window unavailable",
            "AI game window is not responsive",
            "AI window is not playable",
            "Human MC01 is not running",
            "Hybrid human MC01 has an AI input channel",
            "Verified human MC01 window unavailable",
            "Human and AI game identities not independent",
            "MC01 human foreground changed",
            "MC02 release target does not match owned process",
            "MC02 input predates original channel binding",
            "MC02 release request expired before native delivery",
            "MC02 release predates newer active input",
            "MC02 release predates newer neutral release",
            "MC02 batch cleanup superseded by another native sender",
            "Unbound, expired or invalid MC02 input command",
            "MC02 input request expired before native delivery",
            "MC02 active input predates neutral release",
            "MC02 active input predates newer active input",
            "MC02 press edge count exhausted until input epoch reset",
            "MC02 last native sender unavailable",
        ] {
            assert!(legacy_native_refused_before_publication(
                &Error::new(ErrorKind::InvalidData,message)));
            for uncertain in [
                ErrorKind::BrokenPipe,ErrorKind::TimedOut,
                ErrorKind::Interrupted,ErrorKind::WouldBlock,
                ErrorKind::Other,ErrorKind::PermissionDenied,
            ] {
                assert!(!legacy_native_refused_before_publication(
                    &Error::new(uncertain,message)),
                    "same text from a different error provenance is not proof");
            }
        }
        for message in [
            "Unknown original HWND validation failure",
            "Unknown Windows target process lookup result",
            "The Windows memory view could not be unmapped",
            "Original session request worker returned no result",
            "AI group neutral release unconfirmed",
        ] {
            assert!(!legacy_native_refused_before_publication(
                &Error::new(ErrorKind::InvalidData,message)));
        }
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn original_mc02_batch_pin_refuses_replaced_child_before_legacy_write() {
        // Pure comparison fixture; production checks the retained binding
        // BEFORE touching legacy lease state or native input memory.
        let pin=Mc02BatchOrigin{pid:202,birth:134_000_000_000_000_000,
            session:1};
        let original=Some((pin.pid,pin.birth,pin.session));
        assert!(legacy_mc02_batch_matches_original(Some(pin),original));
        assert!(!legacy_mc02_batch_matches_original(Some(pin),
            Some((999,pin.birth+100,1))),"replacement PID/birth");
        assert!(!legacy_mc02_batch_matches_original(Some(pin),
            Some((pin.pid,pin.birth+1,1))),"reused numeric PID");
        assert!(!legacy_mc02_batch_matches_original(Some(pin),
            Some((pin.pid,pin.birth,2))),"new Windows session");
        assert!(!legacy_mc02_batch_matches_original(Some(pin),None));
        assert!(legacy_mc02_batch_matches_original(None,original),
            "old one-shot route retains its current independent admission");
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn ai_writer_acquisition_requires_matching_original_launch_mode() {
        // Pure mode fixture only. Never launches or reaps a game.
        use crate::engine::native_clients_session::AgentControlMode as Mode;
        for slot in 2..=5 {
            assert!(ai_writer_mode_matches(
                slot,Mode::HumanPlusFourAi,Mode::HumanPlusFourAi));
            assert!(ai_writer_mode_matches(slot,Mode::FiveAi,Mode::FiveAi));
            assert!(!ai_writer_mode_matches(
                slot,Mode::HumanPlusFourAi,Mode::FiveAi));
            assert!(!ai_writer_mode_matches(
                slot,Mode::FiveAi,Mode::HumanPlusFourAi));
        }
        assert!(!ai_writer_mode_matches(1,Mode::HumanPlusFourAi,
            Mode::HumanPlusFourAi));
        assert!(ai_writer_mode_matches(1,Mode::FiveAi,Mode::FiveAi));
        assert!(!ai_writer_mode_matches(1,Mode::FiveAi,
            Mode::HumanPlusFourAi));
        assert!(!ai_writer_mode_matches(6,Mode::FiveAi,Mode::FiveAi));
    }
    #[cfg(feature="mc02-native-input")]
    #[test]
    fn partial_ai_group_failure_attempts_all_original_neutral_releases() {
        // Pure fixture: never opens native IPC, Windows or game processes.
        let mut visited=Vec::new();
        let confirmed=release_attempted_ai_group(&[2,3,4],|slot|{
            visited.push(slot);
            if slot==3 {
                Err(std::io::Error::other("Synthetic neutral refused"))
            }else{Ok(())}
        });
        assert!(!confirmed,"one unconfirmed neutral must be surfaced");
        assert_eq!(visited,[2,3,4],
            "later original channels must still receive their own cleanup");
        assert!(!visited.contains(&5),
            "a not-yet-dispatched slot must not be touched");

        let mut successful=Vec::new();
        assert!(release_attempted_ai_group(&[2,4],|slot|{
            successful.push(slot);Ok(())
        }));
        assert_eq!(successful,[2,4]);
        assert!(release_attempted_ai_group(&[],|_|{
            panic!("no IPC was attempted, so there is nothing to release")
        }));
    }

    #[test]
    fn desktop_folder_and_selective_reads_use_receipts_without_changing_mutation_snapshots() {
        for request in [
            SessionRequest::OpenProfileFolder { slot: 1, selected_profile: None },
            SessionRequest::InspectSelectiveInventory { slot: 2, category: SelectiveCategory::Worlds, selected_profile: Some(false) },
            SessionRequest::ReviewSelectiveInventory { slot: 3, selections: vec![], selected_profile: None },
        ] {
            assert_eq!(ReplySnapshot::for_request(&request), ReplySnapshot::Receipt);
        }
        for (slot, request) in [
            (1, SessionRequest::OpenProfileFolder { slot: 1, selected_profile: Some(true) }),
            (2, SessionRequest::InspectSelectiveInventory { slot: 2, category: SelectiveCategory::Worlds, selected_profile: Some(true) }),
            (3, SessionRequest::ReviewSelectiveInventory { slot: 3, selections: vec![], selected_profile: Some(true) }),
        ] {
            assert_eq!(ReplySnapshot::for_request(&request), ReplySnapshot::Selected(slot));
        }
        for request in [
            SessionRequest::ClearProfileCache { slot: 1, selected_storage: None },
            SessionRequest::ResetGameSettings { slot: 2, selected_storage: Some(true) },
            SessionRequest::ConfirmSelectedWorlds { slot: 3, selections: vec![], confirmation_token: String::new() },
        ] {
            assert_eq!(ReplySnapshot::for_request(&request), ReplySnapshot::Full);
        }
    }

    #[test]
    fn mutation_receipt_scope_defaults_to_full_and_selects_only_the_requested_slot() {
        for slot in 1..=5 {
            assert_eq!(mutation_storage_slots(slot,None),1..=5);
            assert_eq!(mutation_storage_slots(slot,Some(false)),1..=5);
            assert_eq!(mutation_storage_slots(slot,Some(true)),slot..=slot);
        }
    }

    #[test]
    fn reply_snapshot_policy_distinguishes_close_receipts_and_full_observation() {
        // Pure routing assertions; do not launch a game or touch profile data.
        for slot in 1..=5 {
            assert_eq!(ReplySnapshot::for_request(&SessionRequest::InspectAgentTarget{slot}),
                ReplySnapshot::AgentTarget(slot));
        }
        for request in [
            SessionRequest::CloseOne { slot: 1 },
            SessionRequest::CloseOne { slot: 5 },
            SessionRequest::CloseAll,
        ] {
            assert_eq!(ReplySnapshot::for_request(&request), ReplySnapshot::Close);
        }
        for request in [
            SessionRequest::Inspect,
            SessionRequest::InspectSettings,
            SessionRequest::InspectStorage,
            SessionRequest::InspectProfileStorage { slot: 3 },
            SessionRequest::SavePerformance {active_fps:144,background_fps:10},
            SessionRequest::SaveStoragePreferences {preferences:Default::default()},
            SessionRequest::SaveAudio {mode:crate::engine::manager_settings::BackgroundAudio::Keep},
            SessionRequest::Launch { slot: 2 },
        ] {
            assert_eq!(ReplySnapshot::for_request(&request), ReplySnapshot::Full);
        }
        for request in [
            SessionRequest::TestingInspect { slot: 1, owner_id: String::new() },
            SessionRequest::TestingReserve { slot: 1, owner_id: String::new(),
                run_id: String::new(), replace_run_id: None },
            SessionRequest::TestingFinalize { slot: 1, owner_id: String::new(),
                run_id: String::new(), reported_status: String::new(), report_digest: String::new() },
        ] {
            assert_eq!(ReplySnapshot::for_request(&request), ReplySnapshot::AgentReceipt);
        }
        let empty=SessionSnapshot::empty_receipt();
        assert!(empty.profiles.is_empty());
        assert!(empty.agent_activity.is_none());
        assert!(empty.testing_run.is_none() && empty.testing_inspection.is_none());
        #[cfg(feature="mc02-native-input")]
        {
            let frame=crate::engine::native_clients_session::AiInputFrame {
                slot:2,keys:1,dx:0,dy:0,ttl_ms:350,
            };
            let request=SessionRequest::AiInputGroup {
                frames:vec![frame],issued_at_ms:1000,deadline_at_ms:1100,
                owner_id:"a".repeat(32),
                lease_epoch:"1".repeat(32),
            };
            assert_eq!(ReplySnapshot::for_request(&request), ReplySnapshot::InputReceipt);
            for request in [
                SessionRequest::Mc02Input {slot:2,keys:1,dx:0,dy:0,
                    ttl_ms:350,issued_at_ms:1,batch_origin:None},
                SessionRequest::AiLeasedInput {slot:2,keys:0,dx:0,dy:0,
                    ttl_ms:1,issued_at_ms:1,owner_id:"a".repeat(32),lease_epoch:"b".repeat(32)},
                SessionRequest::AiLeaseAcquire {slots:vec![2],owner_id:"a".repeat(32),
                    mode:AgentControlMode::HumanPlusFourAi,lease_epoch:"b".repeat(32),neutral_only:false},
                SessionRequest::AiLeaseRelease {slots:vec![2],owner_id:"a".repeat(32),
                    lease_epoch:"b".repeat(32)},
            ] {
                assert_eq!(ReplySnapshot::for_request(&request),ReplySnapshot::InputReceipt);
            }
            for request in [
                SessionRequest::LaunchMc02Experiment{slot:2},
                SessionRequest::LaunchAiExperiment{slot:2,mode:AgentControlMode::HumanPlusFourAi},
            ] {
                assert_eq!(ReplySnapshot::for_request(&request),ReplySnapshot::Full);
            }
        }
        // CloseOne reaps one verified slot; CloseAll still reaps all five.
        for slot in 1..=5 {
            let selected = normal_close_mask(Some(slot)).unwrap();
            assert_eq!(selected.iter().filter(|&&enabled| enabled).count(), 1);
            assert!(selected[(slot - 1) as usize]);
        }
        assert_eq!(normal_close_mask(None).unwrap(), [true; 5]);
    }

    #[test]
    fn stable_audio_skips_redundant_com_work_but_keeps_focus_and_exit_immediate() {
        use crate::engine::manager_settings::BackgroundAudio;
        let start = Instant::now();
        let roster: AudioRoster = [Some((102, 134000000000000001)), None, None, None, None];
        let mut gate = AudioIdleGate::default();
        assert!(gate.due(start, Some(102), roster));
        gate.success(start, Some(102), roster);
        for elapsed in [250, 500, 750] {
            assert!(!gate.due(start + Duration::from_millis(elapsed), Some(102), roster),
                "Stable mute policy must not read COM every 250 ms");
        }
        assert!(gate.due(start + AUDIO_IDLE_RESYNC, Some(102), roster),
            "New audio sessions can still be discovered once per second");
        assert!(gate.due(start + Duration::from_millis(250), Some(103), roster),
            "A foreground change must trigger the next idle tick");
        let mut replaced = roster;
        replaced[0] = Some((103, 134000000000000002));
        assert!(gate.due(start + Duration::from_millis(250), Some(102), replaced),
            "A reused PID or new original birth must not inherit mute state");
        gate.invalidate();
        assert!(gate.due(start + Duration::from_millis(250), Some(102), roster),
            "Any failed COM sync must be retryable immediately");
        assert!(!audio_maintenance_needed(BackgroundAudio::Keep, false, roster));
        assert!(audio_maintenance_needed(BackgroundAudio::Mute, false, roster));
        assert!(audio_maintenance_needed(BackgroundAudio::Keep, true, [None; 5]),
            "An outstanding mute lease must still be restored");
        assert!(!audio_maintenance_needed(BackgroundAudio::Mute, false, [None; 5]),
            "No owned process and no audio lease needs no COM startup");
    }

    #[test]
    fn unrelated_foreground_changes_do_not_repeat_audio_policy() {
        let start = Instant::now();
        let roster: AudioRoster = [Some((102, 1)), Some((103, 2)), None, None, None];
        let next = start + Duration::from_millis(250);
        let mut gate = AudioIdleGate::default();
        gate.success(start, Some(900), roster);
        assert!(!gate.due(next, Some(901), roster));
        assert!(!gate.due(next, None, roster));
        assert!(gate.due(next, Some(102), roster));
        assert!(gate.due(start + AUDIO_IDLE_RESYNC, Some(901), roster));
        gate.success(next, Some(102), roster);
        let later = next + Duration::from_millis(250);
        assert!(!gate.due(later, Some(102), roster));
        assert!(gate.due(later, Some(103), roster));
        assert!(gate.due(later, Some(900), roster));
        let mut replaced = roster;
        replaced[0] = Some((102, 3));
        assert!(gate.due(later, Some(102), replaced));
        gate.invalidate();
        assert!(gate.due(later, Some(102), roster));
    }

    #[test]
    fn audio_deadline_uses_hint_only_after_success_for_same_roster() {
        let start=Instant::now();
        let roster:AudioRoster=[Some((102,1)),None,None,None,None];
        let mut gate=AudioIdleGate::default();
        assert_eq!(gate.wait_budget(start,roster,true,None),Duration::from_millis(250));
        assert!(!gate.current_success(roster));
        gate.success(start,Some(102),roster);
        assert!(gate.current_success(roster));
        assert_eq!(gate.wait_budget(start,roster,true,None),AUDIO_IDLE_RESYNC);
        // A focus-only success cannot postpone an earlier discovery deadline.
        assert_eq!(gate.wait_budget(start,roster,true,Some(Duration::from_millis(100))),Duration::from_millis(100));
        assert_eq!(gate.wait_budget(start,roster,true,Some(Duration::ZERO)),Duration::ZERO);
        assert_eq!(gate.wait_budget(start,roster,false,None),Duration::from_millis(250));
        let later=start+Duration::from_millis(900);
        assert_eq!(gate.wait_budget(later,roster,true,None),Duration::from_millis(100));
        assert_eq!(gate.wait_budget(later,roster,false,None),Duration::from_millis(100));
        assert_eq!(gate.wait_budget(start+AUDIO_IDLE_RESYNC,roster,true,None),Duration::ZERO);
        let mut changed=roster;changed[0]=Some((102,2));
        assert!(!gate.current_success(changed));
        assert_eq!(gate.wait_budget(start,changed,true,Some(Duration::ZERO)),Duration::from_millis(250));
        gate.invalidate();
        assert!(!gate.current_success(roster));
        assert_eq!(gate.wait_budget(start,roster,true,Some(Duration::ZERO)),Duration::from_millis(250));
    }

    #[test]
    fn idle_controller_cadence_preserves_only_actual_deadlines() {
        assert_eq!(maintenance_interval(false,false,false),Duration::from_secs(1));
        for storage in [false,true] {
            for audio in [false,true] {
                for retry in [false,true] {
                    if storage||audio||retry {
                        assert_eq!(maintenance_interval(storage,audio,retry),Duration::from_millis(250));
                    }
                }
            }
        }
    }

    #[test]
    fn healthy_live_controller_waits_for_events_without_idle_maintenance() {
        for running in [false,true] {
            for storage in [false,true] {
                for audio in [false,true] {
                    for cadence in [Duration::ZERO, Duration::from_millis(250), Duration::from_secs(1)] {
                        let expected = if running && !storage && !audio {Duration::MAX} else {cadence};
                        assert_eq!(native_wait_budget(running,storage,audio,cadence),expected);
                    }
                }
            }
        }
        // The channel fallback still uses the bounded maintenance cadence.
        assert_eq!(maintenance_interval(false,false,false),Duration::from_secs(1));
        assert_eq!(maintenance_interval(false,false,true),Duration::from_millis(250));
    }

    // Private synthetic OGLES class, not Minecraft. Shared registration is
    // initialized once because Rust test cases can execute concurrently.
    fn fixture_game_window() -> windows_sys::Win32::Foundation::HWND {
        use windows_sys::Win32::{
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, RegisterClassW, WNDCLASSW, WS_OVERLAPPEDWINDOW},
        };
        static REGISTERED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        REGISTERED.get_or_init(|| unsafe {
            let mut class: WNDCLASSW = std::mem::zeroed();
            class.hInstance = GetModuleHandleW(std::ptr::null());
            class.lpszClassName = windows_sys::w!("OGLES");
            class.lpfnWndProc = Some(DefWindowProcW);
            assert_ne!(RegisterClassW(&class), 0);
        });
        unsafe {
            CreateWindowExW(0, windows_sys::w!("OGLES"), windows_sys::w!("Fixture"),
                WS_OVERLAPPEDWINDOW, 0, 0, 200, 150,
                std::ptr::null_mut(), std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null()), std::ptr::null())
        }
    }

    #[test]
    fn stopped_profile_cache_allows_other_owned_games_but_rejects_unknown_or_reused_processes() {
        let other = OwnedProcess { pid:42, created_file_time:"123456".into(), session_id:1,
            window_handle:"0x1234".into(), responding:true };
        assert!(require_data_idle(true, &[(42,123456)], std::slice::from_ref(&other)).is_ok());
        assert!(require_data_idle(false, &[(42,123456)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[(42,123455)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[(43,123456)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[], &[]).is_ok());
    }

    #[test]
    fn report_admission_reuses_one_observation_without_granting_failed_identity() {
        let other = OwnedProcess { pid:42, created_file_time:"123456".into(), session_id:1,
            window_handle:"0x1234".into(), responding:true };
        let observed = [other];
        let owned = Ok(vec![(42,123456)]);
        assert!(report_data_admission(true, &owned, &observed).is_ok());
        assert!(report_data_admission(false, &owned, &observed).is_err());
        assert!(report_data_admission(true, &Ok(vec![(42,123455)]), &observed).is_err());
        let failed = Err(io::Error::new(io::ErrorKind::PermissionDenied, "Identity unavailable"));
        for selected_idle in [true, false] {
            let error = report_data_admission(selected_idle, &failed, &observed).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
            assert_eq!(error.to_string(), "Identity unavailable");
        }
    }

    #[test]
    fn normal_close_only_reaps_after_all_selected_processes_have_exited() {
        let mut sessions=Sessions{
            stores:(0..5).map(|_|None).collect(),
            running:(0..5).map(|_|None).collect(),
            issues:vec![None;5],windows:vec![None;5],
            path:String::new(),
            exiting:false,
            pending_storage_maintenance:[false;5],
            #[cfg(feature="mc02-native-input")]
            ai_leases:AiLeaseBook::default(),
        };
        let one=normal_close_mask(Some(2)).unwrap();
        assert!(sessions.selected_processes_exited(&one).unwrap());
        let all=normal_close_mask(None).unwrap();
        assert!(sessions.selected_processes_exited(&all).unwrap());
        // Inspect is still an ordinary reaping operation, while the close
        // path can safely take a non-reaping snapshot of all five slots.
        assert_eq!(sessions.snapshot_without_reaping(true).profiles.len(),5);
        assert!(sessions.running.iter().all(Option::is_none));
        #[cfg(feature="mc02-native-input")]
        assert!(sessions.launch_mc02_experiment(2).unwrap_err().to_string()
            .contains("requires live MC01"));
    }

    #[test]
    fn optional_http_cleanup_drops_pending_work_during_terminal_exit() {
        let mut sessions=Sessions {
            stores:(0..5).map(|_|None).collect(), running:(0..5).map(|_|None).collect(),
            issues:vec![None;5], windows:vec![None;5], path:String::new(),
            exiting:true, pending_storage_maintenance:[true;5],
            #[cfg(feature="mc02-native-input")]
            ai_leases:AiLeaseBook::default(),
        };
        sessions.maintain_storage();
        assert_eq!(sessions.pending_storage_maintenance,[false;5]);
    }

    #[test]
    fn selected_normal_close_mask_affects_only_one_launcher() {
        assert_eq!(normal_close_mask(None).unwrap(),[true;5]);
        for slot in 1..=5 {
            let mask=normal_close_mask(Some(slot)).unwrap();
            for (index,selected) in mask.iter().enumerate() {
                assert_eq!(*selected,index as u32+1==slot);
            }
        }
        for slot in [0,6,u32::MAX] {
            assert_eq!(normal_close_mask(Some(slot)).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn full_exit_of_idle_sessions_does_not_prepare_profiles_and_denies_new_launches() {
        let mut sessions=Sessions{stores:(0..5).map(|_|None).collect(),running:(0..5).map(|_|None).collect(),issues:vec![None;5],windows:vec![None;5],path:String::new(),exiting:false,pending_storage_maintenance:[false;5],#[cfg(feature="mc02-native-input")] ai_leases:AiLeaseBook::default()};
        // The selected-close owner must never report a successful exit
        // for a stopped slot even if it has no process to wait for.
        for slot in 1..=5 {
            assert!(sessions.close_one(slot).unwrap_err().to_string().contains("not running"));
            assert!(!sessions.exiting);
        }
        sessions.close_all().unwrap();
        assert!(sessions.exiting);
        assert!(sessions.stores.iter().all(Option::is_none));
        assert!(sessions.launch(1).unwrap_err().to_string().contains("closing"));
    }

    #[test]
    fn normal_close_checks_exact_window_identity_and_leaves_another_window_alone() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let own=fixture_game_window();let other=fixture_game_window();assert!(!own.is_null()&&!other.is_null());
            let result=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{pid:std::process::id(),created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),session_id:interactive_session(std::process::id()).unwrap(),window_handle:format!("0x{:x}",own as usize),responding:true};
                let mut forged=target.clone();forged.created_file_time="1".into();assert!(close_verified_window(&forged).is_err());
                assert_ne!(IsWindow(own),0);
                let style=GetWindowLongPtrW(own,GWL_STYLE);SetWindowLongPtrW(own,GWL_STYLE,style|WS_DISABLED as isize);
                assert!(close_verified_window(&target).is_err());SetWindowLongPtrW(own,GWL_STYLE,style);
                close_verified_window(&target).unwrap();
                let mut message:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut message,own,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                DispatchMessageW(&message);
                assert_eq!(IsWindow(own),0);
                assert_ne!(IsWindow(other),0);
            });
            if IsWindow(own)!=0{DestroyWindow(own);}DestroyWindow(other);
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }


    #[test]
    fn normal_close_posts_once_per_window_and_tracks_original_window_thread() {
        use windows_sys::Win32::{Foundation::FILETIME, System::Threading::*, UI::WindowsAndMessaging::*};
        unsafe {
            let first=fixture_game_window();let successor=fixture_game_window();
            assert!(!first.is_null()&&!successor.is_null()&&first!=successor);
            let result=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{
                    pid:std::process::id(),
                    created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),
                    session_id:interactive_session(std::process::id()).unwrap(),
                    window_handle:format!("0x{:x}",first as usize),
                    responding:true,
                };
                let mut progress=NormalCloseProgress::default();
                let style=GetWindowLongPtrW(first,GWL_STYLE);
                SetWindowLongPtrW(first,GWL_STYLE,style|WS_DISABLED as isize);
                progress.request_window(&target).unwrap();
                assert!(progress.blocked_by_modal);
                assert!(progress.requested_windows.is_empty());
                let mut none:MSG=std::mem::zeroed();
                assert_eq!(PeekMessageW(&mut none,first,WM_CLOSE,WM_CLOSE,PM_NOREMOVE),0);
                SetWindowLongPtrW(first,GWL_STYLE,style);
                progress.request_window(&target).unwrap();
                progress.request_window(&target).unwrap();
                assert!(!progress.blocked_by_modal);
                assert_eq!(progress.requested_windows.len(),1);
                assert_eq!(progress.original_thread_state(),"alive");
                let mut message:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut message,first,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_eq!(PeekMessageW(&mut message,first,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);

                // A new, uniquely verified top-level HWND receives its own request.
                let new_target=OwnedProcess{window_handle:format!("0x{:x}",successor as usize),..target};
                progress.request_window(&new_target).unwrap();
                progress.request_window(&new_target).unwrap();
                assert_eq!(progress.requested_windows.len(),2);
                assert_ne!(PeekMessageW(&mut message,successor,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_eq!(PeekMessageW(&mut message,successor,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_ne!(IsWindow(successor),0);
            });
            if IsWindow(first)!=0{DestroyWindow(first);}
            if IsWindow(successor)!=0{DestroyWindow(successor);}
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }


    #[test]
    fn hidden_game_close_candidate_refuses_dialog_and_foreign_pid() {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        unsafe {
            let game=fixture_game_window();
            let dialog=CreateWindowExW(0,windows_sys::w!("#32770"),
                windows_sys::w!("Minecraft Education"),WS_POPUP,
                0,0,200,120,std::ptr::null_mut(),std::ptr::null_mut(),
                std::ptr::null_mut(),std::ptr::null());
            assert!(!game.is_null()&&!dialog.is_null());
            let result=std::panic::catch_unwind(||{
                let pid=std::process::id();
                assert_eq!(IsWindowVisible(game),0);
                assert!(hidden_close_candidate(game,pid));
                assert!(!hidden_close_candidate(game,pid.wrapping_add(1)));
                assert!(!hidden_close_candidate(dialog,pid));
                let mut posted:MSG=std::mem::zeroed();
                assert_eq!(PeekMessageW(&mut posted,dialog,WM_CLOSE,WM_CLOSE,PM_NOREMOVE),0);
            });
            DestroyWindow(dialog);
            DestroyWindow(game);
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }

    #[test]
    fn exit_dialog_is_not_accepted_as_an_ogles_game_window() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let dialog=CreateWindowExW(0,windows_sys::w!("#32770"),windows_sys::w!("Minecraft Education"),
                WS_POPUP|WS_VISIBLE,0,0,250,140,ptr::null_mut(),ptr::null_mut(),ptr::null_mut(),ptr::null());
            assert!(!dialog.is_null());
            let outcome=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exit=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exit,&mut kernel,&mut user),0);
                let target=OwnedProcess{
                    pid:std::process::id(),
                    created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),
                    session_id:interactive_session(std::process::id()).unwrap(),
                    window_handle:format!("0x{:x}",dialog as usize),
                    responding:true,
                };
                assert!(!game_window_class(dialog));
                assert!(!valid_retained_window(&target));
                assert!(close_verified_window(&target).is_err());
                let mut posted:MSG=std::mem::zeroed();
                assert_eq!(PeekMessageW(&mut posted,dialog,WM_CLOSE,WM_CLOSE,PM_NOREMOVE),0);
                assert_ne!(IsWindow(dialog),0);
            });
            DestroyWindow(dialog);
            if let Err(error)=outcome{std::panic::resume_unwind(error);}
        }
    }


    #[test]
    fn hwnd_reuse_on_another_thread_is_not_silently_deduplicated() {
        let mut progress=NormalCloseProgress::default();
        progress.requested_windows.push(RequestedWindow {
            handle:"0x42".into(),
            thread_id:77,
            _thread_pin:None,
        });
        assert!(progress.already_requested("0x42",77));
        assert!(!progress.already_requested("0x42",78));
        assert!(!progress.already_requested("0x43",77));
        // Same numeric HWND reused on the same live GUI thread remains
        // unprovable without in-process lifetime evidence. No blind repost.
    }

    #[test]
    fn close_phase_never_claims_process_exit_from_a_posted_message() {
        let mut progress=NormalCloseProgress::default();
        assert_eq!(progress.phase(false),"WAITING_FOR_WINDOW");
        assert_eq!(progress.phase(true),"WINDOW_AVAILABLE");
        progress.blocked_by_modal=true;
        assert_eq!(progress.phase(true),"WAITING_FOR_MODAL");
        progress.requested_windows.push(RequestedWindow {
            handle:"0x42".into(),
            thread_id:77,
            _thread_pin:None,
        });
        assert_eq!(progress.phase(false),"CLOSE_REQUESTED");
        progress.submitted_dialogs.push((0x99,42));
        assert_eq!(progress.phase(false),"DIALOG_CONFIRMATION_SENT");
        // The real process-exit state is owned solely by Sessions::reap().
    }

    #[test]
    fn exit_dialog_refuses_unverified_process_without_enumerating_windows() {
        assert!(!confirm_exit_dialog(std::process::id(), || false).unwrap());
    }

    #[test]
    fn exit_confirmation_is_limited_to_the_exact_owned_prompt() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let make=|class,caption,style,parent,id|CreateWindowExW(WS_EX_NOACTIVATE,class,caption,style,
                -30000,-30000,200,100,parent,id as _,std::ptr::null_mut(),std::ptr::null());
            let game=fixture_game_window();
            let dialog=make(windows_sys::w!("#32770"),windows_sys::w!("Minecraft Education"),WS_POPUP|WS_VISIBLE,
                std::ptr::null_mut(),0usize);
            assert!(!game.is_null()&&!dialog.is_null());
            let result=std::panic::catch_unwind(||{
                let message=make(windows_sys::w!("STATIC"),windows_sys::w!("Unknown question"),WS_CHILD|WS_VISIBLE,dialog,65535);
                let ok=make(windows_sys::w!("BUTTON"),windows_sys::w!("OK"),WS_CHILD|WS_VISIBLE,dialog,1);
                let cancel=make(windows_sys::w!("BUTTON"),windows_sys::w!("Cancel"),WS_CHILD|WS_VISIBLE,dialog,2);
                assert!(!message.is_null()&&!ok.is_null()&&!cancel.is_null());
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{pid:std::process::id(),created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),session_id:interactive_session(std::process::id()).unwrap(),window_handle:format!("0x{:x}",game as usize),responding:true};
                assert!(!confirm_education_exit(&target).unwrap());
                assert_ne!(SetWindowTextW(message,windows_sys::w!("Exit Minecraft Education?")),0);
                let mut submitted=Vec::new();
                let duplicate=make(windows_sys::w!("#32770"),windows_sys::w!("Minecraft Education"),WS_POPUP|WS_VISIBLE,
                    std::ptr::null_mut(),0usize);
                assert!(!duplicate.is_null());
                let second_message=make(windows_sys::w!("STATIC"),windows_sys::w!("Exit Minecraft Education?"),WS_CHILD|WS_VISIBLE,duplicate,65535);
                let second_ok=make(windows_sys::w!("BUTTON"),windows_sys::w!("OK"),WS_CHILD|WS_VISIBLE,duplicate,1);
                let second_cancel=make(windows_sys::w!("BUTTON"),windows_sys::w!("Cancel"),WS_CHILD|WS_VISIBLE,duplicate,2);
                let complete=!second_message.is_null()&&!second_ok.is_null()&&!second_cancel.is_null();
                let ambiguous=confirm_exit_dialog_once(target.pid,||valid_retained_window(&target),&mut submitted);
                DestroyWindow(duplicate);
                assert!(complete);
                assert!(!ambiguous.unwrap());
                assert!(submitted.is_empty());
                assert!(confirm_exit_dialog_once(target.pid,||valid_retained_window(&target),&mut submitted).unwrap());
                let mut received:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut received,dialog,WM_COMMAND,WM_COMMAND,PM_REMOVE),0);
                assert_eq!(received.wParam,1);
                assert_eq!(submitted.len(),1);
                // Still-present same dialog is observed, never spammed with IDOK.
                assert!(confirm_exit_dialog_once(target.pid,||valid_retained_window(&target),&mut submitted).unwrap());
                assert_eq!(submitted.len(),1);
                assert_eq!(PeekMessageW(&mut received,dialog,WM_COMMAND,WM_COMMAND,PM_REMOVE),0);
                assert_ne!(SetWindowTextW(message,windows_sys::w!("Different question")),0);
                assert!(!confirm_exit_dialog_once(target.pid,||valid_retained_window(&target),&mut submitted).unwrap());
                assert_eq!(submitted.len(),1);
                let mut forged=target.clone();forged.pid=forged.pid.wrapping_add(1);
                assert!(!confirm_education_exit(&forged).unwrap());
            });
            DestroyWindow(dialog);DestroyWindow(game);
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }

    #[test]
    fn preset_request_refuses_uncertain_storage_before_writing_or_starting_any_game() {
        let mut sessions=Sessions {
            stores:(0..5).map(|_|None).collect(), running:(0..5).map(|_|None).collect(),
            issues:vec![None;5], windows:vec![None;5], path:String::new(),
            exiting:false,
            pending_storage_maintenance:[false;5],
            #[cfg(feature="mc02-native-input")]
            ai_leases:AiLeaseBook::default(),
        };
        assert!(sessions.apply_preset().unwrap_err().to_string().contains("Close all Minecraft"));
        assert!(sessions.running.iter().all(Option::is_none));
        assert!(sessions.snapshot().profiles.iter().all(|profile|profile.state=="ATTENTION"));
    }


