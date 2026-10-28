use super::*;
use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
use windows_sys::Win32::{Foundation::*, System::Threading::*, UI::WindowsAndMessaging::*};
pub(super) struct ProcessHandle(HANDLE);
impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.0);
        }
    }
}
fn permitted_image(parent: &std::path::Path, current: &std::path::Path) -> bool {
    let same = |a: &std::path::Path, b: &std::path::Path| {
        a.to_string_lossy()
            .replace('\\', "/")
            .eq_ignore_ascii_case(&b.to_string_lossy().replace('\\', "/"))
    };
    if same(parent, current) {
        return true;
    }
    // Developer-only portable regression harness, never a distributed helper mode.
    cfg!(debug_assertions)
        && current
            .parent()
            .is_some_and(|dir| parent.parent().is_some_and(|p| same(p, &dir.join("deps"))))
        && parent.file_name().is_some_and(|name| {
            let name = name.to_string_lossy();
            name.starts_with("m_bedrock_virtual_clients-") && name.ends_with(".exe")
        })
}
pub(super) fn trusted_parent() -> io::Result<()> {
    unsafe {
        use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error());
        }
        let _snapshot = ProcessHandle(snapshot);
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut ok = Process32FirstW(snapshot, &mut entry);
        let mut parent = 0;
        while ok != 0 {
            if entry.th32ProcessID == std::process::id() {
                parent = entry.th32ParentProcessID;
                break;
            }
            ok = Process32NextW(snapshot, &mut entry);
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, parent);
        if process.is_null() {
            return Err(io::Error::other(
                "Workspace helper requires its manager caller",
            ));
        }
        let _parent = ProcessHandle(process);
        let name = crate::engine::windows_process_image::query(process)?;
        let image = std::path::PathBuf::from(String::from_utf16_lossy(&name))
            .canonicalize()?;
        let current=std::env::current_exe()?.canonicalize()?;
        if !permitted_image(&image, &current) && !crate::engine::process_roles::same_main_image(&image,&current)? {
            return Err(io::Error::other(
                "Workspace helper requires its manager caller",
            ));
        }
        Ok(())
    }
}
#[test]
fn helper_caller_is_own_image_or_exact_developer_harness_directory() {
    use std::path::Path;
    let current = Path::new("C:/project/target/debug/manager.exe");
    assert!(permitted_image(current, current));
    assert!(permitted_image(
        Path::new("C:/project/target/debug/deps/m_bedrock_virtual_clients-123.exe"),
        current
    ));
    assert!(!permitted_image(
        Path::new("C:/other/deps/m_bedrock_virtual_clients-123.exe"),
        current
    ));
    assert!(!permitted_image(
        Path::new("C:/Windows/powershell.exe"),
        current
    ));
}
pub(super) fn hwnd(target: &OwnedProcess) -> io::Result<(ProcessHandle, HWND)> {
    let key = super::super::instance_identity::parse_window_identity(
        target.pid,
        Some(&target.created_file_time),
        Some(&target.window_handle),
    )
    .ok_or_else(|| io::Error::other("Invalid surface identity"))?;
    let window = key.hwnd as usize as HWND;
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(window, &mut pid);
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, key.pid);
        if process.is_null() {
            return Err(io::Error::last_os_error());
        }
        let pin = ProcessHandle(process);
        let mut created: FILETIME = std::mem::zeroed();
        let mut a = created;
        let mut b = created;
        let mut c = created;
        let ok = GetProcessTimes(process, &mut created, &mut a, &mut b, &mut c);
        let mut session = 0;
        let mut own_session = 0;
        if ok == 0
            || ProcessIdToSessionId(pid, &mut session) == 0
            || session != target.session_id
            || ProcessIdToSessionId(std::process::id(), &mut own_session) == 0
            || own_session != session
            || pid != key.pid
            || ((created.dwHighDateTime as u64) << 32 | created.dwLowDateTime as u64)
                != key.created_file_time
            || IsWindow(window) == 0
            || !GetParent(window).is_null()
            || GetAncestor(window, GA_ROOT) != window
        {
            return Err(io::Error::other("Surface process/window identity changed"));
        }
        Ok((pin, window))
    }
}
fn style(window: HWND, value: i64) -> io::Result<()> {
    unsafe {
        SetLastError(0);
        let previous = SetWindowLongPtrW(window, GWL_STYLE, value as isize);
        if previous == 0 && GetLastError() != 0 {
            return Err(io::Error::other(format!(
                "Workspace frame update: {}",
                io::Error::last_os_error()
            )));
        }
        Ok(())
    }
}
// Keep the official game playable in a normal Windows window without
// using the optional workspace capture helper. No game is terminated.
pub(super) fn focus_game_window(target: &OwnedProcess) -> io::Result<()> {
    let (_process, window) = hwnd(target)?;
    unsafe {
        if IsWindowVisible(window) == 0
            || !GetWindow(window, GW_OWNER).is_null()
            || IsHungAppWindow(window) != 0
        {
            return Err(io::Error::other(
                "Native Minecraft window is unavailable or unresponsive; refresh first",
            ));
        }
        if IsIconic(window) != 0 {
            ShowWindow(window, SW_RESTORE);
        }
    }
    // Recheck exact PID/creation-FILETIME/session/HWND after restoration,
    // retaining the process HANDLE through the Win32 foreground request.
    let (_rechecked_process, current_window) = hwnd(target)?;
    if current_window != window
        || unsafe { IsWindowVisible(window) } == 0
        || unsafe { IsHungAppWindow(window) } != 0
    {
        return Err(io::Error::other(
            "Native Minecraft window changed during restore; refresh first",
        ));
    }
    if unsafe { SetForegroundWindow(window) } == 0 {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Windows refused to activate Minecraft; select its taskbar window or use Alt+Tab",
        ));
    }
    Ok(())
}

