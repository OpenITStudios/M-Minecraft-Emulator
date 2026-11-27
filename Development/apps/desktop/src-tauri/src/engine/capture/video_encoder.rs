//! Fixed FFmpeg 8.1 hardware-only recording arguments. No shell or user input.
//! gfxcapture emits D3D11 frames for one HWND; NVENC/AMF consume D3D11,
//! QSV uses explicit GPU NV12 conversion and direct hardware mapping.
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

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub(super) enum Encoder { Av1Nvenc=1,H264Nvenc=2,H264Amf=3,H264Qsv=4 }
pub(super) const CANDIDATES:[Encoder;4]=[Encoder::Av1Nvenc,Encoder::H264Nvenc,Encoder::H264Amf,Encoder::H264Qsv];

impl Encoder{
    pub fn codec(self)->&'static str{
        if self==Self::Av1Nvenc{"AV1"}else{"H264"}
    }
    pub fn from_id(id:u8)->Option<Self>{CANDIDATES.into_iter().find(|x|*x as u8==id)}
    fn name(self)->&'static str{
        match self{Self::Av1Nvenc=>"av1_nvenc",Self::H264Nvenc=>"h264_nvenc",Self::H264Amf=>"h264_amf",Self::H264Qsv=>"h264_qsv"}
    }
    fn device(self)->&'static str{
        match self{
            Self::Av1Nvenc|Self::H264Nvenc=>"d3d11va=lt:,vendor_id=0x10de",
            Self::H264Amf=>"d3d11va=lt:,vendor_id=0x1002",
            Self::H264Qsv=>"d3d11va=lt:,vendor_id=0x8086",
        }
    }
}

pub(super) fn args(hwnd:u64,size:VideoSize,encoder:Encoder)->io::Result<Vec<String>>{
    size.validate()?;
    if hwnd==0{return Err(io::Error::other("Minecraft HWND unavailable"));}
    let mut graph=format!("gfxcapture=hwnd={hwnd}:capture_cursor=0:capture_border=0:max_framerate={FPS}:width={}:height={}:resize_mode=crop:output_fmt=bgra",size.width,size.height);
    // Only QSV needs a different surface type. Explicit D3D11 GPU conversion
    // and direct map refuse a system-memory fallback rather than copying
    // each captured frame through the CPU.
    if encoder==Encoder::H264Qsv{
        graph.push_str(",scale_d3d11=format=nv12,hwmap=derive_device=qsv:mode=read+direct,format=qsv");
    }
    graph.push_str("[v]");
    let mut a=vec!["-hide_banner","-loglevel","quiet","-nostats","-xerror",
        "-abort_on","empty_output+empty_output_stream","-stdin","-noauto_conversion_filters",
        "-init_hw_device",encoder.device(),"-filter_hw_device","lt",
        "-filter_complex"].into_iter().map(str::to_owned).collect::<Vec<_>>();
    a.push(graph);
    let pix_fmt=if encoder==Encoder::H264Qsv{"+qsv"}else{"+d3d11"};
    a.extend(["-map","[v]","-an","-sn","-dn","-c:v",encoder.name(),"-pix_fmt",pix_fmt].into_iter().map(str::to_owned));
    // Quality values are candidate defaults, not equivalent cross-codec
    // measurements. Keep the NVENC AV1 baseline and bound encoder queues.
    let quality:&[&str]=match encoder{
        Encoder::Av1Nvenc|Encoder::H264Nvenc=>&["-preset","p4","-tune","hq","-rc","vbr","-cq","23","-b:v","0","-rc-lookahead","0"],
        Encoder::H264Amf=>&["-quality","balanced","-rc","cqp","-qp_i","23","-qp_p","23","-preencode","0"],
        Encoder::H264Qsv=>&["-preset","medium","-global_quality","23","-look_ahead","0","-async_depth","1"],
    };
    a.extend(quality.iter().copied().map(str::to_owned));
    a.extend(["-bf","0","-g","60","-fps_mode","vfr",
        "-max_muxing_queue_size","32","-progress","pipe:2","-stats_period","0.5",
        "-t"].into_iter().map(str::to_owned));
    a.push(MAX_SECONDS.to_string());
    a.push("-fs".into());a.push(MAX_BYTES.to_string());
    // Seekable regular file descriptor, never an arbitrary output path.
    a.extend(["-f","matroska","-fd","1","fd:"].into_iter().map(str::to_owned));
    Ok(a)
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn fixed_gpu_only_graphs_and_muxer(){
        let size=VideoSize{width:1920,height:1080};
        assert_eq!(CANDIDATES.map(Encoder::name),["av1_nvenc","h264_nvenc","h264_amf","h264_qsv"]);
        for encoder in CANDIDATES{
            let a=args(42,size,encoder).unwrap();
            assert!(a.iter().any(|x|x.contains("hwnd=42:")&&x.contains("capture_border=0")&&x.contains("max_framerate=30")));
            assert!(a.windows(2).any(|p|p==["-c:v",encoder.name()]));
            let pix=if encoder==Encoder::H264Qsv{"+qsv"}else{"+d3d11"};
            assert!(a.windows(2).any(|p|p==["-pix_fmt",pix]));
            assert_eq!(a.iter().any(|x|x.contains("scale_d3d11=format=nv12,hwmap=derive_device=qsv:mode=read+direct,format=qsv")),encoder==Encoder::H264Qsv);
            assert!(a.iter().any(|x|x=="-noauto_conversion_filters"));
            assert_eq!(a.last().unwrap(),"fd:");
            assert!(!a.iter().any(|x|x.contains("hwdownload")||x.contains("libaom")||x.contains("libx264")));
            assert_eq!(Encoder::from_id(encoder as u8),Some(encoder));
        }
        assert_eq!(Encoder::from_id(0),None);
        assert!(args(0,size,Encoder::Av1Nvenc).is_err());
        assert!(args(42,VideoSize{width:1919,height:1080},Encoder::Av1Nvenc).is_err());
    }
}
