//! Slot-scoped MC01-MC05 experiment: controller-owned, pagefile-backed input
//! state consumed inside the already enrolled bootstrap-compat DLL.
//! No global SendInput, driver, new named pipe, second game controller, or
//! profile storage write. This module is absent unless its Cargo feature is
//! explicitly enabled. Minecraft Education compatibility is NOT_EXECUTED.
use super::*;
use crate::engine::native_clients_session::AgentControlMode;
use std::sync::atomic::{AtomicI64, AtomicU64, AtomicU8, Ordering};
use windows_sys::Win32::System::SystemInformation::GetTickCount64;

// The ABI v4 pressCounts packs EIGHT 8-bit rising-edge counters. Both
// Win32 asynchronous key polling and the buffered Raw Input reader compare
// byte fields for inequality, so a 256th increment within one live cursor
// epoch would silently wrap to the previously observed value.
// Refuse the ENTIRE new packet before publishing any shared-memory state
// rather than reset the epoch (which could discard other pending presses).
// This may coalesce earlier presses but never misreports a wrap as no edge.
fn checked_rising_press_counts(previous:u64,rising:u8)->Option<u64> {
    let mut counts=previous;
    for index in 0u32..8 {
        if rising & (1u8<<index)!=0 {
            let shift=index*8;
            let prior=((counts>>shift)&0xff) as u8;
            let next=prior.checked_add(1)?;
            counts=(counts & !(0xffu64<<shift)) |
                ((next as u64)<<shift);
        }
    }
    Some(counts)
}

// Group deadlines follow the same original Rust actor to the final
// native publication. Slow HWND checks/MapViewOfFile can outlast
// the actor's initial 150-ms admission; a 500-ms age guard alone
// is too weak. Standalone active input and neutral ReleaseAll retain
// their existing separate admissions.
fn active_group_write_in_time(now:u64,group_deadline_ms:Option<u64>)->bool {
    group_deadline_ms.is_none_or(|limit|now!=0 && limit!=0 && now<=limit)
}

// The original actor serializes per-slot writes and neutral releases,
// but separate CLI senders may arrive out of issue order. An older
// ReleaseAll must not republish an obsolete cursor epoch/revision after
// a newer neutral already completed, even with packet=0. Equal-tick
// neutral is intentionally allowed (release wins the active-input tie).
fn admit_native_release_order(
    issued_at_ms:u64,last_active_ms:u64,last_release_ms:u64
)->Result<(),&'static str> {
    if issued_at_ms<last_active_ms {
        return Err("MC02 release predates newer active input");
    }
    if issued_at_ms<last_release_ms {
        return Err("MC02 release predates newer neutral release");
    }
    Ok(())
}

// The old unleased CLI request contains only an MC02 slot, not the
// original child's PID/birth. If that child is reaped and relaunched
// during the 500-ms request lifetime, target equality against the
// NEW Channel is not proof that this command belongs to the new game.
// Use one monotonic per-child bind witness (NOT part of v4 shared ABI)
// to refuse older and equal-millisecond ambiguous issue times before
// reading/writing shared state. Same-tick independent writers AFTER
// binding still need source identity and remain unresolved.
fn issued_after_original_channel_bind(issued_at_ms:u64,bound_at_ms:u64)->bool {
    bound_at_ms!=0 && issued_at_ms>bound_at_ms
}

pub(super) const INPUT_ENV: &str = "MIVUBI_EDUCATION_MC02_INPUT_SECTION";
const MAGIC: u32 = 0x4c544932; // LTI2 (MC02 only)
const VERSION: u32 = 4;
const MAX_INPUT_REQUEST_AGE_MS:u64=500; // same-machine monotonic uptime


