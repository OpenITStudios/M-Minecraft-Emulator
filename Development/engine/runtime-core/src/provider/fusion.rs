use super::{
    apply_virtual_hardware_policy, base_vmx_path, client_vmx_path, command_output,
    command_output_with_timeout, ensure_parent, guest_tools_state_ready, has_suspend_state,
    listed_as_running, parse_guest_ip, promote_staging_vm, read_vmx_memory, remove_vm_container,
    snapshot_list_contains, staging_client_vmx_path, vm_identity_key, vmx_file_present,
    wait_for_state, Provider, DISK_STATE_TIMEOUT,
};
use crate::client::{ClientId, ClientState};
use crate::policy::READY_SNAPSHOT_NAME;
use std::{io, path::Path, process::Command, time::Duration};

const VMRUN: &str = "/Applications/VMware Fusion.app/Contents/Library/vmrun";

// Never escalate a routine client or Base shutdown into a hard power-off.
const GRACEFUL_STOP_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Debug, Default)]
pub struct VmwareFusionProvider;

impl VmwareFusionProvider {
    fn vmrun(&self) -> &'static Path {
        Path::new(VMRUN)
    }

    fn require_client(&self, client: ClientId) -> io::Result<std::path::PathBuf> {
        let path = client_vmx_path(client)?;
        if vmx_file_present(&path)? {
            Ok(path)
        } else {
            Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not provisioned", client.as_str()),
            ))
        }
    }

    fn running(&self, vmx: &Path) -> io::Result<bool> {
        let output = command_output(self.vmrun(), ["-T", "fusion", "list"])?;
        Ok(listed_as_running(&output, vmx))
    }

    fn graceful_stop_vmx(&self, vmx: &Path) -> io::Result<()> {
        if !self.running(vmx)? {
            return if has_suspend_state(vmx) {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "VM is suspended. Resume explicitly before Stop; no VM was modified.",
                ))
            } else {
                Ok(())
            };
        }

        // A timeout can occur after VMware already changed guest power state.
        // Reconcile it, but never silently force-kill a guest with account data.
        let response = command_output_with_timeout(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "stop",
                vmx.to_string_lossy().as_ref(),
                "soft",
            ],
            GRACEFUL_STOP_TIMEOUT,
        );
        if let Err(error) = response {
            match self.running(vmx) {
                Ok(false) if !has_suspend_state(vmx) => return Ok(()),
                Ok(_) => return Err(io::Error::new(
                    error.kind(),
                    format!("Graceful stop failed: {error}. No hard stop attempted; inspect VM state."),
                )),
                Err(probe_error) => return Err(io::Error::new(
                    error.kind(),
                    format!("Graceful stop failed: {error}. VM state UNKNOWN ({probe_error}); no hard stop attempted."),
                )),
            }
        }

        wait_for_state(|| self.running(vmx), false, GRACEFUL_STOP_TIMEOUT)
            .map_err(|error| io::Error::new(
                error.kind(),
                format!("Graceful stop not confirmed: {error}. No hard stop attempted; inspect VM state."),
            ))?;
        if has_suspend_state(vmx) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VM entered Suspended state instead of Stopped; inspect VM state.",
            ));
        }
        Ok(())
    }
}

