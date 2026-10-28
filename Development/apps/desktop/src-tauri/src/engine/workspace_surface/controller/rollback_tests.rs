use super::*;
#[test]
fn final_restoration_is_not_confused_with_late_helper_replies() {
    let (sender,receiver)=mpsc::channel();
    sender.send(Event::Reply(Reply::Placed)).unwrap();
    sender.send(Event::Reply(Reply::Restored)).unwrap();
    sender.send(Event::End(Ok(()))).unwrap();
    assert!(wait_for_restoration_end(&receiver,Duration::from_millis(10)).unwrap().is_ok());

    let (sender,receiver)=mpsc::channel();
    sender.send(Event::Reply(Reply::Restored)).unwrap();
    drop(sender);
    assert!(wait_for_restoration_end(&receiver,Duration::from_millis(10)).is_err());

    let (sender,receiver)=mpsc::channel();
    sender.send(Event::Reply(Reply::Error("Helper interrupted".into()))).unwrap();
    sender.send(Event::End(Ok(()))).unwrap();
    assert!(wait_for_restoration_end(&receiver,Duration::from_millis(10)).unwrap().is_ok());
}

#[test]
fn failed_restoration_may_close_games_only_after_helper_and_reader_are_done() {
    assert!(!close_only_if_helpers_quiesced(false,false));
    assert!(!close_only_if_helpers_quiesced(false,true));
    assert!(!close_only_if_helpers_quiesced(true,false));
    assert!(close_only_if_helpers_quiesced(true,true));
}
#[test]
fn all_five_slot_close_fallbacks_require_only_their_own_finished_helper() {
    for slot in 1..=5 {
        let client=format!("{slot:02}");
        let own=("LAZY_TESTER_MANAGED_SESSION".to_owned(),client.clone());
        assert!(matches_managed_authority(Some(&own),&client));
        assert!(!matches_managed_authority(Some(&own),&format!("{:02}",slot%5+1)));
        let foreign=("EXTERNAL_SESSION".to_owned(),client.clone());
        assert!(!matches_managed_authority(Some(&foreign),&client));
        let mut evidence=(1..=5)
            .map(|value|(value==slot,true)).collect::<Vec<_>>();
        assert!(selected_helper_quiesced(&evidence));
        evidence[slot as usize-1].1=false;
        assert!(!selected_helper_quiesced(&evidence));
        evidence[slot as usize-1].1=true;
        evidence.push((true,true));
        assert!(!selected_helper_quiesced(&evidence));
    }
    assert!(!selected_helper_quiesced(&[]));
    assert!(!selected_helper_quiesced(&[(false,true),(false,false)]));
}
#[test]
fn broken_helper_pipe_cannot_claim_unobserved_recovery() {
    let corrupted = Some("Malformed helper response before Ready".to_owned());
    assert!(reader_terminal_result(corrupted.clone(), Ok(()), false)
        .unwrap_err().contains("Malformed helper"));
    // A verified Restore acknowledgement or identity-checked native
    // fallback remains authoritative even if the earlier pipe broke.
    assert!(reader_terminal_result(corrupted, Ok(()), true).is_ok());
    // Clean EOF before Start/capture is not a forged window restore.
    assert!(reader_terminal_result(None, Ok(()), false).is_ok());
    assert!(reader_terminal_result(None,
        Err("Original window identity changed".into()), true)
        .unwrap_err().contains("identity changed"));
}
#[test]
fn reader_framing_and_json_errors_are_not_clean_eof() {
    assert!(read_line(&mut io::Cursor::new(vec![b'x'; 4097])).is_err());
    assert!(serde_json::from_str::<Reply>("not-json\\n").is_err());
    assert!(read_line(&mut io::Cursor::new(Vec::<u8>::new()))
        .unwrap().is_none());
}
#[test]
fn preview_and_play_refuse_pre_ready_or_terminal_checkpoint() {
    assert!(presentation_checkpoint_unverified(false, false));
    assert!(presentation_checkpoint_unverified(false, true));
    assert!(presentation_checkpoint_unverified(true, true),
        "Terminal helper is not a live preview source");
    assert!(!presentation_checkpoint_unverified(true, false),
        "Nonterminal Ready can still be observed");
}
#[test]
fn failed_peer_checkpoint_blocks_only_same_game_or_managed_slot() {
    let first = OwnedProcess {
        pid: 101, created_file_time: "12345".into(),
        session_id: 1, window_handle: "0x101".into(), responding: true,
    };
    let second = OwnedProcess {
        pid: 102, created_file_time: "12346".into(),
        session_id: 1, window_handle: "0x102".into(), responding: true,
    };
    let first_slot = ("LAZY_TESTER_MANAGED_SESSION".to_owned(), "01".to_owned());
    let second_slot = ("LAZY_TESTER_MANAGED_SESSION".to_owned(), "02".to_owned());
    assert!(!checkpoint_conflicts(&first, Some(&first_slot),
        &second, Some(&second_slot)), "An MC01 failure must not block MC02");
    assert!(checkpoint_conflicts(&first, Some(&first_slot),
        &first, Some(&first_slot)));
    let mut newer_first = first.clone();
    newer_first.created_file_time = "12347".into();
    newer_first.pid = 110;
    assert!(checkpoint_conflicts(&first, Some(&first_slot),
        &newer_first, Some(&first_slot)),
        "A new game cannot bypass its unresolved old launcher checkpoint");
    assert!(!checkpoint_conflicts(&first, None, &newer_first, None));
    let mut same_target = first.clone();
    same_target.responding = false;
    assert!(checkpoint_conflicts(&first, None, &same_target, None));
    let foreign = ("EXTERNAL_SESSION".to_owned(), "01".to_owned());
    assert!(!checkpoint_conflicts(&first, Some(&foreign),
        &newer_first, Some(&first_slot)));
    assert!(checkpoint_conflicts(&first, Some(&foreign),
        &first, Some(&second_slot)),
        "Exact window identity cannot be bypassed by a mismatched label");
}

