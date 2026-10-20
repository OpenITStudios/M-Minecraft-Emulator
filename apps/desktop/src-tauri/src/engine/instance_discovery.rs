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
    native: Option<&NativeProcessObservation>,
    isolated: Option<&SandboxieClientObservation>,
) -> InstanceOverview {
    let slots = instance_slots();
    // An unavailable/failed collector is UNKNOWN, never a verified empty slot.
    // Preserve successful evidence from the other independent collector.
    let native_observed = native.is_some_and(|view| view.interactive_session);
    let box_observed = isolated.is_some_and(|view| {
        view.interactive_session
            && view.membership_status == "OBSERVED"
            && view.box_configured == "YES"
            && view.box_settings_source == "BOX_SECTION_ENUMERATED"
    });
    InstanceOverview {
        schema: 1,
        slots: [
            SlotDiscovery {
                id: slots[0].id,
                evidence: if native_observed {
                    "NAME_MATCH_UNATTRIBUTED"
                } else {
                    "UNKNOWN"
                },
                candidate_count: native
                    .filter(|_| native_observed)
                    .map(|view| view.candidates.len() as u32),
            },
            SlotDiscovery {
                id: slots[1].id,
                evidence: if box_observed {
                    "BOX_PROCESS_CANDIDATES_ONLY"
                } else {
                    "UNKNOWN"
                },
                candidate_count: isolated
                    .filter(|_| box_observed)
                    .map(|view| view.isolated_candidate_processes),
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
    // Explicit caller action only. Attempt each existing collector once,
    // sequentially. A failure never becomes zero or a launch authorization;
    // one failed source cannot discard a separate source's valid evidence.
    admit_slot_operation("01", SlotOperation::Inspect)?;
    admit_slot_operation("02", SlotOperation::Inspect)?;
    let native = native_clients::inspect_native_candidates().ok();
    let isolated = native_clients::inspect_sandbox_client_02().ok();
    Ok(project_overview(native.as_ref(), isolated.as_ref()))
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
            candidates: (1..=candidates as u32)
                .map(|pid| NativeProcessCandidate {
                    pid,
                    window_handle: None,
                })
                .collect(),
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
        let result = project_overview(Some(&native(2, true)), Some(&isolated("OBSERVED", 1)));
        assert_eq!(result.schema, 1);
        assert_eq!(
            result.slots.iter().map(|slot| slot.id).collect::<Vec<_>>(),
            ["01", "02", "03", "04"]
        );
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
    fn independent_collector_failure_preserves_only_its_peers_evidence() {
        let safe_native = native(2, true);
        let safe_isolated = isolated("OBSERVED", 1);
        let only_native = project_overview(Some(&safe_native), None);
        assert_eq!(only_native.slots[0].evidence, "NAME_MATCH_UNATTRIBUTED");
        assert_eq!(only_native.slots[0].candidate_count, Some(2));
        assert_eq!(only_native.slots[1].evidence, "UNKNOWN");
        assert_eq!(only_native.slots[1].candidate_count, None);
        let only_isolated = project_overview(None, Some(&safe_isolated));
        assert_eq!(only_isolated.slots[0].evidence, "UNKNOWN");
        assert_eq!(only_isolated.slots[0].candidate_count, None);
        assert_eq!(
            only_isolated.slots[1].evidence,
            "BOX_PROCESS_CANDIDATES_ONLY"
        );
        assert_eq!(only_isolated.slots[1].candidate_count, Some(1));
        let neither = project_overview(None, None);
        for slot in &neither.slots[..2] {
            assert_eq!(slot.evidence, "UNKNOWN");
            assert_eq!(slot.candidate_count, None);
        }
        for slot in &neither.slots[2..] {
            assert_eq!(slot.evidence, "PROVIDER_NOT_AVAILABLE");
            assert_eq!(slot.candidate_count, None);
        }
    }

    #[test]
    fn zero_candidates_is_observed_not_verified_stopped() {
        let result = project_overview(Some(&native(0, true)), Some(&isolated("OBSERVED", 0)));
        assert_eq!(result.slots[0].candidate_count, Some(0));
        assert_eq!(result.slots[1].candidate_count, Some(0));
        assert_eq!(result.slots[1].evidence, "BOX_PROCESS_CANDIDATES_ONLY");
    }

    #[test]
    fn ambiguous_or_noninteractive_observations_never_mean_empty() {
        let result = project_overview(Some(&native(0, false)), Some(&isolated("UNKNOWN", 0)));
        assert_eq!(result.slots[0].evidence, "UNKNOWN");
        assert_eq!(result.slots[0].candidate_count, None);
        assert_eq!(result.slots[1].evidence, "UNKNOWN");
        assert_eq!(result.slots[1].candidate_count, None);
        let mut unsafe_box = isolated("OBSERVED", 1);
        unsafe_box.box_settings_source = "UNKNOWN".into();
        let result = project_overview(Some(&native(1, true)), Some(&unsafe_box));
        assert_eq!(result.slots[1].evidence, "UNKNOWN");
        assert_eq!(result.slots[1].candidate_count, None);
    }
}
