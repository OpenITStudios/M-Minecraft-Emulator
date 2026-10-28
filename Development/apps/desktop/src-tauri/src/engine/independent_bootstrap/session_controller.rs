//! One retained bootstrap owner. Full exit requests normal closure of its own games.
use super::*;
use crate::engine::{
    native_clients_session::{
        pipe::{Endpoint, Peer, Server},
        ProfileSession, ProfileStorage, SessionRequest, SessionSnapshot, SettingsReport,
    },
    process_identity::OwnedProcess,
};
use std::{io::Write, time::{Duration, Instant}};
use virtual_clients_core::InspectionBudget;

struct Sessions {
    stores: Vec<Option<SlotStorage>>,
    running: Vec<Option<StartedEducationObservation<1>>>,
    issues: Vec<Option<String>>,
    windows: Vec<Option<OwnedProcess>>,
    path: String,
    image: File,
    exiting: bool,
}
impl Sessions {
    fn prepare() -> io::Result<Self> {
        // Do not adopt arbitrary games or infer a profile from window titles.
        if !super::super::inspect_native_workspace()?
            .processes
            .is_empty()
        {
            return Err(io::Error::new(io::ErrorKind::AlreadyExists,
                "Existing Minecraft sessions are preserved. Close them normally before starting managed profiles."));
        }
        let (path, image) = registered_image(true)?;
        let stores = registered_slots::<5>(true)?.into_iter().map(Some).collect();
        // Validate the reviewed DLLs before the first settings write or game process.
        drop(BootstrapLibrary::reviewed()?);
        Ok(Self {
            stores,
            running: (0..5).map(|_| None).collect(),
            issues: vec![None; 5],
            windows: vec![None; 5],
            path,
            image,
            exiting: false,
        })
    }

    fn reap(&mut self) {
        self.reap_selected(&[true; 5]);
    }

