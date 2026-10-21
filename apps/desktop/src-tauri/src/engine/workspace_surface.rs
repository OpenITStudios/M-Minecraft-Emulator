//! Presentation only: independent top-level game, no parenting or injected input.
use super::owned_provider::OwnedProcess;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

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
        if [self.x, self.y, self.width, self.height, self.pixel_ratio]
            .iter()
            .any(|v| !v.is_finite())
            || self.x < 0.0
            || self.y < 0.0
            || self.width < 160.0
            || self.height < 120.0
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
fn foreground_surface_covered(foreground: u64, game: u64, manager: u64, point_root: u64) -> bool {
    (foreground == game || foreground == manager) && point_root != game
}
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Saved {
    target: OwnedProcess,
    style: i64,
    extended_style: i64,
    rect: Rectangle,
    manager_pid: u32,
}
#[derive(Deserialize, Serialize)]
enum Command {
    Start(OwnedProcess),
    Place { rect: Rectangle, manager_pid: u32 },
    Hide,
    Restore,
}
#[derive(Deserialize, Serialize)]
enum Reply {
    Progress(String),
    Ready(Saved),
    Placed,
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

pub fn maybe_run_helper() -> bool {
    if std::env::args().nth(1).as_deref() != Some("--workspace-surface") {
        return false;
    }
    if !cfg!(debug_assertions) {
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
    let result = (|| -> io::Result<()> {
        while let Some(text) = read_line(&mut input)? {
            let command: Command = serde_json::from_str(&text)?;
            let reply = match command {
                Command::Start(target) if saved.is_none() => {
                    line(&Reply::Progress("start/capture".into()), &mut output)?;
                    let state = native::capture(target)?;
                    saved = Some(state.clone());
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
                Command::Restore => {
                    if let Some(state) = &saved {
                        native::restore_traced(state, &mut |stage| {
                            line(&Reply::Progress(stage.into()), &mut output)
                        })?;
                    }
                    saved = None;
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

#[cfg(windows)]
mod native {
    use super::*;
    use windows_sys::Win32::System::RemoteDesktop::ProcessIdToSessionId;
    use windows_sys::Win32::{Foundation::*, System::Threading::*, UI::WindowsAndMessaging::*};
    struct ProcessHandle(HANDLE);
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
            if !permitted_image(&image, &std::env::current_exe()?.canonicalize()?) {
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
    fn hwnd(target: &OwnedProcess) -> io::Result<(ProcessHandle, HWND)> {
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
    pub(super) fn capture(target: OwnedProcess) -> io::Result<Saved> {
        let (_process, window) = hwnd(&target)?;
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
    pub(super) fn aperture(manager: u64, rect: Option<Rectangle>) -> io::Result<()> {
        use windows_sys::Win32::Graphics::Gdi::*;
        let manager = manager as usize as HWND;
        unsafe {
            let mut pid = 0;
            GetWindowThreadProcessId(manager, &mut pid);
            if IsWindow(manager) == 0 || pid != std::process::id() {
                return Err(io::Error::other("Invalid workspace aperture owner"));
            }
            let Some(rect) = rect else {
                let current = CreateRectRgn(0, 0, 0, 0);
                if current.is_null() {
                    return Err(io::Error::last_os_error());
                }
                let absent = GetWindowRgn(manager, current) == ERROR;
                DeleteObject(current);
                if absent {
                    return Ok(());
                }
                if SetWindowRgn(manager, std::ptr::null_mut(), 1) == 0 {
                    return Err(io::Error::last_os_error());
                }
                return Ok(());
            };
            let mut bounds: RECT = std::mem::zeroed();
            if GetWindowRect(manager, &mut bounds) == 0 {
                return Err(io::Error::last_os_error());
            }
            let outer = CreateRectRgn(0, 0, bounds.right - bounds.left, bounds.bottom - bounds.top);
            let hole = CreateRectRgn(
                rect.x - bounds.left,
                rect.y - bounds.top,
                rect.x - bounds.left + rect.width,
                rect.y - bounds.top + rect.height,
            );
            if outer.is_null() || hole.is_null() {
                if !outer.is_null() {
                    DeleteObject(outer);
                }
                if !hole.is_null() {
                    DeleteObject(hole);
                }
                return Err(io::Error::last_os_error());
            }
            let combined = CombineRgn(outer, outer, hole, RGN_DIFF);
            DeleteObject(hole);
            let current = CreateRectRgn(0, 0, 0, 0);
            if current.is_null() {
                DeleteObject(outer);
                return Err(io::Error::last_os_error());
            }
            // SetWindowRgn emits window-position events; equal regions must not create a loop.
            let unchanged =
                GetWindowRgn(manager, current) != ERROR && EqualRgn(current, outer) != 0;
            DeleteObject(current);
            if unchanged {
                DeleteObject(outer);
                return Ok(());
            }
            if combined == ERROR || SetWindowRgn(manager, outer, 1) == 0 {
                DeleteObject(outer);
                return Err(io::Error::other("Workspace aperture update refused"));
            }
            // Windows now owns outer. The game remains an independent top-level window.
            Ok(())
        }
    }
    pub(super) fn confirm_foreground(
        saved: &Saved,
        rect: Rectangle,
        manager: u64,
        activate: bool,
    ) -> io::Result<()> {
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
    }
    pub(super) fn hide(saved: &Saved) -> io::Result<()> {
        let (_process, window) = hwnd(&saved.target)?;
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
        Ok(())
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
                // An exited/reused process must never authorize touching a replacement HWND.
                if retired(&saved.target) {
                    return Ok(());
                }
                return Err(error);
            }
        };
        unsafe {
            let frame_mask = (WS_CAPTION | WS_THICKFRAME | WS_MAXIMIZE) as i64;
            let shell_mask = (WS_EX_APPWINDOW | WS_EX_TOOLWINDOW) as i64;
            let shell = GetWindowLongPtrW(window, GWL_EXSTYLE) as i64;
            let restored_shell = (shell & !shell_mask) | (saved.extended_style & shell_mask);
            if shell != restored_shell {
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
        }
        Ok(())
    }
    pub(super) fn retired(target: &OwnedProcess) -> bool {
        unsafe {
            let Some(key) = super::super::instance_identity::parse_window_identity(
                target.pid,
                Some(&target.created_file_time),
                Some(&target.window_handle),
            ) else {
                return false;
            };
            let window = key.hwnd as usize as HWND;
            let mut pid = 0;
            GetWindowThreadProcessId(window, &mut pid);
            if IsWindow(window) == 0 || pid != target.pid {
                return true;
            }
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, target.pid);
            if process.is_null() {
                return GetLastError() == ERROR_INVALID_PARAMETER;
            }
            let mut created: FILETIME = std::mem::zeroed();
            let mut a = created;
            let mut b = created;
            let mut c = created;
            let ok = GetProcessTimes(process, &mut created, &mut a, &mut b, &mut c);
            CloseHandle(process);
            ok != 0
                && (((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64)
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
    pub(super) fn aperture(_: u64, _: Option<Rectangle>) -> io::Result<()> {
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
}

mod controller {
    use super::*;
    use std::{
        process::{Child, ChildStdin, Command as ProcessCommand, Stdio},
        sync::{mpsc, Mutex},
        time::{Duration, Instant},
    };
    enum Event {
        Reply(Reply),
        End(Result<(), String>),
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
    }
    impl Lease {
        fn place(&mut self, rect: Rectangle, activate: bool) -> io::Result<()> {
            if !matches!(
                self.send(Command::Place {
                    rect,
                    manager_pid: std::process::id(),
                })?,
                Reply::Placed
            ) {
                return Err(io::Error::other("Surface presentation failed"));
            }
            // A maximized foreground manager must not cover the independently rendered game.
            native::aperture(self.manager, Some(rect))?;
            native::confirm_foreground(
                self.saved
                    .as_ref()
                    .ok_or_else(|| io::Error::other("Surface checkpoint missing"))?,
                rect,
                self.manager,
                activate,
            )
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
        fn stop(&mut self) -> io::Result<()> {
            if let Some(result) = &self.ended {
                return result.clone().map_err(io::Error::other);
            }
            let restored = matches!(self.send(Command::Restore), Ok(Reply::Restored));
            if let Some(result) = &self.ended {
                return result.clone().map_err(io::Error::other);
            }
            if !restored && self.child.try_wait()?.is_none() {
                self.child.kill()?;
            }
            match self.events.recv_timeout(Duration::from_secs(3)) {
                Ok(Event::End(result)) => {
                    self.ended = Some(result.clone());
                    result.map_err(io::Error::other)
                }
                _ => Err(io::Error::other(
                    "Window recovery not confirmed; do not start another surface",
                )),
            }
        }
    }
    struct Surfaces {
        active: Option<Lease>,
        parked: Vec<Lease>,
        suspended: bool,
    }
    static SURFACE: Mutex<Surfaces> = Mutex::new(Surfaces {
        active: None,
        parked: Vec::new(),
        suspended: false,
    });
    fn same(lease: &Lease, target: &OwnedProcess) -> bool {
        lease.saved.as_ref().is_some_and(|s| {
            s.target.pid == target.pid
                && s.target.created_file_time == target.created_file_time
                && s.target.session_id == target.session_id
                && s.target.window_handle == target.window_handle
        })
    }
    pub fn is_active() -> bool {
        SURFACE
            .try_lock()
            .map(|s| s.active.is_some() || !s.parked.is_empty())
            .unwrap_or(true)
    }
    pub fn release() -> io::Result<()> {
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        let mut error = None;
        if let Some(lease) = state.active.iter().chain(state.parked.iter()).next() {
            native::aperture(lease.manager, None)?;
        }
        if let Some(active) = state.active.as_mut() {
            match active.stop() {
                Ok(()) => state.active = None,
                Err(e) => error = Some(e),
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
            return Err(error);
        }
        state.suspended = false;
        Ok(())
    }
    pub fn suspend() -> io::Result<bool> {
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        if let Some(lease) = state.active.as_ref() {
            native::aperture(lease.manager, None)?;
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
        if let Some(lease) = state.active.as_mut() {
            if !matches!(lease.send(Command::Hide)?, Reply::Hidden) {
                return Err(io::Error::other("Surface suspend failed"));
            }
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
            native::aperture(state.active.as_ref().unwrap().manager, None)?;
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
    fn prepare(manager: u64, viewport: Viewport, target: OwnedProcess) -> io::Result<Lease> {
        if !cfg!(debug_assertions) {
            return Err(io::Error::other("Developer workspace only"));
        }
        let viewport = viewport.validate()?;
        native::rectangle(manager, viewport)?;
        #[cfg(not(test))]
        let executable = std::env::current_exe()?;
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
        let input = child
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("Surface stdin unavailable"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("Surface stdout unavailable"))?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut saved = None;
            let mut reader = io::BufReader::new(stdout);
            while let Ok(Some(text)) = read_line(&mut reader) {
                let Ok(mut reply) = serde_json::from_str::<Reply>(&text) else {
                    break;
                };
                match &mut reply {
                    Reply::Ready(state) => {
                        state.manager_pid = std::process::id();
                        saved = Some(state.clone())
                    }
                    Reply::Restored => saved = None,
                    _ => {}
                }
                if tx.send(Event::Reply(reply)).is_err() {
                    break;
                }
            }
            // If the helper alone crashes, the manager still owns the checkpoint.
            let result = saved
                .as_ref()
                .map_or(Ok(()), native::restore)
                .map_err(|e| e.to_string());
            if let Some(saved) = saved.as_ref() {
                if let Ok(mut state) = SURFACE.try_lock() {
                    if state
                        .active
                        .as_ref()
                        .is_some_and(|lease| same(lease, &saved.target))
                    {
                        let _ = native::aperture(manager, None);
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
        };
        match lease.send(Command::Start(target)) {
            Ok(Reply::Ready(saved)) => lease.saved = Some(saved),
            _ => {
                let _ = lease.stop();
                return Err(io::Error::other(
                    "Surface recovery helper could not prepare",
                ));
            }
        }
        Ok(lease)
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
        for index in (0..state.parked.len()).rev() {
            if state.parked[index].child.try_wait()?.is_some()
                || state.parked[index]
                    .saved
                    .as_ref()
                    .is_some_and(|s| native::retired(&s.target))
            {
                state.parked[index].stop()?;
                state.parked.remove(index);
            }
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
                        native::aperture(manager, None)?;
                    }
                    return Err(error);
                }
                state.suspended = false;
                return Ok(true);
            }
        }
        let parked = state.parked.iter().position(|lease| same(lease, &target));
        if parked.is_none() && state.parked.len() + usize::from(state.active.is_some()) >= 4 {
            return Err(io::Error::other("Workspace already manages four clients"));
        }
        // A retained lease owns the original checkpoint, never recapture a hidden tab.
        let mut replacement = if let Some(index) = parked {
            state.parked.remove(index)
        } else {
            prepare(manager, viewport, target)?
        };
        replacement.authority = authority;
        if replacement.manager != manager {
            state.parked.push(replacement);
            return Err(io::Error::other("Wrong workspace window"));
        }
        replacement.viewport = viewport;
        let outcome = replacement.place(rect, true);
        if outcome.is_err() {
            let detail = outcome
                .err()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Invalid surface acknowledgement".into());
            let _ = replacement.send(Command::Hide);
            state.parked.push(replacement);
            if let Some(old) = state.active.as_mut() {
                old.place(rect, false)?;
                state.suspended = false;
            } else {
                native::aperture(manager, None)?;
            }
            return Err(io::Error::other(format!(
                "Workspace placement failed: {detail}. Previous session preserved."
            )));
        }
        if let Some(old) = state.active.as_mut() {
            if !matches!(old.send(Command::Hide), Ok(Reply::Hidden)) {
                let _ = replacement.send(Command::Hide);
                state.parked.push(replacement);
                return Err(io::Error::other(
                    "Previous surface could not suspend; restore windows before continuing",
                ));
            }
        }
        if let Some(old) = state.active.take() {
            state.parked.push(old);
        }
        state.active = Some(replacement);
        state.suspended = false;
        Ok(true)
    }
    pub fn update(manager: u64, viewport: Option<Viewport>) -> io::Result<()> {
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        // Settings and failed presentation have already hidden the active lease.
        // Native focus/resize notifications cannot retry it behind the UI's back.
        if state.suspended {
            return Ok(());
        }
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
            native::aperture(manager, None)?;
            return match lease.send(Command::Hide)? {
                Reply::Hidden => Ok(()),
                _ => Err(io::Error::other("Surface hide failed")),
            };
        }
        let rect = match native::rectangle(manager, next_viewport) {
            Ok(rect) => rect,
            // Native resize/DPI events precede the authoritative webview geometry.
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) => return Err(error),
        };
        lease.viewport = next_viewport;
        // A transient layout/stacking failure must not hide the live session or latch suspension.
        match lease.place(rect, false) {
            Err(error) => {
                state.suspended = matches!(
                    error.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::BrokenPipe
                );
                if state.suspended {
                    native::aperture(manager, None)?;
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
            let directory = root.join("workspace").join(format!(
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
                        assert_ne!(GetWindowRgn(manager, region), ERROR);
                        let mut bounds: RECT = std::mem::zeroed();
                        GetWindowRect(manager, &mut bounds);
                        assert_eq!(
                            PtInRegion(
                                region,
                                composed.rect.x - bounds.left + composed.rect.width / 2,
                                composed.rect.y - bounds.top + composed.rect.height / 2
                            ),
                            0,
                            "Manager covers the game's center"
                        );
                        assert_ne!(PtInRegion(region, 5, 5), 0, "Manager chrome was cut out");
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
            // Browser-like round trips retain all four process identities and checkpoints.
            let (_third_fixture, third) = fixture();
            let (_fourth_fixture, fourth) = fixture();
            let tabs = [target.clone(), second.clone(), third, fourth];
            for tab in &tabs {
                present(manager as usize as u64, viewport, tab.clone()).unwrap();
            }
            for turn in 0..12 {
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
            let mut timings = Vec::new();
            for turn in 0..40 {
                let selected = turn % 4;
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
            present(manager as usize as u64, viewport, second).unwrap();
            release().unwrap();
            unsafe {
                DestroyWindow(manager);
            }
        }
    }
}
pub use controller::{
    is_active, present_retained, present_verified, release, release_target, suspend, update,
};

#[cfg(test)]
mod tests {
    use super::*;
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
