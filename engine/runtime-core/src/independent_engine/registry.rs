//! Registry feasibility contract for cooperating, disposable synthetic hives.
//! NO registry calls: writing even a random name into HKCU before proving
//! the AppContainer registry view could modify real account-bearing data.
//! All collector reports here are untrusted and NEVER admit a game launch.

use super::{ProcessIdentity, SessionSlot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryProbeScope {
    NewlyOwnedFixtureHive,
    HostCurrentUser,
    UnknownOrPreexisting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryCanaryRead {
    NotObserved,
    ExpectedMarker,
    AccessDenied,
    UnexpectedMarkerOrValue,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SyntheticRegistryProbe {
    pub slot: SessionSlot,
    pub process: ProcessIdentity,
    pub observed_process: Option<ProcessIdentity>,
    pub scope: RegistryProbeScope,
    pub hive_owner: SessionSlot,
    pub own_value: RegistryCanaryRead,
    pub peer_value: RegistryCanaryRead,
    pub reopened_process: Option<ProcessIdentity>,
    pub reopened_observed: Option<ProcessIdentity>,
    pub persisted_own_value: RegistryCanaryRead,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryProbeError {
    NativeHostSlot,
    HostHiveForbidden,
    UnverifiedHiveOwner,
    CrossSlotHive,
    UnpinnedOrStaleProcess,
    InvalidRestartPair,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RegistryFixtureIssue {
    OwnCanaryUnverified,
    PeerCanaryNotTested,
    PeerCanaryUnexpectedlyReadable,
    RestartNotTested,
    RestartDataUnverified,
}

/// No Approved/Isolated state. An empty issues list is synthetic
/// self-reporting only, not native registry or authentication proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegistryFixtureReview {
    pub issues: Vec<RegistryFixtureIssue>,
}

pub fn review_synthetic_registry_probe(
    report: SyntheticRegistryProbe,
) -> Result<RegistryFixtureReview, RegistryProbeError> {
    if report.slot.is_host_native() {
        return Err(RegistryProbeError::NativeHostSlot);
    }
    match report.scope {
        RegistryProbeScope::HostCurrentUser => {
            return Err(RegistryProbeError::HostHiveForbidden);
        }
        RegistryProbeScope::UnknownOrPreexisting => {
            return Err(RegistryProbeError::UnverifiedHiveOwner);
        }
        RegistryProbeScope::NewlyOwnedFixtureHive => {}
    }
    if report.hive_owner != report.slot {
        return Err(RegistryProbeError::CrossSlotHive);
    }
    if report.process.pid == 0 || report.process.created_file_time == 0
        || report.observed_process != Some(report.process)
    {
        return Err(RegistryProbeError::UnpinnedOrStaleProcess);
    }
    match (report.reopened_process, report.reopened_observed) {
        (None, None) if report.persisted_own_value == RegistryCanaryRead::NotObserved => {}
        (Some(reopened), Some(observed))
            if reopened == observed && reopened != report.process
                && reopened.pid != 0 && reopened.created_file_time != 0 => {}
        _ => return Err(RegistryProbeError::InvalidRestartPair),
    }

    let mut issues = Vec::new();
    if report.own_value != RegistryCanaryRead::ExpectedMarker {
        issues.push(RegistryFixtureIssue::OwnCanaryUnverified);
    }
    match report.peer_value {
        RegistryCanaryRead::AccessDenied => {}
        RegistryCanaryRead::NotObserved => issues.push(RegistryFixtureIssue::PeerCanaryNotTested),
        RegistryCanaryRead::ExpectedMarker
        | RegistryCanaryRead::UnexpectedMarkerOrValue => {
            issues.push(RegistryFixtureIssue::PeerCanaryUnexpectedlyReadable);
        }
    }
    if report.reopened_process.is_none() {
        issues.push(RegistryFixtureIssue::RestartNotTested);
    } else if report.persisted_own_value != RegistryCanaryRead::ExpectedMarker {
        issues.push(RegistryFixtureIssue::RestartDataUnverified);
    }
    Ok(RegistryFixtureReview { issues })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> ProcessIdentity {
        ProcessIdentity::checked(123, 133400000000000000).unwrap()
    }
    fn sample() -> SyntheticRegistryProbe {
        SyntheticRegistryProbe {
            slot: SessionSlot::Mc02,
            process: id(),
            observed_process: Some(id()),
            scope: RegistryProbeScope::NewlyOwnedFixtureHive,
            hive_owner: SessionSlot::Mc02,
            own_value: RegistryCanaryRead::ExpectedMarker,
            peer_value: RegistryCanaryRead::AccessDenied,
            reopened_process: None,
            reopened_observed: None,
            persisted_own_value: RegistryCanaryRead::NotObserved,
        }
    }

    #[test]
    fn host_hkcu_and_unknown_hives_are_rejected() {
        assert_eq!(
            review_synthetic_registry_probe(SyntheticRegistryProbe {
                scope: RegistryProbeScope::HostCurrentUser, ..sample()
            }).unwrap_err(),
            RegistryProbeError::HostHiveForbidden
        );
        assert_eq!(
            review_synthetic_registry_probe(SyntheticRegistryProbe {
                scope: RegistryProbeScope::UnknownOrPreexisting, ..sample()
            }).unwrap_err(),
            RegistryProbeError::UnverifiedHiveOwner
        );
    }

    #[test]
    fn native_and_peer_slots_never_adopt_fixture_registry() {
        assert_eq!(
            review_synthetic_registry_probe(SyntheticRegistryProbe {
                slot: SessionSlot::Mc01, hive_owner: SessionSlot::Mc01, ..sample()
            }).unwrap_err(),
            RegistryProbeError::NativeHostSlot
        );
        for peer in [SessionSlot::Mc01, SessionSlot::Mc03, SessionSlot::Mc04] {
            assert_eq!(
                review_synthetic_registry_probe(SyntheticRegistryProbe {
                    hive_owner: peer, ..sample()
                }).unwrap_err(),
                RegistryProbeError::CrossSlotHive
            );
        }
    }

    #[test]
    fn stale_creation_time_and_pid_reuse_fail() {
        for observed in [
            None,
            ProcessIdentity::checked(123, id().created_file_time + 1),
            ProcessIdentity::checked(124, id().created_file_time),
        ] {
            assert_eq!(
                review_synthetic_registry_probe(SyntheticRegistryProbe {
                    observed_process: observed, ..sample()
                }).unwrap_err(),
                RegistryProbeError::UnpinnedOrStaleProcess
            );
        }
    }

    #[test]
    fn restart_requires_a_different_verified_process() {
        let next = ProcessIdentity::checked(124, id().created_file_time + 1).unwrap();
        for (reopened, observed) in [
            (Some(next), None),
            (Some(id()), Some(id())),
            (Some(next), Some(id())),
            (None, Some(next)),
        ] {
            assert_eq!(
                review_synthetic_registry_probe(SyntheticRegistryProbe {
                    reopened_process: reopened,
                    reopened_observed: observed,
                    persisted_own_value: RegistryCanaryRead::ExpectedMarker,
                    ..sample()
                }).unwrap_err(),
                RegistryProbeError::InvalidRestartPair
            );
        }
        assert_eq!(
            review_synthetic_registry_probe(SyntheticRegistryProbe {
                persisted_own_value: RegistryCanaryRead::ExpectedMarker, ..sample()
            }).unwrap_err(),
            RegistryProbeError::InvalidRestartPair
        );
    }

    #[test]
    fn denied_peer_access_is_distinct_from_missing_and_readable_data() {
        let result = review_synthetic_registry_probe(SyntheticRegistryProbe {
            own_value: RegistryCanaryRead::NotObserved,
            peer_value: RegistryCanaryRead::ExpectedMarker,
            ..sample()
        }).unwrap();
        assert!(result.issues.contains(&RegistryFixtureIssue::OwnCanaryUnverified));
        assert!(result.issues.contains(&RegistryFixtureIssue::PeerCanaryUnexpectedlyReadable));
        assert!(result.issues.contains(&RegistryFixtureIssue::RestartNotTested));
        assert!(review_synthetic_registry_probe(SyntheticRegistryProbe {
            peer_value: RegistryCanaryRead::NotObserved, ..sample()
        }).unwrap().issues.contains(&RegistryFixtureIssue::PeerCanaryNotTested));
    }

    #[test]
    fn restart_marker_must_be_observed_for_valid_fixture_restart() {
        let next = ProcessIdentity::checked(124, id().created_file_time + 2).unwrap();
        assert!(review_synthetic_registry_probe(SyntheticRegistryProbe {
            reopened_process: Some(next),
            reopened_observed: Some(next),
            persisted_own_value: RegistryCanaryRead::NotObserved,
            ..sample()
        }).unwrap().issues.contains(&RegistryFixtureIssue::RestartDataUnverified));
    }

    #[test]
    fn all_positive_synthetic_report_is_still_not_registry_authorization() {
        let next = ProcessIdentity::checked(124, id().created_file_time + 3).unwrap();
        let report = review_synthetic_registry_probe(SyntheticRegistryProbe {
            reopened_process: Some(next),
            reopened_observed: Some(next),
            persisted_own_value: RegistryCanaryRead::ExpectedMarker,
            ..sample()
        }).unwrap();
        assert!(report.issues.is_empty());
        // No production access, trusted HKCU isolation or broker permission.
    }
}