    // Only the already-exited target of CloseOne needs journal persistence
    // before its acknowledgement. Other slots retain their original handles
    // and journals for the next ordinary idle/Inspect maintenance cycle.
    // CloseAll supplies all five slots and retains its full cleanup contract.
    fn reap_selected(&mut self, selected: &[bool; 5]) {
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
                    continue;
                }
            };
            if code != 0 {
                self.issues[index] = Some(format!("Minecraft exited with code {code}"));
            }
            if let Err(error) = started.save_after_exit(1) {
                self.issues[index] = Some(format!("Could not save startup journal: {error}"));
            }
            let started = self.running[index].take().unwrap();
            let [mut child] = started.children;
            self.stores[index] = child.storage.take();
            self.windows[index] = None;
        }
    }

    fn launch(&mut self, slot: u32) -> io::Result<()> {
        if self.exiting {return Err(invalid("Minecraft sessions are closing. Finish closing them before starting another game."));}
        if !(1..=5).contains(&slot) {
            return Err(invalid("Unknown Minecraft profile"));
        }
        let index = slot as usize - 1;
        self.reap();
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
        crate::engine::manager_settings::load_performance()?;
        // Recheck current registration/signature/path identity and exact bytes
        // against the original retained image without a redundant initial resolve.
        validate_registered_image(&self.path,&self.image,true)?;
        let store = self.stores[index]
            .as_mut()
            .ok_or_else(|| invalid("Profile is unavailable after unconfirmed startup cleanup; existing processes are preserved"))?;
        store.singleton_caller_rva = 0x2cdc30;
        game_preset::apply_inactive(std::slice::from_ref(store))?;
        // Keep the exact validated store until ownership reaches the started
        // transaction. Cloning handles neither reopens paths nor seeds a profile.
        let cleanup = UnstartedCleanup::new();
        let attempt = (|| {
            let child = create_unstarted_tracked(
                Path::new(&self.path),
                self.image.try_clone()?,
                Some(self.stores[index].as_ref().unwrap().duplicate()?),
                true,
                Some(&payload),
                Some(cleanup.clone()),
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
        self.snapshot_without_reaping()
    }

    // During a coordinated normal CloseAll/CloseOne, avoid unrelated journal
    // persistence both before the close request and in its response snapshot.
    // The ordinary Inspect route continues to reap/persist as before.
    fn snapshot_without_reaping(&mut self) -> SessionSnapshot {
        let profiles = (0..5)
            .map(|index| {
                let process = self.running[index].as_ref().and_then(|started| {
                    let child = &started.children[0];
                    child.identity().ok()?;
                    if let Some(previous) = &self.windows[index] {
                        if valid_retained_window(previous) {
                            // The game can reset its caption after a desktop/workspace transition.
                            let hwnd = usize::from_str_radix(&previous.window_handle[2..], 16).ok()?;
                            // Do not mutate game or dialog titles once full exit begins.
                            if !self.exiting {
                                let _ = label_slot_window(hwnd as _, child.pid, index as u32 + 1);
                            }
                            let mut process = previous.clone();
                            process.responding = unsafe {
                                windows_sys::Win32::UI::WindowsAndMessaging::IsHungAppWindow(
                                    usize::from_str_radix(&process.window_handle[2..], 16).ok()?
                                        as _,
                                )
                            } == 0;
                            return Some(process);
                        }
                    }
                    window(child, !self.exiting)
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
            settings: None,
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
        let snapshot=self.snapshot_without_reaping();
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

    fn storage(&self) -> io::Result<Vec<ProfileStorage>> {
        // One cooperative traversal budget for this *entire* read-only request,
        // not two seconds independently for each of five profiles and caches.
        // Native OS calls and the separate clear-confirmation inspection can
        // still block; this is not a hard latency guarantee or a new worker.
        let budget = InspectionBudget::new(Duration::from_secs(2));
        let observed = super::super::inspect_native_workspace()?.processes;
        (1..=5)
            .map(|slot| {
                let store = self.store(slot)?;
                let path = directory_path(&store.private)?;
                // Only read-only reports share this admission observation.
                // Cache deletion, profile clear, launch and settings writes each
                // recheck their own authority at the moment of mutation.
                let admission = self.data_admission(slot, &observed);
                let replay_admission = || {
                    admission.as_ref().map(|_| ()).map_err(|error| {
                        io::Error::new(error.kind(), error.to_string())
                    })
                };
                let mut result = ProfileStorage {
                    slot,
                    bytes: 0,
                    files: 0,
                    partial: false,
                    disk: profile_storage::disk(&store.private).ok(),
                    cache: {
                        let mut cache = profile_storage::inspect(&store.private, admission.is_ok(), budget);
                        if !cache.partial {
                            if let Err(error) = &admission { cache.issue = Some(error.to_string()); }
                        }
                        cache
                    },
                    game_settings: Some(game_preset::inspect_settings(&store.private, replay_admission())),
                    full_clear: Some(product_profiles::clear::inspect(store, replay_admission())),
                };
                scan(&path, &mut result, budget, 0);
                Ok(result)
            })
            .collect()
    }

    fn open_folder(&self, slot: u32) -> io::Result<()> {
        let store = self.store(slot)?;
        let path = directory_path(&store.private)?;
        // Keep the validated directory pinned while opening its canonical location.
        let pinned = open_directory(&path, FILE_READ_ATTRIBUTES)?;
        if !same_image(&directory_id(&pinned)?, &directory_id(&store.private)?) {
            return Err(invalid("Profile directory changed"));
        }
        let path = wide(path.as_os_str())?;
        let outcome = unsafe {
            windows_sys::Win32::UI::Shell::ShellExecuteW(
                ptr::null_mut(),
                windows_sys::w!("open"),
                path.as_ptr(),
                ptr::null(),
                ptr::null(),
                1,
            )
        };
        if outcome as isize <= 32 {
            return Err(io::Error::other(
                "Windows could not open the profile folder",
            ));
        }
        Ok(())
    }

    fn clear_cache(&mut self, slot: u32) -> io::Result<()> {
        self.reap();
        let observed = super::super::inspect_native_workspace()?.processes;
        self.data_admission(slot, &observed)?;
        // The sole session actor serializes admission, exact-directory deletion and launch.
        profile_storage::clear(&self.store(slot)?.private, true)
    }

    fn data_admission(&self, slot: u32, observed: &[OwnedProcess]) -> io::Result<()> {
        self.store(slot)?;
        if self.exiting { return Err(invalid("Minecraft sessions are closing; wait before changing launcher data")); }
        let owned = self.running.iter().flatten().map(|started| {
            let child = &started.children[0];
            child.identity()?;
            Ok((child.pid, child.created))
        }).collect::<io::Result<Vec<_>>>()?;
        require_data_idle(self.running[slot as usize - 1].is_none(), &owned, observed)
    }

    fn clear_data(&mut self,slot:u32,token:&str)->io::Result<()> {
        self.reap();
        let observed=super::super::inspect_native_workspace()?.processes;
        self.data_admission(slot,&observed)?;
        let index=slot as usize-1;
        let store=self.store(slot)?;
        if product_profiles::clear::confirmation(store)?!=token {
            return Err(invalid("Launcher data changed after confirmation; review it again"));
        }
        let logical=store.logical.try_clone()?;
        let root=product_profiles::clear::location(store)?;
        let store=self.stores[index].take().ok_or_else(||invalid("Inactive profile storage is unavailable"))?;
        let result=product_profiles::clear::run(store,token);
        match product_profiles::clear::reopen(logical,&root,slot) {
            Ok(store)=>self.stores[index]=Some(store),
            Err(error)=>{self.issues[index]=Some(format!("Full clear needs recovery: {error}"));return Err(error);}
        }
        let store=self.stores[index].as_ref().unwrap();
        let pending=product_profiles::clear::pending(store)?;
        if !pending && (result.is_ok() || product_profiles::clear::confirmation(store)?!=token) {
            self.issues[index]=None;self.windows[index]=None;
            return Ok(());
        }
        result.map_err(|error|io::Error::new(error.kind(),format!("Full clear did not finish. Refresh Data Storage to review its recovery status. {error}")))
    }

    // Read-only UI settings observation. The exact registered image path was
    // validated and its file HANDLE pinned at controller preparation; reusing
    // that path avoids launching the full PowerShell registration preflight
    // on every idle Settings refresh. It is a DISPLAY HINT, not permission
    // to alter profiles. settings() below keeps the original fresh check.
    fn settings_observation(&mut self) -> io::Result<SettingsReport> {
        self.reap();
        let idle = self.running.iter().all(Option::is_none)
            && self.stores.iter().all(Option::is_some)
            && super::super::inspect_native_workspace_for_pinned_settings(&self.path)?
                .processes.is_empty();
        SettingsReport::policy(idle, None)
    }

    fn settings(&mut self, apply: bool) -> io::Result<SettingsReport> {
        self.reap();
        let idle = self.running.iter().all(Option::is_none)
            && self.stores.iter().all(Option::is_some)
            && super::super::inspect_native_workspace()?.processes.is_empty();
        let mut report = SettingsReport::policy(idle, None)?;
        let changed = if apply {
            require_preset_idle(idle)?;
            // Retain the original directory/registry pins; never open or seed replacement profiles.
            let stores = self.stores.iter().map(|store| store.as_ref().unwrap().duplicate())
                .collect::<io::Result<Vec<_>>>()?;
            Some(game_preset::apply(&stores)? as u32)
        } else { None };
        report.changed_profiles = changed;
        Ok(report)
    }
}

fn require_data_idle(selected_idle: bool, owned: &[(u32,u64)], observed: &[OwnedProcess]) -> io::Result<()> {
    if !selected_idle { return Err(invalid("Close this Minecraft session normally before changing its launcher data.")); }
    if observed.iter().any(|process| !owned.iter().any(|(pid,birth)|
        process.pid == *pid && process.created_file_time.parse::<u64>().ok() == Some(*birth))) {
        return Err(invalid("An unmanaged Minecraft session is open. Close it normally before changing launcher data; managed sessions in other launchers may stay open."));
    }
    Ok(())
}

fn require_preset_idle(idle: bool) -> io::Result<()> {
    if !idle { return Err(invalid("Close all Minecraft sessions normally before applying the graphics preset")); }
    Ok(())
}

// Metadata only, bounded, and no traversal through junctions/symlinks. Not a cache classifier.
fn scan(path: &Path, result: &mut ProfileStorage, budget: InspectionBudget, depth: usize) {
    if depth > 48 || result.files >= 100_000 || budget.exhausted() {
        result.partial = true;
        return;
    }
    let Ok(_pin) = open_directory(path, FILE_READ_ATTRIBUTES) else {
        result.partial = true;
        return;
    };
    let Ok(entries) = std::fs::read_dir(path) else {
        result.partial = true;
        return;
    };
    for entry in entries {
        if result.files >= 100_000 || budget.exhausted() {
            result.partial = true;
            break;
        }
        let Ok(entry) = entry else {
            result.partial = true;
            continue;
        };
        let Ok(metadata) = std::fs::symlink_metadata(entry.path()) else {
            result.partial = true;
            continue;
        };
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            result.partial = true;
            continue;
        }
        if metadata.is_dir() {
            scan(&entry.path(), result, budget, depth + 1);
        } else if metadata.is_file() {
            result.bytes = result.bytes.saturating_add(metadata.len());
            result.files += 1;
        }
    }
}

fn normal_close_mask(slot:Option<u32>)->io::Result<[bool;5]> {
    if slot.is_some_and(|value|!(1..=5).contains(&value)) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile"));
    }
    Ok(std::array::from_fn(|index|slot.is_none_or(|value|value==index as u32+1)))
}

// Only the reviewed Minecraft Education main game surface (OGLES) is
// a normal-close target. A native exit dialog (#32770) can outlive that surface
// and must be handled exclusively by the exact-dialog verifier below.
fn game_window_class(hwnd: windows_sys::Win32::Foundation::HWND) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetClassNameW;
    let mut class = [0u16; 32];
    let count = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) };
    count == 5 && class[..5] == ['O' as u16, 'G' as u16, 'L' as u16, 'E' as u16, 'S' as u16]
}

fn valid_retained_window(target: &OwnedProcess) -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let Ok(number) = usize::from_str_radix(&target.window_handle[2..], 16) else {
        return false;
    };
    let hwnd = number as windows_sys::Win32::Foundation::HWND;
    let mut pid = 0;
    unsafe {
        IsWindow(hwnd) != 0
            && game_window_class(hwnd)
            && GetWindowThreadProcessId(hwnd, &mut pid) != 0
            && pid == target.pid
            && GetAncestor(hwnd, GA_ROOT) == hwnd
            && GetWindow(hwnd, GW_OWNER).is_null()
    }
}

struct RequestedWindow {
    handle: String,
    thread_id: u32,
    // Retain thread identity while this bounded close attempt runs. If that
    // thread exits, Windows cannot recycle its thread ID until the retained
    // handle is released. No game process is owned or terminated by this pin.
    _thread_pin: Option<OwnedHandle>,
}

#[derive(Default)]
struct NormalCloseProgress {
    requested_windows: Vec<RequestedWindow>,
    // Bounded once-per-exact-dialog (HWND, owning thread). Posting IDOK is not
    // evidence that the dialog was processed or the game process exited.
    submitted_dialogs: Vec<(usize, u32)>,
    blocked_by_modal: bool,
}

impl NormalCloseProgress {
    fn already_requested(&self, handle:&str, thread_id:u32) -> bool {
        self.requested_windows.iter().any(|request|
            request.handle==handle && request.thread_id==thread_id)
    }

    // A posted WM_CLOSE is only a request, not proof that the game consumed it.
    // A new verified HWND on a different owner thread is not the same lifetime
    // merely because the numeric HWND was reused.
    fn request_window(&mut self, target: &OwnedProcess) -> io::Result<()> {
        use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;
        let window=target.window_handle.strip_prefix("0x")
            .and_then(|value|usize::from_str_radix(value,16).ok())
            .ok_or_else(|| invalid("Invalid game window"))? as windows_sys::Win32::Foundation::HWND;
        let mut pid=0;
        let thread_id=unsafe {GetWindowThreadProcessId(window,&mut pid)};
        if thread_id==0 || pid!=target.pid || !valid_retained_window(target) {
            // A changing HWND does not authorize targeting a replacement or
            // failing the whole close. The same pinned child is reobserved.
            return Ok(());
        }
        if self.already_requested(&target.window_handle,thread_id) {
            return Ok(());
        }
        if self.requested_windows.len() >= 4 {
            return Err(invalid("Minecraft replaced its main window repeatedly during normal close"));
        }
        let thread_pin=pin_window_thread(target).filter(|handle|unsafe {
            windows_sys::Win32::System::Threading::GetThreadId(handle.as_raw_handle() as _)==thread_id
        });
        match close_verified_window(target) {
            // A native modal prompt disables its game owner. Wait for the
            // verified prompt or a re-enabled main window; do not abort early.
            Ok(()) => self.blocked_by_modal = false,
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                self.blocked_by_modal = true;
                return Ok(());
            }
            // A destroyed/replaced HWND between enumeration and PostMessageW
            // is an observation race. The pinned child, not this HWND, remains
            // the process authority. A fresh lookup can find its successor.
            Err(_) if !valid_retained_window(target) => return Ok(()),
            Err(error) => return Err(error),
        }
        self.requested_windows.push(RequestedWindow {
            handle:target.window_handle.clone(),
            thread_id,
            _thread_pin:thread_pin,
        });
        Ok(())
    }

    fn submit_exit_dialog(&mut self, child: &PendingEducationBootstrap) -> io::Result<bool> {
        confirm_owned_exit_once(child, &mut self.submitted_dialogs)
    }

    fn phase(&self, game_window_present: bool) -> &'static str {
        if !self.submitted_dialogs.is_empty() { "DIALOG_CONFIRMATION_SENT" }
        else if !self.requested_windows.is_empty() { "CLOSE_REQUESTED" }
        else if self.blocked_by_modal { "WAITING_FOR_MODAL" }
        else if game_window_present { "WINDOW_AVAILABLE" }
        else { "WAITING_FOR_WINDOW" }
    }

    fn original_thread_state(&self) -> &'static str {
        let Some(thread) = self.requested_windows.first().and_then(|value|value._thread_pin.as_ref()) else {
            return "not captured";
        };
        match unsafe { WaitForSingleObject(thread.as_raw_handle() as _, 0) } {
            WAIT_OBJECT_0 => "exited",
            WAIT_TIMEOUT => "alive",
            _ => "unavailable",
        }
    }

    fn describe_pending(&self, child: &PendingEducationBootstrap) -> String {
        // Never inspect account, world, dialog text or arbitrary process memory.
        let visible=window(child, false).is_some();
        let hidden=!visible && hidden_window_for_close(child).is_some();
        let game_window_present=visible||hidden;
        let main_window=if visible { "visible" } else if hidden { "hidden but owned" }
            else { "absent or ambiguous" };
        format!(
            "game process still running; close phase: {}; unique verified main HWND: {main_window}; original window thread: {}; distinct WM_CLOSE requests: {}; disabled main window encountered: {}; distinct exact exit-dialog commands posted: {}",
            self.phase(game_window_present),
            self.original_thread_state(),
            self.requested_windows.len(),
            self.blocked_by_modal,
            self.submitted_dialogs.len(),
        )
    }
}