// C++ counterpart: mc02-virtual-input.cpp, private 96-byte ABI v4.
// packet: high 32 bits = GetTickCount() expiry; low 8 = active key mask;
// remaining low bits retain the last command sequence and signed dx/dy.
// Cursor movement is NOT read from that last-writer-wins packet.
// cursor_x/y accumulate all commands published in the same live epoch,
// cursor_epoch advances when the previous command expired, and
// cursor_revision is an even/odd transaction sequence for the child
// reader's consistent lock-free snapshot. No child-writable mapping.
// Commands and queued pointer movement expire after <=500 ms.
// The two window fields are authoritative HWNDs supplied while the
// controller pins MC01 and MC02; they cannot alter actual Windows focus.
// The final 64 bits hold 8 per-key rising-edge counters, incremented only
// for a NEW down transition (including mouse attack/use). This preserves
// short press/release actions across Win32 polling intervals without
// keeping a button physically held after its time limit.
#[repr(C, align(8))]
struct SharedInput {
    magic:u32, version:u32, bytes:u32, slot:u32, pid:u32, session:u32,
    birth:u64,
    packet:AtomicU64,
    cursor_revision:AtomicU64,
    cursor_x:AtomicI64,
    cursor_y:AtomicI64,
    cursor_epoch:AtomicU64,
    human_window:AtomicU64,
    ai_window:AtomicU64,
    press_counts:AtomicU64,
}
// This is a private cross-language ABI, not just a Rust struct. The C++
// game-side reader must interpret every v4 field at the same byte offset.
// A field reorder of unchanged total size must fail compilation on BOTH sides.
const _: () = {
    assert!(std::mem::size_of::<SharedInput>() == 96);
    assert!(std::mem::align_of::<SharedInput>() == 8);
    assert!(std::mem::size_of::<AtomicU64>() == 8);
    assert!(std::mem::size_of::<AtomicI64>() == 8);
    assert!(std::mem::offset_of!(SharedInput, magic) == 0);
    assert!(std::mem::offset_of!(SharedInput, version) == 4);
    assert!(std::mem::offset_of!(SharedInput, bytes) == 8);
    assert!(std::mem::offset_of!(SharedInput, slot) == 12);
    assert!(std::mem::offset_of!(SharedInput, pid) == 16);
    assert!(std::mem::offset_of!(SharedInput, session) == 20);
    assert!(std::mem::offset_of!(SharedInput, birth) == 24);
    assert!(std::mem::offset_of!(SharedInput, packet) == 32);
    assert!(std::mem::offset_of!(SharedInput, cursor_revision) == 40);
    assert!(std::mem::offset_of!(SharedInput, cursor_x) == 48);
    assert!(std::mem::offset_of!(SharedInput, cursor_y) == 56);
    assert!(std::mem::offset_of!(SharedInput, cursor_epoch) == 64);
    assert!(std::mem::offset_of!(SharedInput, human_window) == 72);
    assert!(std::mem::offset_of!(SharedInput, ai_window) == 80);
    assert!(std::mem::offset_of!(SharedInput, press_counts) == 88);
};

