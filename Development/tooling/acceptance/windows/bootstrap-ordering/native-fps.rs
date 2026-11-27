//! Shared dependency-free native FPS static library.
#[path = "fps-meter.rs"]
mod fps_meter;
#[path = "frame-pacing.rs"]
mod frame_pacing;

#[path = "hud_canvas.rs"]
mod hud_canvas;

#[cfg(windows)]
#[path = "hud_window.rs"]
mod hud_window;

#[cfg(windows)]
mod fps_hud;
