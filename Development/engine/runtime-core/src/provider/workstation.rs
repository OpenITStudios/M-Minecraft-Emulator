use super::{
    apply_virtual_hardware_policy, base_vmx_path, client_vmx_path, command_output,
    command_output_with_timeout, ensure_parent, guest_tools_state_ready, has_suspend_state,
    listed_as_running, parse_guest_ip, read_vmx_memory, snapshot_list_contains, vm_identity_key,
    vmx_file_present, wait_for_state, Provider, DISK_STATE_TIMEOUT,
};
use crate::client::{ClientId, ClientState};
use crate::policy::READY_SNAPSHOT_NAME;
use std::{
    io,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

const VMRUN_CANDIDATES: [&str; 2] = [
    r"C:\Program Files (x86)\VMware\VMware Workstation\vmrun.exe",
    r"C:\Program Files\VMware\VMware Workstation\vmrun.exe",
];

const GUI_CANDIDATES: [&str; 2] = [
    r"C:\Program Files (x86)\VMware\VMware Workstation\vmware.exe",
    r"C:\Program Files\VMware\VMware Workstation\vmware.exe",
];

// Never escalate a routine client or Base shutdown into a hard power-off.
const GRACEFUL_STOP_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Debug, Default)]
pub struct VmwareWorkstationProvider;

