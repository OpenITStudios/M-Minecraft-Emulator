use crate::{guest::GUEST_AGENT_PORT, resources::VIRTUAL_MEMORY_LIMIT_MB};
use serde::Serialize;

pub const MAX_VIRTUAL_CLIENTS: usize = 3;
pub const VIRTUAL_VCPUS: u16 = 2;
pub const READY_SNAPSHOT_NAME: &str = "QA_READY";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnginePolicy {
    pub max_virtual_clients: usize,
    pub virtual_memory_limit_mb: u64,
    pub virtual_vcpus: u16,
    pub guest_agent_port: u16,
    pub ready_snapshot_name: &'static str,
    pub native_is_version_authority: bool,
    pub runtime_self_update_enabled: bool,
}

pub fn engine_policy() -> EnginePolicy {
    EnginePolicy {
        max_virtual_clients: MAX_VIRTUAL_CLIENTS,
        virtual_memory_limit_mb: VIRTUAL_MEMORY_LIMIT_MB,
        virtual_vcpus: VIRTUAL_VCPUS,
        guest_agent_port: GUEST_AGENT_PORT,
        ready_snapshot_name: READY_SNAPSHOT_NAME,
        native_is_version_authority: true,
        runtime_self_update_enabled: false,
    }
}

#[cfg(test)]
mod tests {
    use super::{engine_policy, MAX_VIRTUAL_CLIENTS, READY_SNAPSHOT_NAME, VIRTUAL_VCPUS};

    #[test]
    fn engine_policy_matches_product_invariants() {
        let policy = engine_policy();
        assert_eq!(policy.max_virtual_clients, MAX_VIRTUAL_CLIENTS);
        assert_eq!(policy.virtual_vcpus, VIRTUAL_VCPUS);
        assert_eq!(policy.ready_snapshot_name, READY_SNAPSHOT_NAME);
        assert!(policy.native_is_version_authority);
        assert!(!policy.runtime_self_update_enabled);
    }
}
