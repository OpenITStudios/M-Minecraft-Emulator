//! Read-only orchestration of independently implemented native observation
//! ports. This never launches/stops Minecraft or mutates Windows state.
//! Observations remain untrusted until a future real Windows acceptance run.

use super::{
    NativePortError, ProcessIdentity, ProcessObservation, ProcessObserverPort,
    SessionSlot, StorageObservations, StorageObserverPort, StorageRouteBlocker,
    StorageRouteDomain, StorageRoutePreflight, StorageRouteRequest, StorageSurface,
    StorageFinding, preflight_storage_route,
};

#[derive(Clone, Debug)]
pub struct SessionInspection {
    pub process: ProcessObservation,
    pub storage: StorageObservations,
}

/// Each inspection starts from an already selected, full-width process key.
/// A stale/mismatching observation stops BEFORE querying any storage surface.
/// This snapshot does not convey launch authority, OS handle ownership or an
/// authenticated Minecraft account; the existing desktop gate is unchanged.
pub fn inspect_existing<P: ProcessObserverPort, S: StorageObserverPort>(
    process: &P,
    storage: &S,
    slot: SessionSlot,
    expected: ProcessIdentity,
) -> Result<SessionInspection, NativePortError> {
    let observed = process.observe_exact(expected)?;
    if observed.identity != expected {
        return Err(NativePortError::StaleProcess);
    }
    let surfaces = storage.inspect_boundaries(slot, expected)?;
    Ok(SessionInspection {
        process: observed,
        storage: surfaces,
    })
}


/// A native-bound **diagnostic** outcome, never an access permit. A caller
/// cannot manufacture Windows observations by supplying observed_process.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouteBoundaryInspection {
    Denied(StorageRouteBlocker),
    /// Even zero gaps does NOT grant access. Collector provenance,
    /// authenticated login, official game image and real OS enforcement
    /// have separate owners and acceptance requirements.
    PendingNativeAcceptance {
        domain: StorageRouteDomain,
        required: Vec<StorageSurface>,
        unresolved: Vec<StorageSurface>,
        dangerous: Vec<StorageSurface>,
    },
}

