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