impl VmwareWorkstationProvider {
    fn vmrun(&self) -> Option<&'static Path> {
        VMRUN_CANDIDATES
            .iter()
            .map(Path::new)
            .find(|path| path.is_file())
    }

    fn gui(&self) -> Option<&'static Path> {
        GUI_CANDIDATES
            .iter()
            .map(Path::new)
            .find(|path| path.is_file())
    }

    fn require_vmrun(&self) -> io::Result<&'static Path> {
        self.vmrun().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "VMware Workstation vmrun.exe was not found",
            )
        })
    }

    fn require_client(&self, client: ClientId) -> io::Result<PathBuf> {
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
        let output = command_output(self.require_vmrun()?, ["-T", "ws", "list"])?;
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
            self.require_vmrun()?,
            ["-T", "ws", "stop", vmx.to_string_lossy().as_ref(), "soft"],
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

impl Provider for VmwareWorkstationProvider {
    fn id(&self) -> &'static str {
        "vmware-workstation"
    }

    fn version(&self) -> Option<String> {
        let gui = self.gui()?;
        let version = command_output_with_timeout(gui, ["-v"], Duration::from_secs(10)).ok()?;
        (!version.is_empty()).then_some(version)
    }

    fn detect(&self) -> bool {
        self.vmrun().is_some()
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
            self.require_vmrun()?,
            ["-T", "ws", "start", vmx.to_string_lossy().as_ref(), "nogui"],
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
            self.require_vmrun()?,
            [
                "-T",
                "ws",
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

        // Linked clone disk parent paths can depend on the destination
        // directory. Clone directly to the final VMX: no unverified relocation.
        // If a previous operation left partial data, preserve it and fail closed.
        if target.parent().is_some_and(Path::exists) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "{} has an existing VM directory without a complete VMX; inspect it before retrying",
                    client.as_str()
                ),
            ));
        }
        ensure_parent(&target)?;

        let clone_name = format!("-cloneName={}", client.as_str());
        command_output_with_timeout(
            self.require_vmrun()?,
            [
                "-T",
                "ws",
                "clone",
                base.to_string_lossy().as_ref(),
                target.to_string_lossy().as_ref(),
                "linked",
                clone_name.as_str(),
            ],
            DISK_STATE_TIMEOUT,
        )?;
        // Do not erase a failed clone or rewrite its disk metadata on error.
        if !vmx_file_present(&target)? {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "VMware clone returned success but no final VMX exists; inspect the destination",
            ));
        }
        apply_virtual_hardware_policy(&target)?;
        Ok(ClientState::Stopped)
    }

    fn observe_client_states(&self) -> io::Result<Vec<(ClientId, ClientState)>> {
        // vmrun list is a process spawn. Read it once for all provisioned
        // guests instead of re-running it for each row in a host snapshot.
        let mut entries = Vec::with_capacity(ClientId::VIRTUAL.len());
        let mut any_provisioned = false;
        for client in ClientId::VIRTUAL {
            let vmx = client_vmx_path(client)?;
            let present = vmx_file_present(&vmx)?;
            any_provisioned |= present;
            entries.push((client, vmx, present));
        }
        let running = if any_provisioned {
            Some(command_output(self.require_vmrun()?, ["-T", "ws", "list"])?)
        } else {
            None
        };
        Ok(entries
            .into_iter()
            .map(|(client, vmx, present)| {
                let state = if !present {
                    // An interrupted clone is not a clean missing guest.
                    if vmx.parent().is_some_and(Path::exists) {
                        ClientState::Error
                    } else {
                        ClientState::NotProvisioned
                    }
                } else if running
                    .as_ref()
                    .is_some_and(|list| listed_as_running(list, &vmx))
                {
                    ClientState::Running
                } else if has_suspend_state(&vmx) {
                    ClientState::Suspended
                } else {
                    ClientState::Stopped
                };
                (client, state)
            })
            .collect())
    }

    fn status(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = client_vmx_path(client)?;
        if !vmx_file_present(&vmx)? {
            // An existing container with no VMX may be an interrupted clone.
            // It is not equivalent to a clean, unprovisioned client.
            if vmx.parent().is_some_and(Path::exists) {
                // Expose incomplete clone residue through the existing ERROR
                // state. Doctor and lifecycle projections must not mistake
                // a provider diagnostic failure for clean NOT_PROVISIONED.
                return Ok(ClientState::Error);
            }
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
            self.require_vmrun()?,
            [
                "-T",
                "ws",
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
            let result = command_output_with_timeout(
                self.require_vmrun()?,
                ["-T", "ws", "start", vmx.to_string_lossy().as_ref(), "gui"],
                DISK_STATE_TIMEOUT,
            );
            if let Err(error) = result {
                // vmrun may time out after the resume mutation has already
                // completed. Reconcile observed provider state before surfacing
                // an unknown-mutation failure to the lifecycle owner.
                if self.running(&vmx).unwrap_or(false) {
                    return Ok(ClientState::Running);
                }
                return Err(error);
            }
            wait_for_state(|| self.running(&vmx), true, DISK_STATE_TIMEOUT)?;
        } else {
            let result = command_output(
                self.require_vmrun()?,
                ["-T", "ws", "start", vmx.to_string_lossy().as_ref(), "gui"],
            );
            if let Err(error) = result {
                if self.running(&vmx).unwrap_or(false) {
                    return Ok(ClientState::Running);
                }
                return Err(error);
            }
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

        let result = command_output_with_timeout(
            self.require_vmrun()?,
            [
                "-T",
                "ws",
                "suspend",
                vmx.to_string_lossy().as_ref(),
                "soft",
            ],
            DISK_STATE_TIMEOUT,
        );
        if let Err(error) = result {
            if !self.running(&vmx).unwrap_or(true) && has_suspend_state(&vmx) {
                return Ok(ClientState::Suspended);
            }
            return Err(error);
        }
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

        let result = command_output(
            self.require_vmrun()?,
            ["-T", "ws", "reset", vmx.to_string_lossy().as_ref(), "soft"],
        );
        // RUNNING was already true before reset, so it cannot prove that a
        // failed reset command actually restarted the guest.
        result?;
        wait_for_state(|| self.running(&vmx), true, Duration::from_secs(15))?;
        Ok(ClientState::Running)
    }

    fn has_ready(&self, client: ClientId) -> io::Result<bool> {
        let vmx = self.require_client(client)?;
        let output = command_output(
            self.require_vmrun()?,
            ["-T", "ws", "listSnapshots", vmx.to_string_lossy().as_ref()],
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

        let result = command_output_with_timeout(
            self.require_vmrun()?,
            [
                "-T",
                "ws",
                "snapshot",
                vmx.to_string_lossy().as_ref(),
                READY_SNAPSHOT_NAME,
            ],
            DISK_STATE_TIMEOUT,
        );
        if let Err(error) = result {
            if self.has_ready(client).unwrap_or(false) {
                return Ok(ClientState::Stopped);
            }
            return Err(error);
        }
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

        let revert = command_output_with_timeout(
            self.require_vmrun()?,
            [
                "-T",
                "ws",
                "revertToSnapshot",
                vmx.to_string_lossy().as_ref(),
                READY_SNAPSHOT_NAME,
            ],
            DISK_STATE_TIMEOUT,
        );
        if let Err(error) = revert {
            // Snapshot revert has no cheap provider fact that proves the exact
            // disk state reached. Do not replay an unknown destructive mutation.
            return Err(io::Error::new(
                error.kind(),
                format!(
                    "QA_READY revert outcome is unknown; inspect the VM before retrying: {error}"
                ),
            ));
        }
        let start = command_output(
            self.require_vmrun()?,
            ["-T", "ws", "start", vmx.to_string_lossy().as_ref(), "gui"],
        );
        if let Err(error) = start {
            if self.running(&vmx).unwrap_or(false) {
                return Ok(ClientState::Running);
            }
            return Err(error);
        }
        wait_for_state(|| self.running(&vmx), true, Duration::from_secs(15))?;
        Ok(ClientState::Running)
    }

    fn open(&self, client: ClientId) -> io::Result<ClientState> {
        let vmx = self.require_client(client)?;
        self.open_vm_ui(&vmx)?;
        self.status(client)
    }

    fn open_vm_ui(&self, vmx: &Path) -> io::Result<()> {
        let gui = self.gui().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "VMware Workstation UI was not found",
            )
        })?;
        Command::new(gui).arg("-n").arg(vmx).spawn()?;
        Ok(())
    }
}
