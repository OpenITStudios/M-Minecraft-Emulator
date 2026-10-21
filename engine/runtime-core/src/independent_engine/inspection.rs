//! Read-only orchestration of independently implemented native observation
//! ports. This never launches/stops Minecraft or mutates Windows state.
//! Observations remain untrusted until a future real Windows acceptance run.

use super::{
    NativePortError, ProcessIdentity, ProcessObservation, ProcessObserverPort,
    SessionSlot, StorageObservations, StorageObserverPort, StorageRouteBlocker,
    StorageRouteDomain, StorageRoutePreflight, StorageRouteRequest, StorageSurface,
    StorageFinding, StorageEvidenceSource, PairBoundaryObserverPort, preflight_storage_route,
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
    if expected.pid == 0 || expected.created_file_time == 0 {
        return Err(NativePortError::InvalidIdentity);
    }
    let observed = process.observe_exact(expected)?;
    if observed.identity != expected {
        return Err(NativePortError::StaleProcess);
    }
    let surfaces = storage.inspect_boundaries(slot, expected)?;
    // A reading made by another collector is not a process-lifetime lease.
    // Revalidate full PID/creation identity and token category after storage,
    // so an exit/reuse or containment change cannot create a valid snapshot.
    let after = process.observe_exact(expected)?;
    if after != observed {
        return Err(NativePortError::StaleProcess);
    }
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
        /// Private-looking paths outside the profile are not confirmed leaks.
        path_risks: Vec<StorageSurface>,
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
    let path_risks = required
        .iter()
        .copied()
        .filter(|surface| observed.finding(*surface) == StorageFinding::PathOutsideProfile)
        .collect();
    let dangerous = required
        .iter()
        .copied()
        .filter(|surface| observed.finding(*surface) == StorageFinding::SharedOrLeaking)
        .collect();
    Ok(RouteBoundaryInspection::PendingNativeAcceptance {
        domain,
        required: required.to_vec(),
        unresolved,
        path_risks,
        dangerous,
    })
}


