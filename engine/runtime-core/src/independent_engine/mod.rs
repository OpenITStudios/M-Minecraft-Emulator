//! Independent, Minecraft-focused engine source contracts.
//!
//! Domain-only architecture: no OS calls, credentials, persistent lifecycle
//! registry, launcher, daemon, driver, or Sandboxie-derived implementation.
//! The existing desktop admission and runtime lifecycle remain authoritative.

mod acceptance;
mod evidence;
mod fixture_wire;
mod identity;
mod inspection;
mod paths;
mod registry;
mod operations;
mod ports;
mod storage;

pub use acceptance::{
    review_game_acceptance, ClientGameCheck, ClientGameSample,
    GameTrial, GameTrialError, GroupGameCheck, JointGameSample,
    PendingGameAcceptance, ReportedGameFinding,
};
pub use evidence::{
    review_bound_storage_facts, BoundStorageFact, EvidenceReviewError,
    StorageEvidenceSource, StorageProbeMethod,
};
pub use fixture_wire::{
    decode_synthetic_fixture_pair, decode_synthetic_fixture_paths,
    SyntheticFixturePhase, SyntheticFrameError, UnboundFixturePaths,
    SYNTHETIC_FRAME_BYTES,
};
pub use identity::{ProcessIdentity, SessionSlot};
pub use inspection::{
    inspect_existing, inspect_pair_boundaries, inspect_storage_route,
    PairBoundaryError, PairBoundaryFact, PairBoundaryFinding, PairBoundaryIssue,
    PairBoundarySnapshot, PendingPairInspection, RouteBoundaryInspection, SessionInspection,
};
pub use paths::{
    inspect_cooperative_paths, review_process_path_snapshot,
    BoundPathInspectionError, PathLocation, ProcessPathSnapshot,
};
pub use registry::{
    review_synthetic_registry_probe, RegistryCanaryRead, RegistryFixtureIssue,
    RegistryFixtureReview, RegistryProbeError, RegistryProbeScope,
    SyntheticRegistryProbe,
};
pub use operations::{OperationClass, SessionOperation};
pub use ports::{
    CooperativePathObserverPort, DisabledIndependentProvider,
    IndependentProviderPort, NativePortError, PairBoundaryObserverPort, ProcessContainment,
    ProcessObservation, ProcessObserverPort, StorageObserverPort,
    WorkspacePresentationPort,
};
pub use storage::{
    preflight_storage_route, StorageFinding, StorageObservations,
    StorageRouteAction, StorageRouteBlocker, StorageRouteDomain, StorageRoutePreflight,
    StorageRouteRequest, StorageRouteTarget, StorageSurface,
};
