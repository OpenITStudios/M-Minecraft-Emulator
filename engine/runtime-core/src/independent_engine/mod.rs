//! Independent, Minecraft-focused engine source contracts.
//!
//! Domain-only architecture: no OS calls, credentials, persistent lifecycle
//! registry, launcher, daemon, driver, or Sandboxie-derived implementation.
//! The existing desktop admission and runtime lifecycle remain authoritative.

mod identity;
mod inspection;
mod operations;
mod ports;
mod storage;

pub use identity::{ProcessIdentity, SessionSlot};
pub use inspection::{inspect_existing, SessionInspection};
pub use operations::{OperationClass, SessionOperation};
pub use ports::{
    DisabledIndependentProvider, IndependentProviderPort, NativePortError,
    ProcessContainment, ProcessObservation, ProcessObserverPort, StorageObserverPort,
    WorkspacePresentationPort,
};
pub use storage::{StorageFinding, StorageObservations, StorageSurface};
