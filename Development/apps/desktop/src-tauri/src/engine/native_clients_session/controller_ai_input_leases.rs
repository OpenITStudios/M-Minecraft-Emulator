//! Experimental AI input contract and controller-owned writer leases.
//! No worker, input delivery or lifecycle authority is created here.
use serde::{Deserialize, Serialize};
use std::io;

// Correlation-only original child witness for the existing legacy
// Mc02Input request. Not an authority delegated to the CLI: the
// retained actor compares it to its own original process/channel
// before the legacy lease book or game state is modified.
#[cfg(feature="mc02-native-input")]
#[derive(Clone,Copy,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Mc02BatchOrigin {
    pub(crate) pid:u32,
    pub(crate) birth:u64,
    pub(crate) session:u32,
}
#[cfg(feature="mc02-native-input")]
impl Mc02BatchOrigin {
    pub(crate) fn valid(self)->bool {
        self.pid!=0 && self.birth!=0 && self.session!=0
    }
}

#[cfg(feature="mc02-native-input")]
#[derive(Clone,Copy,Debug,Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct AiInputFrame {
    pub slot:u32,pub keys:u32,pub dx:i32,pub dy:i32,pub ttl_ms:u32,
}

// Validate the COMPLETE private group before any controller-side mutation.
// Require strictly ascending, distinct native slots and fixed experimental
// pulse TTL. This cannot authorize a child that lacks its opted-in channel.
#[cfg(feature="mc02-native-input")]
pub(crate) fn validate_ai_input_group(
    frames:&[AiInputFrame],issued_at_ms:u64,deadline_at_ms:u64,
)->io::Result<()> {
    if frames.is_empty() || frames.len()>5 || issued_at_ms==0 ||
       deadline_at_ms<issued_at_ms ||
       deadline_at_ms-issued_at_ms>150 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "Invalid experimental AI timestamp group"));
    }
    let mut previous_slot=0;
    for frame in frames {
        if frame.slot<=previous_slot || frame.ttl_ms!=350 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "AI group must have distinct ascending slots and fixed TTL"));
        }
        admit_native_input_frame(frame.slot,frame.keys,frame.dx,frame.dy,
            frame.ttl_ms)?;
        previous_slot=frame.slot;
    }
    Ok(())
}

// A bounded, controller-owned writer lease for the feature-gated AI input
// experiment. The 32-hex owner ID is an ephemeral correlation ID supplied
// only via the local MCP child environment: NOT an OS authentication
// credential. Same-user malicious processes can imitate it; this prevents
// accidental independent MCP writers, not hostile same-user tampering.
// Native PID/birth/mode checks and per-process game input gates still apply.
#[cfg(feature="mc02-native-input")]
pub(crate) const AI_WRITER_LEASE_MS:u64=10_000;
#[cfg(feature="mc02-native-input")]
pub(crate) fn valid_ai_writer_id(value:&str)->bool {
    value.len()==32 && value.bytes().all(|b|b.is_ascii_hexdigit()&&
        !b.is_ascii_uppercase())
}

// P31 temporal binding prevents a delayed pre-relaunch acquisition from
// adopting a newer original Minecraft process. Birth is the retained
// Windows GetProcessTimes FILETIME, never a window title or MCP claim.
// The epoch is a 32-hex private correlator, not a signed credential;
// this is not resistance to a malicious same-user process.
#[cfg(feature="mc02-native-input")]
pub(crate) fn current_ai_filetime()->io::Result<u64> {
    const UNIX_TO_FILETIME_100NS:u128=116_444_736_000_000_000;
    let elapsed=std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_|io::Error::other("AI epoch clock unavailable"))?;
    u64::try_from(UNIX_TO_FILETIME_100NS+
        elapsed.as_nanos()/100)
        .map_err(|_|io::Error::other("AI epoch clock out of range"))
}
#[cfg(feature="mc02-native-input")]
fn ai_epoch_issued_filetime(epoch:&str)->Option<u64> {
    // The original e1 transaction ID carries a 100-ns FILETIME prefix.
    // It is an issuance/replay fence, not same-user authentication.
    if !valid_ai_writer_id(epoch)||!epoch.ends_with("e1") {return None;}
    u64::from_str_radix(&epoch[..16],16).ok().filter(|at|*at!=0)
}
#[cfg(feature="mc02-native-input")]
pub(crate) fn validate_ai_acquire_epoch(
    epoch:&str,now_filetime:u64,targets:&[(u32,u32,u64)]
)->io::Result<()> {
    // CLI acquisition is at most six seconds, allow a bounded ten-second
    // window. One-second clock lead tolerance is fail-closed on larger drift.
    const MAX_AGE_100NS:u64=100_000_000;
    const MAX_FUTURE_100NS:u64=10_000_000;
    let issued=ai_epoch_issued_filetime(epoch);
    if now_filetime==0||targets.is_empty()||
       !issued.is_some_and(|at|at!=0 &&
           at<=now_filetime.saturating_add(MAX_FUTURE_100NS) &&
           now_filetime<=at.saturating_add(MAX_AGE_100NS) &&
           targets.iter().all(|(_,pid,birth)|
               *pid!=0&&*birth!=0&&*birth<=at)) {
        return Err(io::Error::new(io::ErrorKind::PermissionDenied,
            "AI transaction predates original game or is stale"));
    }
    Ok(())
}

#[cfg(feature="mc02-native-input")]
pub(crate) fn validate_ai_lease_slots(slots:&[u32],owner_id:&str)->io::Result<()> {
    validate_ai_lease_slot_order(slots.iter().copied(),owner_id)
}