#[test]
fn hide_noop_requires_recent_helper_receipt_and_exact_hidden_readback() {
    let start = Instant::now();
    let mut probes = 0;
    for (receipt, terminal, helper_alive, elapsed) in [
        (None, false, true, 1),
        (Some(start), true, true, 1),
        (Some(start), false, false, 1),
        (Some(start), false, true, 1000),
    ] {
        assert!(!recent_hidden_receipt(
            receipt, start + Duration::from_millis(elapsed),
            terminal, helper_alive, || { probes += 1; true }));
    }
    assert_eq!(probes, 0,
        "Unverified or terminal helper may not probe a game HWND for fast-path");
    assert!(!recent_hidden_receipt(Some(start),
        start + Duration::from_millis(100), false, true,
        || { probes += 1; false }),
        "A visible or replaced game window requires the helper");
    assert!(recent_hidden_receipt(Some(start),
        start + Duration::from_millis(100), false, true,
        || { probes += 1; true }));
    assert_eq!(probes, 2);
    assert!(!recent_hidden_receipt(Some(start),
        start + HELPER_PASSIVE_HIDE_REASSERT, false, true, || true));
}
#[test]
fn passive_helper_place_skips_only_fresh_acknowledged_same_rectangle() {
    let start = Instant::now();
    let rect = Rectangle { x: 20, y: 30, width: 700, height: 450 };
    assert!(!recent_passive_place(None, rect, start, false));
    let ack = Some((rect, start));
    assert!(recent_passive_place(ack, rect,
        start + Duration::from_millis(10), false));
    for elapsed in [100, 250, 500, 999] {
        assert!(recent_passive_place(ack, rect,
            start + Duration::from_millis(elapsed), false));
    }
    assert!(!recent_passive_place(ack, rect, start, true),
        "An explicit Play activation never skips the helper");
    assert!(!recent_passive_place(ack,
        Rectangle { x: 21, ..rect }, start, false),
        "Even a one-pixel move needs a new ordered Place");
    assert!(!recent_passive_place(ack,
        Rectangle { width: 701, ..rect }, start, false));
    assert!(!recent_passive_place(ack, rect,
        start + HELPER_PASSIVE_PLACE_REASSERT, false),
        "A retained helper must be contacted again after the bounded reassertion");
    // This pure policy alone cannot authorize a no-op. The live
    // parent also verifies child state and exact native HWND/style
    // readback before actually bypassing the helper pipe.
}
#[test]
fn start_ready_must_match_exact_retained_process_and_window() {
    let requested = OwnedProcess {
        pid: 101,
        created_file_time: "134000000000000001".into(),
        session_id: 1,
        window_handle: "0x123".into(),
        responding: true,
    };
    let ready = Saved {
        target: requested.clone(),
        window_thread_id: 17,
        window_class_name: "OGLES".into(),
        style: 0,
        extended_style: 0,
        rect: Rectangle { x: 0, y: 0, width: 640, height: 360 },
        manager_pid: 5,
    };
    assert!(ready_matches_start(&ready, &requested));
    let mut changed = ready.clone();
    changed.target.responding = false;
    assert!(ready_matches_start(&changed, &requested),
        "Responsive state is not a stable identity key");
    changed.target.pid += 1;
    assert!(!ready_matches_start(&changed, &requested));
    let mut changed = ready.clone();
    changed.target.created_file_time = "1".into();
    assert!(!ready_matches_start(&changed, &requested));
    let mut changed = ready.clone();
    changed.target.session_id += 1;
    assert!(!ready_matches_start(&changed, &requested));
    let mut changed = ready.clone();
    changed.target.window_handle = "0x124".into();
    assert!(!ready_matches_start(&changed, &requested));
}
#[test]
fn failed_pre_ready_retirement_requires_both_exits() {
    let failed: Result<(), String> = Err("Ready unavailable".into());
    let passed: Result<(), String> = Ok(());
    let mut probes = 0;
    for (terminal, helper_exited, has_saved) in [
        (None, true, false),
        (Some(&passed), true, false),
        (Some(&failed), false, false),
        (Some(&failed), true, true),
    ] {
        assert!(retire_failed_pre_ready(terminal, helper_exited, has_saved, || {
            probes += 1;
            Ok(true)
        }).is_none());
    }
    assert_eq!(probes, 0, "Never inspect until helper is terminal and exited");
    let live = retire_failed_pre_ready(Some(&failed), true, false, || {
        probes += 1;
        Ok(false)
    }).unwrap().unwrap_err();
    assert_eq!(live.kind(), io::ErrorKind::WouldBlock);
    let uncertain = retire_failed_pre_ready(Some(&failed), true, false, || {
        probes += 1;
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "Original PID unavailable"))
    }).unwrap().unwrap_err();
    assert_eq!(uncertain.kind(), io::ErrorKind::PermissionDenied);
    assert!(retire_failed_pre_ready(Some(&failed), true, false, || {
        probes += 1;
        Ok(true)
    }).unwrap().is_ok());
    assert_eq!(probes, 3);
}
#[test]
fn failed_terminal_retry_requires_dead_helper_and_original_saved_window() {
    let original = OwnedProcess {
        pid: 101, created_file_time: "134000000000000001".into(),
        session_id: 1, window_handle: "0x123".into(), responding: true,
    };
    let saved = Saved {
        target: original, window_thread_id: 17,
        window_class_name: "OGLES".into(),
        style: 0, extended_style: 0,
        rect: Rectangle {x: 0, y: 0, width: 640, height: 360},
        manager_pid: 5,
    };
    let failed: Result<(), String> = Err("Original restore failed".into());
    let success: Result<(), String> = Ok(());
    let mut attempts = 0;
    for (terminal, exited, checkpoint) in [
        (None, true, Some(&saved)),
        (Some(&success), true, Some(&saved)),
        (Some(&failed), false, Some(&saved)),
        (Some(&failed), true, None),
    ] {
        assert!(retry_failed_terminal_restore(terminal, exited, checkpoint, |_| {
            attempts += 1;
            Ok(())
        }).is_none());
    }
    assert_eq!(attempts, 0, "No recovery writes while helper or identity is unknown");
    let error = retry_failed_terminal_restore(
        Some(&failed), true, Some(&saved), |checkpoint| {
            attempts += 1;
            assert_eq!(checkpoint.target.pid, 101);
            Err(io::Error::new(io::ErrorKind::WouldBlock,
                "Original HWND readback differs"))
        },
    ).expect("One existing-checkpoint retry").unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
    assert_eq!(attempts, 1);
    assert!(completed_helper_status(Some(&failed), true).is_err(),
        "A failed retry never promotes the original End error");
    assert!(retry_failed_terminal_restore(Some(&failed), true, Some(&saved),
        |checkpoint| {
            attempts += 1;
            assert_eq!(checkpoint.target.window_handle, "0x123");
            Ok(())
        }).unwrap().is_ok());
    assert_eq!(attempts, 2);
}
#[test]
fn terminal_end_alone_does_not_retire_a_live_helper() {
    let success: Result<(), String> = Ok(());
    let failed: Result<(), String> = Err("Native restore failed".into());
    assert!(completed_helper_status(None, true).is_err());
    assert_eq!(
        completed_helper_status(Some(&success), false).unwrap_err().kind(),
        io::ErrorKind::WouldBlock,
    );
    assert!(completed_helper_status(Some(&success), true).is_ok());
    assert!(completed_helper_status(Some(&failed), true)
        .unwrap_err().to_string().contains("Native restore failed"));
    assert!(completed_helper_status(Some(&failed), false).is_err());
}

#[test]
fn checkpoint_retirement_refuses_missing_or_duplicate_selected_leases() {
    assert_eq!(require_unique_selected_checkpoint(0).unwrap_err().kind(),
        io::ErrorKind::NotFound);
    assert!(require_unique_selected_checkpoint(1).is_ok());
    assert!(require_unique_selected_checkpoint(2).unwrap_err()
        .to_string().contains("Ambiguous Minecraft"));
    assert!(require_unique_selected_checkpoint(5).is_err());
    // Pure policy; no helper, process, native HWND, game or profile.
}
#[test]
fn incomplete_recovery_reports_each_failed_step() {
    let ok = Ok(());
    let parking = Err(io::Error::other("park failed"));
    let restore = Err(io::Error::other("restore failed"));
    assert!(rollback_error(&ok, &ok).is_none());
    assert!(rollback_error(&parking, &ok).unwrap().contains("park failed"));
    assert!(rollback_error(&ok, &restore).unwrap().contains("restore failed"));
    let both = rollback_error(&parking, &restore).unwrap();
    assert!(both.contains("park failed") && both.contains("restore failed"));
}
