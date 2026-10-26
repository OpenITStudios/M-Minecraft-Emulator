use serde::Serialize;
use std::{io, path::Path, time::Duration};

use crate::{
    client::ClientId,
    provider::{command_output_with_timeout, current_platform_provider},
    runtime::RuntimeStatus,
};
use sysinfo::System;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostDiagnostics {
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub cpu: Option<String>,
    pub logical_cpus: usize,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub graphics: Vec<String>,
    pub hypervisor_present: Option<bool>,
    pub vbs_status: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDiagnostics {
    pub id: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualHardwareDiagnostics {
    pub id: &'static str,
    pub network_mode: Option<String>,
    pub graphics_3d_enabled: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsReport {
    pub app_version: &'static str,
    pub host: HostDiagnostics,
    pub provider: ProviderDiagnostics,
    pub runtime: RuntimeStatus,
    pub virtual_hardware: Vec<VirtualHardwareDiagnostics>,
}

pub fn collect(runtime: RuntimeStatus) -> io::Result<DiagnosticsReport> {
    let provider = current_platform_provider();

    let provider_diagnostics = ProviderDiagnostics {
        id: provider.as_ref().map(|provider| provider.id().to_string()),
        version: provider.as_ref().and_then(|provider| provider.version()),
    };

    let virtual_hardware = if let Some(provider) = provider.as_ref() {
        ClientId::VIRTUAL
            .into_iter()
            .map(|client| {
                let network_mode = provider.network_mode(client).ok().flatten();
                let graphics_3d_enabled = provider.graphics_3d_enabled(client).ok().flatten();
                VirtualHardwareDiagnostics {
                    id: client.as_str(),
                    network_mode,
                    graphics_3d_enabled,
                }
            })
            .collect()
    } else {
        ClientId::VIRTUAL
            .into_iter()
            .map(|client| VirtualHardwareDiagnostics {
                id: client.as_str(),
                network_mode: None,
                graphics_3d_enabled: None,
            })
            .collect()
    };

    Ok(DiagnosticsReport {
        app_version: env!("CARGO_PKG_VERSION"),
        host: host_diagnostics(),
        provider: provider_diagnostics,
        runtime,
        virtual_hardware,
    })
}

fn host_diagnostics() -> HostDiagnostics {
    let mut system = System::new_all();
    system.refresh_all();

    let virtualization = windows_virtualization_state();

    HostDiagnostics {
        os: System::name(),
        os_version: System::os_version(),
        cpu: system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().trim().to_string())
            .filter(|cpu| !cpu.is_empty()),
        logical_cpus: system.cpus().len(),
        total_memory_mb: system.total_memory() / 1024 / 1024,
        available_memory_mb: system.available_memory() / 1024 / 1024,
        graphics: graphics_summary(),
        hypervisor_present: virtualization.map(|state| state.0),
        vbs_status: virtualization.and_then(|state| state.1),
    }
}

#[cfg(target_os = "windows")]
fn graphics_summary() -> Vec<String> {
    let output = command_output_with_timeout(
        Path::new("powershell.exe"),
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            r#"Get-CimInstance Win32_VideoController | ForEach-Object { "$($_.Name)|$($_.DriverVersion)" }"#,
        ],
        Duration::from_secs(12),
    );
    match output {
        Ok(output) => output
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .take(8)
            .map(str::to_string)
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(target_os = "macos")]
fn graphics_summary() -> Vec<String> {
    let output = command_output_with_timeout(
        Path::new("system_profiler"),
        ["SPDisplaysDataType"],
        Duration::from_secs(20),
    );
    match output {
        Ok(output) => output
            .lines()
            .map(str::trim)
            .filter(|line| {
                line.starts_with("Chipset Model:")
                    || line.starts_with("Metal Support:")
                    || line.starts_with("VRAM")
            })
            .take(8)
            .map(str::to_string)
            .collect(),
        Err(_) => Vec::new(),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn graphics_summary() -> Vec<String> {
    Vec::new()
}

#[cfg(target_os = "windows")]
fn windows_virtualization_state() -> Option<(bool, Option<u32>)> {
    let script = r#"$h=(Get-CimInstance Win32_ComputerSystem).HypervisorPresent; $v=(Get-CimInstance -Namespace root\Microsoft\Windows\DeviceGuard -ClassName Win32_DeviceGuard -ErrorAction SilentlyContinue).VirtualizationBasedSecurityStatus; Write-Output "$h|$v""#;
    let output = command_output_with_timeout(
        Path::new("powershell.exe"),
        [
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            script,
        ],
        Duration::from_secs(12),
    )
    .ok()?;
    let (hypervisor, vbs) = output.trim().split_once('|')?;
    let hypervisor = match hypervisor.trim().to_ascii_lowercase().as_str() {
        "true" => true,
        "false" => false,
        _ => return None,
    };
    let vbs = vbs.trim().parse::<u32>().ok();
    Some((hypervisor, vbs))
}

#[cfg(not(target_os = "windows"))]
fn windows_virtualization_state() -> Option<(bool, Option<u32>)> {
    None
}
