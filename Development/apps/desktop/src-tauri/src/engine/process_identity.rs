//! Metadata-only identity of a known Minecraft process and its observed native window.
//! OS handle and birth-time verification remain the responsibility of the
//! owning session controller/window adapter, never this transport value.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OwnedProcess {
    pub pid: u32,
    pub created_file_time: String,
    pub session_id: u32,
    pub window_handle: String,
    pub responding: bool,
}

#[cfg(test)]
mod tests {
    use super::OwnedProcess;

    #[test]
    fn identity_wire_preserves_existing_fields_and_rejects_unexpected_metadata() {
        let original = r#"{"pid":42,"createdFileTime":"134000000000000001","sessionId":1,"windowHandle":"0x123","responding":true}"#;
        let process: OwnedProcess = serde_json::from_str(original).unwrap();
        assert_eq!(process.created_file_time, "134000000000000001");
        assert_eq!(serde_json::to_value(&process).unwrap()["windowHandle"], "0x123");
        assert!(serde_json::from_str::<OwnedProcess>(
            r#"{"pid":42,"createdFileTime":"1","sessionId":1,"windowHandle":"0x1","responding":true,"legacyBox":"Sandboxie"}"#
        ).is_err());
    }
}
