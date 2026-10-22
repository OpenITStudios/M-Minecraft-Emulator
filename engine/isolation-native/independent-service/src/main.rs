//! Original MIVUBI Windows Service process entrypoint (source only).
//! No service installation, driver, pipe, Minecraft/account/profile access.
//! SCM RUNNING status indicates a live inert host, NEVER isolation readiness.

mod lifecycle;
#[cfg(windows)]
mod windows_host;

#[cfg(windows)]
fn main() {
    windows_host::run_dispatcher();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("MIVUBI independent service requires Windows SCM.");
    std::process::exit(1);
}
