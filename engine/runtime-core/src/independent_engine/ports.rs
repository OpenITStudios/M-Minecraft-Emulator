//! Independent provider boundaries. These traits describe future adapters,
//! they are NOT registered Tauri commands, launch permission or implementations.
//! The platform-specific owner is apps/desktop; runtime-core holds policy.

use super::{ProcessIdentity, ProcessPathSnapshot, SessionSlot, StorageObservations};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativePortError {
    NotImplemented,
    InvalidIdentity,
    StaleProcess,
    AccessDenied,
    UnsupportedGame,
    OperationFailed,
}

/// Windows process-token category only. An AppContainer token alone cannot
/// establish filesystem, registry, broker, singleton or account isolation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProcessContainment {
    #[default]
    Unknown,
    NoAppContainerToken,
    AppContainerTokenObserved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessObservation {
    pub identity: ProcessIdentity,
    pub containment: ProcessContainment,
    // Candidate observations alone do not prove official signature, account,
    // rendering, multiplayer, or the scope of the game's singleton.
    pub official_image_signature_observed: bool,
    pub visible_window_observed: bool,
}

pub trait ProcessObserverPort {
    fn observe_exact(&self, expected: ProcessIdentity) -> Result<ProcessObservation, NativePortError>;
}

pub trait StorageObserverPort {
    /// Bind storage evidence to the exact process already inspected. Any
    /// native implementation must revalidate or pin the OS process handle
    /// during its own inspection; slot labels alone are never sufficient.
    fn inspect_boundaries(
        &self,
        slot: SessionSlot,
        expected: ProcessIdentity,
    ) -> Result<StorageObservations, NativePortError>;
}

/// The path collector must be in the cooperating process itself, or must
/// obtain its readings over a bounded channel whose peer is authenticated
/// against the exact Windows process handle. Returning an A/B fixture label
/// or a host-side SHGetKnownFolderPath reading is insufficient.
pub trait CooperativePathObserverPort {
    fn observe_process_local_paths(
        &self,
        slot: SessionSlot,
        expected: ProcessIdentity,
    ) -> Result<ProcessPathSnapshot, NativePortError>;
}

/// The existing workspace_surface implementation owns window behavior.
/// A presentation port NEVER exposes stop/launch/reprovision as side effects.
pub trait WorkspacePresentationPort {
    fn select_exact(&mut self, expected: ProcessIdentity) -> Result<(), NativePortError>;
    fn restore_exact(&mut self, expected: ProcessIdentity) -> Result<(), NativePortError>;
}

/// Contract only. Every later adapter still requires the CURRENT admission
/// owner to authorize a mutation and must verify the signed official image,
/// exact token/process identity and persistent profile boundaries.
/// No credentials, tokens or Minecraft binary changes are accepted.
pub trait IndependentProviderPort {
    fn inspect(&self, slot: SessionSlot) -> Result<Option<ProcessIdentity>, NativePortError>;
    fn launch(&mut self, slot: SessionSlot) -> Result<ProcessIdentity, NativePortError>;
    fn request_normal_close(
        &mut self,
        expected: ProcessIdentity,
    ) -> Result<(), NativePortError>;
}

/// A deliberate safe default. Merely linking this module or creating a
/// provider value cannot launch a game or touch account-bearing profiles.
pub struct DisabledIndependentProvider;

impl IndependentProviderPort for DisabledIndependentProvider {
    fn inspect(&self, _slot: SessionSlot) -> Result<Option<ProcessIdentity>, NativePortError> {
        Err(NativePortError::NotImplemented)
    }

    fn launch(&mut self, _slot: SessionSlot) -> Result<ProcessIdentity, NativePortError> {
        Err(NativePortError::NotImplemented)
    }

    fn request_normal_close(
        &mut self,
        _expected: ProcessIdentity,
    ) -> Result<(), NativePortError> {
        Err(NativePortError::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_provider_must_refuse_all_mutating_and_read_routes() {
        let mut provider = DisabledIndependentProvider;
        let key = ProcessIdentity::checked(42, 133400000000000000).unwrap();
        for slot in SessionSlot::ALL {
            assert_eq!(provider.inspect(slot), Err(NativePortError::NotImplemented));
            assert_eq!(provider.launch(slot), Err(NativePortError::NotImplemented));
        }
        assert_eq!(
            provider.request_normal_close(key),
            Err(NativePortError::NotImplemented)
        );
    }
}
