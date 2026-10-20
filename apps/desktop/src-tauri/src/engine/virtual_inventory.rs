//! Read-only Sandboxie configuration section-name candidates for the desktop.
//! A section-name candidate is NOT proof of an actual box, a MIVUBI virtual, a running
//! process, a verified Microsoft account, or a provider migration.

use serde::{Deserialize, Serialize};
use std::{collections::HashSet, io};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VirtualBoxInventory {
    pub schema: u32,
    pub source: String,
    pub evidence: String,
    pub boxes: Vec<String>,
    pub owned_provider_selected: bool,
    pub gameplay_verified: bool,
}

fn invalid_inventory() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Untrusted or incomplete sandbox inventory",
    )
}

/// Rejects any untrusted output and refuses to fabricate a current state.
pub fn parse_virtual_inventory(raw: &str) -> io::Result<VirtualBoxInventory> {
    if raw.len() > 8192 {
        return Err(invalid_inventory());
    }
    let record: VirtualBoxInventory = serde_json::from_str(raw).map_err(|_| invalid_inventory())?;
    if record.schema != 1
        || record.source != "EXISTING_OFFICIAL_SANDBOXIE_PILOT"
        || record.evidence != "SECTION_NAME_CANDIDATES"
        || record.owned_provider_selected
        || record.gameplay_verified
        || record.boxes.len() > 128
    {
        return Err(invalid_inventory());
    }
    let mut seen = HashSet::new();
    for box_name in &record.boxes {
        if box_name.is_empty()
            || box_name.len() > 32
            || !box_name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            || !seen.insert(box_name.to_ascii_lowercase())
        {
            return Err(invalid_inventory());
        }
    }
    Ok(record)
}

#[cfg(windows)]
pub fn discover_existing_virtuals() -> io::Result<VirtualBoxInventory> {
    use std::time::Duration;
    // Signed SbieIni discovery is read-only and restricted to known Program
    // Files roots by the embedded source; no user-input box names or paths.
    const SCRIPT: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../tooling/acceptance/windows/collect-virtual-boxes.ps1"
    ));
    let raw = virtual_clients_core::bounded_command_output(
        &super::native_clients::system_powershell()?,
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            SCRIPT,
        ],
        Duration::from_secs(15),
    )?;
    parse_virtual_inventory(&raw)
}

#[cfg(not(windows))]
pub fn discover_existing_virtuals() -> io::Result<VirtualBoxInventory> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Existing Windows sandbox discovery requires Windows",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inventory(names: &[&str]) -> String {
        serde_json::json!({
            "schema": 1,
            "source": "EXISTING_OFFICIAL_SANDBOXIE_PILOT",
            "evidence": "SECTION_NAME_CANDIDATES",
            "boxes": names,
            "ownedProviderSelected": false,
            "gameplayVerified": false
        })
        .to_string()
    }

    #[test]
    fn exact_valid_existing_inventory_does_not_claim_owned_runtime() {
        let value =
            parse_virtual_inventory(&inventory(&["MinecraftClient02", "OtherBox"])).unwrap();
        assert_eq!(value.boxes.len(), 2);
        assert!(!value.owned_provider_selected && !value.gameplay_verified);
    }

    #[test]
    fn no_boxes_is_an_observation_not_a_mivubi_create_result() {
        assert!(parse_virtual_inventory(&inventory(&[]))
            .unwrap()
            .boxes
            .is_empty());
    }

    #[test]
    fn invalid_or_ambiguous_names_fail_closed() {
        for names in [
            vec!["A", "a"],
            vec!["Bad Box"],
            vec!["../DefaultBox"],
            vec!["A\\B"],
            vec![""],
            vec!["a:b"],
        ] {
            assert!(parse_virtual_inventory(&inventory(&names)).is_err());
        }
    }

    #[test]
    fn rejects_fake_owned_provider_gameplay_and_unknown_fields() {
        let base = inventory(&["MinecraftClient02"]);
        for (field, value) in [("ownedProviderSelected", true), ("gameplayVerified", true)] {
            let mut row: serde_json::Value = serde_json::from_str(&base).unwrap();
            row[field] = serde_json::json!(value);
            assert!(parse_virtual_inventory(&row.to_string()).is_err());
        }
        let mut old_schema: serde_json::Value = serde_json::from_str(&base).unwrap();
        old_schema["evidence"] = serde_json::json!("BOX_NAMES_ONLY");
        assert!(parse_virtual_inventory(&old_schema.to_string()).is_err());
        let mut row: serde_json::Value = serde_json::from_str(&base).unwrap();
        row["userIdentity"] = serde_json::json!("not allowed");
        assert!(parse_virtual_inventory(&row.to_string()).is_err());
        row.as_object_mut().unwrap().remove("userIdentity");
        row["source"] = serde_json::json!("MIVUBI_OWNED_READY");
        assert!(parse_virtual_inventory(&row.to_string()).is_err());
    }
}
