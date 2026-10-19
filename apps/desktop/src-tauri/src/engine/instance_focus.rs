//! One-shot read-only focus projection for four stable Minecraft slots.
//! Only Client 02 is currently attributable to a freshly verified window.
//! No event hook, FPS write, process priority, resident worker or game change.
use serde::Serialize;
use std::io;

use super::{
    instance_catalog::{admit_slot_operation, instance_slots, SlotOperation},
    native_clients::{self, ProcessIdentityCandidate},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SlotFocusEvidence {
    pub id: &'static str,
    pub evidence: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceFocusObservation {
    pub schema: u32,
    pub mode: &'static str,
    pub slots: [SlotFocusEvidence; 4],
}

fn classify_foreground(verified_hwnd: u64, foreground_hwnd: Option<u64>) -> &'static str {
    match foreground_hwnd {
        None | Some(0) => "UNKNOWN",
        Some(current) if verified_hwnd != 0 && current == verified_hwnd => "FOREGROUND",
        Some(_) if verified_hwnd != 0 => "NOT_FOREGROUND",
        Some(_) => "UNKNOWN",
    }
}

fn project_focus(client_02_evidence: &'static str) -> InstanceFocusObservation {
    let ids = instance_slots();
    InstanceFocusObservation {
        schema: 2,
        mode: "OBSERVE_ONLY",
        slots: [
            SlotFocusEvidence { id: ids[0].id, evidence: "UNATTRIBUTED" },
            SlotFocusEvidence { id: ids[1].id, evidence: client_02_evidence },
            SlotFocusEvidence { id: ids[2].id, evidence: "PROVIDER_NOT_AVAILABLE" },
            SlotFocusEvidence { id: ids[3].id, evidence: "PROVIDER_NOT_AVAILABLE" },
        ],
    }
}

#[cfg(windows)]
pub fn inspect_instance_focus(
    target: ProcessIdentityCandidate,
) -> io::Result<InstanceFocusObservation> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    admit_slot_operation("02", SlotOperation::Inspect)?;
    // One exact Client 02 identity check before a single Windows snapshot.
    // This is the same fresh Sandboxie/image/FILETIME/HWND ownership boundary
    // used for existing Focus/Place, NOT caller-provided attribution.
    let mut evidence = "UNKNOWN";
    native_clients::with_verified_client_02_window(target, |verified_hwnd| {
        let foreground = unsafe { GetForegroundWindow() };
        evidence = classify_foreground(
            verified_hwnd as usize as u64,
            if foreground.is_null() { None } else { Some(foreground as usize as u64) },
        );
        Ok(())
    })?;
    Ok(project_focus(evidence))
}

#[cfg(not(windows))]
pub fn inspect_instance_focus(
    _target: ProcessIdentityCandidate,
) -> io::Result<InstanceFocusObservation> {
    Err(io::Error::new(io::ErrorKind::Unsupported,
        "Four-slot focus inspection requires interactive Windows"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_only_the_freshly_verified_client_02_window() {
        assert_eq!(classify_foreground(0x200, Some(0x200)), "FOREGROUND");
        assert_eq!(classify_foreground(0x200, Some(0x300)), "NOT_FOREGROUND");
        assert_eq!(classify_foreground(0x200, None), "UNKNOWN");
        assert_eq!(classify_foreground(0x200, Some(0)), "UNKNOWN");
        assert_eq!(classify_foreground(0, Some(0x200)), "UNKNOWN");
    }

    #[test]
    fn one_snapshot_preserves_unattributed_and_unavailable_slots() {
        for client_02 in ["FOREGROUND", "NOT_FOREGROUND", "UNKNOWN"] {
            let focus = project_focus(client_02);
            assert_eq!(focus.schema, 2);
            assert_eq!(focus.mode, "OBSERVE_ONLY");
            assert_eq!(focus.slots.iter().map(|slot| slot.id).collect::<Vec<_>>(),
                ["01", "02", "03", "04"]);
            assert_eq!(focus.slots[0].evidence, "UNATTRIBUTED");
            assert_eq!(focus.slots[1].evidence, client_02);
            for later in &focus.slots[2..] {
                assert_eq!(later.evidence, "PROVIDER_NOT_AVAILABLE");
            }
            let payload = serde_json::to_value(focus).unwrap();
            assert_eq!(payload.as_object().unwrap().len(), 3);
            assert!(payload.get("fps").is_none());
            assert!(payload.get("running").is_none());
            assert!(payload.get("authenticated").is_none());
            assert!(payload.get("limiterEnabled").is_none());
        }
    }
}
