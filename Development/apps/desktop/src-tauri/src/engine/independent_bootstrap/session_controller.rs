//! One retained bootstrap owner. Full exit requests normal closure of its own games.
use super::*;
use crate::engine::{
    native_clients_session::{
        pipe::{Activity, Endpoint, Peer, Server},
        ProfileSession, ProfileStorage, SessionRequest, SessionSnapshot, SettingsReport,
        SelectiveCategory, SelectiveInventory, SelectiveInventoryEntry, SelectiveSelection, SelectiveReviewReceipt,
    },
    process_identity::OwnedProcess,
};
use std::{io::Write, time::{Duration, Instant}};
use virtual_clients_core::InspectionBudget;
#[cfg(feature="mc02-native-input")]
use crate::engine::native_clients_session::{
    AgentControlMode,AiInputFrame,AiLeaseBook,Mc02BatchOrigin,
    validate_ai_input_group,validate_ai_lease_slots,
    validate_ai_acquire_epoch,current_ai_filetime,
};

#[cfg(feature="mc02-native-input")]
fn ai_writer_mode_matches(slot:u32,requested:AgentControlMode,
                          actual:AgentControlMode)->bool {
    (1..=5).contains(&slot) && requested==actual &&
    (requested!=AgentControlMode::HumanPlusFourAi || slot!=1)
}

// Only the retained actor's already-bound original child may satisfy
// the one-time batch pin. The old one-shot path remains unpinned and
// keeps its existing 500-ms issue/bind/lease/window admission.
#[cfg(feature="mc02-native-input")]
fn legacy_mc02_batch_matches_original(
    pin:Option<Mc02BatchOrigin>,original:Option<(u32,u64,u32)>
)->bool {
    pin.is_none_or(|p|original==Some((p.pid,p.birth,p.session)))
}

// Only exact InvalidData from the original actor's early window/mode
// checks OR the original Channel's pre-v4 admission proves this
// attempted command never published input. A changed MC01 foreground
// or unavailable MC02 window can fail BEFORE Channel::write and must
// not strand a new legacy neutral-pending fence for a no-write request.
// Do NOT treat unknown Win32/HWND errors, an Unmap after publication,
// a lost transport ACK or other same-text error kinds as proof.
// Always preserve any older unresolved input fence.
#[cfg(feature="mc02-native-input")]
fn legacy_native_refused_before_publication(error:&io::Error)->bool {
    error.kind()==io::ErrorKind::InvalidData &&
        matches!(error.to_string().as_str(),
            // Exact original Sessions::mc02_input early returns.
            // These branches precede the only Channel::write call.
            "Native AI experiment is closing" |
            "AI profile is not running" |
            "AI profile was not opted in at launch" |
            "Verified AI game window unavailable" |
            "AI game window is not responsive" |
            "AI window is not playable" |
            "Human MC01 is not running" |
            "Hybrid human MC01 has an AI input channel" |
            "Verified human MC01 window unavailable" |
            "Human and AI game identities not independent" |
            "MC01 human foreground changed" |
            // Exact Channel pre-v4 publication rejects.
            "MC02 release target does not match owned process" |
            "MC02 input predates original channel binding" |
            "MC02 release request expired before native delivery" |
            "MC02 release predates newer active input" |
            "MC02 release predates newer neutral release" |
            "MC02 batch cleanup superseded by another native sender" |
            "Unbound, expired or invalid MC02 input command" |
            "MC02 input request expired before native delivery" |
            "MC02 active input predates neutral release" |
            "MC02 active input predates newer active input" |
            "MC02 press edge count exhausted until input epoch reset" |
            "MC02 last native sender unavailable")
}

