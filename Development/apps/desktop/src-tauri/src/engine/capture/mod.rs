//! On-demand product capture. Existing session controller remains the game owner.
use super::process_identity::OwnedProcess;
use serde::Serialize;
use std::io;

#[cfg(windows)]
mod screenshot;
#[cfg(windows)]
mod worker;
#[cfg(any(windows,test))]
mod output;
#[cfg(any(windows,test))]
mod video_encoder;
#[cfg(windows)]
mod recording_target;
#[cfg(windows)]
mod recording;
#[cfg(windows)]
pub use recording::RecordingReceipt;
#[cfg(not(windows))]
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct RecordingReceipt{pub slot:u32,pub state:&'static str,pub frames:u64}

#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct ScreenshotReceipt {
    pub slot:u32,
    pub width:u32,
    pub height:u32,
    pub bytes:usize,
}

pub fn screenshot_available()->bool {
    #[cfg(windows)]
    {worker::available()}
    #[cfg(not(windows))]
    {false}
}

pub fn save_screenshot(slot:u32,expected:OwnedProcess)->io::Result<Option<ScreenshotReceipt>> {
    #[cfg(windows)]
    {screenshot::save(slot,expected)}
    #[cfg(not(windows))]
    {let _=(slot,expected);Err(io::Error::new(io::ErrorKind::Unsupported,"Screenshot requires Windows"))}
}

pub fn recording_available()->bool{
    #[cfg(windows)] {video_encoder::available()}
    #[cfg(not(windows))] {false}
}
pub fn recording_status()->io::Result<Vec<RecordingReceipt>>{
    #[cfg(windows)] {recording::status()}
    #[cfg(not(windows))] {Ok(Vec::new())}
}
pub fn start_recording(slot:u32,expected:OwnedProcess)->io::Result<bool>{
    #[cfg(windows)] {recording::start(slot,expected)}
    #[cfg(not(windows))] {let _=(slot,expected);Err(io::Error::new(io::ErrorKind::Unsupported,"Recording requires Windows"))}
}
pub fn stop_recording(slot:u32)->io::Result<()>{
    #[cfg(windows)] {recording::stop(slot)}
    #[cfg(not(windows))] {let _=slot;Err(io::Error::new(io::ErrorKind::Unsupported,"Recording requires Windows"))}
}
pub fn require_recording_idle(slot:Option<u32>)->io::Result<()>{
    #[cfg(windows)] {recording::require_idle(slot)}
    #[cfg(not(windows))] {let _=slot;Ok(())}
}
