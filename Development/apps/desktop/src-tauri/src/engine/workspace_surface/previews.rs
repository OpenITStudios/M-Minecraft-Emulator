//! DWM renders monitoring tiles and the unfocused main game into owned windows.
//! No screenshot loop, encoder, game input forwarding, or new lifecycle authority.
use super::{controller, native, OwnedProcess, Rectangle, Viewport};
use serde::{Deserialize, Serialize};
use std::io;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreviewTile {
    pub slot: u32,
    pub viewport: Viewport,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TileStatus {
    pub slot: u32,
    pub registered: bool,
    pub reason: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewReport {
    pub schema: u32,
    pub tiles: Vec<TileStatus>,
}

// A per-launcher close must never discard the remaining Minecraft tiles.
fn retain_other_preview_slot(observed:u32,closing:u32)->bool {
    observed!=closing
}

fn overlaps(a: Rectangle, b: Rectangle) -> bool {
    a.x < b.x + b.width && b.x < a.x + a.width && a.y < b.y + b.height && b.y < a.y + a.height
}
fn validate_tiles(tiles: &[PreviewTile], active: Option<u32>) -> io::Result<()> {
    let mut seen = std::collections::HashSet::new();
    if tiles.len() > 4 {
        return Err(io::Error::other("At most four preview tiles are supported"));
    }
    for tile in tiles {
        tile.viewport.validate_preview()?;
        if !(1..=5).contains(&tile.slot) || active == Some(tile.slot) || !seen.insert(tile.slot) {
            return Err(io::Error::other("Invalid or duplicate preview profile"));
        }
    }
    Ok(())
}

pub(super) fn configure(
    window: tauri::Window,
    viewport: Viewport,
    tiles: Vec<PreviewTile>,
    profiles: Vec<(u32, OwnedProcess)>,
) -> io::Result<PreviewReport> {
    #[cfg(windows)]
    {
        let manager = window.hwnd().map_err(io::Error::other)?.0 as usize as u64;
        let active = controller::active_preview_slot(manager)?;
        validate_tiles(&tiles, active)?;
        let main = native::rectangle(manager, viewport.validate()?)?;
        let rectangles = tiles
            .iter()
            .map(|tile| native::rectangle(manager, tile.viewport))
            .collect::<io::Result<Vec<_>>>()?;
        for (index, rect) in rectangles.iter().enumerate() {
            if overlaps(main, *rect)
                || rectangles[..index]
                    .iter()
                    .any(|other| overlaps(*other, *rect))
            {
                return Err(io::Error::other(
                    "Preview images must not overlap the game area or each other",
                ));
            }
        }
        let monitored=profiles.iter().filter(|(slot,_)|tiles.iter().any(|tile|tile.slot==*slot)).cloned().collect::<Vec<_>>();
        let unavailable = controller::adopt_preview_sources(manager, viewport, &monitored)?;
        let primary = active.map(|slot| PreviewTile { slot, viewport });
        windows::apply(window, tiles, primary, profiles, unavailable)
    }
    #[cfg(not(windows))]
    {
        let _ = (window, viewport, tiles, profiles);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Windows previews unavailable",
        ))
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use std::{
        mem, ptr,
        sync::{
            atomic::{AtomicBool, AtomicU32, Ordering},
            mpsc, Arc, Mutex,
        },
        thread::{self, JoinHandle},
        time::Duration,
    };
    use tauri::Emitter;
    use windows_sys::Win32::{
        Foundation::*,
        Graphics::{Dwm::*, Gdi::*},
        System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
        UI::WindowsAndMessaging::*,
    };
    const WAKE: u32 = WM_APP + 0x437;
    const SELECT: u32 = WM_APP + 0x438;
    const CLASS: &[u16] = &[
        76, 97, 122, 121, 84, 101, 115, 116, 101, 114, 80, 114, 101, 118, 105, 101, 119, 0,
    ];
    enum Job {
        Layout {
            window: tauri::Window,
            tiles: Vec<PreviewTile>,
            primary: Option<PreviewTile>,
            profiles: Vec<(u32, OwnedProcess)>,
            unavailable: Vec<(u32, String)>,
            reply: mpsc::SyncSender<io::Result<PreviewReport>>,
        },
        Clear(mpsc::SyncSender<io::Result<()>>),
        ClearSlot {slot:u32,reply:mpsc::SyncSender<io::Result<()>>},
        ClearPrimary(mpsc::SyncSender<io::Result<()>>),
    }
    struct Worker {
        sender: mpsc::SyncSender<Job>,
        id: Arc<AtomicU32>,
        join: JoinHandle<()>,
    }
    static WORKER: Mutex<Option<Worker>> = Mutex::new(None);
    static THREAD: AtomicU32 = AtomicU32::new(0);
    static REFRESH_PENDING: AtomicBool = AtomicBool::new(false);
    fn notify(id: u32) -> io::Result<()> {
        if !REFRESH_PENDING.swap(true, Ordering::AcqRel)
            && unsafe { PostThreadMessageW(id, WAKE, 0, 0) } == 0
        {
            REFRESH_PENDING.store(false, Ordering::Release);
            return Err(io::Error::last_os_error());
        }
        Ok(())
    }

    unsafe extern "system" fn procedure(hwnd: HWND, message: u32, w: WPARAM, l: LPARAM) -> LRESULT {
        match message {
            WM_MOUSEACTIVATE => MA_NOACTIVATE as isize,
            WM_LBUTTONUP => {
                let slot = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
                if (1..=5).contains(&slot) {
                    // An actual user click focuses the manager before its normal selection path.
                    let owner = GetWindow(hwnd, GW_OWNER);
                    if !owner.is_null() {
                        SetForegroundWindow(owner);
                    }
                    PostThreadMessageW(GetCurrentThreadId(), SELECT, slot as usize, 0);
                }
                0
            }
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                let mut paint: PAINTSTRUCT = mem::zeroed();
                let dc = BeginPaint(hwnd, &mut paint);
                let mut rect: RECT = mem::zeroed();
                GetClientRect(hwnd, &mut rect);
                let brush = CreateSolidBrush(0x00140f0d);
                FillRect(dc, &rect, brush);
                DeleteObject(brush);
                EndPaint(hwnd, &paint);
                0
            }
            _ => DefWindowProcW(hwnd, message, w, l),
        }
    }
    struct Thumbnail {
        slot: u32,
        target: OwnedProcess,
        viewport: Viewport,
        manager: u64,
        window: HWND,
        thumbnail: isize,
        _process: native::ProcessHandle,
    }
    impl Thumbnail {
        fn create(manager: u64, tile: &PreviewTile, target: OwnedProcess) -> io::Result<Self> {
            let (process, source) = native::hwnd(&target)?;
            let rect = native::rectangle(manager, tile.viewport)?;
            let title: Vec<u16> = format!("Minecraft - {} preview", tile.slot)
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let window = unsafe {
                CreateWindowExW(
                    WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                    CLASS.as_ptr(),
                    title.as_ptr(),
                    WS_POPUP,
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    manager as usize as HWND,
                    ptr::null_mut(),
                    GetModuleHandleW(ptr::null()),
                    ptr::null(),
                )
            };
            if window.is_null() {
                return Err(io::Error::last_os_error());
            }
            unsafe {
                SetWindowLongPtrW(window, GWLP_USERDATA, tile.slot as isize);
            }
            let mut thumbnail = 0;
            let result = unsafe { DwmRegisterThumbnail(window, source, &mut thumbnail) };
            if result < 0 {
                unsafe {
                    DestroyWindow(window);
                }
                return Err(io::Error::other(format!(
                    "Native preview registration failed: {result:#x}"
                )));
            }
            Ok(Self {
                slot: tile.slot,
                target,
                viewport: tile.viewport,
                manager,
                window,
                thumbnail,
                _process: process,
            })
        }
        fn matches(&self, slot: u32, target: &OwnedProcess) -> bool {
            self.slot == slot
                && self.target.pid == target.pid
                && self.target.created_file_time == target.created_file_time
                && self.target.session_id == target.session_id
                && self.target.window_handle == target.window_handle
                && unsafe { IsWindow(self.window) != 0 }
        }
        fn update(&mut self, visible: bool) -> io::Result<()> {
            if !visible {
                self.hide();
                let props=DWM_THUMBNAIL_PROPERTIES{dwFlags:DWM_TNP_VISIBLE,fVisible:0,..unsafe{mem::zeroed()}};
                let result=unsafe{DwmUpdateThumbnailProperties(self.thumbnail,&props)};
                return if result<0 {Err(io::Error::other("Native preview hide failed"))}else{Ok(())};
            }
            let (_pin, source) = native::hwnd(&self.target)?;
            let rect = native::rectangle(self.manager, self.viewport)?;
            let mut client: RECT = unsafe { mem::zeroed() };
            if unsafe { GetClientRect(source, &mut client) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let fit = super::fit_rectangle(
                rect.width,
                rect.height,
                client.right - client.left,
                client.bottom - client.top,
            )?;
            let props = DWM_THUMBNAIL_PROPERTIES {
                dwFlags: DWM_TNP_RECTDESTINATION
                    | DWM_TNP_VISIBLE
                    | DWM_TNP_OPACITY
                    | DWM_TNP_SOURCECLIENTAREAONLY,
                rcDestination: RECT {
                    left: fit.x,
                    top: fit.y,
                    right: fit.x + fit.width,
                    bottom: fit.y + fit.height,
                },
                opacity: 255,
                fVisible: visible as i32,
                fSourceClientAreaOnly: 1,
                ..unsafe { mem::zeroed() }
            };
            let result = unsafe { DwmUpdateThumbnailProperties(self.thumbnail, &props) };
            if result < 0 {
                return Err(io::Error::other(format!(
                    "Native preview update failed: {result:#x}"
                )));
            }
            if unsafe {
                SetWindowPos(
                    self.window,
                    ptr::null_mut(),
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    SWP_NOACTIVATE
                        | SWP_NOZORDER
                        | if visible {
                            SWP_SHOWWINDOW
                        } else {
                            SWP_HIDEWINDOW
                        },
                )
            } == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        }
        fn hide(&self) {
            unsafe {
                ShowWindow(self.window, SW_HIDE);
            }
        }
    }
    impl Drop for Thumbnail {
        fn drop(&mut self) {
            unsafe {
                DwmUnregisterThumbnail(self.thumbnail);
                DestroyWindow(self.window);
            }
        }
    }

    struct Scene {
        window: Option<tauri::Window>,
        manager: u64,
        profiles: Vec<(u32, OwnedProcess)>,
        thumbnails: Vec<Thumbnail>,
        primary: Option<Thumbnail>,
    }
    impl Scene {
        fn visibility(&self) -> bool {
            unsafe {
                let manager = self.manager as usize as HWND;
                if manager.is_null()
                    || IsWindow(manager) == 0
                    || IsWindowVisible(manager) == 0
                    || IsIconic(manager) != 0
                {
                    return false;
                }
                // Owned, non-topmost destinations follow the manager's stacking.
                // Losing input focus must not blank a still-visible monitoring panel.
                true
            }
        }
        fn refresh(&mut self) {
            let visible = self.visibility();
            for thumbnail in &mut self.thumbnails {
                if thumbnail.update(visible).is_err_and(|error|error.kind()!=io::ErrorKind::WouldBlock) {
                    thumbnail.hide();
                }
            }
            if let Some(primary) = &mut self.primary {
                // Never overlay the interactive game. Its native HWND receives all input.
                let playing = native::interactive_at(&primary.target, primary.manager, primary.viewport);
                if let Err(error) = primary.update(visible && !playing) {
                    if error.kind() != io::ErrorKind::WouldBlock {
                        primary.hide();
                    }
                }
            }
        }
        fn layout(
            &mut self,
            window: tauri::Window,
            tiles: Vec<PreviewTile>,
            primary: Option<PreviewTile>,
            profiles: Vec<(u32, OwnedProcess)>,
            unavailable: Vec<(u32, String)>,
        ) -> io::Result<PreviewReport> {
            let manager = window.hwnd().map_err(io::Error::other)?.0 as usize as u64;
            if self.manager != manager {
                self.thumbnails.clear();
                self.primary = None;
            }
            self.window = Some(window);
            self.manager = manager;
            self.profiles = profiles;
            self.thumbnails.retain(|thumbnail| {
                tiles.iter().any(|tile| tile.slot == thumbnail.slot)
                    && self
                        .profiles
                        .iter()
                        .any(|(slot, target)| thumbnail.matches(*slot, target))
                    && !unavailable.iter().any(|(slot, _)| *slot == thumbnail.slot)
            });
            let visible = self.visibility();
            match primary {
                Some(tile) => {
                    let target = self.profiles.iter().find(|(slot,_)| *slot == tile.slot)
                        .map(|(_,target)|target).ok_or_else(||io::Error::other("Active game preview source is unavailable"))?;
                    if self.primary.as_ref().is_none_or(|image| !image.matches(tile.slot,target)) {
                        self.primary = None;
                        self.primary = Some(Thumbnail::create(manager, &tile, target.clone())?);
                    }
                    let image = self.primary.as_mut().unwrap();
                    image.viewport = tile.viewport;
                    image.update(visible && !native::interactive_at(target, manager, tile.viewport))?;
                }
                None => self.primary = None,
            }
            let mut statuses = Vec::with_capacity(tiles.len());
            for tile in tiles {
                let mut status = TileStatus {
                    slot: tile.slot,
                    registered: false,
                    reason: None,
                };
                if let Some((_, reason)) = unavailable.iter().find(|(slot, _)| *slot == tile.slot) {
                    status.reason = Some(reason.clone());
                    statuses.push(status);
                    continue;
                }
                let Some((_, target)) = self.profiles.iter().find(|(slot, _)| *slot == tile.slot)
                else {
                    statuses.push(status);
                    continue;
                };
                let index = match self
                    .thumbnails
                    .iter()
                    .position(|thumbnail| thumbnail.matches(tile.slot, target))
                {
                    Some(index) => index,
                    None => match Thumbnail::create(manager, &tile, target.clone()) {
                        Ok(thumbnail) => {
                            self.thumbnails.push(thumbnail);
                            self.thumbnails.len() - 1
                        }
                        Err(error) => {
                            status.reason = Some(error.to_string());
                            statuses.push(status);
                            continue;
                        }
                    },
                };
                self.thumbnails[index].viewport = tile.viewport;
                match self.thumbnails[index].update(visible) {
                    Ok(()) => status.registered = true,
                    Err(error) => {
                        self.thumbnails[index].hide();
                        status.reason = Some(error.to_string());
                    }
                }
                statuses.push(status);
            }
            Ok(PreviewReport {
                schema: 1,
                tiles: statuses,
            })
        }
    }
    fn start() -> Worker {
        let (sender, receiver) = mpsc::sync_channel(1);
        let id = Arc::new(AtomicU32::new(0));
        let thread_id = Arc::clone(&id);
        let join = thread::spawn(move || unsafe {
            let mut message: MSG = mem::zeroed();
            PeekMessageW(&mut message, ptr::null_mut(), 0, 0, PM_NOREMOVE);
            let class = WNDCLASSW {
                lpfnWndProc: Some(procedure),
                hInstance: GetModuleHandleW(ptr::null()),
                lpszClassName: CLASS.as_ptr(),
                hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
                ..mem::zeroed()
            };
            if RegisterClassW(&class) == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
                return;
            }
            REFRESH_PENDING.store(false, Ordering::Release);
            thread_id.store(GetCurrentThreadId(), Ordering::Release);
            THREAD.store(GetCurrentThreadId(), Ordering::Release);
            let mut scene = Scene {
                window: None,
                manager: 0,
                profiles: Vec::new(),
                thumbnails: Vec::new(),
                primary: None,
            };
            while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
                if message.message == SELECT {
                    if let Some(window) = &scene.window {
                        let _ = window.emit("lazy-tester-preview-select", message.wParam as u32);
                    }
                } else if message.message == WAKE {
                    REFRESH_PENDING.store(false, Ordering::Release);
                    let mut laid_out = false;
                    while let Ok(job) = receiver.try_recv() {
                        match job {
                            Job::Layout {
                                window,
                                tiles,
                                primary,
                                profiles,
                                unavailable,
                                reply,
                            } => {
                                let _ =
                                    reply.send(scene.layout(window, tiles, primary, profiles, unavailable));
                                laid_out = true;
                            }
                            Job::Clear(reply) => {
                                scene.thumbnails.clear();
                                scene.primary = None;
                                scene.profiles.clear();
                                let _ = reply.send(Ok(()));
                            }
                            Job::ClearSlot {slot,reply} => {
                                // This executes on the existing sole DWM owner
                                // thread, never on a game or launcher thread.
                                scene.thumbnails.retain(|thumbnail|
                                    retain_other_preview_slot(thumbnail.slot,slot));
                                if scene.primary.as_ref().is_some_and(|image|
                                    !retain_other_preview_slot(image.slot,slot)) {
                                    scene.primary=None;
                                }
                                scene.profiles.retain(|(profile,_)|
                                    retain_other_preview_slot(*profile,slot));
                                let _ = reply.send(Ok(()));
                            }
                            Job::ClearPrimary(reply) => {
                                scene.primary = None;
                                let _ = reply.send(Ok(()));
                            }
                        }
                    }
                    if !laid_out {
                        scene.refresh();
                    }
                } else {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            scene.thumbnails.clear();
            THREAD.store(0, Ordering::Release);
        });
        Worker { sender, id, join }
    }
    fn send<T>(
        make: impl FnOnce(mpsc::SyncSender<io::Result<T>>) -> Job,
        start_if_missing: bool,
    ) -> io::Result<Option<T>> {
        let mut owner = WORKER
            .lock()
            .map_err(|_| io::Error::other("Preview worker unavailable"))?;
        if owner
            .as_ref()
            .is_some_and(|worker| worker.join.is_finished())
        {
            let worker = owner.take().unwrap();
            let _ = worker.join.join();
        }
        if owner.is_none() {
            if !start_if_missing {
                return Ok(None);
            }
            *owner = Some(start());
        }
        let worker = owner.as_ref().unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        while worker.id.load(Ordering::Acquire) == 0
            && !worker.join.is_finished()
            && std::time::Instant::now() < deadline
        {
            thread::sleep(Duration::from_millis(1));
        }
        let id = worker.id.load(Ordering::Acquire);
        if id == 0 || worker.join.is_finished() {
            return Err(io::Error::other("Preview worker did not start"));
        }
        let (reply, receive) = mpsc::sync_channel(1);
        worker.sender.try_send(make(reply)).map_err(|_| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "Previous preview update is still pending",
            )
        })?;
        notify(id)?;
        receive
            .recv_timeout(Duration::from_secs(3))
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Preview update is pending; worker retained",
                )
            })?
            .map(Some)
    }
    pub(super) fn apply(
        window: tauri::Window,
        tiles: Vec<PreviewTile>,
        primary: Option<PreviewTile>,
        profiles: Vec<(u32, OwnedProcess)>,
        unavailable: Vec<(u32, String)>,
    ) -> io::Result<PreviewReport> {
        let base = PreviewReport {
            schema: 1,
            tiles: tiles
                .iter()
                .map(|tile| TileStatus {
                    slot: tile.slot,
                    registered: false,
                    reason: unavailable
                        .iter()
                        .find(|(slot, _)| *slot == tile.slot)
                        .map(|(_, reason)| reason.clone()),
                })
                .collect(),
        };
        if primary.is_none() && !tiles.iter().any(|tile| {
            profiles.iter().any(|(slot, _)| *slot == tile.slot)
                && !unavailable.iter().any(|(slot, _)| *slot == tile.slot)
        }) {
            clear()?;
            return Ok(base);
        }
        send(
            |reply| Job::Layout {
                window,
                tiles,
                primary,
                profiles,
                unavailable,
                reply,
            },
            true,
        )?
        .ok_or_else(|| io::Error::other("Preview worker missing"))
    }
    pub(super) fn clear() -> io::Result<()> {
        send(Job::Clear, false).map(|_| ())
    }
    pub(super) fn clear_slot(slot:u32)->io::Result<()> {
        send(|reply|Job::ClearSlot {slot,reply},false).map(|_| ())
    }
    pub(super) fn clear_primary() -> io::Result<()> {
        send(Job::ClearPrimary, false).map(|_| ())
    }
    pub(super) fn refresh() {
        let id = THREAD.load(Ordering::Acquire);
        if id != 0 {
            let _ = notify(id);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
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
                thumbnail.update(false).unwrap();
                let mut size: SIZE = mem::zeroed();
                assert!(DwmQueryThumbnailSourceSize(thumbnail.thumbnail, &mut size) >= 0);
                assert!(size.cx > 0 && size.cy > 0);
                assert_eq!(IsWindowVisible(thumbnail.window), 0);
                ShowWindow(manager, SW_SHOWNOACTIVATE);
                let primary_tile = PreviewTile { slot: 5, viewport: Viewport {
                    x: 10.0, y: 100.0, width: 640.0, height: 360.0, ..tile.viewport } };
                let primary = Thumbnail::create(manager as usize as u64, &primary_tile, target).unwrap();
                let mut scene=Scene{window:None,manager:manager as usize as u64,profiles:Vec::new(),thumbnails:Vec::new(),primary:Some(primary)};
                assert_ne!(GetForegroundWindow(),manager);
                assert!(scene.visibility(),"Visible monitoring must survive focus moving elsewhere");
                thumbnail.update(scene.visibility()).unwrap();
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
                drop(thumbnail);
                drop(scene);
                DestroyWindow(source);
                DestroyWindow(manager);
            }
        }
    }
}

