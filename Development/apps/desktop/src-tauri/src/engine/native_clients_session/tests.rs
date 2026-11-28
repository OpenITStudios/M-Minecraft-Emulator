//! Existing transport fixtures; original module namespace retained.
    use super::*;

    #[cfg(windows)]
    #[test]
    fn existence_lease_excludes_second_creator_until_original_handle_closes() {
        let name=format!(r"Local\LazyTester-GateFixture-{}",std::process::id());
        let acquire=||acquire_existence_gate(&name,io::Error::new(io::ErrorKind::WouldBlock,"held"));
        let first=acquire().unwrap();
        assert_eq!(acquire().unwrap_err().kind(),io::ErrorKind::WouldBlock);
        drop(first);
        assert!(acquire().is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn pending_timeout_reconciliation_requires_fresh_session_inspect_not_settings() {
        // Pure admission policy fixture: no controller, pipe, Windows
        // game window or source test executor is started by this code.
        assert!(can_reconcile_pending_transport(&SessionRequest::Inspect));
        assert!(!can_reconcile_pending_transport(&SessionRequest::InspectSettings));
        assert!(!can_reconcile_pending_transport(&SessionRequest::InspectStorage));
        assert!(!can_reconcile_pending_transport(&SessionRequest::InspectProfileStorage {slot:1}));
        assert!(!can_reconcile_pending_transport(&SessionRequest::CloseOne{slot:2}));
        assert!(!can_reconcile_pending_transport(&SessionRequest::CloseAll));
        for read in [SessionRequest::InspectSettings, SessionRequest::InspectStorage,
            SessionRequest::InspectProfileStorage {slot:1}] {
            assert!(read_needs_reconciliation(&read,true));
            assert!(!read_needs_reconciliation(&read,false));
        }
        for other in [SessionRequest::Inspect,SessionRequest::InspectAgentTarget {slot:1},
            SessionRequest::CloseOne {slot:2},SessionRequest::CloseAll,
            SessionRequest::Launch {slot:3}] {
            assert!(!read_needs_reconciliation(&other,true));
        }
        #[cfg(feature="mc02-native-input")]
        {
            let owner="a".repeat(32);
            let epoch="b".repeat(30)+"e1";
            assert!(!can_reconcile_pending_transport(
                &SessionRequest::AiLeaseRelease {
                    slots:vec![2],owner_id:owner.clone(),lease_epoch:epoch.clone(),
                }));
            assert!(!can_reconcile_pending_transport(
                &SessionRequest::AiLeaseAcquire {
                    slots:vec![2],owner_id:owner,
                    lease_epoch:epoch,mode:AgentControlMode::HumanPlusFourAi,
                    neutral_only:true,
                }));
        }
    }

    #[cfg(windows)]
    #[test]
    fn pending_worker_outcomes_keep_one_shot_request_provenance() {
        use std::sync::mpsc::{self,RecvTimeoutError};
        use std::time::Duration;
        // Local channels only: no named pipe, controller, game or VM.
        let (tx,rx)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        assert!(matches!(
            classify_pending_worker_wait(rx.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::StillRunning
        ));
        tx.send(Ok(Ok(SessionSnapshot::stopped()))).unwrap();
        assert!(matches!(
            classify_pending_worker_wait(rx.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::Completed(Ok(Ok(_)))
        ));
        // The old channel's result never appears on another request's
        // distinct receiver, even if their invocation order overlaps.
        let (tx_old,rx_old)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        let (tx_new,rx_new)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        tx_old.send(Ok(Err("Original native refusal".into()))).unwrap();
        assert!(matches!(
            classify_pending_worker_wait(rx_new.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::StillRunning
        ));
        assert!(matches!(
            classify_pending_worker_wait(rx_old.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::Completed(Ok(Err(_)))
        ));
        // A transport error from the correct worker is NOT a successful
        // native reply, even if the worker did finish.
        tx_new.send(Err(io::Error::other("Original result unverified"))).unwrap();
        assert!(matches!(
            classify_pending_worker_wait(rx_new.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::Completed(Err(_))
        ));
        let (tx_lost,rx_lost)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        drop(tx_lost);
        assert!(matches!(
            rx_lost.recv_timeout(Duration::ZERO),
            Err(RecvTimeoutError::Disconnected)
        ));
        // The dead worker's closed response channel is not a timeout.
        let (_sender,receiver)=mpsc::sync_channel::<io::Result<SessionReply>>(1);
        // Senders are dropped before checking to model a crashed worker.
        drop(_sender);
        assert!(matches!(
            classify_pending_worker_wait(receiver.recv_timeout(Duration::ZERO)),
            PendingWorkerWait::ResultChannelLost
        ));
    }

    #[cfg(windows)]
    #[test]
    fn previous_worker_drain_never_replays_or_clears_uncertainty() {
        let (sender, replies) = std::sync::mpsc::sync_channel(1);
        let worker = std::thread::spawn(|| {});
        let mut controller = Controller {
            pending: Some(PendingRequest { replies, worker }),
            uncertain: false,
        };
        assert_eq!(controller.drain_previous_request().unwrap_err().kind(),
            io::ErrorKind::WouldBlock);
        assert!(controller.pending.is_some());
        sender.send(Ok(Ok(SessionSnapshot::stopped()))).unwrap();
        controller.drain_previous_request().unwrap();
        assert!(controller.pending.is_none());
        assert!(controller.uncertain);
        controller.drain_previous_request().unwrap();
        assert!(controller.uncertain);
    }

    #[cfg(windows)]
    #[test]
    fn contended_transport_gate_refuses_a_second_request_without_waiting() {
        // Synthetic gate only: no Minecraft process, profile or controller IPC.
        let gate = Mutex::new(Controller { pending: None, uncertain: false });
        let retained = gate.lock().unwrap();
        let refusal = match admit_controller(&gate) {
            Ok(_) => panic!("Contended transport must never admit another request"),
            Err(error) => error,
        };
        assert_eq!(refusal.kind(), io::ErrorKind::WouldBlock);
        drop(retained);
        assert!(admit_controller(&gate).is_ok());
    }

    #[cfg(windows)]
    #[test]
    fn recovery_rejects_out_of_range_slots_before_controller_or_storage_access() {
        for slot in [0,6,u32::MAX] {
            for operation in [SessionRequest::ResetGameSettings {slot,selected_storage:None},SessionRequest::RestoreGameSettings {slot,selected_storage:None},SessionRequest::DisableImprovedInputResponse {slot},SessionRequest::EnableTextureStreaming {slot},SessionRequest::DisableTextureStreaming {slot},SessionRequest::CloseOne {slot}] {
                assert_eq!(request(operation).unwrap_err().kind(),io::ErrorKind::InvalidInput);
            }
        }
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"RestoreGameSettings","slot":1,"path":"outside"}"#).is_err());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"CloseOne","slot":1,"pid":123}"#).is_err());
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"CloseOne","slot":3}"#),Ok(SessionRequest::CloseOne{slot:3})));
        for (slot,token,name) in [(0,"a".repeat(64),"Minecraft - 0"),(1,"bad".into(),"Minecraft - 1"),(1,"a".repeat(64),"Minecraft - 2")] {
            assert_eq!(request(SessionRequest::ClearProfileData {slot,confirmation_token:token,confirmation_name:name.into(),selected_storage:None}).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
    }
    #[cfg(windows)]
    #[test]
    fn selected_storage_request_is_exact_and_bounded() {
        for slot in 1..=5 {
            let request=SessionRequest::InspectProfileStorage {slot};
            assert!(validate_request(&request).is_ok());
            assert!(matches!(serde_json::from_value::<SessionRequest>(serde_json::to_value(request).unwrap()),
                Ok(SessionRequest::InspectProfileStorage {slot: observed}) if observed==slot));
        }
        for slot in [0,6,u32::MAX] {
            assert_eq!(validate_request(&SessionRequest::InspectProfileStorage {slot}).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectProfileStorage"}"#).is_err());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectProfileStorage","slot":1,"path":"outside"}"#).is_err());
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectStorage"}"#),Ok(SessionRequest::InspectStorage)));
    }
    #[cfg(windows)]
    #[test]
    fn selected_profile_receipt_scope_is_optional_and_strict() {
        for operation in ["OpenProfileFolder", "InspectSelectiveInventory", "ReviewSelectiveInventory"] {
            let mut legacy = serde_json::json!({"operation":operation,"slot":3});
            if operation == "InspectSelectiveInventory" { legacy["category"] = serde_json::json!("Worlds"); }
            if operation == "ReviewSelectiveInventory" {
                legacy["selections"] = serde_json::json!([{"category":"Worlds","root":"minecraftWorlds","name":"one","identity":"a".repeat(64)}]);
            }
            let request: SessionRequest = serde_json::from_value(legacy.clone()).unwrap();
            assert!(validate_request(&request).is_ok());
            assert_eq!(serde_json::to_value(request).unwrap(), legacy);
            for selected in [false, true] {
                let mut wire = legacy.clone();
                wire["selectedProfile"] = serde_json::json!(selected);
                let request: SessionRequest = serde_json::from_value(wire.clone()).unwrap();
                assert!(validate_request(&request).is_ok());
                assert_eq!(serde_json::to_value(request).unwrap(), wire);
            }
            let mut invalid = legacy.clone();
            invalid["selectedProfile"] = serde_json::json!("true");
            assert!(serde_json::from_value::<SessionRequest>(invalid).is_err());
            legacy["selectedProfile"] = serde_json::json!(true);
            legacy["slot"] = serde_json::json!(6);
            assert!(validate_request(&serde_json::from_value(legacy).unwrap()).is_err());
        }
        assert!(serde_json::from_value::<SessionRequest>(serde_json::json!({"operation":"Inspect","selectedProfile":true})).is_err());
        assert!(serde_json::from_value::<SessionRequest>(serde_json::json!({"operation":"Launch","slot":3,"selectedProfile":true})).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn selected_mutation_storage_scope_is_optional_and_strict() {
        for operation in ["ClearProfileCache","ResetGameSettings","RestoreGameSettings","ClearProfileData"] {
            let mut legacy=serde_json::json!({"operation":operation,"slot":3});
            if operation=="ClearProfileData" {
                legacy["confirmationToken"]=serde_json::json!("a".repeat(64));
                legacy["confirmationName"]=serde_json::json!("Minecraft - 3");
            }
            let request:SessionRequest=serde_json::from_value(legacy.clone()).unwrap();
            assert!(validate_request(&request).is_ok());
            assert_eq!(serde_json::to_value(request).unwrap(),legacy);
            let mut selected=legacy.clone();
            selected["selectedStorage"]=serde_json::json!(true);
            let request:SessionRequest=serde_json::from_value(selected.clone()).unwrap();
            assert!(validate_request(&request).is_ok());
            assert_eq!(serde_json::to_value(request).unwrap(),selected);
            selected["selectedStorage"]=serde_json::json!("true");
            assert!(serde_json::from_value::<SessionRequest>(selected).is_err());
            legacy["slot"]=serde_json::json!(6);
            legacy["selectedStorage"]=serde_json::json!(true);
            let invalid:SessionRequest=serde_json::from_value(legacy).unwrap();
            assert_eq!(validate_request(&invalid).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
        assert!(serde_json::from_value::<SessionRequest>(serde_json::json!({"operation":"InspectProfileStorage","slot":3,"selectedStorage":true})).is_err());
    }
    #[test]
    fn settings_report_describes_the_reviewed_policy_without_starting_a_controller() {
        let policy = SettingsReport::policy(false, None).unwrap();
        let saved = super::super::manager_settings::load_performance().unwrap();
        assert_eq!((policy.active_fps, policy.background_fps, policy.render_distance_chunks), (saved.active_fps, saved.background_fps, 12));
        let pacing = include_str!("../../../../../tooling/acceptance/windows/bootstrap-ordering/frame-pacing.h");
        let initial = super::super::manager_settings::PerformanceSettings::default();
        assert!(pacing.contains(&format!(
            "activeFps = {}, backgroundFps = {}", initial.active_fps, initial.background_fps
        )), "The first-run manager FPS limits must match the reviewed native pacer");
        let snapshot = SessionSnapshot::stopped();
        assert!(!serde_json::to_value(&snapshot).unwrap().as_object().unwrap().contains_key("settings"));
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"ApplyPreset"}"#).is_ok());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectSettings"}"#).is_ok());
        #[cfg(windows)]
        assert_eq!(request(SessionRequest::SavePerformance {active_fps:0,background_fps:10}).unwrap_err().kind(),io::ErrorKind::InvalidInput);
    }
    #[cfg(windows)]
    #[test]
    fn existing_incompatible_controller_gate_is_distinguished_from_absence() {
        use std::os::windows::io::{FromRawHandle, OwnedHandle};
        let name = format!(
            "Local\\LazyTester-disposable-gate-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        assert!(!existing_controller_gate(&name).unwrap());
        let wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let raw = unsafe {
            windows_sys::Win32::System::Threading::CreateMutexW(std::ptr::null(), 0, wide.as_ptr())
        };
        assert!(!raw.is_null());
        let handle = unsafe { OwnedHandle::from_raw_handle(raw as _) };
        assert!(existing_controller_gate(&name).unwrap());
        drop(handle);
        assert!(!existing_controller_gate(&name).unwrap());
    }
    #[cfg(windows)]
    #[test]
    fn passive_idle_hint_never_substitutes_for_pending_or_uncertain_inspect() {
        assert!(idle_hint_needs_read(true, || panic!("unsettled bypasses hint"), || panic!("no candidate scan")).unwrap());
        assert!(idle_hint_needs_read(false, || Ok(true), || panic!("existing controller requires full read")).unwrap());
        assert!(idle_hint_needs_read(false, || Ok(false), || Ok(true)).unwrap());
        let mut checks=0;
        assert!(idle_hint_needs_read(false, || {checks+=1;Ok(checks==2)}, || Ok(false)).unwrap());
        assert_eq!(checks,2);
        assert!(!idle_hint_needs_read(false, || Ok(false), || Ok(false)).unwrap());
        assert!(idle_hint_needs_read(false, || Ok(false), || Err(io::Error::other("snapshot failed"))).is_err());
        assert!(idle_hint_needs_read(false, || Err(io::Error::other("gate unavailable")), || Ok(false)).is_err());
    }
    #[test]
    fn missing_controller_inspect_and_settings_require_no_observed_games() {
        // Pure admission fixture: no Windows inventory, controller, game or
        // settings mutation. Each fallback operation shares this same gate.
        assert!(require_absent_controller_idle(0).is_ok());
        for observed in [1, 2, 5, 16] {
            let error = require_absent_controller_idle(observed).unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
            assert!(error.to_string().contains("per-slot state is unknown"));
        }
    }

    #[test]
    fn missing_controller_refuses_false_close_when_native_game_candidates_exist() {
        // Full manager shutdown may confirm an already-idle host, but a
        // specific slot needs its retained controller even when none is seen.
        assert!(absent_controller_close_reply(&SessionRequest::CloseAll, 0).is_ok());
        for slot in 1..=5 {
            let selected = SessionRequest::CloseOne {slot};
            assert!(absent_controller_close_reply(&selected, 0).unwrap_err().contains("No retained Minecraft controller"));
            assert!(absent_controller_close_reply(&selected, 1).unwrap_err().contains("no compatible Lazy Tester"));
        }
        assert!(absent_controller_close_reply(&SessionRequest::CloseAll, 1).is_err());
        assert!(absent_controller_close_reply(&SessionRequest::CloseAll, 5).is_err());
        // The established serialized operation remains unchanged.
        assert!(matches!(serde_json::from_str::<SessionRequest>(
            r#"{"operation":"CloseOne","slot":2}"#
        ), Ok(SessionRequest::CloseOne {slot:2})));
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn legacy_mc02_optional_batch_origin_preserves_one_shot_wire_and_rejects_bad_pin() {
        // Pure serde fixture. No controller, native IPC, Windows or game.
        let old=serde_json::json!({"operation":"Mc02Input",
            "slot":2,"keys":0,"dx":0,"dy":0,"ttl_ms":1,"issued_at_ms":100});
        let decoded:SessionRequest=serde_json::from_value(old.clone()).unwrap();
        assert!(matches!(decoded,SessionRequest::Mc02Input{
            batch_origin:None,..}));
        let one_shot=SessionRequest::Mc02Input{slot:2,keys:0,dx:0,dy:0,
            ttl_ms:1,issued_at_ms:100,batch_origin:None};
        assert_eq!(serde_json::to_value(one_shot).unwrap(),old,
            "older legacy one-shot IPC shape must remain unchanged");
        let pin=Mc02BatchOrigin{pid:202,birth:134_000_000_000_000_000,
            session:1};
        let pinned=SessionRequest::Mc02Input{slot:2,keys:1,dx:0,dy:0,
            ttl_ms:350,issued_at_ms:101,batch_origin:Some(pin)};
        let wire=serde_json::to_value(pinned).unwrap();
        assert!(matches!(serde_json::from_value::<SessionRequest>(
            wire.clone()).unwrap(),SessionRequest::Mc02Input{
            batch_origin:Some(found),..} if found==pin));
        for wrong in [
            serde_json::json!({"pid":0,"birth":pin.birth,"session":1}),
            serde_json::json!({"pid":pin.pid,"birth":0,"session":1}),
            serde_json::json!({"pid":pin.pid,"birth":pin.birth,"session":0}),
        ] {
            let invalid=Mc02BatchOrigin{
                pid:wrong["pid"].as_u64().unwrap() as u32,
                birth:wrong["birth"].as_u64().unwrap(),
                session:wrong["session"].as_u64().unwrap() as u32,
            };
            assert!(!invalid.valid());
        }
        let mut extra=wire;
        extra["batch_origin"]["unreviewed_owner"]=serde_json::json!(true);
        assert!(serde_json::from_value::<SessionRequest>(extra).is_err(),
            "private expected-process wire must deny unknown fields");
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn private_acquire_wire_requires_mode_epoch_and_recovery_intent() {
        // Reject old unfenced serialized requests BEFORE controller mutation.
        let valid=serde_json::json!({
            "operation":"AiLeaseAcquire",
            "slots":[2,3,4,5],
            "owner_id":"a".repeat(32),
            "mode":"human-plus-four-ai",
            "lease_epoch":"1".repeat(32),
            "neutral_only":false
        });
        assert!(matches!(serde_json::from_value::<SessionRequest>(valid.clone()),
            Ok(SessionRequest::AiLeaseAcquire {
                mode:AgentControlMode::HumanPlusFourAi,
                neutral_only:false,..})));
        for required in ["mode","owner_id","lease_epoch","neutral_only"] {
            let mut old=valid.clone();
            old.as_object_mut().unwrap().remove(required);
            assert!(serde_json::from_value::<SessionRequest>(old).is_err());
        }
        let mut bad=valid.clone();
        bad["mode"]=serde_json::json!("unrecognized");
        assert!(serde_json::from_value::<SessionRequest>(bad).is_err());
        let mut five=valid.clone();
        five["mode"]=serde_json::json!("five-ai");
        five["slots"]=serde_json::json!([1,2,3,4,5]);
        assert!(matches!(serde_json::from_value::<SessionRequest>(five),
            Ok(SessionRequest::AiLeaseAcquire {
                mode:AgentControlMode::FiveAi,..})));
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn private_input_receipt_is_not_a_session_roster_or_status_report() {
        let receipt=SessionSnapshot::empty_receipt();
        assert!(receipt.is_input_receipt());
        assert_eq!(serde_json::to_value(&receipt).unwrap(),
            serde_json::json!({"schema":1,"profiles":[],"storage":null}));
        let roundtrip:SessionSnapshot=serde_json::from_value(
            serde_json::to_value(&receipt).unwrap()).unwrap();
        assert!(roundtrip.is_input_receipt());
        assert!(!SessionSnapshot::stopped().is_input_receipt());
        let mut wrong=receipt;
        wrong.schema=2;
        assert!(!wrong.is_input_receipt());
        assert!(!SessionRequest::Inspect.expects_input_receipt());
        assert!(!SessionRequest::LaunchMc02Experiment{slot:2}.expects_input_receipt());
        assert!(!SessionRequest::LaunchAiExperiment{slot:2,
            mode:AgentControlMode::HumanPlusFourAi}.expects_input_receipt());
        assert!(SessionRequest::AiLeasedInput{slot:2,keys:0,dx:0,dy:0,
            ttl_ms:1,issued_at_ms:1,owner_id:"a".repeat(32),
            lease_epoch:"b".repeat(32)}.expects_input_receipt());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn grouped_ai_input_admission_rejects_ambiguous_and_expired_deadlines() {
        // Pure source-authored fixture; no IPC, game windows or input sent.
        let frame=|slot|AiInputFrame{slot,keys:1,dx:0,dy:0,ttl_ms:350};
        for slots in [vec![2,3,4,5],vec![1,2,3,4,5]] {
            let valid=slots.into_iter().map(frame).collect::<Vec<_>>();
            assert!(validate_ai_input_group(&valid,1000,1150).is_ok());
            assert!(validate_ai_input_group(&valid,1000,1000).is_ok());
            assert!(validate_ai_input_group(&valid,1000,1151).is_err());
            assert!(validate_ai_input_group(&valid,1100,1099).is_err());
            assert!(validate_ai_input_group(&valid,0,100).is_err());
        }
        for slots in [vec![],vec![2,2],vec![3,2],vec![1,2,3,4,5,6],
            vec![0],vec![6]] {
            let invalid=slots.into_iter().map(frame).collect::<Vec<_>>();
            assert!(validate_ai_input_group(&invalid,1000,1100).is_err());
        }
        let mut invalid=frame(2);
        invalid.keys=3; // conflicting W/S must not enter the actor
        assert!(validate_ai_input_group(&[invalid],1000,1100).is_err());
        invalid=frame(2);
        invalid.ttl_ms=1;
        assert!(validate_ai_input_group(&[invalid],1000,1100).is_err());
        let json=r#"{"operation":"AiInputGroup","frames":[{"slot":2,
            "keys":1,"dx":0,"dy":0,"ttl_ms":350}],
            "issued_at_ms":1000,"deadline_at_ms":1100,
            "owner_id":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "lease_epoch":"11111111111111111111111111111111"}"#;
        assert!(matches!(serde_json::from_str::<SessionRequest>(json),
            Ok(SessionRequest::AiInputGroup{..})));
        let original:serde_json::Value=serde_json::from_str(json).unwrap();
        for required in ["owner_id","lease_epoch"] {
            let mut old=original.clone();
            old.as_object_mut().unwrap().remove(required);
            // Remove the actual JSON object field, not a textual
            // substring that might not exist due to whitespace.
            assert!(serde_json::from_value::<SessionRequest>(old).is_err());
        }
        assert!(serde_json::from_str::<SessionRequest>(
            &json.replace("\"ttl_ms\":350", "\"ttl_ms\":350,\"pid\":123")
        ).is_err());
    }
    #[cfg(feature="mc02-native-input")]
    #[test]
    fn per_client_input_admission_validates_five_slots_and_opposing_controls() {
        // Pure source fixture: does not create a retained controller, process
        // or input section and is NOT Minecraft gameplay evidence.
        for slot in [0,6,u32::MAX] {
            assert_eq!(admit_native_input_frame(slot,1,0,0,150)
                .unwrap_err().kind(),io::ErrorKind::InvalidInput);
            assert_eq!(admit_native_input_frame(slot,0,0,0,1)
                .unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
        for slot in 1..=5 {
            assert_eq!(admit_native_input_frame(slot,1,0,0,150).unwrap(),
                NativeInputIntent::Active);
            assert_eq!(admit_native_input_frame(slot,0,0,0,1).unwrap(),
                NativeInputIntent::ReleaseAll);
        }
        for keys in [3,12,15,255,256] {
            assert_eq!(admit_native_input_frame(2,keys,0,0,150)
                .unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
        for (keys,dx,dy,ttl) in [(1,128,0,150),(1,0,-128,150),
                                 (1,0,0,0),(1,0,0,501)] {
            assert!(admit_native_input_frame(2,keys,dx,dy,ttl).is_err());
        }
        assert_eq!(admit_native_input_frame(2,0,0,0,1).unwrap(),
            NativeInputIntent::ReleaseAll);
        assert_eq!(admit_native_input_frame(2,0,10,-10,150).unwrap(),
            NativeInputIntent::Active);
        assert_eq!(admit_native_input_frame(2,1|4|16|64,0,0,150).unwrap(),
            NativeInputIntent::Active);
    }
    #[cfg(windows)]
    #[test]
    fn agent_target_admission_and_receipt_are_exact_slot_only() {
        for slot in [0,6,u32::MAX] {
            assert!(validate_request(&SessionRequest::InspectAgentTarget{slot}).is_err());
        }
        for slot in 1..=5 {
            let request=SessionRequest::InspectAgentTarget{slot};
            assert!(request.is_agent_target());
            assert!(validate_request(&request).is_ok());
            assert!(!can_reconcile_pending_transport(&request));
            let mut snapshot=SessionSnapshot::stopped();
            assert!(!snapshot.is_agent_target_receipt(slot));
            snapshot.profiles.retain(|p|p.slot==slot);
            assert!(snapshot.is_agent_target_receipt(slot));
            assert!(!snapshot.is_agent_target_receipt(if slot==1{2}else{1}));
        }
        assert!(!SessionRequest::Inspect.is_agent_target());
        assert!(serde_json::from_str::<SessionRequest>(r#"{"operation":"InspectAgentTarget","slot":2,"pid":1}"#).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn full_inspect_reconciliation_refuses_partial_duplicate_and_unrelated_receipts() {
        let full=SessionSnapshot::stopped();
        assert!(full.is_full_inspect_receipt());
        let mut partial=full.clone();
        partial.profiles.retain(|p|p.slot==2);
        assert!(partial.is_agent_target_receipt(2));
        assert!(!partial.is_full_inspect_receipt());
        let mut duplicate=full.clone();
        duplicate.profiles[4].slot=4;
        assert!(!duplicate.is_full_inspect_receipt());
        let mut reordered=full.clone();
        reordered.profiles.swap(0,1);
        assert!(!reordered.is_full_inspect_receipt());
        let mut wrong=full.clone();
        wrong.schema=2;
        assert!(!wrong.is_full_inspect_receipt());
        let mut storage=full;
        storage.storage=Some(Vec::new());
        assert!(!storage.is_full_inspect_receipt());
    }

    #[cfg(windows)]
    #[test]
    fn desktop_target_uses_exact_slot_unless_original_transport_needs_reconciliation() {
        for slot in 1..=5 {
            for uncertain in [false,true] {
                let selected=desktop_observation_request(SessionRequest::InspectAgentTarget{slot},true,uncertain);
                if uncertain {
                    assert!(matches!(selected,SessionRequest::Inspect));
                } else {
                    assert!(matches!(selected,SessionRequest::InspectAgentTarget{slot:observed} if observed==slot));
                }
                // The agent's own selected observation never reconciles.
                let agent=desktop_observation_request(SessionRequest::InspectAgentTarget{slot},false,uncertain);
                assert!(matches!(agent,SessionRequest::InspectAgentTarget{slot:observed} if observed==slot));
            }
        }
    }

    #[test]
    fn agent_request_classes_keep_private_routes_out_of_desktop_and_spawn() {
        for request in [
            SessionRequest::TestingInspect { slot: 1, owner_id: String::new() },
            SessionRequest::TestingReserve { slot: 1, owner_id: String::new(),
                run_id: String::new(), replace_run_id: None },
            SessionRequest::TestingFinalize { slot: 1, owner_id: String::new(),
                run_id: String::new(), reported_status: String::new(), report_digest: String::new() },
        ] {
            assert!(request.is_agent_report());
            assert!(!request.is_experimental_input());
        }
        for request in [SessionRequest::Inspect, SessionRequest::Launch { slot: 1 }] {
            assert!(!request.is_agent_report());
            assert!(!request.is_experimental_input());
        }
        #[cfg(feature="mc02-native-input")]
        for request in [
            SessionRequest::LaunchMc02Experiment { slot: 2 },
            SessionRequest::LaunchAiExperiment { slot: 2, mode: AgentControlMode::HumanPlusFourAi },
            SessionRequest::Mc02Input { slot: 2, keys: 0, dx: 0, dy: 0,
                ttl_ms: 350, issued_at_ms: 1, batch_origin: None },
            SessionRequest::AiLeasedInput { slot: 2, keys: 0, dx: 0, dy: 0,
                ttl_ms: 350, issued_at_ms: 1, owner_id: String::new(), lease_epoch: String::new() },
            SessionRequest::AiLeaseAcquire { slots: vec![2], owner_id: String::new(),
                mode: AgentControlMode::HumanPlusFourAi, lease_epoch: String::new(), neutral_only: true },
            SessionRequest::AiLeaseRelease { slots: vec![2], owner_id: String::new(), lease_epoch: String::new() },
            SessionRequest::AiInputGroup { frames: vec![], issued_at_ms: 1, deadline_at_ms: 2,
                owner_id: String::new(), lease_epoch: String::new() },
        ] {
            assert!(request.is_experimental_input());
            assert!(!request.is_agent_report());
        }
    }

    #[test]
    fn protocol_rejects_paths_unknown_operations_and_unbounded_input() {
        for input in [
            r#"{"operation":"Launch","slot":1,"path":"other.exe"}"#,
            r#"{"operation":"Delete","slot":1}"#,
            r#"{"operation":"SavePerformance","activeFps":60,"backgroundFps":5,"path":"other.json"}"#,
            r#"{"operation":"SavePerformance","activeFps":60.5,"backgroundFps":5}"#,
            r#"{"operation":"DisableImprovedInputResponse","slot":1,"enabled":false}"#,
            r#"{"operation":"EnableTextureStreaming","slot":1,"value":1}"#,
        ] {
            assert!(serde_json::from_str::<SessionRequest>(input).is_err());
        }
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"DisableImprovedInputResponse","slot":2}"#), Ok(SessionRequest::DisableImprovedInputResponse {slot:2})));
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"EnableTextureStreaming","slot":4}"#), Ok(SessionRequest::EnableTextureStreaming {slot:4})));
        assert!(matches!(serde_json::from_str::<SessionRequest>(r#"{"operation":"DisableTextureStreaming","slot":4}"#), Ok(SessionRequest::DisableTextureStreaming {slot:4})));
        assert!(read_message(&mut io::Cursor::new(vec![b'x'; 32769])).is_err());
        assert!(read_message(&mut io::Cursor::new(b"{}".to_vec())).is_err());
        assert_eq!(
            SessionSnapshot::stopped()
                .profiles
                .iter()
                .map(|p| p.slot)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
    }

    #[test]
    fn serialized_size_matches_exact_json_and_frame_boundary() {
        use super::session_protocol::serialized_request_size;
        let request = |name: String| SessionRequest::ClearProfileData {
            slot: 1, confirmation_token: "a".repeat(64),
            confirmation_name: name, selected_storage: None,
        };
        for name in ["Minecraft - 1", "quote\" slash\\ newline\n", "界"] {
            let request = request(name.into());
            assert_eq!(serialized_request_size(&request).unwrap(), serde_json::to_vec(&request).unwrap().len());
        }
        let overhead = serialized_request_size(&request(String::new())).unwrap();
        for size in [SESSION_FRAME_LIMIT - 1, SESSION_FRAME_LIMIT, SESSION_FRAME_LIMIT + 1] {
            let request = request("x".repeat(size - overhead));
            assert_eq!(serialized_request_size(&request).unwrap(), size);
            let mut frame = serde_json::to_vec(&request).unwrap();
            frame.push(b'\n');
            assert_eq!(read_message(&mut io::Cursor::new(frame)).is_ok(), size < SESSION_FRAME_LIMIT);
        }
    }

    #[cfg(windows)]
    #[test]
    fn selective_request_admission_retains_serialized_size_limit() {
        let selection = |index: usize| SelectiveSelection {
            category: SelectiveCategory::Worlds, root: "minecraftWorlds".into(),
            name: format!("{index:03}{}", "x".repeat(252)), identity: format!("{index:064x}"),
        };
        let request = |count| SessionRequest::ReviewSelectiveInventory {
            slot: 1, selections: (0..count).map(selection).collect(), selected_profile: None,
        };
        let small = request(1);
        assert!(validate_request(&small).is_ok());
        let oversized = request(128);
        assert!(serde_json::to_vec(&oversized).unwrap().len() >= SESSION_FRAME_LIMIT);
        let error = validate_request(&oversized).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert_eq!(error.to_string(), "Selection exceeds the session request limit");
    }
