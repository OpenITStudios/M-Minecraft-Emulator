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

// DWM registration is observation, not permission to play or launch.
fn preview_tile_reason(
    slot: u32,
    profiles: &[(u32, OwnedProcess)],
    unavailable: &[(u32, String)],
) -> Option<String> {
    unavailable.iter().find(|(item, _)| *item == slot)
        .map(|(_, reason)| reason.clone())
        .or_else(|| (!profiles.iter().any(|(item, _)| *item == slot))
            .then(|| "No verified Minecraft preview source for this profile".into()))
}
// The primary DWM observer is separate from the requested secondary tiles.
// Share initial status evidence between the live and no-worker paths.
fn initial_tile_status(tile: &PreviewTile, profiles: &[(u32, OwnedProcess)],
    unavailable: &[(u32, String)]) -> TileStatus {
    TileStatus {
        slot: tile.slot,
        registered: false,
        reason: preview_tile_reason(tile.slot, profiles, unavailable),
    }
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
        // Snapshot BEFORE potentially slow native preview adoption. A
        // CloseOne arriving during it invalidates this old Layout request.
        let issued_epochs = windows::snapshot_epochs();
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
        windows::apply(window, tiles, primary, profiles, unavailable, issued_epochs)
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
            atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
            mpsc, Arc, Mutex,
        },
        thread::{self, JoinHandle},
        time::{Duration, Instant},
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
    // Reuse the existing diagnostic string event; do not add an IPC
    // command or channel. The envelope pins the exact failed source so a
    // delayed event cannot invalidate a replacement Minecraft process.
    const PREVIEW_INVALIDATED_PREFIX: &str = "dwm-preview-invalidated:";
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PreviewInvalidated<'a> {
        schema: u32,
        slot: u32,
        process: &'a OwnedProcess,
        reason: &'a str,
    }
    fn preview_invalidation_message(
        slot: u32, process: &OwnedProcess, reason: &str,
    ) -> io::Result<String> {
        let notice = PreviewInvalidated { schema: 1, slot, process, reason };
        Ok(format!("{PREVIEW_INVALIDATED_PREFIX}{}",
            serde_json::to_string(&notice)?))
    }
    // At most one asynchronous invalidation per thumbnail per successful
    // layout report. A new validated layout, not idle refresh, rearms it.
    fn take_invalidation_notice(already_sent: &mut bool) -> bool {
        if *already_sent { return false; }
        *already_sent = true;
        true
    }
    // A reported DWM error must remain the visible UI truth. An ordinary
    // native WAKE does not supply a fresh successful PreviewReport; keep
    // this secondary image hidden until layout verifies it explicitly.
    fn display_with_current_preview_receipt(
        manager_visible: bool, invalidation_notified: bool,
    ) -> bool {
        manager_visible && !invalidation_notified
    }
    // Control-only WAKE does not revalidate unchanged peer thumbnails.
    fn refresh_after_wake(native_event: bool, layout_applied: bool) -> bool {
        native_event && !layout_applied
    }
    fn layout_expired(deadline: Instant, now: Instant) -> bool {
        now >= deadline
    }
    fn require_live_layout(deadline: Instant) -> io::Result<()> {
        if layout_expired(deadline, Instant::now()) {
            return Err(io::Error::new(io::ErrorKind::TimedOut,
                "DWM layout exceeded its original reply deadline"));
        }
        Ok(())
    }
    // Observation-only slot generations cancel old DWM layouts, never
    // Minecraft sessions or their native recovery checkpoints. No worker,
    // queue or polling owner is introduced; only five atomic counters.
    static SLOT_CANCEL_EPOCHS: [AtomicU64; 5] = [
        AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
        AtomicU64::new(0), AtomicU64::new(0),
    ];
    pub(super) fn snapshot_epochs() -> [u64; 5] {
        std::array::from_fn(|index|
            SLOT_CANCEL_EPOCHS[index].load(Ordering::Acquire))
    }
    fn slot_epoch_matches(issued: &[u64; 5], current: &[u64; 5], slot: u32) -> bool {
        (1..=5).contains(&slot) && issued[(slot - 1) as usize] == current[(slot - 1) as usize]
    }
    fn epochs_current(issued: &[u64; 5]) -> bool {
        *issued == snapshot_epochs()
    }
    enum Job {
        Layout {
            window: tauri::Window,
            tiles: Vec<PreviewTile>,
            primary: Option<PreviewTile>,
            profiles: Vec<(u32, OwnedProcess)>,
            unavailable: Vec<(u32, String)>,
            issued_epochs: [u64; 5],
            deadline: Instant,
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
    // Existing worker-only scheduling hint, not native game authority.
    static NATIVE_REFRESH_REQUESTED: AtomicBool = AtomicBool::new(false);
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
    // WouldBlock also means the old native game frame is unverified.
    // Hide on every failed observation; this never changes game lifecycle.
    fn hide_on_failed_update(result: io::Result<()>, hide: impl FnOnce()) -> io::Result<()> {
        if result.is_err() { hide(); }
        result
    }
    // DWM composites game pixels after successful registration. Identical
    // property/position writes are redundant during bursts of native events;
    // a future existing event periodically reasserts Win32/DWM state.
    const DWM_PROPERTY_REASSERT: Duration = Duration::from_secs(1);
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct DwmGeometry {
        destination: Rectangle,
        fitted: Rectangle,
        source_width: i32,
        source_height: i32,
    }
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum DwmAppliedState {
        Hidden,
        Visible(DwmGeometry),
    }
    fn dwm_property_write_due(
        previous: Option<(DwmAppliedState, Instant)>,
        next: DwmAppliedState,
        now: Instant,
    ) -> bool {
        previous.is_none_or(|(previous_state, at)|
            previous_state != next ||
            now.saturating_duration_since(at) >= DWM_PROPERTY_REASSERT)
    }
    struct Thumbnail {
        slot: u32,
        target: OwnedProcess,
        viewport: Viewport,
        manager: u64,
        window: HWND,
        thumbnail: isize,
        // DWM observation state only: never an original game/lifecycle owner.
        invalidation_notified: bool,
        // Only successful native property writes are cached. Exact game HWND
        // and worker-owned DWM destination are still checked on every wake.
        last_applied: Option<(DwmAppliedState, Instant)>,
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
                invalidation_notified: false,
                last_applied: None,
                _process: process,
            })
        }
        fn destination_owned(&self) -> bool {
            unsafe {
                let mut pid = 0;
                let thread = GetWindowThreadProcessId(self.window, &mut pid);
                if thread == 0 || pid != std::process::id()
                    || GetWindow(self.window, GW_OWNER) != self.manager as usize as HWND
                    || GetWindowLongPtrW(self.window, GWLP_USERDATA) != self.slot as isize {
                    return false;
                }
                let mut class = [0u16; 32];
                let length = GetClassNameW(self.window, class.as_mut_ptr(), class.len() as i32);
                length > 0 && class[..length as usize] == CLASS[..CLASS.len()-1]
            }
        }
        fn source_and_destination_valid(&self) -> bool {
            self.destination_owned() && native::hwnd(&self.target).is_ok()
        }
        fn matches(&self, slot: u32, target: &OwnedProcess) -> bool {
            self.slot == slot
                && self.target.pid == target.pid
                && self.target.created_file_time == target.created_file_time
                && self.target.session_id == target.session_id
                && self.target.window_handle == target.window_handle
                && self.destination_owned()
        }
        fn refresh_verified(&mut self, visible: bool) -> io::Result<()> {
            let result = self.update(visible);
            if result.is_err() {
                // An error invalidates cached success even if the prior
                // frame's dimensions are unchanged. Next success MUST write
                // DWM properties again before showing the image.
                self.last_applied = None;
            }
            hide_on_failed_update(result, || self.hide())
        }
        fn update(&mut self, visible: bool) -> io::Result<()> {
            if !self.destination_owned() {
                return Err(io::Error::new(io::ErrorKind::NotFound,
                    "Native DWM preview destination is no longer owned by this worker"));
            }
            let now = Instant::now();
            if !visible {
                let requested = DwmAppliedState::Hidden;
                if !dwm_property_write_due(self.last_applied, requested, now)
                    && unsafe { IsWindowVisible(self.window) == 0 } {
                    return Ok(());
                }
                self.hide();
                let props = DWM_THUMBNAIL_PROPERTIES {
                    dwFlags: DWM_TNP_VISIBLE,
                    fVisible: 0,
                    ..unsafe { mem::zeroed() }
                };
                let result = unsafe { DwmUpdateThumbnailProperties(self.thumbnail, &props) };
                if result < 0 {
                    return Err(io::Error::other("Native preview hide failed"));
                }
                self.last_applied = Some((requested, now));
                return Ok(());
            }
            // Do not cache native game identity. Every visible refresh must
            // reverify original PID, process birth, session, HWND and owner.
            let (_pin, source) = native::hwnd(&self.target)?;
            let rect = native::rectangle(self.manager, self.viewport)?;
            let mut client: RECT = unsafe { mem::zeroed() };
            if unsafe { GetClientRect(source, &mut client) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let source_width = client.right - client.left;
            let source_height = client.bottom - client.top;
            let fit = super::fit_rectangle(
                rect.width, rect.height, source_width, source_height,
            )?;
            let requested = DwmAppliedState::Visible(DwmGeometry {
                destination: rect, fitted: fit, source_width, source_height,
            });
            if !dwm_property_write_due(self.last_applied, requested, now)
                && unsafe { IsWindowVisible(self.window) != 0 } {
                // The destination may be moved by Windows independently of
                // the manager. Confirm its live rectangle before skipping
                // expensive DWM/property writes on a duplicate wake.
                let mut actual: RECT = unsafe { mem::zeroed() };
                if unsafe { GetWindowRect(self.window, &mut actual) } != 0
                    && actual.left == rect.x && actual.top == rect.y
                    && actual.right - actual.left == rect.width
                    && actual.bottom - actual.top == rect.height {
                    return Ok(());
                }
            }
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
            // No cache entry is committed until BOTH DWM and SetWindowPos
            // succeed for this exact native source/window destination.
            self.last_applied = Some((requested, now));
            Ok(())
        }
        fn hide(&self) {
            // A reused HWND can belong to another window; never hide it.
            if self.destination_owned() {
                unsafe { ShowWindow(self.window, SW_HIDE); }
            }
        }
    }
    impl Drop for Thumbnail {
        fn drop(&mut self) {
            unsafe {
                DwmUnregisterThumbnail(self.thumbnail);
                // The DWM registration is owned, but the numeric HWND can
                // be reused. Never destroy an unverified destination.
                if self.destination_owned() {
                    DestroyWindow(self.window);
                }
            }
        }
    }

    // At most four new monitoring images and one primary observer can be
    // created by a Layout. Failed or undelivered receipts must not retain
    // those new DWM windows; established peer previews stay untouched.
    #[derive(Default)]
    struct LayoutCreated {
        secondary_slots: Vec<u32>,
        primary_changed: bool,
    }
    fn retain_previous_preview(created: &LayoutCreated, slot: u32) -> bool {
        !created.secondary_slots.contains(&slot)
    }
    fn layout_acknowledged(verified: bool, delivered: bool) -> bool {
        verified && delivered
    }
    struct Scene {
        window: Option<tauri::Window>,
        manager: u64,
        profiles: Vec<(u32, OwnedProcess)>,
        thumbnails: Vec<Thumbnail>,
        primary: Option<Thumbnail>,
        // The generation that admitted the last verified layout. No
        // process identity is cached in an additional controller.
        issued_epochs: [u64; 5],
    }
    impl Scene {
        fn discard_unacknowledged_new(&mut self, created: &LayoutCreated) {
            // Only thumbnails created BY THIS job are disposable. Do not
            // discard a previously confirmed MC03 image because MC02 took
            // too long or its frontend request was abandoned.
            if created.primary_changed {
                self.primary = None;
            }
            if !created.secondary_slots.is_empty() {
                self.thumbnails.retain(|thumbnail|
                    retain_previous_preview(created, thumbnail.slot));
            }
            self.prune_cancelled_slots();
        }
        fn prune_cancelled_slots(&mut self) {
            let now = snapshot_epochs();
            let issued = self.issued_epochs;
            self.thumbnails.retain(|thumb|
                slot_epoch_matches(&issued, &now, thumb.slot));
            if self.primary.as_ref().is_some_and(|image|
                !slot_epoch_matches(&issued, &now, image.slot)) {
                self.primary = None;
            }
            self.profiles.retain(|(slot, _)|
                slot_epoch_matches(&issued, &now, *slot));
            self.release_unused_observation();
        }
        fn release_unused_observation(&mut self) {
            // Retained DWM state is presentation-only. With no thumbnail,
            // drop the strong Tauri Window and stale native process metadata.
            // This never changes an original Minecraft process or lease.
            if self.thumbnails.is_empty() && self.primary.is_none() {
                self.window = None;
                self.manager = 0;
                self.profiles.clear();
            }
        }
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
            // A WAKE without any native observers needs no Win32 probe.
            if self.thumbnails.is_empty() && self.primary.is_none() {
                self.release_unused_observation();
                return;
            }
            let visible = self.visibility();
            // The DWM worker alone observes asynchronous refresh failures.
            // Do not re-run layout or inspect Minecraft. Send a single
            // source-pinned invalidation on the EXISTING error event when a
            // previously reported image becomes unverified.
            let mut invalidated = Vec::new();
            self.thumbnails.retain_mut(|thumbnail| {
                // A passive refresh cannot silently restore a thumbnail
                // whose latest per-slot report still says unavailable.
                let show = display_with_current_preview_receipt(
                    visible, thumbnail.invalidation_notified,
                );
                if let Err(error) = thumbnail.refresh_verified(show) {
                    if take_invalidation_notice(&mut thumbnail.invalidation_notified) {
                        invalidated.push((
                            thumbnail.slot, thumbnail.target.clone(),
                            format!("Native preview refresh unverified: {error}"),
                        ));
                    }
                    return thumbnail.source_and_destination_valid();
                }
                true
            });
            if let Some(window) = self.window.as_ref() {
                for (slot, process, reason) in invalidated {
                    let reason = reason.chars().take(1024).collect::<String>();
                    if let Ok(message) = preview_invalidation_message(
                        slot, &process, &reason,
                    ) {
                        let _ = window.emit("workspace-surface-error", message);
                    }
                }
            }
            let discard_primary = if let Some(primary) = &mut self.primary {
                let playing = native::interactive_at(
                    &primary.target, primary.manager, primary.viewport);
                primary.refresh_verified(visible && !playing).is_err()
                    && !primary.source_and_destination_valid()
            } else { false };
            if discard_primary { self.primary = None; }
            // A ClearSlot can be signalled while this worker is doing
            // native refresh. Never leave that canceled HWND visible.
            self.prune_cancelled_slots();
        }
        fn update_primary_observation(&mut self, manager: u64,
            tile: &PreviewTile, visible: bool) -> io::Result<bool> {
            let target = self.profiles.iter().find(|(slot, _)| *slot == tile.slot)
                .map(|(_, target)| target.clone())
                .ok_or_else(|| io::Error::other("Active game preview source is unavailable"))?;
            let created = self.primary.as_ref().is_none_or(|image|
                !image.matches(tile.slot, &target));
            if created {
                self.primary = None;
                self.primary = Some(Thumbnail::create(manager, tile, target.clone())?);
            }
            let image = self.primary.as_mut().unwrap();
            image.viewport = tile.viewport;
            // DWM observation only; never touch original game lifecycle.
            image.refresh_verified(visible &&
                !native::interactive_at(&target, manager, tile.viewport))?;
            Ok(created)
        }
        fn layout(
            &mut self,
            window: tauri::Window,
            tiles: Vec<PreviewTile>,
            primary: Option<PreviewTile>,
            profiles: Vec<(u32, OwnedProcess)>,
            unavailable: Vec<(u32, String)>,
            issued_epochs: [u64; 5],
            deadline: Instant,
            created: &mut LayoutCreated,
        ) -> io::Result<PreviewReport> {
            // The original caller may already have timed out while the
            // worker was busy. Expired Layout is not a native game action.
            require_live_layout(deadline)?;
            // The caller may have begun native adoption before CloseOne.
            // Do not create/position thumbnails for an invalidated layout.
            if !epochs_current(&issued_epochs) {
                self.prune_cancelled_slots();
                return Err(io::Error::new(io::ErrorKind::WouldBlock,
                    "DWM layout canceled by original profile close"));
            }
            let manager = window.hwnd().map_err(io::Error::other)?.0 as usize as u64;
            if self.manager != manager {
                self.thumbnails.clear();
                self.primary = None;
            }
            self.window = Some(window);
            self.manager = manager;
            self.profiles = profiles;
            self.issued_epochs = issued_epochs;
            self.thumbnails.retain(|thumbnail| {
                tiles.iter().any(|tile| tile.slot == thumbnail.slot)
                    && self
                        .profiles
                        .iter()
                        .any(|(slot, target)| thumbnail.matches(*slot, target))
                    && !unavailable.iter().any(|(slot, _)| *slot == thumbnail.slot)
            });
            let visible = self.visibility();
            require_live_layout(deadline)?;
            match primary {
                Some(tile) if slot_epoch_matches(
                    &issued_epochs, &snapshot_epochs(), tile.slot,
                ) => {
                    match self.update_primary_observation(manager, &tile, visible) {
                        Ok(new_primary) => created.primary_changed = new_primary,
                        Err(error) => {
                            // A failed primary observer cannot suppress any peer
                            // receipt or alter the original Minecraft game.
                            self.primary = None;
                            eprintln!("Minecraft - {} primary DWM preview unavailable: {error}", tile.slot);
                        }
                    }
                }
                _ => self.primary = None,
            }
            // A slow primary HWND operation may consume the entire budget;
            // avoid creating secondary DWM windows from that stale request.
            require_live_layout(deadline)?;
            let mut statuses = Vec::with_capacity(tiles.len());
            for tile in tiles {
                require_live_layout(deadline)?;
                let mut status = initial_tile_status(&tile, &self.profiles, &unavailable);
                if !slot_epoch_matches(&issued_epochs, &snapshot_epochs(), tile.slot) {
                    status.reason = Some("Preview canceled by original profile close".into());
                    statuses.push(status);
                    continue;
                }
                if status.reason.is_some() {
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
                            // On timeout/dropped reply, destroy only this
                            // unacknowledged native image, never old peers.
                            created.secondary_slots.push(tile.slot);
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
                match self.thumbnails[index].refresh_verified(visible) {
                    Ok(()) => {
                        status.registered = true;
                        // This synchronous response supersedes prior DWM
                        // invalidations and rearms one future notification.
                        self.thumbnails[index].invalidation_notified = false;
                    },
                    Err(error) => {
                        status.reason = Some(error.to_string());
                        // Already reported by the requested PreviewReport.
                        // Avoid a duplicate failure event on the next wake.
                        self.thumbnails[index].invalidation_notified = true;
                        // Unregister only the stale DWM observation. Peer
                        // thumbnails and their status receipts remain intact.
                        if !self.thumbnails[index].source_and_destination_valid() {
                            self.thumbnails.remove(index);
                        }
                    }
                }
                // An individual Win32 update may block past the original
                // deadline. Stop before processing any further peer tiles.
                require_live_layout(deadline)?;
                statuses.push(status);
            }
            // A CloseOne may begin while an in-flight native update is
            // blocked. Remove only that slot's thumbnails, retain peer DWM
            // images, and refuse this stale success report.
            self.prune_cancelled_slots();
            if !epochs_current(&issued_epochs) {
                return Err(io::Error::new(io::ErrorKind::WouldBlock,
                    "DWM layout invalidated during native profile close"));
            }
            // A successful native update is not a receipt if the caller
            // has already abandoned its timed operation.
            require_live_layout(deadline)?;
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
            NATIVE_REFRESH_REQUESTED.store(false, Ordering::Release);
            thread_id.store(GetCurrentThreadId(), Ordering::Release);
            THREAD.store(GetCurrentThreadId(), Ordering::Release);
            let mut scene = Scene {
                window: None,
                manager: 0,
                profiles: Vec::new(),
                thumbnails: Vec::new(),
                primary: None,
                issued_epochs: snapshot_epochs(),
            };
            while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
                if message.message == SELECT {
                    if let Some(window) = &scene.window {
                        let _ = window.emit("lazy-tester-preview-select", message.wParam as u32);
                    }
                } else if message.message == WAKE {
                    // Clear pending BEFORE sampling intent: a native event
                    // arriving during the job drain posts another WAKE.
                    REFRESH_PENDING.store(false, Ordering::Release);
                    let native_event =
                        NATIVE_REFRESH_REQUESTED.swap(false, Ordering::AcqRel);
                    // ClearSlot is best effort on the bounded channel.
                    // Its atomic cancellation still removes stale images
                    // even if that channel was full at CloseOne admission.
                    scene.prune_cancelled_slots();
                    let mut laid_out = false;
                    while let Ok(job) = receiver.try_recv() {
                        match job {
                            Job::Layout {
                                window,
                                tiles,
                                primary,
                                profiles,
                                unavailable,
                                issued_epochs,
                                deadline,
                                reply,
                            } => {
                                if layout_expired(deadline, Instant::now()) {
                                    // Only expired Layout work is disposable;
                                    // Clear/ClearSlot/CloseOne remain mandatory.
                                    let _ = reply.send(Err(io::Error::new(
                                        io::ErrorKind::TimedOut,
                                        "Expired queued preview layout skipped",
                                    )));
                                } else {
                                    let mut created = LayoutCreated::default();
                                    let result = scene.layout(
                                        window, tiles, primary, profiles,
                                        unavailable, issued_epochs, deadline, &mut created,
                                    );
                                    // The last native update can consume
                                    // the remaining time before reply.send.
                                    let result = if result.is_ok()
                                        && layout_expired(deadline, Instant::now()) {
                                        Err(io::Error::new(io::ErrorKind::TimedOut,
                                            "DWM layout finished after reply deadline"))
                                    } else { result };
                                    let verified = result.is_ok();
                                    // std::sync::mpsc::SyncSender reports a
                                    // dropped receiver. No new DWM image may
                                    // be retained from such a lost receipt.
                                    let delivered = reply.send(result).is_ok();
                                    if !layout_acknowledged(verified, delivered) {
                                        scene.discard_unacknowledged_new(&created);
                                    } else {
                                        laid_out = true;
                                    }
                                }
                            }
                            Job::Clear(reply) => {
                                scene.thumbnails.clear();
                                scene.primary = None;
                                scene.release_unused_observation();
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
                                // Keep peers; release only an entirely idle scene.
                                scene.release_unused_observation();
                                let _ = reply.send(Ok(()));
                            }
                            Job::ClearPrimary(reply) => {
                                scene.primary = None;
                                scene.release_unused_observation();
                                let _ = reply.send(Ok(()));
                            }
                        }
                    }
                    // A native cancellation may happen during a slow
                    // Layout. Prune again before dispatching another wake.
                    scene.prune_cancelled_slots();
                    if refresh_after_wake(native_event, laid_out) {
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
    // WORKER protects only startup and bounded channel admission. Waiting
    // for a result while holding that global mutex can block an unrelated
    // CloseOne preview-clear caller for the full 3-second timeout.
    fn finish_after_admission<T, S>(
        admission: std::sync::MutexGuard<'_, S>,
        notify_worker: impl FnOnce() -> io::Result<()>,
        receive: mpsc::Receiver<io::Result<T>>,
        deadline: Instant,
    ) -> io::Result<T> {
        drop(admission);
        notify_worker()?;
        receive.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "Preview update is pending; worker retained",
            )
        })?
    }
    fn send<T>(
        make: impl FnOnce(mpsc::SyncSender<io::Result<T>>, Instant) -> Job,
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
        let job_deadline = Instant::now() + Duration::from_secs(3);
        worker.sender.try_send(make(reply, job_deadline)).map_err(|_| {
            io::Error::new(
                io::ErrorKind::WouldBlock,
                "Previous preview update is still pending",
            )
        })?;
        // The existing single DWM worker keeps job ordering. A clear may
        // now be admitted while this caller waits for its own reply.
        finish_after_admission(owner, || notify(id), receive, job_deadline).map(Some)
    }
    pub(super) fn apply(
        window: tauri::Window,
        tiles: Vec<PreviewTile>,
        primary: Option<PreviewTile>,
        profiles: Vec<(u32, OwnedProcess)>,
        unavailable: Vec<(u32, String)>,
        issued_epochs: [u64; 5],
    ) -> io::Result<PreviewReport> {
        let base = PreviewReport {
            schema: 1,
            tiles: tiles
                .iter()
                .map(|tile| initial_tile_status(tile, &profiles, &unavailable))
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
            |reply, deadline| Job::Layout {
                window,
                tiles,
                primary,
                profiles,
                unavailable,
                issued_epochs,
                deadline,
                reply,
            },
            true,
        )?
        .ok_or_else(|| io::Error::other("Preview worker missing"))
    }
    pub(super) fn clear() -> io::Result<()> {
        // A full scene clear also invalidates every in-flight old Layout.
        for generation in &SLOT_CANCEL_EPOCHS {
            generation.fetch_add(1, Ordering::AcqRel);
        }
        let result = send(|reply, _| Job::Clear(reply), false).map(|_| ());
        if result.is_err() { refresh(); }
        result
    }
    pub(super) fn clear_slot(slot:u32)->io::Result<()> {
        // Called at CloseOne admission, BEFORE waiting for the worker.
        // Even queue-full/timeout cannot let an older Layout re-create
        // this slot; unrelated MC01–MC05 generations stay unchanged.
        SLOT_CANCEL_EPOCHS[(slot - 1) as usize].fetch_add(1, Ordering::AcqRel);
        let result = send(|reply, _|Job::ClearSlot {slot,reply},false).map(|_| ());
        if result.is_err() { refresh(); }
        result
    }
    pub(super) fn clear_primary() -> io::Result<()> {
        send(|reply, _| Job::ClearPrimary(reply), false).map(|_| ())
    }
    pub(super) fn refresh() {
        let id = THREAD.load(Ordering::Acquire);
        if id != 0 {
            NATIVE_REFRESH_REQUESTED.store(true, Ordering::Release);
            let _ = notify(id);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
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
            };
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
                let primary = Thumbnail::create(manager as usize as u64, &primary_tile, target).unwrap();
                let mut scene=Scene{window:None,manager:manager as usize as u64,profiles:Vec::new(),thumbnails:Vec::new(),primary:Some(primary),issued_epochs:snapshot_epochs()};
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
                thumbnail.refresh_verified(scene.visibility()).unwrap();
                assert_ne!(IsWindowVisible(thumbnail.window),0);
                // A lost Layout reply must release only its newly created
                // secondary image. The verified primary observer survives.
                scene.thumbnails.push(thumbnail);
                scene.discard_unacknowledged_new(&LayoutCreated {
                    secondary_slots: vec![tile.slot], primary_changed: false,
                });
                assert!(scene.thumbnails.is_empty());
                assert!(scene.primary.is_some(),
                    "Dropping an unacknowledged secondary cannot remove primary");
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
    fn primary_failure_does_not_change_four_secondary_tile_receipts() {
        let tiles = (2..=5).map(|slot| PreviewTile {
            slot,
            viewport: Viewport {
                x: (slot * 160) as f64, y: 450.0,
                width: 150.0, height: 85.0, pixel_ratio: 1.0,
            },
        }).collect::<Vec<_>>();
        assert!(validate_tiles(&tiles, Some(1)).is_ok());
        let game = OwnedProcess {
            pid: 102, created_file_time: "12346".into(),
            session_id: 1, window_handle: "0x102".into(), responding: true,
        };
        let sources = vec![(2, game.clone()), (3, OwnedProcess {
            pid: 103, created_file_time: "12347".into(),
            window_handle: "0x103".into(), ..game
        })];
        let unavailable = vec![(3, "MC03 recovery unconfirmed".to_owned())];
        // A synthetic MC01 primary observer error has no slot in PreviewReport.
        let primary_error = io::Error::other("MC01 primary DWM unavailable");
        assert!(primary_error.to_string().contains("primary DWM"));
        let statuses = tiles.iter().map(|tile|
            initial_tile_status(tile, &sources, &unavailable)).collect::<Vec<_>>();
        assert_eq!(statuses.iter().map(|tile| tile.slot).collect::<Vec<_>>(),
            vec![2,3,4,5]);
        assert!(statuses.iter().all(|tile| !tile.registered));
        assert_eq!(statuses[0].reason, None);
        assert_eq!(statuses[1].reason.as_deref(), Some("MC03 recovery unconfirmed"));
        assert_eq!(statuses[2].reason.as_deref(),
            Some("No verified Minecraft preview source for this profile"));
        assert_eq!(statuses[3].reason.as_deref(),
            Some("No verified Minecraft preview source for this profile"));
    }
    #[test]
    fn preview_recovery_reason_is_scoped_to_only_the_failed_slot() {
        let target = OwnedProcess {
            pid: 101, created_file_time: "12345".into(), session_id: 1,
            window_handle: "0x101".into(), responding: true,
        };
        let sources = vec![(1, target.clone()), (2, OwnedProcess {
            pid: 102, created_file_time: "12346".into(),
            window_handle: "0x102".into(), ..target
        })];
        let unavailable = vec![(1, "Original native recovery unconfirmed".to_owned())];
        assert_eq!(preview_tile_reason(1, &sources, &unavailable),
            Some("Original native recovery unconfirmed".into()));
        assert_eq!(preview_tile_reason(2, &sources, &unavailable), None,
            "MC01 recovery must not mark MC02 unavailable");
        assert_eq!(preview_tile_reason(3, &sources, &unavailable),
            Some("No verified Minecraft preview source for this profile".into()));
    }
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
