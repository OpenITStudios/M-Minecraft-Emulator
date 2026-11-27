//! On-demand product capture. Existing session controller remains the game owner.
use super::process_identity::OwnedProcess;
use serde::Serialize;
use std::io;
#[cfg(windows)]
use super::native_clients_session::{request,SessionRequest};

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

#[cfg(any(windows,test))]
fn same_target(a:&OwnedProcess,b:&OwnedProcess)->bool {
    a.responding&&b.responding&&a.pid==b.pid&&a.session_id==b.session_id&&
        a.created_file_time==b.created_file_time&&a.window_handle.eq_ignore_ascii_case(&b.window_handle)
}

#[cfg(windows)]
fn verified_target(slot:u32,expected:&OwnedProcess)->io::Result<OwnedProcess>{
    if !(1..=5).contains(&slot){return Err(io::Error::other("Unknown Minecraft profile"));}
    let target=request(SessionRequest::Inspect)?.profiles.into_iter()
        .find(|p|p.slot==slot&&p.state=="RUNNING").and_then(|p|p.process)
        .filter(|p|same_target(p,expected))
        .ok_or_else(||io::Error::other("Minecraft target changed or is not responding"))?;
    Ok(target)
}

#[cfg(test)]
mod target_tests{
    use super::*;
    #[test]
    fn replacement_or_unresponsive_target_cannot_adopt_screenshot(){
        let a=OwnedProcess{pid:5,created_file_time:"1".into(),session_id:1,window_handle:"0x42".into(),responding:true};
        assert!(same_target(&a,&a));
        let mut b=a.clone();b.created_file_time="2".into();assert!(!same_target(&a,&b));
        let mut b=a.clone();b.window_handle="0x43".into();assert!(!same_target(&a,&b));
        let mut b=a.clone();b.responding=false;assert!(!same_target(&a,&b));
    }
}
