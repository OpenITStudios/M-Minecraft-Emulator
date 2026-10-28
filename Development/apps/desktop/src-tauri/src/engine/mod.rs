// Only the active native pilot belongs to the current desktop compile graph.
// Retired VMware/layout/overlay modules were removed after confirming they
// were absent from this compile graph; native session/window owners remain.
// The retained single-instance lease relies on Windows-owned app-data and
// exclusive file semantics. Non-Windows has no Minecraft lifecycle to lock.
#[cfg(windows)]
pub mod app_instance;
#[cfg(windows)]
pub(crate) mod process_roles;
// One bounded, read-only Win32 image-path query shared by process identity owners.
#[cfg(windows)]
pub(crate) mod windows_process_image;
pub mod native_clients_session;
pub mod process_identity;
pub mod dpi;
pub mod instance_focus_events;
pub mod instance_identity;
// Fixture-only Windows primitives are compiled exclusively for Rust unit tests.
// No mounted desktop owner imports them; exclude all six from release/check builds.
// Native Win32 job-owned fixture; never mounted as a game launch provider.
#[cfg(all(windows, test))]
mod independent_job;
// Original handle-scoped Win32 application-hive prototype; fixtures only.
#[cfg(all(windows, test))]
mod independent_private_hive;
// Handle-relative native filesystem I/O; only disposable fixture callers.
#[cfg(all(windows, test))]
mod independent_private_files;
// Read-only real Windows AccessCheck against pinned fixture process tokens.
#[cfg(all(windows, test))]
mod independent_root_access;
// Native pending pipe-peer PID/FILETIME check; no IPC server/launch wiring.
#[cfg(all(windows, test))]
mod independent_pipe_peer;
// Explicitly ACL-protected native named-pipe fixture; unmounted/read-only.
#[cfg(all(windows, test))]
mod independent_pipe_server;
pub mod manager_settings;
pub mod native_clients;
pub mod workspace_surface;
