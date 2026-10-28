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
mod rollback_tests;
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
        // clear_slot() already invalidated this slot and posted the
        // existing control-only WAKE if admission/reply failed.
        // A second native refresh would needlessly check healthy peers.
        // Preview uncertainty must never block original game CloseOne.
        eprintln!("Minecraft - {slot} preview cleanup pending during normal close: {error}");
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
mod windows_tests;
