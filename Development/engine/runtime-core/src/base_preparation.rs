use crate::{
    profile::{current_base_vmx_path, native_minecraft_profile, BaseState},
    provider::{base_state_for_path, current_platform_provider, read_vmx_value, vmx_file_present},
};
use serde::Serialize;
use std::io;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BasePreparationReport {
    pub platform: &'static str,
    pub provider: Option<String>,
    pub provider_version: Option<String>,
    pub native_version: Option<String>,
    pub native_install_type: Option<String>,
    pub base_expected_path: Option<String>,
    pub base_present: bool,
    pub base_stopped: Option<bool>,
    pub base_state: Option<BaseState>,
    pub configured_memory_mb: Option<u64>,
    pub configured_vcpus: Option<u16>,
    pub graphics_3d_enabled: Option<bool>,
    pub network_present: Option<bool>,
    pub network_start_connected: Option<bool>,
    pub network_connection_type: Option<String>,
}

pub fn inspect_base_preparation() -> io::Result<BasePreparationReport> {
    let native = native_minecraft_profile();
    let provider = current_platform_provider();
    let base = current_base_vmx_path().ok();
    let base_present = base
        .as_ref()
        .is_some_and(|path| vmx_file_present(path).unwrap_or(false));

    let base_stopped = match (provider.as_ref(), base.as_ref()) {
        (Some(provider), Some(path)) if base_present => {
            provider.is_running_path(path).ok().map(|running| !running)
        }
        _ => None,
    };

    let base_state = base
        .as_ref()
        .filter(|_| base_present)
        .and_then(|path| base_state_for_path(path).ok())
        .flatten();

    let vmx = base.as_ref().filter(|_| base_present);
    let configured_memory_mb = vmx
        .and_then(|path| read_vmx_value(path, "memsize").ok().flatten())
        .and_then(|value| value.parse::<u64>().ok());
    let configured_vcpus = vmx
        .and_then(|path| read_vmx_value(path, "numvcpus").ok().flatten())
        .and_then(|value| value.parse::<u16>().ok());
    let graphics_3d_enabled = vmx
        .and_then(|path| read_vmx_value(path, "mks.enable3d").ok().flatten())
        .map(|value| value.eq_ignore_ascii_case("TRUE"));
    let network_present = vmx
        .and_then(|path| read_vmx_value(path, "ethernet0.present").ok().flatten())
        .map(|value| value.eq_ignore_ascii_case("TRUE"));
    let network_start_connected = vmx
        .and_then(|path| {
            read_vmx_value(path, "ethernet0.startConnected")
                .ok()
                .flatten()
        })
        .map(|value| value.eq_ignore_ascii_case("TRUE"));
    let network_connection_type = vmx.and_then(|path| {
        read_vmx_value(path, "ethernet0.connectionType")
            .ok()
            .flatten()
    });

    Ok(BasePreparationReport {
        platform: std::env::consts::OS,
        provider: provider.as_ref().map(|provider| provider.id().to_string()),
        provider_version: provider.as_ref().and_then(|provider| provider.version()),
        native_version: native.as_ref().map(|profile| profile.version.clone()),
        native_install_type: native
            .as_ref()
            .map(|profile| format!("{:?}", profile.install_type).to_ascii_uppercase()),
        base_expected_path: base.map(|path| path.display().to_string()),
        base_present,
        base_stopped,
        base_state,
        configured_memory_mb,
        configured_vcpus,
        graphics_3d_enabled,
        network_present,
        network_start_connected,
        network_connection_type,
    })
}

#[cfg(test)]
mod tests {
    use super::BasePreparationReport;

    #[test]
    fn report_is_read_only_fact_shape() {
        let report = BasePreparationReport {
            platform: "windows",
            provider: Some("vmware-workstation".into()),
            provider_version: Some("17".into()),
            native_version: Some("1.21.120.0".into()),
            native_install_type: Some("DESKTOP".into()),
            base_expected_path: Some("C:/Base/Base.vmx".into()),
            base_present: true,
            base_stopped: Some(true),
            base_state: None,
            configured_memory_mb: Some(4096),
            configured_vcpus: Some(2),
            graphics_3d_enabled: Some(true),
            network_present: Some(true),
            network_start_connected: Some(true),
            network_connection_type: Some("nat".into()),
        };
        let json = serde_json::to_value(report).unwrap();
        assert_eq!(json["basePresent"], true);
        assert!(json.get("ready").is_none());
        assert!(json.get("nextAction").is_none());
    }
}
