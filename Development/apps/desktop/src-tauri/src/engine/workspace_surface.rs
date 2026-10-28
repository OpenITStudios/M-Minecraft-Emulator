//! Presentation only: independent top-level game, no parenting or injected input.
use super::process_identity::OwnedProcess;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

#[path = "workspace_surface/previews.rs"]
mod previews;
#[path = "workspace_surface/recovery_gate.rs"]
mod recovery_gate;
#[cfg(windows)]
#[path = "workspace_surface/native_timing.rs"]
mod native_timing;
pub use previews::{PreviewTile, PreviewReport};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub pixel_ratio: f64,
}
impl Viewport {
    pub fn validate(self) -> io::Result<Self> {
        self.validate_size(160.0, 120.0)
    }
    fn validate_preview(self) -> io::Result<Self> {
        self.validate_size(32.0, 24.0)
    }
    fn validate_size(self, minimum_width: f64, minimum_height: f64) -> io::Result<Self> {
        if [self.x, self.y, self.width, self.height, self.pixel_ratio]
            .iter()
            .any(|v| !v.is_finite())
            || self.x < 0.0
            || self.y < 0.0
            || self.width < minimum_width
            || self.height < minimum_height
            || self.x + self.width > 20000.0
            || self.y + self.height > 20000.0
            || self.pixel_ratio < 0.5
            || self.pixel_ratio > 8.0
        {
            return Err(io::Error::other("Invalid workspace viewport"));
        }
        Ok(self)
    }
}
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
struct Rectangle {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}
#[cfg(any(windows, test))]
fn interactive_surface(focused: bool, visible: bool, actual: Rectangle, expected: Rectangle) -> bool {
    focused && visible && actual.x.abs_diff(expected.x) <= 2 && actual.y.abs_diff(expected.y) <= 2
        && actual.width.abs_diff(expected.width) <= 2 && actual.height.abs_diff(expected.height) <= 2
}
#[cfg(any(windows, test))]
fn foreground_surface_covered(foreground: u64, game: u64, manager: u64, point_root: u64) -> bool {
    (foreground == game || foreground == manager) && point_root != game
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Saved {
    target: OwnedProcess,
    // Original native HWND identity at capture. A reused numeric HWND must
    // never silently receive frame/geometry from a different game surface.
    window_thread_id: u32,
    window_class_name: String,
    style: i64,
    extended_style: i64,
    rect: Rectangle,
    manager_pid: u32,
}
#[derive(Deserialize, Serialize)]
enum Command {
    Start(OwnedProcess),
    Place { rect: Rectangle, manager_pid: u32 },
    Park { rect: Rectangle, manager_pid: u32 },
    Hide,
    Restore,
}
#[derive(Deserialize, Serialize)]
enum Reply {
    Progress(String),
    Ready(Saved),
    Placed,
    Parked,
    Hidden,
    Restored,
    Error(String),
}
fn line<T: Serialize>(value: &T, out: &mut impl Write) -> io::Result<()> {
    serde_json::to_writer(&mut *out, value)?;
    out.write_all(b"\n")?;
    out.flush()
}
fn read_line(input: &mut impl BufRead) -> io::Result<Option<String>> {
    let mut bytes = Vec::new();
    let read = std::io::Read::take(&mut *input, 4097).read_until(b'\n', &mut bytes)?;
    if read == 0 {
        return Ok(None);
    }
    if read > 4096 || bytes.last() != Some(&b'\n') {
        return Err(io::Error::other("Invalid surface message"));
    }
    String::from_utf8(bytes).map(Some).map_err(io::Error::other)
}

// The normal release build remains disabled for native window capture. Only
// the separately identified, source-reviewed preview build may use the
// existing exact-parent/PID/HWND recovery helper outside debug builds.
fn workspace_enabled() -> bool {
    cfg!(debug_assertions) || cfg!(feature = "native-workspace-preview")
}

#[cfg(windows)]
pub fn maybe_run_helper() -> bool {
    if std::env::args().nth(1).as_deref() != Some("--workspace-surface") {
        return false;
    }
    if !workspace_enabled() {
        return true;
    }
    if super::dpi::configure().is_err() {
        return true;
    }
    if let Err(error) = native::trusted_parent() {
        let _ = line(&Reply::Error(error.to_string()), &mut io::stdout());
        return true;
    }
    let input = io::stdin();
    let mut input = input.lock();
    let output = io::stdout();
    let mut output = output.lock();
    let mut saved: Option<Saved> = None;
    let mut _recovery: Option<recovery_gate::RecoveryGate> = None;
    let result = (|| -> io::Result<()> {
        while let Some(text) = read_line(&mut input)? {
            let command: Command = serde_json::from_str(&text)?;
            let reply = match command {
                Command::Start(target) if saved.is_none() => {
                    line(&Reply::Progress("start/capture".into()), &mut output)?;
                    let gate = recovery_gate::open(&target)?;
                    let state = native::capture(target)?;
                    saved = Some(state.clone());
                    _recovery = Some(gate);
                    Reply::Ready(state)
                }
                Command::Place { rect, manager_pid } => {
                    let state = saved
                        .as_mut()
                        .ok_or_else(|| io::Error::other("Surface not prepared"))?;
                    state.manager_pid = manager_pid;
                    native::place(state, rect, &mut |stage| {
                        line(&Reply::Progress(stage.into()), &mut output)
                    })?;
                    Reply::Placed
                }
                Command::Hide => {
                    line(&Reply::Progress("hide/window".into()), &mut output)?;
                    native::hide(
                        saved
                            .as_ref()
                            .ok_or_else(|| io::Error::other("Surface not prepared"))?,
                    )?;
                    Reply::Hidden
                }
                Command::Park { rect, manager_pid } => {
                    let state = saved.as_mut().ok_or_else(|| io::Error::other("Surface not prepared"))?;
                    state.manager_pid = manager_pid;
                    native::park(state, rect, &mut |stage| line(&Reply::Progress(stage.into()), &mut output))?;
                    Reply::Parked
                }
                Command::Restore => {
                    if let Some(state) = &saved {
                        native::restore_traced(state, &mut |stage| {
                            line(&Reply::Progress(stage.into()), &mut output)
                        })?;
                    }
                    saved = None;
                    _recovery = None;
                    line(&Reply::Restored, &mut output)?;
                    break;
                }
                _ => return Err(io::Error::other("Surface already prepared")),
            };
            line(&reply, &mut output)?;
        }
        Ok(())
    })();
    // stdin EOF means the manager exited/crashed. Never close the game.
    if let Some(state) = &saved {
        if let Err(error) = native::restore(state) {
            let _ = line(&Reply::Error(error.to_string()), &mut output);
        }
    }
    if let Err(error) = result {
        let _ = line(&Reply::Error(error.to_string()), &mut output);
    }
    true
}

// Preserve the reserved internal role spelling without starting a worker,
// reading helper IPC or inspecting any process on unsupported platforms.
// A command bearing this Windows-only role must not fall through into the
// ordinary Tauri window startup.
#[cfg(not(windows))]
pub fn maybe_run_helper() -> bool {
    std::env::args_os().nth(1).as_deref() == Some(std::ffi::OsStr::new("--workspace-surface"))
}

#[cfg(windows)]
pub(crate) fn verify_same_executable_parent() -> io::Result<()> {
    native::trusted_parent()
}

#[cfg(windows)]
mod native {
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
            let mut name = vec![0u16; 32768];
            let mut length = name.len() as u32;
            if QueryFullProcessImageNameW(process, 0, name.as_mut_ptr(), &mut length) == 0 {
                return Err(io::Error::last_os_error());
            }
            let image =
                std::path::PathBuf::from(String::from_utf16_lossy(&name[..length as usize]))
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
}
#[cfg(not(windows))]
mod native {
    use super::*;
    fn unavailable<T>() -> io::Result<T> {
        Err(io::Error::other("Windows workspace required"))
    }
    pub(super) fn trusted_parent() -> io::Result<()> {
        unavailable()
    }
    pub(super) fn capture(_: OwnedProcess) -> io::Result<Saved> {
        unavailable()
    }
    pub(super) fn place(
        _: &Saved,
        _: Rectangle,
        _: &mut dyn FnMut(&str) -> io::Result<()>,
    ) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn hide(_: &Saved) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn hidden_at(_: &Saved) -> bool { false }
    pub(super) fn park(_: &Saved, _: Rectangle, _: &mut dyn FnMut(&str) -> io::Result<()>) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn can_park(_: &OwnedProcess) -> io::Result<bool> { unavailable() }
    pub(super) fn parked_at(_: &Saved, _: Rectangle) -> bool { false }
    pub(super) fn receiving_input(_: &OwnedProcess) -> bool { false }
    pub(super) fn clear_aperture(_: u64) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn confirm_foreground(_: &Saved, _: Rectangle, _: u64, _: bool) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn restore(_: &Saved) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn restore_traced(
        _: &Saved,
        _: &mut dyn FnMut(&str) -> io::Result<()>,
    ) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn rectangle(_: u64, _: Viewport) -> io::Result<Rectangle> {
        unavailable()
    }
    pub(super) fn minimized(_: u64) -> io::Result<bool> {
        unavailable()
    }
    pub(super) fn retired(_: &OwnedProcess) -> bool {
        false
    }
    pub(super) fn original_game_process_exited(_: &OwnedProcess) -> io::Result<bool> {
        unavailable()
    }
}

/// Activate the exact official game process in Windows, without parenting,
/// moving or launching it. A UI-provided PID/HWND is never sufficient.
pub fn focus_observed_native_window(target: OwnedProcess) -> io::Result<()> {
    let fresh = super::native_clients::inspect_native_workspace()?;
    if !super::native_clients::matches_native_workspace_target(&fresh, &target) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Official Minecraft process/window identity changed; refresh before activating",
        ));
    }
    #[cfg(windows)]
    {
        native::focus_game_window(&target)
    }
    #[cfg(not(windows))]
    {
        let _ = target;
        Err(io::Error::new(io::ErrorKind::Unsupported, "Minecraft activation requires Windows"))
    }
}