#[cfg(feature="mc02-native-input")]
fn validate_ai_lease_slot_order(
    slots:impl ExactSizeIterator<Item=u32>,owner_id:&str,
)->io::Result<()> {
    let invalid=||io::Error::new(io::ErrorKind::InvalidInput,
        "Invalid experimental AI writer lease");
    if !valid_ai_writer_id(owner_id)||!(1..=5).contains(&slots.len()) {
        return Err(invalid());
    }
    let mut previous=0;
    for slot in slots {
        if slot<=previous || slot>5 {
            return Err(invalid());
        }
        previous=slot;
    }
    Ok(())
}
#[cfg(feature="mc02-native-input")]
#[derive(Clone,Debug)]
struct AiWriterClaim {
    owner_id:String,lease_epoch:String,pid:u32,birth:u64,
    until_ms:u64,neutral:bool,neutral_only:bool,
}
#[cfg(feature="mc02-native-input")]
#[derive(Debug)]
pub(crate) struct AiLeaseBook {
    claims:[Option<AiWriterClaim>;5],
    // Retain issued-time high-water even after a neutral lease releases.
    // Only reaping the original child resets its corresponding watermark.
    last_admitted_epoch_filetime:[u64;5],
    legacy_until_ms:[u64;5],
    // Packet TTL only bounds virtual polling; it cannot attest that the
    // opted-in game consumed a neutral release. Keep this per-slot
    // unresolved legacy input bit until a confirmed native neutral write
    // or original-process reap, just like the modern writer book.
    legacy_neutral_pending:[bool;5],
}
#[cfg(feature="mc02-native-input")]
impl Default for AiLeaseBook {
    fn default()->Self {
        Self{claims:std::array::from_fn(|_|None),
            last_admitted_epoch_filetime:[0;5],legacy_until_ms:[0;5],
            legacy_neutral_pending:[false;5]}
    }
}
#[cfg(feature="mc02-native-input")]
impl AiLeaseBook {
    // The ephemeral lease_epoch is a per-TRANSACTION fencing value.
    // It is not OS authentication. A delayed CLI from a previous
    // lease/release (even with the same MCP writer ID) must never
    // touch a newly launched child after the original slot was reaped.
    pub(crate) fn acquire(&mut self,targets:&[(u32,u32,u64)],
        owner:&str,epoch:&str,now:u64,neutral_only:bool)->io::Result<()> {
        validate_ai_lease_slot_order(targets.iter().map(|(slot,_,_)|*slot),owner)?;
        let issued=ai_epoch_issued_filetime(epoch).ok_or_else(||
            io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid AI transaction fencing epoch"))?;
        // All selected original slots must pass BEFORE any claim changes.
        // A released transaction must not re-acquire with a replayed,
        // earlier, or same-FILETIME different-nonce epoch. The actor has
        // already verified the current time and every child process birth.
        if targets.iter().any(|(slot,_,_)|
            issued<=self.last_admitted_epoch_filetime[(*slot-1) as usize]) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI transaction predates original game or is stale"));
        }
        if now==0 || targets.iter().any(|(slot,pid,birth)| {
            *pid==0||*birth==0 ||
            self.legacy_until_ms[(*slot-1) as usize]>now ||
            self.legacy_neutral_pending[(*slot-1) as usize] ||
            self.claims[(*slot-1) as usize].as_ref().is_some_and(|held|
                // A live action claim must not be silently replaced by
                // another acquire from the SAME MCP writer, even while
                // the previous transaction is still neutral. Delayed
                // acquires can arrive out of order over separate CLI
                // channels; replacing the epoch would invalidate the
                // original claim and could authorize the stale caller.
                // Normal actions therefore require the prior live
                // claim to be explicitly released. Expired but ACTIVE
                // claims still require fresh recovery-only neutral.
                (!neutral_only && (!held.neutral || held.until_ms>now)) ||
                // Emergency recovery may replace its OWN live claim,
                // including uncertain input; foreign live owners and
                // a different original process remain protected.
                (held.until_ms>now && (
                    held.owner_id!=owner || held.pid!=*pid ||
                    held.birth!=*birth
                )))
        }) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI writer lease conflict; existing game input preserved"));
        }
        for &(slot,pid,birth) in targets {
            let index=(slot-1) as usize;
            // An expired claim is NEVER evidence of native neutral
            // delivery. Every recovery-only acquisition starts
            // non-neutral, even for an unclaimed slot, until a fresh
            // exact-owner neutral write has actually succeeded.
            // A normal claim can inherit only prior confirmed neutral
            // state; pending active input always requires recovery.
            let neutral=!neutral_only &&
                self.claims[index].as_ref().is_none_or(|held|held.neutral);
            self.claims[index]=Some(AiWriterClaim {
                owner_id:owner.into(),lease_epoch:epoch.into(),pid,birth,
                until_ms:now.saturating_add(AI_WRITER_LEASE_MS),
                neutral,neutral_only,
            });
            self.last_admitted_epoch_filetime[index]=issued;
        }
        Ok(())
    }
    pub(crate) fn require(&self,slot:u32,owner:&str,epoch:&str,
        pid:u32,birth:u64,now:u64)->io::Result<()> {
        if !valid_ai_writer_id(owner)||!valid_ai_writer_id(epoch)||
           !(1..=5).contains(&slot)||now==0||
           !self.claims[(slot-1) as usize].as_ref().is_some_and(|held|
             held.owner_id==owner && held.lease_epoch==epoch &&
             held.pid==pid && held.birth==birth && held.until_ms>now) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI slot is not owned by this live writer lease"));
        }
        Ok(())
    }
    pub(crate) fn begin_write(&mut self,slot:u32,owner:&str,epoch:&str,
        pid:u32,birth:u64,now:u64,neutral:bool)->io::Result<()> {
        self.require(slot,owner,epoch,pid,birth,now)?;
        let held=self.claims[(slot-1) as usize].as_mut().unwrap();
        if held.neutral_only && !neutral {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "AI neutral-only recovery lease refuses active game input"));
        }
        // Native I/O may fail AFTER publishing a shared packet. Keep
        // the original lease non-neutral until a NEW neutral ACK.
        held.neutral=false;
        held.until_ms=now.saturating_add(AI_WRITER_LEASE_MS);
        Ok(())
    }
    pub(crate) fn wrote(&mut self,slot:u32,owner:&str,epoch:&str,
        pid:u32,birth:u64,now:u64,neutral:bool)->io::Result<()> {
        self.require(slot,owner,epoch,pid,birth,now)?;
        let held=self.claims[(slot-1) as usize].as_mut().unwrap();
        held.until_ms=now.saturating_add(AI_WRITER_LEASE_MS);
        held.neutral=neutral;
        Ok(())
    }
    pub(crate) fn release(&mut self,targets:&[(u32,u32,u64)],
        owner:&str,epoch:&str,now:u64)->io::Result<()> {
        validate_ai_lease_slot_order(targets.iter().map(|(slot,_,_)|*slot),owner)?;
        for &(slot,pid,birth) in targets {
            self.require(slot,owner,epoch,pid,birth,now)?;
            if !self.claims[(slot-1) as usize].as_ref().unwrap().neutral {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                    "AI slot must be neutral before relinquishing writer lease"));
            }
        }
        for &(slot,_,_) in targets {self.claims[(slot-1) as usize]=None;}
        Ok(())
    }
    pub(crate) fn legacy_allowed(&self,slot:u32,now:u64)->io::Result<()> {
        if !(1..=5).contains(&slot)||now==0 ||
           self.claims[(slot-1) as usize].as_ref().is_some_and(|c|
               c.until_ms>now || !c.neutral) {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "An AI slot is held by a different native writer"));
        }
        Ok(())
    }
    // The legacy MC02 native call can fail AFTER publishing a packet.
    // Mark that slot possibly active BEFORE entering native I/O so a
    // concurrent/following 4/5-AI lease cannot adopt it while the input
    // channel's bounded (<=500ms) TTL may still be alive. The retained
    // session actor calls legacy_allowed first and serializes both steps.
    pub(crate) fn legacy_begin_write(&mut self,slot:u32,now:u64)->(u64,bool) {
        // Preserve the EXACT previous legacy fence, including an older
        // unresolved input. A later native rejection proven to occur
        // BEFORE v4 publication may restore it; a lost transport reply
        // or failure after packet publication MUST retain POSSIBLY_ACTIVE.
        let index=(slot-1) as usize; // Original actor checked slot first.
        let prior=(self.legacy_until_ms[index],
                   self.legacy_neutral_pending[index]);
        self.legacy_neutral_pending[index]=true;
        self.legacy_until_ms[index]=now.saturating_add(500);
        prior
    }
    // Only the original serialized actor may invoke this immediately
    // after a DEFINITIVE pre-publication native writer refusal. Never
    // use it for Unmap errors, missing IPC ACKs, timeout or unknown IO.
    pub(crate) fn legacy_restore_unpublished(
        &mut self,slot:u32,prior:(u64,bool)
    ) {
        let index=(slot-1) as usize;
        self.legacy_until_ms[index]=prior.0;
        self.legacy_neutral_pending[index]=prior.1;
    }
    pub(crate) fn legacy_wrote(&mut self,slot:u32,now:u64,neutral:bool) {
        self.claims[(slot-1) as usize]=None;
        // Called only AFTER the existing exact-process shared-channel
        // write returns success. This confirms native transport only;
        // Minecraft key-up and Raw Input polling remain unverified.
        self.legacy_neutral_pending[(slot-1) as usize]=!neutral;
        self.legacy_until_ms[(slot-1) as usize]=
            if neutral{0}else{now.saturating_add(500)};
    }
    pub(crate) fn forget_slot(&mut self,slot:u32) {
        if (1..=5).contains(&slot) {
            self.claims[(slot-1) as usize]=None;
            self.last_admitted_epoch_filetime[(slot-1) as usize]=0;
            self.legacy_until_ms[(slot-1) as usize]=0;
            self.legacy_neutral_pending[(slot-1) as usize]=false;
        }
    }
}

