//! Screenshot action reuses the existing retained session controller and UI lock.
use super::{output,worker,verified_target,ScreenshotReceipt};
use crate::engine::process_identity::OwnedProcess;
use std::io;

pub(super) fn save(slot:u32,expected:OwnedProcess)->io::Result<Option<ScreenshotReceipt>>{
    if !worker::available(){return Err(io::Error::new(io::ErrorKind::Unsupported,"Screenshot worker approval pending"));}
    // Expected UI identity is a comparison only; original controller supplies
    // the authoritative observed slot before and after the worker.
    let target=verified_target(slot,&expected)?;
    let bytes=worker::screenshot(&target)?;
    let size=output::inspect_webp(&bytes)?;
    verified_target(slot,&target)?;
    let Some(path)=rfd::FileDialog::new().set_title("Save Minecraft screenshot")
        .add_filter("WebP Lossless",&["webp"])
        .set_file_name(format!("Minecraft-{slot:02}.webp")).save_file() else {return Ok(None)};
    output::publish(&path,&bytes)?;
    Ok(Some(ScreenshotReceipt{slot,width:size.width,height:size.height,bytes:bytes.len()}))
}
