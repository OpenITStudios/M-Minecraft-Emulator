//! Explicit storage surfaces, not a virtualized path mapping or credentials API.
//! A path-looking-private observation cannot certify read/write isolation.
//! All findings here are non-authoritative diagnostics, not provider readiness.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageSurface {
    LocalAppData,
    RoamingEnvironment,
    RoamingKnownFolder,
    PrivateRegistry,
    NamedObjects,
    PeerDataAccess,
    HostDataAccess,
    RestartPersistence,
    AuthenticationBroker,
}

impl StorageSurface {
    pub const ALL: [Self; 9] = [
        Self::LocalAppData,
        Self::RoamingEnvironment,
        Self::RoamingKnownFolder,
        Self::PrivateRegistry,
        Self::NamedObjects,
        Self::PeerDataAccess,
        Self::HostDataAccess,
        Self::RestartPersistence,
        Self::AuthenticationBroker,
    ];

    const fn index(self) -> usize {
        match self {
            Self::LocalAppData => 0,
            Self::RoamingEnvironment => 1,
            Self::RoamingKnownFolder => 2,
            Self::PrivateRegistry => 3,
            Self::NamedObjects => 4,
            Self::PeerDataAccess => 5,
            Self::HostDataAccess => 6,
            Self::RestartPersistence => 7,
            Self::AuthenticationBroker => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StorageFinding {
    #[default]
    Unknown,
    PathOnly,
    /// A queried path points outside the expected private profile. This is
    /// a compatibility warning, NOT evidence that the process read host data.
    PathOutsideProfile,
    PrivateAccessObserved,
    AccessDenied,
    SharedOrLeaking,
}

#[derive(Clone, Debug, Default)]
pub struct StorageObservations {
    entries: [StorageFinding; 9],
}

impl StorageObservations {
    // Only runtime-core can directly build synthetic test observations.
    // External Windows adapters must use reviewed, identity-bound evidence,
    // not freely assign positive or negative isolation findings.
    pub(crate) fn observe(&mut self, surface: StorageSurface, finding: StorageFinding) {
        self.entries[surface.index()] = finding;
    }

    pub fn finding(&self, surface: StorageSurface) -> StorageFinding {
        self.entries[surface.index()]
    }

    /// Classify missing/unsafe observations, never promote them to a launch
    /// permission. A trusted collector and Minecraft acceptance are separate.
    pub fn unresolved(&self) -> Vec<StorageSurface> {
        StorageSurface::ALL
            .iter()
            .copied()
            .filter(|surface| {
                let expected = match surface {
                    StorageSurface::PeerDataAccess | StorageSurface::HostDataAccess => {
                        StorageFinding::AccessDenied
                    }
                    _ => StorageFinding::PrivateAccessObserved,
                };
                self.finding(*surface) != expected
            })
            .collect()
    }
}


// Source-level access preflight for an independent (non-MC01) session.
//
// This is NOT an operating-system path resolver, a filesystem virtualizer,
// an ACL, a Windows registry view or an authorization result. Only the
// future OS adapter can confirm a profile owner and enforce actual access.
// Keeping the outcome non-authorizing prevents a developer from treating
// synthetic canaries or chosen folder names as an active sandbox.
use super::{ProcessIdentity, SessionSlot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRouteDomain {
    LocalFiles,
    RoamingFiles,
    CurrentUserRegistry,
    NamedObjects,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRouteAction {
    Read,
    Create,
    Update,
    Delete,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRouteTarget {
    /// Logical owner identity only; no path, SID, registry key or ACL claim.
    PrivateSession(SessionSlot),
    /// One official game install may be shared READ ONLY after signature checks.
    OfficialInstallation,
    /// Credentials, worlds, host roaming/registry and caches are not shared.
    HostAccountData,
    /// No canonical profile owner or verified official install.
    Unclassified,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageRouteRequest {
    pub slot: SessionSlot,
    pub expected_process: ProcessIdentity,
    pub observed_process: Option<ProcessIdentity>,
    pub domain: StorageRouteDomain,
    pub target: StorageRouteTarget,
    pub action: StorageRouteAction,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRouteBlocker {
    NativeSlotOwnedElsewhere,
    UnpinnedOrStaleProcess,
    AnotherSession,
    HostAccountBoundary,
    UnknownOwner,
    OfficialImageReadOnly,
    OfficialImageNotFilesystem,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageRoutePreflight {
    Denied(StorageRouteBlocker),
    /// An adapter must still prove each boundary on a pinned Windows handle.
    /// This is a plan for an audit, NEVER permission to perform an OS call.
    NeedsNativeVerification {
        domain: StorageRouteDomain,
        required: &'static [StorageSurface],
    },
}

const LOCAL_REQUIRED: &[StorageSurface] = &[
    StorageSurface::LocalAppData,
    StorageSurface::PeerDataAccess,
    StorageSurface::HostDataAccess,
    StorageSurface::RestartPersistence,
];
const ROAMING_REQUIRED: &[StorageSurface] = &[
    StorageSurface::RoamingEnvironment,
    StorageSurface::RoamingKnownFolder,
    StorageSurface::PeerDataAccess,
    StorageSurface::HostDataAccess,
    StorageSurface::RestartPersistence,
];
const REGISTRY_REQUIRED: &[StorageSurface] = &[
    StorageSurface::PrivateRegistry,
    StorageSurface::PeerDataAccess,
    StorageSurface::HostDataAccess,
    StorageSurface::RestartPersistence,
];
const OBJECTS_REQUIRED: &[StorageSurface] = &[
    StorageSurface::NamedObjects,
    StorageSurface::PeerDataAccess,
    StorageSurface::HostDataAccess,
];
const INSTALL_REQUIRED: &[StorageSurface] = &[
    StorageSurface::HostDataAccess,
];

impl StorageRouteDomain {
    pub const fn required_boundaries(self) -> &'static [StorageSurface] {
        match self {
            Self::LocalFiles => LOCAL_REQUIRED,
            Self::RoamingFiles => ROAMING_REQUIRED,
            Self::CurrentUserRegistry => REGISTRY_REQUIRED,
            Self::NamedObjects => OBJECTS_REQUIRED,
        }
    }
}

/// Reject cross-instance, host-account or mutating official-install paths
/// BEFORE a native adapter is even considered. A caller-supplied target is
/// not trustworthy provenance; positive results still require Windows
/// ownership, signed-image verification and actual storage/broker acceptance.
pub fn preflight_storage_route(request: StorageRouteRequest) -> StorageRoutePreflight {
    use StorageRouteBlocker as Block;
    use StorageRoutePreflight as Result;

    if request.slot.is_host_native() {
        return Result::Denied(Block::NativeSlotOwnedElsewhere);
    }
    if request.expected_process.pid == 0
        || request.expected_process.created_file_time == 0
        || request.observed_process != Some(request.expected_process)
    {
        return Result::Denied(Block::UnpinnedOrStaleProcess);
    }

    let required = match request.target {
        StorageRouteTarget::PrivateSession(owner) if owner != request.slot => {
            return Result::Denied(Block::AnotherSession);
        }
        StorageRouteTarget::PrivateSession(_) => request.domain.required_boundaries(),
        StorageRouteTarget::HostAccountData => {
            return Result::Denied(Block::HostAccountBoundary);
        }
        StorageRouteTarget::Unclassified => {
            return Result::Denied(Block::UnknownOwner);
        }
        StorageRouteTarget::OfficialInstallation => {
            if request.domain != StorageRouteDomain::LocalFiles {
                return Result::Denied(Block::OfficialImageNotFilesystem);
            }
            if request.action != StorageRouteAction::Read {
                return Result::Denied(Block::OfficialImageReadOnly);
            }
            INSTALL_REQUIRED
        }
    };
    Result::NeedsNativeVerification {
        domain: request.domain,
        required,
    }
}

#[cfg(test)]
mod route_tests {
    use super::*;

    fn process() -> ProcessIdentity {
        ProcessIdentity::checked(321, 133400000000000000).unwrap()
    }
    fn request(slot: SessionSlot) -> StorageRouteRequest {
        StorageRouteRequest {
            slot,
            expected_process: process(),
            observed_process: Some(process()),
            domain: StorageRouteDomain::LocalFiles,
            target: StorageRouteTarget::PrivateSession(slot),
            action: StorageRouteAction::Read,
        }
    }

    #[test]
    fn native_host_slot_cannot_be_adopted_by_independent_storage() {
        assert_eq!(
            preflight_storage_route(request(SessionSlot::Mc01)),
            StorageRoutePreflight::Denied(StorageRouteBlocker::NativeSlotOwnedElsewhere)
        );
    }

    #[test]
    fn every_independent_slot_refuses_peer_profile_reads_and_mutations() {
        for slot in [SessionSlot::Mc02, SessionSlot::Mc03, SessionSlot::Mc04] {
            for peer in SessionSlot::ALL {
                if slot == peer { continue; }
                for action in [StorageRouteAction::Read, StorageRouteAction::Create,
                    StorageRouteAction::Update, StorageRouteAction::Delete] {
                    for domain in [StorageRouteDomain::LocalFiles, StorageRouteDomain::RoamingFiles,
                        StorageRouteDomain::CurrentUserRegistry, StorageRouteDomain::NamedObjects] {
                        let result = preflight_storage_route(StorageRouteRequest {
                            target: StorageRouteTarget::PrivateSession(peer),
                            action,
                            domain,
                            ..request(slot)
                        });
                        assert_eq!(result, StorageRoutePreflight::Denied(StorageRouteBlocker::AnotherSession));
                    }
                }
            }
        }
    }

    #[test]
    fn host_account_boundary_and_unknown_owner_are_never_routed() {
        let original = request(SessionSlot::Mc02);
        for action in [StorageRouteAction::Read, StorageRouteAction::Create,
            StorageRouteAction::Update, StorageRouteAction::Delete] {
            assert_eq!(
                preflight_storage_route(StorageRouteRequest {
                    target: StorageRouteTarget::HostAccountData, action, ..original
                }),
                StorageRoutePreflight::Denied(StorageRouteBlocker::HostAccountBoundary)
            );
            assert_eq!(
                preflight_storage_route(StorageRouteRequest {
                    target: StorageRouteTarget::Unclassified, action, ..original
                }),
                StorageRoutePreflight::Denied(StorageRouteBlocker::UnknownOwner)
            );
        }
    }

    #[test]
    fn missing_reused_or_zero_creation_identity_is_always_refused() {
        let original = request(SessionSlot::Mc02);
        for observed in [None, ProcessIdentity::checked(321, original.expected_process.created_file_time + 1),
            ProcessIdentity::checked(322, original.expected_process.created_file_time)] {
            assert_eq!(preflight_storage_route(StorageRouteRequest {
                observed_process: observed, ..original
            }), StorageRoutePreflight::Denied(StorageRouteBlocker::UnpinnedOrStaleProcess));
        }
        assert_eq!(preflight_storage_route(StorageRouteRequest {
            expected_process: ProcessIdentity { pid: 0, created_file_time: 1 },
            ..original
        }), StorageRoutePreflight::Denied(StorageRouteBlocker::UnpinnedOrStaleProcess));
        assert_eq!(preflight_storage_route(StorageRouteRequest {
            expected_process: ProcessIdentity { pid: 321, created_file_time: 0 },
            ..original
        }), StorageRoutePreflight::Denied(StorageRouteBlocker::UnpinnedOrStaleProcess));
    }

    #[test]
    fn official_game_install_can_only_be_proposed_as_read_only_local_files() {
        let original = request(SessionSlot::Mc03);
        for action in [StorageRouteAction::Create, StorageRouteAction::Update, StorageRouteAction::Delete] {
            assert_eq!(preflight_storage_route(StorageRouteRequest {
                action, target: StorageRouteTarget::OfficialInstallation, ..original
            }), StorageRoutePreflight::Denied(StorageRouteBlocker::OfficialImageReadOnly));
        }
        for domain in [StorageRouteDomain::RoamingFiles,
            StorageRouteDomain::CurrentUserRegistry, StorageRouteDomain::NamedObjects] {
            assert_eq!(preflight_storage_route(StorageRouteRequest {
                domain, target: StorageRouteTarget::OfficialInstallation, ..original
            }), StorageRoutePreflight::Denied(StorageRouteBlocker::OfficialImageNotFilesystem));
        }
        assert_eq!(preflight_storage_route(StorageRouteRequest {
            target: StorageRouteTarget::OfficialInstallation, ..original
        }), StorageRoutePreflight::NeedsNativeVerification {
            domain: StorageRouteDomain::LocalFiles, required: INSTALL_REQUIRED
        });
    }

    #[test]
    fn roaming_must_preserve_environment_and_known_folder_as_separate_proofs() {
        let value = preflight_storage_route(StorageRouteRequest {
            domain: StorageRouteDomain::RoamingFiles,
            ..request(SessionSlot::Mc02)
        });
        assert_eq!(value, StorageRoutePreflight::NeedsNativeVerification {
            domain: StorageRouteDomain::RoamingFiles,
            required: ROAMING_REQUIRED
        });
        assert!(ROAMING_REQUIRED.contains(&StorageSurface::RoamingEnvironment));
        assert!(ROAMING_REQUIRED.contains(&StorageSurface::RoamingKnownFolder));
        assert!(ROAMING_REQUIRED.contains(&StorageSurface::PeerDataAccess));
        assert!(ROAMING_REQUIRED.contains(&StorageSurface::RestartPersistence));
    }

    #[test]
    fn registry_and_named_objects_have_distinct_native_proof_requirements() {
        for (domain, main) in [
            (StorageRouteDomain::CurrentUserRegistry, StorageSurface::PrivateRegistry),
            (StorageRouteDomain::NamedObjects, StorageSurface::NamedObjects)
        ] {
            let result = preflight_storage_route(StorageRouteRequest {
                domain, action: StorageRouteAction::Delete, ..request(SessionSlot::Mc04)
            });
            if let StorageRoutePreflight::NeedsNativeVerification {required, ..} = result {
                assert!(required.contains(&main));
                assert!(required.contains(&StorageSurface::PeerDataAccess));
                assert!(required.contains(&StorageSurface::HostDataAccess));
            } else { panic!("Static route evaluation must remain non-authorizing"); }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_private_looking_path_does_not_clear_any_actual_access_requirement() {
        let mut report = StorageObservations::default();
        report.observe(StorageSurface::RoamingEnvironment, StorageFinding::PathOnly);
        report.observe(StorageSurface::RoamingKnownFolder, StorageFinding::PathOnly);
        report.observe(StorageSurface::PeerDataAccess, StorageFinding::AccessDenied);
        report.observe(StorageSurface::HostDataAccess, StorageFinding::AccessDenied);
        assert!(report.unresolved().contains(&StorageSurface::RoamingEnvironment));
        assert!(report.unresolved().contains(&StorageSurface::RoamingKnownFolder));
        assert!(!report.unresolved().contains(&StorageSurface::PeerDataAccess));
        assert!(!report.unresolved().contains(&StorageSurface::HostDataAccess));
    }

    #[test]
    fn missing_broker_and_registry_never_disappear_with_synthetic_storage() {
        let mut report = StorageObservations::default();
        for surface in StorageSurface::ALL {
            report.observe(surface, StorageFinding::PrivateAccessObserved);
        }
        report.observe(StorageSurface::PeerDataAccess, StorageFinding::AccessDenied);
        report.observe(StorageSurface::HostDataAccess, StorageFinding::AccessDenied);
        assert!(report.unresolved().is_empty());
        report.observe(StorageSurface::AuthenticationBroker, StorageFinding::Unknown);
        report.observe(StorageSurface::PrivateRegistry, StorageFinding::SharedOrLeaking);
        assert_eq!(
            report.unresolved(),
            vec![StorageSurface::PrivateRegistry, StorageSurface::AuthenticationBroker]
        );
    }
}