/// Results are sample-specific, non-authorizing collector claims.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PairBoundaryFinding { Unknown, NoConflictInSample, ConflictObserved }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PairBoundaryFact {
    pub source: StorageEvidenceSource,
    pub finding: PairBoundaryFinding,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PairBoundarySnapshot {
    pub first_slot: SessionSlot,
    pub second_slot: SessionSlot,
    pub first: ProcessIdentity,
    pub second: ProcessIdentity,
    pub singleton: PairBoundaryFact,
    pub broker: PairBoundaryFact,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PairBoundaryError {
    InvalidPair, WrongSlot, WrongProcess, InadequateSource, Native(NativePortError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PairBoundaryIssue {
    SingletonUnknown, SingletonConflict, BrokerUnknown, BrokerConflict,
}

/// Pending even when issues is empty; never grants access or launch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingPairInspection {
    pub first: ProcessIdentity,
    pub second: ProcessIdentity,
    pub issues: Vec<PairBoundaryIssue>,
}

fn validate_pair_fact(fact: PairBoundaryFact, broker: bool) -> Result<(), PairBoundaryError> {
    match fact.finding {
        PairBoundaryFinding::Unknown => Ok(()),
        PairBoundaryFinding::NoConflictInSample
            if fact.source == StorageEvidenceSource::MinecraftRuntime => Ok(()),
        PairBoundaryFinding::ConflictObserved
            if fact.source == StorageEvidenceSource::MinecraftRuntime
                || (!broker && fact.source == StorageEvidenceSource::WindowsProcess) => Ok(()),
        _ => Err(PairBoundaryError::InadequateSource),
    }
}

/// Pin two different processes before/after a cooperating observer. Windows
/// identity, AppContainer SID and named-object canaries do not authenticate
/// game singleton or Microsoft broker compatibility. Collector source labels
/// require independent verification and are never production readiness.
pub fn inspect_pair_boundaries<P: ProcessObserverPort, Q: PairBoundaryObserverPort>(
    process: &P, observer: &Q, first_slot: SessionSlot, first: ProcessIdentity,
    second_slot: SessionSlot, second: ProcessIdentity,
) -> Result<PendingPairInspection, PairBoundaryError> {
    if first_slot == second_slot || first.pid == 0 || second.pid == 0
        || first.created_file_time == 0 || second.created_file_time == 0
        || first.pid == second.pid
    { return Err(PairBoundaryError::InvalidPair); }
    let left = process.observe_exact(first).map_err(PairBoundaryError::Native)?;
    if left.identity != first { return Err(PairBoundaryError::Native(NativePortError::StaleProcess)); }
    let right = process.observe_exact(second).map_err(PairBoundaryError::Native)?;
    if right.identity != second { return Err(PairBoundaryError::Native(NativePortError::StaleProcess)); }
    let snapshot = observer.observe_pair_boundaries(first_slot, first, second_slot, second)
        .map_err(PairBoundaryError::Native)?;
    if snapshot.first_slot != first_slot || snapshot.second_slot != second_slot {
        return Err(PairBoundaryError::WrongSlot);
    }
    if snapshot.first != first || snapshot.second != second {
        return Err(PairBoundaryError::WrongProcess);
    }
    validate_pair_fact(snapshot.singleton, false)?;
    validate_pair_fact(snapshot.broker, true)?;
    let after_left = process.observe_exact(first).map_err(PairBoundaryError::Native)?;
    let after_right = process.observe_exact(second).map_err(PairBoundaryError::Native)?;
    if left != after_left || right != after_right {
        return Err(PairBoundaryError::Native(NativePortError::StaleProcess));
    }
    let mut issues = Vec::new();
    match snapshot.singleton.finding {
        PairBoundaryFinding::Unknown => issues.push(PairBoundaryIssue::SingletonUnknown),
        PairBoundaryFinding::ConflictObserved => issues.push(PairBoundaryIssue::SingletonConflict),
        PairBoundaryFinding::NoConflictInSample => {}
    }
    match snapshot.broker.finding {
        PairBoundaryFinding::Unknown => issues.push(PairBoundaryIssue::BrokerUnknown),
        PairBoundaryFinding::ConflictObserved => issues.push(PairBoundaryIssue::BrokerConflict),
        PairBoundaryFinding::NoConflictInSample => {}
    }
    Ok(PendingPairInspection { first, second, issues })
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
    fn invalid_identity_prevents_all_existing_inspection_io() {
        let storage = FakeStorage::default();
        let process = CountingProcess::default();
        let valid = route_request().expected_process;
        for invalid in [
            ProcessIdentity { pid: 0, created_file_time: valid.created_file_time },
            ProcessIdentity { pid: valid.pid, created_file_time: 0 },
        ] {
            assert_eq!(
                inspect_existing(&process, &storage, SessionSlot::Mc02, invalid).unwrap_err(),
                NativePortError::InvalidIdentity
            );
        }
        assert_eq!(process.calls.get(), 0);
        assert_eq!(storage.calls.get(), 0);
    }

    #[test]
    fn existing_inspection_rechecks_exact_process_after_storage_read() {
        let key = route_request().expected_process;
        let proc = CountingProcess::default();
        let storage = FakeStorage::default();
        let report = inspect_existing(&proc, &storage, SessionSlot::Mc02, key).unwrap();
        assert_eq!(report.process.identity, key);
        assert_eq!(proc.calls.get(), 2);
        assert_eq!(storage.calls.get(), 1);
    }

    #[test]
    fn existing_inspection_rejects_post_read_process_exit_or_token_change() {
        let key = route_request().expected_process;
        for proc in [
            CountingProcess { stale_on: Some(2), ..Default::default() },
            CountingProcess { changed_token_on: Some(2), ..Default::default() },
        ] {
            let storage = FakeStorage::default();
            assert_eq!(
                inspect_existing(&proc, &storage, SessionSlot::Mc02, key).unwrap_err(),
                NativePortError::StaleProcess
            );
            assert_eq!(proc.calls.get(), 2);
            assert_eq!(storage.calls.get(), 1);
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
                domain, required, unresolved, path_risks, dangerous
            } => {
                assert_eq!(domain, StorageRouteDomain::RoamingFiles);
                assert!(required.contains(&StorageSurface::RoamingEnvironment));
                assert!(required.contains(&StorageSurface::RoamingKnownFolder));
                assert_eq!(required, unresolved);
                assert!(path_risks.is_empty());
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
                unresolved, path_risks, dangerous, ..
            } => {
                assert!(unresolved.is_empty());
                assert!(path_risks.is_empty());
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
    fn outside_profile_path_is_not_mislabeled_as_confirmed_host_leak() {
        let result = inspect_storage_route(
            &CountingProcess::default(),
            &LeakingStorage,
            route_request(),
        ).unwrap();
        match result {
            RouteBoundaryInspection::PendingNativeAcceptance {
                unresolved, path_risks, dangerous, ..
            } => {
                assert!(unresolved.contains(&StorageSurface::RoamingKnownFolder));
                assert!(unresolved.contains(&StorageSurface::HostDataAccess));
                assert_eq!(path_risks, vec![StorageSurface::RoamingKnownFolder]);
                assert_eq!(dangerous, vec![StorageSurface::HostDataAccess]);
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

#[cfg(test)]
mod pair_tests {
    use super::*;
    use crate::independent_engine::ProcessContainment;
    use std::cell::Cell;

    fn left() -> ProcessIdentity { ProcessIdentity::checked(701, 133400000000000701).unwrap() }
    fn right() -> ProcessIdentity { ProcessIdentity::checked(702, 133400000000000702).unwrap() }
    fn unknown() -> PairBoundaryFact {
        PairBoundaryFact { source: StorageEvidenceSource::WindowsProcess,
            finding: PairBoundaryFinding::Unknown }
    }
    fn sample() -> PairBoundarySnapshot {
        PairBoundarySnapshot { first_slot: SessionSlot::Mc02,
            second_slot: SessionSlot::Mc03, first: left(), second: right(),
            singleton: unknown(), broker: unknown() }
    }
    #[derive(Default)]
    struct Process {
        calls: Cell<u32>,
        change_at: Option<u32>,
        exit_at: Option<u32>,
    }
    impl ProcessObserverPort for Process {
        fn observe_exact(&self, expected: ProcessIdentity)
            -> Result<ProcessObservation, NativePortError> {
            let n = self.calls.get() + 1;
            self.calls.set(n);
            if self.exit_at == Some(n) { return Err(NativePortError::StaleProcess); }
            Ok(ProcessObservation { identity: expected,
                containment: if self.change_at == Some(n) {
                    ProcessContainment::NoAppContainerToken
                } else { ProcessContainment::AppContainerTokenObserved },
                official_image_signature_observed: false, visible_window_observed: false })
        }
    }
    struct Probe { calls: Cell<u32>, value: PairBoundarySnapshot }
    impl PairBoundaryObserverPort for Probe {
        fn observe_pair_boundaries(
            &self, _: SessionSlot, _: ProcessIdentity,
            _: SessionSlot, _: ProcessIdentity,
        ) -> Result<PairBoundarySnapshot, NativePortError> {
            self.calls.set(self.calls.get() + 1);
            Ok(self.value)
        }
    }
    fn probe(value: PairBoundarySnapshot) -> Probe {
        Probe { calls: Cell::new(0), value }
    }
    fn inspect(p: &Process, q: &Probe)
        -> Result<PendingPairInspection, PairBoundaryError> {
        inspect_pair_boundaries(p, q, SessionSlot::Mc02, left(), SessionSlot::Mc03, right())
    }

    #[test]
    fn rejects_duplicate_slot_and_pid_without_touching_observers() {
        let p = Process::default();
        let q = probe(sample());
        for (a, x, b, y) in [
            (SessionSlot::Mc02, left(), SessionSlot::Mc02, right()),
            (SessionSlot::Mc02, left(), SessionSlot::Mc03, left()),
            (SessionSlot::Mc02, left(), SessionSlot::Mc03,
                ProcessIdentity { pid: left().pid, created_file_time: right().created_file_time }),
            (SessionSlot::Mc02, ProcessIdentity { pid: 0, created_file_time: 1 },
                SessionSlot::Mc03, right()),
        ] {
            assert_eq!(inspect_pair_boundaries(&p, &q, a, x, b, y),
                Err(PairBoundaryError::InvalidPair));
        }
        assert_eq!(p.calls.get(), 0);
        assert_eq!(q.calls.get(), 0);
    }

    #[test]
    fn absent_real_game_observations_remain_two_diagnostic_gaps() {
        let p = Process::default();
        let q = probe(sample());
        assert_eq!(inspect(&p, &q).unwrap().issues,
            vec![PairBoundaryIssue::SingletonUnknown, PairBoundaryIssue::BrokerUnknown]);
        assert_eq!(p.calls.get(), 4);
        assert_eq!(q.calls.get(), 1);
    }

    #[test]
    fn swapped_slot_and_process_receipts_are_not_accepted() {
        assert_eq!(inspect(&Process::default(), &probe(PairBoundarySnapshot {
            second_slot: SessionSlot::Mc04, ..sample()
        })), Err(PairBoundaryError::WrongSlot));
        assert_eq!(inspect(&Process::default(), &probe(PairBoundarySnapshot {
            second: left(), ..sample()
        })), Err(PairBoundaryError::WrongProcess));
    }

    #[test]
    fn fixtures_and_windows_metadata_cannot_certify_no_singleton_conflict() {
        for source in [StorageEvidenceSource::SyntheticFixture,
            StorageEvidenceSource::WindowsProcess] {
            let q = probe(PairBoundarySnapshot { singleton: PairBoundaryFact {
                source, finding: PairBoundaryFinding::NoConflictInSample },
                ..sample() });
            assert_eq!(inspect(&Process::default(), &q),
                Err(PairBoundaryError::InadequateSource));
        }
    }

    #[test]
    fn broker_outcomes_require_real_minecraft_observation() {
        for source in [StorageEvidenceSource::WindowsProcess,
            StorageEvidenceSource::SyntheticFixture] {
            for finding in [PairBoundaryFinding::NoConflictInSample,
                PairBoundaryFinding::ConflictObserved] {
                let q = probe(PairBoundarySnapshot {
                    broker: PairBoundaryFact { source, finding }, ..sample()
                });
                assert_eq!(inspect(&Process::default(), &q),
                    Err(PairBoundaryError::InadequateSource));
            }
        }
    }

    #[test]
    fn observed_windows_singleton_conflict_is_only_a_warning() {
        let q = probe(PairBoundarySnapshot { singleton: PairBoundaryFact {
            source: StorageEvidenceSource::WindowsProcess,
            finding: PairBoundaryFinding::ConflictObserved }, ..sample() });
        assert_eq!(inspect(&Process::default(), &q).unwrap().issues,
            vec![PairBoundaryIssue::SingletonConflict, PairBoundaryIssue::BrokerUnknown]);
    }

    #[test]
    fn even_two_positive_game_reports_remain_pending_not_launch_authority() {
        let ok = PairBoundaryFact { source: StorageEvidenceSource::MinecraftRuntime,
            finding: PairBoundaryFinding::NoConflictInSample };
        let q = probe(PairBoundarySnapshot { singleton: ok, broker: ok, ..sample() });
        let report = inspect(&Process::default(), &q).unwrap();
        assert!(report.issues.is_empty());
        assert_eq!(report.first, left());
        assert_eq!(report.second, right());
    }

    #[test]
    fn exited_process_or_changed_token_invalidates_whole_pair() {
        for p in [
            Process { exit_at: Some(4), ..Default::default() },
            Process { change_at: Some(3), ..Default::default() },
        ] {
            let q = probe(sample());
            assert_eq!(inspect(&p, &q),
                Err(PairBoundaryError::Native(NativePortError::StaleProcess)));
            assert_eq!(q.calls.get(), 1);
        }
    }
}
