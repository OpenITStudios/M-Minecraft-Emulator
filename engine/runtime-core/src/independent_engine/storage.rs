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