/// Connect the canonical storage preflight to *fresh* exact-process and
/// storage observations. Denied routes do not open process/storage probes.
/// In-flight process exit/token-category changes invalidate everything.
/// Read-only observations never mutate filesystem, HKCU, services or accounts.
pub fn inspect_storage_route<P: ProcessObserverPort, S: StorageObserverPort>(
    process: &P,
    storage: &S,
    request: StorageRouteRequest,
) -> Result<RouteBoundaryInspection, NativePortError> {
    // A caller's observed_process is not native attestation. Reject an
    // explicitly conflicting value and then obtain actual port evidence.
    if request.observed_process.is_some()
        && request.observed_process != Some(request.expected_process)
    {
        return Ok(RouteBoundaryInspection::Denied(
            StorageRouteBlocker::UnpinnedOrStaleProcess,
        ));
    }
    let tentative = preflight_storage_route(StorageRouteRequest {
        observed_process: Some(request.expected_process),
        ..request
    });
    if let StorageRoutePreflight::Denied(blocker) = tentative {
        return Ok(RouteBoundaryInspection::Denied(blocker));
    }

    let before = process.observe_exact(request.expected_process)?;
    if before.identity != request.expected_process {
        return Err(NativePortError::StaleProcess);
    }
    let verified = preflight_storage_route(StorageRouteRequest {
        observed_process: Some(before.identity),
        ..request
    });
    let (domain, required) = match verified {
        StorageRoutePreflight::Denied(blocker) => {
            return Ok(RouteBoundaryInspection::Denied(blocker));
        }
        StorageRoutePreflight::NeedsNativeVerification { domain, required } => {
            (domain, required)
        }
    };
    let observed = storage.inspect_boundaries(request.slot, request.expected_process)?;
    let after = process.observe_exact(request.expected_process)?;
    if after != before {
        return Err(NativePortError::StaleProcess);
    }
    let unresolved = required
        .iter()
        .copied()
        .filter(|surface| {
            let wanted = match surface {
                StorageSurface::PeerDataAccess | StorageSurface::HostDataAccess => {
                    StorageFinding::AccessDenied
                }
                _ => StorageFinding::PrivateAccessObserved,
            };
            observed.finding(*surface) != wanted
        })
        .collect();
    let dangerous = required
        .iter()
        .copied()
        .filter(|surface| matches!(
            observed.finding(*surface),
            StorageFinding::PathOutsideProfile | StorageFinding::SharedOrLeaking
        ))
        .collect();
    Ok(RouteBoundaryInspection::PendingNativeAcceptance {
        domain,
        required: required.to_vec(),
        unresolved,
        dangerous,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::independent_engine::{
        ProcessContainment, StorageFinding, StorageSurface,
    };
    use std::cell::Cell;

    struct FakeProcess {
        reported: ProcessIdentity,
        fail: bool,
    }
    impl ProcessObserverPort for FakeProcess {
        fn observe_exact(
            &self,
            _expected: ProcessIdentity,
        ) -> Result<ProcessObservation, NativePortError> {
            if self.fail {
                return Err(NativePortError::AccessDenied);
            }
            Ok(ProcessObservation {
                identity: self.reported,
                containment: ProcessContainment::Unknown,
                official_image_signature_observed: false,
                visible_window_observed: false,
            })
        }
    }

    #[derive(Default)]
    struct FakeStorage {
        calls: Cell<usize>,
        last_identity: Cell<Option<ProcessIdentity>>,
    }
    impl StorageObserverPort for FakeStorage {
        fn inspect_boundaries(
            &self,
            _slot: SessionSlot,
            expected: ProcessIdentity,
        ) -> Result<StorageObservations, NativePortError> {
            self.calls.set(self.calls.get() + 1);
            self.last_identity.set(Some(expected));
            let mut value = StorageObservations::default();
            value.observe(StorageSurface::LocalAppData, StorageFinding::PathOnly);
            Ok(value)
        }
    }


    fn route_request() -> StorageRouteRequest {
        let identity = ProcessIdentity::checked(33, 133400000000000000).unwrap();
        StorageRouteRequest {
            slot: SessionSlot::Mc02,
            expected_process: identity,
            observed_process: None,
            domain: StorageRouteDomain::RoamingFiles,
            target: crate::independent_engine::StorageRouteTarget::PrivateSession(SessionSlot::Mc02),
            action: crate::independent_engine::StorageRouteAction::Read,
        }
    }

    #[test]
    fn denied_routes_never_invoke_even_process_inspection() {
        let request = route_request();
        let process = CountingProcess::default();
        let storage = FakeStorage::default();
        for target in [
            crate::independent_engine::StorageRouteTarget::HostAccountData,
            crate::independent_engine::StorageRouteTarget::PrivateSession(SessionSlot::Mc03),
            crate::independent_engine::StorageRouteTarget::Unclassified,
        ] {
            assert!(matches!(
                inspect_storage_route(
                    &process, &storage,
                    StorageRouteRequest { target, ..request }
                ).unwrap(),
                RouteBoundaryInspection::Denied(_)
            ));
        }
        assert_eq!(process.calls.get(), 0);
        assert_eq!(storage.calls.get(), 0);
    }

    #[derive(Default)]
    struct CountingProcess {
        calls: Cell<u32>,
        stale_on: Option<u32>,
        changed_token_on: Option<u32>,
    }
    impl ProcessObserverPort for CountingProcess {
        fn observe_exact(
            &self,
            expected: ProcessIdentity,
        ) -> Result<ProcessObservation, NativePortError> {
            let call = self.calls.get() + 1;
            self.calls.set(call);
            if self.stale_on == Some(call) {
                return Err(NativePortError::StaleProcess);
            }
            Ok(ProcessObservation {
                identity: expected,
                containment: if self.changed_token_on == Some(call) {
                    ProcessContainment::NoAppContainerToken
                } else {
                    ProcessContainment::AppContainerTokenObserved
                },
                official_image_signature_observed: false,
                visible_window_observed: false,
            })
        }
    }

    #[test]
    fn unknown_native_storage_reports_each_roaming_gap_without_authorizing() {
        let request = route_request();
        let process = CountingProcess::default();
        let storage = FakeStorage::default();
        let result = inspect_storage_route(&process, &storage, request).unwrap();
        match result {
            RouteBoundaryInspection::PendingNativeAcceptance {
                domain, required, unresolved, dangerous
            } => {
                assert_eq!(domain, StorageRouteDomain::RoamingFiles);
                assert!(required.contains(&StorageSurface::RoamingEnvironment));
                assert!(required.contains(&StorageSurface::RoamingKnownFolder));
                assert_eq!(required, unresolved);
                assert!(dangerous.is_empty());
            }
            _ => panic!("No launch grant or authorized routing state exists"),
        }
        assert_eq!(process.calls.get(), 2);
        assert_eq!(storage.calls.get(), 1);
        assert_eq!(storage.last_identity.get(), Some(request.expected_process));
    }

    #[test]
    fn stale_after_storage_and_changed_token_invalidates_result() {
        let req = route_request();
        for process in [
            CountingProcess { stale_on: Some(2), ..Default::default() },
            CountingProcess { changed_token_on: Some(2), ..Default::default() },
        ] {
            let storage = FakeStorage::default();
            assert_eq!(
                inspect_storage_route(&process, &storage, req).unwrap_err(),
                NativePortError::StaleProcess
            );
            assert_eq!(storage.calls.get(), 1);
        }
    }

    #[test]
    fn caller_supplied_stale_identity_does_not_trigger_platform_io() {
        let req = route_request();
        let process = CountingProcess::default();
        let storage = FakeStorage::default();
        let observed = ProcessIdentity::checked(
            req.expected_process.pid, req.expected_process.created_file_time + 1
        ).unwrap();
        let result = inspect_storage_route(&process, &storage, StorageRouteRequest {
            observed_process: Some(observed), ..req
        }).unwrap();
        assert_eq!(result, RouteBoundaryInspection::Denied(
            StorageRouteBlocker::UnpinnedOrStaleProcess
        ));
        assert_eq!(process.calls.get(), 0);
        assert_eq!(storage.calls.get(), 0);
    }

    #[derive(Default)]
    struct PositiveStorage;
    impl StorageObserverPort for PositiveStorage {
        fn inspect_boundaries(
            &self, _: SessionSlot, _: ProcessIdentity
        ) -> Result<StorageObservations, NativePortError> {
            let mut data = StorageObservations::default();
            for surface in StorageSurface::ALL {
                data.observe(surface, match surface {
                    StorageSurface::PeerDataAccess | StorageSurface::HostDataAccess =>
                        StorageFinding::AccessDenied,
                    _ => StorageFinding::PrivateAccessObserved
                });
            }
            Ok(data)
        }
    }

    #[test]
    fn even_perfect_self_reported_findings_cannot_enable_access() {
        let result = inspect_storage_route(
            &CountingProcess::default(),
            &PositiveStorage,
            route_request(),
        ).unwrap();
        match result {
            RouteBoundaryInspection::PendingNativeAcceptance {
                unresolved, dangerous, ..
            } => {
                assert!(unresolved.is_empty());
                assert!(dangerous.is_empty());
                // There is intentionally no Approved or LaunchAllowed result.
            }
            _ => panic!("Self-reported findings cannot be promoted"),
        }
    }

    struct LeakingStorage;
    impl StorageObserverPort for LeakingStorage {
        fn inspect_boundaries(
            &self, _: SessionSlot, _: ProcessIdentity
        ) -> Result<StorageObservations, NativePortError> {
            let mut data = StorageObservations::default();
            data.observe(
                StorageSurface::RoamingKnownFolder,
                StorageFinding::PathOutsideProfile,
            );
            data.observe(StorageSurface::HostDataAccess, StorageFinding::SharedOrLeaking);
            Ok(data)
        }
    }

    #[test]
    fn unsafe_or_host_shared_paths_are_reported_as_dangerous_not_passed() {
        let result = inspect_storage_route(
            &CountingProcess::default(),
            &LeakingStorage,
            route_request(),
        ).unwrap();
        match result {
            RouteBoundaryInspection::PendingNativeAcceptance {
                unresolved, dangerous, ..
            } => {
                assert!(unresolved.contains(&StorageSurface::RoamingKnownFolder));
                assert!(unresolved.contains(&StorageSurface::HostDataAccess));
                assert_eq!(
                    dangerous,
                    vec![StorageSurface::RoamingKnownFolder, StorageSurface::HostDataAccess]
                );
            }
            _ => panic!("Shared host access cannot pass"),
        }
    }
    #[test]
    fn stale_pid_creation_identity_refuses_storage_inspection() {
        let expected = ProcessIdentity::checked(10, 133400000000000000).unwrap();
        let wrong = ProcessIdentity::checked(10, 133400000000000001).unwrap();
        let process = FakeProcess {
            reported: wrong,
            fail: false,
        };
        let storage = FakeStorage::default();
        assert!(matches!(
            inspect_existing(&process, &storage, SessionSlot::Mc02, expected),
            Err(NativePortError::StaleProcess)
        ));
        assert_eq!(storage.calls.get(), 0);
        assert_eq!(storage.last_identity.get(), None);
    }

    #[test]
    fn denied_native_process_proof_cannot_trigger_storage_probe() {
        let expected = ProcessIdentity::checked(10, 133400000000000000).unwrap();
        let process = FakeProcess {
            reported: expected,
            fail: true,
        };
        let storage = FakeStorage::default();
        assert!(matches!(
            inspect_existing(&process, &storage, SessionSlot::Mc02, expected),
            Err(NativePortError::AccessDenied)
        ));
        assert_eq!(storage.calls.get(), 0);
        assert_eq!(storage.last_identity.get(), None);
    }

    #[test]
    fn matching_identity_keeps_storage_uncertainty_explicit() {
        let expected = ProcessIdentity::checked(10, 133400000000000000).unwrap();
        let process = FakeProcess {
            reported: expected,
            fail: false,
        };
        let storage = FakeStorage::default();
        let result =
            inspect_existing(&process, &storage, SessionSlot::Mc02, expected).unwrap();
        assert_eq!(result.process.identity, expected);
        assert_eq!(result.process.containment, ProcessContainment::Unknown);
        assert_eq!(storage.calls.get(), 1);
        assert_eq!(storage.last_identity.get(), Some(expected));
        assert_eq!(
            result.storage.finding(StorageSurface::LocalAppData),
            StorageFinding::PathOnly
        );
        assert!(result.storage.unresolved().contains(&StorageSurface::RoamingKnownFolder));
        assert!(result.storage.unresolved().contains(&StorageSurface::AuthenticationBroker));
    }
}
