use super::*;
use std::{
    os::windows::{io::AsRawHandle, process::CommandExt},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Foundation::*,
    System::{RemoteDesktop::ProcessIdToSessionId, Threading::*},
    UI::WindowsAndMessaging::*,
};
struct Fixture {
    process: Child,
    binary: std::path::PathBuf,
    directory: std::path::PathBuf,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        let _ = std::fs::remove_file(&self.binary);
        let _ = std::fs::remove_dir(&self.directory);
    }
}
struct Search {
    pid: u32,
    window: HWND,
}
unsafe extern "system" fn find(window: HWND, data: isize) -> i32 {
    let search = &mut *(data as *mut Search);
    let mut pid = 0;
    GetWindowThreadProcessId(window, &mut pid);
    if pid == search.pid && IsWindowVisible(window) != 0 && GetParent(window).is_null() {
        search.window = window;
        return 0;
    }
    1
}
fn fixture() -> (Fixture, OwnedProcess) {
    let canonical = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let root =
        std::path::PathBuf::from(canonical.to_string_lossy().trim_start_matches(r"\\?\"));
    let directory = root.join("../workspace").join(format!(
        "surface-regression-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    let binary = directory.join("Fixture.exe");
    let compiler = std::path::PathBuf::from(std::env::var_os("WINDIR").unwrap())
        .join("Microsoft.NET/Framework64/v4.0.30319/csc.exe");
    let output = ProcessCommand::new(compiler)
        .args([
            "/nologo",
            "/target:winexe",
            "/platform:x64",
            "/reference:System.Windows.Forms.dll",
            "/reference:System.Drawing.dll",
        ])
        .arg(format!("/out:{}", binary.display()))
        .arg(
            root.join("tooling")
                .join("acceptance")
                .join("windows")
                .join("workspace-host-fixture.cs"),
        )
        .creation_flags(0x08000000)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let process = ProcessCommand::new(&binary)
        .arg("--fixture-child")
        .spawn()
        .unwrap();
    let fixture = Fixture {
        process,
        binary,
        directory,
    };
    let mut search = Search {
        pid: fixture.process.id(),
        window: std::ptr::null_mut(),
    };
    let deadline = Instant::now() + Duration::from_secs(3);
    while search.window.is_null() && Instant::now() < deadline {
        unsafe {
            EnumWindows(Some(find), &mut search as *mut Search as isize);
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(
        !search.window.is_null(),
        "Synthetic game did not create a window"
    );
    let target = unsafe {
        let mut created: FILETIME = std::mem::zeroed();
        let mut a = created;
        let mut b = created;
        let mut c = created;
        assert_ne!(
            GetProcessTimes(
                fixture.process.as_raw_handle(),
                &mut created,
                &mut a,
                &mut b,
                &mut c
            ),
            0
        );
        let mut session = 0;
        assert_ne!(ProcessIdToSessionId(fixture.process.id(), &mut session), 0);
        OwnedProcess {
            pid: fixture.process.id(),
            created_file_time: (((created.dwHighDateTime as u64) << 32)
                | created.dwLowDateTime as u64)
                .to_string(),
            session_id: session,
            window_handle: format!("0x{:x}", search.window as usize),
            responding: true,
        }
    };
    (fixture, target)
}
#[test]
#[ignore = "Requires the built debug manager and Windows .NET compiler; synthetic windows only"]
fn drag_callback_does_not_leave_an_unordered_game_move_after_selection() {
    crate::engine::dpi::configure().unwrap();
    let class=windows_sys::w!("STATIC");
    let manager=unsafe { CreateWindowExW(0,class,class,WS_OVERLAPPEDWINDOW,
        50,50,1200,900,std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null_mut(),std::ptr::null()) };
    assert!(!manager.is_null());
    let (_fixture,target)=fixture();
    let viewport=Viewport {x:20.0,y:60.0,width:700.0,height:450.0,
        pixel_ratio:unsafe {windows_sys::Win32::UI::HiDpi::GetDpiForWindow(manager)} as f64/96.0};
    present(manager as usize as u64,viewport,target.clone()).unwrap();
    let before=native::capture(target.clone()).unwrap().rect;
    let deferred=follow_manager(manager as usize as u64).unwrap();
    // An async SetWindowPos can arrive after a later, acknowledged selection.
    // Only the ordered helper may place/park the native game; the UI callback defers.
    std::thread::sleep(Duration::from_millis(250));
    let after=native::capture(target).unwrap().rect;
    release().unwrap();
    unsafe {DestroyWindow(manager);}
    assert_eq!(after,before,"A delayed drag move can park the foreground game after selection");
    assert!(!deferred,"Active translation must be reconciled by the existing worker");
}
#[test]
#[ignore = "Requires the built debug manager and Windows .NET compiler; synthetic windows only"]
fn presentation_keeps_manager_opaque_during_a_stalled_drag() {
    crate::engine::dpi::configure().unwrap();
    let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
    let manager = unsafe {
        CreateWindowExW(0, class.as_ptr(), class.as_ptr(), WS_OVERLAPPEDWINDOW,
            50, 50, 1200, 900, std::ptr::null_mut(), std::ptr::null_mut(),
            std::ptr::null_mut(), std::ptr::null())
    };
    assert!(!manager.is_null());
    let (_fixture, target) = fixture();
    let original = native::capture(target.clone()).unwrap();
    let viewport = Viewport { x: 20.0, y: 60.0, width: 700.0, height: 450.0,
        pixel_ratio: unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(manager) } as f64 / 96.0 };
    present(manager as usize as u64, viewport, target.clone()).unwrap();
    let source = u64::from_str_radix(target.window_handle.trim_start_matches("0x"),16).unwrap() as usize as HWND;
    unsafe { assert_ne!(PostMessageW(source, 0x8001, 0, 0), 0); }
    std::thread::sleep(Duration::from_millis(100));
    unsafe { assert_ne!(SetWindowPos(manager, std::ptr::null_mut(), 250, 160, 0, 0,
        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE), 0); }
    let started = Instant::now();
    follow_manager(manager as usize as u64).unwrap();
    let elapsed = started.elapsed();
    let opaque = unsafe {
        use windows_sys::Win32::Graphics::Gdi::*;
        let region = CreateRectRgn(0,0,0,0);
        let mut bounds: RECT = std::mem::zeroed();
        GetWindowRect(manager, &mut bounds);
        let destination = native::rectangle(manager as usize as u64, viewport).unwrap();
        let result = GetWindowRgn(manager, region) == ERROR || PtInRegion(region,
            destination.x - bounds.left + destination.width / 2,
            destination.y - bounds.top + destination.height / 2) != 0;
        DeleteObject(region);
        result
    };
    // Restore our own fixture even when the regression assertion fails.
    release().unwrap();
    let restored = native::capture(target).unwrap();
    unsafe { DestroyWindow(manager); }
    assert_eq!(restored.rect, original.rect);
    assert_eq!(restored.style, original.style);
    assert!(elapsed < Duration::from_millis(100), "Drag blocked on the game thread");
    assert!(opaque, "A delayed game exposes the desktop through the manager viewport");
}
#[test]
#[ignore = "Requires the built debug manager and Windows .NET compiler; synthetic windows only"]
fn stalled_window_reports_the_native_stage_and_recovers_without_game_termination() {
    crate::engine::dpi::configure().unwrap();
    let class: [u16; 7] = [83, 84, 65, 84, 73, 67, 0];
    let manager = unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            50,
            50,
            1200,
            900,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null(),
        )
    };
    assert!(!manager.is_null());
    let (mut fixture, target) = fixture();
    let original = native::capture(target.clone()).unwrap();
    let viewport = Viewport {
        x: 20.0,
        y: 60.0,
        width: 700.0,
        height: 450.0,
        pixel_ratio: unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(manager) }
            as f64
            / 96.0,
    };
    let mut lease = prepare(manager as usize as u64, viewport, target.clone()).unwrap();
    let window = u64::from_str_radix(target.window_handle.trim_start_matches("0x"), 16)
        .unwrap() as usize as HWND;
    // Arm after shell/style setup so the next placement blocks at SetWindowPos.
    lease
        .send(Command::Place {
            rect: native::rectangle(manager as usize as u64, viewport).unwrap(),
            manager_pid: std::process::id(),
        })
        .unwrap();
    unsafe {
        assert_ne!(PostMessageW(window, 0x8001, 0, 0), 0);
    }
    std::thread::sleep(Duration::from_millis(100));
    SURFACE.lock().unwrap().active = Some(lease);
    let translation=Instant::now();
    follow_manager(manager as usize as u64).unwrap();
    assert!(translation.elapsed()<Duration::from_millis(100),"Dragging waited for the stalled game thread");
    let started = Instant::now();
    let error = update(manager as usize as u64, Some(viewport))
        .err()
        .expect("Synthetic placement did not stall");
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    let diagnostic = error.to_string();
    // Recover even when the diagnostic assertion fails on the old implementation.
    let held = SURFACE.lock().unwrap().suspended;
    let followup = update(manager as usize as u64, Some(viewport));
    release().unwrap();
    assert!(fixture.process.try_wait().unwrap().is_none());
    let restored = native::capture(target).unwrap();
    assert_eq!(restored.rect, original.rect);
    assert_eq!(restored.style, original.style);
    assert_eq!(restored.extended_style, original.extended_style);
    unsafe {
        DestroyWindow(manager);
    }
    assert!(started.elapsed() < Duration::from_secs(8));
    assert!(held, "Failed update was not held for explicit recovery");
    assert!(followup.is_ok(), "A native event retried the failed lease");
    assert!(
        diagnostic.contains("place/position"),
        "Missing blocking native stage: {diagnostic}"
    );
}
#[test]
#[ignore = "Requires the built debug manager and Windows .NET compiler; synthetic windows only"]
fn actual_surface_helper_restores_on_release_pipe_loss_and_helper_crash() {
    crate::engine::dpi::configure().unwrap();
    let class: [u16; 7] = [83, 84, 65, 84, 73, 67, 0];
    let manager = unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            class.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            50,
            50,
            1200,
            900,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null(),
        )
    };
    assert!(!manager.is_null());
    let (fixture_a, target) = fixture();
    let viewport = Viewport {
        x: 20.0,
        y: 60.0,
        width: 700.0,
        height: 450.0,
        pixel_ratio: unsafe { windows_sys::Win32::UI::HiDpi::GetDpiForWindow(manager) }
            as f64
            / 96.0,
    };
    for maximized in [false, true] {
        if maximized {
            unsafe {
                ShowWindow(
                    u64::from_str_radix(target.window_handle.trim_start_matches("0x"), 16)
                        .unwrap() as usize as HWND,
                    SW_MAXIMIZE,
                );
            }
        }
        let original = native::capture(target.clone()).unwrap();
        for mode in ["release", "pipe-loss", "helper-crash"] {
            eprintln!("Synthetic surface case: maximized={maximized}, operation={mode}");
            present(manager as usize as u64, viewport, target.clone()).unwrap();
            std::thread::sleep(Duration::from_millis(100));
            let composed = native::capture(target.clone()).unwrap();
            unsafe {
                use windows_sys::Win32::Graphics::Gdi::*;
                let region = CreateRectRgn(0, 0, 0, 0);
                assert_eq!(
                    GetWindowRgn(manager, region), ERROR,
                    "Presentation must preserve the opaque manager and DWM frame"
                );
                DeleteObject(region);
            }
            assert_ne!(composed.style, original.style);
            assert_ne!(composed.extended_style & WS_EX_TOOLWINDOW as i64, 0);
            assert_eq!(composed.extended_style & WS_EX_APPWINDOW as i64, 0);
            assert_eq!(
                composed.extended_style & WS_EX_TOPMOST as i64,
                original.extended_style & WS_EX_TOPMOST as i64,
                "Presentation retained temporary topmost priority"
            );
            assert_eq!(
                composed.rect,
                native::rectangle(manager as usize as u64, viewport).unwrap()
            );
            assert_eq!(
                unsafe {
                    GetParent(
                        u64::from_str_radix(
                            target.window_handle.trim_start_matches("0x"),
                            16,
                        )
                        .unwrap() as usize as HWND,
                    )
                },
                std::ptr::null_mut()
            );
            if mode == "release" {
                release().unwrap();
            } else {
                {
                    let mut state = SURFACE.lock().unwrap();
                    let lease = state.active.as_mut().unwrap();
                    if mode == "pipe-loss" {
                        drop(lease.input.take());
                    } else {
                        lease.child.kill().unwrap();
                    }
                }
                let deadline = Instant::now() + Duration::from_secs(3);
                loop {
                    // The fixture manager's owning test thread must service cross-thread
                    // region restoration just as Tauri's UI message loop does.
                    unsafe {
                        let mut message: MSG = std::mem::zeroed();
                        while PeekMessageW(
                            &mut message,
                            std::ptr::null_mut(),
                            0,
                            0,
                            PM_REMOVE,
                        ) != 0
                        {
                            TranslateMessage(&message);
                            DispatchMessageW(&message);
                        }
                    }
                    let current = native::capture(target.clone()).unwrap();
                    let region_clear = unsafe {
                        use windows_sys::Win32::Graphics::Gdi::*;
                        let region = CreateRectRgn(0, 0, 0, 0);
                        let clear = GetWindowRgn(manager, region) == ERROR;
                        DeleteObject(region);
                        clear
                    };
                    if current.style == original.style
                        && current.rect == original.rect
                        && region_clear
                    {
                        break;
                    }
                    assert!(
                        Instant::now() < deadline,
                        "{mode}: restoration deadline exceeded"
                    );
                    std::thread::sleep(Duration::from_millis(20));
                }
                release().unwrap();
            }
            std::thread::sleep(Duration::from_millis(100));
            let current = native::capture(target.clone()).unwrap();
            unsafe {
                use windows_sys::Win32::Graphics::Gdi::*;
                let region = CreateRectRgn(0, 0, 0, 0);
                assert_eq!(
                    GetWindowRgn(manager, region),
                    ERROR,
                    "Recovery left a viewport hole"
                );
                DeleteObject(region);
            }
            assert_eq!(current.style, original.style, "{mode}");
            assert_eq!(current.rect, original.rect, "{mode}");
            assert_eq!(
                current.extended_style, original.extended_style,
                "{mode}: shell recovery"
            );
        }
    }
    let mut reused = target.clone();
    reused.created_file_time =
        (target.created_file_time.parse::<u64>().unwrap() + 1).to_string();
    assert!(native::capture(reused).is_err());
    let mut foreign = target.clone();
    foreign.session_id += 1;
    assert!(native::capture(foreign).is_err());
    present(manager as usize as u64, viewport, target.clone()).unwrap();
    let mut invalid = target.clone();
    // Native resize can arrive before the webview sends its replacement rectangle.
    unsafe {
        assert_ne!(
            SetWindowPos(
                manager,
                std::ptr::null_mut(),
                0,
                0,
                500,
                350,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE
            ),
            0
        );
    }
    assert!(
        update(manager as usize as u64, None).is_ok(),
        "A stale native resize must wait for fresh webview geometry, not poison recovery"
    );
    assert!(
        update(manager as usize as u64, Some(viewport)).is_ok(),
        "A queued out-of-bounds webview rectangle must wait for its replacement"
    );
    let smaller = Viewport {
        width: 300.0,
        height: 180.0,
        ..viewport
    };
    update(manager as usize as u64, Some(smaller)).unwrap();
    assert_eq!(
        native::capture(target.clone()).unwrap().rect,
        native::rectangle(manager as usize as u64, smaller).unwrap()
    );
    unsafe {
        assert_ne!(
            SetWindowPos(
                manager,
                std::ptr::null_mut(),
                0,
                0,
                1200,
                900,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE
            ),
            0
        );
    }
    update(manager as usize as u64, Some(viewport)).unwrap();
    invalid.created_file_time =
        (invalid.created_file_time.parse::<u64>().unwrap() + 1).to_string();
    let before = native::capture(target.clone()).unwrap();
    assert!(present(manager as usize as u64, viewport, invalid).is_err());
    assert!(
        SURFACE.lock().unwrap().active.is_some(),
        "Rejected swap lost the previous surface"
    );
    let after = native::capture(target.clone()).unwrap();
    assert_eq!(before.rect, after.rect);
    assert_eq!(before.style, after.style);
    // Stale webview/native events must not override the current native state.
    unsafe {
        ShowWindow(manager, SW_SHOWMINNOACTIVE);
    }
    update(manager as usize as u64, Some(viewport)).unwrap();
    let game_window = u64::from_str_radix(target.window_handle.trim_start_matches("0x"), 16)
        .unwrap() as usize as HWND;
    assert_eq!(
        unsafe { IsWindowVisible(game_window) },
        0,
        "Webview update resurfaced a game over a minimized manager"
    );
    unsafe {
        ShowWindow(manager, SW_SHOWNOACTIVATE);
    }
    update(manager as usize as u64, None).unwrap();
    assert_ne!(
        unsafe { IsWindowVisible(game_window) },
        0,
        "Stale minimize event hid a restored session"
    );
    // Simulate a modal dialog disabling its main window between updates.
    unsafe {
        SetWindowLongPtrW(
            game_window,
            GWL_STYLE,
            GetWindowLongPtrW(game_window, GWL_STYLE) | WS_DISABLED as isize,
        );
    }
    update(manager as usize as u64, Some(viewport)).unwrap();
    assert_ne!(
        unsafe { GetWindowLongPtrW(game_window, GWL_STYLE) & WS_DISABLED as isize },
        0,
        "Placement overrode the game's modal disabled state"
    );
    release().unwrap();
    assert_ne!(
        unsafe { GetWindowLongPtrW(game_window, GWL_STYLE) & WS_DISABLED as isize },
        0,
        "Recovery overrode the game's modal disabled state"
    );
    unsafe {
        SetWindowLongPtrW(
            game_window,
            GWL_STYLE,
            GetWindowLongPtrW(game_window, GWL_STYLE) & !(WS_DISABLED as isize),
        );
    }
    let (_second_fixture, second) = fixture();
    let second_original = native::capture(second.clone()).unwrap();
    let second_window =
        u64::from_str_radix(second.window_handle.trim_start_matches("0x"), 16).unwrap()
            as usize as HWND;
    present(manager as usize as u64, viewport, target.clone()).unwrap();
    present(manager as usize as u64, viewport, second.clone()).unwrap();
    assert_eq!(
        unsafe { IsWindowVisible(game_window) },
        0,
        "Background tab stayed visible"
    );
    assert_ne!(unsafe { IsWindowVisible(second_window) }, 0);
    assert_eq!(SURFACE.lock().unwrap().parked.len(), 1);
    suspend().unwrap();
    update(manager as usize as u64, Some(viewport)).unwrap();
    assert_eq!(
        unsafe { IsWindowVisible(second_window) },
        0,
        "Settings resurfaced its suspended tab"
    );
    present(manager as usize as u64, viewport, target.clone()).unwrap();
    assert_ne!(unsafe { IsWindowVisible(game_window) }, 0);
    assert_eq!(unsafe { IsWindowVisible(second_window) }, 0);
    release_target(&target).unwrap();
    assert_eq!(
        SURFACE.lock().unwrap().parked.len(),
        1,
        "Releasing one tab detached another"
    );
    release().unwrap();
    let second_restored = native::capture(second.clone()).unwrap();
    assert_eq!(second_restored.style, second_original.style);
    assert_eq!(
        second_restored.extended_style,
        second_original.extended_style
    );
    assert_eq!(second_restored.rect, second_original.rect);
    // Browser-like round trips retain all five process identities and checkpoints.
    let (_third_fixture, third) = fixture();
    let (_fourth_fixture, fourth) = fixture();
    let (_fifth_fixture, fifth) = fixture();
    let tabs = [target.clone(), second.clone(), third, fourth, fifth];
    for tab in &tabs {
        present(manager as usize as u64, viewport, tab.clone()).unwrap();
    }
    for turn in 0..15 {
        let selected = turn % tabs.len();
        present(manager as usize as u64, viewport, tabs[selected].clone()).unwrap();
        for (index, tab) in tabs.iter().enumerate() {
            native::capture(tab.clone()).expect("Tab process/window identity was lost");
            let handle = u64::from_str_radix(tab.window_handle.trim_start_matches("0x"), 16)
                .unwrap() as usize as HWND;
            assert_eq!(unsafe { IsWindowVisible(handle) } != 0, index == selected);
        }
    }
    for (index, tab) in tabs.iter().enumerate() {
        present_verified(
            manager as usize as u64,
            viewport,
            tab.clone(),
            "test-session".into(),
            format!("{:02}", index + 1),
        )
        .unwrap();
    }
    assert!(!present_retained(
        manager as usize as u64,
        viewport,
        tabs[0].clone(),
        "other-session".into(),
        "01".into()
    )
    .unwrap());
    assert!(!present_retained(
        manager as usize as u64,
        viewport,
        tabs[0].clone(),
        "test-session".into(),
        "02".into()
    )
    .unwrap());
    let (_sixth_fixture, sixth) = fixture();
    assert!(present(manager as usize as u64, viewport, sixth).is_err(),
        "A sixth surface must not replace an existing five-profile lease");
    let mut timings = Vec::new();
    for turn in 0..40 {
        let selected = turn % tabs.len();
        let start = Instant::now();
        assert!(present_retained(
            manager as usize as u64,
            viewport,
            tabs[selected].clone(),
            "test-session".into(),
            format!("{:02}", selected + 1)
        )
        .unwrap());
        timings.push(start.elapsed().as_millis());
        for (index, tab) in tabs.iter().enumerate() {
            native::capture(tab.clone()).unwrap();
            let handle = u64::from_str_radix(tab.window_handle.trim_start_matches("0x"), 16)
                .unwrap() as usize as HWND;
            assert_eq!(unsafe { IsWindowVisible(handle) } != 0, index == selected);
        }
    }
    timings.sort();
    eprintln!("RETAINED_SWAP_MS p95={} max={}", timings[37], timings[39]);
    assert!(
        timings[39] < 2000,
        "Retained switching blocked like full engine admission"
    );
    release().unwrap();
    present(manager as usize as u64, viewport, target.clone()).unwrap();
    unsafe {
        assert_ne!(PostMessageW(game_window, WM_CLOSE, 0, 0), 0);
        assert_eq!(
            WaitForSingleObject(fixture_a.process.as_raw_handle(), 3000),
            WAIT_OBJECT_0
        );
    }
    present(manager as usize as u64, viewport, second.clone()).unwrap();
    release_for_close().unwrap();
    assert_eq!(unsafe{WaitForSingleObject(_second_fixture.process.as_raw_handle(),0)},WAIT_TIMEOUT,"Closing the manager terminated its game");
    assert!(present(manager as usize as u64,viewport,second).unwrap_err().to_string().contains("closing"));
    SURFACE.lock().unwrap().closing=false;
    unsafe {
        DestroyWindow(manager);
    }
}
