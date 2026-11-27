//! Screenshot action reuses the existing retained session controller and UI lock.
use super::{output,worker,ScreenshotReceipt};
use crate::engine::{native_clients_session::{request,SessionRequest},process_identity::OwnedProcess};
use std::io;

fn same_target(a:&OwnedProcess,b:&OwnedProcess)->bool {
    a.responding&&b.responding&&a.pid==b.pid&&a.session_id==b.session_id&&
        a.created_file_time==b.created_file_time&&a.window_handle.eq_ignore_ascii_case(&b.window_handle)
}
fn verified(slot:u32,expected:&OwnedProcess)->io::Result<OwnedProcess>{
    if !(1..=5).contains(&slot){return Err(io::Error::other("Unknown Minecraft profile"));}
    let target=request(SessionRequest::Inspect)?.profiles.into_iter()
        .find(|p|p.slot==slot&&p.state=="RUNNING").and_then(|p|p.process)
        .filter(|p|same_target(p,expected))
        .ok_or_else(||io::Error::other("Minecraft target changed or is not responding"))?;
    Ok(target)
}
pub(super) fn save(slot:u32,expected:OwnedProcess)->io::Result<Option<ScreenshotReceipt>>{
    if !worker::available(){return Err(io::Error::new(io::ErrorKind::Unsupported,"Screenshot worker approval pending"));}
    // Expected UI identity is a comparison only; original controller supplies
    // the authoritative observed slot before and after the worker.
    let target=verified(slot,&expected)?;
    let bytes=worker::screenshot(&target)?;
    let size=output::inspect_webp(&bytes)?;
    verified(slot,&target)?;
    let Some(path)=rfd::FileDialog::new().set_title("Save Minecraft screenshot")
        .add_filter("WebP Lossless",&["webp"])
        .set_file_name(format!("Minecraft-{slot:02}.webp")).save_file() else {return Ok(None)};
    output::publish(&path,&bytes)?;
    Ok(Some(ScreenshotReceipt{slot,width:size.width,height:size.height,bytes:bytes.len()}))
}
#[cfg(test)]
mod tests{
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
