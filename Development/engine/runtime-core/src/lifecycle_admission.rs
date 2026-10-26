//! Pure lifecycle admission shared by action projection and execution.
//!
//! Runtime collects observations under its operation lock. This module evaluates
//! them without reading the filesystem, contacting guests, or mutating providers.

use crate::client::{ActionAvailability, ClientId, ClientState, LifecycleBlocker};
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LifecycleAction {
    Start,
    StartSetup,
    Suspend,
    Stop,
    Open,
    Restart,
    SetReady,
    Reset,
    Reprovision,
}

pub(super) fn validate_power_state(
    client: ClientId,
    action: LifecycleAction,
    state: ClientState,
    ready_snapshot: bool,
) -> io::Result<()> {
    if client.is_native() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Native lifecycle is host-managed",
        ));
    }

    let valid = match action {
        LifecycleAction::Start | LifecycleAction::StartSetup => matches!(
            state,
            ClientState::Stopped | ClientState::Suspended | ClientState::Running
        ),
        LifecycleAction::Suspend => {
            matches!(state, ClientState::Running | ClientState::Suspended)
        }
        LifecycleAction::Stop => matches!(state, ClientState::Stopped | ClientState::Running),
        LifecycleAction::Open => matches!(
            state,
            ClientState::Stopped | ClientState::Suspended | ClientState::Running
        ),
        LifecycleAction::Restart => state == ClientState::Running,
        LifecycleAction::SetReady => state == ClientState::Stopped && !ready_snapshot,
        LifecycleAction::Reset => {
            // VMware cannot safely stop a suspended VM for snapshot restore.
            matches!(state, ClientState::Stopped | ClientState::Running) && ready_snapshot
        }
        LifecycleAction::Reprovision => state == ClientState::Stopped,
    };

    if valid {
        return Ok(());
    }

    let action_name = match action {
        LifecycleAction::Start => "start",
        LifecycleAction::StartSetup => "start-setup",
        LifecycleAction::Suspend => "suspend",
        LifecycleAction::Stop => "stop",
        LifecycleAction::Open => "open",
        LifecycleAction::Restart => "restart",
        LifecycleAction::SetReady => "set-ready",
        LifecycleAction::Reset => "reset",
        LifecycleAction::Reprovision => "reprovision",
    };

    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!(
            "{} cannot {} from state {:?} with QA_READY={ready_snapshot}",
            client.as_str(),
            action_name,
            state
        ),
    ))
}

fn power_state_availability(
    client: ClientId,
    action: LifecycleAction,
    state: ClientState,
    ready_snapshot: bool,
) -> ActionAvailability {
    if validate_power_state(client, action, state, ready_snapshot).is_ok() {
        return ActionAvailability {
            allowed: true,
            blocker: None,
            reason: None,
        };
    }

    let blocker = if client.is_native() {
        LifecycleBlocker::NativeManaged
    } else if state == ClientState::NotProvisioned {
        LifecycleBlocker::NotProvisioned
    } else {
        match action {
            LifecycleAction::SetReady if ready_snapshot => LifecycleBlocker::ReadySnapshotExists,
            LifecycleAction::Reset if !ready_snapshot => LifecycleBlocker::ReadySnapshotMissing,
            _ => LifecycleBlocker::InvalidState,
        }
    };

    ActionAvailability {
        allowed: false,
        blocker: Some(blocker),
        reason: None,
    }
}

#[derive(Debug, Clone)]
pub(super) struct LifecycleFacts {
    pub(super) state: ClientState,
    pub(super) ready_snapshot: Option<bool>,
    pub(super) schema_ready: bool,
    pub(super) base_compatible: bool,
    pub(super) client_compatible: bool,
    pub(super) saved_vm_identity_matches: bool,
    pub(super) vm_identity_duplicate: bool,
    pub(super) identity_provenance_bound: bool,
    pub(super) fresh_client_profile: bool,
    pub(super) vm_identity_unique: bool,
    pub(super) base_finalized_and_stopped: bool,
    pub(super) can_start: bool,
}