fn window_fingerprint(window: HWND, expected_pid:u32) -> io::Result<(u32,String)> {
    let mut pid=0;
    let thread=unsafe { GetWindowThreadProcessId(window,&mut pid) };
    if thread==0 || pid!=expected_pid {
        return Err(io::Error::other("Game HWND owner changed during window recovery"));
    }
    let mut class=[0u16;128];
    let count=unsafe { GetClassNameW(window,class.as_mut_ptr(),class.len() as i32) };
    if count<=0 || count as usize>=class.len()-1 {
        return Err(io::Error::other("Game HWND class is unavailable or too long"));
    }
    let name=String::from_utf16(&class[..count as usize])
        .map_err(|_|io::Error::other("Invalid game HWND class name"))?;
    Ok((thread,name))
}
fn verify_saved_window(saved:&Saved, window:HWND)->io::Result<()> {
    let (thread,class)=window_fingerprint(window,saved.target.pid)?;
    if thread!=saved.window_thread_id || class!=saved.window_class_name {
        return Err(io::Error::new(io::ErrorKind::WouldBlock,
            "Native window recovery unconfirmed: saved game HWND was replaced by a different window/thread"));
    }
    Ok(())
}
// A successful OS call is not proof that a game's window retained the
// requested checkpoint geometry/frame. Compare only fields that this
// recovery owner actually changes, not gameplay-owned modal flags.
fn restored_checkpoint_matches(saved:&Saved,style:i64,shell:i64,bounds:Rectangle)->bool {
    let frame_mask=(WS_CAPTION | WS_THICKFRAME | WS_MAXIMIZE) as i64;
    let shell_mask=(WS_EX_APPWINDOW | WS_EX_TOOLWINDOW) as i64;
    bounds==saved.rect
        && (style & frame_mask)==(saved.style & frame_mask)
        && (shell & shell_mask)==(saved.extended_style & shell_mask)
}
pub(super) fn capture(target: OwnedProcess) -> io::Result<Saved> {
    let (_process, window) = hwnd(&target)?;
    let (window_thread_id,window_class_name)=window_fingerprint(window,target.pid)?;
    unsafe {
        if IsIconic(window) != 0 {
            return Err(io::Error::other(
                "Restore the game window before presenting it",
            ));
        }
        let mut rect: RECT = std::mem::zeroed();
        if GetWindowRect(window, &mut rect) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(Saved {
            target,
            window_thread_id,
            window_class_name,
            style: GetWindowLongPtrW(window, GWL_STYLE) as i64,
            extended_style: GetWindowLongPtrW(window, GWL_EXSTYLE) as i64,
            rect: Rectangle {
                x: rect.left,
                y: rect.top,
                width: rect.right - rect.left,
                height: rect.bottom - rect.top,
            },
            manager_pid: 0,
        })
    }
}
pub(super) fn place(
    saved: &Saved,
    rect: Rectangle,
    progress: &mut dyn FnMut(&str) -> io::Result<()>,
) -> io::Result<()> {
    if rect.width < 160 || rect.height < 120 || rect.width > 20000 || rect.height > 20000 {
        return Err(io::Error::other("Invalid native viewport"));
    }
    progress("place/identity")?;
    let (_process, window) = hwnd(&saved.target)?;
    unsafe {
        // No owner/parent and no topmost flag: other applications can cover it normally.
        if IsZoomed(window) != 0 {
            progress("place/unmaximize")?;
            ShowWindow(window, SW_SHOWNOACTIVATE);
        }
        let current_style = GetWindowLongPtrW(window, GWL_STYLE) as i64;
        // Minecraft owns live flags (especially modal WS_DISABLED), not our checkpoint.
        let presentation_style =
            current_style & !((WS_CAPTION | WS_THICKFRAME | WS_MAXIMIZE) as i64);
        let frame_changed = current_style != presentation_style;
        let shell_style = GetWindowLongPtrW(window, GWL_EXSTYLE) as i64;
        let managed_shell = (shell_style & !(WS_EX_APPWINDOW as i64)) | WS_EX_TOOLWINDOW as i64;
        if shell_style != managed_shell {
            progress("place/hide-for-shell")?;
            ShowWindow(window, SW_HIDE);
            progress("place/shell-style")?;
            SetLastError(0);
            let previous = SetWindowLongPtrW(window, GWL_EXSTYLE, managed_shell as isize);
            if previous == 0 && GetLastError() != 0 {
                return Err(io::Error::other(format!(
                    "Workspace shell adoption: {}",
                    io::Error::last_os_error()
                )));
            }
        }
        if frame_changed {
            progress("place/frame-style")?;
            style(window, presentation_style)?;
        }
        progress("place/position")?;
        if SetWindowPos(
            window,
            std::ptr::null_mut(),
            rect.x,
            rect.y,
            rect.width,
            rect.height,
            (if frame_changed { SWP_FRAMECHANGED } else { 0 })
                | SWP_NOACTIVATE
                | SWP_SHOWWINDOW
                // Background placement must not reorder against an unrelated application.
                | SWP_NOZORDER,
        ) == 0
        {
            return Err(io::Error::other(format!(
                "Workspace position update: {}",
                io::Error::last_os_error()
            )));
        }
        progress("place/verify-geometry")?;
        hwnd(&saved.target)?;
        let mut actual: RECT = std::mem::zeroed();
        if GetWindowRect(window, &mut actual) == 0
            || (actual.left - rect.x).abs() > 2
            || (actual.top - rect.y).abs() > 2
            || (actual.right - actual.left - rect.width).abs() > 2
            || (actual.bottom - actual.top - rect.height).abs() > 2
        {
            return Err(io::Error::other(format!(
                "Workspace geometry mismatch: requested {},{},{},{} observed {},{},{},{}",
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                actual.left,
                actual.top,
                actual.right - actual.left,
                actual.bottom - actual.top
            )));
        }
    }
    Ok(())
}
pub(super) fn receiving_input(target: &OwnedProcess) -> bool {
    hwnd(target).is_ok_and(|(_pin, window)| unsafe { GetForegroundWindow() == window })
}
pub(super) fn interactive_at(target: &OwnedProcess, manager: u64, viewport: Viewport) -> bool {
    let Ok((_pin, window)) = hwnd(target) else { return false; };
    let Ok(expected) = rectangle(manager, viewport) else { return false; };
    unsafe {
        let mut rect: RECT = std::mem::zeroed();
        if GetWindowRect(window, &mut rect) == 0 { return false; }
        super::interactive_surface(GetForegroundWindow() == window, IsWindowVisible(window) != 0,
            Rectangle { x: rect.left, y: rect.top, width: rect.right - rect.left, height: rect.bottom - rect.top }, expected)
    }
}
pub(super) fn clear_aperture(manager: u64) -> io::Result<()> {
    super::native_timing::measure(super::native_timing::Operation::Aperture, || {
    use windows_sys::Win32::Graphics::Gdi::*;
    let manager = manager as usize as HWND;
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(manager, &mut pid);
        if IsWindow(manager) == 0 || pid != std::process::id() {
            return Err(io::Error::other("Invalid workspace aperture owner"));
        }
        let current = CreateRectRgn(0, 0, 0, 0);
        if current.is_null() { return Err(io::Error::last_os_error()); }
        let absent = GetWindowRgn(manager, current) == ERROR;
        DeleteObject(current);
        if !absent && SetWindowRgn(manager, std::ptr::null_mut(), 1) == 0 {
            return Err(io::Error::last_os_error());
        }
        if RedrawWindow(manager, std::ptr::null(), std::ptr::null_mut(),
            RDW_INVALIDATE | RDW_ERASE | RDW_FRAME | RDW_ALLCHILDREN) == 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }
    })
}
pub(super) fn confirm_foreground(
    saved: &Saved,
    rect: Rectangle,
    manager: u64,
    activate: bool,
) -> io::Result<()> {
    super::native_timing::measure(super::native_timing::Operation::Foreground, || {
    let (_process, window) = hwnd(&saved.target)?;
    let manager = manager as usize as HWND;
    unsafe {
        let foreground = GetForegroundWindow();
        if foreground != manager && foreground != window {
            return Ok(());
        }
        // Passive layout/focus events must not seize input from the manager.
        if activate
            && foreground == manager
            && GetWindowLongPtrW(window, GWL_STYLE) & WS_DISABLED as isize == 0
        {
            SetForegroundWindow(window);
        }
        // Raise across the foreground band, then immediately restore normal shell priority.
        let raise = || {
            let flags = SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS;
            let raised = SetWindowPos(window, HWND_TOPMOST, 0, 0, 0, 0, flags);
            let restored = SetWindowPos(
                window,
                if saved.extended_style & WS_EX_TOPMOST as i64 != 0 {
                    HWND_TOPMOST
                } else {
                    HWND_NOTOPMOST
                },
                0,
                0,
                0,
                0,
                flags,
            );
            raised != 0 && restored != 0
        };
        if GetForegroundWindow() == manager && !raise() {
            return Err(io::Error::other("Workspace stacking update refused"));
        }
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(500);
        loop {
            let current_foreground = GetForegroundWindow();
            if current_foreground == manager {
                raise();
            }
            // WindowFromPoint skips disabled windows while Minecraft owns a modal dialog.
            if GetWindowLongPtrW(window, GWL_STYLE) & WS_DISABLED as isize != 0 {
                hwnd(&saved.target)?;
                return Ok(());
            }
            let hit = WindowFromPoint(POINT {
                x: rect.x + rect.width / 2,
                y: rect.y + rect.height / 2,
            });
            let point_root = GetAncestor(hit, GA_ROOT);
            if !foreground_surface_covered(
                current_foreground as usize as u64,
                window as usize as u64,
                manager as usize as u64,
                point_root as usize as u64,
            ) {
                hwnd(&saved.target)?;
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                return Err(io::Error::other(format!("Workspace game is covered; presentation was not confirmed (game foreground: {}, manager foreground: {}, center covered by manager: {}, visible: {}, enabled: {})",current_foreground==window,current_foreground==manager,point_root==manager,IsWindowVisible(window)!=0,GetWindowLongPtrW(window,GWL_STYLE)&WS_DISABLED as isize==0)));
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    })
}
// A Hidden receipt must describe the same original game window,
// including its PID/birth/session/HWND and saved thread/class.
pub(super) fn hidden_at(saved: &Saved) -> bool {
    let Ok((_pin, window)) = hwnd(&saved.target) else { return false; };
    verify_saved_window(saved, window).is_ok()
        && unsafe { IsWindowVisible(window) == 0 }
}
pub(super) fn hide(saved: &Saved) -> io::Result<()> {
    let (_pin, window) = hwnd(&saved.target)?;
    verify_saved_window(saved, window)?;
    unsafe {
        if IsWindowVisible(window) != 0 { ShowWindow(window, SW_HIDE); }
    }
    // A helper reply is NOT proof of success without a fresh OS
    // readback: no stale or recycled HWND may be acknowledged Hidden.
    if !hidden_at(saved) {
        return Err(io::Error::new(io::ErrorKind::WouldBlock,
            "Original Minecraft game Hide was not verified"));
    }
    Ok(())
}
pub(super) fn park(saved: &Saved, mut rect: Rectangle,
    progress: &mut dyn FnMut(&str) -> io::Result<()>) -> io::Result<()> {
    let (_pin, window) = hwnd(&saved.target)?;
    unsafe {
        if GetWindowLongPtrW(window, GWL_STYLE) & WS_DISABLED as isize != 0 {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "A Minecraft dialog is open"));
        }
    }
    rect = parking_rectangle(rect)?;
    // Keep the current viewport size across switches: no thumbnail-size game rendering
    // or repeated framebuffer allocation. DWM scales its source into the small tile.
    place(saved, rect, progress)
}
fn parking_rectangle(mut rect: Rectangle) -> io::Result<Rectangle> {
    unsafe {
        rect.x = GetSystemMetrics(SM_XVIRTUALSCREEN).checked_add(GetSystemMetrics(SM_CXVIRTUALSCREEN))
            .and_then(|right| right.checked_add(32)).ok_or_else(|| io::Error::other("Unsupported desktop bounds"))?;
        rect.y = GetSystemMetrics(SM_YVIRTUALSCREEN);
    }
    Ok(rect)
}
// A passive no-op is safe only after exact original process/birth/session/
// HWND and captured thread/class checks PLUS the live presentation style,
// visibility and full rectangle. This is a readback, not a new window owner.
pub(super) fn placed_at(saved: &Saved, expected: Rectangle) -> bool {
    let Ok((_pin, window)) = hwnd(&saved.target) else { return false; };
    if verify_saved_window(saved, window).is_err() { return false; }
    unsafe {
        if IsWindowVisible(window) == 0 || IsIconic(window) != 0 || IsZoomed(window) != 0 {
            return false;
        }
        let style = GetWindowLongPtrW(window, GWL_STYLE) as i64;
        let shell = GetWindowLongPtrW(window, GWL_EXSTYLE) as i64;
        // Place clears exactly these frame/shell flags. If Minecraft
        // shows a disabled modal, force the original helper path rather
        // than treating unchanged geometry as a successful action.
        if style & ((WS_CAPTION | WS_THICKFRAME | WS_MAXIMIZE | WS_DISABLED) as i64) != 0
            || shell & WS_EX_APPWINDOW as i64 != 0
            || shell & WS_EX_TOOLWINDOW as i64 == 0 {
            return false;
        }
        let mut rect: RECT = std::mem::zeroed();
        GetWindowRect(window, &mut rect) != 0
            && rect.left == expected.x && rect.top == expected.y
            && rect.right - rect.left == expected.width
            && rect.bottom - rect.top == expected.height
    }
}
pub(super) fn parked_at(saved: &Saved, rect: Rectangle) -> bool {
    let Ok(expected) = parking_rectangle(rect) else { return false; };
    placed_at(saved, expected)
}
pub(super) fn can_park(target: &OwnedProcess) -> io::Result<bool> {
    let (_pin, window) = hwnd(target)?;
    Ok(unsafe { GetWindowLongPtrW(window, GWL_STYLE) & WS_DISABLED as isize == 0 })
}
pub(super) fn minimized(manager: u64) -> io::Result<bool> {
    unsafe {
        let window = manager as usize as HWND;
        let mut pid = 0;
        GetWindowThreadProcessId(window, &mut pid);
        if IsWindow(window) == 0 || pid != std::process::id() {
            return Err(io::Error::other("Invalid workspace window"));
        }
        Ok(IsIconic(window) != 0)
    }
}
// A stale game HWND is not proof that the original Minecraft process
// exited. PID reuse and access-denied must not authorize mutation of
// another window or a successful recovery acknowledgement.
pub(super) fn original_game_process_exited(target: &OwnedProcess) -> io::Result<bool> {
    let expected = target.created_file_time.parse::<u64>()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid game process creation time"))?;
    if target.pid == 0 || expected == 0 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid game process identity"));
    }
    unsafe {
        let raw=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZATION_SYNCHRONIZE,0,target.pid);
        if raw.is_null() {
            let error=io::Error::last_os_error();
            // A nonexistent PID has no live original game process.
            // Access denied or any other failed identity probe is
            // uncertainty, not a license to claim recovery.
            if error.raw_os_error()==Some(ERROR_INVALID_PARAMETER as i32) {
                return Ok(true);
            }
            return Err(error);
        }
        let pin=ProcessHandle(raw);
        let mut created:FILETIME=std::mem::zeroed();
        let mut exited:FILETIME=std::mem::zeroed();
        let mut kernel:FILETIME=std::mem::zeroed();
        let mut user:FILETIME=std::mem::zeroed();
        if GetProcessTimes(pin.0,&mut created,&mut exited,&mut kernel,&mut user)==0 {
            return Err(io::Error::last_os_error());
        }
        let observed=((created.dwHighDateTime as u64)<<32)|created.dwLowDateTime as u64;
        if observed!=expected {
            // PID was reused; the original lifetime has ended, and the
            // replacement is not a window-recovery target.
            return Ok(true);
        }
        match WaitForSingleObject(pin.0,0) {
            WAIT_OBJECT_0=>Ok(true),
            WAIT_TIMEOUT=>Ok(false),
            _=>Err(io::Error::last_os_error()),
        }
    }
}

