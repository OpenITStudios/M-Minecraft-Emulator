use serde::{Deserialize, Serialize};
use std::io;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverlayPosition {
    TopLeft,
    TopRight,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenOverlayItem {
    pub screen_number: usize,
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenOverlayRequest {
    pub enabled: bool,
    pub show_screen_number: bool,
    pub show_label: bool,
    pub position: OverlayPosition,
    pub opacity: f32,
    pub identify: bool,
    pub items: Vec<ScreenOverlayItem>,
}

#[cfg(target_os = "windows")]
mod windows {
    use super::*;
    use std::{
        ffi::c_void,
        ptr::{null, null_mut},
        sync::{
            mpsc::{self, Sender},
            OnceLock,
        },
        thread,
    };
    use windows_sys::Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect, GetStockObject,
            SelectObject, SetBkMode, SetTextColor, DEFAULT_GUI_FONT, DT_LEFT, DT_NOPREFIX,
            DT_SINGLELINE, DT_VCENTER, PAINTSTRUCT, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, LoadCursorW,
            PeekMessageW, RegisterClassW, SetLayeredWindowAttributes, ShowWindow, TranslateMessage,
            CS_HREDRAW, CS_VREDRAW, IDC_ARROW, LWA_ALPHA, MSG, PM_REMOVE, SW_SHOWNOACTIVATE,
            WM_PAINT, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
            WS_EX_TRANSPARENT, WS_POPUP,
        },
    };

    enum OverlayCommand {
        Apply(ScreenOverlayRequest),
        Clear,
        Shutdown,
    }

    struct OverlayWindow {
        hwnd: HWND,
        text: Vec<u16>,
    }

    static SENDER: OnceLock<Sender<OverlayCommand>> = OnceLock::new();
    static STARTUP: std::sync::Mutex<()> = std::sync::Mutex::new(());
    static TEXTS: OnceLock<std::sync::Mutex<Vec<(isize, Vec<u16>)>>> = OnceLock::new();

    fn texts() -> &'static std::sync::Mutex<Vec<(isize, Vec<u16>)>> {
        TEXTS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
    }

    fn wide(value: &str) -> Vec<u16> {
        value.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe extern "system" fn window_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        if message == WM_PAINT {
            let mut paint = std::mem::zeroed::<PAINTSTRUCT>();
            let dc = BeginPaint(hwnd, &mut paint);
            let rect = paint.rcPaint;
            let brush = CreateSolidBrush(0x00201B18 as COLORREF);
            FillRect(dc, &rect, brush);
            DeleteObject(brush);
            SetBkMode(dc, TRANSPARENT as i32);
            SetTextColor(dc, 0x00F5F5F5 as COLORREF);
            SelectObject(dc, GetStockObject(DEFAULT_GUI_FONT));
            if let Ok(items) = texts().lock() {
                if let Some((_, text)) = items.iter().find(|(window, _)| *window == hwnd as isize) {
                    let mut text_rect = rect;
                    text_rect.left += 12;
                    DrawTextW(dc, text.as_ptr(), -1, &mut text_rect, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
                }
            }
            EndPaint(hwnd, &paint);
            return 0;
        }
        DefWindowProcW(hwnd, message, wparam, lparam)
    }

    fn sender() -> io::Result<&'static Sender<OverlayCommand>> {
        if let Some(sender) = SENDER.get() { return Ok(sender); }
        let _startup = STARTUP.lock()
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "Screen Overlay startup lock failed"))?;
        if let Some(sender) = SENDER.get() { return Ok(sender); }
        let (tx, rx) = mpsc::channel::<OverlayCommand>();
        let (ready_tx, ready_rx) = mpsc::sync_channel::<io::Result<()>>(1);
        thread::Builder::new().name("screen-overlay".into()).spawn(move || unsafe {
            let class_name = wide("MIVUBI_VIRTUAL_CLIENTS_SCREEN_OVERLAY");
            let instance = GetModuleHandleW(null());
            let class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                hCursor: LoadCursorW(null_mut(), IDC_ARROW),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            };
            if RegisterClassW(&class) == 0 {
                let _ = ready_tx.send(Err(io::Error::last_os_error()));
                return;
            }
            let _ = ready_tx.send(Ok(()));
            let mut windows: Vec<OverlayWindow> = Vec::new();
            let mut running = true;
            while running {
                // When no overlay exists, block on the command channel rather
                // than polling Windows messages ~60 times each second.
                let mut pending = if windows.is_empty() {
                    match rx.recv() {
                        Ok(command) => Some(command),
                        Err(_) => break,
                    }
                } else {
                    None
                };
                while let Some(command) = pending.take().or_else(|| rx.try_recv().ok()) {
                    match command {
                        OverlayCommand::Clear => clear_windows(&mut windows),
                        OverlayCommand::Shutdown => { clear_windows(&mut windows); running = false; }
                        OverlayCommand::Apply(request) => {
                            clear_windows(&mut windows);
                            if request.enabled {
                                let alpha = (request.opacity.clamp(0.35, 1.0) * 255.0).round() as u8;
                                for item in request.items {
                                    let mut parts = Vec::new();
                                    if request.show_screen_number { parts.push(format!("SCREEN {}", item.screen_number)); }
                                    if request.show_label && !item.label.trim().is_empty() { parts.push(item.label.trim().to_string()); }
                                    if parts.is_empty() { continue; }
                                    let text = wide(&parts.join("  ·  "));
                                    let overlay_width = if request.identify { item.width.min(360) } else { item.width.min(260) };
                                    let overlay_height = if request.identify { 64 } else { 38 };
                                    let x = match request.position {
                                        OverlayPosition::TopLeft => item.x + 12,
                                        OverlayPosition::TopRight => item.x + item.width - overlay_width - 12,
                                    };
                                    let hwnd = CreateWindowExW(
                                        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
                                        class_name.as_ptr(), wide("").as_ptr(), WS_POPUP,
                                        x, item.y + 12, overlay_width, overlay_height,
                                        null_mut(), null_mut(), instance, null_mut::<c_void>(),
                                    );
                                    if !hwnd.is_null() {
                                        SetLayeredWindowAttributes(hwnd, 0, alpha, LWA_ALPHA);
                                        windows.push(OverlayWindow { hwnd, text });
                                    }
                                }
                                if let Ok(mut items) = texts().lock() {
                                    *items = windows.iter().map(|window| (window.hwnd as isize, window.text.clone())).collect();
                                }
                                for window in &windows { ShowWindow(window.hwnd, SW_SHOWNOACTIVATE); }
                            }
                        }
                    }
                }
                if !windows.is_empty() {
                    // Active overlays still pump paint messages at the original cadence.
                    let mut message = std::mem::zeroed::<MSG>();
                    while PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) != 0 {
                        TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                    thread::sleep(std::time::Duration::from_millis(16));
                }
            }
        }).map_err(|error| io::Error::new(io::ErrorKind::Other, format!("failed to start Screen Overlay thread: {error}")))?;
        ready_rx.recv().map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Screen Overlay thread stopped during startup"))??;
        let _ = SENDER.set(tx);
        SENDER.get().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Screen Overlay channel unavailable"))
    }

    unsafe fn clear_windows(windows: &mut Vec<OverlayWindow>) {
        for window in windows.drain(..) { DestroyWindow(window.hwnd); }
        if let Ok(mut items) = texts().lock() { items.clear(); }
    }

    pub fn apply(request: ScreenOverlayRequest) -> io::Result<()> {
        sender()?.send(OverlayCommand::Apply(request))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "Screen Overlay thread stopped"))
    }

    pub fn clear() {
        if let Ok(sender) = sender() { let _ = sender.send(OverlayCommand::Clear); }
    }

    pub fn shutdown() {
        if let Some(sender) = SENDER.get() { let _ = sender.send(OverlayCommand::Shutdown); }
    }
}

#[cfg(target_os = "windows")]
pub use windows::{apply, clear, shutdown};

#[cfg(not(target_os = "windows"))]
pub fn apply(_request: ScreenOverlayRequest) -> io::Result<()> {
    Err(io::Error::new(io::ErrorKind::Unsupported, "Screen Overlay is currently available on Windows only"))
}
#[cfg(not(target_os = "windows"))]
pub fn clear() {}
#[cfg(not(target_os = "windows"))]
pub fn shutdown() {}