pub(super) struct Channel {
    section: Configuration,
    slot: u32,
    mode: AgentControlMode,
    target: Option<(u32,u64,u32)>,
    // Original child-only binding witness, not shared with game code.
    bound_at_ms:u64,
    sequence: AtomicU8,
    // Controller-local, NOT part of the inherited 96-byte game ABI.
    // An already-issued active pulse must never re-arm MC02 after a
    // later neutral ReleaseAll was admitted ahead of it over IPC.
    last_release_issued_at_ms: AtomicU64,
    // Same controller-side ordering fence for acknowledged active pulses.
    // Delayed packets and delayed old neutral requests must not overwrite
    // a newer active input written by another already-authorized CLI.
    last_active_issued_at_ms: AtomicU64,
}
impl Channel {
    pub(super) fn create(slot:u32, mode:Option<AgentControlMode>) -> io::Result<Option<Self>> {
        // Each authenticated explicit launch binds ONE original slot and
        // mode. Shared memory is never multiplexed between game processes.
        let Some(mode)=mode else {return Ok(None);};
        if !(1..=5).contains(&slot) ||
           (slot==1 && mode==AgentControlMode::HumanPlusFourAi) {
            return Err(invalid("Invalid experimental AI slot/mode"));
        }
        Ok(Some(Self {
            section:Configuration::create_sized(std::mem::size_of::<SharedInput>() as u32)?,
            slot,mode,
            target:None,bound_at_ms:0,
            sequence:AtomicU8::new(0),
            last_release_issued_at_ms:AtomicU64::new(0),
            last_active_issued_at_ms:AtomicU64::new(0),
        }))
    }
    pub(super) fn mode(&self)->AgentControlMode {self.mode}
    pub(super) fn original_binding(&self)->Option<(u32,u64,u32)> {
        self.target
    }
    pub(super) fn reader(&self)->HANDLE {
        self.section.readonly.as_raw_handle() as HANDLE
    }
    fn mapped(&self)->io::Result<windows_sys::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS> {
        let view=unsafe {MapViewOfFile(self.section.writable.as_raw_handle() as HANDLE,
            FILE_MAP_WRITE,0,0,std::mem::size_of::<SharedInput>())};
        if view.Value.is_null() {Err(io::Error::last_os_error())} else {Ok(view)}
    }
    pub(super) fn bind(&mut self, pid:u32, birth:u64, session:u32)->io::Result<()> {
        if pid==0 || birth==0 || session==0 || self.target.is_some() {
            return Err(invalid("MC02 input binding is not an unstarted owned client"));
        }
        let view=self.mapped()?;
        unsafe {
            std::ptr::write(view.Value.cast::<SharedInput>(), SharedInput {
                magic:MAGIC,version:VERSION,
                bytes:std::mem::size_of::<SharedInput>() as u32,
                slot:self.slot,pid,session,birth,packet:AtomicU64::new(0),
                cursor_revision:AtomicU64::new(0),
                cursor_x:AtomicI64::new(0),cursor_y:AtomicI64::new(0),
                cursor_epoch:AtomicU64::new(0),
                human_window:AtomicU64::new(0),
                ai_window:AtomicU64::new(0),
                press_counts:AtomicU64::new(0),
            });
        }
        if unsafe{UnmapViewOfFile(view)}==0 {return Err(io::Error::last_os_error());}
        // Inherited by the ONE already-created MC02 child. Never leak it to
        // unrelated children spawned later by the controller.
        if unsafe{SetHandleInformation(self.reader(),HANDLE_FLAG_INHERIT,0)}==0 {
            return Err(io::Error::last_os_error());
        }
        // One-time bind occurs with exact original process ownership
        // while the child has not resumed normal Minecraft gameplay.
        let bound_at_ms=unsafe{GetTickCount64()};
        if bound_at_ms==0 {
            return Err(invalid("Original MC02 input channel binding time unavailable"));
        }
        self.bound_at_ms=bound_at_ms;
        self.target=Some((pid,birth,session));
        Ok(())
    }
    // Release is a distinct state transition, not a one-millisecond
    // movement command. Only the retained, original MC02 process object
    // may authorize it. The human MC01 window does NOT need foreground:
    // losing focus must never block the attempt to clear MC02's controls.
    // No device, desktop cursor, external process or foreground is touched.
    pub(super) fn release_all(&self,pid:u32,birth:u64,session:u32,
                              issued_at_ms:u64)->io::Result<()> {
        if self.target!=Some((pid,birth,session)) {
            return Err(invalid("MC02 release target does not match owned process"));
        }
        if !issued_after_original_channel_bind(issued_at_ms,self.bound_at_ms) {
            return Err(invalid("MC02 input predates original channel binding"));
        }
        let view=self.mapped()?;
        let now=unsafe{GetTickCount64()};
        if issued_at_ms==0 || now.checked_sub(issued_at_ms)
            .is_none_or(|age|age>MAX_INPUT_REQUEST_AGE_MS) {
            let _=unsafe{UnmapViewOfFile(view)};
            return Err(invalid("MC02 release request expired before native delivery"));
        }
        // Compare BOTH original issue-time watermarks. An older neutral
        // must not reset cursor epoch/press counters after a newer accepted
        // neutral, even when no intervening active command was published.
        // Active and neutral timestamps tied in the same millisecond still
        // favor release, without authorizing a new input packet.
        if let Err(reason)=admit_native_release_order(
            issued_at_ms,
            self.last_active_issued_at_ms.load(Ordering::SeqCst),
            self.last_release_issued_at_ms.load(Ordering::SeqCst)) {
            let _=unsafe{UnmapViewOfFile(view)};
            return Err(invalid(reason));
        }
        let input=unsafe{&*view.Value.cast::<SharedInput>()};
        // Use the same v4 even/odd publication rule as movement writes.
        // Packet zero makes all virtual key/mouse polling return released;
        // a new cursor epoch prevents old look and rising-edge replay.
        input.cursor_revision.fetch_add(1,Ordering::SeqCst);
        input.human_window.store(0,Ordering::SeqCst);
        // IMPORTANT: Keep the original pinned MC02 root HWND until this
        // process/channel exits. The opt-in buffered Raw Input consumer
        // needs this window to identify its thread after packet=0 and
        // emit the outstanding key-up/mouse-up records. Erasing ai_window
        // here made explicit ReleaseAll suppress those release records.
        // It cannot authorize new input: packet=0, human_window=0, and
        // ownedGameWindow is never offered as spoofed focus without a
        // fresh authenticated packet. The HWND is still validated
        // process-locally by the existing C++ reader on every poll.
        input.cursor_epoch.fetch_add(1,Ordering::SeqCst);
        input.cursor_x.store(0,Ordering::SeqCst);
        input.cursor_y.store(0,Ordering::SeqCst);
        input.press_counts.store(0,Ordering::SeqCst);
        input.packet.store(0,Ordering::SeqCst);
        input.cursor_revision.fetch_add(1,Ordering::SeqCst);
        // The retained controller handles MC02 requests on one actor.
        // Its IPC connections can originate from DIFFERENT short-lived
        // CLI processes: after cancellation a newly issued neutral
        // request may overtake an earlier, delayed active request.
        // Preserve the greatest accepted same-machine GetTickCount64
        // release issue time. Any release older than an accepted active
        // OR neutral issue time was refused before altering the v4 state.
        // Never let a late pre-release active pulse resurrect controls.
        self.last_release_issued_at_ms.fetch_max(
            issued_at_ms,Ordering::SeqCst);
        if unsafe{UnmapViewOfFile(view)}==0 {return Err(io::Error::last_os_error());}
        Ok(())
    }
    pub(super) fn write(&self,pid:u32,birth:u64,session:u32,
                        keys:u32,dx:i32,dy:i32,ttl_ms:u32,
                        issued_at_ms:u64,human_hwnd:usize,ai_hwnd:usize,
                        group_deadline_ms:Option<u64>)->io::Result<()> {
        // Final source-side defense: no direct caller may bypass the
        // admission rules or turn a release request into an active write.
        let action=crate::engine::native_clients_session::admit_native_input_frame(
            self.slot,keys,dx,dy,ttl_ms)?;
        if !matches!(action,
                crate::engine::native_clients_session::NativeInputIntent::Active)
            || ai_hwnd==0 ||
            (self.mode==AgentControlMode::HumanPlusFourAi &&
                (human_hwnd==0 || human_hwnd==ai_hwnd)) ||
            (self.mode==AgentControlMode::FiveAi && human_hwnd!=0) ||
            self.target!=Some((pid,birth,session)) {
            return Err(invalid("Unbound, expired or invalid MC02 input command"));
        }
        if !issued_after_original_channel_bind(issued_at_ms,self.bound_at_ms) {
            return Err(invalid("MC02 input predates original channel binding"));
        }
        let view=self.mapped()?;
        // The only authoritative expiry check is immediately before the
        // native memory write. A pending authenticated IPC may complete
        // after the requesting Codex/CLI process has already timed out.
        let now=unsafe{GetTickCount64()};
        if issued_at_ms==0 || now.checked_sub(issued_at_ms)
            .is_none_or(|age|age>MAX_INPUT_REQUEST_AGE_MS) {
            let _=unsafe{UnmapViewOfFile(view)};
            return Err(invalid("MC02 input request expired before native delivery"));
        }
        // The original controller's single actor serializes Channel
        // writes/releases. Distinct CLI IPC connections need not arrive
        // in issue order. Any active pulse issued BEFORE (or on the same
        // tick as) the latest completed neutral release is stale, even
        // if its standalone 500-ms age check still passes.
        if issued_at_ms<=self.last_release_issued_at_ms.load(Ordering::SeqCst) {
            let _=unsafe{UnmapViewOfFile(view)};
            return Err(invalid("MC02 active input predates neutral release"));
        }
        // The original single actor processes IPC in arrival order, not
        // necessarily GetTickCount64 issue order. Refuse an older active
        // pulse after a newer active write: otherwise stale key masks
        // and one-shot camera deltas can replace valid current controls.
        // Equal-tick active writes retain actor order; any equal-tick
        // ReleaseAll above still takes precedence over all of them.
        if issued_at_ms<self.last_active_issued_at_ms.load(Ordering::SeqCst) {
            let _=unsafe{UnmapViewOfFile(view)};
            return Err(invalid("MC02 active input predates newer active input"));
        }
        let expires=(now as u32).wrapping_add(ttl_ms);
        // Do not advance the command sequence before the shared-state
        // edge counters have passed their all-or-nothing overflow gate.
        // The authenticated native controller is the ONLY writer. A fresh
        // input packet may overwrite the held button state, but it must NOT
        // erase cursor deltas that MC02 has not polled yet. Cursor totals
        // reset after expiration; the epoch prevents a later fresh command
        // from replaying pointer movement that was missed while idle.
        let input=unsafe{&*view.Value.cast::<SharedInput>()};
        let old=input.packet.load(Ordering::SeqCst);
        let ahead=((old>>32) as u32).wrapping_sub(now as u32) as i32;
        let new_epoch=old==0 || ahead<=0 || ahead>500;
        let epoch=input.cursor_epoch.load(Ordering::SeqCst)
            .wrapping_add(u64::from(new_epoch));
        let x=if new_epoch {dx as i64} else {
            input.cursor_x.load(Ordering::SeqCst).saturating_add(dx as i64)
        };
        let y=if new_epoch {dy as i64} else {
            input.cursor_y.load(Ordering::SeqCst).saturating_add(dy as i64)
        };
        // GetAsyncKeyState bit 0 signals a press since its last read.
        // A last-writer-wins key mask loses short press/release sequences
        // when MC02 reads more slowly than the native command producer.
        // Keep per-action rising edges in 8-bit independent fields.
        // The reader compares current and previously consumed byte
        // values; wrap from 255->0 would alias zero even after 256
        // presses without a game poll. Never manufacture a new epoch
        // while old buttons/camera might still be pending. Fail closed
        // before any shared state or command sequence changes.
        let previous_keys=if new_epoch {0u8} else {old as u8};
        let rising=(keys as u8) & !previous_keys;
        let prior_counts=if new_epoch {0u64} else {
            input.press_counts.load(Ordering::SeqCst)
        };
        let Some(counts)=checked_rising_press_counts(prior_counts,rising) else {
            let _=unsafe{UnmapViewOfFile(view)};
            return Err(invalid("MC02 press edge count exhausted until input epoch reset"));
        };
        // The actor's pre-group check happened before potentially delayed
        // native identity, HWND and view mapping work. Fail closed at the
        // LAST original-writer boundary, before changing its sequence or
        // any v4 shared-memory field. Group neutral cleanup uses ReleaseAll
        // and is never blocked by this active-only deadline.
        if !active_group_write_in_time(unsafe{GetTickCount64()},group_deadline_ms) {
            let _=unsafe{UnmapViewOfFile(view)};
            return Err(invalid("AI timestamp group missed its native deadline"));
        }
        let sequence=self.sequence.fetch_add(1,Ordering::Relaxed).wrapping_add(1);
        let word=((expires as u64)<<32) | ((dy as i8 as u8 as u64)<<24)
             | ((dx as i8 as u8 as u64)<<16)
             | ((sequence as u64)<<8) | keys as u64;
        // Odd revision = in-flight transaction. Publish keys/expiry BEFORE
        // returning to even; MC02 only accepts matching even snapshots.
        input.cursor_revision.fetch_add(1,Ordering::SeqCst);
        input.cursor_x.store(x,Ordering::SeqCst);
        input.cursor_y.store(y,Ordering::SeqCst);
        input.cursor_epoch.store(epoch,Ordering::SeqCst);
        // Publish the exact HWNDs from the retained controller BEFORE the
        // live packet: the optional MC02-only focus-read hooks must refuse
        // to spoof focus if Windows no longer shows MC01 as foreground.
        input.human_window.store(human_hwnd as u64,Ordering::SeqCst);
        input.ai_window.store(ai_hwnd as u64,Ordering::SeqCst);
        // Covered by the same even/odd revision used for cursor totals.
        input.press_counts.store(counts,Ordering::SeqCst);
        input.packet.store(word,Ordering::SeqCst);
        input.cursor_revision.fetch_add(1,Ordering::SeqCst);
        // The same retained session actor completes the shared-state
        // transaction before accepting the next IPC request. A maximum
        // watermark rejects any delayed older active packet thereafter,
        // without changing the game-visible 96-byte shared-state ABI.
        self.last_active_issued_at_ms.fetch_max(issued_at_ms,Ordering::SeqCst);
        if unsafe{UnmapViewOfFile(view)}==0 {return Err(io::Error::last_os_error());}
        Ok(())
    }
}

