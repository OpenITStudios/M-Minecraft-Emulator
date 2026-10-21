//! Process-bound known-folder path observations only.
//!
//! This layer never reads the Windows host profile, probes a registry, opens
//! Minecraft, or treats private-looking paths as real isolation. It accepts
//! sanitized observations made inside a trusted/cooperating test context.
//! Querying SHGetKnownFolderPath with another user's token is NOT equivalent to
//! inspecting the filesystem view of an arbitrary sandboxed game process.

use super::{
    review_bound_storage_facts, BoundStorageFact, CooperativePathObserverPort,
    EvidenceReviewError, NativePortError, ProcessIdentity, ProcessObserverPort,
    SessionSlot, StorageEvidenceSource, StorageFinding, StorageObservations,
    StorageProbeMethod, StorageSurface,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathLocation {
    /// The process-local query failed or was not conducted.
    Unavailable,
    /// The returned path was reported under the expected private profile.
    /// This is a string relationship, not proof of readable/private data.
    UnderExpectedProfile,
    /// The path was reported outside the expected private profile.
    /// This is not proof that game data actually escaped.
    OutsideExpectedProfile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessPathSnapshot {
    pub slot: SessionSlot,
    pub process: ProcessIdentity,
    /// The capture source remains a non-attested label: collectors are
    /// responsible for proving the actual process context and provenance.
    pub source: StorageEvidenceSource,
    pub local_app_data: PathLocation,
    pub roaming_environment: PathLocation,
    pub roaming_known_folder: PathLocation,
}

fn path_fact(
    snapshot: ProcessPathSnapshot,
    surface: StorageSurface,
    location: PathLocation,
) -> BoundStorageFact {
    let (finding, method) = match location {
        PathLocation::Unavailable => (StorageFinding::Unknown, StorageProbeMethod::Unavailable),
        PathLocation::UnderExpectedProfile => {
            (StorageFinding::PathOnly, StorageProbeMethod::PathQuery)
        }
        PathLocation::OutsideExpectedProfile => {
            (StorageFinding::PathOutsideProfile, StorageProbeMethod::PathQuery)
        }
    };
    BoundStorageFact {
        slot: snapshot.slot,
        process: snapshot.process,
        surface,
        finding,
        method,
        source: snapshot.source,
    }
}

/// Review precisely three path query outcomes. No serialized raw folder paths
/// or user/machine identity; no authority to launch, write or delete profiles.
pub fn review_process_path_snapshot(
    slot: SessionSlot,
    expected: ProcessIdentity,
    snapshot: ProcessPathSnapshot,
) -> Result<StorageObservations, EvidenceReviewError> {
    if snapshot.slot != slot {
        return Err(EvidenceReviewError::WrongSlot);
    }
    if snapshot.process != expected {
        return Err(EvidenceReviewError::WrongProcess);
    }
    // Game sign-in observations cannot double as OS known-folder queries.
    if snapshot.source == StorageEvidenceSource::MinecraftRuntime {
        return Err(EvidenceReviewError::InvalidObservation);
    }
    let facts = [
        path_fact(snapshot, StorageSurface::LocalAppData, snapshot.local_app_data),
        path_fact(snapshot, StorageSurface::RoamingEnvironment, snapshot.roaming_environment),
        path_fact(snapshot, StorageSurface::RoamingKnownFolder, snapshot.roaming_known_folder),
    ];
    review_bound_storage_facts(slot, expected, &facts)
}


/// Diagnostics for a cooperating Windows process, not arbitrary external
/// processes. The implementation must independently bind its returned
/// observations to an OS process handle whose PID/FILETIME still matches.
/// Synthetic A/B fixture names never satisfy this contract on their own.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundPathInspectionError {
    Native(NativePortError),
    Evidence(EvidenceReviewError),
}

/// A two-phase observer: pin the expected process before querying its own
/// path APIs, and independently verify identity/liveness again afterwards.
/// This cannot authenticate a collector or authorize the game's launch.
/// Its only output is the same non-authoritative path-only evidence model.
pub fn inspect_cooperative_paths<P, Q>(
    process: &P,
    probe: &Q,
    slot: SessionSlot,
    expected: ProcessIdentity,
) -> Result<StorageObservations, BoundPathInspectionError>
where
    P: ProcessObserverPort,
    Q: CooperativePathObserverPort,
{
    if expected.pid == 0 || expected.created_file_time == 0 {
        return Err(BoundPathInspectionError::Native(NativePortError::InvalidIdentity));
    }
    let before = process
        .observe_exact(expected)
        .map_err(BoundPathInspectionError::Native)?;
    if before.identity != expected {
        return Err(BoundPathInspectionError::Native(NativePortError::StaleProcess));
    }
    let snapshot = probe
        .observe_process_local_paths(slot, expected)
        .map_err(BoundPathInspectionError::Native)?;
    if snapshot.slot != slot {
        return Err(BoundPathInspectionError::Evidence(EvidenceReviewError::WrongSlot));
    }
    if snapshot.process != expected {
        return Err(BoundPathInspectionError::Evidence(EvidenceReviewError::WrongProcess));
    }
    // A failed final process observation invalidates this entire capture.
    let after = process
        .observe_exact(expected)
        .map_err(BoundPathInspectionError::Native)?;
    if after != before {
        // Identity alone misses a containment/token-category change during
        // an untrusted cooperative known-folder probe.
        return Err(BoundPathInspectionError::Native(NativePortError::StaleProcess));
    }
    review_process_path_snapshot(slot, expected, snapshot)
        .map_err(BoundPathInspectionError::Evidence)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected() -> ProcessIdentity {
        ProcessIdentity::checked(92, 133400000000000000).unwrap()
    }
    fn sample() -> ProcessPathSnapshot {
        ProcessPathSnapshot {
            slot: SessionSlot::Mc02,
            process: expected(),
            source: StorageEvidenceSource::SyntheticFixture,
            local_app_data: PathLocation::UnderExpectedProfile,
            roaming_environment: PathLocation::OutsideExpectedProfile,
            roaming_known_folder: PathLocation::OutsideExpectedProfile,
        }
    }

    #[test]
    fn current_synthetic_fixture_paths_remain_unresolved() {
        let result = review_process_path_snapshot(SessionSlot::Mc02, expected(), sample()).unwrap();
        assert_eq!(result.finding(StorageSurface::LocalAppData), StorageFinding::PathOnly);
        assert_eq!(
            result.finding(StorageSurface::RoamingKnownFolder),
            StorageFinding::PathOutsideProfile
        );
        assert!(result.unresolved().contains(&StorageSurface::LocalAppData));
        assert!(result.unresolved().contains(&StorageSurface::RoamingKnownFolder));
        assert!(result.unresolved().contains(&StorageSurface::AuthenticationBroker));
        assert!(result.unresolved().contains(&StorageSurface::PrivateRegistry));
    }

    #[test]
    fn all_private_looking_paths_still_do_not_certify_file_or_account_isolation() {
        let mut value = sample();
        value.source = StorageEvidenceSource::WindowsProcess;
        value.roaming_environment = PathLocation::UnderExpectedProfile;
        value.roaming_known_folder = PathLocation::UnderExpectedProfile;
        let result = review_process_path_snapshot(SessionSlot::Mc02, expected(), value).unwrap();
        for surface in [StorageSurface::LocalAppData, StorageSurface::RoamingEnvironment,
            StorageSurface::RoamingKnownFolder] {
            assert_eq!(result.finding(surface), StorageFinding::PathOnly);
            assert!(result.unresolved().contains(&surface));
        }
        assert!(result.unresolved().contains(&StorageSurface::AuthenticationBroker));
    }

    #[test]
    fn no_known_folder_query_is_unknown_not_implicit_private_access() {
        let mut value = sample();
        value.roaming_known_folder = PathLocation::Unavailable;
        let result = review_process_path_snapshot(SessionSlot::Mc02, expected(), value).unwrap();
        assert_eq!(result.finding(StorageSurface::RoamingKnownFolder), StorageFinding::Unknown);
        assert!(result.unresolved().contains(&StorageSurface::RoamingKnownFolder));
    }

    #[test]
    fn different_slot_or_reused_pid_creation_time_is_rejected() {
        let value = sample();
        assert_eq!(
            review_process_path_snapshot(SessionSlot::Mc03, expected(), value).unwrap_err(),
            EvidenceReviewError::WrongSlot
        );
        let stale = ProcessIdentity::checked(expected().pid, expected().created_file_time + 1).unwrap();
        assert_eq!(
            review_process_path_snapshot(SessionSlot::Mc02, stale, value).unwrap_err(),
            EvidenceReviewError::WrongProcess
        );
    }


    #[derive(Default)]
    struct FakeProcess {
        checks: std::cell::Cell<u32>,
        wrong_on_check: Option<u32>,
        failed_on_check: Option<u32>,
        changed_containment_on: Option<u32>,
    }

    impl ProcessObserverPort for FakeProcess {
        fn observe_exact(
            &self,
            expected: ProcessIdentity,
        ) -> Result<super::super::ProcessObservation, NativePortError> {
            let call = self.checks.get() + 1;
            self.checks.set(call);
            if self.failed_on_check == Some(call) {
                return Err(NativePortError::StaleProcess);
            }
            let identity = if self.wrong_on_check == Some(call) {
                ProcessIdentity::checked(expected.pid, expected.created_file_time + 1).unwrap()
            } else {
                expected
            };
            Ok(super::super::ProcessObservation {
                identity,
                containment: if self.changed_containment_on == Some(call) {
                    super::super::ProcessContainment::AppContainerTokenObserved
                } else {
                    super::super::ProcessContainment::Unknown
                },
                official_image_signature_observed: false,
                visible_window_observed: false,
            })
        }
    }

    struct FakeProbe {
        calls: std::cell::Cell<u32>,
        result: ProcessPathSnapshot,
    }

    impl CooperativePathObserverPort for FakeProbe {
        fn observe_process_local_paths(
            &self,
            _slot: SessionSlot,
            _expected: ProcessIdentity,
        ) -> Result<ProcessPathSnapshot, NativePortError> {
            self.calls.set(self.calls.get() + 1);
            Ok(self.result)
        }
    }

    #[test]
    fn cooperating_path_snapshot_is_bound_by_two_process_observations() {
        let process = FakeProcess::default();
        let probe = FakeProbe {
            calls: std::cell::Cell::new(0),
            result: sample(),
        };
        let report = inspect_cooperative_paths(
            &process, &probe, SessionSlot::Mc02, expected(),
        ).unwrap();
        assert_eq!(process.checks.get(), 2);
        assert_eq!(probe.calls.get(), 1);
        assert_eq!(
            report.finding(StorageSurface::RoamingKnownFolder),
            StorageFinding::PathOutsideProfile
        );
        assert!(report.unresolved().contains(&StorageSurface::AuthenticationBroker));
    }

    #[test]
    fn invalid_process_identity_does_not_call_cooperating_path_observer() {
        let process = FakeProcess::default();
        let probe = FakeProbe {
            calls: std::cell::Cell::new(0),
            result: sample(),
        };
        let valid = expected();
        for invalid in [
            ProcessIdentity { pid: 0, created_file_time: valid.created_file_time },
            ProcessIdentity { pid: valid.pid, created_file_time: 0 },
        ] {
            assert_eq!(
                inspect_cooperative_paths(&process, &probe, SessionSlot::Mc02, invalid).unwrap_err(),
                BoundPathInspectionError::Native(NativePortError::InvalidIdentity)
            );
        }
        assert_eq!(process.checks.get(), 0);
        assert_eq!(probe.calls.get(), 0);
    }

    #[test]
    fn stale_process_before_probe_prevents_any_path_capture() {
        let process = FakeProcess {
            wrong_on_check: Some(1),
            ..Default::default()
        };
        let probe = FakeProbe {
            calls: std::cell::Cell::new(0),
            result: sample(),
        };
        assert_eq!(
            inspect_cooperative_paths(&process, &probe, SessionSlot::Mc02, expected()).unwrap_err(),
            BoundPathInspectionError::Native(NativePortError::StaleProcess)
        );
        assert_eq!(probe.calls.get(), 0);
    }

    #[test]
    fn process_exit_after_probe_invalidates_all_path_evidence() {
        let process = FakeProcess {
            failed_on_check: Some(2),
            ..Default::default()
        };
        let probe = FakeProbe {
            calls: std::cell::Cell::new(0),
            result: sample(),
        };
        assert_eq!(
            inspect_cooperative_paths(&process, &probe, SessionSlot::Mc02, expected()).unwrap_err(),
            BoundPathInspectionError::Native(NativePortError::StaleProcess)
        );
        assert_eq!(probe.calls.get(), 1);
        assert_eq!(process.checks.get(), 2);
    }

    #[test]
    fn containment_change_during_path_probe_invalidates_all_observations() {
        let process = FakeProcess {
            changed_containment_on: Some(2),
            ..Default::default()
        };
        let probe = FakeProbe {
            calls: std::cell::Cell::new(0),
            result: sample(),
        };
        assert_eq!(
            inspect_cooperative_paths(&process, &probe, SessionSlot::Mc02, expected()).unwrap_err(),
            BoundPathInspectionError::Native(NativePortError::StaleProcess)
        );
        assert_eq!(process.checks.get(), 2);
        assert_eq!(probe.calls.get(), 1);
    }

    #[test]
    fn mismatched_fixture_binding_is_rejected_without_secondary_recheck() {
        let process = FakeProcess::default();
        let probe = FakeProbe {
            calls: std::cell::Cell::new(0),
            result: ProcessPathSnapshot {
                process: ProcessIdentity::checked(92, expected().created_file_time + 1).unwrap(),
                ..sample()
            },
        };
        assert_eq!(
            inspect_cooperative_paths(&process, &probe, SessionSlot::Mc02, expected()).unwrap_err(),
            BoundPathInspectionError::Evidence(EvidenceReviewError::WrongProcess)
        );
        assert_eq!(process.checks.get(), 1);
        assert_eq!(probe.calls.get(), 1);
    }

    #[test]
    fn source_mislabelled_as_minecraft_runtime_is_not_path_proof() {
        let process = FakeProcess::default();
        let probe = FakeProbe {
            calls: std::cell::Cell::new(0),
            result: ProcessPathSnapshot {
                source: StorageEvidenceSource::MinecraftRuntime,
                ..sample()
            },
        };
        assert_eq!(
            inspect_cooperative_paths(&process, &probe, SessionSlot::Mc02, expected()).unwrap_err(),
            BoundPathInspectionError::Evidence(EvidenceReviewError::InvalidObservation)
        );
        assert_eq!(process.checks.get(), 2);
    }

    #[test]
    fn minecraft_observation_cannot_claim_process_local_known_folder_routing() {
        let value = ProcessPathSnapshot {
            source: StorageEvidenceSource::MinecraftRuntime,
            ..sample()
        };
        assert_eq!(
            review_process_path_snapshot(SessionSlot::Mc02, expected(), value).unwrap_err(),
            EvidenceReviewError::InvalidObservation
        );
    }
}
