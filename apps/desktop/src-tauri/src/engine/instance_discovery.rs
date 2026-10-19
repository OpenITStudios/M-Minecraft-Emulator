//! Operator-triggered read-only projection of existing provider observations.
//! Never installs/provisions/launches Minecraft, infers account state, or runs
//! unattended. Native name matches do NOT establish Host 01 ownership.
use serde::Serialize;
use std::io;

use super::{
    instance_catalog::{admit_slot_operation, instance_slots, SlotOperation},
    native_clients::{self, NativeProcessObservation, SandboxieClientObservation},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotDiscovery {
    pub id: &'static str,
    pub evidence: &'static str,
    pub candidate_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceOverview {
    pub schema: u32,
    pub slots: [SlotDiscovery; 4],
}

/// Source-only projection; successful collector calls provide candidates,
/// not proof of a signed-in, playable game or eligibility to launch.
fn project_overview(
    native: &NativeProcessObservation,
    isolated: &SandboxieClientObservation,
) -> InstanceOverview {
    let slots = instance_slots();
    InstanceOverview {
        schema: 1,
        slots: [
            SlotDiscovery {
                id: slots[0].id,
                evidence: if native.interactive_session {
                    "NAME_MATCH_UNATTRIBUTED"
                } else {
                    "UNKNOWN"
                },
                candidate_count: native.interactive_session
                    .then_some(native.candidates.len() as u32),
            },
            SlotDiscovery {
                id: slots[1].id,
                evidence: if isolated.interactive_session
                    && isolated.membership_status == "OBSERVED"
                    && isolated.box_configured == "YES"
                    && isolated.box_settings_source == "BOX_SECTION_ENUMERATED"
                {
                    "BOX_PROCESS_CANDIDATES_ONLY"
                } else {
                    "UNKNOWN"
                },
                candidate_count: (isolated.interactive_session
                    && isolated.membership_status == "OBSERVED"
                    && isolated.box_configured == "YES"
                    && isolated.box_settings_source == "BOX_SECTION_ENUMERATED")
                    .then_some(isolated.isolated_candidate_processes),
            },
            SlotDiscovery {
                id: slots[2].id,
                evidence: "PROVIDER_NOT_AVAILABLE",
                candidate_count: None,
            },
            SlotDiscovery {
                id: slots[3].id,
                evidence: "PROVIDER_NOT_AVAILABLE",
                candidate_count: None,
            },
        ],
    }
}

pub fn inspect_instance_overview() -> io::Result<InstanceOverview> {
    // Explicit caller action only. Both collectors run in sequence and fail
    // closed: an incomplete result must never appear as an empty game slot.
    admit_slot_operation("01", SlotOperation::Inspect)?;
    admit_slot_operation("02", SlotOperation::Inspect)?;
    let native = native_clients::inspect_native_candidates()?;
    let isolated = native_clients::inspect_sandbox_client_02()?;
    Ok(project_overview(&native, &isolated))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::native_clients::NativeProcessCandidate;

    fn native(candidates: usize, interactive: bool) -> NativeProcessObservation {
        NativeProcessObservation {
            schema: 1,
            evidence_class: "NAME_MATCH_CANDIDATE_ONLY".into(),
            interactive_session: interactive,
            candidates: (1..=candidates as u32).map(|pid| NativeProcessCandidate {
                pid,
                window_handle: None,
            }).collect(),
        }
    }
    fn isolated(membership: &str, candidates: u32) -> SandboxieClientObservation {
        SandboxieClientObservation {
            schema: 6,
            box_name: "MinecraftClient02".into(),
            stage: "INSPECT".into(),
            box_configured: "YES".into(),
            box_settings_source: "BOX_SECTION_ENUMERATED".into(),
            auto_delete: "OFF".into(),
            auto_remove: "OFF".into(),
            credential_host_access: "OFF".into(),
            membership_status: membership.into(),
            interactive_session: true,
            process_evidence_class: "NAME_MATCH_CANDIDATE_ONLY".into(),
            candidate_process_count: candidates,
            process_candidates: vec![],
            official_image_outside_selected_box_count: 0,
            direct_pid_query: "NOT_REQUESTED".into(),
            signed_image_other_box_count: 0,
            signed_image_unknown_box_count: 0,
            isolated_candidate_processes: candidates,
            native_visible_candidates: 0,
            isolated_visible_candidates: 0,
            interpretation: "CONFIG_AND_PROCESS_CANDIDATES_ONLY_NO_AUTHENTICATION_PROOF".into(),
        }
    }

    #[test]
    fn projects_four_slots_without_claiming_host_01_or_gameplay() {
        let result = project_overview(&native(2, true), &isolated("OBSERVED", 1));
        assert_eq!(result.schema, 1);
        assert_eq!(result.slots.iter().map(|slot| slot.id).collect::<Vec<_>>(), ["01", "02", "03", "04"]);
        assert_eq!(result.slots[0].evidence, "NAME_MATCH_UNATTRIBUTED");
        assert_eq!(result.slots[0].candidate_count, Some(2));
        assert_eq!(result.slots[1].evidence, "BOX_PROCESS_CANDIDATES_ONLY");
        assert_eq!(result.slots[1].candidate_count, Some(1));
        for slot in &result.slots[2..] {
            assert_eq!(slot.evidence, "PROVIDER_NOT_AVAILABLE");
            assert_eq!(slot.candidate_count, None);
        }
        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["slots"][0]["candidateCount"], 2);
        assert!(value["slots"][0].get("running").is_none());
        assert!(value["slots"][1].get("authenticated").is_none());
    }

    #[test]
    fn zero_candidates_is_observed_not_verified_stopped() {
        let result = project_overview(&native(0, true), &isolated("OBSERVED", 0));
        assert_eq!(result.slots[0].candidate_count, Some(0));
        assert_eq!(result.slots[1].candidate_count, Some(0));
        assert_eq!(result.slots[1].evidence, "BOX_PROCESS_CANDIDATES_ONLY");
    }

    #[test]
    fn ambiguous_or_noninteractive_observations_never_mean_empty() {
        let result = project_overview(&native(0, false), &isolated("UNKNOWN", 0));
        assert_eq!(result.slots[0].evidence, "UNKNOWN");
        assert_eq!(result.slots[0].candidate_count, None);
        assert_eq!(result.slots[1].evidence, "UNKNOWN");
        assert_eq!(result.slots[1].candidate_count, None);
        let mut unsafe_box = isolated("OBSERVED", 1);
        unsafe_box.box_settings_source = "UNKNOWN".into();
        let result = project_overview(&native(1, true), &unsafe_box);
        assert_eq!(result.slots[1].evidence, "UNKNOWN");
        assert_eq!(result.slots[1].candidate_count, None);
    }
}
