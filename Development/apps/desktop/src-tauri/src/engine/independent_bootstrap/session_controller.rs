//! One retained bootstrap owner. Full exit requests normal closure of its own games.
use super::*;
use crate::engine::{
    native_clients_session::{
        pipe::{Activity, Endpoint, Peer, Server},
        ProfileSession, SessionRequest, SessionSnapshot, SettingsReport,
    },
    process_identity::OwnedProcess,
};
use std::{io::Write, time::{Duration, Instant}};
#[cfg(test)]
use crate::engine::native_clients_session::SelectiveCategory;
#[cfg(feature="mc02-native-input")]
use crate::engine::native_clients_session::{
    AgentControlMode,AiLeaseBook,
};

#[cfg(all(test,feature="mc02-native-input"))]
use crate::engine::native_clients_session::Mc02BatchOrigin;
#[cfg(feature="mc02-native-input")]
#[path = "session_controller/experimental_input_operations.rs"]
mod experimental_input_operations;
#[cfg(all(test,feature="mc02-native-input"))]
use self::experimental_input_operations::{ai_writer_mode_matches, legacy_mc02_batch_matches_original,
    legacy_native_refused_before_publication, release_attempted_ai_group};

struct Sessions {
    stores: Vec<Option<SlotStorage>>,
    running: Vec<Option<StartedEducationObservation<1>>>,
    issues: Vec<Option<String>>,
    windows: Vec<Option<OwnedProcess>>,
    path: String,
    exiting: bool,
    // One best-effort storage maintenance job per confirmed successful exit, never a worker.
    pending_storage_maintenance: [bool; 5],
    #[cfg(feature="mc02-native-input")]
    ai_leases:AiLeaseBook, // ONE native actor: no additional game owner.
}
impl Sessions {
    fn prepare() -> io::Result<Self> {
        // Do not adopt arbitrary games or infer a profile from window titles.
        // Resolve once for this initial observation and the image to be pinned.
        // The existing post-pin RegisteredOnly check still runs before profile work.
        let target = super::super::registered_education_target()?;
        if !super::super::native_workspace_windows::inspect_for_registered_target(&target)?
            .processes
            .is_empty()
        {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists,
                "Existing Minecraft sessions are preserved. Close them normally before starting managed profiles."));
        }
        // An unsupported game version blocks Launch, not settings or profile observation.
        // Release this temporary image pin before retaining the idle controller.
        let (path, image, _) = registered_image::prepare_registered_image(target)?;
        let launch_issue = match registered_image::reviewed_singleton_recipe(file_digest(&image, 256 * 1024 * 1024)?) {
            Ok(_) => None,
            Err(error) if error.kind() == io::ErrorKind::Unsupported => Some(error.to_string()),
            Err(error) => return Err(error),
        };
        drop(image);
        // Opening five product profiles can recover an interrupted clear or
        // persist the data-location pointer. Reject missing/unreviewed DLLs
        // BEFORE any of those profile-side effects or a game launch.
        drop(BootstrapLibrary::reviewed()?);
        let stores = registered_slots::<5>()?.into_iter().map(Some).collect();
        Ok(Self {
            stores,
            running: (0..5).map(|_| None).collect(),
            issues: vec![launch_issue; 5],
            windows: vec![None; 5],
            path,
            exiting: false,
            pending_storage_maintenance: [false; 5],
            #[cfg(feature="mc02-native-input")]
            ai_leases:AiLeaseBook::default(),
        })
    }

    fn reap(&mut self) -> bool {
        self.reap_selected(&[true; 5])
    }

    // Only the already-exited target of CloseOne needs journal persistence
    // before its acknowledgement. Other slots retain their original handles
    // and journals for the next ordinary idle/Inspect maintenance cycle.
    // CloseAll supplies all five slots and retains its full cleanup contract.
    // Return whether an original exit observation failed; a signaled
    // handle must not spin when its exit receipt cannot be collected.
    fn reap_selected(&mut self, selected: &[bool; 5]) -> bool {
        let mut failed = false;
        for index in 0..5 {
            if !selected[index] {
                continue;
            }
            let Some(started) = self.running[index].as_ref() else {
                continue;
            };
            let code = match started.children[0].exit_code_after_exit() {
                Ok(code) => code,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => continue,
                Err(error) => {
                    self.issues[index] = Some(error.to_string());
                    failed = true;
                    continue;
                }
            };
            if code != 0 {
                self.issues[index] = Some(format!("Minecraft exited with code {code}"));
            }
            let journal_saved = match started.save_after_exit(1) {
                Ok(_) => true,
                Err(error) => {
                    self.issues[index] = Some(format!("Could not save startup journal: {error}"));
                    false
                }
            };
            let started = self.running[index].take().unwrap();
            let [mut child] = started.children;
            self.stores[index] = child.storage.take();
            self.pending_storage_maintenance[index] = code == 0 && journal_saved && self.stores[index].is_some();
            self.windows[index] = None;
            #[cfg(feature="mc02-native-input")]
            self.ai_leases.forget_slot(index as u32+1);
        }
        failed
    }

    fn launch(&mut self, slot: u32) -> io::Result<()> {
        self.launch_inner(slot,#[cfg(feature="mc02-native-input")] None)
    }
    #[cfg(feature="mc02-native-input")]
    fn launch_mc02_experiment(&mut self, slot:u32)->io::Result<()> {
        if slot!=2 {return Err(invalid("Legacy MC02 experiment is slot 2 only"));}
        self.launch_ai_experiment(slot,AgentControlMode::HumanPlusFourAi)
    }
    #[cfg(feature="mc02-native-input")]
    fn launch_ai_experiment(&mut self,slot:u32,mode:AgentControlMode)->io::Result<()> {
        self.launch_inner(slot,Some(mode))
    }
    fn launch_inner(&mut self,slot:u32,
        #[cfg(feature="mc02-native-input")] experimental_input:Option<AgentControlMode>,
    )->io::Result<()> {
        if self.exiting {return Err(invalid("Minecraft sessions are closing. Finish closing them before starting another game."));}
        if !(1..=5).contains(&slot) {
            return Err(invalid("Unknown Minecraft profile"));
        }
        let index = slot as usize - 1;
        self.reap();
        // A new launch supersedes optional cleanup from the prior exit.
        self.pending_storage_maintenance[index] = false;
        #[cfg(feature="mc02-native-input")]
        if let Some(mode)=experimental_input {
            // A single controller must not mix hybrid and full-AI channels.
            for running in self.running.iter().flatten() {
                if let Some(channel)=running.children[0].mc02_input.as_ref() {
                    if channel.mode()!=mode {
                        return Err(invalid("Cannot mix hybrid and five-AI input modes"));
                    }
                }
            }
            if mode==AgentControlMode::HumanPlusFourAi {
                if slot==1 {return Err(invalid("MC01 belongs to the human"));}
                let host=self.running[0].as_ref()
                    .ok_or_else(||invalid("Hybrid AI launch requires live MC01"))?;
                let child=&host.children[0];
                child.identity()?;
                if child.mc02_input.is_some() ||
                   !window(child,false).is_some_and(|target|target.responding) {
                    return Err(invalid("Hybrid requires a responding human-owned MC01"));
                }
            }
        }
        if self.running[index].is_some() {
            return Err(invalid(
                "Minecraft profile is already running or its outcome is uncertain",
            ));
        }
        #[cfg(feature="lazy-tester-package")]
        if product_profiles::clear::pending(self.store(slot)?)? {
            return Err(invalid("Finish clearing this launcher's old data in Data Storage before starting it"));
        }
        let payload = BootstrapLibrary::reviewed()?;
        // Freeze the validated launch preference before any profile mutation.
        // The same value reaches the original presentation ABI.
        let limits = crate::engine::manager_settings::load_performance()?;
        // Every launch still resolves signed registration, pins the current
        // image and checks the updater. Reuse the exact recipe only while
        // original controller-owned child pins continuously protect its bytes.
        let admission = self.running.iter().flatten().map(|started| {
            let child = &started.children[0];
            (&child._image, child.storage.as_ref().map_or(0, |store| store.singleton_caller_rva))
        });
        let (path, image, singleton_caller_rva) = match registered_image::registered_image_for_generation(true, admission) {
            Ok(target) => target,
            Err(error) => { self.issues[index] = Some(error.to_string()); return Err(error); }
        };
        // A positively reviewed replacement resolves only stale compatibility
        // issues, not unrelated profile errors or uncertain live processes.
        for other in 0..self.issues.len() {
            if self.running[other].is_none() && self.stores[other].is_some()
                && self.issues[other].as_deref() == Some(registered_image::UNREVIEWED_IMAGE_MESSAGE)
            {
                self.issues[other] = None;
            }
        }
        self.path = path.clone();
        let store = self.stores[index]
            .as_mut()
            .ok_or_else(|| invalid("Profile is unavailable after unconfirmed startup cleanup; existing processes are preserved"))?;
        store.singleton_caller_rva = singleton_caller_rva;
        game_preset::apply_inactive(std::slice::from_ref(store))?;
        // Keep the exact validated store until ownership reaches the started
        // transaction. Cloning handles neither reopens paths nor seeds a profile.
        let cleanup = UnstartedCleanup::new();
        let attempt = (|| {
            let child = create_suspended_process(
                Path::new(&path),
                image.try_clone()?,
                Some(self.stores[index].as_ref().unwrap().duplicate()?),
                true,
                Some((&payload, limits)),
                Some(cleanup.clone()),
                #[cfg(feature="mc02-native-input")]
                experimental_input,
                #[cfg(test)]
                None,
            )?;
            payload.enroll(&child)?;
            PreparedEducationEnrollment {
                _children: [child],
                _payload: payload,
            }
            .resume_for_authorized_observation()
        })();
        let started = match attempt {
            Ok(started) => started,
            Err(error) => {
                let issue = if cleanup.retry_safe() {
                    error.to_string()
                } else {
                    // Never restore retry permission merely because an error
                    // returned: cancellation must confirm this child's exit.
                    self.stores[index] = None;
                    format!("Startup cleanup is unconfirmed; retry disabled and existing processes preserved: {error}")
                };
                self.issues[index] = Some(issue.clone());
                return Err(io::Error::other(issue));
            }
        };
        self.stores[index] = None; // Started transaction now owns the duplicate pins.
        self.issues[index] = match &started.attempts[0] {
            StartupAttempt::Resumed => None,
            _ => Some("Resume outcome is uncertain; game preserved without retry".into()),
        };
        self.running[index] = Some(started);
        Ok(())
    }

    fn snapshot(&mut self) -> SessionSnapshot {
        self.reap();
        self.snapshot_without_reaping(true)
    }

    // During a coordinated normal CloseAll/CloseOne, avoid unrelated journal
    // persistence both before the close request and in its response snapshot.
    // Read-only status receipts should not write game-window titles for
    // all five profiles. Keep the SAME process/HWND verification for those
    // roster observations. Private input ACKs need no roster at all.
    fn snapshot_without_reaping(&mut self, relabel: bool) -> SessionSnapshot {
        self.snapshot_slots(relabel, 0..5)
    }

    // Selected desktop receipts and private agent target observations reuse
    // the same profile observation logic without relabeling unrelated windows.
    fn snapshot_slots(&mut self, relabel: bool, slots: std::ops::Range<usize>) -> SessionSnapshot {
        let profiles = slots
            .map(|index| {
                let process = self.running[index].as_ref().and_then(|started| {
                    let child = &started.children[0];
                    child.identity().ok()?;
                    if let Some(previous) = &self.windows[index] {
                        if let Some(hwnd) = verified_retained_hwnd(previous) {
                            // The game can reset its caption after a desktop/workspace transition.
                            // Do not mutate game or dialog titles once full exit begins.
                            if relabel && !self.exiting {
                                let _ = window_caption::label_slot_window(hwnd as _, child.pid, index as u32 + 1);
                            }
                            let mut process = previous.clone();
                            process.responding = unsafe {
                                windows_sys::Win32::UI::WindowsAndMessaging::IsHungAppWindow(hwnd)
                            } == 0;
                            return Some(process);
                        }
                    }
                    window(child, relabel && !self.exiting)
                });
                self.windows[index] = process.clone();
                let mut issue=self.issues[index].clone();
                #[cfg(feature="lazy-tester-package")]
                if self.running[index].is_none() {
                    if let Some(store)=&self.stores[index] {
                        match product_profiles::clear::pending(store) {
                            Ok(false)=>{},
                            Ok(true)=>issue=Some("Full data clear is pending. Open Data Storage to finish this launcher; other launchers are unchanged.".into()),
                            Err(error)=>issue=Some(format!("Launcher cleanup needs attention: {error}")),
                        }
                    }
                }
                let state = if self.running[index].is_some() {
                    match process.as_ref() {
                        Some(p) if p.responding => "RUNNING",
                        Some(_) => "NOT_RESPONDING",
                        // This owned process is still alive but has no usable game HWND.
                        // NOT_RESPONDING prevents an unsafe second Launch in the UI;
                        // ATTENTION without a process is reserved for stopped profiles.
                        None if issue.is_some() => "NOT_RESPONDING",
                        None => "STARTING",
                    }
                } else if issue.is_some() || self.stores[index].is_none() {
                    "ATTENTION"
                } else {
                    "STOPPED"
                };
                ProfileSession {
                    slot: index as u32 + 1,
                    state: state.into(),
                    process,
                    issue,
                }
            })
            .collect();
        SessionSnapshot {
            schema: 1,
            profiles,
            storage: None,
            selective_inventory: None,
            selective_review: None,
            settings: None,
            agent_activity: None,
            testing_run: None,
            testing_inspection: None,
        }
    }

    fn close_all(&mut self) -> io::Result<()> { self.close_sessions(None) }
    fn close_one(&mut self, slot:u32) -> io::Result<()> { self.close_sessions(Some(slot)) }
    fn close_sessions(&mut self,slot:Option<u32>)->io::Result<()> {
        let mask=normal_close_mask(slot)?;
        // A selected close must be authorized by THIS retained game's
        // controller, not by an earlier general Inspect response. Refuse
        // idle/stale slots before any FPS policy, dialog or lifecycle action.
        // normal_close_mask already validates the range 1..=5.
        if let Some(requested) = slot {
            if self.running[requested as usize - 1].is_none() {
                return Err(invalid("Minecraft session is not running"));
            }
        }
        // Acquire all selected running slots before any close side effect.
        // On contention already acquired leases drop without closing a game.
        let _capture_gates=mask.iter().enumerate()
            .filter(|(index,selected)|**selected && self.running[*index].is_some())
            .map(|(index,_)|crate::engine::native_clients_session::acquire_recording_close_gate(index as u32+1))
            .collect::<io::Result<Vec<_>>>()?;
        self.exiting=true;
        let result=self.close_selected_sessions(mask);
        // Full exit preserves its terminal gate; selected exit never blocks
        // later launcher operations after its request completes.
        if slot.is_some() || result.is_err() {self.exiting=false;}
        result
    }
    fn selected_processes_exited(&self,mask:&[bool;5])->io::Result<bool> {
        for index in 0..5 {
            if !mask[index] { continue; }
            let Some(started)=self.running[index].as_ref() else { continue; };
            match started.children[0].exit_code_after_exit() {
                Ok(_) => {},
                Err(error) if error.kind()==io::ErrorKind::WouldBlock => return Ok(false),
                Err(error) => return Err(error),
            }
        }
        Ok(true)
    }

    fn close_selected_sessions(&mut self,mask:[bool;5])->io::Result<()> {
        // No journal write or sync_all until every targeted game process
        // has exited. MC01 disk persistence must not stall MC02's native
        // prompt confirmation on this single-threaded session actor.
        let snapshot=self.snapshot_without_reaping(true);
        let mut issues=Vec::new();
        let mut progress: [NormalCloseProgress; 5] = std::array::from_fn(|_| NormalCloseProgress::default());
        for profile in snapshot.profiles {
            let index=profile.slot as usize-1;
            if !mask[index] {continue;}
            let Some(started)=&self.running[index] else {continue;};
            let child=&started.children[0];
            let outcome=(||{
                child.identity()?;
                // Stop only this retained game's FPS pacing BEFORE WM_CLOSE or
                // dialog acknowledgement. Otherwise the controller would wait
                // for process exit while PolicyChannel::Drop (which signals
                // STOP) could only run *after* that same process exit.
                if let Some(policy)=child.policy.as_ref() {
                    policy.begin_normal_close()?;
                }
                // A posted dialog command is NOT an exit acknowledgement. Always
                // retain the game process until its original HANDLE is signaled.
                if progress[index].submit_exit_dialog(child)? { return Ok(()); }
                // A game may hide its main OGLES HWND before exiting. The
                // ordinary Play selector is intentionally visible-only; this
                // close-only fallback may still reach one verified hidden
                // original game HWND, never a native dialog or foreign PID.
                let Some(target)=profile.process.or_else(||hidden_window_for_close(child)) else {return Ok(());};
                if target.pid!=child.pid || target.created_file_time!=child.created.to_string() {
                    return Err(invalid("Game identity changed"));
                }
                progress[index].request_window(&target)
            })();
            if let Err(error)=outcome {
                // A normal exit may race the identity/window checks. Other games
                // still receive their close requests before we report the error.
                if child.exit_code_after_exit().is_err() {
                    issues.push(format!("Minecraft - {}: {error}",profile.slot));
                }
            }
        }
        let deadline=std::time::Instant::now()+Duration::from_secs(20);
        loop {
            // Query only original retained process handles while any target
            // remains alive. Reaping would invoke blocking disk sync_all for
            // an already-exited peer, starving the still-live game's dialog.
            if self.selected_processes_exited(&mask)? {
                // Persist only the selected, confirmed exited processes.
                // A single CloseOne must not wait for another slot's sync_all.
                self.reap_selected(&mask);
                if (0..5).all(|index|!mask[index]||self.running[index].is_none()) {
                    return Ok(());
                }
                return Err(io::Error::other("Selected Minecraft process exit was observed, but its startup journal cleanup is still pending"));
            }
            if !issues.is_empty(){return Err(io::Error::other(issues.join("; ")));}
            for index in 0..5 {
                if !mask[index] {continue;}
                let Some(started)=&self.running[index] else {continue;};
                let child=&started.children[0];
                let result=progress[index].submit_exit_dialog(child).and_then(|submitted| {
                    // Once a dialog is posted, continue observing it. Do not
                    // re-post the same IDOK or assume Minecraft processed it.
                    if !submitted {
                        if let Some(target)=window(child, false).or_else(||hidden_window_for_close(child)) {
                            progress[index].request_window(&target)?;
                        }
                    }
                    Ok(())
                });
                if let Err(error)=result {
                    if child.exit_code_after_exit().is_err() {
                        return Err(error);
                    }
                }
            }
            if std::time::Instant::now()>=deadline {
                let pending = (0..5).filter_map(|index| {
                    if !mask[index] {return None;}
                    let started = self.running[index].as_ref()?;
                    let child=&started.children[0];
                    // The snapshot may still retain an exited child until
                    // all selected processes have exited. Do not accuse
                    // an already-stopped MC01 of delaying MC02 shutdown.
                    if child.exit_code_after_exit().is_ok() {return None;}
                    let evidence = progress[index].describe_pending(child);
                    self.issues[index] = Some(format!("Normal exit pending: {evidence}"));
                    Some(format!("Minecraft - {}: {evidence}", index + 1))
                }).collect::<Vec<_>>();
                return Err(io::Error::new(io::ErrorKind::TimedOut,format!(
                    "Normal Minecraft exit is still pending. {}. Finish a remaining game prompt normally and retry; no game was force-terminated.",
                    pending.join("; ")
                )));
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    fn store(&self, slot: u32) -> io::Result<&SlotStorage> {
        let index = slot
            .checked_sub(1)
            .filter(|index| *index < 5)
            .ok_or_else(|| invalid("Unknown profile"))? as usize;
        self.stores[index]
            .as_ref()
            .or_else(|| {
                self.running[index]
                    .as_ref()
                    .and_then(|started| started.children[0].storage.as_ref())
            })
            .ok_or_else(|| invalid("Profile storage is unavailable"))
    }

    // A display hint may reuse the previously verified image path; mutations
    // must use fresh native observation. Both retain the same idle predicate.
    // Read the current actor roster; the caller owns lifecycle observation.
    fn settings_report(&self, inspect: impl FnOnce(&Self) -> io::Result<bool>) -> io::Result<SettingsReport> {
        let idle = self.running.iter().all(Option::is_none)
            && self.stores.iter().all(Option::is_some)
            && inspect(self)?;
        SettingsReport::policy(idle, None)
    }

    fn settings_observation(&self) -> io::Result<SettingsReport> {
        self.settings_report(|sessions| Ok(
            super::super::inspect_native_workspace_for_pinned_settings(&sessions.path)?
                .processes.is_empty()))
    }

    // Caller must first observe/reap the current roster. Unlike the
    // read-only pinned-image hint, this still resolves fresh registration.
    fn fresh_settings_report(&self) -> io::Result<SettingsReport> {
        self.settings_report(|_| Ok(super::super::inspect_native_workspace()?.processes.is_empty()))
    }

    fn apply_preset(&mut self) -> io::Result<SettingsReport> {
        self.reap(); // Separate pre-mutation admission, not a reply observation.
        let mut report = self.fresh_settings_report()?;
        require_preset_idle(report.can_apply_preset)?;
        // Retain original directory/registry pins; never seed replacement profiles.
        let stores = self.stores.iter().map(|store| store.as_ref().unwrap().duplicate())
            .collect::<io::Result<Vec<_>>>()?;
        report.changed_profiles = Some(game_preset::apply(&stores)? as u32);
        Ok(report)
    }


}

fn require_preset_idle(idle: bool) -> io::Result<()> {
    if !idle { return Err(invalid("Close all Minecraft sessions normally before applying the graphics preset")); }
    Ok(())
}

fn normal_close_mask(slot:Option<u32>)->io::Result<[bool;5]> {
    if slot.is_some_and(|value|!(1..=5).contains(&value)) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile"));
    }
    Ok(std::array::from_fn(|index|slot.is_none_or(|value|value==index as u32+1)))
}

#[path = "session_controller/profile_storage_operations.rs"]
mod profile_storage_operations;
use self::profile_storage_operations::mutation_storage_slots;
#[cfg(test)]
use self::profile_storage_operations::{all_observed_processes_owned, report_data_admission, require_data_idle};

#[path = "session_controller/owned_game_windows.rs"]
mod owned_game_windows;
use self::owned_game_windows::{verified_retained_hwnd, hidden_window_for_close, window, NormalCloseProgress};
#[cfg(test)]
use self::owned_game_windows::{game_window_class, valid_retained_window, RequestedWindow, close_verified_window, confirm_education_exit, confirm_exit_dialog, confirm_exit_dialog_once, hidden_close_candidate};

#[path = "session_controller/audio_maintenance.rs"]
mod audio_maintenance;
use self::audio_maintenance::{audio_roster, audio_maintenance_needed, sync_audio, AudioIdleGate};
#[cfg(test)]
use self::audio_maintenance::{AudioRoster, AUDIO_IDLE_RESYNC};

#[path = "session_controller/reply_snapshot.rs"]
mod reply_snapshot;
use self::reply_snapshot::ReplySnapshot;

// Keep the bounded cadence for real maintenance, peer reconciliation,
// and degraded native waits.
fn maintenance_interval(pending_storage: bool, audio_needed: bool, exit_retry: bool) -> Duration {
    Duration::from_millis(if pending_storage || audio_needed || exit_retry {250} else {1000})
}

// Only the healthy native wait may omit the periodic maintenance wake.
// Original game handles, request event and pipe-worker exit still wake it;
// the pipe converts this budget to the maximum finite Win32 wait.
fn native_wait_budget(running: bool, pending_storage: bool, audio_needed: bool, cadence: Duration) -> Duration {
    if running && !pending_storage && !audio_needed { Duration::MAX } else { cadence }
}

fn run() -> io::Result<()> {
    crate::engine::workspace_surface::verify_same_executable_parent()?;
    crate::engine::dpi::configure()?;
    // Same gate as historical pair/five CLI: no parallel lifecycle authority.
    let _gate = acquire_startup_gate(
        crate::engine::native_clients_session::CONTROLLER_GATE_NAME,
        invalid("Another Minecraft controller is still active; existing games are preserved"),
    )?;
    let mut sessions = Sessions::prepare()?;
    // Only the original actor keeps one ephemeral UI report mirror.
    // This does not own Probe progress, games, input, or agent processes.
    let mut agent_activity:Option<(crate::engine::native_clients_session::AgentActivityReport,Instant)>=None;
    // Singleton native actor arbitrates run identity across independent
    // MCP stdio clients. This does not run tests or own Probe evidence.
    let mut testing_runs=crate::engine::native_clients_session::NativeTestingRunBook::default();
    let mut audio=None;
    let mut audio_mode=crate::engine::manager_settings::load_audio()?;
    // Audio keeps its existing bounded maintenance; game exit has its own
    // retained-handle wake and does not need periodic process scanning.
    let mut audio_idle = AudioIdleGate::default();
    let mut audio_focus = None;
    let mut audio_focus_failed = false;
    // No audio failure observed before the first idle maintenance cycle.
    let mut audio_issue: Option<String> = None;
    let server = Server::start(Endpoint::current()?)?;
    let mut manager: Option<Peer> = None;
    let mut exit_retry = false;
    let mut native_wait_failed = false;
    let startup = std::time::Instant::now();
    {
        let mut output = io::stdout().lock();
        serde_json::to_writer(&mut output, &Result::<(), String>::Ok(()))?;
        output.write_all(b"\n")?;
        output.flush()?;
    }
    loop {
        // A queued CloseOne/CloseAll should not wait behind journal sync_all
        // or COM audio discovery for an unrelated, already-exited game.
        // Native I/O is still unpreemptible once idle maintenance begins.
        let running = sessions.running.iter().any(Option::is_some);
        let pending_storage = sessions.pending_storage_maintenance.iter().any(|pending|*pending);
        let roster = audio_roster(&sessions);
        let audio_needed = audio_maintenance_needed(audio_mode, audio.is_some(), roster);
        if !audio_needed {
            audio_focus = None;
            audio_focus_failed = false;
        } else if audio_focus.is_none() && !audio_focus_failed && !native_wait_failed {
            match instance_focus_events::AudioFocusWake::subscribe() {
                Ok(wake) => audio_focus=Some(wake),
                Err(error) => {
                    eprintln!("Audio focus hint unavailable: {error}");
                    audio_focus_failed=true;
                }
            }
        }
        // Focus hints are an acceleration only for healthy audio. A burst
        // must not bypass the bounded retry or pending storage cadence.
        let focus_ready = !native_wait_failed && !exit_retry && !pending_storage &&
            audio_idle.current_success(roster) &&
            audio_focus.as_ref().is_some_and(|wake|wake.observer_active()) &&
            sessions.running.iter().flatten().all(|started|started.children[0].policy.is_some());
        let cadence = maintenance_interval(pending_storage, audio_needed && !focus_ready,
            exit_retry || native_wait_failed && running);
        let cadence = if audio_needed {
            let now=Instant::now();
            let discovery=audio.as_ref().map(|a|a.discovery_budget(now));
            cadence.min(audio_idle.wait_budget(now,roster,focus_ready,discovery))
        } else {cadence};
        // Direct wait on the original actor's handles. A dead manager cannot
        // repeatedly wake a controller that still owns games; it is relevant
        // to detach only when no game remains.
        let activity = if !native_wait_failed && !exit_retry {
            let lifetimes = sessions.running.iter().flatten()
                .map(|started| started.children[0].process.as_handle())
                .chain(manager.as_ref().filter(|_| !running).map(Peer::wait_handle));
            let hint = audio_focus.as_ref().filter(|_|focus_ready).map(|wake|wake.handle());
            let budget = native_wait_budget(running, pending_storage, audio_needed, cadence);
            match server.wait_for_activity(lifetimes, hint, budget) {
                Ok(activity) => Some(activity),
                Err(error) => {
                    // Actual native wait failure: preserve ownership and the
                    // original bounded maintenance for this controller lifetime.
                    eprintln!("Session lifetime wait unavailable: {error}");
                    native_wait_failed = true;
                    audio_focus = None;
                    None
                }
            }
        } else { None };
        let activity = activity.unwrap_or_else(|| {
            let cadence=if audio_needed || running {cadence.min(Duration::from_millis(250))} else {cadence};
            match server.requests.recv_timeout(cadence) {
                Ok(envelope) => Activity::Request(envelope),
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Activity::Lifetime,
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Activity::Disconnected,
            }
        });
        let envelope = match activity {
            // A new authenticated peer wins over the previous peer's detach
            // indication. Its own liveness is still checked before admission.
            Activity::Request(envelope) => envelope,
            idle => {
                let disconnected = matches!(idle, Activity::Disconnected);
                if matches!(idle, Activity::Lifetime | Activity::Disconnected) {
                    exit_retry = sessions.reap();
                }
                // A successful ordinary timeout performs no per-game exit
                // scan. Storage and audio retain their original admission.
                sessions.maintain_storage();
                let roster = audio_roster(&sessions);
                if !audio_maintenance_needed(audio_mode, audio.is_some(), roster) {
                    audio_issue = None;
                    audio_idle.invalidate();
                } else {
                    // Clear before fresh observation. A focus change during
                    // COM work remains pending; a queued request cannot consume it.
                    if let Some(wake)=audio_focus.as_ref() {
                        if let Err(error)=wake.clear() {
                            eprintln!("Audio focus hint reset unavailable: {error}");
                            audio_focus_failed=true;
                            audio_focus=None;
                        }
                    }
                    let foreground = super::session_audio::foreground();
                    let now = Instant::now();
                    let discovery_due=audio_idle.current_success(roster) &&
                        audio.as_ref().is_some_and(|a|a.discovery_budget(now).is_zero());
                    if discovery_due || audio_idle.due(now, foreground, roster) {
                        match sync_audio(&sessions, &mut audio, audio_mode, foreground) {
                            Ok(()) => {
                                audio_issue = None;
                                audio_idle.success(Instant::now(), foreground, roster);
                            }
                            Err(error) => {
                                audio_issue = Some(error.to_string());
                                audio_idle.invalidate();
                            }
                        }
                    }
                }
                if sessions.running.iter().all(Option::is_none) {
                    let detached = manager.as_ref().map_or_else(
                        || startup.elapsed() > Duration::from_secs(45),
                        |peer| !peer.alive(),
                    );
                    if detached { return Ok(()); }
                    if disconnected { return Err(invalid("Session transport stopped")); }
                }
                // A disconnected channel is permanently ready. Preserve the
                // original bounded delay rather than spin or abandon games.
                if disconnected { std::thread::sleep(Duration::from_millis(250)); }
                continue;
            }
        };
        if !envelope.peer.alive() {
            let _ = envelope
                .reply
                .send(Err("Manager exited before request admission".into()));
            continue;
        }
        manager = Some(envelope.peer);
        let reply = (|| {
            let mut storage = None;
            let mut selective_inventory = None;
            let mut selective_review = None;
            let mut settings = None;
            let reply_snapshot = ReplySnapshot::for_request(&envelope.request);
            let inspect_settings = matches!(&envelope.request, SessionRequest::InspectSettings);
            let saved_settings = matches!(&envelope.request,
                SessionRequest::SavePerformance { .. } |
                SessionRequest::SaveStoragePreferences { .. } |
                SessionRequest::SaveAudio { .. });
            let mut testing_reply=None;
            let mut inspection_reply=None;
            match envelope.request {
                SessionRequest::Inspect | SessionRequest::InspectSettings => {}
                SessionRequest::InspectAgentTarget {slot} => {
                    let selected=normal_close_mask(Some(slot)).map_err(|error|error.to_string())?;
                    sessions.reap_selected(&selected);
                }
                SessionRequest::TestingInspect {slot,owner_id} => {
                    inspection_reply=Some(testing_runs.inspect(slot,&owner_id)
                        .map_err(|error|error.to_string())?);
                }
                SessionRequest::TestingReserve {
                    slot,owner_id,run_id,replace_run_id
                } => {
                    testing_reply=Some(testing_runs.reserve_from_peer(
                        slot,&owner_id,&run_id,replace_run_id.as_deref(),
                        manager.as_ref().ok_or_else(||"Missing verified Testing peer".to_string())?
                    ).map_err(|e|e.to_string())?);
                }
                SessionRequest::TestingFinalize {
                    slot,owner_id,run_id,reported_status,report_digest
                } => {
                    testing_reply=Some(testing_runs.finalize_from_peer(
                        slot,&owner_id,&run_id,&reported_status,&report_digest,
                        manager.as_ref().ok_or_else(||"Missing verified Testing peer".to_string())?
                    ).map_err(|e|e.to_string())?);
                }
                SessionRequest::PublishAgentActivity {activity} => {
                    if !activity.valid_ingress() {
                        return Err("Invalid agent activity report".into());
                    }
                    agent_activity=Some((activity,Instant::now()));
                }
                SessionRequest::ApplyPreset => {
                    settings = Some(sessions.apply_preset().map_err(|error| error.to_string())?);
                }
                SessionRequest::ApplyGraphicsPreset { .. } => {
                    return Err("Individual graphics preferences are retired; all launchers use the optimized preset.".into());
                }
                SessionRequest::SavePerformance {active_fps,background_fps} => {
                    crate::engine::manager_settings::save_performance(crate::engine::manager_settings::PerformanceSettings {active_fps,background_fps})
                        .map_err(|error|error.to_string())?;
                }
                SessionRequest::SaveStoragePreferences {preferences}=>{
                    crate::engine::manager_settings::save_storage_preferences(preferences).map_err(|e|e.to_string())?;
                    if !preferences.has_maintenance() {sessions.pending_storage_maintenance=[false;5];}
                }
                SessionRequest::SaveAudio {mode}=>{
                    // Explicit user audio changes remain immediate and are
                    // never delayed by the idle cadence. On any failure the
                    // old saved mode is retried at the next original tick.
                    audio_idle.invalidate();
                    let foreground = super::session_audio::foreground();
                    sync_audio(&sessions,&mut audio,mode,foreground).map_err(|e|e.to_string())?;
                    crate::engine::manager_settings::save_audio(mode).map_err(|e|e.to_string())?;
                    audio_mode=mode;audio_issue=None;
                    audio_idle.success(Instant::now(),foreground,audio_roster(&sessions));
                }
                SessionRequest::Launch { slot } => {
                    sessions.launch(slot).map_err(|error| error.to_string())?
                }
                #[cfg(feature="mc02-native-input")]
                SessionRequest::LaunchMc02Experiment {slot} => {
                    sessions.launch_mc02_experiment(slot).map_err(|error|error.to_string())?
                }
                #[cfg(feature="mc02-native-input")]
                SessionRequest::LaunchAiExperiment {slot,mode} => {
                    sessions.launch_ai_experiment(slot,mode).map_err(|error|error.to_string())?
                }
                #[cfg(feature="mc02-native-input")]
                SessionRequest::Mc02Input {
                    slot,keys,dx,dy,ttl_ms,issued_at_ms,batch_origin,
                } => {
                    // Reuse SAME verified named-pipe peer already pinned
                    // by Endpoint::verify; no user-provided owner token,
                    // extra per-pulse OS lookup or second controller.
                    let source=manager.as_ref()
                        .ok_or_else(||"Original MC02 CLI peer unavailable".to_string())?
                        .input_sender();
                    sessions.legacy_mc02_input(slot,keys,dx,dy,ttl_ms,
                        issued_at_ms,batch_origin,source)
                        .map_err(|error|error.to_string())?
                }
                #[cfg(feature="mc02-native-input")]
                SessionRequest::AiLeasedInput {slot,keys,dx,dy,ttl_ms,
                    issued_at_ms,owner_id,lease_epoch} => {
                    sessions.ai_leased_input(slot,keys,dx,dy,ttl_ms,
                        issued_at_ms,&owner_id,&lease_epoch,None)
                        .map_err(|error|error.to_string())?
                }
                #[cfg(feature="mc02-native-input")]
                SessionRequest::AiLeaseAcquire {
                    slots,owner_id,mode,lease_epoch,neutral_only,
                } => sessions.ai_lease_acquire(
                    &slots,&owner_id,mode,&lease_epoch,neutral_only)
                        .map_err(|error|error.to_string())?,
                #[cfg(feature="mc02-native-input")]
                SessionRequest::AiLeaseRelease {
                    slots,owner_id,lease_epoch,
                } => sessions.ai_lease_release(&slots,&owner_id,&lease_epoch)
                        .map_err(|error|error.to_string())?,
                #[cfg(feature="mc02-native-input")]
                SessionRequest::AiInputGroup {
                    frames,issued_at_ms,deadline_at_ms,owner_id,lease_epoch,
                } => {
                    sessions.ai_input_group(&frames,issued_at_ms,deadline_at_ms,
                        &owner_id,&lease_epoch)
                        .map_err(|error|error.to_string())?
                }
                SessionRequest::CloseAll => sessions.close_all().map_err(|error|error.to_string())?,
                SessionRequest::CloseOne {slot} => sessions.close_one(slot).map_err(|error|error.to_string())?,
                SessionRequest::InspectStorage => {
                    storage = Some(sessions.inspect_storage(1..=5).map_err(|error| error.to_string())?)
                }
                SessionRequest::InspectProfileStorage { slot } => {
                    storage = Some(sessions.inspect_storage(slot..=slot).map_err(|error| error.to_string())?)
                }
                SessionRequest::InspectSelectiveInventory { slot, category, .. } => {
                    selective_inventory = Some(sessions.selective_inventory(slot, category).map_err(|error| error.to_string())?);
                }
                SessionRequest::ReviewSelectiveInventory { slot, selections, .. } => {
                    selective_review = Some(sessions.review_selective_inventory(slot, selections).map_err(|error| error.to_string())?);
                }
                SessionRequest::ConfirmSelectedPacks { slot, selections, confirmation_token } => {
                    // Dependency/activation evidence and reference stability must
                    // be independently accepted before enabling this route.
                    if !SELECTED_PACK_DISPOSITION_REVIEWED {
                        return Err("Pack deletion is held for native dependency and filesystem validation".into());
                    }
                    sessions.confirm_selected_data(slot,&selections,&confirmation_token,true).map_err(|error|error.to_string())?;
                    storage=Some(sessions.inspect_storage(1..=5).map_err(|error|error.to_string())?);
                }
                SessionRequest::ConfirmSelectedWorlds { slot, selections, confirmation_token } => {
                    // Source integration only: the existing native disposition hold remains.
                    if !SELECTED_WORLD_DISPOSITION_REVIEWED {
                        return Err("World deletion is held for native filesystem validation".into());
                    }
                    sessions.confirm_selected_data(slot, &selections, &confirmation_token, false).map_err(|error| error.to_string())?;
                    storage = Some(sessions.inspect_storage(1..=5).map_err(|error| error.to_string())?);
                }
                SessionRequest::ClearProfileDownloadCache { slot } => {
                    if !crate::engine::manager_settings::DOWNLOAD_CACHE_REVIEWED {
                        return Err("Download cache location is not native-verified".into());
                    }
                    sessions.clear_cache_target(slot, profile_storage::CacheTarget::Download).map_err(|error|error.to_string())?;
                    storage=Some(sessions.inspect_storage(1..=5).map_err(|error|error.to_string())?);
                }
                SessionRequest::ClearProfileCache { slot, selected_storage } => {
                    sessions
                        .clear_cache_target(slot, profile_storage::CacheTarget::Web)
                        .map_err(|error| error.to_string())?;
                    storage = Some(sessions.inspect_storage(mutation_storage_slots(slot, selected_storage)).map_err(|error| error.to_string())?);
                }
                SessionRequest::ResetGameSettings { slot, selected_storage } =>
                    storage = Some(sessions.change_game_settings(slot, game_preset::reset_settings, selected_storage).map_err(|error|error.to_string())?),
                SessionRequest::RestoreGameSettings { slot, selected_storage } =>
                    storage = Some(sessions.change_game_settings(slot, game_preset::restore_settings, selected_storage).map_err(|error|error.to_string())?),
                SessionRequest::DisableImprovedInputResponse { slot } =>
                    storage = Some(sessions.change_game_settings(slot, game_preset::disable_input_response, None).map_err(|error|error.to_string())?),
                SessionRequest::EnableTextureStreaming { slot } =>
                    storage = Some(sessions.change_game_settings(slot, game_preset::enable_texture_streaming, None).map_err(|error|error.to_string())?),
                SessionRequest::DisableTextureStreaming { slot } =>
                    storage = Some(sessions.change_game_settings(slot, game_preset::disable_texture_streaming, None).map_err(|error|error.to_string())?),
                SessionRequest::ClearProfileData {slot,confirmation_token,confirmation_name,selected_storage}=>{
                    if confirmation_name!=format!("Minecraft - {slot}") {return Err("Launcher confirmation does not match".into());}
                    sessions.clear_data(slot,&confirmation_token).map_err(|error|error.to_string())?;
                    storage=Some(sessions.inspect_storage(mutation_storage_slots(slot, selected_storage)).map_err(|error|error.to_string())?);
                }
                SessionRequest::OpenProfileFolder { slot, .. } => sessions
                    .open_folder(slot)
                    .map_err(|error| error.to_string())?,
            }
            // All admitted input mutations (including neutral cleanup) use
            // the original authenticated actor reply, without observing five
            // unrelated profile windows, touching storage or cloning activity.
            let mut snapshot = reply_snapshot.collect(&mut sessions);
            #[cfg(feature="mc02-native-input")]
            if reply_snapshot==ReplySnapshot::InputReceipt {
                return Ok(snapshot);
            }
            if matches!(reply_snapshot,ReplySnapshot::AgentTarget(_)) {
                return Ok(snapshot); // No unrelated status/activity payload.
            }
            if reply_snapshot==ReplySnapshot::AgentReceipt {
                // Testing correlation uses only its own result. Activity
                // publication echoes the admitted report, not a game roster
                // or a later UI freshness projection. This also keeps its
                // receipt distinct from the empty experimental input ACK.
                if testing_reply.is_none() && inspection_reply.is_none() {
                    snapshot.agent_activity=agent_activity.as_ref().map(|(report,_)|report.clone());
                }
                snapshot.testing_run=testing_reply;
                snapshot.testing_inspection=inspection_reply;
                return Ok(snapshot);
            }
            // Both Settings observation and post-save replies share this
            // full snapshot's reap. No actor request can change its roster
            // between these observations. Saves still use fresh registration;
            // ApplyPreset's independent pre-mutation admission is unchanged.
            // External process exit may still race any point-in-time report.
            if inspect_settings {
                settings = Some(sessions.settings_observation().map_err(|error| error.to_string())?);
            } else if saved_settings {
                settings = Some(sessions.fresh_settings_report().map_err(|error| error.to_string())?);
            }
            if let Some(report)=settings.as_mut(){report.audio_issue=audio_issue.clone();}
            snapshot.storage = storage;
            snapshot.selective_inventory = selective_inventory;
            snapshot.selective_review = selective_review;
            snapshot.settings = settings;
            snapshot.testing_run=testing_reply;
            snapshot.testing_inspection=inspection_reply;
            snapshot.agent_activity=agent_activity.as_ref().and_then(|(report,at)|{
                if at.elapsed()<=Duration::from_secs(12) {
                    Some(report.clone())
                } else if report.display_hint=="ATTENTION" {
                    // The old issue may remain: show loss of current
                    // reporting, not stale control or QA execution claims.
                    let mut stale=report.clone();
                    stale.stale=true;
                    Some(stale)
                } else {None}
            });
            Ok(snapshot)
        })();
        let _ = envelope.reply.send(reply); // A disconnected UI never cancels an admitted launch.
    }
}

pub(super) fn maybe_run() -> bool {
    if std::env::args_os().nth(1).as_deref() != Some(OsStr::new("--lazy-tester-controller")) {
        return false;
    }
    let result = if std::env::args_os().count() == 2 {
        run()
    } else {
        Err(invalid("Controller accepts no extra arguments"))
    };
    if let Err(error) = result {
        let reply: Result<SessionSnapshot, String> = Err(error.to_string());
        let mut output = io::stdout().lock();
        let _ = serde_json::to_writer(&mut output, &reply);
        let _ = output.write_all(b"\n");
        let _ = output.flush();
    }
    true
}

#[cfg(test)]
#[path = "session_controller/tests.rs"]
mod tests;

// Enable only after matching native directory sharing/disposition acceptance and explicit approval.
const SELECTED_WORLD_DISPOSITION_REVIEWED: bool = false;
const SELECTED_PACK_DISPOSITION_REVIEWED: bool = false;
