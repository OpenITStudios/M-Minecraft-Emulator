//! Shared native FPS and presentation policy library.
mod fps_meter;
mod frame_pacing;
mod hud_canvas;
#[cfg(windows)]
mod hud_window;
#[cfg(windows)]
mod fps_hud;
mod presentation_policy;

#[cfg(windows)]
mod presentation_image;

#[cfg(windows)]
mod presentation_transport;