// Read-only, original-window-thread evidence for a pinned managed game. Failure to
// obtain a thread handle must not prevent its ordinary WM_CLOSE/normal shutdown.
fn pin_window_thread(target: &OwnedProcess) -> Option<OwnedHandle> {
    use windows_sys::Win32::{
        System::Threading::{
            GetProcessIdOfThread, OpenThread, SYNCHRONIZATION_SYNCHRONIZE,
            THREAD_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::GetWindowThreadProcessId,
    };
    let hwnd = usize::from_str_radix(target.window_handle.strip_prefix("0x")?, 16).ok()?;
    let mut pid = 0;
    let thread_id = unsafe { GetWindowThreadProcessId(hwnd as _, &mut pid) };
    if thread_id == 0 || pid != target.pid {
        return None;
    }
    let raw = unsafe {
        OpenThread(
            SYNCHRONIZATION_SYNCHRONIZE | THREAD_QUERY_LIMITED_INFORMATION,
            0,
            thread_id,
        )
    };
    if raw.is_null() {
        return None;
    }
    let thread = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    if unsafe { GetProcessIdOfThread(thread.as_raw_handle() as _) } != target.pid {
        return None;
    }
    Some(thread)
}

fn close_verified_window(target:&OwnedProcess)->io::Result<()> {
    use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
    let handle=unsafe{OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION|SYNCHRONIZATION_SYNCHRONIZE,0,target.pid)};
    if handle.is_null(){return Err(io::Error::last_os_error());}
    let handle=unsafe{OwnedHandle::from_raw_handle(handle as _)};
    let mut birth:FILETIME=unsafe{std::mem::zeroed()};
    let mut exited=birth;let mut kernel=birth;let mut user=birth;
    if unsafe{GetProcessTimes(handle.as_raw_handle() as _,&mut birth,&mut exited,&mut kernel,&mut user)}==0 {
        return Err(io::Error::last_os_error());
    }
    let created=(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string();
    if created!=target.created_file_time || interactive_session(target.pid)?!=target.session_id || !valid_retained_window(target) {
        return Err(invalid("Game window identity changed; close it normally"));
    }
    let window=usize::from_str_radix(&target.window_handle[2..],16).map_err(|_|invalid("Invalid game window"))? as windows_sys::Win32::Foundation::HWND;
    if unsafe{GetWindowLongPtrW(window,GWL_STYLE)} & WS_DISABLED as isize != 0 {
        return Err(io::Error::new(io::ErrorKind::WouldBlock, "Minecraft game window has a modal dialog"));
    }
    if unsafe{PostMessageW(window,WM_CLOSE,0,0)}==0 {return Err(io::Error::last_os_error());}
    Ok(())
}

// Minecraft Education's standard English exit prompt is an unowned native dialog.
// Unknown/localized prompts remain for the user; never click arbitrary dialogs.
#[cfg(test)]
fn confirm_education_exit(target:&OwnedProcess)->io::Result<bool> {
    if !valid_retained_window(target) { return Ok(false); }
    confirm_exit_dialog(target.pid, || valid_retained_window(target))
}

fn confirm_owned_exit_once(child:&PendingEducationBootstrap, sent:&mut Vec<(usize,u32)>)->io::Result<bool> {
    let (pid, birth)=child.identity()?;
    confirm_exit_dialog_once(pid, || child.identity().ok()==Some((pid,birth)), sent)
}

// Wrapper retained for the existing exact-prompt fixture. Production CloseAll
// owns one bounded 'sent' set for each managed child throughout its attempt.
#[cfg(test)]
fn confirm_exit_dialog(owned_pid:u32, verify:impl Fn()->bool)->io::Result<bool> {
    confirm_exit_dialog_once(owned_pid,verify,&mut Vec::new())
}

fn confirm_exit_dialog_once(owned_pid:u32, verify:impl Fn()->bool, sent:&mut Vec<(usize,u32)>)->io::Result<bool> {
    use windows_sys::Win32::{Foundation::HWND,UI::{Input::KeyboardAndMouse::IsWindowEnabled,WindowsAndMessaging::*}};
    if !verify() { return Ok(false); }
    struct Search { pid:u32, matches:Vec<isize> }
    unsafe fn text(window:HWND)->Option<String> {
        let mut buffer=[0u16;256];let mut count=0usize;
        if SendMessageTimeoutW(window,WM_GETTEXT,buffer.len(),buffer.as_mut_ptr() as isize,
            SMTO_ABORTIFHUNG|SMTO_BLOCK|SMTO_ERRORONEXIT,100,&mut count)==0 {return None;}
        Some(String::from_utf16_lossy(&buffer[..count.min(buffer.len()-1)]))
    }
    unsafe fn matches_exact_prompt(window:HWND, expected_pid:u32)->bool {
        let mut pid=0;
        if GetWindowThreadProcessId(window,&mut pid)==0 || pid!=expected_pid
            || IsWindowVisible(window)==0 || IsWindowEnabled(window)==0 {return false;}
        let mut class=[0u16;32];let length=GetClassNameW(window,class.as_mut_ptr(),class.len() as i32);
        if length<=0 || String::from_utf16_lossy(&class[..length as usize])!="#32770" {return false;}
        let mut title=[0u16;64];let length=GetWindowTextW(window,title.as_mut_ptr(),title.len() as i32);
        if length<=0 || String::from_utf16_lossy(&title[..length as usize])!="Minecraft Education" {return false;}
        let message=GetDlgItem(window,65535);let ok=GetDlgItem(window,1);let cancel=GetDlgItem(window,2);
        if message.is_null() || ok.is_null() || cancel.is_null() {return false;}
        text(message).as_deref()==Some("Exit Minecraft Education?")
            && text(ok).as_deref()==Some("OK")
            && text(cancel).as_deref()==Some("Cancel")
    }
    unsafe extern "system" fn find(window:HWND,parameter:isize)->i32 {
        let search=&mut *(parameter as *mut Search);
        if matches_exact_prompt(window,search.pid) {search.matches.push(window as isize);}
        1
    }
    let mut search=Search{pid:owned_pid,matches:Vec::new()};
    if unsafe {EnumWindows(Some(find),&mut search as *mut Search as isize)}==0 {return Err(io::Error::last_os_error());}
    if search.matches.len()!=1 || !verify() {return Ok(false);}
    let dialog=search.matches[0] as HWND;
    // HWNDs can be destroyed/reused after EnumWindows. Revalidate ALL dialog
    // properties immediately before posting IDOK, not merely its owning PID.
    if !unsafe {matches_exact_prompt(dialog,owned_pid)} || !verify() {return Ok(false);}
    let mut actual_pid=0;
    let thread=unsafe {GetWindowThreadProcessId(dialog,&mut actual_pid)};
    if thread==0 || actual_pid!=owned_pid {return Ok(false);}
    let identity=(dialog as usize,thread);
    if sent.contains(&identity) { return Ok(true); }
    if sent.len()>=4 {
        return Err(invalid("Minecraft opened too many distinct exit prompts during normal close"));
    }
    if unsafe {PostMessageW(dialog,WM_COMMAND,1,0)}==0 {return Err(io::Error::last_os_error());}
    sent.push(identity);
    Ok(true)
}

// Exactly the confirmed game's original top-level OGLES root can receive
// WM_CLOSE while hidden. Ordinary launcher selection is deliberately
// visibility-only. A modal #32770 prompt must NEVER be eligible here.
fn hidden_close_candidate(hwnd:windows_sys::Win32::Foundation::HWND,expected_pid:u32)->bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let mut actual_pid=0;
    unsafe {
        IsWindow(hwnd)!=0
            && GetWindowThreadProcessId(hwnd,&mut actual_pid)!=0
            && actual_pid==expected_pid
            && IsWindowVisible(hwnd)==0
            && game_window_class(hwnd)
            && GetAncestor(hwnd,GA_ROOT)==hwnd
            && GetWindow(hwnd,GW_OWNER).is_null()
    }
}

