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
#[path = "workspace_surface/native.rs"]
mod native;
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

// The experimental MC02 diagnostic reuses this identity owner. Retain the
// verified process HANDLE during its one-shot window-scoped operation; this
// accessor does not grant game/session lifecycle or public input authority.
#[cfg(windows)]
pub(crate) fn with_verified_native_window<R>(
    target: &OwnedProcess,
    use_window: impl FnOnce(usize) -> io::Result<R>,
) -> io::Result<R> {
    let (_pin, window) = native::hwnd(target)?;
    use_window(window as usize)
}

#[cfg(windows)]
pub(crate) fn matches_verified_native_window(
    target: &OwnedProcess, expected: usize,
) -> io::Result<bool> {
    with_verified_native_window(target, |window| Ok(window == expected))
}

// Only Windows compiles the existing HWND/OGLES recovery controller. Its
// process roles and exact native checkpoint ownership have no Mac analogue.
#[cfg(windows)]
#[path = "workspace_surface/controller.rs"]
mod controller;
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