// After partial-group failure, attempt a neutral release on EVERY slot
// which may have reached native I/O. A failed release must not suppress
// the remaining attempts or be hidden behind the original group error.
// Unattempted slots must never be touched by this controller cleanup.
#[cfg(feature="mc02-native-input")]
fn release_attempted_ai_group(
    attempted:&[u32], mut neutral:impl FnMut(u32)->io::Result<()>,
)->bool {
    let mut all_confirmed=true;
    for &slot in attempted {
        if neutral(slot).is_err() {all_confirmed=false;}
    }
    all_confirmed
}

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

    #[cfg(feature="mc02-native-input")]
    fn mc02_input(&self,slot:u32,keys:u32,dx:i32,dy:i32,
                  ttl_ms:u32,issued_at_ms:u64,
                  group_deadline_ms:Option<u64>,
                  cli_sender:Option<(u32,u64)>,
                  batch_cleanup:bool)->io::Result<()> {
        use crate::engine::workspace_surface::with_verified_native_window;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow,IsHungAppWindow,IsIconic,IsWindowVisible,
        };
        let action=crate::engine::native_clients_session::admit_native_input_frame(
            slot,keys,dx,dy,ttl_ms)?;
        let release_only=matches!(action,
            crate::engine::native_clients_session::NativeInputIntent::ReleaseAll);
        if self.exiting && !release_only {
            return Err(invalid("Native AI experiment is closing"));
        }
        let started=self.running.get((slot-1) as usize)
            .and_then(|entry|entry.as_ref())
            .ok_or_else(||invalid("AI profile is not running"))?;
        let child=&started.children[0];
        let (pid,birth)=child.identity()?;
        let input=child.mc02_input.as_ref()
            .ok_or_else(||invalid("AI profile was not opted in at launch"))?;
        let session=interactive_session(pid)?;
        if release_only {
            // Independent exact-owner release never depends on another slot
            // still running or on any window keeping foreground focus.
            return input.release_all(pid,birth,session,issued_at_ms,
                cli_sender,batch_cleanup);
        }
        let target=window(child,false)
            .ok_or_else(||invalid("Verified AI game window unavailable"))?;
        if !target.responding {
            return Err(invalid("AI game window is not responsive"));
        }
        let write_target=|ai_hwnd:usize,human_hwnd:usize| {
            if unsafe{IsWindowVisible(ai_hwnd as _)}==0 ||
               unsafe{IsIconic(ai_hwnd as _)}!=0 ||
               unsafe{IsHungAppWindow(ai_hwnd as _)}!=0 {
                return Err(invalid("AI window is not playable"));
            }
            input.write(pid,birth,session,keys,dx,dy,ttl_ms,issued_at_ms,
                human_hwnd,ai_hwnd,group_deadline_ms,cli_sender)
        };
        if input.mode()==AgentControlMode::FiveAi {
            // No artificial human foreground requirement in all-five mode.
            // The game-local hook still isolates physical keyboard/mouse
            // state and checks this child's exact root HWND on polling.
            return with_verified_native_window(&target,|ai_hwnd|
                write_target(ai_hwnd,0));
        }
        let human=self.running[0].as_ref()
            .ok_or_else(||invalid("Human MC01 is not running"))?;
        let human_child=&human.children[0];
        human_child.identity()?;
        if human_child.mc02_input.is_some() {
            return Err(invalid("Hybrid human MC01 has an AI input channel"));
        }
        let human_window=window(human_child,false)
            .ok_or_else(||invalid("Verified human MC01 window unavailable"))?;
        if !human_window.responding || human_window.pid==target.pid ||
           human_window.session_id!=target.session_id {
            return Err(invalid("Human and AI game identities not independent"));
        }
        with_verified_native_window(&human_window,|human_hwnd| {
            with_verified_native_window(&target,|ai_hwnd| {
                if human_hwnd==ai_hwnd ||
                   unsafe{GetForegroundWindow()}!=human_hwnd as _ {
                    return Err(invalid("MC01 human foreground changed"));
                }
                write_target(ai_hwnd,human_hwnd)
            })
        })
    }

    // The native ACTOR, not MCP or the ephemeral CLI, owns all slot
    // arbitration. No release bypass when a different live lease exists.
    #[cfg(feature="mc02-native-input")]
    fn ai_lease_target(&self,slot:u32)->io::Result<(u32,u32,u64)> {
        let index=slot.checked_sub(1).filter(|s|*s<5)
            .ok_or_else(||invalid("Invalid AI lease slot"))? as usize;
        let started=self.running[index].as_ref()
            .ok_or_else(||invalid("Original AI child is not running"))?;
        let child=&started.children[0];
        let (pid,birth)=child.identity()?;
        if child.mc02_input.is_none() {
            return Err(invalid("No opted-in native AI input channel"));
        }
        Ok((slot,pid,birth))
    }
    #[cfg(feature="mc02-native-input")]
    fn ai_lease_targets(&self,slots:&[u32])->io::Result<Vec<(u32,u32,u64)>> {
        slots.iter().map(|slot|self.ai_lease_target(*slot)).collect()
    }
    #[cfg(feature="mc02-native-input")]
    fn ai_lease_acquire(&mut self,slots:&[u32],owner:&str,
                        mode:AgentControlMode,epoch:&str,
                        neutral_only:bool)->io::Result<()> {
        use windows_sys::Win32::System::SystemInformation::GetTickCount64;
        validate_ai_lease_slots(slots,owner)?;
        let mut targets=Vec::with_capacity(slots.len());
        for &slot in slots {
            // The caller's selected mode is NOT sufficient authority:
            // every original child must have been launched with the SAME
            // mode before acquiring any slot. Never adopt a legacy hybrid
            // MC02 child into a five-AI batch, or vice versa.
            let target=self.ai_lease_target(slot)?;
            let channel=self.running[(slot-1) as usize].as_ref()
                .and_then(|game|game.children[0].mc02_input.as_ref())
                .ok_or_else(||invalid("Original AI channel disappeared"))?;
            if !ai_writer_mode_matches(slot,mode,channel.mode()) {
                return Err(invalid("AI writer mode conflicts with original game channel"));
            }
            targets.push(target);
        }
        // All slot/mode/identity checks completed. The transaction
        // epoch must postdate EVERY original child birth; a delayed
        // acquisition from before one slot was relaunched cannot bind
        // that replacement, even if its prior lease was forgotten.
        // No partial mutation or game write occurs on refusal.
        validate_ai_acquire_epoch(epoch,current_ai_filetime()?,&targets)?;
        self.ai_leases.acquire(&targets,owner,epoch,
            unsafe{GetTickCount64()},neutral_only)
    }
    #[cfg(feature="mc02-native-input")]
    fn ai_lease_release(&mut self,slots:&[u32],owner:&str,
                        epoch:&str)->io::Result<()> {
        use windows_sys::Win32::System::SystemInformation::GetTickCount64;
        validate_ai_lease_slots(slots,owner)?;
        let targets=self.ai_lease_targets(slots)?;
        self.ai_leases.release(&targets,owner,epoch,unsafe{GetTickCount64()})
    }
    #[cfg(feature="mc02-native-input")]
    fn ai_leased_input(&mut self,slot:u32,keys:u32,dx:i32,dy:i32,
        ttl_ms:u32,issued_at_ms:u64,owner:&str,epoch:&str,
        group_deadline_ms:Option<u64>)->io::Result<()> {
        use windows_sys::Win32::System::SystemInformation::GetTickCount64;
        let (_,pid,birth)=self.ai_lease_target(slot)?;
        // Before touching the original shared channel, conservatively
        // record POSSIBLY_ACTIVE. Native input I/O can return an error
        // after a packet was published; never let a prior neutral ACK
        // authorize lease release if this write becomes uncertain.
        self.ai_leases.begin_write(slot,owner,epoch,pid,birth,
            unsafe{GetTickCount64()},keys==0&&dx==0&&dy==0)?;
        self.mc02_input(slot,keys,dx,dy,ttl_ms,issued_at_ms,
            group_deadline_ms,None,false)?;
        // A new neutral ACK is the ONLY event that restores releasability.
        // This is native transport evidence, not Minecraft key-up proof.
        self.ai_leases.wrote(slot,owner,epoch,pid,birth,
            unsafe{GetTickCount64()},keys==0&&dx==0&&dy==0)
    }
    #[cfg(feature="mc02-native-input")]
    fn legacy_mc02_input(&mut self,slot:u32,keys:u32,dx:i32,dy:i32,
        ttl_ms:u32,issued_at_ms:u64,
        batch_origin:Option<Mc02BatchOrigin>,
        original_cli_sender:(u32,u64))->io::Result<()> {
        use windows_sys::Win32::System::SystemInformation::GetTickCount64;
        // Historical unleased Mc02Input is ONLY the hybrid MC02 pilot.
        // All 4/5-AI controls, including per-slot cleanup, now use
        // AiLeasedInput. Never let the compatibility wire bypass a lease
        // on MC01 or MC03-MC05, or touch a full-five AI mode channel.
        let channel=self.running.get(1).and_then(|entry|entry.as_ref())
            .and_then(|game|game.children[0].mc02_input.as_ref());
        if slot!=2 || !channel.is_some_and(|input|
            input.mode()==AgentControlMode::HumanPlusFourAi) {
            return Err(invalid("Legacy MC02 input is not bound to a hybrid child"));
        }
        // A legacy batch pinned its exact ORIGINAL child once. Do not
        // allow later pulse OR cleanup to adopt another Minecraft process
        // after reap/relaunch, even when its new input issue time passes
        // the replacement Channel's monotonic bind fence. This comparison
        // reads only the retained Channel tuple, not Win32 per heartbeat.
        // Refuse BEFORE legacy_begin_write or any v4 memory mutation.
        if !legacy_mc02_batch_matches_original(
            batch_origin,channel.and_then(|input|input.original_binding())) {
            return Err(invalid("MC02 batch original process changed before native input"));
        }
        let now=unsafe{GetTickCount64()};
        self.ai_leases.legacy_allowed(slot,now)?;
        // Native IO may fail AFTER v4 publication, so fence first.
        // Only a proven pre-publication original writer refusal may
        // restore the EXACT old fence, including old pending activity.
        // Never let a lost ACK, Unmap error or unknown return make an
        // uncertain native write appear neutral.
        let prior=self.ai_leases.legacy_begin_write(slot,now);
        if let Err(error)=self.mc02_input(slot,keys,dx,dy,ttl_ms,issued_at_ms,
            None,Some(original_cli_sender),batch_origin.is_some()) {
            if legacy_native_refused_before_publication(&error) {
                self.ai_leases.legacy_restore_unpublished(slot,prior);
            }
            return Err(error);
        }
        self.ai_leases.legacy_wrote(slot,unsafe{GetTickCount64()},
            keys==0&&dx==0&&dy==0);
        Ok(())
    }
    #[cfg(feature="mc02-native-input")]
    fn ai_input_group(&mut self,frames:&[AiInputFrame],
                      issued_at_ms:u64,deadline_at_ms:u64,
                      owner:&str,epoch:&str)->io::Result<()> {
        use windows_sys::Win32::System::SystemInformation::GetTickCount64;
        // Never trust a CLI that previously validated the group. An
        // authenticated client cannot bypass the same bounded constraints.
        validate_ai_input_group(frames,issued_at_ms,deadline_at_ms)?;
        // ALL source-owned channels must belong to the SAME current
        // writer before the FIRST member of the time group is written.
        // No partial input if another MCP process holds one of the slots.
        let targets=self.ai_lease_targets(&frames.iter()
            .map(|f|f.slot).collect::<Vec<_>>())?;
        for &(slot,pid,birth) in &targets {
            self.ai_leases.require(slot,owner,epoch,pid,birth,
                unsafe{GetTickCount64()})?;
        }
        let mut attempted=Vec::with_capacity(frames.len());
        let mut failure=None;
        for frame in frames {
            if unsafe{GetTickCount64()}>deadline_at_ms {
                failure=Some(invalid("AI timestamp group missed its native deadline"));
                break;
            }
            // Record BEFORE writing: a failed/uncertain operation might
            // have published state into this exact original game channel.
            attempted.push(frame.slot);
            if let Err(error)=self.ai_leased_input(frame.slot,frame.keys,
                frame.dx,frame.dy,frame.ttl_ms,issued_at_ms,owner,epoch,
                Some(deadline_at_ms)) {
                failure=Some(error);
                break;
            }
        }
        if let Some(error)=failure {
            // Best effort on EVERY possibly written slot of THIS group;
            // continue if one exact original channel cannot be neutralized.
            // Unattempted slots are not written by this actor cleanup.
            // The CLI separately releases earlier groups and MCP retains
            // uncertain cohorts until explicit recovery. Neither a failed
            // release nor a native TTL is proof of physical Minecraft key-up.
            let released=release_attempted_ai_group(&attempted,|slot|
                self.ai_leased_input(
                    slot,0,0,0,1,unsafe{GetTickCount64()},owner,epoch,
                    None)
            );
            if !released {
                // Prior error may describe an ordinary deadline/target
                // failure. An unconfirmed neutral is the stronger safety
                // outcome and must never be silently discarded.
                return Err(invalid("AI group neutral release unconfirmed"));
            }
            return Err(error);
        }
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

    fn inspect_storage(&self, slots: std::ops::RangeInclusive<u32>) -> io::Result<Vec<ProfileStorage>> {
        // One cooperative traversal budget for this *entire* read-only request,
        // not two seconds independently for each of five profiles and caches.
        // Native OS calls and the separate clear-confirmation inspection can
        // still block; this is not a hard latency guarantee or a new worker.
        let budget = InspectionBudget::new(Duration::from_secs(2));
        let observed = super::super::inspect_native_workspace()?.processes;
        // One report-local observation of the original retained handles.
        // Do not carry this result into a later request or mutation.
        let ownership = self.owned_data_processes()
            .map(|owned| all_observed_processes_owned(&owned, &observed));
        slots
            .map(|slot| {
                let store = self.store(slot)?;
                let path = directory_path(&store.private)?;
                // Only read-only reports share this admission observation.
                // Cache deletion, profile clear, launch and settings writes each
                // recheck their own authority at the moment of mutation.
                let admission = report_data_admission(
                    self.running[slot as usize - 1].is_none(), &ownership);
                let replay_admission = || {
                    admission.as_ref().map(|_| ()).map_err(|error| {
                        io::Error::new(error.kind(), error.to_string())
                    })
                };
                let full_clear = product_profiles::clear::inspect(store, replay_admission());
                let mut result = ProfileStorage {
                    slot,
                    bytes: 0,
                    files: 0,
                    partial: false,
                    disk: profile_storage::disk_at_verified_path(&path).ok(),
                    cache: {
                        let mut cache = profile_storage::inspect(&store.private, admission.is_ok(), budget);
                        if !cache.partial {
                            if let Err(error) = &admission { cache.issue = Some(error.to_string()); }
                        }
                        cache
                    },
                    download_cache: Some(profile_storage::inspect_cache(&store.private, profile_storage::CacheTarget::Download, admission.is_ok(), budget)),
                    // The inventory reads saved world *pack declaration metadata*.
                    // Do not inspect a selected profile while its own Minecraft
                    // session is active or the admission result is uncertain.
                    resource_packs: admission.is_ok().then(||
                        resource_pack_inventory::inspect(&store.private, budget)),
                    game_settings: Some(game_preset::inspect_settings(&store.private, replay_admission())),
                    full_clear: Some(full_clear),
                };
                // A pending full clear is exclusive with either cache action.
                // Preserve observed size/partial metadata; only revoke admission.
                if result.full_clear.as_ref().is_some_and(|report| report.pending) {
                    for cache in std::iter::once(&mut result.cache).chain(result.download_cache.iter_mut()) {
                        cache.can_clear = false;
                        if !cache.partial {
                            cache.issue = Some("Finish pending profile cleanup first".into());
                        }
                    }
                }
                profile_storage::scan_profile_size(&store.private, &mut result, budget);
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

    // Every profile mutation takes a fresh observation inside the sole actor.
    // Read-only storage reports must never substitute their cached admission.
    fn admit_profile_mutation(&mut self, slot: u32) -> io::Result<()> {
        self.reap();
        let observed = super::super::inspect_native_workspace()?.processes;
        self.data_admission(slot, &observed)
    }

    // Run only from the existing idle maintenance lane, after queued requests
    // have priority. Never delay a close reply or retry an uncertain deletion.
    fn maintain_storage(&mut self) {
        if self.exiting { self.pending_storage_maintenance = [false; 5]; return; }
        let Some(index) = self.pending_storage_maintenance.iter().position(|pending| *pending) else { return; };
        self.pending_storage_maintenance[index] = false;
        // Missing preferences preserve the established default; corrupt/unreadable preferences fail closed.
        let Ok(preferences)=crate::engine::manager_settings::load_storage_preferences() else {return;};
        if !preferences.has_maintenance() {return;}
        let slot = index as u32 + 1;
        let Ok(workspace) = super::super::inspect_native_workspace() else { return; };
        if self.data_admission(slot, &workspace.processes).is_err() { return; }
        let Ok(store) = self.store(slot) else { return; };
        #[cfg(feature="lazy-tester-package")]
        match product_profiles::clear::pending(store) {
            Ok(false) => {},
            _ => return,
        }
        // Best effort: a locked file, incomplete preflight, or lost admission
        // is not a game exit failure and never triggers a retry loop.
        // Do not retry or continue to another target after an uncertain failure.
        if preferences.auto_clear_web_cache && profile_storage::clear(&store.private,true).is_err() {return;}
        if preferences.auto_clear_download_cache && profile_storage::clear_cache_target(&store.private,profile_storage::CacheTarget::Download,true).is_err() {return;}
        if preferences.auto_clear_old_logs {let _=profile_storage::clear_old_logs(&store.private,true);}
    }

    fn selective_inventory(&mut self, slot: u32, category: SelectiveCategory) -> io::Result<SelectiveInventory> {
        self.admit_profile_mutation(slot)?;
        let store = self.store(slot)?;
        #[cfg(feature="lazy-tester-package")]
        if product_profiles::clear::pending(store)? {
            return Err(invalid("Finish pending profile cleanup first"));
        }
        let inventory = resource_pack_inventory::individual_inventory(&store.private, category,
            InspectionBudget::new(Duration::from_secs(2)))?;
        let items = if inventory.is_empty() {
            Vec::new()
        } else {
            let profile = directory_id(&store.private)?;
            inventory.into_iter().map(|item| {
                let identity = resource_pack_inventory::inventory_identity(&profile, slot, &item);
                SelectiveInventoryEntry { root: item.root.into(), name: item.name, identity }
            }).collect()
        };
        Ok(SelectiveInventory { slot, category, items })
    }

    fn resolve_selective_review(&mut self, slot: u32, selections: &[SelectiveSelection], budget: InspectionBudget) -> io::Result<resource_pack_inventory::SelectiveReview> {
        self.admit_profile_mutation(slot)?;
        let store = self.store(slot)?;
        #[cfg(feature="lazy-tester-package")]
        if product_profiles::clear::pending(store)? {
            return Err(invalid("Finish pending profile cleanup first"));
        }
        use resource_pack_inventory::ReviewSelection;
        let native: Vec<_> = selections.iter().map(|s| ReviewSelection {
            category: s.category,
            root: s.root.clone(), name: s.name.clone(),
        }).collect();
        let review = resource_pack_inventory::review_selection(&store.private, &native,
            budget)?;
        // Revalidate the retained profile once after the review, not once per item.
        // Every item still binds the same verified profile, slot and directory ID.
        let profile_identity = directory_id(&store.private)?;
        if !same_image(&profile_identity, &review.profile_identity) {
            return Err(invalid("Profile identity changed; refresh selection"));
        }
        for (selection, requested) in native.iter().zip(selections) {
            let item = review.items.iter().find(|i| i.category == selection.category &&
                i.root == selection.root.as_str() && i.name == selection.name)
                .ok_or_else(|| invalid("Selection no longer matches inventory"))?;
            if resource_pack_inventory::inventory_identity(&profile_identity, slot, item) != requested.identity {
                return Err(invalid("Inventory changed; refresh selection"));
            }
        }
        if budget.exhausted() { return Err(invalid("Selective review deadline exceeded")); }
        Ok(review)
    }

    fn review_selective_inventory(&mut self, slot: u32, selections: Vec<SelectiveSelection>) -> io::Result<SelectiveReviewReceipt> {
        let review = self.resolve_selective_review(slot, &selections, InspectionBudget::new(Duration::from_secs(2)))?;
        let confirmation_token = if review.warnings.is_empty() { Some(review.confirmation_token(slot)?) } else { None };
        Ok(SelectiveReviewReceipt { slot, bytes: review.bytes, files: review.files, items: selections, warnings: review.warnings, confirmation_token })
    }

    // Integrated through a fail-closed session request: mutation admission stays held until native
    // directory sharing/disposition evidence and wire consumer review exist.
    // The fingerprint is not authorization; repeat the original admission,
    // selection identity review and exclusive preflight before disposition.
    fn confirm_selected_worlds(&mut self, slot: u32, selections: &[SelectiveSelection],
        confirmation_token: &str) -> io::Result<()> {
        self.confirm_selected_data(slot,selections,confirmation_token,false)
    }

    fn confirm_selected_data(&mut self, slot: u32, selections: &[SelectiveSelection],
        confirmation_token: &str, packs: bool) -> io::Result<()> {
        if selections.is_empty() || selections.len() > 128 ||
            selections.iter().any(|s| (s.category != SelectiveCategory::Worlds) != packs) ||
            confirmation_token.len() != 64 ||
            !confirmation_token.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err(invalid("Invalid selected-data confirmation"));
        }
        let budget = InspectionBudget::new(Duration::from_secs(2));
        let review = self.resolve_selective_review(slot, selections, budget)?;
        if review.confirmation_token(slot)? != confirmation_token {
            return Err(invalid("Selected data changed; review again"));
        }
        self.admit_profile_mutation(slot)?;
        let store = self.store(slot)?;
        #[cfg(feature="lazy-tester-package")]
        if product_profiles::clear::pending(store)? {
            return Err(invalid("Finish pending profile cleanup first"));
        }
        let plan = if packs { profile_storage::preflight_selected_packs(&store.private,&review,budget)? }
            else { profile_storage::preflight_selected_worlds(&store.private,&review,budget)? };
        // Keep every deletion candidate pinned while checking native
        // ownership once more; never call a path-based fallback.
        let observed = super::super::inspect_native_workspace()?.processes;
        self.data_admission(slot, &observed)?;
        if budget.exhausted() { return Err(invalid("Selected data confirmation deadline exceeded")); }
        plan.dispose()
    }

    fn clear_cache(&mut self, slot: u32) -> io::Result<()> {
        self.admit_profile_mutation(slot)?;
        #[cfg(feature="lazy-tester-package")]
        if product_profiles::clear::pending(self.store(slot)?)? {
            return Err(invalid("Finish pending profile cleanup first"));
        }
        self.pending_storage_maintenance[slot as usize - 1] = false;
        profile_storage::clear(&self.store(slot)?.private, true)
    }

    fn change_game_settings(
        &mut self, slot: u32, change: fn(&File) -> io::Result<bool>, selected_storage: Option<bool>,
    ) -> io::Result<Vec<ProfileStorage>> {
        self.admit_profile_mutation(slot)?;
        change(&self.store(slot)?.private)?;
        self.inspect_storage(mutation_storage_slots(slot, selected_storage))
    }

    fn owned_data_processes(&self) -> io::Result<Vec<(u32, u64)>> {
        if self.exiting { return Err(invalid("Minecraft sessions are closing; wait before changing launcher data")); }
        self.running.iter().flatten().map(|started| {
            let child = &started.children[0];
            child.identity()?;
            Ok((child.pid, child.created))
        }).collect()
    }

    fn data_admission(&self, slot: u32, observed: &[OwnedProcess]) -> io::Result<()> {
        self.store(slot)?;
        // Mutations always obtain fresh retained-handle observations.
        let owned = self.owned_data_processes()?;
        require_data_idle(self.running[slot as usize - 1].is_none(), &owned, observed)
    }

    fn clear_data(&mut self,slot:u32,token:&str)->io::Result<()> {
        self.admit_profile_mutation(slot)?;
        let index=slot as usize-1;
        self.pending_storage_maintenance[index] = false;
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

// Reply scope only: mutation admission and the five-profile roster are unchanged.
fn mutation_storage_slots(slot: u32, selected_storage: Option<bool>) -> std::ops::RangeInclusive<u32> {
    if selected_storage == Some(true) { slot..=slot } else { 1..=5 }
}

// Reuse one observation for the read-only report, retaining per-slot admission
// and propagating a failed native identity check to every report entry.
fn all_observed_processes_owned(owned: &[(u32,u64)], observed: &[OwnedProcess]) -> bool {
    observed.iter().all(|process| {
        let birth = process.created_file_time.parse::<u64>().ok();
        owned.iter().any(|(pid,expected_birth)|
            process.pid == *pid && birth == Some(*expected_birth))
    })
}

fn report_data_admission(selected_idle: bool, ownership: &io::Result<bool>) -> io::Result<()> {
    let all_owned = *ownership.as_ref().map_err(|error| io::Error::new(error.kind(), error.to_string()))?;
    require_profile_data_idle(selected_idle, || all_owned)
}

fn require_data_idle(selected_idle: bool, owned: &[(u32,u64)], observed: &[OwnedProcess]) -> io::Result<()> {
    require_profile_data_idle(selected_idle, || all_observed_processes_owned(owned, observed))
}

fn require_profile_data_idle(selected_idle: bool, all_owned: impl FnOnce() -> bool) -> io::Result<()> {
    if !selected_idle { return Err(invalid("Close this Minecraft session normally before changing its launcher data.")); }
    if !all_owned() {
        return Err(invalid("An unmanaged Minecraft session is open. Close it normally before changing launcher data; managed sessions in other launchers may stay open."));
    }
    Ok(())
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
                    sessions.confirm_selected_worlds(slot, &selections, &confirmation_token).map_err(|error| error.to_string())?;
                    storage = Some(sessions.inspect_storage(1..=5).map_err(|error| error.to_string())?);
                }
                SessionRequest::ClearProfileDownloadCache { slot } => {
                    if !crate::engine::manager_settings::DOWNLOAD_CACHE_REVIEWED {
                        return Err("Download cache location is not native-verified".into());
                    }
                    sessions.admit_profile_mutation(slot).map_err(|error|error.to_string())?;
                    let store=sessions.store(slot).map_err(|error|error.to_string())?;
                    #[cfg(feature="lazy-tester-package")]
                    if product_profiles::clear::pending(store).map_err(|error|error.to_string())? {
                        return Err("Finish pending profile cleanup first".into());
                    }
                    profile_storage::clear_cache_target(&store.private,profile_storage::CacheTarget::Download,true).map_err(|error|error.to_string())?;
                    storage=Some(sessions.inspect_storage(1..=5).map_err(|error|error.to_string())?);
                }
                SessionRequest::ClearProfileCache { slot, selected_storage } => {
                    sessions
                        .clear_cache(slot)
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
