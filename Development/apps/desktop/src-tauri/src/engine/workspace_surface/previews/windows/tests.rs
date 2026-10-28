use super::*;
#[test]
fn failed_wake_reconciles_admitted_job_on_next_received_message() {
    // Simulate PostThreadMessageW returning an error AFTER bounded
    // try_send. The next worker message may be SELECT or WM_PAINT,
    // not a WAKE; the production drain helper is message-agnostic.
    for next in [SELECT, WM_PAINT, WM_NULL] {
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.try_send(2u32).unwrap(); // ClearSlot MC02
        assert!(sender.try_send(3).is_err(),
            "The same DWM command queue remains capacity one");
        let mut processed = Vec::new();
        drain_ready_jobs(&receiver, |slot| processed.push(slot));
        assert_eq!(processed, vec![2], "Lost wake retained job on message {next}");
        drain_ready_jobs(&receiver, |_| panic!("Job replayed twice"));
    }
    assert!(!refresh_after_wake(false, false),
        "Control-only cleanup never requires peer DWM rechecks");
}
#[test]
fn unconfirmed_control_cleanup_wakes_once_without_masking_the_error() {
    let mut wakes = 0;
    let failure = control_cleanup_receipt::<()>(
        Err(io::Error::new(io::ErrorKind::WouldBlock,
            "Existing bounded DWM queue is full")),
        || wakes += 1,
    ).unwrap_err();
    assert_eq!(failure.kind(), io::ErrorKind::WouldBlock);
    assert_eq!(wakes, 1,
        "A failed admission must ask the existing worker to inspect cancellation");
    assert!(!refresh_after_wake(false, false),
        "A cleanup-only WAKE never revalidates all surviving DWM peers");
    let accepted = control_cleanup_receipt(Ok(11usize), || wakes += 1).unwrap();
    assert_eq!(accepted, 11);
    assert_eq!(wakes, 1,
        "An acknowledged control job must not post a second WAKE");
    let timeout = control_cleanup_receipt::<()>(
        Err(io::Error::new(io::ErrorKind::TimedOut,
            "Worker still owns its original queued Clear")),
        || wakes += 1,
    ).unwrap_err();
    assert_eq!(timeout.kind(), io::ErrorKind::TimedOut);
    assert_eq!(wakes, 2,
        "A reply timeout keeps its error and still signals epoch cleanup");
}
#[test]
fn clear_and_closeone_invalidate_native_source_admission_before_dwm_queue() {
    // Pure policy used by configure() both before and after native
    // adoption: no original game/helper/Win32 calls are required.
    let issued = [1, 4, 8, 2, 7];
    assert!(epoch_snapshot_matches(&issued, &issued));
    let mut after_closeone = issued;
    after_closeone[1] += 1;
    assert!(!epoch_snapshot_matches(&issued, &after_closeone),
        "CloseOne during adoption must reject the old whole Layout");
    assert!(slot_epoch_matches(&issued, &after_closeone, 3),
        "MC03 peer epoch is unaffected by a canceled MC02 preview");
    assert!(!slot_epoch_matches(&issued, &after_closeone, 2));
    let after_clear = issued.map(|epoch| epoch + 1);
    assert!(!epoch_snapshot_matches(&issued, &after_clear),
        "Full Clear must reject every earlier source admission");
    assert!((1..=5).all(|slot|
        !slot_epoch_matches(&issued, &after_clear, slot)));
    assert!(epoch_snapshot_matches(&after_clear, &after_clear),
        "A newly issued verified Layout may use the current epoch");
}
#[test]
fn queued_preview_reply_releases_owner_before_notification() {
    // The exact production helper, exercised with synthetic channels
    // and a Mutex, never requires a Tauri window or game process.
    let owner = Mutex::new(());
    let (reply, receive) = mpsc::sync_channel(1);
    let result = finish_after_admission(owner.lock().unwrap(), || {
        assert!(owner.try_lock().is_ok(),
            "Another preview caller must access WORKER during reply wait");
        reply.send(Ok(7u32)).unwrap();
        Ok(())
    }, receive, Instant::now() + Duration::from_secs(3)).unwrap();
    assert_eq!(result, 7);
    assert!(owner.try_lock().is_ok());
}
#[test]
fn unacknowledged_layout_only_discards_newly_created_thumbnails() {
    let new = LayoutCreated {
        secondary_slots: vec![2, 4], primary_changed: true,
        ..LayoutCreated::default()
    };
    assert!(!retain_previous_preview(&new, 2));
    assert!(!retain_previous_preview(&new, 4));
    for peer in [1, 3, 5] {
        assert!(retain_previous_preview(&new, peer),
            "Slow MC02 layout must not drop already registered peers");
    }
    assert!(!layout_acknowledged(true, false),
        "Dropped frontend receiver cannot acknowledge new native images");
    assert!(!layout_acknowledged(false, true),
        "An expired/error reply cannot acknowledge new DWM images");
    assert!(layout_acknowledged(true, true));
}
#[test]
fn primary_replacement_admission_rejects_stale_manager_source_and_close() {
    let prior = PreviousImage {
        slot: 5, manager: 51, destination: 0x901, registration: 99,
        viewport: Viewport {x:0.0,y:0.0,width:640.0,height:360.0,pixel_ratio:1.0},
        invalidated: false, applied: None,
    };
    assert!(previous_primary_can_restore(prior, 51, true, true));
    assert!(!previous_primary_can_restore(prior, 52, true, true));
    assert!(!previous_primary_can_restore(prior, 51, false, true),
        "Superseded Minecraft HWND must not be restored");
    assert!(!previous_primary_can_restore(prior, 51, true, false),
        "CloseOne epoch cannot restore its retired primary image");
}
#[test]
fn reused_preview_geometry_and_failed_state_are_rollback_evidence() {
    let original = Viewport {
        x: 20.0, y: 30.0, width: 400.0, height: 300.0, pixel_ratio: 1.0,
    };
    let moved = Viewport {x:21.0, ..original};
    let scaled = Viewport {pixel_ratio:1.5, ..original};
    assert!(!viewport_changed(original, original),
        "Identical geometry never needs a Win32 rollback");
    assert!(viewport_changed(original, moved));
    assert!(viewport_changed(original, scaled));
    let old = PreviousImage {
        slot: 2, manager: 88, destination: 0x120, registration: 42,
        viewport: original, invalidated: true, applied: None,
    };
    let discarded = LayoutCreated {
        reused_secondary: vec![old],
        reused_primary: Some(old),
        ..LayoutCreated::default()
    };
    assert!(!viewport_changed(old.viewport, original));
    assert!(!old.previously_visible(true),
        "An unacknowledged recovery cannot turn an invalidated MC02 visible");
    let visible = PreviousImage {
        invalidated: false,
        applied: Some(DwmAppliedState::Visible(DwmGeometry {
            destination: Rectangle {x:20,y:30,width:400,height:300},
            fitted: Rectangle {x:0,y:0,width:400,height:300},
            source_width: 400, source_height: 300,
        })),
        ..old
    };
    assert!(visible.previously_visible(true));
    assert!(!visible.previously_visible(false));
    let hidden = PreviousImage {
        applied: Some(DwmAppliedState::Hidden), ..visible
    };
    assert!(!hidden.previously_visible(true),
        "A previously hidden tile cannot be shown by an unacknowledged layout");
    for peer in [1, 3, 4, 5] {
        assert!(retain_previous_preview(&discarded, peer));
    }
}
#[test]
fn in_flight_deadline_expiry_is_fail_closed_before_next_tile() {
    let start = Instant::now();
    let deadline = start + Duration::from_secs(3);
    assert!(!layout_expired(deadline, start + Duration::from_millis(1)));
    assert!(!layout_expired(deadline, deadline - Duration::from_millis(1)));
    assert!(layout_expired(deadline, deadline));
    assert!(layout_expired(deadline, deadline + Duration::from_millis(1)));
    // The same production guard is checked before the primary,
    // every secondary and after each native Win32 update.
}
#[test]
fn closing_one_slot_cancels_stale_layout_without_changing_peer_epochs() {
    // The pure epoch comparison is the production worker policy;
    // no real game, HWND, Tauri window, or extra actor is needed.
    let issued = [5, 8, 3, 4, 1];
    let mut observed = issued;
    assert!((1..=5).all(|slot|
        slot_epoch_matches(&issued, &observed, slot)));
    observed[1] += 1; // CloseOne MC02, even when DWM queue is full.
    assert!(!slot_epoch_matches(&issued, &observed, 2));
    for peer in [1, 3, 4, 5] {
        assert!(slot_epoch_matches(&issued, &observed, peer),
            "MC02 cancellation must not erase surviving peers");
    }
    assert!(!slot_epoch_matches(&issued, &observed, 0));
    assert!(!slot_epoch_matches(&issued, &observed, 6));
    // A layout issued after completion can use the newer snapshot;
    // a layout issued before CloseOne cannot reuse the old one.
    let current = observed;
    assert!(slot_epoch_matches(&current, &observed, 2));
    assert!(!slot_epoch_matches(&issued, &observed, 2));
}
#[test]
fn control_jobs_do_not_waste_native_refresh_or_lose_move_events() {
    assert!(!refresh_after_wake(false, false));
    assert!(!refresh_after_wake(false, true));
    assert!(refresh_after_wake(true, false));
    assert!(!refresh_after_wake(true, true));
}
#[test]
fn expired_layout_cannot_reanimate_old_native_preview() {
    let now = Instant::now();
    let deadline = now + Duration::from_secs(3);
    assert!(!layout_expired(deadline, now));
    assert!(!layout_expired(deadline, deadline-Duration::from_millis(1)));
    assert!(layout_expired(deadline, deadline));
    assert!(layout_expired(deadline, deadline+Duration::from_millis(1)));
    // No expiry gate is applied to ClearSlot or CloseOne.
}
#[test]
fn empty_dwm_scene_releases_manager_and_profile_metadata() {
    let game = OwnedProcess {
        pid: 101, created_file_time: "12345".into(),
        session_id: 1, window_handle: "0x101".into(), responding: true,
    };
    let mut scene = Scene {
        window: None, manager: 77,
        profiles: vec![(2, game)],
        thumbnails: Vec::new(), primary: None,
        issued_epochs: snapshot_epochs(),
        layout_in_progress: true,
    };
    scene.refresh();
    assert_eq!(scene.manager, 77,
        "Current Layout must hold context to permit secondary rollback");
    scene.layout_in_progress = false;
    scene.refresh();
    assert_eq!(scene.manager, 0);
    assert!(scene.profiles.is_empty());
    assert!(scene.window.is_none());
}
#[test]
fn redundant_dwm_writes_skip_stable_bursts_but_refresh_real_changes() {
    let now = Instant::now();
    let geometry = DwmGeometry {
        destination: Rectangle {x:10,y:20,width:640,height:360},
        fitted: Rectangle {x:0,y:0,width:640,height:360},
        source_width:1920,source_height:1080,
    };
    let visible = DwmAppliedState::Visible(geometry);
    assert!(dwm_property_write_due(None,visible,now));
    let last = Some((visible,now));
    for millis in [1,8,16,100,750,999] {
        assert!(!dwm_property_write_due(
            last,visible,now+Duration::from_millis(millis)),
            "Identical move/focus bursts need no additional DWM writes");
    }
    assert!(dwm_property_write_due(last,visible,now+DWM_PROPERTY_REASSERT));
    let moved = DwmAppliedState::Visible(DwmGeometry {
        destination: Rectangle {x:11,..geometry.destination},..geometry
    });
    assert!(dwm_property_write_due(
        last,moved,now+Duration::from_millis(5)));
    let resized = DwmAppliedState::Visible(DwmGeometry {
        destination: Rectangle {width:700,..geometry.destination},..geometry
    });
    assert!(dwm_property_write_due(
        last,resized,now+Duration::from_millis(5)));
    let source_resized = DwmAppliedState::Visible(DwmGeometry {
        source_width:1600,..geometry
    });
    assert!(dwm_property_write_due(
        last,source_resized,now+Duration::from_millis(5)));
    assert!(dwm_property_write_due(
        last,DwmAppliedState::Hidden,now+Duration::from_millis(5)));
    let hidden = Some((DwmAppliedState::Hidden,now));
    assert!(!dwm_property_write_due(
        hidden,DwmAppliedState::Hidden,now+Duration::from_millis(5)));
    assert!(dwm_property_write_due(
        hidden,visible,now+Duration::from_millis(5)));
}
#[test]
fn failed_tile_stays_hidden_until_verified_layout_result() {
    assert!(display_with_current_preview_receipt(true, false));
    assert!(!display_with_current_preview_receipt(true, true));
    assert!(!display_with_current_preview_receipt(false, false));
    assert!(!display_with_current_preview_receipt(false, true));
    let mut invalidated = false;
    assert!(take_invalidation_notice(&mut invalidated));
    assert!(!display_with_current_preview_receipt(true, invalidated),
        "A native update is not a fresh success receipt");
    invalidated = false; // Only a successful layout may rearm.
    assert!(display_with_current_preview_receipt(true, invalidated));
}
#[test]
fn async_invalidation_is_source_pinned_and_coalesced_per_layout() {
    let target = OwnedProcess {
        pid: 102, created_file_time: "134000000000000001".into(),
        session_id: 1, window_handle: "0x124".into(), responding: true,
    };
    let message = preview_invalidation_message(
        2, &target, "Native preview refresh unverified",
    ).unwrap();
    let encoded = message.strip_prefix(PREVIEW_INVALIDATED_PREFIX).unwrap();
    let decoded: serde_json::Value = serde_json::from_str(encoded).unwrap();
    assert_eq!(decoded["schema"], 1);
    assert_eq!(decoded["slot"], 2);
    assert_eq!(decoded["process"]["pid"], target.pid);
    assert_eq!(decoded["process"]["createdFileTime"], target.created_file_time);
    assert_eq!(decoded["process"]["windowHandle"], target.window_handle);
    assert_eq!(decoded["reason"], "Native preview refresh unverified");
    let mut notified = false;
    assert!(take_invalidation_notice(&mut notified));
    assert!(!take_invalidation_notice(&mut notified),
        "An idle refresh must never flood one failure per frame");
    notified = false; // Only a new validated layout rearms.
    assert!(take_invalidation_notice(&mut notified));
}
#[test]
fn refresh_errors_hide_stale_images_even_on_would_block() {
    let mut hidden = 0;
    for kind in [io::ErrorKind::WouldBlock, io::ErrorKind::TimedOut,
                 io::ErrorKind::NotFound, io::ErrorKind::Other] {
        let failure = hide_on_failed_update(
            Err(io::Error::new(kind, "Native preview is unverified")),
            || hidden += 1);
        assert_eq!(failure.unwrap_err().kind(), kind);
    }
    assert_eq!(hidden, 4);
    assert!(hide_on_failed_update(Ok(()), || hidden += 1).is_ok());
    assert_eq!(hidden, 4, "Verified preview must not be hidden");
}
#[test]
fn hidden_own_windows_register_update_and_release_dwm_without_desktop_capture() {
    // Fixtures remain offscreen and are never focused. This proves DWM interop only,
    // not live pixels, Minecraft rendering, gameplay or background-host behavior.
    unsafe {
        let class = WNDCLASSW {
            lpfnWndProc: Some(procedure),
            hInstance: GetModuleHandleW(ptr::null()),
            lpszClassName: CLASS.as_ptr(),
            ..mem::zeroed()
        };
        assert!(
            RegisterClassW(&class) != 0 || GetLastError() == ERROR_CLASS_ALREADY_EXISTS
        );
        let make = || {
            CreateWindowExW(
                0,
                CLASS.as_ptr(),
                CLASS.as_ptr(),
                WS_OVERLAPPEDWINDOW,
                -32000,
                -32000,
                800,
                600,
                ptr::null_mut(),
                ptr::null_mut(),
                GetModuleHandleW(ptr::null()),
                ptr::null(),
            )
        };
        let manager = make();
        let source = make();
        assert!(!manager.is_null() && !source.is_null());
        let mut created: FILETIME = mem::zeroed();
        let mut exit = created;
        let mut kernel = created;
        let mut user = created;
        assert_ne!(
            windows_sys::Win32::System::Threading::GetProcessTimes(
                windows_sys::Win32::System::Threading::GetCurrentProcess(),
                &mut created,
                &mut exit,
                &mut kernel,
                &mut user
            ),
            0
        );
        let mut session = 0;
        assert_ne!(
            windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId(
                std::process::id(),
                &mut session
            ),
            0
        );
        let target = OwnedProcess {
            pid: std::process::id(),
            created_file_time: (((created.dwHighDateTime as u64) << 32)
                | created.dwLowDateTime as u64)
                .to_string(),
            session_id: session,
            window_handle: format!("0x{:x}", source as usize),
            responding: true,
        };
        let tile = PreviewTile {
            slot: 5,
            viewport: Viewport {
                x: 5.0,
                y: 5.0,
                width: 150.0,
                height: 85.0,
                pixel_ratio: windows_sys::Win32::UI::HiDpi::GetDpiForWindow(manager) as f64
                    / 96.0,
            },
        };
        let mut thumbnail =
            Thumbnail::create(manager as usize as u64, &tile, target.clone()).unwrap();
        assert!(thumbnail.destination_owned());
        thumbnail.refresh_verified(false).unwrap();
        let mut size: SIZE = mem::zeroed();
        assert!(DwmQueryThumbnailSourceSize(thumbnail.thumbnail, &mut size) >= 0);
        assert!(size.cx > 0 && size.cy > 0);
        assert_eq!(IsWindowVisible(thumbnail.window), 0);
        // A changed slot identity invalidates the original HWND
        // for all DWM updates, even before the worker sees a new
        // manager layout. Restore this synthetic fixture thereafter.
        SetWindowLongPtrW(thumbnail.window, GWLP_USERDATA, 0);
        assert!(!thumbnail.destination_owned());
        assert_eq!(thumbnail.refresh_verified(true).unwrap_err().kind(),
            io::ErrorKind::NotFound);
        SetWindowLongPtrW(thumbnail.window, GWLP_USERDATA, tile.slot as isize);
        assert!(thumbnail.destination_owned());
        ShowWindow(manager, SW_SHOWNOACTIVATE);
        let primary_tile = PreviewTile { slot: 5, viewport: Viewport {
            x: 10.0, y: 100.0, width: 640.0, height: 360.0, ..tile.viewport } };
        let primary = Thumbnail::create(manager as usize as u64, &primary_tile, target.clone()).unwrap();
        // Production Scene carries the exact source inventory. The
        // rollback source-authority guard must be tested with it,
        // never weakened to accommodate an empty fixture roster.
        let mut scene=Scene{window:None,manager:manager as usize as u64,profiles:vec![(tile.slot,target)],thumbnails:Vec::new(),primary:Some(primary),issued_epochs:snapshot_epochs(),layout_in_progress:false};
        assert_ne!(GetForegroundWindow(),manager);
        assert!(scene.visibility(),"Visible monitoring must survive focus moving elsewhere");
        thumbnail.update(scene.visibility()).unwrap();
        assert_ne!(IsWindowVisible(thumbnail.window),0);
        // Offscreen DWM fixture: a reported failure does not
        // re-show its thumbnail after ordinary native WAKE events.
        thumbnail.invalidation_notified = true;
        for _ in 0..2 {
            thumbnail.refresh_verified(display_with_current_preview_receipt(
                scene.visibility(), thumbnail.invalidation_notified,
            )).unwrap();
            assert_eq!(IsWindowVisible(thumbnail.window),0,
                "Invalidated tile reappeared without a fresh layout receipt");
        }
        thumbnail.invalidation_notified = false;
        thumbnail.refresh_verified(display_with_current_preview_receipt(
            scene.visibility(), thumbnail.invalidation_notified,
        )).unwrap();
        assert_ne!(IsWindowVisible(thumbnail.window),0);
        for (x,y) in [(-31000,-31500),(-31500,-31000),(-32000,-32000)] {
            assert_ne!(SetWindowPos(manager,ptr::null_mut(),x,y,0,0,
                SWP_NOSIZE|SWP_NOZORDER|SWP_NOACTIVATE),0);
            scene.refresh();
            let primary = scene.primary.as_ref().unwrap();
            assert_ne!(IsWindowVisible(primary.window),0);
            let expected = native::rectangle(manager as usize as u64,primary_tile.viewport).unwrap();
            let mut observed: RECT = mem::zeroed();
            assert_ne!(GetWindowRect(primary.window,&mut observed),0);
            assert_eq!((observed.left,observed.top,observed.right-observed.left,observed.bottom-observed.top),
                (expected.x,expected.y,expected.width,expected.height),"Main DWM image lagged its manager");
            assert_eq!(GetWindow(primary.window,GW_OWNER),manager);
            let region = CreateRectRgn(0,0,0,0);
            assert_eq!(GetWindowRgn(manager,region),ERROR,"Preview cut a desktop hole");
            DeleteObject(region);
        }
        ShowWindow(manager,SW_MINIMIZE);
        assert!(!scene.visibility());
        scene.refresh();
        assert_eq!(IsWindowVisible(scene.primary.as_ref().unwrap().window),0);
        thumbnail.update(scene.visibility()).unwrap();
        assert_eq!(IsWindowVisible(thumbnail.window),0);
        // A nonactivating restore should show the same verified
        // observer again. Losing input focus is not source failure.
        ShowWindow(manager, SW_SHOWNOACTIVATE);
        assert!(scene.visibility(),"Restored manager remains a valid DWM destination");
        scene.refresh();
        assert_ne!(IsWindowVisible(scene.primary.as_ref().unwrap().window),0);
        assert!(scene.selection_is_verified(tile.slot),
            "A current owner/source must permit native thumbnail select");
        thumbnail.refresh_verified(scene.visibility()).unwrap();
        assert_ne!(IsWindowVisible(thumbnail.window),0);
        // An abandoned layout changed an ALREADY REGISTERED tile.
        // Rewind to its original live manager-relative geometry
        // and previously reported native visibility, leaving the
        // separately verified primary observer intact.
        let old_viewport = thumbnail.viewport;
        let moved_viewport = Viewport {
            x: old_viewport.x + 24.0, ..old_viewport
        };
        thumbnail.viewport = moved_viewport;
        thumbnail.refresh_verified(true).unwrap();
        scene.thumbnails.push(thumbnail);
        let mut previous = PreviousImage::capture(&scene.thumbnails[0]);
        previous.viewport = old_viewport;
        assert!(previous.same_registration(&scene.thumbnails[0]));
        let wrong_manager = PreviousImage {manager: 9, ..previous};
        assert!(!wrong_manager.same_registration(&scene.thumbnails[0]));
        let wrong_registration = PreviousImage {registration: previous.registration + 1, ..previous};
        assert!(!wrong_registration.same_registration(&scene.thumbnails[0]));
        scene.discard_unacknowledged_new(&mut LayoutCreated {
            reused_secondary: vec![previous],
            ..LayoutCreated::default()
        });
        assert_eq!(scene.thumbnails.len(), 1);
        let restored = &scene.thumbnails[0];
        assert_eq!(restored.viewport.x, old_viewport.x);
        let expected = native::rectangle(
            manager as usize as u64, old_viewport).unwrap();
        let mut actual: RECT = mem::zeroed();
        assert_ne!(GetWindowRect(restored.window, &mut actual), 0);
        assert_eq!((actual.left,actual.top,actual.right-actual.left,
                   actual.bottom-actual.top),
                   (expected.x,expected.y,expected.width,expected.height));
        assert_ne!(IsWindowVisible(restored.window), 0);
        // A failed tile must not be resurrected by an abandoned
        // layout, even when its requested viewport is unchanged.
        scene.thumbnails[0].invalidation_notified = false;
        scene.discard_unacknowledged_new(&mut LayoutCreated {
            reused_secondary: vec![PreviousImage {
                invalidated: true,
                applied: Some(DwmAppliedState::Hidden),
                ..previous
            }],
            ..LayoutCreated::default()
        });
        assert!(scene.thumbnails[0].invalidation_notified);
        assert_eq!(IsWindowVisible(scene.thumbnails[0].window), 0);
        // A newly registered, unacknowledged tile is still removed.
        scene.discard_unacknowledged_new(&mut LayoutCreated {
            secondary_slots: vec![tile.slot],
            ..LayoutCreated::default()
        });
        assert!(scene.thumbnails.is_empty());
        assert!(scene.primary.is_some(),
            "Rolling back a secondary cannot discard a healthy primary");
        // If a new session inventory replaces this source, even
        // a still-alive old HWND cannot become the active observer.
        // No game process is actually changed by this fixture.
        let retired = Thumbnail::create(
            manager as usize as u64, &tile,
            scene.primary.as_ref().unwrap().target.clone(),
        ).unwrap();
        scene.thumbnails.push(retired);
        let pinned = PreviousImage::capture(&scene.thumbnails[0]);
        scene.profiles.clear();
        assert!(!scene.selection_is_verified(tile.slot),
            "Retired source must not emit a queued SELECT after cancellation");
        scene.discard_unacknowledged_new(&mut LayoutCreated {
            reused_secondary: vec![pinned],
            ..LayoutCreated::default()
        });
        assert!(scene.thumbnails.is_empty(),
            "Retired source must not survive a rejected Layout");
        assert!(scene.primary.is_some(),
            "Retiring one secondary observer must not kill its primary peer");
        // Mixed partial failure: a proposed Layout removes one
        // accepted secondary and swaps primary, but its reply is
        // lost. The SAME transaction must restore both originals.
        let owned_target = scene.primary.as_ref().unwrap().target.clone();
        let previous_tile = PreviewTile {slot:4, viewport:tile.viewport};
        scene.profiles.push((4, owned_target.clone()));
        let mut previous_peer = Thumbnail::create(
            manager as usize as u64, &previous_tile, owned_target.clone(),
        ).unwrap();
        previous_peer.refresh_verified(true).unwrap();
        let peer_registration = previous_peer.thumbnail;
        scene.thumbnails.push(previous_peer);
        let primary_registration = scene.primary.as_ref().unwrap().thumbnail;
        let mut mixed = LayoutCreated::default();
        scene.stage_unrequested_secondary(&[], &[], &mut mixed);
        assert_eq!(mixed.displaced_secondary.len(), 1);
        assert!(scene.thumbnails.is_empty());
        assert_eq!(IsWindowVisible(mixed.displaced_secondary[0].0.window), 0);
        scene.profiles.push((3, owned_target.clone()));
        let next_primary = PreviewTile {slot:3, viewport:primary_tile.viewport};
        let visible = scene.visibility();
        scene.update_primary_observation(
            manager as usize as u64, &next_primary, visible, &mut mixed,
        ).unwrap();
        assert!(mixed.primary_changed);
        scene.discard_unacknowledged_new(&mut mixed);
        assert_eq!(scene.primary.as_ref().unwrap().thumbnail, primary_registration);
        assert_eq!(scene.thumbnails.len(), 1);
        assert_eq!(scene.thumbnails[0].thumbnail, peer_registration);
        assert_ne!(IsWindowVisible(scene.thumbnails[0].window), 0,
            "Mixed rollback must restore the accepted secondary");
        // In a successfully delivered layout, disposal commits
        // normally by dropping its one transaction; no Win32 retry.
        let mut accepted = LayoutCreated::default();
        scene.stage_unrequested_secondary(&[], &[], &mut accepted);
        assert_eq!(accepted.displaced_secondary.len(), 1);
        drop(accepted);
        assert!(scene.thumbnails.is_empty());
        scene.profiles.retain(|(slot,_)| *slot == tile.slot);

        // A new primary must be registered and verified before
        // the previous DWM registration is relinquished.
        scene.profiles = vec![(tile.slot, owned_target.clone()),
            (4, owned_target.clone())];
        let candidate_tile = PreviewTile {slot:4,viewport:primary_tile.viewport};
        let original_registration = scene.primary.as_ref().unwrap().thumbnail;
        let visible = scene.visibility();
        let mut replaced = LayoutCreated::default();
        scene.update_primary_observation(
            manager as usize as u64, &candidate_tile, visible, &mut replaced,
        ).unwrap();
        assert!(replaced.primary_changed);
        assert!(replaced.replaced_primary.is_some());
        assert_eq!(scene.primary.as_ref().unwrap().slot, 4);
        scene.discard_unacknowledged_new(&mut replaced);
        assert_eq!(scene.primary.as_ref().unwrap().slot, tile.slot);
        assert_eq!(scene.primary.as_ref().unwrap().thumbnail, original_registration);
        assert!(scene.primary.as_ref().unwrap().destination_owned());

        // A candidate source without a real HWND cannot destroy
        // its old verified primary observer.
        let invalid = OwnedProcess {
            pid: 0, window_handle: "0x0".into(), ..owned_target.clone()
        };
        scene.profiles.push((3, invalid));
        let existing_window = scene.primary.as_ref().unwrap().window;
        let visible = scene.visibility();
        let mut refused = LayoutCreated::default();
        assert!(scene.update_primary_observation(
            manager as usize as u64,
            &PreviewTile {slot:3,viewport:primary_tile.viewport},
            visible, &mut refused,
        ).is_err());
        assert_eq!(scene.primary.as_ref().unwrap().window, existing_window);
        scene.profiles.retain(|(slot,_)| *slot != 3);

        // A source replaced during the pending reply cannot revive
        // the previous observer, even if its native HWND still exists.
        let visible = scene.visibility();
        let mut superseded = LayoutCreated::default();
        scene.update_primary_observation(
            manager as usize as u64, &candidate_tile, visible, &mut superseded,
        ).unwrap();
        scene.profiles.retain(|(slot,_)| *slot != tile.slot);
        scene.discard_unacknowledged_new(&mut superseded);
        assert!(scene.primary.is_none(),
            "A retired primary source cannot survive a lost reply");
        scene.manager = 0;
        scene.discard_unacknowledged_new(&mut LayoutCreated::default());
        assert!(scene.window.is_none());
        drop(scene);
        DestroyWindow(source);
        DestroyWindow(manager);
    }
}
