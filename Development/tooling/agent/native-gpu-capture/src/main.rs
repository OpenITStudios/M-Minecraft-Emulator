//! Optional Windows Graphics Capture producer for one controller-owned Minecraft.
//! Source only: not built, not installed, not executed. No game inputs.
//! stdout is LTG1 PNG for observation, or one raw WebP for explicit screenshot mode.
//! stderr is for errors; no file paths, input or game lifecycle commands.
//! Existing Lazy Tester MCP remains the only AI-facing tool owner.
#![cfg_attr(not(windows), allow(unused))]

#[cfg(not(windows))]
fn main() {
    eprintln!("WINDOWS_ONLY_GPU_CAPTURE");
    std::process::exit(2);
}

#[cfg(windows)]
mod screenshot;

#[cfg(windows)]
mod windows_capture {
    use image::{codecs::png::PngEncoder, ExtendedColorType, ImageEncoder};
    use std::{
        ffi::c_void,
        io::{self, Write},
        time::{Duration, Instant, SystemTime, UNIX_EPOCH},
    };
    use wgc::{FrameSize, PixelFormat, Wgc};
    use windows::Win32::Foundation::HWND;
    use windows_sys::Win32::{
        Foundation::{CloseHandle, FILETIME, HANDLE, WAIT_TIMEOUT},
        System::{
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{GetProcessTimes, OpenProcess, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION, SYNCHRONIZATION_SYNCHRONIZE},
        },
        UI::WindowsAndMessaging::{
            GetAncestor, GetParent, GetWindowThreadProcessId, IsHungAppWindow, IsIconic,
            IsWindow, IsWindowVisible, GA_ROOT,
        },
    };

    const FRAME_WIDTH: u32 = 960;
    const FRAME_HEIGHT: u32 = 540;
    const MAX_PNG_BYTES: usize = 3 * 1024 * 1024;
    const HEADER: &[u8; 4] = b"LTG1";
    const HEADER_BYTES: usize = 28;

    struct ProcessPin(HANDLE);
    impl Drop for ProcessPin {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    struct Target {
        pid: u32,
        window_id: u64,
        birth: u64,
        session_id: u32,
        duration_ms: u64,
        fps: u32,
    }

    enum Mode { Observe(Target), Screenshot(Target) }

    fn read_args() -> io::Result<Mode> {
        parse_args(&std::env::args().skip(1).collect::<Vec<_>>())
    }

    fn parse_args(args: &[String]) -> io::Result<Mode> {
        let screenshot = args.first().is_some_and(|arg| arg == "--screenshot");
        let args = if screenshot { &args[1..] } else { args };
        if args.len() != (if screenshot { 8 } else { 12 }) ||
            args[0] != "--pid" || args[2] != "--window-id" ||
            args[4] != "--birth" || args[6] != "--session" ||
            (!screenshot && (args[8] != "--duration-ms" || args[10] != "--fps")) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Expected bounded exact-target capture flags"));
        }
        let parse = |index: usize| args[index].parse::<u64>().ok();
        let pid = parse(1).and_then(|n| u32::try_from(n).ok()).filter(|n| *n != 0);
        let window_id = parse(3).filter(|n| *n > 0 && *n <= 0x001f_ffff_ffff_ffff);
        let birth = parse(5).filter(|n| *n != 0);
        let session_id = parse(7).and_then(|n| u32::try_from(n).ok());
        let duration_ms = if screenshot { Some(5000) } else { parse(9).filter(|n| (5000..=90000).contains(n)) };
        let fps = if screenshot { Some(1) } else { parse(11).and_then(|n| u32::try_from(n).ok())
            .filter(|n| (1..=10).contains(n)) };
        match (pid, window_id, birth, session_id, duration_ms, fps) {
            (Some(pid), Some(window_id), Some(birth), Some(session_id),
             Some(duration_ms), Some(fps)) => {
                let target=Target { pid, window_id, birth, session_id, duration_ms, fps };
                Ok(if screenshot { Mode::Screenshot(target) } else { Mode::Observe(target) })
            },
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid capture target or limits")),
        }
    }