impl Provider for VmwareFusionProvider {
    fn id(&self) -> &'static str {
        "vmware-fusion"
    }

    fn version(&self) -> Option<String> {
        let version = command_output_with_timeout(
            Path::new("plutil"),
            [
                "-extract",
                "CFBundleShortVersionString",
                "raw",
                "-o",
                "-",
                "/Applications/VMware Fusion.app/Contents/Info.plist",
            ],
            Duration::from_secs(10),
        )
        .ok()?;
        (!version.is_empty()).then_some(version)
    }

    fn detect(&self) -> bool {
        self.vmrun().is_file()
    }

    fn host_working_sets_mb(&self) -> io::Result<Vec<(ClientId, u64)>> {
        Ok(super::host_working_sets_mb())
    }

    fn base_finalized_and_stopped(&self) -> io::Result<bool> {
        super::vmware_base_finalized_and_stopped(self)
    }

    fn network_mode(&self, client: ClientId) -> io::Result<Option<String>> {
        super::vmware_network_mode(client)
    }

    fn graphics_3d_enabled(&self, client: ClientId) -> io::Result<Option<bool>> {
        super::vmware_graphics_3d_enabled(client)
    }

    fn is_running_path(&self, vmx: &Path) -> io::Result<bool> {
        self.running(vmx)
    }

    fn start_validation_vm(&self, vmx: &Path) -> io::Result<()> {
        if self.running(vmx)? {
            return Ok(());
        }
        command_output_with_timeout(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "start",
                vmx.to_string_lossy().as_ref(),
                "nogui",
            ],
            DISK_STATE_TIMEOUT,
        )?;
        wait_for_state(|| self.running(vmx), true, DISK_STATE_TIMEOUT)
    }

    fn stop_validation_vm(&self, vmx: &Path) -> io::Result<()> {
        self.graceful_stop_vmx(vmx)
    }

    fn guest_ip_for_path(&self, vmx: &Path) -> io::Result<Option<String>> {
        if !self.running(vmx)? {
            return Ok(None);
        }

        match command_output_with_timeout(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "getGuestIPAddress",
                vmx.to_string_lossy().as_ref(),
            ],
            Duration::from_secs(5),
        ) {
            Ok(output) => Ok(parse_guest_ip(&output)),
            Err(_) => Ok(None),
        }
    }

    fn provision(&self, client: ClientId) -> io::Result<ClientState> {
        let target = client_vmx_path(client)?;
        if vmx_file_present(&target)? {
            return self.status(client);
        }

        let base = base_vmx_path()?;
        if !vmx_file_present(&base)? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("base VM is missing: {}", base.display()),
            ));
        }
        if self.running(&base)? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "base VM must be powered off before provisioning",
            ));
        }

        let staging = staging_client_vmx_path(client)?;
        remove_vm_container(&staging)?;
        ensure_parent(&staging)?;

        let clone_name = format!("-cloneName={}", client.as_str());
        let clone_result = command_output_with_timeout(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "clone",
                base.to_string_lossy().as_ref(),
                staging.to_string_lossy().as_ref(),
                "linked",
                clone_name.as_str(),
            ],
            DISK_STATE_TIMEOUT,
        );

        if let Err(error) = clone_result {
            // Failed VMware operations may leave uncertain disk work behind.
            return Err(error);
        }

        if let Err(error) = apply_virtual_hardware_policy(&staging) {
            let _ = remove_vm_container(&staging);
            return Err(error);
        }

        if let Err(error) = promote_staging_vm(&staging, &target) {
            let _ = remove_vm_container(&staging);
            return Err(error);
        }

        Ok(ClientState::Stopped)
    }

    fn status(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = client_vmx_path(client)?;
        if !vmx_file_present(&vmx)? {
            return Ok(ClientState::NotProvisioned);
        }
        Ok(if self.running(&vmx)? {
            ClientState::Running
        } else if has_suspend_state(&vmx) {
            ClientState::Suspended
        } else {
            ClientState::Stopped
        })
    }

    fn reprovision(&self, _client: ClientId) -> io::Result<ClientState> {
        // Public runtime also rejects reprovision; this closes provider bypasses.
        // Never delete an account-bearing original before proving replacement.
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Reprovision is disabled until a recoverable, account-safe cutover is proven.",
        ))
    }

    fn memory_limit_mb(&self, client: ClientId) -> io::Result<u64> {
        let vmx = self.require_client(client)?;
        read_vmx_memory(&vmx)
    }

    fn guest_tools_ready(&self, client: ClientId) -> io::Result<Option<bool>> {
        let vmx = self.require_client(client)?;
        if !self.running(&vmx)? {
            return Ok(Some(false));
        }

        match command_output(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "checkToolsState",
                vmx.to_string_lossy().as_ref(),
            ],
        ) {
            Ok(state) => Ok(Some(guest_tools_state_ready(&state))),
            Err(_) => Ok(None),
        }
    }

    fn guest_status(
        &self,
        client: ClientId,
        timeout: Duration,
    ) -> io::Result<Option<crate::guest::GuestStatus>> {
        super::vmware_guest_status(self, client, timeout)
    }

    fn launch_minecraft(&self, client: ClientId, timeout: Duration) -> io::Result<()> {
        super::vmware_launch_minecraft(self, client, timeout)
    }

    fn identity_key(&self, client: ClientId) -> io::Result<Option<String>> {
        let vmx = self.require_client(client)?;
        vm_identity_key(&vmx)
    }

    fn start(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = self.require_client(client)?;
        if self.running(&vmx)? {
            return Ok(ClientState::Running);
        }

        let resume = has_suspend_state(&vmx);
        if resume {
            command_output_with_timeout(
                self.vmrun(),
                [
                    "-T",
                    "fusion",
                    "start",
                    vmx.to_string_lossy().as_ref(),
                    "gui",
                ],
                DISK_STATE_TIMEOUT,
            )?;
            wait_for_state(|| self.running(&vmx), true, DISK_STATE_TIMEOUT)?;
        } else {
            command_output(
                self.vmrun(),
                [
                    "-T",
                    "fusion",
                    "start",
                    vmx.to_string_lossy().as_ref(),
                    "gui",
                ],
            )?;
            wait_for_state(|| self.running(&vmx), true, Duration::from_secs(15))?;
        }
        Ok(ClientState::Running)
    }

    fn suspend(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = self.require_client(client)?;
        if !self.running(&vmx)? {
            return Ok(if has_suspend_state(&vmx) {
                ClientState::Suspended
            } else {
                ClientState::Stopped
            });
        }

        command_output_with_timeout(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "suspend",
                vmx.to_string_lossy().as_ref(),
                "soft",
            ],
            DISK_STATE_TIMEOUT,
        )?;
        wait_for_state(|| self.running(&vmx), false, DISK_STATE_TIMEOUT)?;
        Ok(ClientState::Suspended)
    }

    fn stop(&self, client: ClientId) -> io::Result<ClientState> {
        self.graceful_stop_vmx(&self.require_client(client)?)?;
        Ok(ClientState::Stopped)
    }

    fn restart(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = self.require_client(client)?;
        if !self.running(&vmx)? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{} is not running", client.as_str()),
            ));
        }

        command_output(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "reset",
                vmx.to_string_lossy().as_ref(),
                "soft",
            ],
        )?;
        wait_for_state(|| self.running(&vmx), true, Duration::from_secs(15))?;
        Ok(ClientState::Running)
    }

    fn has_ready(&self, client: ClientId) -> io::Result<bool> {
        let vmx = self.require_client(client)?;
        let output = command_output(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "listSnapshots",
                vmx.to_string_lossy().as_ref(),
            ],
        )?;
        Ok(snapshot_list_contains(&output, READY_SNAPSHOT_NAME))
    }

    fn set_ready(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = self.require_client(client)?;
        if self.running(&vmx)? || has_suspend_state(&vmx) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{} must be fully stopped before setting QA_READY",
                    client.as_str()
                ),
            ));
        }
        if self.has_ready(client)? {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("{} already has QA_READY", client.as_str()),
            ));
        }

        command_output_with_timeout(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "snapshot",
                vmx.to_string_lossy().as_ref(),
                READY_SNAPSHOT_NAME,
            ],
            DISK_STATE_TIMEOUT,
        )?;
        Ok(ClientState::Stopped)
    }

    fn reset(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = self.require_client(client)?;
        if !self.has_ready(client)? {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} has no QA_READY snapshot", client.as_str()),
            ));
        }

        // Mirror public admission for direct provider callers.
        if has_suspend_state(&vmx) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Resume the suspended Virtual and shut it down before restoring QA_READY.",
            ));
        }
        if self.running(&vmx)? {
            self.stop(client)?;
        }

        command_output_with_timeout(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "revertToSnapshot",
                vmx.to_string_lossy().as_ref(),
                READY_SNAPSHOT_NAME,
            ],
            DISK_STATE_TIMEOUT,
        )?;
        command_output(
            self.vmrun(),
            [
                "-T",
                "fusion",
                "start",
                vmx.to_string_lossy().as_ref(),
                "gui",
            ],
        )?;
        wait_for_state(|| self.running(&vmx), true, Duration::from_secs(15))?;
        Ok(ClientState::Running)
    }

    fn open(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = self.require_client(client)?;
        self.open_vm_ui(&vmx)?;
        self.status(client)
    }

    fn open_vm_ui(&self, vmx: &Path) -> io::Result<()> {
        Command::new("open").arg(vmx).spawn()?;
        Ok(())
    }
}