// Only Windows compiles the existing HWND/OGLES recovery controller. Its
// process roles and exact native checkpoint ownership have no Mac analogue.
#[cfg(windows)]
mod controller {
    use super::*;
    use std::{
        process::{Child, ChildStdin, Command as ProcessCommand, Stdio},
        sync::{mpsc, Mutex},
        time::{Duration, Instant},
    };
    // A stdout reader End(Ok) is not proof that its source process has
    // exited. The child may have closed stdout while still holding the
    // checkpoint or performing native HWND work. A failed End is never success.
    // A passive layout may omit the identical helper Place round trip only
    // within a bounded acknowledgement age. Activation, unknown checkpoint,
    // actual geometry change or age expiry always uses the original helper.
    // Exact native owner/window verification happens separately on every
    // candidate skip; this cache never authorizes another game/window.
    const HELPER_PASSIVE_PLACE_REASSERT: Duration = Duration::from_secs(1);
    const HELPER_PASSIVE_HIDE_REASSERT: Duration = Duration::from_secs(1);
    // The native readback closure is never called for an unknown, stale
    // or terminal helper receipt; no second worker/controller is involved.
    fn recent_hidden_receipt(
        acknowledged: Option<Instant>, now: Instant,
        terminal_seen: bool, helper_alive: bool,
        still_hidden: impl FnOnce() -> bool,
    ) -> bool {
        acknowledged.is_some_and(|at|
            now.saturating_duration_since(at) < HELPER_PASSIVE_HIDE_REASSERT)
            && !terminal_seen && helper_alive && still_hidden()
    }
    fn recent_passive_place(
        acknowledged: Option<(Rectangle, Instant)>,
        requested: Rectangle,
        now: Instant,
        activate: bool,
    ) -> bool {
        !activate && acknowledged.is_some_and(|(previous, at)|
            previous == requested
                && now.saturating_duration_since(at) < HELPER_PASSIVE_PLACE_REASSERT)
    }
    // A failed terminal End alone never permits discarding a checkpoint.
    // A later explicit stop/release may retry restoration ONLY after the
    // original helper process is gone and this lease still owns a Saved
    // checkpoint. The caller's native::restore performs fresh exact PID,
    // process birth, Windows session, HWND/thread and geometry checks.
    // No new helper, game lifecycle request or background retry is started.
    fn retry_failed_terminal_restore(
        terminal: Option<&Result<(), String>>,
        helper_exited: bool,
        checkpoint: Option<&Saved>,
        restore: impl FnOnce(&Saved) -> io::Result<()>,
    ) -> Option<io::Result<()>> {
        match (terminal, helper_exited, checkpoint) {
            (Some(Err(_)), true, Some(saved)) => Some(restore(saved)),
            _ => None,
        }
    }
    // An unacknowledged Start is not proof of a captured window. Only
    // original-game exit verified AFTER helper terminal+exit can retire
    // its failed pre-Ready recovery lease without Saved geometry.
    fn retire_failed_pre_ready(
        terminal: Option<&Result<(), String>>,
        helper_exited: bool,
        saved_present: bool,
        original_game_exited: impl FnOnce() -> io::Result<bool>,
    ) -> Option<io::Result<()>> {
        if !matches!(terminal, Some(Err(_))) || !helper_exited || saved_present {
            return None;
        }
        Some(match original_game_exited() {
            Ok(true) => Ok(()),
            Ok(false) => Err(io::Error::new(io::ErrorKind::WouldBlock,
                "Original Minecraft game remains alive; pre-Ready checkpoint retained")),
            Err(error) => Err(io::Error::new(error.kind(),
                format!("Original Minecraft exit could not be verified; pre-Ready checkpoint retained: {error}"))),
        })
    }
    fn completed_helper_status(
        terminal: Option<&Result<(), String>>,
        helper_exited: bool,
    ) -> io::Result<()> {
        match terminal {
            Some(Ok(())) if helper_exited => Ok(()),
            Some(Ok(())) => Err(io::Error::new(io::ErrorKind::WouldBlock,
                "Window recovery helper has not exited; checkpoint retained")),
            Some(Err(error)) => Err(io::Error::other(error.clone())),
            None => Err(io::Error::other("Final window recovery acknowledgement unavailable")),
        }
    }
    // The same owned game/window that opened the recovery gate must be the
    // one described by the helper's first Ready receipt. Responding is a
    // transient observation, not process or HWND identity.
    // A corrupt/truncated helper reply is not a clean EOF. Do not turn
    // an unreadable pre-Ready pipe into End(Ok) unless a real Restored receipt
    // or identity-checked native fallback has proved recovery. Clean EOF
    // before any capture remains harmless because Start only observes HWND.
    fn reader_terminal_result(
        pipe_issue: Option<String>,
        native_recovery: Result<(), String>,
        recovery_verified: bool,
    ) -> Result<(), String> {
        match native_recovery {
            Err(error) => Err(error),
            Ok(()) if recovery_verified => Ok(()),
            Ok(()) => pipe_issue.map_or(Ok(()), Err),
        }
    }
    // Compare exact pinned process + native window identity, never transient
    // responsiveness. Failed MC01 recovery cannot authorize or block MC02
    // merely because both belong to the same desktop window.
    fn same_native_target(a: &OwnedProcess, b: &OwnedProcess) -> bool {
        a.pid == b.pid
            && a.created_file_time == b.created_file_time
            && a.session_id == b.session_id
            && a.window_handle == b.window_handle
    }
    fn ready_matches_start(ready: &Saved, requested: &OwnedProcess) -> bool {
        same_native_target(&ready.target, requested)
    }
    // A stale *managed* lease for the same launcher is not an invitation to
    // re-capture a new PID/window while its previous checkpoint is uncertain.
    // Exact same-window identity also protects unlabelled test/legacy leases.
    // Only a confirmed, nonterminal Ready lease may provide DWM/Play
    // presentation. This cannot decide Minecraft gameplay availability.
    fn presentation_checkpoint_unverified(ready: bool, terminal_seen: bool) -> bool {
        !ready || terminal_seen
    }
    fn checkpoint_conflicts(
        pinned: &OwnedProcess,
        pinned_authority: Option<&(String, String)>,
        requested: &OwnedProcess,
        requested_authority: Option<&(String, String)>,
    ) -> bool {
        same_native_target(pinned, requested)
            || matches!((pinned_authority, requested_authority),
                (Some((name, slot)), Some((other_name, other_slot)))
                    if name == "LAZY_TESTER_MANAGED_SESSION"
                        && other_name == name && slot == other_slot)
    }

