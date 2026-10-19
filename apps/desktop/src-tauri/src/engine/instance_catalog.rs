//! Read-only identity and route catalog for exactly four Minecraft slots.
//!
//! This does NOT duplicate the runtime lifecycle, discover processes, create
//! Sandboxie boxes or authorize a game launch. It describes mounted routes
//! and explicitly unavailable future v1.18.6 slots without any background work.
use serde::Serialize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceSlotDescriptor {
    pub id: &'static str,
    pub kind: &'static str,
    pub route: &'static str,
    pub availability: &'static str,
}

pub fn instance_slots() -> [InstanceSlotDescriptor; 4] {
    [
        InstanceSlotDescriptor {
            id: "01",
            kind: "NATIVE",
            route: "OFFICIAL_WINDOWS",
            availability: "PILOT_ROUTE_ONLY",
        },
        InstanceSlotDescriptor {
            id: "02",
            kind: "ISOLATED",
            route: "OFFICIAL_SANDBOXIE",
            availability: "PILOT_ROUTE_ONLY",
        },
        InstanceSlotDescriptor {
            id: "03",
            kind: "ISOLATED",
            route: "OWNED_V1186",
            availability: "NOT_AVAILABLE",
        },
        InstanceSlotDescriptor {
            id: "04",
            kind: "ISOLATED",
            route: "OWNED_V1186",
            availability: "NOT_AVAILABLE",
        },
    ]
}

// One admission table for the four stable slots. It does not assert whether
// the game is running; the existing provider performs fresh Windows checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotOperation {
    Inspect,
    SelectExecutable,
    Launch,
    Focus,
    Place,
    Provision,
    ApplyFrameCap,
}

pub fn admit_slot_operation(id: &str, operation: SlotOperation) -> std::io::Result<()> {
    use std::io::{Error, ErrorKind};
    let slot = instance_slots()
        .into_iter()
        .find(|slot| slot.id == id)
        .ok_or_else(|| Error::new(ErrorKind::InvalidInput, "Unknown Minecraft instance slot"))?;

    // An uninstalled future provider cannot be promoted by UI data or direct
    // Tauri invocation. Provisioning and FPS control have no active adapter.
    if slot.availability != "PILOT_ROUTE_ONLY" {
        return Err(Error::new(ErrorKind::Unsupported,
            "Owned v1.18.6 runtime is not installed and verified; slot unavailable"));
    }
    let allowed = matches!(
        (slot.id, operation),
        ("01", SlotOperation::Inspect | SlotOperation::Launch)
        | ("02", SlotOperation::Inspect | SlotOperation::SelectExecutable
            | SlotOperation::Launch | SlotOperation::Focus | SlotOperation::Place)
    );
    if !allowed {
        return Err(Error::new(ErrorKind::Unsupported,
            "This slot operation has no verified active provider route"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_registry_describes_four_distinct_slots_without_claiming_gameplay() {
        let slots = instance_slots();
        assert_eq!(slots.map(|slot| slot.id), ["01", "02", "03", "04"]);
        assert_eq!(slots[0].kind, "NATIVE");
        assert!(slots[1..].iter().all(|slot| slot.kind == "ISOLATED"));
        assert_eq!(slots[1].route, "OFFICIAL_SANDBOXIE");
        assert!(slots[..2].iter().all(|slot| slot.availability == "PILOT_ROUTE_ONLY"));
        assert!(slots[2..].iter().all(|slot| {
            slot.route == "OWNED_V1186" && slot.availability == "NOT_AVAILABLE"
        }));
    }


    #[test]
    fn all_slot_operations_have_a_single_fail_closed_admission_table() {
        use SlotOperation::*;
        for op in [Inspect, SelectExecutable, Launch, Focus, Place, Provision, ApplyFrameCap] {
            assert_eq!(admit_slot_operation("01", op).is_ok(),
                matches!(op, Inspect | Launch));
            assert_eq!(admit_slot_operation("02", op).is_ok(),
                matches!(op, Inspect | SelectExecutable | Launch | Focus | Place));
            for future in ["03", "04"] {
                assert_eq!(admit_slot_operation(future, op).unwrap_err().kind(),
                    std::io::ErrorKind::Unsupported);
            }
        }
        for unknown in ["", "1", "05", "00", "Client02", "02 ", "02\\0"] {
            assert_eq!(admit_slot_operation(unknown, Inspect).unwrap_err().kind(),
                std::io::ErrorKind::InvalidInput);
        }
    }

    #[test]
    fn serialized_catalog_is_read_only_metadata_without_window_or_login_claims() {
        let json = serde_json::to_value(instance_slots()).expect("serialize four slot identities");
        let slots = json.as_array().expect("one catalog");
        assert_eq!(slots.len(), 4);
        for (index, slot) in slots.iter().enumerate() {
            assert_eq!(slot["id"], format!("{:02}", index + 1));
            let fields = slot.as_object().expect("slot fields");
            assert_eq!(fields.len(), 4);
            assert!(fields.contains_key("kind"));
            assert!(fields.contains_key("route"));
            assert!(fields.contains_key("availability"));
            assert!(!fields.contains_key("running"));
            assert!(!fields.contains_key("authenticated"));
        }
    }
}
