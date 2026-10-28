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
// A successfully received Windows message is sufficient to drain jobs
// admitted before a failed PostThreadMessageW. No timer or retry thread
// is needed; the same bounded worker owns each queued operation.
fn drain_ready_jobs<T>(
    receiver: &mpsc::Receiver<T>, mut process: impl FnMut(T),
) {
    while let Ok(job) = receiver.try_recv() {
        process(job);
    }
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
fn epoch_snapshot_matches(issued: &[u64; 5], current: &[u64; 5]) -> bool {
    issued == current
}
fn epochs_current(issued: &[u64; 5]) -> bool {
    epoch_snapshot_matches(issued, &snapshot_epochs())
}
pub(super) fn require_current_admission(issued: &[u64; 5]) -> io::Result<()> {
    if !epochs_current(issued) {
        return Err(io::Error::new(io::ErrorKind::WouldBlock,
            "DWM preview source adoption canceled by Clear or CloseOne"));
    }
    Ok(())
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

// At most four new images and one primary can be created. A rejected
// Layout must also restore the original viewport and failure flag of
// *reused* thumbnails, rather than leaving a stale geometry/visibility
// change that no frontend report acknowledged.
// Pinned native registration and destination, not just a slot number.
// Reusing a numeric HWND after manager/source replacement must never
// authorize a rollback of a different DWM observation.
#[derive(Clone, Copy)]
struct PreviousImage {
    slot: u32,
    manager: u64,
    destination: usize,
    registration: isize,
    viewport: Viewport,
    invalidated: bool,
    applied: Option<DwmAppliedState>,
}
impl PreviousImage {
    fn capture(image: &Thumbnail) -> Self {
        Self {
            slot: image.slot,
            manager: image.manager,
            destination: image.window as usize,
            registration: image.thumbnail,
            viewport: image.viewport,
            invalidated: image.invalidation_notified,
            applied: image.last_applied.map(|(state, _)| state),
        }
    }
    fn same_registration(self, image: &Thumbnail) -> bool {
        self.slot == image.slot
            && self.manager == image.manager
            && self.destination == image.window as usize
            && self.registration == image.thumbnail
    }
    fn previously_visible(self, manager_visible: bool) -> bool {
        display_with_current_preview_receipt(manager_visible, self.invalidated)
            && matches!(self.applied, Some(DwmAppliedState::Visible(_)))
    }
}
#[derive(Default)]
struct LayoutCreated {
    secondary_slots: Vec<u32>,
    primary_changed: bool,
    // The former observer remains owned by this single Layout until
    // the replacement's verified reply is delivered or rolled back.
    replaced_primary: Option<(Thumbnail, PreviousImage)>,
    // At most four previous accepted secondaries stay owned until
    // this Layout receives a valid frontend acknowledgement.
    displaced_secondary: Vec<(Thumbnail, PreviousImage)>,
    reused_secondary: Vec<PreviousImage>,
    reused_primary: Option<PreviousImage>,
}
fn previous_primary_can_restore(
    old: PreviousImage, manager: u64,
    source_current: bool, slot_current: bool,
) -> bool {
    old.manager == manager && source_current && slot_current
}
fn viewport_changed(a: Viewport, b: Viewport) -> bool {
    a.x != b.x || a.y != b.y || a.width != b.width
        || a.height != b.height || a.pixel_ratio != b.pixel_ratio
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
    // Keeps the native owner valid only until this Layout commits or
    // aborts, not another source of process or game lifetime truth.
    layout_in_progress: bool,
}
impl Scene {
    fn stage_unrequested_secondary(
        &mut self, tiles: &[PreviewTile],
        unavailable: &[(u32, String)], created: &mut LayoutCreated,
    ) {
        // Reuse the existing Vec instead of re-allocating on every
        // Layout. Four thumbnails max; remove preserves ordering.
        let mut index = 0;
        while index < self.thumbnails.len() {
            let image = &self.thumbnails[index];
            let current_source = self.profiles.iter().any(|(slot, target)|
                image.matches(*slot, target));
            let unavailable_now = unavailable.iter().any(|(slot, _)|
                *slot == image.slot);
            if current_source && !unavailable_now &&
                tiles.iter().any(|tile| tile.slot == image.slot) {
                index += 1;
                continue;
            }
            let image = self.thumbnails.remove(index);
            if current_source && !unavailable_now
                && image.source_and_destination_valid() {
                // Hide, but don't unregister an accepted observer from
                // an unacknowledged proposed Layout.
                let snapshot = PreviousImage::capture(&image);
                image.hide();
                created.displaced_secondary.push((image, snapshot));
            }
            // Retired/invalid original HWNDs fail closed.
        }
    }
    fn rollback_manager_owned(&self) -> bool {
        let manager = self.manager as usize as HWND;
        if manager.is_null() { return false; }
        unsafe {
            let mut pid = 0;
            if IsWindow(manager) == 0
                || GetWindowThreadProcessId(manager, &mut pid) == 0
                || pid != std::process::id() {
                return false;
            }
        }
        // The real Tauri owner must still resolve to the SAME manager;
        // synthetic Win32 fixtures do not have a Tauri Window wrapper.
        self.window.as_ref().is_none_or(|window|
            window.hwnd().ok().is_some_and(|hwnd|
                hwnd.0 as usize as u64 == self.manager))
    }
    fn discard_unacknowledged_new(&mut self, created: &mut LayoutCreated) {
        // Manager replacement or HWND reuse makes old preview rollback
        // unsafe. Fail closed instead of moving old DWM registrations
        // into another host. This does not touch the original game.
        if !self.rollback_manager_owned() {
            self.thumbnails.clear();
            self.primary = None;
            self.release_unused_observation();
            return;
        }
        if created.primary_changed {
            // New primary never had a valid delivered receipt.
            self.primary = None;
            if let Some((mut old, saved)) = created.replaced_primary.take() {
                let current_source = saved.same_registration(&old)
                    && old.source_and_destination_valid()
                    && self.profiles.iter().any(|(slot, process)|
                        old.matches(*slot, process));
                let current_epoch = slot_epoch_matches(
                    &self.issued_epochs, &snapshot_epochs(), old.slot,
                );
                if previous_primary_can_restore(
                    saved, self.manager, current_source, current_epoch,
                ) {
                    old.viewport = saved.viewport;
                    let playing = native::interactive_at(
                        &old.target, old.manager, saved.viewport);
                    // The previous observer can return only through its
                    // original exact source/HWND/DWM verification path.
                    let show = saved.previously_visible(self.visibility()) && !playing;
                    if let Err(error) = old.refresh_verified(show) {
                        eprintln!("Minecraft - {} displaced primary DWM recovery unverified: {error}",
                            old.slot);
                    } else {
                        self.primary = Some(old);
                    }
                }
            }
        }
        if !created.secondary_slots.is_empty() {
            self.thumbnails.retain(|thumbnail|
                retain_previous_preview(created, thumbnail.slot));
        }
        let visible = self.visibility();
        let epochs = snapshot_epochs();
        let issued = self.issued_epochs;
        // Only a still-current, exact native registration can recover
        // the old viewport; a replacement source/manager cannot inherit
        // geometry or validation status from this abandoned Layout.
        if let (Some(previous), Some(primary)) =
            (created.reused_primary, self.primary.as_mut()) {
            if previous.same_registration(primary)
                && slot_epoch_matches(&issued, &epochs, primary.slot)
                && self.profiles.iter().any(|(slot, target)|
                    primary.matches(*slot, target))
                && primary.source_and_destination_valid() {
                let changed = viewport_changed(primary.viewport, previous.viewport)
                    || primary.last_applied.map(|(state, _)| state) != previous.applied;
                primary.viewport = previous.viewport;
                if changed {
                    let playing = native::interactive_at(
                        &primary.target, primary.manager, previous.viewport);
                    let show = previous.previously_visible(visible) && !playing;
                    if let Err(error) = primary.refresh_verified(show) {
                        eprintln!("Minecraft - {} primary DWM rollback unverified: {error}",
                            primary.slot);
                    }
                }
            } else if previous.same_registration(primary) {
                // Exact original process/window cannot be confirmed.
                // Never leave an old primary DWM image behind.
                self.primary = None;
            }
        }
        let mut stale_slots = Vec::new();
        let mut notices = Vec::new();
        for &previous in &created.reused_secondary {
            if !slot_epoch_matches(&issued, &epochs, previous.slot) { continue; }
            let Some(image) = self.thumbnails.iter_mut()
                .find(|image| image.slot == previous.slot) else { continue; };
            if !previous.same_registration(image) {
                // A new source or reused HWND is not this checkpoint.
                // Do not update its geometry or discard a newer image.
                continue;
            }
            if !self.profiles.iter().any(|(slot, target)|
                image.matches(*slot, target))
                || !image.source_and_destination_valid() {
                stale_slots.push(previous.slot);
                if !previous.invalidated {
                    notices.push((image.slot, image.target.clone(),
                        "Previous DWM source or destination no longer verified".to_owned()));
                }
                continue;
            }
            let needs_native_restore =
                viewport_changed(image.viewport, previous.viewport)
                    || image.invalidation_notified != previous.invalidated
                    || image.last_applied.map(|(state, _)| state) != previous.applied;
            image.viewport = previous.viewport;
            image.invalidation_notified = previous.invalidated;
            if !needs_native_restore { continue; }
            if let Err(error) = image.refresh_verified(previous.previously_visible(visible)) {
                if take_invalidation_notice(&mut image.invalidation_notified) {
                    notices.push((image.slot, image.target.clone(),
                        format!("Previous native preview geometry could not be restored: {error}")));
                }
            }
        }
        if !stale_slots.is_empty() {
            self.thumbnails.retain(|image| !stale_slots.contains(&image.slot));
        }
        // An unacknowledged Layout must not permanently remove the
        // accepted secondary observations it merely proposed to omit.
        // Reinsert only the SAME original DWM registration, source,
        // manager and slot generation; never overwrite a new observer.
        for (mut old, saved) in created.displaced_secondary.drain(..) {
            if !slot_epoch_matches(&issued, &epochs, old.slot)
                || self.thumbnails.iter().any(|image| image.slot == old.slot)
                || !saved.same_registration(&old)
                || !self.profiles.iter().any(|(slot, target)|
                    old.matches(*slot, target))
                || !old.source_and_destination_valid() {
                continue;
            }
            old.viewport = saved.viewport;
            old.invalidation_notified = saved.invalidated;
            if let Err(error) = old.refresh_verified(saved.previously_visible(visible)) {
                if take_invalidation_notice(&mut old.invalidation_notified) {
                    notices.push((old.slot, old.target.clone(),
                        format!("Previous secondary DWM observer unavailable: {error}")));
                }
            } else {
                self.thumbnails.push(old);
            }
        }
        if let Some(window) = self.window.as_ref() {
            for (slot, process, reason) in notices {
                let reason = reason.chars().take(1024).collect::<String>();
                if let Ok(message) = preview_invalidation_message(
                    slot, &process, &reason,
                ) {
                    let _ = window.emit("workspace-surface-error", message);
                }
            }
        }
        // A concurrent CloseOne wins even if a native readback above
        // stalled. Peer preview source/process identities remain pinned.
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
        if !self.layout_in_progress
            && self.thumbnails.is_empty() && self.primary.is_none() {
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
    fn selection_is_verified(&self, slot: u32) -> bool {
        // A SELECT from a queued click on an already-closed thumbnail
        // is not authority to activate another Minecraft process.
        if !(1..=5).contains(&slot)
            || !slot_epoch_matches(
                &self.issued_epochs, &snapshot_epochs(), slot,
            )
            || !self.visibility() {
            return false;
        }
        self.thumbnails.iter().chain(self.primary.iter()).any(|image| {
            image.slot == slot
                && image.manager == self.manager
                && unsafe { IsWindowVisible(image.window) != 0 }
                && self.profiles.iter().any(|(profile, target)|
                    *profile == slot && image.matches(*profile, target))
                && image.source_and_destination_valid()
        })
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
        tile: &PreviewTile, visible: bool,
        transaction: &mut LayoutCreated,
    ) -> io::Result<()> {
        let target = self.profiles.iter().find(|(slot, _)| *slot == tile.slot)
            .map(|(_, target)| target.clone())
            .ok_or_else(|| io::Error::other("Active game preview source is unavailable"))?;
        let replacing = self.primary.as_ref().is_none_or(|image|
            !image.matches(tile.slot, &target));
        if replacing {
            // The candidate can fail to register or be updated. Until
            // both steps succeed, do not mutate the previous primary.
            let mut candidate = Thumbnail::create(manager, tile, target.clone())?;
            candidate.refresh_verified(visible &&
                !native::interactive_at(&target, manager, tile.viewport))?;
            if let Some(old) = self.primary.take() {
                let proof = PreviousImage::capture(&old);
                old.hide();
                transaction.replaced_primary = Some((old, proof));
            }
            self.primary = Some(candidate);
            transaction.primary_changed = true;
            return Ok(());
        }
        let image = self.primary.as_mut().unwrap();
        transaction.reused_primary = Some(PreviousImage::capture(image));
        image.viewport = tile.viewport;
        // Reused thumbnails keep their old snapshot even if this update
        // fails partway through and hides the image.
        image.refresh_verified(visible &&
            !native::interactive_at(&target, manager, tile.viewport))
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
        self.layout_in_progress = true;
        self.stage_unrequested_secondary(&tiles, &unavailable, created);
        let visible = self.visibility();
        require_live_layout(deadline)?;
        match primary {
            Some(tile) if slot_epoch_matches(
                &issued_epochs, &snapshot_epochs(), tile.slot,
            ) => {
                if let Err(error) =
                    self.update_primary_observation(manager, &tile, visible, created) {
                    // A failed NEW candidate cannot delete an earlier
                    // verified primary. A retired source still fails closed.
                    if self.primary.as_ref().is_some_and(|image|
                        !self.profiles.iter().any(|(slot, target)|
                            image.matches(*slot, target))
                        || !image.source_and_destination_valid()) {
                        self.primary = None;
                    }
                    eprintln!("Minecraft - {} primary DWM preview unavailable: {error}", tile.slot);
                }
            }
            _ => {
                // Removal also needs a delivered Layout; a dropped reply
                // can restore the old observer only if still current.
                if let Some(old) = self.primary.take() {
                    let proof = PreviousImage::capture(&old);
                    old.hide();
                    created.replaced_primary = Some((old, proof));
                    created.primary_changed = true;
                }
            },
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
                Some(index) => {
                    let previous = &self.thumbnails[index];
                    // Capture old geometry AND its latest published
                    // invalidation. An abandoned recovery report must
                    // not show a previously invalidated tile again.
                    created.reused_secondary.push(PreviousImage::capture(previous));
                    index
                }
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
            layout_in_progress: false,
        };
        while GetMessageW(&mut message, ptr::null_mut(), 0, 0) > 0 {
            let is_wake = message.message == WAKE;
            if is_wake {
                // A delivered WAKE releases coalescing admission BEFORE
                // draining jobs. A concurrent new request can post again.
                REFRESH_PENDING.store(false, Ordering::Release);
            }
            // A failed PostThreadMessageW can leave an admitted job on
            // the bounded channel. SELECT, WM_PAINT or ANY later valid
            // worker message must reconcile it, not only WAKE.
            let native_event =
                NATIVE_REFRESH_REQUESTED.swap(false, Ordering::AcqRel);
            scene.prune_cancelled_slots();
            let mut laid_out = false;
            drain_ready_jobs(&receiver, |job| {
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
                                    scene.discard_unacknowledged_new(&mut created);
                                } else {
                                    laid_out = true;
                                }
                                // Only this Layout may temporarily
                                // retain an otherwise empty Scene.
                                scene.layout_in_progress = false;
                                scene.release_unused_observation();
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
            });
            // A cleanup epoch may have changed during a long Layout.
            // Reconcile it before a SELECT can escape the DWM worker.
            scene.prune_cancelled_slots();
            if refresh_after_wake(native_event, laid_out) {
                scene.refresh();
            }
            if message.message == SELECT {
                // Validate wParam BEFORE narrowing it to a slot ID.
                // A stale or recycled destination must not select a
                // different live game after CloseOne/Clear.
                if (1..=5).contains(&message.wParam) {
                    let slot = message.wParam as u32;
                    if scene.selection_is_verified(slot) {
                        if let Some(window) = &scene.window {
                            let _ = window.emit("lazy-tester-preview-select", slot);
                        }
                    }
                }
            } else if !is_wake {
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
// Native event refresh and cleanup admission are different wake causes.
// The latter only needs the worker to prune canceled epochs; do not
// mark its four still-current peers for an extra HWND/DWM refresh.
fn control_cleanup_receipt<T>(
    result: io::Result<T>, wake: impl FnOnce(),
) -> io::Result<T> {
    if result.is_err() { wake(); }
    result
}
fn notify_control_cleanup() {
    let id = THREAD.load(Ordering::Acquire);
    if id != 0 {
        let _ = notify(id); // Existing coalesced WAKE; no native-refresh bit.
    }
}
pub(super) fn clear() -> io::Result<()> {
    // Invalidating these epochs remains authoritative even when the
    // bounded channel is full or a queued control reply times out.
    for generation in &SLOT_CANCEL_EPOCHS {
        generation.fetch_add(1, Ordering::AcqRel);
    }
    control_cleanup_receipt(
        send(|reply, _| Job::Clear(reply), false).map(|_| ()),
        notify_control_cleanup,
    )
}
pub(super) fn clear_slot(slot:u32)->io::Result<()> {
    // CloseOne owns the original game; DWM only invalidates its
    // presentation, without any other slot's native refresh.
    SLOT_CANCEL_EPOCHS[(slot - 1) as usize].fetch_add(1, Ordering::AcqRel);
    control_cleanup_receipt(
        send(|reply, _|Job::ClearSlot {slot,reply},false).map(|_| ()),
        notify_control_cleanup,
    )
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
mod tests;