    // Keep a failed startup lease inside the existing SURFACE owner whenever
    // native shutdown has not been verified. The retained lease pins the
    // exact window recovery gate, child and reader against another capture.
    struct PrepareFailure {
        error: io::Error,
        retained: Option<Lease>,
    }
    impl From<io::Error> for PrepareFailure {
        fn from(error: io::Error) -> Self {
            Self { error, retained: None }
        }
    }
    impl std::fmt::Debug for PrepareFailure {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("PrepareFailure")
                .field("error", &self.error)
                .field("retained_checkpoint", &self.retained.is_some())
                .finish()
        }
    }
    fn rollback_error(parked: &io::Result<()>, restored: &io::Result<()>) -> Option<String> {
        let mut problems = Vec::new();
        if let Err(error) = parked { problems.push(format!("Could not park the new game: {error}")); }
        if let Err(error) = restored { problems.push(format!("Could not restore the previous view: {error}")); }
        if problems.is_empty() { None } else { Some(problems.join("; ")) }
    }
    #[cfg(test)]
    mod rollback_tests {
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
    }
    enum Event {
        Reply(Reply),
        End(Result<(), String>),
    }

    // A timed-out helper may still deliver its previous Place/Park/Restore
    // reply before stdout EOF. Only the reader's final End result confirms
    // restoration (helper acknowledgement or identity-checked owner fallback).
    fn wait_for_restoration_end(events: &mpsc::Receiver<Event>, timeout: Duration) -> io::Result<Result<(), String>> {
        let deadline=Instant::now()+timeout;
        let mut latest_error=None;
        loop {
            match events.recv_timeout(deadline.saturating_duration_since(Instant::now())) {
                Ok(Event::End(result)) => return Ok(result),
                Ok(Event::Reply(Reply::Error(error))) => latest_error=Some(error),
                Ok(Event::Reply(_)) => {}, // An earlier reply is never final recovery evidence.
                Err(_) => {
                    let detail=latest_error.unwrap_or_else(||"final owner restoration was not observed".into());
                    return Err(io::Error::other(format!(
                        "Window recovery not confirmed: {detail}. Do not start another surface."
                    )));
                }
            }
        }
    }

    struct Lease {
        child: Child,
        input: Option<ChildStdin>,
        events: mpsc::Receiver<Event>,
        viewport: Viewport,
        manager: u64,
        authority: Option<(String, String)>,
        ended: Option<Result<(), String>>,
        saved: Option<Saved>,
        // Exact requested process, pinned before Start; NOT Ready/capture proof.
        requested: OwnedProcess,
        // Last fully acknowledged and foreground-checked helper Place.
        // Cleared by Park/Hide or any failed native placement; never a
        // second HWND/controller authority or a fabricated Ready receipt.
        last_confirmed_place: Option<(Rectangle, Instant)>,
        // Scoped to this exact Lease; only an OS-verified Hidden reply
        // is kept as a short-lived, non-game-lifecycle optimization.
        last_confirmed_hide: Option<Instant>,
        _recovery_gate: recovery_gate::RecoveryGate,
    }
    impl Lease {
        fn background(&mut self, rect: Rectangle, live: bool) -> io::Result<()> {
            // Park and Hide revoke any previous active Place evidence.
            self.last_confirmed_place = None;
            if live {
                self.last_confirmed_hide = None;
                // The existing Park shortcut retains exact native
                // window/thread/class/style and frame readback.
                if self.saved.as_ref().is_some_and(|saved|native::parked_at(saved,rect)) {
                    return Ok(());
                }
                match self.send(Command::Park { rect, manager_pid: std::process::id() })? {
                    Reply::Parked => Ok(()), _ => Err(io::Error::other("Live preview parking failed")),
                }
            } else {
                let helper_alive = if self.last_confirmed_hide.is_some()
                    && self.ended.is_none() {
                    self.child.try_wait()?.is_none()
                } else { false };
                // Each skipped Hide requires a recent receipt AND a fresh
                // readback of the original hidden game window.
                if recent_hidden_receipt(self.last_confirmed_hide, Instant::now(),
                    self.ended.is_some(), helper_alive,
                    || self.saved.as_ref().is_some_and(native::hidden_at)) {
                    return Ok(());
                }
                self.last_confirmed_hide = None;
                match self.send(Command::Hide)? {
                    Reply::Hidden => {
                        if !self.saved.as_ref().is_some_and(native::hidden_at) {
                            return Err(io::Error::new(io::ErrorKind::WouldBlock,
                                "Minecraft Hide reply did not verify its original window"));
                        }
                        self.last_confirmed_hide = Some(Instant::now());
                        Ok(())
                    }
                    _ => Err(io::Error::other("Surface hide failed")),
                }
            }
        }
        fn place(&mut self, rect: Rectangle, activate: bool) -> io::Result<()> {
            self.last_confirmed_hide = None;
            if activate { super::previews::clear_primary()?; }
            // Even a successful prior Place is not enough to bypass a live
            // helper if its child exited, its reader became terminal, or
            // Windows changed the actual process/window/style/geometry.
            let skip_helper = recent_passive_place(
                self.last_confirmed_place, rect, Instant::now(), activate,
            ) && self.ended.is_none()
                && self.child.try_wait()?.is_none()
                && self.saved.as_ref().is_some_and(|saved| native::placed_at(saved, rect));
            if !skip_helper {
                // Clear the previous proof BEFORE any new helper write;
                // a timed-out Place may have mutated HWND asynchronously.
                self.last_confirmed_place = None;
                if !matches!(
                    self.send(Command::Place {
                        rect,
                        manager_pid: std::process::id(),
                    })?,
                    Reply::Placed
                ) {
                    return Err(io::Error::other("Surface presentation failed"));
                }
            }
            // Preserve the original safety postconditions even on a no-op.
            // Do not bypass aperture cleanup or foreground/stacking readback.
            let result = (|| {
                native::clear_aperture(self.manager)?;
                native::confirm_foreground(
                    self.saved
                        .as_ref()
                        .ok_or_else(|| io::Error::other("Surface checkpoint missing"))?,
                    rect, self.manager, activate,
                )
            })();
            if result.is_err() {
                self.last_confirmed_place = None;
            } else if !skip_helper {
                // A skip does NOT move this timestamp; continuous event
                // bursts cannot postpone the mandatory helper reassertion.
                self.last_confirmed_place = Some((rect, Instant::now()));
            }
            result
        }
        fn receive(&mut self) -> io::Result<Reply> {
            // Progress must never extend the command's original deadline.
            let deadline = Instant::now() + Duration::from_secs(3);
            let mut stage = String::from("awaiting helper");
            loop {
                match self
                    .events
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                {
                    Ok(Event::Reply(Reply::Progress(next))) => {
                        stage = next;
                        if Instant::now() < deadline {
                            continue;
                        }
                    }
                    Ok(Event::Reply(Reply::Error(error))) => {
                        return Err(io::Error::other(format!(
                            "Surface helper failed at {stage}: {error}"
                        )))
                    }
                    Ok(Event::Reply(reply)) => return Ok(reply),
                    Ok(Event::End(result)) => {
                        self.ended = Some(result.clone());
                        return Err(io::Error::other(format!(
                            "Surface helper ended at {stage}: {result:?}"
                        )));
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => {
                        return Err(io::Error::new(
                            io::ErrorKind::BrokenPipe,
                            format!("Surface helper disconnected at {stage}"),
                        ))
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    format!("Surface helper timed out after 3000 ms at {stage}"),
                ));
            }
        }
        fn send(&mut self, command: Command) -> io::Result<Reply> {
            if self.child.try_wait()?.is_some() {
                return self.receive();
            }
            line(
                &command,
                self.input.as_mut().ok_or_else(|| {
                    io::Error::new(io::ErrorKind::BrokenPipe, "Workspace connection closed")
                })?,
            )?;
            let result = self.receive();
            if result
                .as_ref()
                .is_err_and(|error| error.kind() == io::ErrorKind::TimedOut)
            {
                let _ = self.child.kill();
            }
            result
        }
        fn confirm_finished_helper(&mut self) -> io::Result<()> {
            // Terminal reader End precedes the helper's process-exit signal
            // on some Windows schedules. Never restore while the helper can
            // still make a later native window write.
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                let helper_exited = self.child.try_wait()?.is_some();
                if let Some(retired) = retire_failed_pre_ready(
                    self.ended.as_ref(),
                    helper_exited,
                    self.saved.is_some(),
                    || native::original_game_process_exited(&self.requested),
                ) {
                    // No Saved means no safe HWND/frame to restore. Only
                    // verified original process exit makes restore unnecessary.
                    return retired;
                }
                if let Some(retry) = retry_failed_terminal_restore(
                    self.ended.as_ref(),
                    helper_exited,
                    self.saved.as_ref(),
                    native::restore,
                ) {
                    // The source terminal End remains Err; the verified
                    // parent-side recovery is stronger *new* evidence.
                    // On failure the caller keeps this exact lease/gate
                    // for an explicit later attempt. No blind retry loop.
                    return retry.map_err(|error| io::Error::new(error.kind(),
                        format!("Native restoration retry failed; original recovery checkpoint retained: {error}")));
                }
                match completed_helper_status(self.ended.as_ref(), helper_exited) {
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        if Instant::now() >= deadline {
                            return Err(io::Error::new(io::ErrorKind::TimedOut,
                                "Final window recovery was observed but its helper process is still running; checkpoint retained"));
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    result => return result,
                }
            }
        }
        fn stop(&mut self) -> io::Result<()> {
            // A prior Place/Hide receipt is never native recovery evidence.
            self.last_confirmed_hide = None;
            self.last_confirmed_place = None;
            if self.ended.is_some() {
                return self.confirm_finished_helper();
            }
            let restored = matches!(self.send(Command::Restore), Ok(Reply::Restored));
            if self.ended.is_some() {
                return self.confirm_finished_helper();
            }
            if !restored && self.child.try_wait()?.is_none() {
                // Kill only our own recovery helper, never Minecraft. The
                // reader retains the checkpoint and attempts native restore.
                self.child.kill()?;
            }
            let result=wait_for_restoration_end(&self.events,Duration::from_secs(3))?;
            self.ended=Some(result);
            self.confirm_finished_helper()
        }
    }
    struct Surfaces {
        active: Option<Lease>,
        parked: Vec<Lease>,
        suspended: bool,
        live_previews: bool,
        closing: bool,
    }
    static SURFACE: Mutex<Surfaces> = Mutex::new(Surfaces {
        active: None,
        parked: Vec::new(),
        suspended: false,
        live_previews: false,
        closing: false,
    });
    fn same(lease: &Lease, target: &OwnedProcess) -> bool {
        // Never promote the pre-Ready request to an acknowledged Play surface.
        lease.saved.as_ref().is_some_and(|saved| same_native_target(&saved.target, target))
    }
    fn parked_cleanup_due(lease: &mut Lease) -> io::Result<bool> {
        Ok(lease.child.try_wait()?.is_some()
            || lease.saved.as_ref().is_some_and(|saved| native::retired(&saved.target)))
    }
    pub fn is_active() -> bool {
        SURFACE
            .try_lock()
            .map(|s| s.active.is_some() || !s.parked.is_empty())
            .unwrap_or(true)
    }
    pub fn release_for_close() -> io::Result<()> {
        SURFACE.lock().map_err(|_|io::Error::other("Workspace state unavailable"))?.closing=true;
        let result=release();
        if result.is_err() {
            if let Ok(mut state)=SURFACE.lock(){state.closing=false;}
        }
        result
    }
    // A failed Restore is not permission to skip the user's normal game
    // Exit forever. It is safe to *request* that exit only after every
    // original native presentation helper has completed and the manager
    // reader has received its terminal End result. A failed End is still
    // unconfirmed window recovery, never a successful restoration.
    fn close_only_if_helpers_quiesced(ended:bool, child_exited:bool)->bool {
        ended && child_exited
    }
    pub fn allow_game_close_after_failed_restore()->io::Result<bool> {
        let mut state=SURFACE.lock().map_err(|_|io::Error::other("Workspace state unavailable"))?;
        if let Some(lease)=state.active.as_mut() {
            if !close_only_if_helpers_quiesced(lease.ended.is_some(),lease.child.try_wait()?.is_some()) {
                return Ok(false);
            }
        }
        for lease in &mut state.parked {
            if !close_only_if_helpers_quiesced(lease.ended.is_some(),lease.child.try_wait()?.is_some()) {
                return Ok(false);
            }
        }
        // Prevent foreground/resize native surface updates while the
        // existing controller is servicing normal WM_CLOSE and dialog IDOK.
        state.closing=true;
        Ok(true)
    }
    // Fixed five-slot presentation identity, not a process-title/PID guess.
    // Keep one rule for normal release, retry after uncertain restore, and
    // checkpoint retirement after an actually exited managed game.
    fn matches_managed_authority(authority:Option<&(String,String)>,client:&str)->bool {
        authority.is_some_and(|(name,value)|
            name=="LAZY_TESTER_MANAGED_SESSION" && value==client)
    }
    fn slot_lease(lease:&Lease,client:&str)->bool {
        matches_managed_authority(lease.authority.as_ref(),client)
    }
    // Exactly one original lease for the selected slot may permit a close
    // despite unconfirmed window restoration. No status of any peer grants
    // authority over a different Minecraft instance.
    fn selected_helper_quiesced(evidence:&[(bool,bool)])->bool {
        let mut selected=0;
        for (matches,finished) in evidence {
            if *matches {
                selected+=1;
                if !*finished {return false;}
            }
        }
        selected==1
    }
    pub fn allow_managed_slot_close_after_failed_restore(slot:u32)->io::Result<bool> {
        if !(1..=5).contains(&slot) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile"));
        }
        let client=format!("{slot:02}");
        let mut state=SURFACE.lock().map_err(|_|io::Error::other("Workspace state unavailable"))?;
        if state.closing {return Ok(false);}
        let mut evidence=Vec::with_capacity(1+state.parked.len());
        if let Some(lease)=state.active.as_mut() {
            let matches=slot_lease(lease,&client);
            // The helper and reader must be quiescent, including failed
            // pre-Ready leases. The game controller separately verifies
            // original process identity before posting normal WM_CLOSE.
            let completed=matches && close_only_if_helpers_quiesced(
                lease.ended.is_some(),lease.child.try_wait()?.is_some());
            evidence.push((matches,completed));
        }
        for lease in &mut state.parked {
            let matches=slot_lease(lease,&client);
            // The helper and reader must be quiescent, including failed
            // pre-Ready leases. The game controller separately verifies
            // original process identity before posting normal WM_CLOSE.
            let completed=matches && close_only_if_helpers_quiesced(
                lease.ended.is_some(),lease.child.try_wait()?.is_some());
            evidence.push((matches,completed));
        }
        Ok(selected_helper_quiesced(&evidence))
    }
    // Called ONLY after the retained session controller positively completed
    // CloseOne. A previously failed native Restore can leave a cached End(err)
    // lease. Discard it only after the ORIGINAL game process is also proven
    // exited; otherwise keep its checkpoint and do not touch another HWND.
    // Retiring the checkpoint after a failed selected Restore is legal only
    // for one known lease of THIS slot. No matching lease is NOT proof that
    // restoration completed or that a foreign checkpoint can be discarded.
    fn require_unique_selected_checkpoint(matches: usize) -> io::Result<()> {
        match matches {
            1 => Ok(()),
            0 => Err(io::Error::new(io::ErrorKind::NotFound,
                "Selected Minecraft recovery checkpoint is missing; native restoration cannot be confirmed")),
            _ => Err(io::Error::other("Ambiguous Minecraft presentation checkpoints for one slot")),
        }
    }
    pub fn release_exited_managed_slot_checkpoint(slot:u32)->io::Result<()> {
        if !(1..=5).contains(&slot) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile"));
        }
        let client=format!("{slot:02}");
        let mut state=SURFACE.lock().map_err(|_|io::Error::other("Workspace state unavailable"))?;
        let active_match=state.active.as_ref().is_some_and(|lease|slot_lease(lease,&client));
        let parked_matches=state.parked.iter().filter(|lease|slot_lease(lease,&client)).count();
        require_unique_selected_checkpoint(usize::from(active_match)+parked_matches)?;
        if active_match {
            let lease=state.active.as_mut().unwrap();
            if !close_only_if_helpers_quiesced(lease.ended.is_some(),lease.child.try_wait()?.is_some())
                || !native::original_game_process_exited(&lease.requested).unwrap_or(false) {
                // A missing Saved is not process-exit evidence; this exact
                // pinned original lifetime must have ended before retirement.
                return Err(io::Error::new(io::ErrorKind::WouldBlock,
                    "Native checkpoint still active or original Minecraft exit unverified"));
            }
            state.active=None;
        }
        if let Some(index)=state.parked.iter().position(|lease|slot_lease(lease,&client)) {
            let lease=&mut state.parked[index];
            if !close_only_if_helpers_quiesced(lease.ended.is_some(),lease.child.try_wait()?.is_some())
                || !native::original_game_process_exited(&lease.requested).unwrap_or(false) {
                // A missing Saved is not process-exit evidence; this exact
                // pinned original lifetime must have ended before retirement.
                return Err(io::Error::new(io::ErrorKind::WouldBlock,
                    "Native checkpoint still active or original Minecraft exit unverified"));
            }
            state.parked.remove(index);
        }
        if state.active.is_none() && state.parked.is_empty() {
            state.suspended=false;
        }
        Ok(())
    }
    pub fn cancel_close() {if let Ok(mut state)=SURFACE.lock(){state.closing=false;}}
    pub fn release() -> io::Result<()> {
        let preview_error = previews::clear().err();
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        let mut error = preview_error;
        if let Some(lease) = state.active.iter().chain(state.parked.iter()).next() {
            // An aperture error must not skip independent per-profile Restore.
            // Preserve the failure receipt but attempt every original helper.
            if let Err(aperture) = native::clear_aperture(lease.manager) {
                if error.is_none() { error = Some(aperture); }
            }
        }
        if let Some(active) = state.active.as_mut() {
            match active.stop() {
                Ok(()) => state.active = None,
                Err(e) => { if error.is_none() { error = Some(e); } }
            }
        }
        state.parked.retain_mut(|lease| match lease.stop() {
            Ok(()) => false,
            Err(e) => {
                if error.is_none() {
                    error = Some(e);
                }
                true
            }
        });
        if let Some(error) = error {
            drop(state);
            super::native_timing::report();
            return Err(error);
        }
        state.suspended = false;
        state.live_previews = false;
        drop(state);
        // Aggregates only on explicit release, outside native lease lock.
        super::native_timing::report();
        Ok(())
    }
    // Restore only the slot's previously admitted native presentation; do
    // not touch windows leased by other Minecraft profiles or by other apps.
    pub fn release_managed_slot(slot:u32)->Result<(),(bool,io::Error)> {
        // The bool reports whether mutation of a game window was attempted.
        // An error AFTER that point must not be represented to Play as a
        // no-op: clear_aperture or helper Restore can partially succeed.
        if !(1..=5).contains(&slot) {
            return Err((false,io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile")));
        }
        let client=format!("{slot:02}");
        let owned=|lease:&Lease|slot_lease(lease,&client);
        // DWM thumbnails are observation only. A busy preview worker must
        // not prevent a user's confirmed *game* CloseOne. Remove only this
        // slot's preview, leaving MC01–MC05 peers intact.
        if let Err(error)=previews::clear_slot(slot) {
            eprintln!("Minecraft - {slot} preview refresh pending during normal close: {error}");
            previews::refresh();
        }
        let mut state=SURFACE.lock().map_err(|_|(false,io::Error::other("Workspace state unavailable")))?;
        if state.closing {return Err((false,io::Error::new(io::ErrorKind::WouldBlock,"Workspace is closing")));}
        if state.active.as_ref().is_some_and(&owned) {
            let manager=state.active.as_ref().unwrap().manager;
            native::clear_aperture(manager).map_err(|error|(true,error))?;
            state.active.as_mut().unwrap().stop().map_err(|error|(true,error))?;
            state.active=None;
        }
        if let Some(index)=state.parked.iter().position(&owned) {
            state.parked[index].stop().map_err(|error|(true,error))?;
            state.parked.remove(index);
        }
        Ok(())
    }
    pub fn suspend() -> io::Result<bool> {
        previews::clear()?;
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        if let Some(lease) = state.active.as_ref() {
            native::clear_aperture(lease.manager)?;
        }
        if let Some(lease) = state.active.as_mut() {
            if lease.child.try_wait()?.is_some()
                || lease
                    .saved
                    .as_ref()
                    .is_some_and(|saved| native::retired(&saved.target))
            {
                lease.stop()?;
                state.active = None;
            }
        }
        let live = state.live_previews;
        if let Some(lease) = state.active.as_mut() {
            let rect = native::rectangle(lease.manager, lease.viewport)?;
            lease.background(rect, live)?;
        }
        state.suspended = true;
        Ok(state.active.is_some())
    }
    pub fn release_target(target: &OwnedProcess) -> io::Result<()> {
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        if state
            .active
            .as_ref()
            .is_some_and(|lease| same(lease, target))
        {
            native::clear_aperture(state.active.as_ref().unwrap().manager)?;
            state.active.as_mut().unwrap().stop()?;
            state.active = None;
            return Ok(());
        }
        if let Some(index) = state.parked.iter().position(|lease| same(lease, target)) {
            state.parked[index].stop()?;
            state.parked.remove(index);
            return Ok(());
        }
        Err(io::Error::other("Workspace target no longer managed"))
    }
    fn prepare(manager: u64, viewport: Viewport, target: OwnedProcess) -> Result<Lease, PrepareFailure> {
        if !workspace_enabled() {
            return Err(io::Error::other(
                "Native workspace disabled outside authorized preview or developer build",
            ).into());
        }
        let viewport = viewport.validate()?;
        native::rectangle(manager, viewport)?;
        let recovery_gate = recovery_gate::create(&target)?;
        #[cfg(not(test))]
        let role_image=crate::engine::process_roles::image("recovery")?;
        #[cfg(not(test))]
        let executable = &role_image.path;
        #[cfg(test)]
        let executable = std::env::var_os("MIVUBI_TEST_SURFACE_EXECUTABLE")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("target/debug/m-bedrock-virtual-clients.exe")
            });
        let mut command = ProcessCommand::new(executable);
        command
            .arg("--workspace-surface")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x08000000);
        }
        let mut child = command.spawn()?;
        // No Start/capture request has been sent yet. Missing piped handles
        // are an invalid helper bootstrap; stop only this helper before the
        // parent checkpoint lease can be released, never the Minecraft game.
        let input = match child.stdin.take() {
            Some(input) => input,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::other("Surface stdin unavailable").into());
            }
        };
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(io::Error::other("Surface stdout unavailable").into());
            }
        };
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut saved = None;
            let mut reader = io::BufReader::new(stdout);
            let mut pipe_issue: Option<String> = None;
            let mut acknowledged_restore = false;
            loop {
                let text = match read_line(&mut reader) {
                    Ok(Some(text)) => text,
                    Ok(None) => break, // Real EOF, not an invalid/partial reply.
                    Err(error) => {
                        pipe_issue = Some(format!("Surface helper pipe read failed: {error}"));
                        break;
                    }
                };
                let mut reply = match serde_json::from_str::<Reply>(&text) {
                    Ok(reply) => reply,
                    Err(_) => {
                        // Never put arbitrary or malformed helper text into
                        // recovery diagnostics or treat it as a clean exit.
                        pipe_issue = Some("Invalid surface helper JSON reply".into());
                        break;
                    }
                };
                match &mut reply {
                    Reply::Ready(state) => {
                        state.manager_pid = std::process::id();
                        saved = Some(state.clone());
                    }
                    Reply::Restored => {
                        saved = None;
                        acknowledged_restore = true;
                    }
                    Reply::Error(error) => {
                        // The final outcome can still be successful if a later
                        // valid Restored or native fallback verifies recovery.
                        pipe_issue = Some(format!("Surface helper reported failure: {error}"));
                    }
                    _ => {}
                }
                if tx.send(Event::Reply(reply)).is_err() {
                    break;
                }
            }
            // Reader-side fallback is the existing exact-HWND/geometry
            // recovery owner. If there was a Saved checkpoint, native::restore
            // must actually succeed; malformed pipe input alone proves nothing.
            let recovered_checkpoint = saved.is_some();
            let native_recovery = saved
                .as_ref()
                .map_or(Ok(()), native::restore)
                .map_err(|error| error.to_string());
            let result = reader_terminal_result(
                pipe_issue,
                native_recovery,
                acknowledged_restore || recovered_checkpoint,
            );
            if let Some(saved) = saved.as_ref() {
                if let Ok(mut state) = SURFACE.try_lock() {
                    if state
                        .active
                        .as_ref()
                        .is_some_and(|lease| same(lease, &saved.target))
                    {
                        let _ = native::clear_aperture(manager);
                        state.suspended = true;
                    }
                }
            }
            let _ = tx.send(Event::End(result));
        });
        let mut lease = Lease {
            child,
            input: Some(input),
            events: rx,
            viewport,
            manager,
            authority: None,
            ended: None,
            saved: None,
            requested: target.clone(),
            last_confirmed_place: None,
            last_confirmed_hide: None,
            _recovery_gate: recovery_gate,
        };
        let start = lease.send(Command::Start(target.clone()));
        match start {
            Ok(Reply::Ready(saved)) if ready_matches_start(&saved, &target) => {
                lease.saved = Some(saved);
                Ok(lease)
            }
            other => {
                let detail = match other {
                    Ok(Reply::Ready(_)) =>
                        "Surface helper Ready identity differed from the requested process/window".to_owned(),
                    Ok(_) => "Surface helper returned an unexpected Start acknowledgement".to_owned(),
                    Err(error) => format!("Surface recovery helper could not prepare: {error}"),
                };
                // Do not silently discard a possibly active Start/Restore
                // helper. A failed native cleanup must return its pinned
                // lease to this same SURFACE owner; no retry/capture follows.
                match lease.stop() {
                    Ok(()) => Err(PrepareFailure {
                        error: io::Error::other(detail),
                        retained: None,
                    }),
                    Err(cleanup) => Err(PrepareFailure {
                        error: io::Error::other(format!(
                            "{detail}. Native recovery cleanup unconfirmed; original checkpoint retained: {cleanup}"
                        )),
                        retained: Some(lease),
                    }),
                }
            }
        }
    }
    pub fn present_verified(
        manager: u64,
        viewport: Viewport,
        target: OwnedProcess,
        session: String,
        client: String,
    ) -> io::Result<()> {
        present_bound(manager, viewport, target, Some((session, client)), false).map(|_| ())
    }
    pub(super) fn active_preview_slot(manager: u64) -> io::Result<Option<u32>> {
        let state = SURFACE.lock().map_err(|_| io::Error::other("Workspace state unavailable"))?;
        if state.active.iter().chain(state.parked.iter()).any(|lease|
            lease.manager != manager || lease.authority.as_ref().is_none_or(|(authority,_)| authority != "LAZY_TESTER_MANAGED_SESSION")) {
            return Err(io::Error::other("Release the existing workspace first"));
        }
        let Some(active) = &state.active else { return Ok(None); };
        let Some((authority, client)) = &active.authority else { return Err(io::Error::other("Release the existing workspace first")); };
        if active.manager != manager || authority != "LAZY_TESTER_MANAGED_SESSION" {
            return Err(io::Error::other("Release the existing workspace first"));
        }
        client.parse::<u32>().map(Some).map_err(io::Error::other)
    }
    pub(super) fn adopt_preview_sources(manager: u64, viewport: Viewport,
        sources: &[(u32, OwnedProcess)]) -> io::Result<Vec<(u32, String)>> {
        let rect = native::rectangle(manager, viewport.validate()?)?;
        let mut state = SURFACE.lock().map_err(|_| io::Error::other("Workspace state unavailable"))?;
        if state.closing { return Err(io::Error::other("Lazy Tester is closing")); }
        state.live_previews = true;
        let mut unavailable = Vec::new();
        // These are only diagnostics for the original, retained checkpoints.
        // Never drop a failed lease or globally abort preview for a peer.
        let mut cleanup_failures: Vec<(OwnedProcess, Option<(String, String)>, String)> = Vec::new();
        for index in (0..state.parked.len()).rev() {
            let due = match parked_cleanup_due(&mut state.parked[index]) {
                Ok(due) => due,
                Err(error) => {
                    let lease = &state.parked[index];
                    cleanup_failures.push((
                        lease.requested.clone(), lease.authority.clone(), error.to_string()));
                    continue;
                }
            };
            if due {
                match state.parked[index].stop() {
                    Ok(()) => { state.parked.remove(index); }
                    Err(error) => {
                        let lease = &state.parked[index];
                        cleanup_failures.push((
                            lease.requested.clone(), lease.authority.clone(), error.to_string()));
                    }
                }
            }
        }
        for (slot, target) in sources {
            let requested_authority = (
                "LAZY_TESTER_MANAGED_SESSION".to_owned(), format!("{slot:02}"));
            if let Some((_, _, detail)) = cleanup_failures.iter().find(|(pinned, owner, _)| {
                checkpoint_conflicts(pinned, owner.as_ref(),
                    target, Some(&requested_authority))
            }) {
                unavailable.push((*slot, format!(
                    "Minecraft - {slot} native recovery remains unconfirmed: {detail}")));
                continue;
            }
            // A terminal helper is no longer a live native preview even
            // when its previous Ready/Saved receipt was valid.
            if state.parked.iter().any(|lease|
                presentation_checkpoint_unverified(lease.saved.is_some(), lease.ended.is_some())
                && checkpoint_conflicts(&lease.requested, lease.authority.as_ref(),
                    target, Some(&requested_authority))) {
                unavailable.push((*slot,
                    "Original Minecraft native recovery checkpoint is unconfirmed; preview unavailable".into()));
                continue;
            }
            if state.active.as_ref().is_some_and(|active| same(active, target)) { continue; }
            if !target.responding || !native::can_park(target).unwrap_or(false) {
                unavailable.push((*slot, "Jendela belum tersedia atau dialog Minecraft sedang terbuka".into())); continue;
            }
            let existing = state.parked.iter().position(|lease| same(lease, target));
            if existing.is_none() && state.parked.len() + usize::from(state.active.is_some()) >= 5 {
                unavailable.push((*slot, "Workspace already manages five clients".into()));
                continue;
            }
            let mut lease = match existing {
                Some(index) => state.parked.remove(index),
                None => match prepare(manager, viewport, target.clone()) {
                    Ok(lease) => lease,
                    Err(mut failed) => {
                        if let Some(mut pending) = failed.retained.take() {
                            // Reuse the one existing SURFACE lease list. An
                            // unknown Start outcome never authorizes a fresh
                            // helper or changes another slot's presentation.
                            pending.authority = Some((
                                "LAZY_TESTER_MANAGED_SESSION".into(),
                                format!("{slot:02}"),
                            ));
                            state.parked.push(pending);
                        }
                        unavailable.push((*slot, failed.error.to_string()));
                        continue;
                    }
                },
            };
            lease.authority = Some(("LAZY_TESTER_MANAGED_SESSION".into(), format!("{slot:02}")));
            lease.viewport = viewport;
            if let Err(error) = lease.background(rect, true) { unavailable.push((*slot,error.to_string())); }
            state.parked.push(lease);
        }
        Ok(unavailable)
    }
    pub fn present_retained(
        manager: u64,
        viewport: Viewport,
        target: OwnedProcess,
        session: String,
        client: String,
    ) -> io::Result<bool> {
        present_bound(manager, viewport, target, Some((session, client)), true)
    }
    #[cfg(test)]
    fn present(manager: u64, viewport: Viewport, target: OwnedProcess) -> io::Result<()> {
        present_bound(manager, viewport, target, None, false).map(|_| ())
    }
    fn present_bound(
        manager: u64,
        viewport: Viewport,
        target: OwnedProcess,
        authority: Option<(String, String)>,
        retained_only: bool,
    ) -> io::Result<bool> {
        let viewport = viewport.validate()?;
        let rect = native::rectangle(manager, viewport)?;
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        // Only a previously provider-verified lease may skip expensive engine admission.
        if state.closing { return Err(io::Error::other("Lazy Tester is closing")); }
        if retained_only
            && !state.active.iter().chain(state.parked.iter()).any(|lease| {
                lease.manager == manager
                    && lease.authority == authority
                    && same(lease, &target)
                    && lease
                        .saved
                        .as_ref()
                        .is_some_and(|saved| !native::retired(&saved.target))
            })
        {
            return Ok(false);
        }
        // A finished recovery helper must confirm restoration before its slot is reused.
        if let Some(active) = state.active.as_mut() {
            if active.child.try_wait()?.is_some()
                || active
                    .saved
                    .as_ref()
                    .is_some_and(|s| native::retired(&s.target))
            {
                active.stop()?;
                state.active = None;
            }
        }
        // A failed parked MC01 recovery is not an MC02 Play failure.
        // Retain its exact lease/gate and continue, but reject the requested
        // target if the error is on the same process/window or managed slot.
        for index in (0..state.parked.len()).rev() {
            let due = match parked_cleanup_due(&mut state.parked[index]) {
                Ok(due) => due,
                Err(error) => {
                    let lease = &state.parked[index];
                    if checkpoint_conflicts(&lease.requested, lease.authority.as_ref(),
                        &target, authority.as_ref()) {
                        return Err(error);
                    }
                    continue;
                }
            };
            if due {
                match state.parked[index].stop() {
                    Ok(()) => { state.parked.remove(index); }
                    Err(error) => {
                        let lease = &state.parked[index];
                        if checkpoint_conflicts(&lease.requested, lease.authority.as_ref(),
                            &target, authority.as_ref()) {
                            return Err(io::Error::new(error.kind(), format!(
                                "Selected Minecraft checkpoint recovery is unconfirmed: {error}")));
                        }
                        // Unrelated failure: never drop or mutate this lease.
                    }
                }
            }
        }
        if state.parked.iter().any(|lease|
            presentation_checkpoint_unverified(lease.saved.is_some(), lease.ended.is_some())
            && checkpoint_conflicts(&lease.requested, lease.authority.as_ref(),
                &target, authority.as_ref())) {
            return Err(io::Error::new(io::ErrorKind::WouldBlock,
                "Selected Minecraft native checkpoint is unconfirmed; original helper and recovery gate retained"));
        }
        if retained_only
            && !state.active.iter().chain(state.parked.iter()).any(|lease| {
                lease.manager == manager && lease.authority == authority && same(lease, &target)
            })
        {
            return Ok(false);
        }
        if let Some(old) = state.active.as_mut() {
            if same(old, &target) {
                if old.manager != manager {
                    return Err(io::Error::other("Wrong workspace window"));
                }
                old.authority = authority;
                old.viewport = viewport;
                if let Err(error) = old.place(rect, true) {
                    state.suspended = matches!(
                        error.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::BrokenPipe
                    );
                    if state.suspended {
                        native::clear_aperture(manager)?;
                    }
                    return Err(error);
                }
                state.suspended = false;
                return Ok(true);
            }
        }
        let parked = state.parked.iter().position(|lease| same(lease, &target));
        if parked.is_none() && state.parked.len() + usize::from(state.active.is_some()) >= 5 {
            return Err(io::Error::other("Workspace already manages five clients"));
        }
        // A retained lease owns the original checkpoint, never recapture a hidden tab.
        let mut replacement = if let Some(index) = parked {
            state.parked.remove(index)
        } else {
            match prepare(manager, viewport, target) {
                Ok(lease) => lease,
                Err(mut failed) => {
                    if let Some(mut pending) = failed.retained.take() {
                        // The lease is still owned here even if no Ready was
                        // acknowledged; its native checkpoint cannot be
                        // recaptured while early Restore remains uncertain.
                        pending.authority = authority.clone();
                        state.parked.push(pending);
                    }
                    return Err(failed.error);
                }
            }
        };
        replacement.authority = authority;
        if replacement.manager != manager {
            state.parked.push(replacement);
            return Err(io::Error::other("Wrong workspace window"));
        }
        replacement.viewport = viewport;
        let live_previews = state.live_previews;
        let outcome = replacement.place(rect, true);
        if outcome.is_err() {
            let detail = outcome
                .err()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Invalid surface acknowledgement".into());
            let parked = replacement.background(rect, live_previews);
            state.parked.push(replacement);
            let had_previous = state.active.is_some();
            let restored = if let Some(old) = state.active.as_mut() {
                old.place(rect, false)
            } else {
                native::clear_aperture(manager)
            };
            let recovery = rollback_error(&parked, &restored);
            state.suspended = recovery.is_some();
            if let Some(recovery) = recovery {
                return Err(io::Error::other(format!("Workspace placement failed: {detail}. Recovery was not confirmed: {recovery}.")));
            }
            let status = if had_previous { "Previous view position restored; select it to play." }
                else { "No game was selected; try again." };
            return Err(io::Error::other(format!(
                "Workspace placement failed: {detail}. {status}"
            )));
        }
        if let Some(old) = state.active.as_mut() {
            if let Err(error) = old.background(rect, live_previews) {
                let parked = replacement.background(rect, live_previews);
                let restored = old.place(rect, false);
                state.parked.push(replacement);
                let recovery = rollback_error(&parked, &restored);
                state.suspended = recovery.is_some();
                let status = recovery.map_or_else(|| "Previous view position restored; select it to play.".to_string(),
                    |detail| format!("Recovery was not confirmed: {detail}."));
                return Err(io::Error::other(format!("Previous surface could not suspend: {error}. {status}")));
            }
        }
        if let Some(old) = state.active.take() {
            state.parked.push(old);
        }
        state.active = Some(replacement);
        state.suspended = false;
        Ok(true)
    }
    #[cfg(windows)]
    pub fn follow_manager(manager: u64) -> io::Result<bool> {
        // This runs on the UI thread. DWM follows independently; native placement must
        // use the ordered helper on the existing worker, never an async HWND move that
        // can execute after a later acknowledged selection and park the active game.
        let Ok(state) = SURFACE.try_lock() else { return Ok(false); };
        if state.suspended || state.closing { return Ok(true); }
        let Some(lease) = &state.active else { return Ok(true); };
        if lease.manager != manager || native::minimized(manager)? { return Ok(true); }
        Ok(false)
    }
    pub fn update(manager: u64, viewport: Option<Viewport>) -> io::Result<()> {
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        // Settings and failed presentation have already hidden the active lease.
        // Native focus/resize notifications cannot retry it behind the UI's back.
        if state.suspended || state.closing {
            return Ok(());
        }
        let live = state.live_previews;
        let Some(lease) = state.active.as_mut() else {
            return Ok(());
        };
        if manager != lease.manager {
            return Err(io::Error::other("Wrong workspace window"));
        }
        let next_viewport = match viewport {
            Some(value) => value.validate()?,
            None => lease.viewport,
        };
        if native::minimized(manager)? {
            native::clear_aperture(manager)?;
            return lease.background(Rectangle { x:0,y:0,
                width:(lease.viewport.width*lease.viewport.pixel_ratio).round() as i32,
                height:(lease.viewport.height*lease.viewport.pixel_ratio).round() as i32 }, live);
        }
        let rect = match native::rectangle(manager, next_viewport) {
            Ok(rect) => rect,
            // Native resize/DPI events precede the authoritative webview geometry.
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) => return Err(error),
        };
        lease.viewport = next_viewport;
        let result = if live && lease.saved.as_ref().is_some_and(|saved| !native::receiving_input(&saved.target)) {
            // Keep native gameplay independent; a click on its DWM image reuses present_bound.
            // The parked surface stays live while the manager is moved/resized or configured.
            lease.background(rect, true).and_then(|()| {
                // Focus can change while the helper is parking the game.
                if lease.saved.as_ref().is_some_and(|saved| native::receiving_input(&saved.target)) {
                    lease.place(rect, false)
                } else { Ok(()) }
            })
        } else { lease.place(rect, false) };
        // A transient layout/stacking failure must not hide the live session or latch suspension.
        match result {
            Err(error) => {
                state.suspended = matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::BrokenPipe
                );
                if state.suspended {
                    native::clear_aperture(manager)?;
                }
                Err(error)
            }
            Ok(()) => Ok(()),
        }
    }

    #[cfg(all(test, windows))]
    mod windows_tests {
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
    }
}
// Unsupported platforms retain the same command signatures without loading
// Win32 recovery process code or pretending to own a Minecraft HWND.
#[cfg(not(windows))]
#[path = "workspace_surface/unsupported_controller.rs"]
mod controller;
pub use controller::{
    is_active, present_retained, present_verified, release, release_for_close, allow_game_close_after_failed_restore, allow_managed_slot_close_after_failed_restore, release_exited_managed_slot_checkpoint, cancel_close, release_target, release_managed_slot, suspend, update,
};
#[cfg(windows)]
pub use controller::follow_manager;

