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


// Manager-owned **relative** file paths only. These are never Minecraft's
// arbitrary world names or paths passed to Win32 file APIs. In particular,
// a lexical path check does not detect a reparse point, ACL or volume escape.
// Native adapters must still open an already verified private directory
// handle, reject reparse traversals and enforce access with Windows ACLs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingPrivateFilePlan {
    pub slot: SessionSlot,
    pub domain: StorageRouteDomain,
    pub action: StorageRouteAction,
    pub components: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrivateFilePlanError {
    Denied(StorageRouteBlocker),
    NotPrivateFileStorage,
    EmptyOrOversizedPath,
    UnsafeComponent,
    ReservedWindowsName,
}

fn is_reserved_windows_component(name: &str) -> bool {
    // Windows reserves device basenames even with extensions and in any case.
    // This is a conservative ASCII namespace for manager-owned metadata only.
    let stem = name.split('.').next().unwrap_or("");
    let stem = stem.to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit())
}

/// Plan a bounded, manager-owned file location under one logical session.
/// This returns only a *pending plan*; it never resolves absolute paths,
/// creates directories, follows symlinks, opens files, or grants access.
///
/// Deliberately restrict metadata names to portable ASCII. Minecraft world
/// filenames and arbitrary application paths MUST NOT be passed here.
pub fn plan_private_file_access(
    request: StorageRouteRequest,
    relative: &str,
) -> Result<PendingPrivateFilePlan, PrivateFilePlanError> {
    use PrivateFilePlanError as Failure;

    match preflight_storage_route(request) {
        StorageRoutePreflight::Denied(blocker) => return Err(Failure::Denied(blocker)),
        StorageRoutePreflight::NeedsNativeVerification { .. } => {}
    }
    if !matches!(request.target, StorageRouteTarget::PrivateSession(owner) if owner == request.slot)
        || !matches!(request.domain, StorageRouteDomain::LocalFiles | StorageRouteDomain::RoamingFiles)
    {
        return Err(Failure::NotPrivateFileStorage);
    }
    // Backslash is intentionally not accepted on *any* host platform so a
    // Linux-based test cannot accidentally approve Windows traversal.
    if relative.is_empty() || relative.len() > 240 || relative.starts_with('/')
        || relative.ends_with('/') || relative.contains('\\')
    {
        return Err(Failure::EmptyOrOversizedPath);
    }
    let mut components = Vec::new();
    for component in relative.split('/') {
        if component.is_empty() || component.len() > 64 || component.starts_with('.')
            || component.ends_with('.') || component.ends_with(' ')
            || !component.bytes().all(|ch| {
                ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-' || ch == b'.'
            })
        {
            return Err(Failure::UnsafeComponent);
        }
        if is_reserved_windows_component(component) {
            return Err(Failure::ReservedWindowsName);
        }
        components.push(component.to_owned());
        if components.len() > 12 {
            return Err(Failure::EmptyOrOversizedPath);
        }
    }
    Ok(PendingPrivateFilePlan {
        slot: request.slot, domain: request.domain, action: request.action,
        components,
    })
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
    fn manager_owned_file_plan_accepts_bounded_relative_metadata_only() {
        for slot in [SessionSlot::Mc02, SessionSlot::Mc03, SessionSlot::Mc04] {
            for domain in [StorageRouteDomain::LocalFiles, StorageRouteDomain::RoamingFiles] {
                for action in [
                    StorageRouteAction::Read, StorageRouteAction::Create,
                    StorageRouteAction::Update, StorageRouteAction::Delete,
                ] {
                    let plan = plan_private_file_access(StorageRouteRequest {
                        domain, action, ..request(slot)
                    }, "state/checkpoint-01.json").unwrap();
                    assert_eq!(plan.slot, slot);
                    assert_eq!(plan.domain, domain);
                    assert_eq!(plan.action, action);
                    assert_eq!(plan.components, ["state", "checkpoint-01.json"]);
                }
            }
        }
        // Planning is not a direct file path / native access permission.
    }

    #[test]
    fn private_file_plan_rejects_cross_slot_host_and_unverified_keys() {
        let own = request(SessionSlot::Mc02);
        for peer in [SessionSlot::Mc01, SessionSlot::Mc03, SessionSlot::Mc04] {
            assert_eq!(plan_private_file_access(StorageRouteRequest {
                target: StorageRouteTarget::PrivateSession(peer), ..own
            }, "state.json"), Err(PrivateFilePlanError::Denied(
                StorageRouteBlocker::AnotherSession
            )));
        }
        assert_eq!(plan_private_file_access(StorageRouteRequest {
            target: StorageRouteTarget::HostAccountData, ..own
        }, "state.json"), Err(PrivateFilePlanError::Denied(
            StorageRouteBlocker::HostAccountBoundary
        )));
        assert_eq!(plan_private_file_access(StorageRouteRequest {
            slot: SessionSlot::Mc01,
            target: StorageRouteTarget::PrivateSession(SessionSlot::Mc01), ..own
        }, "state.json"), Err(PrivateFilePlanError::Denied(
            StorageRouteBlocker::NativeSlotOwnedElsewhere
        )));
        assert_eq!(plan_private_file_access(StorageRouteRequest {
            observed_process: None, ..own
        }, "state.json"), Err(PrivateFilePlanError::Denied(
            StorageRouteBlocker::UnpinnedOrStaleProcess
        )));
        // An official installation must not become a mutable metadata root.
        assert_eq!(plan_private_file_access(StorageRouteRequest {
            target: StorageRouteTarget::OfficialInstallation,
            action: StorageRouteAction::Read, ..own
        }, "state.json"), Err(PrivateFilePlanError::NotPrivateFileStorage));
        for domain in [
            StorageRouteDomain::CurrentUserRegistry,
            StorageRouteDomain::NamedObjects,
        ] {
            assert_eq!(plan_private_file_access(StorageRouteRequest {
                domain, ..own
            }, "state.json"), Err(PrivateFilePlanError::NotPrivateFileStorage));
        }
    }

    #[test]
    fn private_file_plan_refuses_windows_path_ambiguity_and_traversal() {
        let req = request(SessionSlot::Mc02);
        for bad in [
            ".", "..", "../peer", "data/../peer", "./data", ".private",
            "/root", "data/", "data//x", "\\\\server\\share",
            "C:\\host", "C:host", "data\\peer", "a:b", "data/x*",
            "data/x?", "data/<test>", "data/pipe|secret",
            "data/name ", "data/name.", "data/name\tmore",
            "data/has space", "data/emoji🙂", "data/NUl", "com1.txt",
            "AUX.log", "prn", "LPT9", "CONIN$.txt", "CONOUT$",
            "foo/COM0.bin",
        ] {
            assert!(
                plan_private_file_access(req, bad).is_err(),
                "Unexpected accepted unsafe relative filename: {bad}"
            );
        }
        assert_eq!(
            plan_private_file_access(req, ""),
            Err(PrivateFilePlanError::EmptyOrOversizedPath)
        );
        assert_eq!(
            plan_private_file_access(req, &"x".repeat(241)),
            Err(PrivateFilePlanError::EmptyOrOversizedPath)
        );
        assert_eq!(
            plan_private_file_access(req, &"x".repeat(65)),
            Err(PrivateFilePlanError::UnsafeComponent)
        );
        assert_eq!(
            plan_private_file_access(req, &vec!["x"; 13].join("/")),
            Err(PrivateFilePlanError::EmptyOrOversizedPath)
        );
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
