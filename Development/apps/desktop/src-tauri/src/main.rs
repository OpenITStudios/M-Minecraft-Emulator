#![cfg_attr(windows, windows_subsystem = "windows")]

mod app_bootstrap;
mod commands;
mod engine;

fn main() {
    if engine::native_clients::maybe_run_education_startup_observation() {
        return;
    }
    if engine::workspace_surface::maybe_run_helper() {
        return;
    }
    app_bootstrap::run();
}
