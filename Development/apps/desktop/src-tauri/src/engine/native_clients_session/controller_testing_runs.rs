//! Controller-owned volatile Testing Run reservations and immutable final references.
//! The session transport validates ingress; only the retained controller owns this state.
use serde::{Deserialize, Serialize};
#[cfg(windows)]
use super::pipe;

/* A correlation/fencing book in the retained ORIGINAL native controller.
   It does not execute QA, read Skills Probe evidence or certify game/world.
   One reservation and immutable FINAL reference per native MC slot.
   Controller restart loses this volatile book: it is NOT a durable archive. */
// Read-only snapshot of the original controller's in-memory Testing run
// reservation, never proof that the owning AI process is alive/dead.
#[derive(Clone,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct TestingRunInspection {
    pub schema:u32,
    pub slot:u32,
    pub phase:String,
    pub run_id:Option<String>,
    pub owner_token_match:bool,
    pub reported_status:Option<String>,
    pub report_digest:Option<String>,
    pub owner_process_proof:String,
    pub operator_approval:String,
    pub transfer_allowed:bool,
    pub recovery_action:String,
    pub provenance:String,
}
#[derive(Clone,Debug,PartialEq,Eq,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct TestingRunView {
    pub schema:u32,
    pub slot:u32,
    pub run_id:String,
    pub phase:String,
    pub owner_match:bool,
    // Serialize explicit null in a RESERVED receipt: the JS one-shot
    // adapter must distinguish "not finalized" from a malformed reply.
    #[serde(default)]
    pub reported_status:Option<String>,
    #[serde(default)]
    pub report_digest:Option<String>,
    pub provenance:String,
}
#[derive(Debug)]
struct TestingRecord {
    owner:String,
    view:TestingRunView,
    #[cfg(windows)]
    owner_process:Option<pipe::McpOwnerLifetime>,
}
// Test-only prospective recovery contract; never a production capability.
#[cfg(test)]
struct VerifiedTestingOperatorPresence(());
#[cfg(test)]
struct VerifiedNativeTestingArchive(());
#[cfg(test)]
struct TestingRecoveryAdmission {
    slot:u32,
    expected_run_id:String,
    expected_owner:String,
    new_run_id:String,
    new_owner:String,
    _operator:VerifiedTestingOperatorPresence,
    _archive:VerifiedNativeTestingArchive,
}
pub(crate) struct NativeTestingRunBook {
    slots:[Option<TestingRecord>;5],
}
impl Default for NativeTestingRunBook {
    fn default()->Self {
        Self {slots:std::array::from_fn(|_|None)}
    }
}
pub(crate) fn valid_testing_id(id:&str)->bool {
    id.len()==32 && id.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())
}
pub(super) fn valid_testing_digest(id:&str)->bool {
    id.len()==64 && id.bytes().all(|b|b.is_ascii_hexdigit()&&!b.is_ascii_uppercase())
}
pub(super) fn valid_testing_outcome(outcome:&str)->bool {
    matches!(outcome,"PROBE_REPORTED_PASS"|"FAIL"|"NEEDS_REVIEW"|"INCOMPLETE"|"BLOCKED")
}
impl NativeTestingRunBook {
    pub(crate) fn inspect(
        &self,slot:u32,caller_token:&str,
    )->Result<TestingRunInspection,&'static str> {
        if !(1..=5).contains(&slot)||!valid_testing_id(caller_token) {
            return Err("TESTING_INSPECTION_INVALID");
        }
        let record=self.slots[(slot-1) as usize].as_ref();
        let (phase,run_id,owner_match,reported_status,digest,action)=
            if let Some(held)=record {
                (held.view.phase.clone(),Some(held.view.run_id.clone()),
                 held.owner==caller_token,held.view.reported_status.clone(),
                 held.view.report_digest.clone(),
                 if held.view.phase=="RESERVED" {
                    "REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL"
                 } else {"READ_FINAL_REPORT_NO_OWNER_TRANSFER"})
            } else {
                // A controller restart ALSO produces NO_RECORD. Do not
                // infer there was never a prior run from this state.
                ("NO_RECORD".into(),None,false,None,None,
                 "CHECK_ARCHIVE_AND_CONTROLLER_EPOCH")
            };
        // A token match is not a process proof. Only a retained OS handle
        // can report its pinned parent lifetime; no takeover is permitted.
        #[cfg(windows)]
        let owner_proof=record.and_then(|held|held.owner_process.as_ref())
            .map_or("NOT_BOUND_TO_PROCESS_HANDLE",|pin|pin.status());
        #[cfg(not(windows))]
        let owner_proof="NOT_BOUND_TO_PROCESS_HANDLE";
        Ok(TestingRunInspection {
            schema:1,slot,phase,run_id,
            owner_token_match:owner_match,
            reported_status,report_digest:digest,
            owner_process_proof:owner_proof.into(),
            operator_approval:"NOT_AVAILABLE_IN_PROTOCOL".into(),
            transfer_allowed:false,
            recovery_action:action.into(),
            provenance:"RETAINED_CONTROLLER_MEMORY_ONLY_NOT_GAMEPLAY_PROOF".into(),
        })
    }
    // Internal pure transition; live native requests use reserve_from_peer.
    fn reserve(
        &mut self,slot:u32,owner:&str,run_id:&str,replace_run_id:Option<&str>,
    )->Result<TestingRunView,&'static str> {
        if !(1..=5).contains(&slot)||
           !valid_testing_id(owner)||!valid_testing_id(run_id)||
           replace_run_id.is_some_and(|s|!valid_testing_id(s)) {
            return Err("TESTING_RUN_INVALID_ID");
        }
        let entry=&mut self.slots[(slot-1) as usize];
        if let Some(held)=entry {
            let same=held.owner==owner;
            if same&&held.view.run_id==run_id&&replace_run_id.is_none() {
                let mut view=held.view.clone();
                view.owner_match=true;
                return Ok(view);
            }
            if same&&held.view.phase=="FINAL"&&
              replace_run_id==Some(held.view.run_id.as_str())&&
              run_id!=held.view.run_id {
                // Explicit restart after immutable terminal reference only.
                *entry=None;
            } else {
                let mut view=held.view.clone();
                view.owner_match=false; // Not permission to overwrite.
                return Ok(view);
            }
        } else if replace_run_id.is_some() {
            return Err("TESTING_PREVIOUS_RUN_UNAVAILABLE");
        }
        let view=TestingRunView {
            schema:1,slot,run_id:run_id.into(),
            phase:"RESERVED".into(),owner_match:true,
            reported_status:None,report_digest:None,
            provenance:"NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF".into(),
        };
        *entry=Some(TestingRecord{
            owner:owner.into(),view:view.clone(),
            #[cfg(windows)]
            owner_process:None,
        });
        Ok(view)
    }
    // Internal pure transition; live native requests use finalize_from_peer.
    fn finalize(
        &mut self,slot:u32,owner:&str,run_id:&str,
        outcome:&str,digest:&str,
    )->Result<TestingRunView,&'static str> {
        if !(1..=5).contains(&slot)||!valid_testing_id(owner)||
           !valid_testing_id(run_id)||!valid_testing_digest(digest)||
           !valid_testing_outcome(outcome) {
            return Err("TESTING_FINAL_REPORT_INVALID");
        }
        let record=self.slots[(slot-1) as usize].as_mut()
            .ok_or("TESTING_RESERVATION_REQUIRED")?;
        if record.owner!=owner||record.view.run_id!=run_id {
            return Err("TESTING_RUN_OWNER_MISMATCH");
        }
        if record.view.phase=="FINAL" {
            if record.view.reported_status.as_deref()==Some(outcome) &&
               record.view.report_digest.as_deref()==Some(digest) {
                return Ok(record.view.clone());
            }
            return Err("TESTING_FINAL_RESULT_IMMUTABLE");
        }
        record.view.phase="FINAL".into();
        record.view.reported_status=Some(outcome.into());
        record.view.report_digest=Some(digest.into());
        Ok(record.view.clone())
    }

    #[cfg(windows)]
    pub(crate) fn reserve_from_peer(
        &mut self,slot:u32,owner:&str,run_id:&str,
        replace_run_id:Option<&str>,peer:&pipe::Peer,
    )->Result<TestingRunView,&'static str> {
        if !(1..=5).contains(&slot) {return Err("TESTING_RUN_INVALID_ID");}
        let incoming=peer.mcp_parent()
            .map_err(|_|"TESTING_MCP_OWNER_PROCESS_UNVERIFIED")?;
        if let Some(held)=self.slots[(slot-1) as usize].as_ref() {
            if held.owner==owner && !held.owner_process.as_ref().is_some_and(
                |original|original.same(&incoming) &&
                  original.status()=="BOUND_PARENT_RUNNING_AT_INSPECTION") {
                return Err("TESTING_MCP_OWNER_PROCESS_CHANGED");
            }
        }
        if incoming.status()!="BOUND_PARENT_RUNNING_AT_INSPECTION" {
            return Err("TESTING_MCP_OWNER_PROCESS_UNVERIFIED");
        }
        let view=self.reserve(slot,owner,run_id,replace_run_id)?;
        if view.owner_match {
            // A new/owned reservation pins the same verified parent OS
            // process, rather than the disposable CLI or caller token.
            let held=self.slots[(slot-1) as usize].as_mut()
                .ok_or("TESTING_RESERVATION_REQUIRED")?;
            held.owner_process=Some(incoming);
        }
        Ok(view)
    }

    #[cfg(windows)]
    pub(crate) fn finalize_from_peer(
        &mut self,slot:u32,owner:&str,run_id:&str,
        outcome:&str,digest:&str,peer:&pipe::Peer,
    )->Result<TestingRunView,&'static str> {
        if !(1..=5).contains(&slot) {
            return Err("TESTING_FINAL_REPORT_INVALID");
        }
        let incoming=peer.mcp_parent()
            .map_err(|_|"TESTING_MCP_OWNER_PROCESS_UNVERIFIED")?;
        let held=self.slots[(slot-1) as usize].as_ref()
            .ok_or("TESTING_RESERVATION_REQUIRED")?;
        if held.owner!=owner || held.view.run_id!=run_id ||
           !held.owner_process.as_ref().is_some_and(
              |original|original.same(&incoming) &&
                original.status()=="BOUND_PARENT_RUNNING_AT_INSPECTION") ||
           incoming.status()!="BOUND_PARENT_RUNNING_AT_INSPECTION" {
            return Err("TESTING_MCP_OWNER_PROCESS_CHANGED");
        }
        self.finalize(slot,owner,run_id,outcome,digest)
    }
    // Test-only compare of a prospective transfer. No authentication or
    // ownership change is available in the live controller.
    #[cfg(test)]
    fn check_recovery_cas(
        &self,permit:&TestingRecoveryAdmission,
    )->Result<(),&'static str> {
        if !(1..=5).contains(&permit.slot)||
           !valid_testing_id(&permit.expected_run_id)||
           !valid_testing_id(&permit.expected_owner)||
           !valid_testing_id(&permit.new_run_id)||
           !valid_testing_id(&permit.new_owner)||
           permit.new_run_id==permit.expected_run_id||
           permit.new_owner==permit.expected_owner {
            return Err("TESTING_TRANSFER_INVALID_CONTRACT");
        }
        let current=self.slots[(permit.slot-1) as usize].as_ref()
            .ok_or("TESTING_TRANSFER_OLD_RUN_UNAVAILABLE")?;
        if current.view.run_id!=permit.expected_run_id||
           current.owner!=permit.expected_owner {
            return Err("TESTING_TRANSFER_STALE_EXPECTED_RUN");
        }
        if current.view.phase!="RESERVED"||
           current.view.reported_status.is_some()||
           current.view.report_digest.is_some() {
            return Err("TESTING_TRANSFER_FINAL_OR_INCONSISTENT");
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn staged_testing_recovery_cas_rejects_stale_or_finalized_slots() {
        // Pure contract fixtures, NOT authenticated operator approval.
        // No production constructor, native transfer request, or game action.
        fn fixture(slot:u32,old_run:&str,old_owner:&str,
                   new_run:&str,new_owner:&str)->TestingRecoveryAdmission {
            TestingRecoveryAdmission{
                slot,expected_run_id:old_run.into(),
                expected_owner:old_owner.into(),
                new_run_id:new_run.into(),new_owner:new_owner.into(),
                _operator:VerifiedTestingOperatorPresence(()),
                _archive:VerifiedNativeTestingArchive(()),
            }
        }
        let mut book=NativeTestingRunBook::default();
        let old="a".repeat(32);
        let other="b".repeat(32);
        let next="c".repeat(32);
        let old_run="d".repeat(32);
        let stale_run="9".repeat(32);
        let new_run="e".repeat(32);
        assert_eq!(
            book.check_recovery_cas(&fixture(2,&old_run,&old,&new_run,&next)),
            Err("TESTING_TRANSFER_OLD_RUN_UNAVAILABLE"));
        let owned=book.reserve(2,&old,&old_run,None).unwrap();
        let proposed=fixture(2,&old_run,&old,&new_run,&next);
        assert_eq!(book.check_recovery_cas(&proposed),Ok(()));
        assert_eq!(book.inspect(2,&other).unwrap().run_id.as_deref(),
            Some(old_run.as_str()),"Preflight cannot change ownership");
        for candidate in [
            fixture(0,&old_run,&old,&new_run,&next),
            fixture(6,&old_run,&old,&new_run,&next),
            fixture(2,&old_run,&old,&old_run,&next),
            fixture(2,&old_run,&old,&new_run,&old),
            fixture(2,&old_run,&old,"INVALID",&next),
        ] {
            assert_eq!(book.check_recovery_cas(&candidate),
                Err("TESTING_TRANSFER_INVALID_CONTRACT"));
        }
        for candidate in [
            fixture(2,&stale_run,&old,&new_run,&next),
            fixture(2,&old_run,&other,&new_run,&next),
        ] {
            assert_eq!(book.check_recovery_cas(&candidate),
                Err("TESTING_TRANSFER_STALE_EXPECTED_RUN"));
        }
        assert_eq!(book.inspect(2,&other).unwrap().phase,"RESERVED");
        assert_eq!(book.reserve(2,&old,&old_run,None).unwrap(),owned);
        book.finalize(2,&old,&old_run,"BLOCKED",&"f".repeat(64)).unwrap();
        assert_eq!(
            book.check_recovery_cas(&fixture(2,&old_run,&old,&new_run,&next)),
            Err("TESTING_TRANSFER_FINAL_OR_INCONSISTENT"));
        assert_eq!(book.inspect(2,&other).unwrap().phase,"FINAL");
        assert!(!book.inspect(2,&other).unwrap().transfer_allowed);
    }

    #[test]
    fn testing_inspection_is_read_only_and_never_authorizes_orphan_takeover() {
        // No controller/game/OS access. Token comparison is not PID proof.
        let mut book=NativeTestingRunBook::default();
        let owner="a".repeat(32);
        let other="b".repeat(32);
        let run="c".repeat(32);
        let digest="1".repeat(64);
        let empty=book.inspect(2,&other).unwrap();
        assert_eq!(empty.phase,"NO_RECORD");
        assert!(empty.run_id.is_none());
        assert!(!empty.transfer_allowed);
        assert_eq!(empty.recovery_action,"CHECK_ARCHIVE_AND_CONTROLLER_EPOCH");
        assert_eq!(empty.owner_process_proof,"NOT_BOUND_TO_PROCESS_HANDLE");
        assert_eq!(empty.operator_approval,"NOT_AVAILABLE_IN_PROTOCOL");
        assert_eq!(book.inspect(0,&owner),Err("TESTING_INSPECTION_INVALID"));
        assert_eq!(book.inspect(2,"INVALID"),Err("TESTING_INSPECTION_INVALID"));

        let reserved=book.reserve(2,&owner,&run,None).unwrap();
        let owner_view=book.inspect(2,&owner).unwrap();
        let other_view=book.inspect(2,&other).unwrap();
        assert_eq!(owner_view.phase,"RESERVED");
        assert!(owner_view.owner_token_match);
        assert!(!other_view.owner_token_match);
        assert_eq!(other_view.run_id.as_deref(),Some(run.as_str()));
        assert_eq!(other_view.reported_status,None);
        assert_eq!(other_view.report_digest,None);
        assert_eq!(other_view.recovery_action,
            "REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL");
        assert!(!owner_view.transfer_allowed);
        assert!(!other_view.transfer_allowed);
        assert_eq!(other_view.operator_approval,"NOT_AVAILABLE_IN_PROTOCOL");
        assert_eq!(book.reserve(2,&owner,&run,None).unwrap(),reserved,
            "Inspection must not mutate a reservation");
        assert!(!book.reserve(2,&other,&"d".repeat(32),None)
            .unwrap().owner_match);
        assert_eq!(book.inspect(2,&owner).unwrap(),owner_view,
            "A competing reservation cannot change the original owner or recovery gate");

        book.finalize(2,&owner,&run,"FAIL",&digest).unwrap();
        let final_view=book.inspect(2,&other).unwrap();
        assert_eq!(final_view.phase,"FINAL");
        assert_eq!(final_view.reported_status.as_deref(),Some("FAIL"));
        assert_eq!(final_view.report_digest.as_deref(),Some(digest.as_str()));
        assert_eq!(final_view.recovery_action,
            "READ_FINAL_REPORT_NO_OWNER_TRANSFER");
        assert!(!final_view.transfer_allowed);
        assert_eq!(book.finalize(2,&other,&run,"FAIL",&digest),
            Err("TESTING_RUN_OWNER_MISMATCH"));
    }

    #[test]
    fn testing_run_book_deduplicates_and_fences_terminal_outcome() {
        // Pure Rust source fixture; no Windows IPC or gameplay.
        let mut book=NativeTestingRunBook::default();
        let owner="a".repeat(32);
        let rival="b".repeat(32);
        let first="c".repeat(32);
        let other="d".repeat(32);
        let digest="1".repeat(64);
        let wrong="2".repeat(64);
        let reserved=book.reserve(2,&owner,&first,None).unwrap();
        assert_eq!(reserved.phase,"RESERVED");
        assert!(reserved.owner_match);
        assert_eq!(reserved.provenance,
            "NATIVE_RUN_CORRELATION_ONLY_NOT_GAMEPLAY_PROOF");
        assert_eq!(book.reserve(2,&owner,&first,None).unwrap(),reserved);
        let rival_view=book.reserve(2,&rival,&other,None).unwrap();
        assert_eq!(rival_view.run_id,first);
        assert!(!rival_view.owner_match);
        assert_eq!(book.finalize(2,&rival,&first,"FAIL",&digest),
            Err("TESTING_RUN_OWNER_MISMATCH"));
        let complete=book.finalize(
            2,&owner,&first,"PROBE_REPORTED_PASS",&digest).unwrap();
        assert_eq!(complete.phase,"FINAL");
        assert_eq!(complete.report_digest.as_deref(),Some(digest.as_str()));
        assert_eq!(book.finalize(2,&owner,&first,"FAIL",&wrong),
            Err("TESTING_FINAL_RESULT_IMMUTABLE"));
        assert_eq!(book.finalize(
            2,&owner,&first,"PROBE_REPORTED_PASS",&digest).unwrap(),complete);
        assert!(!book.reserve(2,&owner,&other,None).unwrap().owner_match);
        assert!(!book.reserve(2,&rival,&other,Some(&first)).unwrap().owner_match);
        let restarted=book.reserve(2,&owner,&other,Some(&first)).unwrap();
        assert!(restarted.owner_match);
        assert_eq!(restarted.phase,"RESERVED");
        assert_eq!(restarted.run_id,other);
        assert!(restarted.report_digest.is_none());
    }

    #[test]
    fn testing_run_book_refuses_invalid_claims_and_active_restart() {
        let mut book=NativeTestingRunBook::default();
        let a="a".repeat(32);
        let b="b".repeat(32);
        let sha="1".repeat(64);
        assert_eq!(book.reserve(0,&a,&b,None),Err("TESTING_RUN_INVALID_ID"));
        assert_eq!(book.reserve(2,&a,"UPPERCASE",None),
            Err("TESTING_RUN_INVALID_ID"));
        assert_eq!(book.reserve(2,&a,&b,Some(&b)),
            Err("TESTING_PREVIOUS_RUN_UNAVAILABLE"));
        assert!(book.reserve(2,&a,&b,None).unwrap().owner_match);
        assert!(!book.reserve(2,&a,&a,Some(&b)).unwrap().owner_match,
            "Even owner cannot restart while a run is reserved");
        assert_eq!(book.finalize(2,&a,&b,"PASS",&sha),
            Err("TESTING_FINAL_REPORT_INVALID"));
        assert_eq!(book.finalize(2,&a,&b,"BLOCKED",&sha)
            .unwrap().phase,"FINAL");
    }
}