// One policy projection for both UI availability and mutation admission.
// Facts are observations, not a second persisted lifecycle state.
pub(super) fn evaluate_lifecycle_admission(
    client: ClientId,
    action: LifecycleAction,
    facts: &LifecycleFacts,
) -> ActionAvailability {
    if !client.is_native()
        && matches!(
            action,
            LifecycleAction::SetReady | LifecycleAction::Reset | LifecycleAction::StartSetup
        )
        && facts.ready_snapshot.is_none()
    {
        return ActionAvailability {
            allowed: false,
            blocker: Some(LifecycleBlocker::InvalidState),
            reason: Some("Recovery point availability could not be verified."),
        };
    }
    let state = power_state_availability(
        client,
        action,
        facts.state,
        facts.ready_snapshot.unwrap_or(false),
    );
    if !state.allowed {
        return state;
    }
    let blocked = |reason| ActionAvailability {
        allowed: false,
        blocker: Some(LifecycleBlocker::InvalidState),
        reason: Some(reason),
    };
    if !facts.schema_ready {
        return blocked("Runtime data is incompatible with this application.");
    }
    // Stop and Suspend remain possible when Minecraft compatibility is broken.
    if matches!(action, LifecycleAction::Stop | LifecycleAction::Suspend) {
        return state;
    }
    if !facts.base_compatible {
        return blocked("Base compatibility with Native and Guest Agent cannot be verified.");
    }
    if action == LifecycleAction::Reprovision {
        // The current VMware provider replaces the account-bearing VM before
        // it can guarantee clone success. Block the action in both UI and API
        // admission until provider/storage implements recoverable cutover.
        return blocked("Recreate client is temporarily unavailable until VM and account data can be preserved on failure.");
    }
    if !facts.client_compatible {
        return blocked("This client no longer matches Native. Recreate the affected client.");
    }
    if action == LifecycleAction::Open {
        return state;
    }
    if !facts.saved_vm_identity_matches {
        return blocked("VM identity changed after verification. Check client identities.");
    }
    if facts.vm_identity_duplicate {
        return blocked("A VM UUID or MAC address is shared with another client.");
    }
    if action == LifecycleAction::StartSetup
        && (!facts.fresh_client_profile
            || facts.ready_snapshot != Some(false)
            || !facts.vm_identity_unique
            || !facts.base_finalized_and_stopped)
    {
        return blocked("First-time setup requires a fresh client, unique VM identity, no recovery point, and a finalized stopped Base.");
    }
    if action == LifecycleAction::SetReady && !facts.identity_provenance_bound {
        return blocked("Verify client identities before saving a recovery point.");
    }
    if matches!(
        action,
        LifecycleAction::Start | LifecycleAction::Reset | LifecycleAction::StartSetup
    ) && facts.state != ClientState::Running
        && !facts.can_start
    {
        return blocked("Host memory is insufficient to start or resume this client.");
    }
    state
}

