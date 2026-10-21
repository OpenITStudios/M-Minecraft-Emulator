//! Process-bound known-folder path observations only.
//!
//! This layer never reads the Windows host profile, probes a registry, opens
//! Minecraft, or treats private-looking paths as real isolation. It accepts
//! sanitized observations made inside a trusted/cooperating test context.
//! Querying SHGetKnownFolderPath with another user's token is NOT equivalent to
//! inspecting the filesystem view of an arbitrary sandboxed game process.

use super::{
    review_bound_storage_facts, BoundStorageFact, EvidenceReviewError,
    ProcessIdentity, SessionSlot, StorageEvidenceSource, StorageFinding,
    StorageObservations, StorageProbeMethod, StorageSurface,
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