fn hidden_window_for_close(child:&PendingEducationBootstrap)->Option<OwnedProcess> {
    use windows_sys::Win32::{Foundation::HWND,UI::WindowsAndMessaging::*};
    struct Found { pid:u32, total:u32, hidden:HWND }
    unsafe extern "system" fn select(hwnd:HWND,context:isize)->i32 {
        let found=&mut *(context as *mut Found);
        let mut pid=0;
        if GetWindowThreadProcessId(hwnd,&mut pid)!=0 && pid==found.pid
            && IsWindow(hwnd)!=0 && game_window_class(hwnd)
            && GetAncestor(hwnd,GA_ROOT)==hwnd && GetWindow(hwnd,GW_OWNER).is_null() {
            // Refuse any ambiguity, including an additional visible OGLES.
            found.total+=1;
            if IsWindowVisible(hwnd)==0 {found.hidden=hwnd;}
        }
        1
    }
    let (pid,birth)=child.identity().ok()?;
    let mut found=Found{pid,total:0,hidden:ptr::null_mut()};
    if unsafe{EnumWindows(Some(select),&mut found as *mut Found as isize)}==0
        || found.total!=1 || found.hidden.is_null() {return None;}
    if child.identity().ok()!=Some((pid,birth)) || !hidden_close_candidate(found.hidden,pid){
        return None;
    }
    let candidate=OwnedProcess{
        pid,
        created_file_time:birth.to_string(),
        session_id:interactive_session(pid).ok()?,
        window_handle:format!("0x{:x}",found.hidden as usize),
        responding:false, // Hidden is not equivalent to playable or visible.
    };
    valid_retained_window(&candidate).then_some(candidate)
}

fn window(child: &PendingEducationBootstrap, label: bool) -> Option<OwnedProcess> {
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId},
    };
    struct Selection {
        pid: u32,
        hwnd: HWND,
        count: u32,
    }
    unsafe extern "system" fn select(hwnd: HWND, data: isize) -> i32 {
        let selection = &mut *(data as *mut Selection);
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == selection.pid
            && game_window_class(hwnd)
            && super::super::verify_live_client_02_window(hwnd, pid).is_ok() {
            selection.hwnd = hwnd;
            selection.count += 1;
        }
        1
    }
    child.identity().ok()?;
    let mut selected = Selection {
        pid: child.pid,
        hwnd: ptr::null_mut(),
        count: 0,
    };
    if unsafe { EnumWindows(Some(select), &mut selected as *mut _ as isize) } == 0
        || selected.count != 1
    {
        return None;
    }
    child.identity().ok()?;
    let slot = child.storage.as_ref()?.slot;
    if label {
        let _ = label_slot_window(selected.hwnd, child.pid, slot);
    }
    let process=OwnedProcess {
        pid: child.pid,
        created_file_time: child.created.to_string(),
        session_id: interactive_session(child.pid).ok()?,
        window_handle: format!("0x{:x}", selected.hwnd as usize),
        // Enumeration only verifies a visible root. It does not establish
        // message-loop responsiveness; match the retained-HWND snapshot path.
        responding: unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::IsHungAppWindow(selected.hwnd)==0
        },
    };
    // Recheck immediately after observing/optionally labeling. A recycled
    // HWND or destroyed game surface is never published as a playable game.
    valid_retained_window(&process).then_some(process)
}

