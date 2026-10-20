#![cfg_attr(windows, windows_subsystem = "windows")]

mod app_bootstrap;
mod commands;
mod engine;

fn main() {
    if engine::workspace_surface::maybe_run_helper() {
        return;
    }
    if engine::owned_provider::maybe_run_profile_helper() {
        return;
    }
    app_bootstrap::run();
}
