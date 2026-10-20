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
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Saved {
    target: OwnedProcess,
    style: i64,
    rect: Rectangle,
    manager_pid: u32,
}
#[derive(Deserialize, Serialize)]
enum Command {
    Start(OwnedProcess),
    Place {
        rect: Rectangle,
        focus: bool,
        manager_pid: u32,
        manager_window: u64,
    },
    Hide,
    Restore,
}
#[derive(Deserialize, Serialize)]
enum Reply {
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
                    let state = native::capture(target)?;
                    saved = Some(state.clone());
                    Reply::Ready(state)
                }
                Command::Place {
                    rect,
                    focus,
                    manager_pid,
                    manager_window,
                } => {
                    let state = saved
                        .as_mut()
                        .ok_or_else(|| io::Error::other("Surface not prepared"))?;
                    state.manager_pid = manager_pid;
                    native::place(state, rect, focus, manager_pid, manager_window)?;
                    Reply::Placed
                }
                Command::Hide => {
                    native::hide(
                        saved
                            .as_ref()
                            .ok_or_else(|| io::Error::other("Surface not prepared"))?,
                    )?;
                    Reply::Hidden
                }
                Command::Restore => {
                    if let Some(state) = &saved {
                        native::restore(state)?;
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
                return Err(io::Error::last_os_error());
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
        focus: bool,
        manager_pid: u32,
        manager_window: u64,
    ) -> io::Result<()> {
        if rect.width < 160 || rect.height < 120 || rect.width > 20000 || rect.height > 20000 {
            return Err(io::Error::other("Invalid native viewport"));
        }
        let (_process, window) = hwnd(&saved.target)?;
        unsafe {
            // No owner/parent and no topmost flag: other applications can cover it normally.
            if IsZoomed(window) != 0 {
                ShowWindow(window, SW_SHOWNOACTIVATE);
            }
            let presentation_style =
                saved.style & !((WS_CAPTION | WS_THICKFRAME | WS_MAXIMIZE) as i64);
            let frame_changed = GetWindowLongPtrW(window, GWL_STYLE) as i64 != presentation_style;
            if frame_changed {
                style(window, presentation_style)?;
            }
            let foreground = GetForegroundWindow();
            let mut foreground_pid = 0;
            GetWindowThreadProcessId(foreground, &mut foreground_pid);
            let raise = (foreground == window
                || (foreground_pid == manager_pid
                    && foreground == manager_window as usize as HWND))
                && GetWindowLongPtrW(window, GWL_STYLE) & WS_DISABLED as isize == 0;
            if SetWindowPos(
                window,
                if raise || foreground.is_null() {
                    std::ptr::null_mut()
                } else {
                    foreground
                },
                rect.x,
                rect.y,
                rect.width,
                rect.height,
                (if frame_changed { SWP_FRAMECHANGED } else { 0 })
                    | SWP_NOACTIVATE
                    | SWP_SHOWWINDOW
                    | if !raise && foreground.is_null() {
                        SWP_NOZORDER
                    } else {
                        0
                    },
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
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
            if focus && raise && GetForegroundWindow() != window {
                SetForegroundWindow(window);
            }
        }
        Ok(())
    }
    pub(super) fn hide(saved: &Saved) -> io::Result<()> {
        let (_process, window) = hwnd(&saved.target)?;
        unsafe {
            ShowWindow(window, SW_HIDE);
        }
        Ok(())
    }
    pub(super) fn foreground(manager: u64) -> bool {
        unsafe { GetForegroundWindow() == manager as usize as HWND }
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
            style(window, saved.style)?;
            let r = saved.rect;
            let foreground = GetForegroundWindow();
            let mut pid = 0;
            GetWindowThreadProcessId(foreground, &mut pid);
            let lower = foreground != window && pid == saved.manager_pid;
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
    fn retired(target: &OwnedProcess) -> bool {
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
                return Err(io::Error::other(
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
                return Err(io::Error::other("Workspace layout changed; try again"));
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
    pub(super) fn place(_: &Saved, _: Rectangle, _: bool, _: u32, _: u64) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn hide(_: &Saved) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn restore(_: &Saved) -> io::Result<()> {
        unavailable()
    }
    pub(super) fn rectangle(_: u64, _: Viewport) -> io::Result<Rectangle> {
        unavailable()
    }
    pub(super) fn foreground(_: u64) -> bool {
        false
    }
    pub(super) fn minimized(_: u64) -> io::Result<bool> {
        unavailable()
    }
}

mod controller {
    use super::*;
    use std::{
        process::{Child, ChildStdin, Command as ProcessCommand, Stdio},
        sync::{mpsc, Mutex},
        time::Duration,
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
        ended: Option<Result<(), String>>,
        saved: Option<Saved>,
    }
    impl Lease {
        fn receive(&mut self) -> io::Result<Reply> {
            match self.events.recv_timeout(Duration::from_secs(3)) {
                Ok(Event::Reply(Reply::Error(error))) => Err(io::Error::other(error)),
                Ok(Event::Reply(reply)) => Ok(reply),
                Ok(Event::End(result)) => {
                    self.ended = Some(result.clone());
                    Err(io::Error::other(format!(
                        "Surface helper ended: {result:?}"
                    )))
                }
                Err(_) => Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Surface helper timed out",
                )),
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
    static SURFACE: Mutex<Option<Lease>> = Mutex::new(None);
    pub fn is_active() -> bool {
        SURFACE
            .try_lock()
            .map(|state| state.is_some())
            .unwrap_or(true)
    }
    pub fn release() -> io::Result<()> {
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        if let Some(lease) = state.as_mut() {
            lease.stop()?;
        }
        *state = None;
        Ok(())
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
        let executable = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target/debug/m-bedrock-virtual-clients.exe");
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
            let _ = tx.send(Event::End(result));
        });
        let mut lease = Lease {
            child,
            input: Some(input),
            events: rx,
            viewport,
            manager,
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
    pub fn present(manager: u64, viewport: Viewport, target: OwnedProcess) -> io::Result<()> {
        let viewport = viewport.validate()?;
        let rect = native::rectangle(manager, viewport)?;
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        if let Some(old) = state.as_mut() {
            if old.saved.as_ref().is_some_and(|s| {
                s.target.pid == target.pid
                    && s.target.created_file_time == target.created_file_time
                    && s.target.window_handle == target.window_handle
            }) {
                old.viewport = viewport;
                return match old.send(Command::Place {
                    rect,
                    focus: native::foreground(manager),
                    manager_pid: std::process::id(),
                    manager_window: manager,
                })? {
                    Reply::Placed => Ok(()),
                    _ => Err(io::Error::other("Unexpected surface acknowledgement")),
                };
            }
        }
        // Prepare and validate the replacement before touching the playable client.
        let mut replacement = prepare(manager, viewport, target)?;
        let previous = state.as_ref().and_then(|old| {
            old.saved
                .as_ref()
                .map(|saved| (old.manager, old.viewport, saved.target.clone()))
        });
        if let Some(old) = state.as_mut() {
            if let Err(error) = old.stop() {
                let _ = replacement.stop();
                return Err(error);
            }
        }
        let placement = replacement.send(Command::Place {
            rect,
            focus: native::foreground(manager),
            manager_pid: std::process::id(),
            manager_window: manager,
        });
        let placed = matches!(placement, Ok(Reply::Placed));
        let detail = match placement {
            Err(error) => error.to_string(),
            _ => "Unexpected surface acknowledgement".into(),
        };
        if placed {
            *state = Some(replacement);
            return Ok(());
        }
        if let Err(error) = replacement.stop() {
            *state = Some(replacement);
            return Err(error);
        }
        *state = None;
        if let Some((manager, viewport, target)) = previous {
            let mut old = prepare(manager, viewport, target)?;
            let restored = matches!(
                old.send(Command::Place {
                    rect: native::rectangle(manager, viewport)?,
                    focus: false,
                    manager_pid: std::process::id(),
                    manager_window: manager,
                }),
                Ok(Reply::Placed)
            );
            *state = Some(old);
            if !restored {
                return Err(io::Error::other(
                    "Previous surface could not resume. Restore windows before continuing.",
                ));
            }
        }
        Err(io::Error::other(format!(
            "Workspace placement failed: {detail}. Previous session preserved."
        )))
    }
    pub fn update(manager: u64, viewport: Option<Viewport>) -> io::Result<()> {
        let mut state = SURFACE
            .lock()
            .map_err(|_| io::Error::other("Workspace state unavailable"))?;
        let Some(lease) = state.as_mut() else {
            return Ok(());
        };
        if manager != lease.manager {
            return Err(io::Error::other("Wrong workspace window"));
        }
        if let Some(viewport) = viewport {
            lease.viewport = viewport.validate()?;
        }
        if native::minimized(manager)? {
            return match lease.send(Command::Hide)? {
                Reply::Hidden => Ok(()),
                _ => Err(io::Error::other("Surface hide failed")),
            };
        }
        let rect = native::rectangle(manager, lease.viewport)?;
        match lease.send(Command::Place {
            rect,
            focus: false,
            manager_pid: std::process::id(),
            manager_window: manager,
        })? {
            Reply::Placed => Ok(()),
            _ => Err(io::Error::other("Surface update failed")),
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
            let (_fixture, target) = fixture();
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
                    assert_ne!(composed.style, original.style);
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
                            let lease = state.as_mut().unwrap();
                            if mode == "pipe-loss" {
                                drop(lease.input.take());
                            } else {
                                lease.child.kill().unwrap();
                            }
                        }
                        let deadline = Instant::now() + Duration::from_secs(3);
                        loop {
                            let current = native::capture(target.clone()).unwrap();
                            if current.style == original.style && current.rect == original.rect {
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
                    assert_eq!(current.style, original.style, "{mode}");
                    assert_eq!(current.rect, original.rect, "{mode}");
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
            invalid.created_file_time =
                (invalid.created_file_time.parse::<u64>().unwrap() + 1).to_string();
            let before = native::capture(target.clone()).unwrap();
            assert!(present(manager as usize as u64, viewport, invalid).is_err());
            assert!(
                SURFACE.lock().unwrap().is_some(),
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
            release().unwrap();
            unsafe {
                DestroyWindow(manager);
            }
        }
    }
}
pub use controller::{is_active, present, release, update};

#[cfg(test)]
mod tests {
    use super::*;
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
