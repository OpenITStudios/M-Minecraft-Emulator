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
// Optional same-executable read/launch adapter for local MCP; no second game owner.
pub(crate) mod agent_cli;
// Windows-only MC02 input feasibility experiment; not mounted as gameplay input.
#[cfg(windows)]
pub(crate) mod background_input_probe;
pub mod process_identity;
pub mod dpi;
pub mod instance_focus_events;
pub mod instance_identity;
// Disposable native isolation research, excluded from all non-test builds.
#[cfg(all(windows, test))]
mod native_isolation_fixtures;
pub mod manager_settings;
pub mod coordinate_bookmarks;
pub mod native_clients;
pub mod workspace_surface;
