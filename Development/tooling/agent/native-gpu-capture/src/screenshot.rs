//! One native-resolution client-area screenshot, no PNG intermediary.
//! WGC supplies premultiplied RGBA8; opaque client pixels preserve RGB exactly.
use image_webp::{ColorType, WebPEncoder};
use std::{io, mem::size_of};
use windows_sys::Win32::{
    Foundation::{HWND, POINT, RECT},
    Graphics::{Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS}, Gdi::ClientToScreen},
    UI::{HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        WindowsAndMessaging::GetClientRect},
};

const MAX_SIDE:u32=4096;
const MAX_RGBA_BYTES:usize=64*1024*1024;
const MAX_WEBP_BYTES:usize=80*1024*1024;

pub(super) struct DpiContext(DPI_AWARENESS_CONTEXT);
impl DpiContext {
    pub(super) fn enter()->io::Result<Self> {
        let previous=unsafe{SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)};
        if previous.is_null(){return Err(io::Error::other("Physical screenshot geometry unavailable"));}
        Ok(Self(previous))
    }
}
impl Drop for DpiContext {
    fn drop(&mut self){unsafe{SetThreadDpiAwarenessContext(self.0);}}
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Bounds { left:i32,top:i32,right:i32,bottom:i32 }
impl From<RECT> for Bounds {
    fn from(r:RECT)->Self{Self{left:r.left,top:r.top,right:r.right,bottom:r.bottom}}
}
impl Bounds {
    fn dimensions(self)->io::Result<(u32,u32)> {
        let width=u32::try_from(i64::from(self.right)-i64::from(self.left)).map_err(|_|invalid())?;
        let height=u32::try_from(i64::from(self.bottom)-i64::from(self.top)).map_err(|_|invalid())?;
        rgba_len(width,height)?;
        Ok((width,height))
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) struct Geometry { frame:Bounds,client:Bounds }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) struct Crop { x:u32,y:u32,width:u32,height:u32 }

fn invalid()->io::Error {io::Error::new(io::ErrorKind::InvalidData,"Screenshot dimensions or client geometry invalid")}
fn rgba_len(width:u32,height:u32)->io::Result<usize>{
    if width==0||height==0||width>MAX_SIDE||height>MAX_SIDE{return Err(invalid());}
    let len=(width as usize).checked_mul(height as usize).and_then(|n|n.checked_mul(4)).ok_or_else(invalid)?;
    if len>MAX_RGBA_BYTES{return Err(invalid());}
    Ok(len)
}

pub(super) fn geometry(hwnd:HWND)->io::Result<Geometry>{
    let mut frame=RECT{left:0,top:0,right:0,bottom:0};
    let mut client=frame;
    let hr=unsafe{DwmGetWindowAttribute(hwnd,DWMWA_EXTENDED_FRAME_BOUNDS as u32,
        (&mut frame as *mut RECT).cast(),size_of::<RECT>() as u32)};
    if hr<0||unsafe{GetClientRect(hwnd,&mut client)}==0{return Err(invalid());}
    let mut tl=POINT{x:client.left,y:client.top};
    let mut br=POINT{x:client.right,y:client.bottom};
    if unsafe{ClientToScreen(hwnd,&mut tl)}==0||unsafe{ClientToScreen(hwnd,&mut br)}==0{return Err(invalid());}
    let result=Geometry{frame:frame.into(),client:Bounds{left:tl.x,top:tl.y,right:br.x,bottom:br.y}};
    result.frame.dimensions()?;
    result.client.dimensions()?;
    Ok(result)
}
impl Geometry {
    pub(super) fn crop(self,width:u32,height:u32)->io::Result<Crop>{
        // A WGC frame must match the physical DWM bounds exactly; never
        // guess a crop or substitute the desktop on disagreement.
        if self.frame.dimensions()? != (width,height) {return Err(invalid());}
        let (w,h)=self.client.dimensions()?;
        let x=u32::try_from(i64::from(self.client.left)-i64::from(self.frame.left)).map_err(|_|invalid())?;
        let y=u32::try_from(i64::from(self.client.top)-i64::from(self.frame.top)).map_err(|_|invalid())?;
        let crop=Crop{x,y,width:w,height:h};
        crop.validate(width,height)?;
        Ok(crop)
    }
}
impl Crop {
    fn validate(self,width:u32,height:u32)->io::Result<()> {
        rgba_len(width,height)?;
        rgba_len(self.width,self.height)?;
        if self.x.checked_add(self.width).is_none_or(|n|n>width)||
           self.y.checked_add(self.height).is_none_or(|n|n>height){return Err(invalid());}
        Ok(())
    }
}
pub(super) fn encode_client(width:u32,height:u32,mut rgba:Vec<u8>,crop:Crop)->io::Result<Vec<u8>>{
    crop.validate(width,height)?;
    if rgba.len()!=rgba_len(width,height)?{return Err(invalid());}
    let stride=width as usize*4;
    let row=crop.width as usize*4;
    // Compact in place; no second full-sized raw image or resampling.
    for y in 0..crop.height as usize {
        let start=(crop.y as usize+y)*stride+crop.x as usize*4;
        rgba.copy_within(start..start+row,y*row);
    }
    rgba.truncate(rgba_len(crop.width,crop.height)?);
    // Premultiplied translucent input is not straight RGBA. Minecraft's
    // opaque client is required; fail closed rather than alter pixel colors.
    if rgba.chunks_exact(4).any(|pixel|pixel[3]!=255){
        return Err(io::Error::new(io::ErrorKind::InvalidData,"Screenshot client is not opaque"));
    }
    let mut encoded=Vec::new();
    WebPEncoder::new(&mut encoded).encode(&rgba,crop.width,crop.height,ColorType::Rgba8)
        .map_err(|_|io::Error::other("WebP lossless encode failed"))?;
    if encoded.len()>MAX_WEBP_BYTES{return Err(io::Error::other("Screenshot output budget exceeded"));}
    Ok(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image_webp::WebPDecoder;
    use std::io::Cursor;

    #[test]
    fn crop_preserves_pixels_and_original_client_dimensions(){
        let g=Geometry{frame:Bounds{left:-10,top:30,right:-6,bottom:33},
            client:Bounds{left:-9,top:31,right:-7,bottom:33}};
        let crop=g.crop(4,3).unwrap();
        assert_eq!(crop,Crop{x:1,y:1,width:2,height:2});
        let pixels=(0..12u8).flat_map(|v|[v,v+1,v+2,255]).collect::<Vec<_>>();
        let webp=encode_client(4,3,pixels,crop).unwrap();
        assert_eq!(&webp[..4],b"RIFF");
        assert_eq!(&webp[8..16],b"WEBPVP8L");
        let mut decoder=WebPDecoder::new(Cursor::new(webp)).unwrap();
        assert_eq!(decoder.dimensions(),(2,2));
        let mut decoded=vec![0;decoder.output_buffer_size().unwrap()];
        decoder.read_image(&mut decoded).unwrap();
        // The encoder may omit an all-opaque alpha channel.
        let rgb=[5,6,7,6,7,8,9,10,11,10,11,12];
        if decoded.len()==12 {assert_eq!(decoded,rgb);}
        else {assert_eq!(decoded,[5,6,7,255,6,7,8,255,9,10,11,255,10,11,12,255]);}
    }

    #[test]
    fn rejects_mismatched_frame_out_of_bounds_and_nonopaque_data(){
        let g=Geometry{frame:Bounds{left:0,top:0,right:4,bottom:3},
            client:Bounds{left:1,top:1,right:5,bottom:3}};
        assert!(g.crop(4,3).is_err());
        assert!(g.crop(5,3).is_err());
        let crop=Crop{x:0,y:0,width:1,height:1};
        assert!(encode_client(1,1,vec![1,2,3],crop).is_err());
        assert!(encode_client(1,1,vec![1,2,3,128],crop).is_err());
        assert!(rgba_len(4097,1).is_err());
        assert!(rgba_len(0,1).is_err());
    }
}
