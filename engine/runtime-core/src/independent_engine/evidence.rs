//! Strict offline/source-level review of process-bound storage evidence.
//!
//! A Windows token, environment variable or path string does not attest to
//! file/registry virtualization, named-object separation or Microsoft sign-in.
//! This pure module has no process/file/network access and never authorizes
//! the desktop provider or promotes Minecraft acceptance.

use super::{
    ProcessIdentity, SessionSlot, StorageFinding, StorageObservations, StorageSurface,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageEvidenceSource {
    /// Fixture results cannot establish installed game isolation.
    SyntheticFixture,
    /// A native Windows measurement tied to a verified process identity.
    WindowsProcess,
    /// Explicit Minecraft runtime evidence, never a token/cache dump.
    MinecraftRuntime,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageProbeMethod {
    Unavailable,
    PathQuery,
    LocalFileCanary,
    RoamingEnvironmentCanary,
    RoamingKnownFolderCanary,
    RegistryCanary,
    NamedObjectCanary,
    PeerAccessCanary,
    HostAccessCanary,
    RestartPersistenceCanary,
    ManualGameSignIn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundStorageFact {
    pub slot: SessionSlot,
    pub process: ProcessIdentity,
    pub surface: StorageSurface,
    pub finding: StorageFinding,
    pub method: StorageProbeMethod,
    pub source: StorageEvidenceSource,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceReviewError {
    WrongSlot,
    WrongProcess,
    DuplicateSurface,
    InvalidObservation,
    InadequateProofSource,
}

fn method_for(surface: StorageSurface) -> StorageProbeMethod {
    match surface {
        StorageSurface::LocalAppData => StorageProbeMethod::LocalFileCanary,
        StorageSurface::RoamingEnvironment => StorageProbeMethod::RoamingEnvironmentCanary,
        StorageSurface::RoamingKnownFolder => StorageProbeMethod::RoamingKnownFolderCanary,
        StorageSurface::PrivateRegistry => StorageProbeMethod::RegistryCanary,
        StorageSurface::NamedObjects => StorageProbeMethod::NamedObjectCanary,
        StorageSurface::PeerDataAccess => StorageProbeMethod::PeerAccessCanary,
        StorageSurface::HostDataAccess => StorageProbeMethod::HostAccessCanary,
        StorageSurface::RestartPersistence => StorageProbeMethod::RestartPersistenceCanary,
        StorageSurface::AuthenticationBroker => StorageProbeMethod::ManualGameSignIn,
    }
}

/// Convert observations into a bounded, diagnostic-only surface summary.
///
/// Reports are statements by their collectors, NOT attestations: the
/// platform adapter must independently establish the trust of each source,
/// prove that the PID/creation time still identifies the same process and
/// only perform authorized, non-destructive measurements. A file canary
/// from a synthetic profile is not evidence for a real Minecraft profile.
pub fn review_bound_storage_facts(
    slot: SessionSlot,
    expected: ProcessIdentity,
    facts: &[BoundStorageFact],
) -> Result<StorageObservations, EvidenceReviewError> {
    if facts.len() > StorageSurface::ALL.len() {
        return Err(EvidenceReviewError::DuplicateSurface);
    }
    let mut observed = [false; 9];
    let mut result = StorageObservations::default();
    for fact in facts {
        if fact.slot != slot {
            return Err(EvidenceReviewError::WrongSlot);
        }
        if fact.process != expected {
            return Err(EvidenceReviewError::WrongProcess);
        }
        let index = StorageSurface::ALL
            .iter()
            .position(|&surface| surface == fact.surface)
            .expect("all StorageSurface enum variants are included in ALL");
        if observed[index] {
            return Err(EvidenceReviewError::DuplicateSurface);
        }
        observed[index] = true;

        let valid = match fact.finding {
            StorageFinding::Unknown => fact.method == StorageProbeMethod::Unavailable,
            StorageFinding::PathOnly | StorageFinding::PathOutsideProfile => matches!(
                fact.surface,
                StorageSurface::LocalAppData
                    | StorageSurface::RoamingEnvironment
                    | StorageSurface::RoamingKnownFolder
            ) && fact.method == StorageProbeMethod::PathQuery,
            StorageFinding::PrivateAccessObserved | StorageFinding::AccessDenied
            | StorageFinding::SharedOrLeaking => {
                fact.method == method_for(fact.surface)
                    && match fact.surface {
                        StorageSurface::PeerDataAccess | StorageSurface::HostDataAccess => {
                            fact.finding != StorageFinding::PrivateAccessObserved
                        }
                        StorageSurface::AuthenticationBroker => {
                            fact.finding != StorageFinding::AccessDenied
                        }
                        _ => fact.finding != StorageFinding::AccessDenied,
                    }
            }
        };
        if !valid {
            return Err(EvidenceReviewError::InvalidObservation);
        }

        // A positive conclusion never comes from a purely synthetic
        // experiment. Broker sign-in observations require actual game evidence,
        // not a Win32 token, storage path or MSAL library metadata.
        if !matches!(fact.finding, StorageFinding::Unknown
            | StorageFinding::PathOnly
            | StorageFinding::PathOutsideProfile) {
            let required = if fact.surface == StorageSurface::AuthenticationBroker {
                StorageEvidenceSource::MinecraftRuntime
            } else {
                StorageEvidenceSource::WindowsProcess
            };
            if fact.source != required {
                return Err(EvidenceReviewError::InadequateProofSource);
            }
        }
        result.observe(fact.surface, fact.finding);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> ProcessIdentity {
        ProcessIdentity::checked(42, 133400000000000000).unwrap()
    }
    fn fact(surface: StorageSurface, finding: StorageFinding,
        method: StorageProbeMethod, source: StorageEvidenceSource) -> BoundStorageFact {
        BoundStorageFact {
            slot: SessionSlot::Mc02, process: identity(), surface, finding, method, source,
        }
    }

    #[test]
    fn empty_and_unknown_facts_preserve_all_unresolved_boundaries() {
        let empty = review_bound_storage_facts(SessionSlot::Mc02, identity(), &[]).unwrap();
        assert_eq!(empty.unresolved(), StorageSurface::ALL.to_vec());
        let unavailable = fact(StorageSurface::AuthenticationBroker,
            StorageFinding::Unknown, StorageProbeMethod::Unavailable,
            StorageEvidenceSource::SyntheticFixture);
        let summary = review_bound_storage_facts(SessionSlot::Mc02,
            identity(), &[unavailable]).unwrap();
        assert_eq!(summary.finding(StorageSurface::AuthenticationBroker),
            StorageFinding::Unknown);
    }

    #[test]
    fn identity_and_slot_mismatches_stop_before_any_facts_are_applied() {
        let good = fact(StorageSurface::RoamingKnownFolder, StorageFinding::PathOnly,
            StorageProbeMethod::PathQuery, StorageEvidenceSource::WindowsProcess);
        assert!(matches!(
            review_bound_storage_facts(SessionSlot::Mc03, identity(), &[good]),
            Err(EvidenceReviewError::WrongSlot)
        ));
        let different = ProcessIdentity::checked(42, identity().created_file_time + 1).unwrap();
        assert!(matches!(
            review_bound_storage_facts(SessionSlot::Mc02, different, &[good]),
            Err(EvidenceReviewError::WrongProcess)
        ));
    }

    #[test]
    fn duplicates_and_conflicting_observations_fail_closed() {
        let path = fact(StorageSurface::RoamingKnownFolder, StorageFinding::PathOnly,
            StorageProbeMethod::PathQuery, StorageEvidenceSource::WindowsProcess);
        assert!(matches!(
            review_bound_storage_facts(SessionSlot::Mc02, identity(), &[path, path]),
            Err(EvidenceReviewError::DuplicateSurface)
        ));
        let cannot_claim_private_from_path = fact(StorageSurface::RoamingKnownFolder,
            StorageFinding::PrivateAccessObserved, StorageProbeMethod::PathQuery,
            StorageEvidenceSource::WindowsProcess);
        assert!(matches!(
            review_bound_storage_facts(SessionSlot::Mc02, identity(),
                &[cannot_claim_private_from_path]),
            Err(EvidenceReviewError::InvalidObservation)
        ));
    }

    #[test]
    fn synthetic_canaries_and_token_metadata_do_not_prove_real_isolation() {
        let fixture = fact(StorageSurface::LocalAppData,
            StorageFinding::PrivateAccessObserved, StorageProbeMethod::LocalFileCanary,
            StorageEvidenceSource::SyntheticFixture);
        assert!(matches!(
            review_bound_storage_facts(SessionSlot::Mc02, identity(), &[fixture]),
            Err(EvidenceReviewError::InadequateProofSource)
        ));
        let native_path = fact(StorageSurface::RoamingKnownFolder,
            StorageFinding::PathOnly, StorageProbeMethod::PathQuery,
            StorageEvidenceSource::WindowsProcess);
        let summary = review_bound_storage_facts(SessionSlot::Mc02,
            identity(), &[native_path]).unwrap();
        assert!(summary.unresolved().contains(&StorageSurface::RoamingKnownFolder));
        assert!(summary.unresolved().contains(&StorageSurface::AuthenticationBroker));
    }

    #[test]
    fn broker_signin_requires_game_observation_and_no_secret_values() {
        let fake_signin = fact(StorageSurface::AuthenticationBroker,
            StorageFinding::PrivateAccessObserved, StorageProbeMethod::ManualGameSignIn,
            StorageEvidenceSource::WindowsProcess);
        assert!(matches!(
            review_bound_storage_facts(SessionSlot::Mc02, identity(), &[fake_signin]),
            Err(EvidenceReviewError::InadequateProofSource)
        ));
        let actual_game_report = BoundStorageFact {
            source: StorageEvidenceSource::MinecraftRuntime,
            ..fake_signin
        };
        let result = review_bound_storage_facts(SessionSlot::Mc02,
            identity(), &[actual_game_report]).unwrap();
        assert!(!result.unresolved().contains(&StorageSurface::AuthenticationBroker));
        assert!(result.unresolved().contains(&StorageSurface::RoamingKnownFolder));
    }


    #[test]
    fn path_outside_private_profile_is_not_a_proven_data_leak() {
        let outside = fact(
            StorageSurface::RoamingKnownFolder,
            StorageFinding::PathOutsideProfile,
            StorageProbeMethod::PathQuery,
            StorageEvidenceSource::SyntheticFixture,
        );
        let report = review_bound_storage_facts(SessionSlot::Mc02, identity(), &[outside]).unwrap();
        assert_eq!(
            report.finding(StorageSurface::RoamingKnownFolder),
            StorageFinding::PathOutsideProfile
        );
        assert!(report.unresolved().contains(&StorageSurface::RoamingKnownFolder));
        assert!(report.unresolved().contains(&StorageSurface::AuthenticationBroker));

        let invalid = fact(
            StorageSurface::AuthenticationBroker,
            StorageFinding::PathOutsideProfile,
            StorageProbeMethod::PathQuery,
            StorageEvidenceSource::WindowsProcess,
        );
        assert_eq!(
            review_bound_storage_facts(SessionSlot::Mc02, identity(), &[invalid]).unwrap_err(),
            EvidenceReviewError::InvalidObservation
        );
    }

    #[test]
    fn actual_access_denial_and_leaks_are_distinct_from_private_storage() {
        let denied = fact(StorageSurface::PeerDataAccess,
            StorageFinding::AccessDenied, StorageProbeMethod::PeerAccessCanary,
            StorageEvidenceSource::WindowsProcess);
        let leaking = fact(StorageSurface::HostDataAccess,
            StorageFinding::SharedOrLeaking, StorageProbeMethod::HostAccessCanary,
            StorageEvidenceSource::WindowsProcess);
        let result = review_bound_storage_facts(SessionSlot::Mc02,
            identity(), &[denied, leaking]).unwrap();
        assert!(!result.unresolved().contains(&StorageSurface::PeerDataAccess));
        assert!(result.unresolved().contains(&StorageSurface::HostDataAccess));
    }
}