#[cfg(test)]
mod rising_edge_counter_tests {
    use super::{checked_rising_press_counts,active_group_write_in_time,
        admit_native_release_order,issued_after_original_channel_bind};

    #[test]
    fn delayed_legacy_mc02_command_cannot_adopt_relaunched_child() {
        // Pure authored source fixture, no mapped section or live game.
        // Applies to a per-original-game channel in either 4/5-AI mode.
        for cohort in [&[2,3,4,5][..],&[1,2,3,4,5][..]] {
            for _slot in cohort {
                assert!(!issued_after_original_channel_bind(1999,2000),
                    "an old still-fresh CLI command cannot bind new child");
                assert!(!issued_after_original_channel_bind(2000,2000),
                    "same-tick child origin remains ambiguous; refuse");
                assert!(issued_after_original_channel_bind(2001,2000));
                assert!(!issued_after_original_channel_bind(2001,0));
                assert!(!issued_after_original_channel_bind(0,2000));
            }
        }
    }

    #[test]
    fn stale_release_all_does_not_republish_a_newer_neutral_epoch() {
        // Pure authored Rust fixture, no mapped game channel or Windows call.
        // One per-slot channel is used in either original 4/5-AI cohort.
        for cohort in [&[2,3,4,5][..],&[1,2,3,4,5][..]] {
            for _slot in cohort {
                assert_eq!(admit_native_release_order(100,0,101),
                    Err("MC02 release predates newer neutral release"));
                assert_eq!(admit_native_release_order(100,101,0),
                    Err("MC02 release predates newer active input"));
                assert_eq!(admit_native_release_order(100,102,103),
                    Err("MC02 release predates newer active input"));
                // Repeating the exact same issue tick remains a safe
                // neutral tie, and a later explicit recovery can proceed.
                assert_eq!(admit_native_release_order(101,101,101),Ok(()));
                assert_eq!(admit_native_release_order(102,101,101),Ok(()));
            }
        }
    }

