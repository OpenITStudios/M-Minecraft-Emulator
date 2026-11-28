//! Single FFmpeg 8.1 recording path: HEVC NVENC and Matroska. No encoder selection or retry.
//! gfxcapture owns a WGC item for one HWND and emits D3D11 frames; NVENC
//! accepts D3D11 hardware frames. The Matroska muxer owns codec headers/cues.
use std::io;

pub(super) const FPS:u32=30;
pub(super) const MAX_SECONDS:u64=3600;
pub(super) const MAX_BYTES:u64=4*1024*1024*1024;
// One reviewed executable (self-contained, no adjacent third-party DLLs).
// Digest is intentionally unavailable until native + licensing review.
pub(super) const REVIEWED_FFMPEG_SHA256:Option<[u8;32]>=None;
pub(super) fn available()->bool{REVIEWED_FFMPEG_SHA256.is_some()}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub(super) struct VideoSize{pub width:u32,pub height:u32}
impl VideoSize{
    pub fn validate(self)->io::Result<Self>{
        if self.width<2||self.height<2||self.width>4096||self.height>4096||
            self.width%2!=0||self.height%2!=0{
            return Err(io::Error::other("Recording requires original even client dimensions within 4096 pixels"));
        }
        Ok(self)
    }
    pub fn pixels_per_second(self)->u64{u64::from(self.width)*u64::from(self.height)*u64::from(FPS)}
}

pub(super) fn args(hwnd:u64,size:VideoSize)->io::Result<Vec<String>>{
    size.validate()?;
    if hwnd==0{return Err(io::Error::other("Minecraft HWND unavailable"));}
    let graph=format!("gfxcapture=hwnd={hwnd}:capture_cursor=0:capture_border=0:max_framerate={FPS}:width={}:height={}:resize_mode=crop:output_fmt=bgra[v]",size.width,size.height);
    // VFR capped at 30, not synthesized CFR. Do not auto-insert software
    // conversions or silently select a software encoder. CQ 23/P4 is an
    // initial quality candidate, not a measured Minecraft benchmark.
    let mut a=vec!["-hide_banner","-loglevel","quiet","-nostats","-xerror",
        "-abort_on","empty_output+empty_output_stream","-stdin","-noauto_conversion_filters",
        "-init_hw_device","d3d11va=lt:,vendor_id=0x10de","-filter_hw_device","lt",
        "-filter_complex"].into_iter().map(str::to_owned).collect::<Vec<_>>();
    a.push(graph);
    a.extend(["-map","[v]","-an","-sn","-dn","-c:v","hevc_nvenc","-pix_fmt","+d3d11",
        "-profile:v","main","-rgb_mode","yuv420",
        "-preset","p4","-tune","hq","-rc","vbr","-cq","23","-b:v","0",
        "-bf","0","-rc-lookahead","0","-g","60","-fps_mode","vfr",
        "-max_muxing_queue_size","32","-progress","pipe:2","-stats_period","0.5",
        "-t"].into_iter().map(str::to_owned));
    a.push(MAX_SECONDS.to_string());
    a.push("-fs".into());a.push(MAX_BYTES.to_string());
    // fd protocol, unlike pipe, supports seeking when stdout is a regular
    // file. The manager passes its owned temporary file handle, never a path.
    a.extend(["-f","matroska","-fd","1","fd:"].into_iter().map(str::to_owned));
    Ok(a)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn fixed_gpu_only_graph_and_muxer(){
        let a=args(42,VideoSize{width:1920,height:1080}).unwrap();
        assert!(a.iter().any(|x|x.contains("hwnd=42:")&&x.contains("capture_border=0")&&x.contains("max_framerate=30")));
        assert_eq!(a.iter().filter(|arg|arg.as_str()=="-c:v").count(),1);
        assert!(a.windows(2).any(|p|p==["-c:v","hevc_nvenc"]));
        assert!(a.windows(2).any(|p|p==["-pix_fmt","+d3d11"]));
        assert!(a.windows(2).any(|p|p==["-profile:v","main"]));
        assert!(a.windows(2).any(|p|p==["-rgb_mode","yuv420"]));
        assert!(a.windows(2).any(|p|p==["-init_hw_device","d3d11va=lt:,vendor_id=0x10de"]));
        assert!(a.windows(2).any(|p|p==["-f","matroska"]));
        assert!(a.iter().any(|x|x=="-noauto_conversion_filters"));
        assert_eq!(a.last().unwrap(),"fd:");
        assert!(!a.iter().any(|x|x.contains("hwdownload")||x.contains("libaom")||x.contains("libx265")||x.contains("av1")||x.contains("cuda")||x.contains("h264")||x.contains("amf")||x.contains("qsv")));
        assert!(args(0,VideoSize{width:1920,height:1080}).is_err());
        assert!(args(42,VideoSize{width:1919,height:1080}).is_err());
    }
}