    // A HANDLE pins the process object through the capture attempt. A bare
    // HWND/PID from an external MCP request is NEVER authorization by itself.
    fn pin_target(target: &Target) -> io::Result<ProcessPin> {
        let handle=unsafe {OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | SYNCHRONIZATION_SYNCHRONIZE,0,target.pid)};
        if handle.is_null() {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "Target process cannot be verified"));
        }
        let pin=ProcessPin(handle);
        verify_target(target,&pin)?;
        Ok(pin)
    }

    fn verify_target(target: &Target, pin: &ProcessPin) -> io::Result<()> {
        let window=target.window_id as usize as *mut c_void;
        let mut observed_pid=0u32;
        let mut game_session=0u32;
        let mut own_session=0u32;
        let mut created=FILETIME{dwLowDateTime:0,dwHighDateTime:0};
        let mut exited=created;
        let mut kernel=created;
        let mut user=created;
        let ok=unsafe {
            WaitForSingleObject(pin.0,0)==WAIT_TIMEOUT &&
            GetWindowThreadProcessId(window,&mut observed_pid)!=0 &&
            GetProcessTimes(pin.0,&mut created,&mut exited,&mut kernel,&mut user)!=0 &&
            ProcessIdToSessionId(target.pid,&mut game_session)!=0 &&
            ProcessIdToSessionId(std::process::id(),&mut own_session)!=0
        };
        let birth=((created.dwHighDateTime as u64)<<32)|created.dwLowDateTime as u64;
        let window_valid=unsafe {
            IsWindow(window)!=0 &&
            GetParent(window).is_null() &&
            GetAncestor(window,GA_ROOT)==window &&
            IsWindowVisible(window)!=0 &&
            IsIconic(window)==0 &&
            IsHungAppWindow(window)==0
        };
        if !ok || observed_pid!=target.pid || birth!=target.birth ||
            game_session!=target.session_id || own_session!=target.session_id ||
            !window_valid {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied,
                "Game PID/birth/session/root HWND or visibility changed"));
        }
        Ok(())
    }

    fn frame_ms() -> io::Result<u64> {
        let elapsed=SystemTime::now().duration_since(UNIX_EPOCH)
            .map_err(|_|io::Error::other("Clock unavailable"))?;
        Ok(elapsed.as_millis() as u64)
    }

    fn write_frame(out: &mut impl Write,png: &mut Vec<u8>,
                   sequence: u32,pixels: &[u8],readback_at_ms:u64) -> io::Result<()> {
        if pixels.len()!=FRAME_WIDTH as usize*FRAME_HEIGHT as usize*4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "Unexpected GPU capture pixel format/size"));
        }
        // This worker retains one scratch output across all emitted frames.
        // Clear its length without dropping the allocated PNG capacity.
        png.clear();
        PngEncoder::new(&mut *png)
            .write_image(pixels,FRAME_WIDTH,FRAME_HEIGHT,ExtendedColorType::Rgba8)
            .map_err(|_|io::Error::other("PNG encoding unavailable"))?;
        if png.len()>MAX_PNG_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "Captured frame exceeds memory/IPC budget"));
        }
        // LTG1 header: magic(4), png bytes(4), width(4), height(4),
        // sequence(4), Unix *readback completion* ms(8). Big-endian.
        // This time is sampled BEFORE PNG encoding and stdout delay,
        // not after. The WGC frame may still represent older pixels;
        // it is not a proof of presentation or Minecraft gameplay.
        // 28 fixed bytes: no heap allocation for the frame header.
        let mut header=[0u8;HEADER_BYTES];
        header[0..4].copy_from_slice(HEADER);
        header[4..8].copy_from_slice(&(png.len() as u32).to_be_bytes());
        header[8..12].copy_from_slice(&FRAME_WIDTH.to_be_bytes());
        header[12..16].copy_from_slice(&FRAME_HEIGHT.to_be_bytes());
        header[16..20].copy_from_slice(&sequence.to_be_bytes());
        header[20..28].copy_from_slice(&readback_at_ms.to_be_bytes());
        out.write_all(&header)?;
        out.write_all(png.as_slice())?;
        out.flush()?;
        Ok(())
    }

    fn capture(target: Target) -> io::Result<()> {
        let pin=pin_target(&target)?;
        let window=HWND(target.window_id as usize as *mut c_void);
        // Persistent WGC frame pool: captures a single exact window, NEVER
        // a monitor/desktop or a picker-selected window.
        let session=Wgc::new_from_hwnd(window)
            .map_err(|_|io::Error::other("Windows Graphics Capture HWND unavailable"))?
            .pixel_format(PixelFormat::RGBA8)
            .frame_queue_length(2)
            .build()
            .map_err(|_|io::Error::other("Windows GPU capture session refused"))?;
        let deadline=Instant::now()+Duration::from_millis(target.duration_ms);
        let emit_gap=Duration::from_millis((1000/target.fps) as u64);
        let mut last_emit:Option<Instant>=None;
        let mut sequence=0u32;
        let mut png=Vec::new(); // Reuse one bounded output buffer per worker.
        let out=io::stdout();
        let mut out=out.lock();
        for candidate in session {
            if Instant::now()>=deadline { break; }
            let frame=candidate.map_err(|_|io::Error::other("WGC frame unavailable"))?;
            if last_emit.is_some_and(|time|time.elapsed()<emit_gap) {continue;}
            verify_target(&target,&pin)?; // fail closed before every frame
            let fitted=frame.pixels_fitted(FrameSize {
                width:FRAME_WIDTH,height:FRAME_HEIGHT
            }).map_err(|_|io::Error::other("GPU pixel readback unavailable"))?;
            // Timestamp IMMEDIATELY upon GPU readback completion. Capturing
            // after the subsequent window verification could incorrectly
            // date older pixels as post-action during a slow native check.
            let readback_at_ms=frame_ms()?;
            verify_target(&target,&pin)?; // do not publish after window reuse
            sequence=sequence.checked_add(1).ok_or_else(||
                io::Error::other("Capture sequence exhausted"))?;
            write_frame(&mut out,&mut png,sequence,&fitted,readback_at_ms)?;
            last_emit=Some(Instant::now());
        }
        // WGC's blocking iterator may not yield when a window stops rendering.
        // The MCP parent MUST enforce a separate wall-clock kill timeout.
        Ok(())
    }

    fn capture_screenshot(target: Target) -> io::Result<()> {
        // The parent must supervise this one-shot process: WGC can block
        // without delivering a frame. No user-provided output path is accepted.
        let _dpi=super::screenshot::DpiContext::enter()?;
        let pin=pin_target(&target)?;
        let hwnd=target.window_id as usize as *mut c_void;
        let geometry=super::screenshot::geometry(hwnd)?;
        let mut session=Wgc::new_from_hwnd(HWND(hwnd))
            .map_err(|_|io::Error::other("Windows Graphics Capture HWND unavailable"))?
            .pixel_format(PixelFormat::RGBA8)
            .frame_queue_length(1)
            .capture_cursor(false)
            .build()
            .map_err(|_|io::Error::other("Windows GPU capture session refused"))?;
        let deadline=Instant::now()+Duration::from_millis(target.duration_ms);
        let frame=session.next()
            .ok_or_else(||io::Error::other("WGC screenshot frame unavailable"))?
            .map_err(|_|io::Error::other("WGC screenshot frame unavailable"))?;
        if Instant::now()>=deadline {return Err(io::Error::new(io::ErrorKind::TimedOut,"Screenshot deadline elapsed"));}
        verify_target(&target,&pin)?;
        let size=frame.size().map_err(|_|io::Error::other("Screenshot frame size unavailable"))?;
        let crop=geometry.crop(size.width,size.height)?;
        let pixels=frame.pixels().map_err(|_|io::Error::other("Screenshot readback unavailable"))?;
        drop(frame);
        drop(session); // release GPU capture resources before CPU encoding
        verify_target(&target,&pin)?;
        if super::screenshot::geometry(hwnd)?!=geometry {
            return Err(io::Error::other("Screenshot window geometry changed"));
        }
        let webp=super::screenshot::encode_client(size.width,size.height,pixels,crop)?;
        verify_target(&target,&pin)?;
        if super::screenshot::geometry(hwnd)?!=geometry || Instant::now()>=deadline {
            return Err(io::Error::other("Screenshot target changed or deadline elapsed"));
        }
        let mut output=io::stdout().lock();
        output.write_all(&webp)?;
        output.flush()
    }

    pub fn run() -> io::Result<()> {
        match read_args()? {
            Mode::Observe(target) => capture(target),
            Mode::Screenshot(target) => capture_screenshot(target),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn screenshot_and_observation_have_distinct_strict_arguments() {
            let args=|s:&str|s.split_whitespace().map(str::to_owned).collect::<Vec<_>>();
            let target="--pid 42 --window-id 123 --birth 134000000000000001 --session 1";
            assert!(matches!(parse_args(&args(&format!("--screenshot {target}"))),Ok(Mode::Screenshot(_))));
            assert!(matches!(parse_args(&args(&format!("{target} --duration-ms 5000 --fps 10"))),Ok(Mode::Observe(_))));
            assert!(parse_args(&args(&format!("--screenshot {target} --duration-ms 5000 --fps 1"))).is_err());
            assert!(parse_args(&args(target)).is_err());
            assert!(parse_args(&args("--screenshot")).is_err());
        }

        #[test]
        fn two_frames_reuse_png_scratch_and_preserve_lt_g1_wire_bytes() {
            let pixels=vec![0u8;(FRAME_WIDTH*FRAME_HEIGHT*4) as usize];
            let mut png=Vec::new();
            let mut first=Vec::new();
            write_frame(&mut first,&mut png,1,&pixels,12345).unwrap();
            let first_len=u32::from_be_bytes(first[4..8].try_into().unwrap()) as usize;
            assert_eq!(&first[0..4],&HEADER[..]);
            assert_eq!(first.len(),HEADER_BYTES+first_len);
            assert_eq!(&first[8..12],&FRAME_WIDTH.to_be_bytes());
            assert_eq!(&first[12..16],&FRAME_HEIGHT.to_be_bytes());
            assert_eq!(&first[16..20],&1u32.to_be_bytes());
            assert_eq!(&first[20..28],&12345u64.to_be_bytes());
            let first_capacity=png.capacity();

            let mut second=Vec::new();
            write_frame(&mut second,&mut png,2,&pixels,12346).unwrap();
            assert_eq!(png.capacity(),first_capacity);
            assert_eq!(&second[0..4],&HEADER[..]);
            assert_eq!(&second[16..20],&2u32.to_be_bytes());
            assert_eq!(&second[20..28],&12346u64.to_be_bytes());
            assert_eq!(&first[HEADER_BYTES..],&second[HEADER_BYTES..]);
        }
    }
}

#[cfg(windows)]
fn main() {
    if let Err(error)=windows_capture::run() {
        // Never print process IDs, profile paths, session values or frames.
        eprintln!("GPU_CAPTURE_UNAVAILABLE: {}",error.kind());
        std::process::exit(2);
    }
}
