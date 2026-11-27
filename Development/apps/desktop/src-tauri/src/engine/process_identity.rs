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

impl OwnedProcess {
    // Exact wire identity, not liveness, responsiveness or OS verification.
    // Callers still own slot, native handle and presentation admission.
    pub(crate) fn same_window_identity(&self, other: &Self) -> bool {
        self.pid == other.pid
            && self.created_file_time == other.created_file_time
            && self.session_id == other.session_id
            && self.window_handle == other.window_handle
    }
}

#[cfg(test)]
mod tests {
    use super::OwnedProcess;

    #[test]
    fn window_identity_is_exact_and_independent_of_responsiveness() {
        let target = OwnedProcess {
            pid: 42, created_file_time: "134000000000000001".into(),
            session_id: 1, window_handle: "0xabc".into(), responding: true,
        };
        assert!(target.same_window_identity(&target));
        let mut other = target.clone();
        other.responding = false;
        assert!(target.same_window_identity(&other));
        other = target.clone(); other.pid += 1;
        assert!(!target.same_window_identity(&other));
        other = target.clone(); other.created_file_time.push('0');
        assert!(!target.same_window_identity(&other));
        other = target.clone(); other.session_id += 1;
        assert!(!target.same_window_identity(&other));
        other = target.clone(); other.window_handle = "0xdef".into();
        assert!(!target.same_window_identity(&other));
        other.window_handle = "0xABC".into();
        assert!(!target.same_window_identity(&other));
    }

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
