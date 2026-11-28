//! Bounded agent activity wire contract and ingress validation.
use serde::{Deserialize, Serialize};

// A bounded, short-lived UX mirror of an agent CLI report. Not a second
// game/QA owner, not proof of Windows desktop control or Minecraft input.
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AgentProbeCounts {
    pub expected:u32,
    pub pass:u32,
    pub fail:u32,
    pub info:u32,
    pub look:u32,
}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AgentActivityItem {
    pub kind:String,
    pub slot:u32,
    pub status:String,
    pub importance:String,
    pub evidence:String,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub reported_outcome:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub probe_counts:Option<AgentProbeCounts>,
}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AgentActivityReport {
    pub schema:u32,
    pub source:String,
    pub snapshot_at:String,
    pub display_hint:String,
    pub desktop_control:String,
    pub minecraft_input_effect:String,
    pub map_gameplay_acceptance:String,
    pub items:Vec<AgentActivityItem>,
    #[serde(default)]
    pub stale:bool,
}
impl AgentActivityReport {
    pub fn valid_ingress(&self)->bool {
        if self.schema!=1||self.stale||self.items.len()>15||
           self.source!="CURRENT_MCP_PROCESS_MEMORY_ONLY"||
           !(20..=40).contains(&self.snapshot_at.len())||
           !self.snapshot_at.ends_with('Z')||
           self.desktop_control!="NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER"||
           self.minecraft_input_effect!="NOT_VERIFIED"||
           self.map_gameplay_acceptance!="NOT_TESTED" {
            return false;
        }
        // Exactly three activity kinds across five slots; no heap or hash needed.
        let mut seen = [[false; 5]; 3];
        let mut attention=false;
        for item in &self.items {
            if !(1..=5).contains(&item.slot) ||
               !matches!(item.importance.as_str(),"INFORMATION"|"ATTENTION") {
                return false;
            }
            let kind = match item.kind.as_str() {
                "AI_INPUT" => 0,
                "VISUAL_OBSERVATION" => 1,
                "SKILLS_PROBE" => 2,
                _ => return false,
            };
            let entry = &mut seen[kind][(item.slot - 1) as usize];
            if *entry {
                return false;
            }
            *entry = true;
            attention|=item.importance=="ATTENTION";
            let allowed=match item.kind.as_str() {
                "AI_INPUT" => {
                    item.evidence=="LOCAL_MCP_RECOVERY_HOLD" &&
                      matches!(item.status.as_str(),
                        "NATIVE_RELEASE_UNCONFIRMED"|
                        "STARTUP_RECONCILIATION_REQUIRED")
                    || item.evidence=="LOCAL_MCP_REQUEST_ONLY" &&
                       item.status=="EXPERIMENTAL_INPUT_REQUEST_IN_FLIGHT"
                },
                "VISUAL_OBSERVATION" =>
                    item.evidence=="LOCAL_SAMPLER_ONLY"&&
                    matches!(item.status.as_str(),
                      "SAMPLER_ACTIVE_NO_RECENT_FRAME"|
                      "SAMPLER_ACTIVE_RECENT_FRAME"),
                "SKILLS_PROBE" => {
                    let expected=if item.status=="PROBE_REPORT_AVAILABLE" {
                        "ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN"
                    } else if matches!(item.status.as_str(),
                        "LOCAL_QA_REVIEW_IN_PROGRESS"|
                        "LOCAL_QA_PREPARATION_IN_PROGRESS") {
                        "MCP_PROCESS_ONLY"
                    } else { "SOURCE_CHECKPOINT_ONLY" };
                    item.evidence==expected&&
                    matches!(item.status.as_str(),
                      "PROBE_REPORT_AVAILABLE"|
                      "LOCAL_QA_REVIEW_IN_PROGRESS"|
                      "LOCAL_QA_PREPARATION_IN_PROGRESS"|
                      "AWAITING_AUTHORIZED_COMMAND_OR_EVIDENCE"|
                      "CHECKPOINT_EXPIRED_OR_UNAVAILABLE")
                },
                _=>false,
            };
            let outcome_allowed=match item.reported_outcome.as_deref() {
                None=>item.status!="PROBE_REPORT_AVAILABLE",
                Some(value)=>item.kind=="SKILLS_PROBE" &&
                    item.status=="PROBE_REPORT_AVAILABLE" &&
                    matches!(value,"PROBE_REPORTED_PASS"|"FAIL"|
                      "NEEDS_REVIEW"|"INCOMPLETE"|"BLOCKED"|
                      "UNRECOGNIZED_TERMINAL_STATUS"),
            };
            // Counts may be forwarded only from a complete original
            // Probe status, never from partial/blocked reports or AI input.
            let counts_allowed=match item.probe_counts.as_ref() {
                None=>true,
                Some(counts)=>{
                    let outcome=item.reported_outcome.as_deref();
                    let completed=matches!(outcome,
                        Some("PROBE_REPORTED_PASS"|"FAIL"|"NEEDS_REVIEW"));
                    let total=u64::from(counts.pass)+u64::from(counts.fail)+
                        u64::from(counts.info)+u64::from(counts.look);
                    completed&&item.kind=="SKILLS_PROBE"&&
                        item.status=="PROBE_REPORT_AVAILABLE"&&
                        (1..=80).contains(&counts.expected)&&
                        [counts.pass,counts.fail,counts.info,counts.look]
                          .iter().all(|value|*value<=80)&&
                        total==u64::from(counts.expected)&&
                        match outcome {
                            Some("PROBE_REPORTED_PASS") =>
                                counts.pass==counts.expected,
                            Some("FAIL")=>counts.fail>0,
                            Some("NEEDS_REVIEW")=>
                                counts.fail==0&&counts.info+counts.look>0,
                            _=>false,
                        }
                }
            };
            let important=matches!(item.status.as_str(),
                "NATIVE_RELEASE_UNCONFIRMED"|
                "STARTUP_RECONCILIATION_REQUIRED"|
                "CHECKPOINT_EXPIRED_OR_UNAVAILABLE") ||
                (item.status=="PROBE_REPORT_AVAILABLE" &&
                 item.reported_outcome.as_deref()!=Some("PROBE_REPORTED_PASS"));
            if !allowed||!outcome_allowed||!counts_allowed||
               (item.importance=="ATTENTION")!=important{return false;}
        }
        self.display_hint==if attention{"ATTENTION"}
          else if self.items.is_empty(){"HIDDEN_NO_LOCAL_SIGNAL"}
          else {"ACTIVITY"}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_activity_report_is_bounded_and_cannot_claim_desktop_control() {
        // Pure metadata only: no native controller, game or Tauri execution.
        let mut report=AgentActivityReport{
            schema:1,source:"CURRENT_MCP_PROCESS_MEMORY_ONLY".into(),
            snapshot_at:"2026-11-02T12:00:00.000Z".into(),
            display_hint:"ATTENTION".into(),
            desktop_control:"NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER".into(),
            minecraft_input_effect:"NOT_VERIFIED".into(),
            map_gameplay_acceptance:"NOT_TESTED".into(),stale:false,
            items:vec![AgentActivityItem{
                kind:"AI_INPUT".into(),slot:2,
                status:"NATIVE_RELEASE_UNCONFIRMED".into(),
                importance:"ATTENTION".into(),
                evidence:"LOCAL_MCP_RECOVERY_HOLD".into(),
                reported_outcome:None,
                probe_counts:None,
            }],
        };
        assert!(report.valid_ingress());
        report.items.push(report.items[0].clone());
        assert!(!report.valid_ingress(), "Duplicate kind and slot must be refused");
        report.items[1].slot = 3;
        assert!(report.valid_ingress(), "The same kind may use different slots");
        report.items[1] = AgentActivityItem {
            kind: "VISUAL_OBSERVATION".into(), slot: 2,
            status: "SAMPLER_ACTIVE_RECENT_FRAME".into(),
            importance: "INFORMATION".into(), evidence: "LOCAL_SAMPLER_ONLY".into(),
            reported_outcome: None, probe_counts: None,
        };
        assert!(report.valid_ingress(), "Different kinds may share a slot");
        report.items.push(report.items[1].clone());
        assert!(!report.valid_ingress(), "Duplicate visual activity must be refused");
        report.items.truncate(1);
        report.desktop_control="AI_CONTROLLING_PC".into();
        assert!(!report.valid_ingress());
        report.desktop_control="NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER".into();
        report.items[0].importance="INFORMATION".into();
        assert!(!report.valid_ingress(),
            "Unconfirmed input release cannot be reduced to normal activity");
        report.items[0].importance="ATTENTION".into();
        report.map_gameplay_acceptance="PASS".into();
        assert!(!report.valid_ingress());
        report.map_gameplay_acceptance="NOT_TESTED".into();
        report.items[0].slot=6;
        assert!(!report.valid_ingress());
        report.items[0].slot=2;
        report.stale=true;
        assert!(!report.valid_ingress(),
            "Only the controller may add a stale notice after monotonic expiry");
        report.stale=false;
        report.display_hint="ACTIVITY".into();
        report.items[0]=AgentActivityItem{
            kind:"SKILLS_PROBE".into(),slot:2,
            status:"PROBE_REPORT_AVAILABLE".into(),
            importance:"INFORMATION".into(),
            evidence:"ORIGINAL_PROBE_LOG_UNVERIFIED_ORIGIN".into(),
            reported_outcome:Some("PROBE_REPORTED_PASS".into()),
            probe_counts:Some(AgentProbeCounts{
                expected:2,pass:2,fail:0,info:0,look:0,
            }),
        };
        assert!(report.valid_ingress());
        report.items.push(report.items[0].clone());
        assert!(!report.valid_ingress(), "Duplicate Probe activity must be refused");
        report.items.truncate(1);
        report.items[0].probe_counts.as_mut().unwrap().pass=3;
        assert!(!report.valid_ingress(),"Counts cannot exceed the reported total");
        report.items[0].probe_counts.as_mut().unwrap().pass=2;
        report.items[0].reported_outcome=Some("INCOMPLETE".into());
        report.items[0].importance="ATTENTION".into();
        report.display_hint="ATTENTION".into();
        assert!(!report.valid_ingress(),
            "Incomplete results may not carry completed Probe counts");
    }
}
