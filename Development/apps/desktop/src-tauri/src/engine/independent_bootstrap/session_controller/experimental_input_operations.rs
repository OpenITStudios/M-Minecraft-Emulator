//! Feature-gated input admission and delivery on the one retained Sessions owner.
//! No second lease book, worker or process authority.
use super::{Sessions, window};
use super::super::{interactive_session, invalid};
use crate::engine::native_clients_session::{
    AgentControlMode, AiInputFrame, Mc02BatchOrigin, validate_ai_input_group,
    validate_ai_lease_slots, validate_ai_acquire_epoch, current_ai_filetime,
};
use std::io;

#[cfg(feature="mc02-native-input")]
pub(super) fn ai_writer_mode_matches(slot:u32,requested:AgentControlMode,
                          actual:AgentControlMode)->bool {
    (1..=5).contains(&slot) && requested==actual &&
    (requested!=AgentControlMode::HumanPlusFourAi || slot!=1)
}

// Only the retained actor's already-bound original child may satisfy
// the one-time batch pin. The old one-shot path remains unpinned and
// keeps its existing 500-ms issue/bind/lease/window admission.
#[cfg(feature="mc02-native-input")]
pub(super) fn legacy_mc02_batch_matches_original(
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
pub(super) fn legacy_native_refused_before_publication(error:&io::Error)->bool {
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
pub(super) fn release_attempted_ai_group(
    attempted:&[u32], mut neutral:impl FnMut(u32)->io::Result<()>,
)->bool {
    let mut all_confirmed=true;
    for &slot in attempted {
        if neutral(slot).is_err() {all_confirmed=false;}
    }
    all_confirmed
}

impl Sessions {
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
    pub(super) fn ai_lease_acquire(&mut self,slots:&[u32],owner:&str,
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
    pub(super) fn ai_lease_release(&mut self,slots:&[u32],owner:&str,
                        epoch:&str)->io::Result<()> {
        use windows_sys::Win32::System::SystemInformation::GetTickCount64;
        validate_ai_lease_slots(slots,owner)?;
        let targets=self.ai_lease_targets(slots)?;
        self.ai_leases.release(&targets,owner,epoch,unsafe{GetTickCount64()})
    }
    #[cfg(feature="mc02-native-input")]
    pub(super) fn ai_leased_input(&mut self,slot:u32,keys:u32,dx:i32,dy:i32,
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
    pub(super) fn legacy_mc02_input(&mut self,slot:u32,keys:u32,dx:i32,dy:i32,
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
    pub(super) fn ai_input_group(&mut self,frames:&[AiInputFrame],
                      issued_at_ms:u64,deadline_at_ms:u64,
                      owner:&str,epoch:&str)->io::Result<()> {
        use windows_sys::Win32::System::SystemInformation::GetTickCount64;
        // Never trust a CLI that previously validated the group. An
        // authenticated client cannot bypass the same bounded constraints.
        validate_ai_input_group(frames,issued_at_ms,deadline_at_ms)?;
        // ALL source-owned channels must belong to the SAME current
        // writer before the FIRST member of the time group is written.
        // No partial input if another MCP process holds one of the slots.
        // The validated group has at most five frames. Observe every
        // original target before checking the first lease, as before.
        let mut targets=[(0,0,0);5];
        for (target,frame) in targets.iter_mut().zip(frames) {
            *target=self.ai_lease_target(frame.slot)?;
        }
        for &(slot,pid,birth) in &targets[..frames.len()] {
            self.ai_leases.require(slot,owner,epoch,pid,birth,
                unsafe{GetTickCount64()})?;
        }
        let mut attempted=[0u32;5];
        let mut attempted_len=0;
        let mut failure=None;
        for frame in frames {
            if unsafe{GetTickCount64()}>deadline_at_ms {
                failure=Some(invalid("AI timestamp group missed its native deadline"));
                break;
            }
            // Record BEFORE writing: a failed/uncertain operation might
            // have published state into this exact original game channel.
            attempted[attempted_len]=frame.slot;
            attempted_len+=1;
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
            let released=release_attempted_ai_group(&attempted[..attempted_len],|slot|
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
}
