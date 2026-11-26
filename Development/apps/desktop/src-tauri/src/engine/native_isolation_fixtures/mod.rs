//! Disposable Windows isolation fixtures; never a product runtime or launch provider.
//! Compiled only by the parent engine's Windows test gate.

mod process_job;
mod registry_hive;
mod private_files;
mod root_acl;
mod pipe_peer;
mod pipe_server;