    #[test]
    fn per_slot_group_deadline_is_rechecked_before_v4_publication() {
        // Pure authored negative fixture, no Windows/game/IPC execution.
        // The original actor may check at 1100, but the final native
        // writer must refuse a slot arriving after the 1150 cutoff.
        for cohort in [&[2,3,4,5][..],&[1,2,3,4,5][..]] {
            for _slot in cohort {
                assert!(active_group_write_in_time(1150,Some(1150)));
                assert!(!active_group_write_in_time(1151,Some(1150)));
                assert!(!active_group_write_in_time(1150,Some(0)));
            }
        }
        // Direct opt-in writes use the existing per-request TTL and epoch
        // admission; ReleaseAll bypasses this active-only writer entirely.
        assert!(active_group_write_in_time(1151,None));
    }

    #[test]
    fn per_key_counter_rejects_wrap_before_any_partial_publish() {
        let mut counts=0u64;
        for _ in 0..254 {
            counts=checked_rising_press_counts(counts,1).unwrap();
        }
        assert_eq!(counts,254);
        assert_eq!(checked_rising_press_counts(counts,1),Some(255));
        let max=checked_rising_press_counts(counts,1).unwrap();
        assert_eq!(checked_rising_press_counts(max,1),None,
            "256th rising edge must not alias the old zero value");
        assert_eq!(checked_rising_press_counts(max,0),Some(max),
            "no new edge leaves an exhausted field unchanged");
        assert_eq!(checked_rising_press_counts(0,1),Some(1),
            "only a genuine new input epoch restarts the counter");
    }

    #[test]
    fn multiple_buttons_cannot_publish_partially_on_one_overflow() {
        let first_at_limit=255u64 | (7u64<<8) | (41u64<<16);
        assert_eq!(checked_rising_press_counts(first_at_limit,0b0000_0010),
            Some(255u64 | (8u64<<8) | (41u64<<16)));
        assert_eq!(checked_rising_press_counts(first_at_limit,0b0000_0001),None);
        assert_eq!(checked_rising_press_counts(first_at_limit,0b0000_0011),None,
            "a later bit cannot be incremented when another bit must fail");
        // All eight independent fields work, including attack and use.
        assert_eq!(checked_rising_press_counts(0,0xff),
            Some(0x0101_0101_0101_0101));
        assert_eq!(checked_rising_press_counts(u64::MAX,0xff),None);
        assert_eq!(checked_rising_press_counts(u64::MAX,0),Some(u64::MAX));
    }
}