// Audio COM reads are separate from process reaping. The existing
// controller still reaps every idle 250 ms; stable audio needs no repeated
// COM GetMute call until its 1-second discovery deadline. Focus or an exact
// retained game process change bypasses that interval immediately.
const AUDIO_IDLE_RESYNC: Duration = Duration::from_secs(1);
type AudioRoster = [Option<(u32, u64)>; 5];
fn audio_roster(sessions: &Sessions) -> AudioRoster {
    // Retained handles are the only source of ownership. Cheap metadata
    // sampling neither enumerates PIDs nor authorizes any new process.
    std::array::from_fn(|index| sessions.running[index].as_ref()
        .map(|running| (running.children[0].pid, running.children[0].created)))
}
#[derive(Default)]
struct AudioIdleGate {
    last_success: Option<Instant>,
    foreground: Option<Option<u32>>,
    roster: Option<AudioRoster>,
}
impl AudioIdleGate {
    fn due(&self, now: Instant, foreground: Option<u32>, roster: AudioRoster) -> bool {
        self.last_success.is_none_or(|last|
            now.saturating_duration_since(last) >= AUDIO_IDLE_RESYNC)
            || self.foreground != Some(foreground)
            || self.roster != Some(roster)
    }
    fn success(&mut self, now: Instant, foreground: Option<u32>, roster: AudioRoster) {
        self.last_success = Some(now);
        self.foreground = Some(foreground);
        self.roster = Some(roster);
    }
    fn invalidate(&mut self) {
        // A failed COM sync is retried on the next existing idle tick, not
        // suppressed by a previously successful cadence checkpoint.
        self.last_success = None;
    }
}
fn audio_maintenance_needed(
    mode: crate::engine::manager_settings::BackgroundAudio,
    audio_present: bool, roster: AudioRoster,
) -> bool {
    use crate::engine::manager_settings::BackgroundAudio;
    audio_present || mode == BackgroundAudio::Mute
        && roster.iter().any(Option::is_some)
}
fn sync_audio(sessions:&Sessions,audio:&mut Option<super::session_audio::Audio>,
    mode:crate::engine::manager_settings::BackgroundAudio, foreground:Option<u32>)->io::Result<()> {
    use crate::engine::manager_settings::BackgroundAudio;
    if mode==BackgroundAudio::Keep {
        if let Some(current)=audio.as_mut(){current.restore()?;}
        *audio=None;return Ok(());
    }
    let targets=sessions.running.iter().flatten().filter_map(|s|s.children[0].identity().ok()).collect::<Vec<_>>();
    if targets.is_empty(){if let Some(current)=audio.as_mut(){current.restore()?;}*audio=None;return Ok(());}
    if audio.is_none(){*audio=Some(super::session_audio::Audio::new()?);}
    audio.as_mut().unwrap().sync(&targets,foreground,|pid,born|sessions.running.iter().flatten().any(|s|s.children[0].identity().ok()==Some((pid,born))))
}

// Admit an authenticated, queued request before optional idle maintenance on
// this SAME controller actor. It introduces no additional worker or game owner.
fn receive_before_maintenance<T>(
    receiver: &std::sync::mpsc::Receiver<T>,
    timeout: Duration,
    on_idle: impl FnOnce(),
) -> Result<T, std::sync::mpsc::RecvTimeoutError> {
    let result = receiver.recv_timeout(timeout);
    if result.is_err() {
        // Include transport disconnect: retain and reap exact game handles
        // rather than mistaking a disconnected manager for game exit.
        on_idle();
    }
    result
}

// Only a successful coordinated close has already reaped its selected slots.
// The reply must not re-enter global journal persistence for unrelated games.
// Ordinary Inspect and other operations retain their existing full reaping.
fn response_reaps_all(request: &SessionRequest) -> bool {
    !matches!(request, SessionRequest::CloseOne { .. } | SessionRequest::CloseAll)
}