// Private experimental launch mode: pinned in the original game owner.
// Hybrid reserves MC01 for a human. FiveAi treats MC01 as an opted-in AI;
// input frames cannot change a process's launch mode or game identity.
#[cfg(feature="mc02-native-input")]
#[derive(Clone,Copy,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="kebab-case")]
pub enum AgentControlMode { HumanPlusFourAi, FiveAi }

// The historical Mc02Input wire name is retained, but exact native
// slot+channel binding is the final dispatch authority, not this range check.
// Normal product builds have NO native AI input.
#[cfg(feature="mc02-native-input")]
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(crate) enum NativeInputIntent { Active, ReleaseAll }

#[cfg(feature="mc02-native-input")]
pub(crate) fn admit_native_input_frame(
    slot:u32,keys:u32,dx:i32,dy:i32,ttl_ms:u32,
)->io::Result<NativeInputIntent> {
    // This is ONLY range/value validation, not authorization: native
    // controller requires an opted-in channel on the exact original child.
    if !(1..=5).contains(&slot) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "No authorized native input backend for requested slot"));
    }
    // Reject impossible opposing movement controls even when a caller
    // bypasses the MCP's action planner or sends its own IPC payload.
    // Bits: W=1, S=2, A=4, D=8, jump=16, sneak=32, attack=64, use=128.
    if keys>255 || (keys & 0x03)==0x03 || (keys & 0x0c)==0x0c ||
       !(-127..=127).contains(&dx) || !(-127..=127).contains(&dy) ||
       !(1..=500).contains(&ttl_ms) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,
            "Invalid experimental native input frame"));
    }
    Ok(if keys==0 && dx==0 && dy==0 {
        NativeInputIntent::ReleaseAll
    } else {
        NativeInputIntent::Active
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature="mc02-native-input")]
    fn synthetic_ai_epoch(issued:u64)->String {
        format!("{issued:016x}{}e1","a".repeat(14))
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn temporal_epoch_fences_relaunched_child_before_native_acquire() {
        // Pure contract fixture: 100-ns Windows FILETIME witness. Does not
        // create any game process, IPC server, or input channel.
        let at=134_000_000_000_000_000u64;
        let token=|issued:u64|format!("{issued:016x}{}e1","a".repeat(14));
        let old=token(at);
        let fresh=token(at+2_000_000);
        let four=vec![
            (2,202,at-10_000_000),(3,203,at-10_000_000),
            (4,204,at-10_000_000),(5,205,at-10_000_000),
        ];
        let mut five=vec![(1,201,at-10_000_000)];
        five.extend(four.iter().copied());
        assert!(validate_ai_acquire_epoch(&old,at+1_000_000,&four).is_ok());
        assert!(validate_ai_acquire_epoch(&old,at+1_000_000,&five).is_ok());

        // One slot replaced after the old MCP request was minted.
        // Atomic multi-slot acquisition must fail for BOTH 4-AI and 5-AI.
        let replaced_four=vec![
            (2,902,at+1_000_000),(3,203,at-10_000_000),
            (4,204,at-10_000_000),(5,205,at-10_000_000),
        ];
        let mut replaced_five=vec![(1,201,at-10_000_000)];
        replaced_five.extend(replaced_four.iter().copied());
        assert!(validate_ai_acquire_epoch(&old,at+2_300_000,
            &replaced_four).is_err());
        assert!(validate_ai_acquire_epoch(&old,at+2_300_000,
            &replaced_five).is_err());
        assert!(validate_ai_acquire_epoch(&fresh,at+2_300_000,
            &replaced_four).is_ok());
        assert!(validate_ai_acquire_epoch(&fresh,at+2_300_000,
            &replaced_five).is_ok());

        // The original retained lease book still fences stale input and
        // release requests by exact owner, epoch, pid and process birth.
        let owner="b".repeat(32);
        let mut book=AiLeaseBook::default();
        book.acquire(&four,&owner,&old,1000,false).unwrap();
        book.forget_slot(2);
        assert!(book.require(2,&owner,&old,902,at+1_000_000,1001).is_err());
        // Replacing MC02 does NOT authorize stomping still-live
        // leases of MC03-05 merely because they are neutral and carry
        // the same MCP writer ID. Those old claims must be released
        // under their original epoch before a new group is claimed.
        assert!(book.acquire(&replaced_four,&owner,&fresh,1002,false).is_err());
        for &(slot,pid,birth) in &four[1..] {
            assert!(book.require(slot,&owner,&old,pid,birth,1002).is_ok());
        }
        book.release(&four[1..],&owner,&old,1003).unwrap();
        book.acquire(&replaced_four,&owner,&fresh,1004,false).unwrap();
        assert!(book.require(2,&owner,&old,902,at+1_000_000,1005).is_err());
        assert!(book.require(2,&owner,&fresh,902,at+1_000_000,1005).is_ok());
        assert!(book.release(&[(2,902,at+1_000_000)],&owner,&old,1006).is_err());

        // Disabled, malformed, stale and far-future epochs fail closed.
        for invalid in [
            "1".repeat(32),
            "A".repeat(32),
            format!("{:016x}{}e2",at,"a".repeat(14)),
            token(at-120_000_000),
            token(at+20_000_000),
        ] {
            assert!(validate_ai_acquire_epoch(&invalid,at+1_000_000,
                &four).is_err());
        }
        assert!(validate_ai_acquire_epoch(&fresh,at+2_300_000,&[]).is_err());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn native_ai_lease_replays_and_old_epochs_refuse_after_release() {
        let owner="a".repeat(32);
        let at=134_000_000_000_000_000u64;
        let token=|offset:u64|synthetic_ai_epoch(at+offset);
        let old=token(10_000);
        let earlier=token(15_000);
        let later=token(30_000);
        let recovery=token(40_000);
        let four=[(2,202,at-10_000_000),(3,203,at-10_000_000),
                  (4,204,at-10_000_000),(5,205,at-10_000_000)];
        let five=[(1,201,at-10_000_000),(2,202,at-10_000_000),
                  (3,203,at-10_000_000),(4,204,at-10_000_000),
                  (5,205,at-10_000_000)];
        for cohort in [&four[..],&five[..]] {
            let mut book=AiLeaseBook::default();
            book.acquire(cohort,&owner,&old,1000,false).unwrap();
            book.release(cohort,&owner,&old,1001).unwrap();
            assert!(book.acquire(cohort,&owner,&old,1002,false).is_err(),
                "released epoch cannot reacquire its old original slot");
            let same_time_other_nonce=
                format!("{}{}e1",&old[..16],"b".repeat(14));
            assert!(book.acquire(cohort,&owner,&same_time_other_nonce,
                1002,false).is_err(),"same issued time must fail closed");
            book.acquire(cohort,&owner,&later,1003,false).unwrap();
            book.release(cohort,&owner,&later,1004).unwrap();
            assert!(book.acquire(cohort,&owner,&earlier,1005,false).is_err(),
                "older transaction cannot arrive after newer lease release");
            // Denying any member must not burn a timestamp on its peers.
            let mut partial=AiLeaseBook::default();
            partial.acquire(&cohort[..1],&owner,&later,1000,false).unwrap();
            assert!(partial.acquire(cohort,&owner,&earlier,1001,false).is_err());
            partial.acquire(&cohort[1..],&owner,&earlier,1002,false).unwrap();
            // Recovery-only remains a separately admitted NEW epoch;
            // it still cannot release without exact native neutral ACK.
            book.acquire(cohort,&owner,&recovery,1006,true).unwrap();
            assert!(book.release(cohort,&owner,&recovery,1007).is_err());
            for &(slot,pid,birth) in cohort {
                book.begin_write(slot,&owner,&recovery,pid,birth,1008,true)
                    .unwrap();
                book.wrote(slot,&owner,&recovery,pid,birth,1009,true).unwrap();
            }
            book.release(cohort,&owner,&recovery,1010).unwrap();
            // Only actual original-child reap forgets per-slot watermark.
            let &(slot,_,_)=cohort.first().unwrap();
            let new_birth=at+35_000_000;
            let next=token(40_000_000);
            book.forget_slot(slot);
            assert!(validate_ai_acquire_epoch(&old,at+40_500_000,
                &[(slot,999,new_birth)]).is_err(),
                "the existing controller still fences pre-birth epochs");
            assert!(validate_ai_acquire_epoch(&next,at+40_500_000,
                &[(slot,999,new_birth)]).is_ok());
            book.acquire(&[(slot,999,new_birth)],&owner,&next,1011,false)
                .unwrap();
        }
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn four_five_ai_normal_acquire_cannot_steal_live_same_writer_lease() {
        // Pure contract only: no native IPC, processes, Minecraft or DLL.
        // The old CLI may be delayed while a newer same-writer request
        // is in flight. Neutrality is not permission to replace its
        // live transaction epoch or extend a replayed acquisition.
        let writer="a".repeat(32);
        let first=synthetic_ai_epoch(1);
        let second=synthetic_ai_epoch(2);
        let recovery=synthetic_ai_epoch(3);
        let four=[(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let five=[(1,201,2201),(2,202,2202),(3,203,2203),
                  (4,204,2204),(5,205,2205)];
        for cohort in [&four[..],&five[..]] {
            let mut book=AiLeaseBook::default();
            book.acquire(cohort,&writer,&first,1000,false).unwrap();
            for stale in [&first,&second] {
                assert!(book.acquire(cohort,&writer,stale,1001,false).is_err(),
                    "a live neutral lease cannot be silently renewed or replaced");
            }
            for &(slot,pid,birth) in cohort {
                assert!(book.require(slot,&writer,&first,pid,birth,1002).is_ok());
                assert!(book.require(slot,&writer,&second,pid,birth,1002).is_err());
            }
            book.release(cohort,&writer,&first,1003).unwrap();
            book.acquire(cohort,&writer,&second,1004,false).unwrap();
            let &(slot,pid,birth)=cohort.first().unwrap();
            book.begin_write(slot,&writer,&second,pid,birth,1005,false).unwrap();
            assert!(book.acquire(cohort,&writer,&first,1006,false).is_err(),
                "possible active input blocks every normal same-writer takeover");
            // The existing SAME-owner emergency route may supersede an
            // uncertain active claim only with a new recovery-only
            // epoch, and every slot must then receive a native neutral
            // acknowledgement before its claim can be released.
            book.acquire(cohort,&writer,&recovery,1007,true).unwrap();
            assert!(book.require(slot,&writer,&second,pid,birth,1008).is_err());
            assert!(book.release(cohort,&writer,&recovery,1008).is_err());
            for &(slot,pid,birth) in cohort {
                book.begin_write(slot,&writer,&recovery,pid,birth,1009,true)
                    .unwrap();
                book.wrote(slot,&writer,&recovery,pid,birth,1010,true).unwrap();
            }
            book.release(cohort,&writer,&recovery,1011).unwrap();
        }
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn fenced_writer_conflict_is_atomic_and_process_bound() {
        // Pure Rust source fixture; no Windows, IPC, or Minecraft execution.
        let a="a".repeat(32);
        let b="b".repeat(32);
        let e1=synthetic_ai_epoch(1);
        let e2=synthetic_ai_epoch(2);
        let e3=synthetic_ai_epoch(3);
        let e4=synthetic_ai_epoch(4);
        let four=vec![(2,102,1002),(3,103,1003),(4,104,1004),(5,105,1005)];
        let mut five=vec![(1,101,1001)];
        five.extend(four.iter().copied());
        let mut book=AiLeaseBook::default();
        assert!(book.acquire(&four,&a,&e1,1000,false).is_ok());
        for &(slot,pid,birth) in &four {
            assert!(book.require(slot,&a,&e1,pid,birth,1001).is_ok());
            assert!(book.require(slot,&a,&e2,pid,birth,1001).is_err());
            assert!(book.require(slot,&b,&e1,pid,birth,1001).is_err());
            assert!(book.require(slot,&a,&e1,pid,birth+1,1001).is_err());
        }
        // Another writer cannot claim idle MC01 while peers conflict.
        assert!(book.acquire(&five,&b,&e2,1002,false).is_err());
        assert!(book.require(1,&b,&e2,101,1001,1002).is_err());
        book.begin_write(2,&a,&e1,102,1002,1010,false).unwrap();
        assert!(book.release(&four,&a,&e1,1011).is_err());
        assert!(book.acquire(&[(2,102,1002)],&a,&e2,1012,false).is_err());
        // Emergency neutral takeover fences the old same-writer epoch.
        assert!(book.acquire(&four,&a,&e2,1020,true).is_ok());
        assert!(book.require(2,&a,&e1,102,1002,1021).is_err());
        assert!(book.release(&four,&a,&e2,1022).is_err());
        assert!(book.begin_write(2,&a,&e2,102,1002,1023,false).is_err());
        book.begin_write(2,&a,&e2,102,1002,1024,true).unwrap();
        book.wrote(2,&a,&e2,102,1002,1025,true).unwrap();
        // Recovery-only never assumes the other three are neutral just
        // because they were neutral on a previous epoch. Each gets an
        // explicit native neutral ACK before the group can be released.
        assert!(book.release(&four,&a,&e2,1025).is_err());
        for &(slot,pid,birth) in &four[1..] {
            book.begin_write(slot,&a,&e2,pid,birth,1025,true).unwrap();
            book.wrote(slot,&a,&e2,pid,birth,1025,true).unwrap();
        }
        assert!(book.release(&four,&a,&e2,1026).is_ok());
        assert!(book.acquire(&five,&b,&e1,1030,false).is_err(),
            "old issue time cannot follow a newer completed recovery");
        assert!(book.acquire(&five,&b,&e3,1030,false).is_ok());
        assert!(book.require(1,&b,&e3,101,1001,1031).is_ok());
        // Process exit invalidates an original claim. A freshly claimed
        // replacement uses a DIFFERENT epoch, not a reused owner ID.
        assert!(book.acquire(&[(2,999,1002)],&b,&e2,1032,false).is_err());
        book.forget_slot(2);
        assert!(book.acquire(&[(2,999,9002)],&b,&e2,1033,false).is_ok());
        assert!(book.require(2,&b,&e1,102,1002,1034).is_err());
        assert!(book.require(2,&b,&e2,999,9002,1034).is_ok());
        assert!(book.release(&[(2,999,9002)],&b,&e1,1035).is_err());
        assert!(book.begin_write(2,&b,&e1,999,9002,1035,true).is_err());
        assert!(book.require(2,&b,&e2,999,9002,
            1033+AI_WRITER_LEASE_MS).is_err());
        assert!(book.acquire(&[(2,999,9002)],&a,&e1,
            1034+AI_WRITER_LEASE_MS,false).is_err());
        assert!(book.acquire(&[(2,999,9002)],&a,&e4,
            1034+AI_WRITER_LEASE_MS,false).is_ok());
    }


    #[cfg(feature="mc02-native-input")]
    #[test]
    fn expired_active_writer_requires_exact_native_neutral_recovery_in_four_five_modes() {
        // Pure retained native lease-book fixture; no Windows game process,
        // input mapping, client CLI or Minecraft is created or executed.
        let first="a".repeat(32);
        let second="b".repeat(32);
        let e1=synthetic_ai_epoch(1);
        let e2=synthetic_ai_epoch(2);
        let e3=synthetic_ai_epoch(3);
        let four=vec![(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let mut five=vec![(1,201,2201)];
        five.extend(four.iter().copied());
        for cohort in [&four,&five] {
            let mut book=AiLeaseBook::default();
            book.acquire(cohort,&first,&e1,1000,false).unwrap();
            // Exercise the REAL legacy MC02 collision in both cohort
            // shapes, not merely the second element of the slot list.
            let &(slot,pid,birth)=cohort.iter()
                .find(|(slot,_,_)|*slot==2).unwrap();
            book.begin_write(slot,&first,&e1,pid,birth,1010,false).unwrap();
            // The first input may have reached the game. There is no
            // matching native neutral ACK, not even after 10s expiry.
            let expired=1010+AI_WRITER_LEASE_MS+2;
            assert!(book.require(slot,&first,&e1,pid,birth,expired).is_err());
            assert!(book.acquire(cohort,&second,&e2,expired,false).is_err(),
                "a single unconfirmed slot denies the whole 4/5 cohort");
            assert!(book.acquire(&[(slot,pid,birth)],&first,&e2,expired,false).is_err(),
                "same MCP writer cannot convert expiry into neutral proof");
            assert!(book.acquire(&[(slot,pid,birth)],&second,&e2,expired,false).is_err());
            assert!(book.legacy_allowed(slot,expired).is_err(),
                "expired active MC02 claim may not unlock legacy input");
            let healthy=cohort.iter().copied()
                .filter(|(s,_,_)|*s!=slot).collect::<Vec<_>>();
            assert!(book.acquire(&healthy,&second,&e2,expired,false).is_ok(),
                "other original clients may be claimed independently");
            assert!(book.release(&healthy,&second,&e2,expired+1).is_ok(),
                "only prior neutral-ACKed normal claims may release");

            book.acquire(&[(slot,pid,birth)],&second,&e2,expired,true).unwrap();
            assert!(book.require(slot,&first,&e1,pid,birth,expired+1).is_err(),
                "old epoch is fenced after neutral-only takeover");
            assert!(book.release(&[(slot,pid,birth)],&second,&e2,expired+1).is_err(),
                "expiry and recovery acquire are not a neutral ACK");
            assert!(book.begin_write(slot,&second,&e2,pid,birth,expired+1,false).is_err(),
                "recovery-only writer may never send active input");
            book.begin_write(slot,&second,&e2,pid,birth,expired+2,true).unwrap();
            assert!(book.release(&[(slot,pid,birth)],&second,&e2,expired+3).is_err(),
                "a possibly published neutral write still needs its ACK");
            book.wrote(slot,&second,&e2,pid,birth,expired+3,true).unwrap();
            book.release(&[(slot,pid,birth)],&second,&e2,expired+4).unwrap();
            assert!(book.acquire(&[(slot,pid,birth)],&second,&e3,expired+5,false).is_ok(),
                "new action may start only after verified native neutral and release");
        }
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn recovery_only_new_claim_never_starts_preconfirmed_neutral() {
        let writer="b".repeat(32);
        let epoch=synthetic_ai_epoch(2);
        let target=[(3,303,3303)];
        let mut book=AiLeaseBook::default();
        book.acquire(&target,&writer,&epoch,1000,true).unwrap();
        assert!(book.release(&target,&writer,&epoch,1001).is_err(),
            "a first-time recovery must obtain its own native neutral ACK");
        book.begin_write(3,&writer,&epoch,303,3303,1002,true).unwrap();
        assert!(book.release(&target,&writer,&epoch,1003).is_err());
        book.wrote(3,&writer,&epoch,303,3303,1003,true).unwrap();
        book.release(&target,&writer,&epoch,1004).unwrap();

        // Only a confirmed original process exit can forget the stale
        // claim without writing to a possibly replaced game process.
        let mut exited=AiLeaseBook::default();
        exited.acquire(&target,&writer,&epoch,2000,false).unwrap();
        exited.begin_write(3,&writer,&epoch,303,3303,2001,false).unwrap();
        exited.forget_slot(3);
        assert!(exited.acquire(&[(3,909,9909)],&writer,
            &synthetic_ai_epoch(3),2002,false).is_ok());
        assert!(exited.require(3,&writer,&epoch,909,9909,2003).is_err());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn interrupted_native_write_remains_non_neutral_and_isolated() {
        let a="a".repeat(32);
        let e1=synthetic_ai_epoch(1);
        let e2=synthetic_ai_epoch(2);
        let targets=vec![(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let mut book=AiLeaseBook::default();
        book.acquire(&targets,&a,&e1,1000,false).unwrap();
        book.begin_write(3,&a,&e1,203,2203,1010,false).unwrap();
        assert!(book.release(&targets,&a,&e1,1011).is_err());
        let peers=vec![(2,202,2202),(4,204,2204),(5,205,2205)];
        book.release(&peers,&a,&e1,1020).unwrap();
        assert!(book.require(3,&a,&e1,203,2203,1021).is_ok());
        book.begin_write(3,&a,&e1,203,2203,1030,true).unwrap();
        assert!(book.release(&[(3,203,2203)],&a,&e1,1031).is_err());
        book.wrote(3,&a,&e1,203,2203,1032,true).unwrap();
        book.release(&[(3,203,2203)],&a,&e1,1033).unwrap();
        book.acquire(&[(3,203,2203)],&a,&e2,2000,true).unwrap();
        book.begin_write(3,&a,&e2,203,2203,2001,true).unwrap();
        assert!(book.require(3,&a,&e2,203,2203,
            2001+AI_WRITER_LEASE_MS).is_err());
        assert!(book.acquire(&[(3,203,2203)],&"b".repeat(32),&e1,
            2002+AI_WRITER_LEASE_MS,true).is_err());
        assert!(book.acquire(&[(3,203,2203)],&"b".repeat(32),
            &synthetic_ai_epoch(3),2002+AI_WRITER_LEASE_MS,true).is_ok());
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn fencing_protocol_rejects_invalid_ids_and_legacy_overlap() {
        let a="a".repeat(32);
        let e1=synthetic_ai_epoch(1);
        let e2=synthetic_ai_epoch(2);
        let e3=synthetic_ai_epoch(3);
        let mut book=AiLeaseBook::default();
        assert!(validate_ai_lease_slots(&[2,3,4,5],&a).is_ok());
        for invalid in ["short".to_owned(),"A".repeat(32),"x".repeat(32)] {
            assert!(validate_ai_lease_slots(&[2],&invalid).is_err());
            assert!(book.acquire(&[(2,202,2002)],&a,&invalid,1000,false).is_err());
        }
        for invalid_slots in [vec![],vec![2,2],vec![3,2],vec![0],vec![6],
            vec![1,2,3,4,5,6]] {
            assert!(validate_ai_lease_slots(&invalid_slots,&a).is_err());
        }
        book.legacy_wrote(2,1000,false);
        assert!(book.acquire(&[(2,202,2002)],&a,&e1,1300,false).is_err());
        assert!(book.acquire(&[(2,202,2002)],&a,&e1,1501,false).is_err(),
            "expired legacy pulse without native neutral ACK stays fenced");
        // Simulate the original controller's separately acknowledged
        // legacy neutral write. Only then may a modern writer acquire.
        book.legacy_wrote(2,1502,true);
        assert!(book.acquire(&[(2,202,2002)],&a,&e1,1503,false).is_ok());
        assert!(book.legacy_allowed(2,1504).is_err());
        book.begin_write(2,&a,&e1,202,2002,1505,false).unwrap();
        assert!(book.acquire(&[(2,202,2002)],&a,&e2,1506,true).is_ok());
        assert!(book.require(2,&a,&e1,202,2002,1507).is_err());
        book.begin_write(2,&a,&e2,202,2002,1508,true).unwrap();
        book.wrote(2,&a,&e2,202,2002,1509,true).unwrap();
        book.release(&[(2,202,2002)],&a,&e2,1510).unwrap();
        book.legacy_wrote(2,1511,true);
        assert!(book.acquire(&[(2,202,2002)],&a,&e1,1512,false).is_err(),
            "legacy neutral cannot reauthorize an older AI epoch");
        assert!(book.acquire(&[(2,202,2002)],&a,&e3,1512,false).is_ok());
    }


    #[cfg(feature="mc02-native-input")]
    #[test]
    fn definite_legacy_no_write_restores_prior_fence_for_four_and_five_ai() {
        // Pure author-time lease-book fixture, no Windows native writer,
        // Minecraft, IPC or source test execution. Restoration must NOT
        // convert an already unresolved older input into neutral state.
        let owner="a".repeat(32);
        let four=[(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let five=[(1,201,2201),(2,202,2202),(3,203,2203),
                  (4,204,2204),(5,205,2205)];
        for cohort in [&four[..],&five[..]] {
            let mut book=AiLeaseBook::default();
            book.legacy_allowed(2,1000).unwrap();
            let before=book.legacy_begin_write(2,1000);
            assert_eq!(before,(0,false));
            // No packet was published by the refused attempt:
            // a clean original channel stays available to new leases.
            book.legacy_restore_unpublished(2,before);
            let first=synthetic_ai_epoch(1);
            book.acquire(cohort,&owner,&first,1001,false).unwrap();
            book.release(cohort,&owner,&first,1002).unwrap();

            // Prior acknowledged active input still has pending key-up
            // even after 350/500-ms TTL expires. Restoring a failed
            // newer request cannot erase this earlier uncertainty.
            book.legacy_wrote(2,1010,false);
            book.legacy_allowed(2,1511).unwrap();
            let prior_active=book.legacy_begin_write(2,1511);
            assert_eq!(prior_active,(1510,true));
            book.legacy_restore_unpublished(2,prior_active);
            let newer=synthetic_ai_epoch(2);
            assert!(book.acquire(cohort,&owner,&newer,10_000,false).is_err(),
                "previous unresolved legacy input is still quarantined");
            // If native failed AFTER publishing or its ACK was lost,
            // do NOT call restore: the latest conservative fence stays.
            let uncertain=book.legacy_begin_write(2,10_001);
            assert_eq!(uncertain,(1510,true));
            assert!(book.acquire(cohort,&owner,&newer,11_000,false).is_err());
            book.legacy_wrote(2,11_001,true); // fresh native neutral ACK
            book.acquire(cohort,&owner,&newer,11_002,false).unwrap();
            book.release(cohort,&owner,&newer,11_003).unwrap();
        }
    }

    #[cfg(feature="mc02-native-input")]
    #[test]
    fn interrupted_legacy_mc02_send_fences_multi_writer_until_neutral_ack_or_reap() {
        // Pure lease-book simulation: legacy MC02 packet may already be
        // published when the native API reports an error. No game process,
        // Windows channel, MCP, or test runner is started by this fixture.
        let a="a".repeat(32);
        let e=synthetic_ai_epoch(1);
        let fresh=synthetic_ai_epoch(2);
        let four=vec![(2,102,1002),(3,103,1003),(4,104,1004),(5,105,1005)];
        let five=vec![(1,101,1001),(2,102,1002),
                      (3,103,1003),(4,104,1004),(5,105,1005)];
        let mut book=AiLeaseBook::default();
        // legacy_allowed() admitted the exact original hybrid MC02 slot.
        book.legacy_allowed(2,1000).unwrap();
        book.legacy_begin_write(2,1000);
        // No legacy_wrote(): simulate a native failure after possible publish.
        // The all-slot claim is atomic; it never captures healthy peers.
        assert!(book.acquire(&four,&a,&e,1001,false).is_err());
        assert!(book.acquire(&five,&a,&e,1002,false).is_err());
        assert!(book.require(3,&a,&e,103,1003,1002).is_err());
        // Other selected slots can still be acquired/released separately.
        book.acquire(&[(3,103,1003)],&a,&e,1003,false).unwrap();
        book.release(&[(3,103,1003)],&a,&e,1004).unwrap();
        assert!(book.acquire(&four,&a,&e,1499,false).is_err());
        assert!(book.acquire(&four,&a,&e,1500,false).is_err(),
            "500ms packet TTL is not a native neutral acknowledgement");
        assert!(book.acquire(&five,&a,&e,10_000,false).is_err(),
            "a late takeover still requires native neutral ACK");
        book.legacy_allowed(2,10_001).unwrap();
        book.legacy_begin_write(2,10_001);
        book.legacy_wrote(2,10_002,true);
        assert!(book.acquire(&four,&a,&e,10_003,false).is_err(),
            "replay from released MC03 lease cannot claim all four AI");
        assert!(book.acquire(&four,&a,&fresh,10_003,false).is_ok());

        // Explicit successful neutral input is the ONLY early clearance.
        let mut recovered=AiLeaseBook::default();
        recovered.legacy_allowed(2,2000).unwrap();
        recovered.legacy_begin_write(2,2000);
        assert!(recovered.acquire(&five,&a,&e,2001,false).is_err());
        recovered.legacy_wrote(2,2002,true);
        assert!(recovered.acquire(&five,&a,&e,2003,false).is_ok());

        // A failed neutral write must retain the potential activity fence.
        let mut uncertain=AiLeaseBook::default();
        uncertain.legacy_begin_write(2,3000);
        assert!(uncertain.acquire(&four,&a,&e,3001,false).is_err());
        uncertain.forget_slot(2); // Only native confirmed process reap.
        assert!(uncertain.acquire(&four,&a,&e,3002,false).is_ok());
    }


    #[cfg(feature="mc02-native-input")]
    #[test]
    fn legacy_packet_ttl_never_unlocks_four_or_five_ai_without_neutral_ack() {
        // Source-only ledger fixture: the native consumer may stop polling
        // at 500ms but there is no ACK that it received key-up. One legacy
        // MC02 channel is shared with each selected group of AI slots.
        let owner="a".repeat(32);
        let epoch=synthetic_ai_epoch(1);
        let fresh=synthetic_ai_epoch(2);
        let four=vec![(2,202,2202),(3,203,2203),(4,204,2204),(5,205,2205)];
        let mut five=vec![(1,201,2201)];
        five.extend(four.iter().copied());
        for cohort in [&four,&five] {
            let mut book=AiLeaseBook::default();
            book.legacy_allowed(2,1000).unwrap();
            book.legacy_begin_write(2,1000);
            // A packet may have been published, then native I/O failed;
            // legacy_wrote is intentionally NOT called after that error.
            assert!(book.acquire(cohort,&owner,&epoch,1500,false).is_err());
            assert!(book.acquire(cohort,&owner,&epoch,15_000,false).is_err());
            assert!(book.acquire(&[(2,202,2202)],&owner,&epoch,
                15_001,true).is_err(),
                "recovery-only multi-AI cannot bypass unresolved legacy input");
            // Independently healthy original clients retain their lease
            // admission even when one legacy channel requires recovery.
            let peers=cohort.iter().copied()
                .filter(|(slot,_,_)|*slot!=2).collect::<Vec<_>>();
            book.acquire(&peers,&owner,&epoch,15_002,false).unwrap();
            book.release(&peers,&owner,&epoch,15_003).unwrap();

            // A legacy neutral attempt that fails after possible publish
            // must keep the unresolved fence, including after another TTL.
            book.legacy_begin_write(2,15_004);
            assert!(book.acquire(cohort,&owner,&epoch,16_000,false).is_err());
            book.legacy_wrote(2,16_001,true); // original channel ACK
            assert!(book.acquire(cohort,&owner,&epoch,16_002,false).is_err(),
                "releasing legacy MC02 does not erase peer epoch history");
            book.acquire(cohort,&owner,&fresh,16_002,false).unwrap();
            book.release(cohort,&owner,&fresh,16_003).unwrap();

            // Confirmed process reap clears only this slot and does not
            // admit a stale original process identity into a replacement.
            let mut exited=AiLeaseBook::default();
            exited.legacy_begin_write(2,1000);
            exited.forget_slot(2);
            let replacement=cohort.iter().copied().map(|(slot,pid,birth)|
                if slot==2{(slot,902,birth+1000)}else{(slot,pid,birth)}
            ).collect::<Vec<_>>();
            exited.acquire(&replacement,&owner,&epoch,1002,false).unwrap();
        }
    }

    #[test]
    fn writer_slot_order_is_bounded_for_slice_and_target_iterators() {
        let owner="a".repeat(32);
        for slots in [vec![],vec![0],vec![6],vec![2,2],vec![3,2],
                      vec![1,2,3,4,5,6],vec![1],vec![1,3,5],vec![1,2,3,4,5]] {
            let expected=!slots.is_empty()&&slots.len()<=5&&
                slots.iter().all(|slot|(1..=5).contains(slot))&&
                slots.windows(2).all(|pair|pair[0]<pair[1]);
            let targets=slots.iter().map(|slot|(*slot,1u32,1u64)).collect::<Vec<_>>();
            assert_eq!(validate_ai_lease_slots(&slots,&owner).is_ok(),expected);
            assert_eq!(validate_ai_lease_slot_order(
                targets.iter().map(|(slot,_,_)|*slot),&owner).is_ok(),expected);
        }
        assert!(validate_ai_lease_slots(&[1],"invalid").is_err());
        assert!(validate_ai_lease_slot_order([1].into_iter(),"invalid").is_err());
    }
}