pub(super) fn require_lifecycle_admission(
    client: ClientId,
    action: LifecycleAction,
    facts: &LifecycleFacts,
) -> io::Result<()> {
    let admission = evaluate_lifecycle_admission(client, action, facts);
    if admission.allowed {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            admission.reason.map(ToOwned::to_owned).unwrap_or_else(|| {
                format!(
                    "{} action blocked: {:?}",
                    client.as_str(),
                    admission.blocker
                )
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        power_state_availability, validate_power_state, ClientId, ClientState, LifecycleAction,
        LifecycleBlocker,
    };

    #[test]
    fn power_state_availability_reuses_engine_policy() {
        let start = power_state_availability(
            ClientId::Virtual01,
            LifecycleAction::Start,
            ClientState::Stopped,
            false,
        );
        assert!(start.allowed);
        assert_eq!(start.blocker, None);

        let reset = power_state_availability(
            ClientId::Virtual01,
            LifecycleAction::Reset,
            ClientState::Stopped,
            false,
        );
        assert!(!reset.allowed);
        assert_eq!(reset.blocker, Some(LifecycleBlocker::ReadySnapshotMissing));

        let set_ready = power_state_availability(
            ClientId::Virtual01,
            LifecycleAction::SetReady,
            ClientState::Stopped,
            true,
        );
        assert!(!set_ready.allowed);
        assert_eq!(
            set_ready.blocker,
            Some(LifecycleBlocker::ReadySnapshotExists)
        );
    }

    #[test]
    fn lifecycle_matrix_rejects_illegal_transitions_without_provider_mutation() {
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Start,
            ClientState::Stopped,
            false,
        )
        .is_ok());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Start,
            ClientState::NotProvisioned,
            false,
        )
        .is_err());

        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Suspend,
            ClientState::Running,
            false,
        )
        .is_ok());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Suspend,
            ClientState::Stopped,
            false,
        )
        .is_err());

        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Stop,
            ClientState::Suspended,
            false,
        )
        .is_err());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Stop,
            ClientState::NotProvisioned,
            false,
        )
        .is_err());

        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Open,
            ClientState::Stopped,
            false,
        )
        .is_ok());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Open,
            ClientState::Error,
            false,
        )
        .is_err());

        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Restart,
            ClientState::Running,
            false,
        )
        .is_ok());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Restart,
            ClientState::Stopped,
            false,
        )
        .is_err());

        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::SetReady,
            ClientState::Stopped,
            false,
        )
        .is_ok());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::SetReady,
            ClientState::Stopped,
            true,
        )
        .is_err());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::SetReady,
            ClientState::Suspended,
            false,
        )
        .is_err());

        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Reset,
            ClientState::Running,
            true,
        )
        .is_ok());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Reset,
            ClientState::Suspended,
            true,
        )
        .is_err());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Reset,
            ClientState::Stopped,
            false,
        )
        .is_err());

        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Reprovision,
            ClientState::Stopped,
            true,
        )
        .is_ok());
        assert!(validate_power_state(
            ClientId::Virtual01,
            LifecycleAction::Reprovision,
            ClientState::Suspended,
            true,
        )
        .is_err());
    }

    fn admitted_facts(state: ClientState) -> super::LifecycleFacts {
        super::LifecycleFacts {
            state,
            ready_snapshot: Some(false),
            schema_ready: true,
            base_compatible: true,
            client_compatible: true,
            saved_vm_identity_matches: true,
            vm_identity_duplicate: false,
            identity_provenance_bound: true,
            fresh_client_profile: false,
            vm_identity_unique: true,
            base_finalized_and_stopped: true,
            can_start: true,
        }
    }

    #[test]
    fn action_projection_and_execution_share_admission_decisions() {
        for state in [
            ClientState::NotProvisioned,
            ClientState::Stopped,
            ClientState::Suspended,
            ClientState::Running,
            ClientState::Error,
        ] {
            for action in [
                LifecycleAction::Start,
                LifecycleAction::StartSetup,
                LifecycleAction::Suspend,
                LifecycleAction::Stop,
                LifecycleAction::Open,
                LifecycleAction::Restart,
                LifecycleAction::SetReady,
                LifecycleAction::Reset,
                LifecycleAction::Reprovision,
            ] {
                let mut facts = admitted_facts(state);
                for ready in [Some(false), Some(true), None] {
                    facts.ready_snapshot = ready;
                    let projected =
                        super::evaluate_lifecycle_admission(ClientId::Virtual01, action, &facts);
                    assert_eq!(
                        projected.allowed,
                        super::require_lifecycle_admission(ClientId::Virtual01, action, &facts)
                            .is_ok()
                    );
                }
            }
        }
    }

    #[test]
    fn start_admission_blocks_stale_versions_identity_and_memory() {
        let base = admitted_facts(ClientState::Stopped);
        let cases = [
            super::LifecycleFacts {
                base_compatible: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                client_compatible: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                saved_vm_identity_matches: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                vm_identity_duplicate: true,
                ..base.clone()
            },
            super::LifecycleFacts {
                can_start: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                schema_ready: false,
                ..base
            },
        ];
        for facts in cases {
            let result = super::evaluate_lifecycle_admission(
                ClientId::Virtual01,
                LifecycleAction::Start,
                &facts,
            );
            assert!(!result.allowed);
            assert!(result.reason.is_some());
        }
    }

    #[test]
    fn compatibility_failures_do_not_prevent_stop_or_suspend() {
        let facts = super::LifecycleFacts {
            base_compatible: false,
            client_compatible: false,
            saved_vm_identity_matches: false,
            vm_identity_duplicate: true,
            can_start: false,
            ..admitted_facts(ClientState::Running)
        };
        for action in [LifecycleAction::Stop, LifecycleAction::Suspend] {
            assert!(
                super::evaluate_lifecycle_admission(ClientId::Virtual01, action, &facts).allowed
            );
        }
    }

    #[test]
    fn recreating_a_stale_client_requires_a_healthy_stopped_base() {
        let mut facts = super::LifecycleFacts {
            client_compatible: false,
            ..admitted_facts(ClientState::Stopped)
        };
        assert!(
            !super::evaluate_lifecycle_admission(
                ClientId::Virtual01,
                LifecycleAction::Reprovision,
                &facts,
            )
            .allowed
        );
        facts.base_finalized_and_stopped = false;
        assert!(
            !super::evaluate_lifecycle_admission(
                ClientId::Virtual01,
                LifecycleAction::Reprovision,
                &facts,
            )
            .allowed
        );
    }

    #[test]
    fn recovery_point_admission_requires_identity_and_known_snapshot_state() {
        let mut facts = admitted_facts(ClientState::Stopped);
        facts.identity_provenance_bound = false;
        assert!(
            !super::evaluate_lifecycle_admission(
                ClientId::Virtual01,
                LifecycleAction::SetReady,
                &facts,
            )
            .allowed
        );
        facts.identity_provenance_bound = true;
        facts.ready_snapshot = None;
        assert!(
            !super::evaluate_lifecycle_admission(
                ClientId::Virtual01,
                LifecycleAction::SetReady,
                &facts,
            )
            .allowed
        );
        facts.ready_snapshot = Some(true);
        facts.can_start = false;
        assert!(
            !super::evaluate_lifecycle_admission(
                ClientId::Virtual01,
                LifecycleAction::Reset,
                &facts,
            )
            .allowed
        );
    }

    #[test]
    fn running_client_admission_does_not_require_extra_boot_memory() {
        let facts = super::LifecycleFacts {
            can_start: false,
            ..admitted_facts(ClientState::Running)
        };
        assert!(
            super::evaluate_lifecycle_admission(
                ClientId::Virtual01,
                LifecycleAction::Start,
                &facts,
            )
            .allowed
        );
        assert!(super::evaluate_lifecycle_admission(
            ClientId::Virtual01, LifecycleAction::Open, &facts,
        ).allowed);
    }

    #[test]
    fn native_client_never_receives_vm_lifecycle_permission() {
        let facts = admitted_facts(ClientState::Running);
        for action in [
            LifecycleAction::Start,
            LifecycleAction::StartSetup,
            LifecycleAction::Suspend,
            LifecycleAction::Stop,
            LifecycleAction::Open,
            LifecycleAction::Restart,
            LifecycleAction::SetReady,
            LifecycleAction::Reset,
            LifecycleAction::Reprovision,
        ] {
            let result = super::evaluate_lifecycle_admission(ClientId::Native, action, &facts);
            assert!(!result.allowed);
            assert_eq!(result.blocker, Some(LifecycleBlocker::NativeManaged));
            assert!(super::require_lifecycle_admission(ClientId::Native, action, &facts).is_err());
        }
    }

    #[test]
    fn stop_and_suspend_still_require_compatible_runtime_data() {
        let facts = super::LifecycleFacts {
            schema_ready: false,
            ..admitted_facts(ClientState::Running)
        };
        for action in [LifecycleAction::Stop, LifecycleAction::Suspend] {
            let result = super::evaluate_lifecycle_admission(ClientId::Virtual01, action, &facts);
            assert!(!result.allowed);
            assert_eq!(
                result.reason,
                Some("Runtime data is incompatible with this application.")
            );
        }
    }

    #[test]
    fn first_boot_requires_fresh_profile_unique_vm_and_no_recovery_point() {
        let base = super::LifecycleFacts {
            identity_provenance_bound: false,
            fresh_client_profile: true,
            ..admitted_facts(ClientState::Stopped)
        };
        assert!(
            super::evaluate_lifecycle_admission(
                ClientId::Virtual01,
                LifecycleAction::StartSetup,
                &base,
            )
            .allowed
        );
        for facts in [
            super::LifecycleFacts {
                fresh_client_profile: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                vm_identity_unique: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                ready_snapshot: Some(true),
                ..base.clone()
            },
            super::LifecycleFacts {
                ready_snapshot: None,
                ..base.clone()
            },
            super::LifecycleFacts {
                can_start: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                client_compatible: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                base_compatible: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                schema_ready: false,
                ..base.clone()
            },
            super::LifecycleFacts {
                base_finalized_and_stopped: false,
                ..base.clone()
            },
        ] {
            assert!(
                !super::evaluate_lifecycle_admission(
                    ClientId::Virtual01,
                    LifecycleAction::StartSetup,
                    &facts,
                )
                .allowed
            );
        }
    }
}
