// Only the active native pilot belongs to the current desktop compile graph.
// Legacy VMware and screen-overlay sources remain in Git but are unmounted.
pub mod app_instance;
pub mod dpi;
pub mod instance_catalog;
pub mod instance_discovery;
pub mod instance_focus;
pub mod instance_focus_events;
pub mod instance_identity;
// Native Win32 job-owned fixture; never mounted as a game launch provider.
#[cfg(windows)]
mod independent_job;
// Original handle-scoped Win32 application-hive prototype; fixtures only.
#[cfg(windows)]
mod independent_private_hive;
// Handle-relative native filesystem I/O; only disposable fixture callers.
#[cfg(windows)]
mod independent_private_files;
// Read-only real Windows AccessCheck against pinned fixture process tokens.
#[cfg(windows)]
mod independent_root_access;
// Native pending pipe-peer PID/FILETIME check; no IPC server/launch wiring.
#[cfg(windows)]
mod independent_pipe_peer;
// Explicitly ACL-protected native named-pipe fixture; unmounted/read-only.
#[cfg(windows)]
mod independent_pipe_server;
pub mod manager_settings;
pub mod native_clients;
pub mod owned_isolation;
pub mod owned_provider;
pub mod virtual_inventory;
pub mod workspace_surface;