fn fit_rectangle(
    width: i32,
    height: i32,
    source_width: i32,
    source_height: i32,
) -> io::Result<Rectangle> {
    if width <= 0 || height <= 0 || source_width <= 0 || source_height <= 0 {
        return Err(io::Error::other("Preview source size unavailable"));
    }
    let scale = (width as f64 / source_width as f64).min(height as f64 / source_height as f64);
    let fitted_width = (source_width as f64 * scale).round().max(1.0) as i32;
    let fitted_height = (source_height as f64 * scale).round().max(1.0) as i32;
    Ok(Rectangle {
        x: (width - fitted_width) / 2,
        y: (height - fitted_height) / 2,
        width: fitted_width,
        height: fitted_height,
    })
}
pub(super) fn clear() -> io::Result<()> {
    #[cfg(windows)]
    {
        windows::clear()
    }
    #[cfg(not(windows))]
    {
        Ok(())
    }
}
pub(super) fn clear_slot(slot:u32)->io::Result<()> {
    if !(1..=5).contains(&slot) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft preview slot"));
    }
    #[cfg(windows)]
    { windows::clear_slot(slot) }
    #[cfg(not(windows))]
    { Ok(()) }
}
pub(super) fn clear_primary() -> io::Result<()> {
    #[cfg(windows)]
    { windows::clear_primary() }
    #[cfg(not(windows))]
    { Ok(()) }
}
pub(super) fn refresh() {
    #[cfg(windows)]
    windows::refresh();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closing_one_of_five_minecraft_slots_preserves_four_peer_previews() {
        for closing in 1..=5 {
            let visible=(1..=5)
                .filter(|observed|retain_other_preview_slot(*observed,closing))
                .collect::<Vec<_>>();
            assert_eq!(visible.len(),4);
            assert!(!visible.contains(&closing));
            for other in 1..=5 {
                assert_eq!(retain_other_preview_slot(other,closing),other!=closing);
            }
        }
    }
    fn tile(slot: u32, x: f64) -> PreviewTile {
        PreviewTile {
            slot,
            viewport: Viewport {
                x,
                y: 450.0,
                width: 150.0,
                height: 85.0,
                pixel_ratio: 1.0,
            },
        }
    }
    #[test]
    fn preview_layout_rejects_duplicates_active_slot_and_invalid_geometry() {
        assert!(validate_tiles(
            &[
                tile(2, 10.0),
                tile(3, 170.0),
                tile(4, 330.0),
                tile(5, 490.0)
            ],
            Some(1)
        )
        .is_ok());
        assert!(validate_tiles(&[tile(2, 10.0), tile(2, 170.0)], Some(1)).is_err());
        assert!(validate_tiles(&[tile(1, 10.0)], Some(1)).is_err());
        assert!(validate_tiles(&[tile(0, 10.0)], None).is_err());
        assert!(validate_tiles(&[tile(2, f64::NAN)], None).is_err());
        assert!(validate_tiles(&[tile(2, -1.0)], None).is_err());
    }
    #[test]
    fn letterboxing_preserves_aspect_and_overlap_excludes_game_input_area() {
        assert_eq!(
            fit_rectangle(200, 150, 1920, 1080).unwrap(),
            Rectangle {
                x: 0,
                y: 18,
                width: 200,
                height: 113
            }
        );
        assert!(fit_rectangle(200, 150, 0, 1080).is_err());
        let game = Rectangle {
            x: 10,
            y: 10,
            width: 800,
            height: 400,
        };
        assert!(!overlaps(
            game,
            Rectangle {
                x: 10,
                y: 420,
                width: 200,
                height: 100
            }
        ));
        assert!(overlaps(
            game,
            Rectangle {
                x: 10,
                y: 400,
                width: 200,
                height: 100
            }
        ));
    }
}