pub fn configure_previews(window: tauri::Window, viewport: Viewport, tiles: Vec<PreviewTile>,
    profiles: Vec<(u32,OwnedProcess)>) -> io::Result<PreviewReport> {
    previews::configure(window,viewport,tiles,profiles)
}
pub fn clear_previews() -> io::Result<()> { previews::clear() }
pub fn refresh_previews() { previews::refresh(); }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn focused_game_outside_viewport_is_not_an_interactive_surface() {
        let viewport=Rectangle{x:78,y:141,width:700,height:450};
        let parked=Rectangle{x:2592,y:0,..viewport};
        assert!(!interactive_surface(true,true,parked,viewport));
        assert!(!interactive_surface(false,true,viewport,viewport));
        assert!(!interactive_surface(true,false,viewport,viewport));
        assert!(interactive_surface(true,true,viewport,viewport));
    }
    #[cfg(windows)]
    #[test]
    fn clearing_aperture_repaints_children_even_when_region_is_already_clear() {
        use windows_sys::Win32::{Graphics::Gdi::*, UI::WindowsAndMessaging::*};
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        unsafe {
            let manager = CreateWindowExW(0, class.as_ptr(), class.as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                0, 0, 300, 200, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null());
            assert!(!manager.is_null());
            let child = CreateWindowExW(0, class.as_ptr(), class.as_ptr(), WS_CHILD | WS_VISIBLE,
                0, 0, 200, 100, manager, std::ptr::null_mut(), std::ptr::null_mut(), std::ptr::null());
            let outcome = std::panic::catch_unwind(|| {
                assert!(!child.is_null());
                ValidateRect(manager, std::ptr::null());
                ValidateRect(child, std::ptr::null());
                assert_eq!(GetUpdateRect(child, std::ptr::null_mut(), 0), 0);
                native::clear_aperture(manager as usize as u64).unwrap();
                assert_ne!(GetUpdateRect(child, std::ptr::null_mut(), 0), 0, "WebView child was not invalidated");
                let region = CreateRectRgn(0, 0, 0, 0);
                assert_eq!(GetWindowRgn(manager, region), ERROR);
                DeleteObject(region);
            });
            DestroyWindow(manager);
            if let Err(error) = outcome { std::panic::resume_unwind(error); }
        }
    }
    #[test]
    fn app_switch_during_placement_is_not_a_failed_foreground_surface() {
        let game = 11;
        let manager = 22;
        let other_app = 33;
        // Replay the observed failure: placement began focused, then another app took focus.
        assert!(!foreground_surface_covered(
            other_app, game, manager, manager
        ));
        assert!(foreground_surface_covered(manager, game, manager, manager));
        assert!(foreground_surface_covered(game, game, manager, other_app));
        assert!(!foreground_surface_covered(manager, game, manager, game));
    }
    #[test]
    fn viewport_rejects_invalid_or_unbounded_layouts() {
        for viewport in [
            Viewport {
                x: f64::NAN,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                pixel_ratio: 1.0,
            },
            Viewport {
                x: -1.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
                pixel_ratio: 1.0,
            },
            Viewport {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 600.0,
                pixel_ratio: 1.0,
            },
        ] {
            assert!(viewport.validate().is_err())
        }
        assert!(Viewport {
            x: 20.0,
            y: 50.0,
            width: 800.0,
            height: 600.0,
            pixel_ratio: 1.0
        }
        .validate()
        .is_ok());
    }
    #[test]
    fn helper_input_is_bounded_and_requires_whole_messages() {
        assert!(read_line(&mut io::Cursor::new(vec![b'x'; 4097])).is_err());
        assert!(read_line(&mut io::Cursor::new(b"unterminated")).is_err());
        assert_eq!(
            read_line(&mut io::Cursor::new(b"{}\n")).unwrap(),
            Some("{}\n".into())
        );
    }
}
