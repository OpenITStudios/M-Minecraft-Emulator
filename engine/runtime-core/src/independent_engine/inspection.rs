//! Read-only orchestration of independently implemented native observation
//! ports. This never launches/stops Minecraft or mutates Windows state.
//! Observations remain untrusted until a future real Windows acceptance run.

use super::{
    NativePortError, ProcessIdentity, ProcessObservation, ProcessObserverPort,
    SessionSlot, StorageObservations, StorageObserverPort,
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
    let surfaces = storage.inspect_boundaries(slot)?;
    Ok(SessionInspection {
        process: observed,
        storage: surfaces,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::independent_engine::{StorageFinding, StorageSurface};
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
                official_image_signature_observed: false,
                visible_window_observed: false,
            })
        }
    }

    struct FakeStorage {
        calls: Cell<usize>,
    }
    impl StorageObserverPort for FakeStorage {
        fn inspect_boundaries(
            &self,
            _slot: SessionSlot,
        ) -> Result<StorageObservations, NativePortError> {
            self.calls.set(self.calls.get() + 1);
            let mut value = StorageObservations::default();
            value.observe(StorageSurface::LocalAppData, StorageFinding::PathOnly);
            Ok(value)
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
        let storage = FakeStorage { calls: Cell::new(0) };
        assert!(matches!(
            inspect_existing(&process, &storage, SessionSlot::Mc02, expected),
            Err(NativePortError::StaleProcess)
        ));
        assert_eq!(storage.calls.get(), 0);
    }

    #[test]
    fn denied_native_process_proof_cannot_trigger_storage_probe() {
        let expected = ProcessIdentity::checked(10, 133400000000000000).unwrap();
        let process = FakeProcess {
            reported: expected,
            fail: true,
        };
        let storage = FakeStorage { calls: Cell::new(0) };
        assert!(matches!(
            inspect_existing(&process, &storage, SessionSlot::Mc02, expected),
            Err(NativePortError::AccessDenied)
        ));
        assert_eq!(storage.calls.get(), 0);
    }

    #[test]
    fn matching_identity_keeps_storage_uncertainty_explicit() {
        let expected = ProcessIdentity::checked(10, 133400000000000000).unwrap();
        let process = FakeProcess {
            reported: expected,
            fail: false,
        };
        let storage = FakeStorage { calls: Cell::new(0) };
        let result =
            inspect_existing(&process, &storage, SessionSlot::Mc02, expected).unwrap();
        assert_eq!(result.process.identity, expected);
        assert_eq!(storage.calls.get(), 1);
        assert_eq!(
            result.storage.finding(StorageSurface::LocalAppData),
            StorageFinding::PathOnly
        );
        assert!(result.storage.unresolved().contains(&StorageSurface::RoamingKnownFolder));
        assert!(result.storage.unresolved().contains(&StorageSurface::AuthenticationBroker));
    }
}
