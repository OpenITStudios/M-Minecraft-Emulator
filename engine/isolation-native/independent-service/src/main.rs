//! Original MIVUBI Windows Service process entrypoint (source only).
//! No service installation, driver, pipe, Minecraft/account/profile access.
//! SCM RUNNING status indicates a live inert host, NEVER isolation readiness.

mod lifecycle;
#[cfg(windows)]
mod windows_host;

// Service-owned, source-only IPC pilot; disabled in default binaries.
// Never enable this feature in distribution before Windows security review.
#[cfg(all(windows, feature = "readonly-ipc-pilot"))]
mod ipc_worker;
#[cfg(all(windows, feature = "readonly-ipc-pilot"))]
mod service_identity;

#[cfg(windows)]
fn main() {
    windows_host::run_dispatcher();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("MIVUBI independent service requires Windows SCM.");
    std::process::exit(1);
}