pub(super) fn restore(saved: &Saved) -> io::Result<()> {
    restore_traced(saved, &mut |_| Ok(()))
}
pub(super) fn restore_traced(
    saved: &Saved,
    progress: &mut dyn FnMut(&str) -> io::Result<()>,
) -> io::Result<()> {
    progress("restore/identity")?;
    let (_process, window) = match hwnd(&saved.target) {
        Ok(window) => window,
        Err(error) => {
            // Disappeared/reused HWND != exited game. Previously a live
            // game that lost its original HWND was reported restored.
            // Never touch another window, and never claim successful
            // restore until the original process lifetime is proven over.
            if retired(&saved.target) {
                return match original_game_process_exited(&saved.target) {
                    Ok(true)=>Ok(()),
                    Ok(false)=>Err(io::Error::new(io::ErrorKind::WouldBlock,
                        "Window recovery unconfirmed: original Minecraft process is still alive, but its saved game HWND changed or disappeared")),
                    Err(probe)=>Err(io::Error::other(format!(
                        "Window recovery unconfirmed: {error}; game process exit could not be verified: {probe}"
                    ))),
                };
            }
            Err(error)
        }
    };
    verify_saved_window(saved,window)?;
    unsafe {
        let frame_mask = (WS_CAPTION | WS_THICKFRAME | WS_MAXIMIZE) as i64;
        let shell_mask = (WS_EX_APPWINDOW | WS_EX_TOOLWINDOW) as i64;
        let shell = GetWindowLongPtrW(window, GWL_EXSTYLE) as i64;
        let restored_shell = (shell & !shell_mask) | (saved.extended_style & shell_mask);
        if shell != restored_shell {
            verify_saved_window(saved,window)?;
            progress("restore/hide-for-shell")?;
            ShowWindow(window, SW_HIDE);
            progress("restore/shell-style")?;
            SetLastError(0);
            let previous = SetWindowLongPtrW(window, GWL_EXSTYLE, restored_shell as isize);
            if previous == 0 && GetLastError() != 0 {
                return Err(io::Error::other(format!(
                    "Workspace shell recovery: {}",
                    io::Error::last_os_error()
                )));
            }
        }
        let current_style = GetWindowLongPtrW(window, GWL_STYLE) as i64;
        verify_saved_window(saved,window)?;
        progress("restore/frame-style")?;
        style(
            window,
            (current_style & !frame_mask) | (saved.style & frame_mask),
        )?;
        let r = saved.rect;
        let foreground = GetForegroundWindow();
        let mut pid = 0;
        GetWindowThreadProcessId(foreground, &mut pid);
        let lower = foreground != window && pid == saved.manager_pid;
        if GetWindowLongPtrW(window, GWL_EXSTYLE) & WS_EX_TOPMOST as isize != 0
            && saved.extended_style & WS_EX_TOPMOST as i64 == 0
        {
            SetWindowPos(
                window,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_ASYNCWINDOWPOS,
            );
        }
        verify_saved_window(saved,window)?;
        progress("restore/position")?;
        if SetWindowPos(
            window,
            if lower {
                foreground
            } else {
                std::ptr::null_mut()
            },
            r.x,
            r.y,
            r.width,
            r.height,
            SWP_FRAMECHANGED
                | SWP_NOACTIVATE
                | SWP_SHOWWINDOW
                | if lower { 0 } else { SWP_NOZORDER },
        ) == 0
        {
            return Err(io::Error::last_os_error());
        }
        // Restore acknowledgement requires readback; game or Windows may
        // have changed its HWND/geometry during the native placement.
        verify_saved_window(saved,window)?;
        progress("restore/verify")?;
        let mut current:RECT=std::mem::zeroed();
        if GetWindowRect(window,&mut current)==0 {
            return Err(io::Error::last_os_error());
        }
        let bounds=Rectangle {
            x:current.left,y:current.top,
            width:current.right-current.left,
            height:current.bottom-current.top,
        };
        if !restored_checkpoint_matches(saved,GetWindowLongPtrW(window,GWL_STYLE) as i64,
            GetWindowLongPtrW(window,GWL_EXSTYLE) as i64,bounds) {
            return Err(io::Error::new(io::ErrorKind::WouldBlock,
                "Window recovery unconfirmed: game frame or geometry differs from its saved checkpoint"));
        }
    }
    Ok(())
}
#[test]
fn lost_game_hwnd_with_original_live_process_never_counts_as_restored() {
    use windows_sys::Win32::System::Threading::{GetCurrentProcess,GetCurrentProcessId,GetProcessTimes};
    unsafe {
        let mut birth:FILETIME=std::mem::zeroed();
        let mut exit:FILETIME=std::mem::zeroed();
        let mut kernel:FILETIME=std::mem::zeroed();
        let mut user:FILETIME=std::mem::zeroed();
        assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exit,&mut kernel,&mut user),0);
        let pid=GetCurrentProcessId();
        let target=OwnedProcess {
            pid,
            created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),
            session_id:1,
            // Synthetic nonexistent HWND. No real game or window is used.
            window_handle:"0xffffffffffff".into(),
            responding:false,
        };
        let saved=Saved {
            target,window_thread_id:1,window_class_name:"SyntheticFixture".into(),
            style:0,extended_style:0,
            rect:Rectangle{x:0,y:0,width:1,height:1},
            manager_pid:pid,
        };
        assert!(retired(&saved.target));
        assert_eq!(original_game_process_exited(&saved.target).unwrap(),false);
        let error=restore(&saved).unwrap_err();
        assert_eq!(error.kind(),io::ErrorKind::WouldBlock);
        assert!(error.to_string().contains("original Minecraft process is still alive"));
    }
}
#[test]
fn restore_requires_saved_frame_shell_and_exact_geometry() {
    let saved=Saved {
        target:OwnedProcess {
            pid:100,created_file_time:"123".into(),session_id:1,
            window_handle:"0x999".into(),responding:true,
        },
        window_thread_id:2,window_class_name:"Synthetic".into(),
        style:(WS_CAPTION | WS_THICKFRAME) as i64,
        extended_style:WS_EX_APPWINDOW as i64,
        rect:Rectangle{x:30,y:75,width:800,height:450},manager_pid:200,
    };
    assert!(restored_checkpoint_matches(&saved,saved.style,
        saved.extended_style,saved.rect));
    assert!(!restored_checkpoint_matches(&saved,0,
        saved.extended_style,saved.rect));
    assert!(!restored_checkpoint_matches(&saved,saved.style,
        WS_EX_TOOLWINDOW as i64,saved.rect));
    assert!(!restored_checkpoint_matches(&saved,saved.style,
        saved.extended_style,Rectangle{x:31,..saved.rect}));
    assert!(restored_checkpoint_matches(&saved,saved.style | WS_DISABLED as i64,
        saved.extended_style,saved.rect));
}
#[test]
fn recovery_never_applies_saved_frame_to_a_different_window_fingerprint() {
    use windows_sys::Win32::System::Threading::{GetCurrentProcess,GetCurrentProcessId,GetProcessTimes};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW,DestroyWindow,GetWindowLongPtrW,GWL_STYLE,WS_OVERLAPPEDWINDOW
    };
    unsafe {
        let class=windows_sys::w!("STATIC");
        let window=CreateWindowExW(0,class,class,WS_OVERLAPPEDWINDOW,
            0,0,250,150,std::ptr::null_mut(),std::ptr::null_mut(),
            std::ptr::null_mut(),std::ptr::null());
        assert!(!window.is_null());
        let result=std::panic::catch_unwind(||{
            let mut birth:FILETIME=std::mem::zeroed();
            let mut exit:FILETIME=std::mem::zeroed();
            let mut kernel:FILETIME=std::mem::zeroed();
            let mut user:FILETIME=std::mem::zeroed();
            assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exit,&mut kernel,&mut user),0);
            let pid=GetCurrentProcessId();
            let mut session=0;
            assert_ne!(ProcessIdToSessionId(pid,&mut session),0);
            let target=OwnedProcess {
                pid,
                created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),
                session_id:session,
                window_handle:format!("0x{:x}",window as usize),
                responding:true,
            };
            let saved=capture(target).unwrap();
            assert!(saved.window_class_name.eq_ignore_ascii_case("static"));
            assert_ne!(saved.window_thread_id,0);
            let original=GetWindowLongPtrW(window,GWL_STYLE);
            let mut different=saved.clone();
            different.window_thread_id=different.window_thread_id.wrapping_add(1);
            assert!(restore(&different).unwrap_err().to_string().contains("saved game HWND was replaced"));
            assert_eq!(GetWindowLongPtrW(window,GWL_STYLE),original);
            different=saved.clone();
            different.window_class_name="WrongClass".into();
            assert!(restore(&different).unwrap_err().to_string().contains("saved game HWND was replaced"));
            assert_eq!(GetWindowLongPtrW(window,GWL_STYLE),original);
        });
        DestroyWindow(window);
        if let Err(error)=result {std::panic::resume_unwind(error);}
    }
}
#[test]
fn reused_pid_never_authorizes_native_window_mutation() {
    use windows_sys::Win32::System::Threading::{GetCurrentProcessId};
    // Same currently live PID but wrong original creation FILETIME:
    // the saved game lifetime has already ended.
    let target=OwnedProcess{
        pid:unsafe{GetCurrentProcessId()},
        created_file_time:"1".into(),
        session_id:1,
        window_handle:"0xffffffffffff".into(),
        responding:false,
    };
    assert!(original_game_process_exited(&target).unwrap());
}
pub(super) fn retired(target: &OwnedProcess) -> bool {
    unsafe {
        let Some(key) = super::super::instance_identity::parse_window_identity(
            target.pid,
            Some(&target.created_file_time),
            Some(&target.window_handle),
        ) else {
            // An unparseable target cannot be retained as a trusted lease.
            return true;
        };
        let window = key.hwnd as usize as HWND;
        let mut pid = 0;
        GetWindowThreadProcessId(window, &mut pid);
        if IsWindow(window) == 0 || pid != target.pid {
            return true;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, target.pid);
        if process.is_null() {
            // ACCESS_DENIED and UNKNOWN are not the same as a live,
            // unchanged game. Deny retention until Windows can attest it.
            return true;
        }
        let mut created: FILETIME = std::mem::zeroed();
        let mut a = created;
        let mut b = created;
        let mut c = created;
        let ok = GetProcessTimes(process, &mut created, &mut a, &mut b, &mut c);
        CloseHandle(process);
        ok == 0
            || (((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64)
                != key.created_file_time
    }
}
pub(super) fn rectangle(manager: u64, viewport: Viewport) -> io::Result<Rectangle> {
    unsafe {
        use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
        use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
        let window = manager as usize as HWND;
        let mut pid = 0;
        GetWindowThreadProcessId(window, &mut pid);
        if pid != std::process::id() || IsWindow(window) == 0 {
            return Err(io::Error::other("Invalid workspace window"));
        }
        let scale = GetDpiForWindow(window) as f64 / 96.0;
        if scale <= 0.0 {
            return Err(io::Error::other("Workspace DPI unavailable"));
        }
        if (scale - viewport.pixel_ratio).abs() > 0.05 {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Workspace DPI changed. Wait for the layout, then try again.",
            ));
        }
        let mut client: RECT = std::mem::zeroed();
        let mut origin: POINT = std::mem::zeroed();
        if GetClientRect(window, &mut client) == 0 || ClientToScreen(window, &mut origin) == 0 {
            return Err(io::Error::last_os_error());
        }
        let x = (viewport.x * scale).round() as i32;
        let y = (viewport.y * scale).round() as i32;
        let width = (viewport.width * scale).round() as i32;
        let height = (viewport.height * scale).round() as i32;
        if x < 0 || y < 0 || x + width > client.right + 1 || y + height > client.bottom + 1 {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "Workspace layout changed; try again",
            ));
        }
        Ok(Rectangle {
            x: origin.x + x,
            y: origin.y + y,
            width,
            height,
        })
    }
}