fn run() -> io::Result<()> {
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS},
        System::Threading::CreateMutexW,
    };
    crate::engine::workspace_surface::verify_same_executable_parent()?;
    crate::engine::dpi::configure()?;
    // Same gate as historical pair/five CLI: no parallel lifecycle authority.
    let name = wide(OsStr::new("Local\\MivubiEducationStartupObservation"))?;
    let raw = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
    if raw.is_null() {
        return Err(io::Error::last_os_error());
    }
    let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let _gate = unsafe { OwnedHandle::from_raw_handle(raw as _) };
    if existed {
        return Err(invalid(
            "Another Minecraft controller is still active; existing games are preserved",
        ));
    }
    let mut sessions = Sessions::prepare()?;
    let mut audio=None;
    let mut audio_mode=crate::engine::manager_settings::load_audio()?;
    // Audio's COM session enumerator already has a one-second discovery
    // cadence. Keep 250 ms original game exit/reap checks unchanged; avoid
    // re-running all COM mute checks while focus and owned processes are stable.
    let mut audio_idle = AudioIdleGate::default();
    // No audio failure observed before the first idle maintenance cycle.
    let mut audio_issue: Option<String> = None;
    let server = Server::start(Endpoint::current()?)?;
    let mut manager: Option<Peer> = None;
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
        let pending = receive_before_maintenance(
            &server.requests,
            Duration::from_millis(250),
            || {
                // An original game exit is still observed within the same
                // 250 ms idle policy. Audio does not gate or replace reaping.
                sessions.reap();
                let roster = audio_roster(&sessions);
                if !audio_maintenance_needed(audio_mode, audio.is_some(), roster) {
                    audio_issue = None;
                    audio_idle.invalidate();
                } else {
                    let foreground = super::session_audio::foreground();
                    let now = Instant::now();
                    if audio_idle.due(now, foreground, roster) {
                        match sync_audio(&sessions, &mut audio, audio_mode, foreground) {
                            Ok(()) => {
                                audio_issue = None;
                                audio_idle.success(now, foreground, roster);
                            }
                            Err(error) => {
                                audio_issue = Some(error.to_string());
                                audio_idle.invalidate();
                            }
                        }
                    }
                }
            },
        );
        let detached = manager.as_ref().map_or_else(
            || startup.elapsed() > Duration::from_secs(45),
            |peer| !peer.alive(),
        );
        if detached && sessions.running.iter().all(Option::is_none) {
            return Ok(());
        }
        let envelope = match pending {
            Ok(envelope) => envelope,
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => continue,
            Err(_) => {
                // Even a failed transport must retain game/storage/FPS ownership until normal exit.
                if sessions.running.iter().all(Option::is_none) {
                    return Err(invalid("Session transport stopped"));
                }
                std::thread::sleep(Duration::from_millis(250));
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
            let mut settings = None;
            let reap_all_for_reply = response_reaps_all(&envelope.request);
            match envelope.request {
                SessionRequest::Inspect => {}
                SessionRequest::InspectSettings => {
                    // This is a read-only UI status, not mutation admission.
                    settings = Some(sessions.settings_observation().map_err(|error| error.to_string())?);
                }
                SessionRequest::ApplyPreset => {
                    settings = Some(sessions.settings(true).map_err(|error| error.to_string())?);
                }
                SessionRequest::ApplyGraphicsPreset { slot, preset } => {
                    let report = sessions.settings(false).map_err(|error| error.to_string())?;
                    require_preset_idle(report.can_apply_preset).map_err(|error| error.to_string())?;
                    let mut values = crate::engine::manager_settings::load_graphics().map_err(|error| error.to_string())?;
                    let changed = game_preset::apply_selected(sessions.store(slot).map_err(|error| error.to_string())?, preset).map_err(|error| error.to_string())?;
                    values[slot as usize - 1] = preset;
                    crate::engine::manager_settings::save_graphics(values).map_err(|error| error.to_string())?;
                    let mut report = sessions.settings(false).map_err(|error| error.to_string())?;
                    report.changed_profiles = Some(changed as u32);
                    settings = Some(report);
                }
                SessionRequest::SavePerformance {active_fps,background_fps} => {
                    crate::engine::manager_settings::save_performance(crate::engine::manager_settings::PerformanceSettings {active_fps,background_fps})
                        .map_err(|error|error.to_string())?;
                    settings=Some(sessions.settings(false).map_err(|error|error.to_string())?);
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
                    settings=Some(sessions.settings(false).map_err(|e|e.to_string())?);
                }
                SessionRequest::Launch { slot } => {
                    sessions.launch(slot).map_err(|error| error.to_string())?
                }
                SessionRequest::CloseAll => sessions.close_all().map_err(|error|error.to_string())?,
                SessionRequest::CloseOne {slot} => sessions.close_one(slot).map_err(|error|error.to_string())?,
                SessionRequest::InspectStorage => {
                    storage = Some(sessions.storage().map_err(|error| error.to_string())?)
                }
                SessionRequest::ClearProfileCache { slot } => {
                    sessions
                        .clear_cache(slot)
                        .map_err(|error| error.to_string())?;
                    storage = Some(sessions.storage().map_err(|error| error.to_string())?);
                }
                SessionRequest::ResetGameSettings { slot } | SessionRequest::RestoreGameSettings { slot } | SessionRequest::DisableImprovedInputResponse { slot } => {
                    sessions.reap();
                    let observed=super::super::inspect_native_workspace().map_err(|error|error.to_string())?.processes;
                    sessions.data_admission(slot,&observed).map_err(|error|error.to_string())?;
                    let store=sessions.store(slot).map_err(|error|error.to_string())?;
                    if matches!(envelope.request,SessionRequest::ResetGameSettings {..}) {
                        game_preset::reset_settings(&store.private)
                    } else if matches!(envelope.request,SessionRequest::DisableImprovedInputResponse {..}) {
                        game_preset::disable_input_response(&store.private)
                    } else {
                        game_preset::restore_settings(&store.private)
                    }.map_err(|error|error.to_string())?;
                    storage=Some(sessions.storage().map_err(|error|error.to_string())?);
                }
                SessionRequest::ClearProfileData {slot,confirmation_token,confirmation_name}=>{
                    if confirmation_name!=format!("Minecraft - {slot}") {return Err("Launcher confirmation does not match".into());}
                    sessions.clear_data(slot,&confirmation_token).map_err(|error|error.to_string())?;
                    storage=Some(sessions.storage().map_err(|error|error.to_string())?);
                }
                SessionRequest::OpenProfileFolder { slot } => sessions
                    .open_folder(slot)
                    .map_err(|error| error.to_string())?,
            }
            let mut snapshot = if reap_all_for_reply {
                sessions.snapshot()
            } else {
                // CloseOne has already persisted its selected journal; CloseAll
                // has already persisted all five. Never flush a peer again here.
                sessions.snapshot_without_reaping()
            };
            if let Some(report)=settings.as_mut(){report.audio_issue=audio_issue.clone();}
            snapshot.storage = storage;
            snapshot.settings = settings;
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
mod tests {
    use super::*;
    #[test]
    fn close_replies_do_not_trigger_unrelated_journal_reap() {
        // Pure routing assertions; do not launch a game or touch profile data.
        for request in [
            SessionRequest::CloseOne { slot: 1 },
            SessionRequest::CloseOne { slot: 5 },
            SessionRequest::CloseAll,
        ] {
            assert!(!response_reaps_all(&request));
        }
        for request in [
            SessionRequest::Inspect,
            SessionRequest::InspectSettings,
            SessionRequest::InspectStorage,
            SessionRequest::Launch { slot: 2 },
        ] {
            assert!(response_reaps_all(&request));
        }
        // CloseOne reaps one verified slot; CloseAll still reaps all five.
        for slot in 1..=5 {
            let selected = normal_close_mask(Some(slot)).unwrap();
            assert_eq!(selected.iter().filter(|&&enabled| enabled).count(), 1);
            assert!(selected[(slot - 1) as usize]);
        }
        assert_eq!(normal_close_mask(None).unwrap(), [true; 5]);
    }

    #[test]
    fn stable_audio_skips_redundant_com_work_but_keeps_focus_and_exit_immediate() {
        use crate::engine::manager_settings::BackgroundAudio;
        let start = Instant::now();
        let roster: AudioRoster = [Some((102, 134000000000000001)), None, None, None, None];
        let mut gate = AudioIdleGate::default();
        assert!(gate.due(start, Some(102), roster));
        gate.success(start, Some(102), roster);
        for elapsed in [250, 500, 750] {
            assert!(!gate.due(start + Duration::from_millis(elapsed), Some(102), roster),
                "Stable mute policy must not read COM every 250 ms");
        }
        assert!(gate.due(start + AUDIO_IDLE_RESYNC, Some(102), roster),
            "New audio sessions can still be discovered once per second");
        assert!(gate.due(start + Duration::from_millis(250), Some(103), roster),
            "A foreground change must trigger the next idle tick");
        let mut replaced = roster;
        replaced[0] = Some((103, 134000000000000002));
        assert!(gate.due(start + Duration::from_millis(250), Some(102), replaced),
            "A reused PID or new original birth must not inherit mute state");
        gate.invalidate();
        assert!(gate.due(start + Duration::from_millis(250), Some(102), roster),
            "Any failed COM sync must be retryable immediately");
        assert!(!audio_maintenance_needed(BackgroundAudio::Keep, false, roster));
        assert!(audio_maintenance_needed(BackgroundAudio::Mute, false, roster));
        assert!(audio_maintenance_needed(BackgroundAudio::Keep, true, [None; 5]),
            "An outstanding mute lease must still be restored");
        assert!(!audio_maintenance_needed(BackgroundAudio::Mute, false, [None; 5]),
            "No owned process and no audio lease needs no COM startup");
    }

    #[test]
    fn queued_close_admission_precedes_idle_journal_maintenance() {
        use std::sync::mpsc::{self, RecvTimeoutError};
        // Pure channel fixture: no native process, journal, helper or game.
        let (sender, receiver) = mpsc::sync_channel(1);
        sender.send(SessionRequest::CloseOne { slot: 2 }).unwrap();
        let mut maintenance = 0;
        let queued = receive_before_maintenance(&receiver, Duration::ZERO, || {
            maintenance += 1;
        }).unwrap();
        assert!(matches!(queued, SessionRequest::CloseOne { slot: 2 }));
        assert_eq!(maintenance, 0, "queued close must precede idle I/O");

        let idle = receive_before_maintenance(&receiver, Duration::ZERO, || {
            maintenance += 1;
        });
        assert!(matches!(idle, Err(RecvTimeoutError::Timeout)));
        assert_eq!(maintenance, 1, "idle maintenance remains scheduled");

        drop(sender);
        let disconnected = receive_before_maintenance(&receiver, Duration::ZERO, || {
            maintenance += 1;
        });
        assert!(matches!(disconnected, Err(RecvTimeoutError::Disconnected)));
        assert_eq!(maintenance, 2, "disconnection retains cleanup");
    }

    // Private synthetic OGLES class, not Minecraft. Shared registration is
    // initialized once because Rust test cases can execute concurrently.
    fn fixture_game_window() -> windows_sys::Win32::Foundation::HWND {
        use windows_sys::Win32::{
            System::LibraryLoader::GetModuleHandleW,
            UI::WindowsAndMessaging::{CreateWindowExW, DefWindowProcW, RegisterClassW, WNDCLASSW, WS_OVERLAPPEDWINDOW},
        };
        static REGISTERED: std::sync::OnceLock<()> = std::sync::OnceLock::new();
        REGISTERED.get_or_init(|| unsafe {
            let mut class: WNDCLASSW = std::mem::zeroed();
            class.hInstance = GetModuleHandleW(std::ptr::null());
            class.lpszClassName = windows_sys::w!("OGLES");
            class.lpfnWndProc = Some(DefWindowProcW);
            assert_ne!(RegisterClassW(&class), 0);
        });
        unsafe {
            CreateWindowExW(0, windows_sys::w!("OGLES"), windows_sys::w!("Fixture"),
                WS_OVERLAPPEDWINDOW, 0, 0, 200, 150,
                std::ptr::null_mut(), std::ptr::null_mut(),
                GetModuleHandleW(std::ptr::null()), std::ptr::null())
        }
    }

    #[test]
    fn stopped_profile_cache_allows_other_owned_games_but_rejects_unknown_or_reused_processes() {
        let other = OwnedProcess { pid:42, created_file_time:"123456".into(), session_id:1,
            window_handle:"0x1234".into(), responding:true };
        assert!(require_data_idle(true, &[(42,123456)], std::slice::from_ref(&other)).is_ok());
        assert!(require_data_idle(false, &[(42,123456)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[(42,123455)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[(43,123456)], std::slice::from_ref(&other)).is_err());
        assert!(require_data_idle(true, &[], &[]).is_ok());
    }

    #[test]
    fn normal_close_only_reaps_after_all_selected_processes_have_exited() {
        let mut sessions=Sessions{
            stores:(0..5).map(|_|None).collect(),
            running:(0..5).map(|_|None).collect(),
            issues:vec![None;5],windows:vec![None;5],
            path:String::new(),
            image:File::open(std::env::current_exe().unwrap()).unwrap(),
            exiting:false,
        };
        let one=normal_close_mask(Some(2)).unwrap();
        assert!(sessions.selected_processes_exited(&one).unwrap());
        let all=normal_close_mask(None).unwrap();
        assert!(sessions.selected_processes_exited(&all).unwrap());
        // Inspect is still an ordinary reaping operation, while the close
        // path can safely take a non-reaping snapshot of all five slots.
        assert_eq!(sessions.snapshot_without_reaping().profiles.len(),5);
        assert!(sessions.running.iter().all(Option::is_none));
    }

    #[test]
    fn selected_normal_close_mask_affects_only_one_launcher() {
        assert_eq!(normal_close_mask(None).unwrap(),[true;5]);
        for slot in 1..=5 {
            let mask=normal_close_mask(Some(slot)).unwrap();
            for (index,selected) in mask.iter().enumerate() {
                assert_eq!(*selected,index as u32+1==slot);
            }
        }
        for slot in [0,6,u32::MAX] {
            assert_eq!(normal_close_mask(Some(slot)).unwrap_err().kind(),io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn full_exit_of_idle_sessions_does_not_prepare_profiles_and_denies_new_launches() {
        let mut sessions=Sessions{stores:(0..5).map(|_|None).collect(),running:(0..5).map(|_|None).collect(),issues:vec![None;5],windows:vec![None;5],path:String::new(),image:File::open(std::env::current_exe().unwrap()).unwrap(),exiting:false};
        // The selected-close owner must never report a successful exit
        // for a stopped slot even if it has no process to wait for.
        for slot in 1..=5 {
            assert!(sessions.close_one(slot).unwrap_err().to_string().contains("not running"));
            assert!(!sessions.exiting);
        }
        sessions.close_all().unwrap();
        assert!(sessions.exiting);
        assert!(sessions.stores.iter().all(Option::is_none));
        assert!(sessions.launch(1).unwrap_err().to_string().contains("closing"));
    }

    #[test]
    fn normal_close_checks_exact_window_identity_and_leaves_another_window_alone() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let own=fixture_game_window();let other=fixture_game_window();assert!(!own.is_null()&&!other.is_null());
            let result=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{pid:std::process::id(),created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),session_id:interactive_session(std::process::id()).unwrap(),window_handle:format!("0x{:x}",own as usize),responding:true};
                let mut forged=target.clone();forged.created_file_time="1".into();assert!(close_verified_window(&forged).is_err());
                assert_ne!(IsWindow(own),0);
                let style=GetWindowLongPtrW(own,GWL_STYLE);SetWindowLongPtrW(own,GWL_STYLE,style|WS_DISABLED as isize);
                assert!(close_verified_window(&target).is_err());SetWindowLongPtrW(own,GWL_STYLE,style);
                close_verified_window(&target).unwrap();
                let mut message:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut message,own,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                DispatchMessageW(&message);
                assert_eq!(IsWindow(own),0);
                assert_ne!(IsWindow(other),0);
            });
            if IsWindow(own)!=0{DestroyWindow(own);}DestroyWindow(other);
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }


    #[test]
    fn normal_close_posts_once_per_window_and_tracks_original_window_thread() {
        use windows_sys::Win32::{Foundation::FILETIME, System::Threading::*, UI::WindowsAndMessaging::*};
        unsafe {
            let first=fixture_game_window();let successor=fixture_game_window();
            assert!(!first.is_null()&&!successor.is_null()&&first!=successor);
            let result=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{
                    pid:std::process::id(),
                    created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),
                    session_id:interactive_session(std::process::id()).unwrap(),
                    window_handle:format!("0x{:x}",first as usize),
                    responding:true,
                };
                let mut progress=NormalCloseProgress::default();
                let style=GetWindowLongPtrW(first,GWL_STYLE);
                SetWindowLongPtrW(first,GWL_STYLE,style|WS_DISABLED as isize);
                progress.request_window(&target).unwrap();
                assert!(progress.blocked_by_modal);
                assert!(progress.requested_windows.is_empty());
                let mut none:MSG=std::mem::zeroed();
                assert_eq!(PeekMessageW(&mut none,first,WM_CLOSE,WM_CLOSE,PM_NOREMOVE),0);
                SetWindowLongPtrW(first,GWL_STYLE,style);
                progress.request_window(&target).unwrap();
                progress.request_window(&target).unwrap();
                assert!(!progress.blocked_by_modal);
                assert_eq!(progress.requested_windows.len(),1);
                assert_eq!(progress.original_thread_state(),"alive");
                let mut message:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut message,first,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_eq!(PeekMessageW(&mut message,first,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);

                // A new, uniquely verified top-level HWND receives its own request.
                let new_target=OwnedProcess{window_handle:format!("0x{:x}",successor as usize),..target};
                progress.request_window(&new_target).unwrap();
                progress.request_window(&new_target).unwrap();
                assert_eq!(progress.requested_windows.len(),2);
                assert_ne!(PeekMessageW(&mut message,successor,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_eq!(PeekMessageW(&mut message,successor,WM_CLOSE,WM_CLOSE,PM_REMOVE),0);
                assert_ne!(IsWindow(successor),0);
            });
            if IsWindow(first)!=0{DestroyWindow(first);}
            if IsWindow(successor)!=0{DestroyWindow(successor);}
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }


    #[test]
    fn hidden_game_close_candidate_refuses_dialog_and_foreign_pid() {
        use windows_sys::Win32::UI::WindowsAndMessaging::*;
        unsafe {
            let game=fixture_game_window();
            let dialog=CreateWindowExW(0,windows_sys::w!("#32770"),
                windows_sys::w!("Minecraft Education"),WS_POPUP,
                0,0,200,120,std::ptr::null_mut(),std::ptr::null_mut(),
                std::ptr::null_mut(),std::ptr::null());
            assert!(!game.is_null()&&!dialog.is_null());
            let result=std::panic::catch_unwind(||{
                let pid=std::process::id();
                assert_eq!(IsWindowVisible(game),0);
                assert!(hidden_close_candidate(game,pid));
                assert!(!hidden_close_candidate(game,pid.wrapping_add(1)));
                assert!(!hidden_close_candidate(dialog,pid));
                let mut posted:MSG=std::mem::zeroed();
                assert_eq!(PeekMessageW(&mut posted,dialog,WM_CLOSE,WM_CLOSE,PM_NOREMOVE),0);
            });
            DestroyWindow(dialog);
            DestroyWindow(game);
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }

    #[test]
    fn exit_dialog_is_not_accepted_as_an_ogles_game_window() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let dialog=CreateWindowExW(0,windows_sys::w!("#32770"),windows_sys::w!("Minecraft Education"),
                WS_POPUP|WS_VISIBLE,0,0,250,140,ptr::null_mut(),ptr::null_mut(),ptr::null_mut(),ptr::null());
            assert!(!dialog.is_null());
            let outcome=std::panic::catch_unwind(||{
                let mut birth:FILETIME=std::mem::zeroed();let mut exit=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exit,&mut kernel,&mut user),0);
                let target=OwnedProcess{
                    pid:std::process::id(),
                    created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),
                    session_id:interactive_session(std::process::id()).unwrap(),
                    window_handle:format!("0x{:x}",dialog as usize),
                    responding:true,
                };
                assert!(!game_window_class(dialog));
                assert!(!valid_retained_window(&target));
                assert!(close_verified_window(&target).is_err());
                let mut posted:MSG=std::mem::zeroed();
                assert_eq!(PeekMessageW(&mut posted,dialog,WM_CLOSE,WM_CLOSE,PM_NOREMOVE),0);
                assert_ne!(IsWindow(dialog),0);
            });
            DestroyWindow(dialog);
            if let Err(error)=outcome{std::panic::resume_unwind(error);}
        }
    }


    #[test]
    fn hwnd_reuse_on_another_thread_is_not_silently_deduplicated() {
        let mut progress=NormalCloseProgress::default();
        progress.requested_windows.push(RequestedWindow {
            handle:"0x42".into(),
            thread_id:77,
            _thread_pin:None,
        });
        assert!(progress.already_requested("0x42",77));
        assert!(!progress.already_requested("0x42",78));
        assert!(!progress.already_requested("0x43",77));
        // Same numeric HWND reused on the same live GUI thread remains
        // unprovable without in-process lifetime evidence. No blind repost.
    }

    #[test]
    fn close_phase_never_claims_process_exit_from_a_posted_message() {
        let mut progress=NormalCloseProgress::default();
        assert_eq!(progress.phase(false),"WAITING_FOR_WINDOW");
        assert_eq!(progress.phase(true),"WINDOW_AVAILABLE");
        progress.blocked_by_modal=true;
        assert_eq!(progress.phase(true),"WAITING_FOR_MODAL");
        progress.requested_windows.push(RequestedWindow {
            handle:"0x42".into(),
            thread_id:77,
            _thread_pin:None,
        });
        assert_eq!(progress.phase(false),"CLOSE_REQUESTED");
        progress.submitted_dialogs.push((0x99,42));
        assert_eq!(progress.phase(false),"DIALOG_CONFIRMATION_SENT");
        // The real process-exit state is owned solely by Sessions::reap().
    }

    #[test]
    fn exit_dialog_refuses_unverified_process_without_enumerating_windows() {
        assert!(!confirm_exit_dialog(std::process::id(), || false).unwrap());
    }

    #[test]
    fn exit_confirmation_is_limited_to_the_exact_owned_prompt() {
        use windows_sys::Win32::{Foundation::FILETIME,System::Threading::*,UI::WindowsAndMessaging::*};
        unsafe {
            let make=|class,caption,style,parent,id|CreateWindowExW(WS_EX_NOACTIVATE,class,caption,style,
                -30000,-30000,200,100,parent,id as _,std::ptr::null_mut(),std::ptr::null());
            let game=fixture_game_window();
            let dialog=make(windows_sys::w!("#32770"),windows_sys::w!("Minecraft Education"),WS_POPUP|WS_VISIBLE,
                std::ptr::null_mut(),0usize);
            assert!(!game.is_null()&&!dialog.is_null());
            let result=std::panic::catch_unwind(||{
                let message=make(windows_sys::w!("STATIC"),windows_sys::w!("Unknown question"),WS_CHILD|WS_VISIBLE,dialog,65535);
                let ok=make(windows_sys::w!("BUTTON"),windows_sys::w!("OK"),WS_CHILD|WS_VISIBLE,dialog,1);
                let cancel=make(windows_sys::w!("BUTTON"),windows_sys::w!("Cancel"),WS_CHILD|WS_VISIBLE,dialog,2);
                assert!(!message.is_null()&&!ok.is_null()&&!cancel.is_null());
                let mut birth:FILETIME=std::mem::zeroed();let mut exited=birth;let mut kernel=birth;let mut user=birth;
                assert_ne!(GetProcessTimes(GetCurrentProcess(),&mut birth,&mut exited,&mut kernel,&mut user),0);
                let target=OwnedProcess{pid:std::process::id(),created_file_time:(((birth.dwHighDateTime as u64)<<32)|birth.dwLowDateTime as u64).to_string(),session_id:interactive_session(std::process::id()).unwrap(),window_handle:format!("0x{:x}",game as usize),responding:true};
                assert!(!confirm_education_exit(&target).unwrap());
                assert_ne!(SetWindowTextW(message,windows_sys::w!("Exit Minecraft Education?")),0);
                let mut submitted=Vec::new();
                assert!(confirm_exit_dialog_once(target.pid,||valid_retained_window(&target),&mut submitted).unwrap());
                let mut received:MSG=std::mem::zeroed();
                assert_ne!(PeekMessageW(&mut received,dialog,WM_COMMAND,WM_COMMAND,PM_REMOVE),0);
                assert_eq!(received.wParam,1);
                assert_eq!(submitted.len(),1);
                // Still-present same dialog is observed, never spammed with IDOK.
                assert!(confirm_exit_dialog_once(target.pid,||valid_retained_window(&target),&mut submitted).unwrap());
                assert_eq!(submitted.len(),1);
                assert_eq!(PeekMessageW(&mut received,dialog,WM_COMMAND,WM_COMMAND,PM_REMOVE),0);
                assert_ne!(SetWindowTextW(message,windows_sys::w!("Different question")),0);
                assert!(!confirm_exit_dialog_once(target.pid,||valid_retained_window(&target),&mut submitted).unwrap());
                assert_eq!(submitted.len(),1);
                let mut forged=target.clone();forged.pid=forged.pid.wrapping_add(1);
                assert!(!confirm_education_exit(&forged).unwrap());
            });
            DestroyWindow(dialog);DestroyWindow(game);
            if let Err(error)=result{std::panic::resume_unwind(error);}
        }
    }

    #[test]
    fn preset_request_refuses_uncertain_storage_before_writing_or_starting_any_game() {
        let mut sessions=Sessions {
            stores:(0..5).map(|_|None).collect(), running:(0..5).map(|_|None).collect(),
            issues:vec![None;5], windows:vec![None;5], path:String::new(),
            image:File::open(std::env::current_exe().unwrap()).unwrap(), exiting:false,
        };
        assert!(sessions.settings(true).unwrap_err().to_string().contains("Close all Minecraft"));
        assert!(sessions.running.iter().all(Option::is_none));
        assert!(sessions.snapshot().profiles.iter().all(|profile|profile.state=="ATTENTION"));
    }

    #[test]
    fn metadata_scan_counts_files_and_reports_bounds_without_opening_contents() {
        let directory = std::env::temp_dir().join(format!(
            "lazy-tester-storage-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        std::fs::create_dir(directory.join("nested")).unwrap();
        std::fs::write(directory.join("a.bin"), [0u8; 17]).unwrap();
        std::fs::write(directory.join("nested/b.bin"), [0u8; 23]).unwrap();
        let mut result = ProfileStorage {
            slot: 5,
            bytes: 0,
            files: 0,
            partial: false,
            disk: None,
            cache: Default::default(),
            game_settings: None,
            full_clear: None,
        };
        scan(&directory, &mut result, InspectionBudget::new(Duration::from_secs(2)), 0);
        assert_eq!((result.bytes, result.files, result.partial), (40, 2, false));
        scan(&directory, &mut result, InspectionBudget::new(Duration::from_secs(2)), 49);
        assert_eq!((result.bytes, result.files, result.partial), (40, 2, true));
        // Expired shared admission never pretends a zero-byte snapshot is complete.
        let mut expired = result.clone();
        expired.bytes = 0;
        expired.files = 0;
        expired.partial = false;
        scan(&directory, &mut expired, InspectionBudget::new(Duration::ZERO), 0);
        assert_eq!((expired.bytes, expired.files, expired.partial), (0, 0, true));
        // Only the unique, create-new disposable fixture directory created above.
        assert_eq!(directory.parent(), Some(std::env::temp_dir().as_path()));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
