//! Optional Windows Graphics Capture producer for one controller-owned Minecraft.
//! Source only: not built, not installed, not executed. No game inputs.
//! stdout is *only* bounded binary PNG frames (LTG1), stderr is for errors.
//! Existing Lazy Tester MCP remains the only AI-facing tool owner.
#![cfg_attr(not(windows), allow(unused))]

#[cfg(not(windows))]
fn main() {
    eprintln!("WINDOWS_ONLY_GPU_CAPTURE");
    std::process::exit(2);
}

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
        Foundation::{CloseHandle, FILETIME, HANDLE},
        System::{
            RemoteDesktop::ProcessIdToSessionId,
            Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
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

    fn read_args() -> io::Result<Target> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        if args.len() != 12 ||
            args[0] != "--pid" || args[2] != "--window-id" ||
            args[4] != "--birth" || args[6] != "--session" ||
            args[8] != "--duration-ms" || args[10] != "--fps" {
            return Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Expected six bounded exact-target flags"));
        }
        let parse = |index: usize| args[index].parse::<u64>().ok();
        let pid = parse(1).and_then(|n| u32::try_from(n).ok()).filter(|n| *n != 0);
        let window_id = parse(3).filter(|n| *n > 0 && *n <= 0x001f_ffff_ffff_ffff);
        let birth = parse(5).filter(|n| *n != 0);
        let session_id = parse(7).and_then(|n| u32::try_from(n).ok());
        let duration_ms = parse(9).filter(|n| (5000..=90000).contains(n));
        let fps = parse(11).and_then(|n| u32::try_from(n).ok())
            .filter(|n| (1..=10).contains(n));
        match (pid, window_id, birth, session_id, duration_ms, fps) {
            (Some(pid), Some(window_id), Some(birth), Some(session_id),
             Some(duration_ms), Some(fps)) => Ok(Target {
                pid, window_id, birth, session_id, duration_ms, fps,
            }),
            _ => Err(io::Error::new(io::ErrorKind::InvalidInput,
                "Invalid capture target or limits")),
        }
    }

    // A HANDLE pins the process object through the capture attempt. A bare
    // HWND/PID from an external MCP request is NEVER authorization by itself.
    fn pin_target(target: &Target) -> io::Result<ProcessPin> {
        let handle=unsafe {OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,0,target.pid)};
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

    fn write_frame(out: &mut impl Write,sequence: u32,pixels: &[u8]) -> io::Result<()> {
        if pixels.len()!=FRAME_WIDTH as usize*FRAME_HEIGHT as usize*4 {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "Unexpected GPU capture pixel format/size"));
        }
        let mut png=Vec::new();
        PngEncoder::new(&mut png)
            .write_image(pixels,FRAME_WIDTH,FRAME_HEIGHT,ExtendedColorType::Rgba8)
            .map_err(|_|io::Error::other("PNG encoding unavailable"))?;
        if png.len()>MAX_PNG_BYTES {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                "Captured frame exceeds memory/IPC budget"));
        }
        // LTG1 header: magic(4), png bytes(4), width(4), height(4),
        // sequence(4), Unix capture time in ms(8). Big-endian numbers.
        let mut header=Vec::with_capacity(HEADER_BYTES);
        header.extend_from_slice(HEADER);
        header.extend_from_slice(&(png.len() as u32).to_be_bytes());
        header.extend_from_slice(&FRAME_WIDTH.to_be_bytes());
        header.extend_from_slice(&FRAME_HEIGHT.to_be_bytes());
        header.extend_from_slice(&sequence.to_be_bytes());
        header.extend_from_slice(&frame_ms()?.to_be_bytes());
        out.write_all(&header)?;
        out.write_all(&png)?;
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
            verify_target(&target,&pin)?; // do not publish after window reuse
            sequence=sequence.checked_add(1).ok_or_else(||
                io::Error::other("Capture sequence exhausted"))?;
            write_frame(&mut out,sequence,&fitted)?;
            last_emit=Some(Instant::now());
        }
        // WGC's blocking iterator may not yield when a window stops rendering.
        // The MCP parent MUST enforce a separate wall-clock kill timeout.
        Ok(())
    }

    pub fn run() -> io::Result<()> {
        capture(read_args()?)
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
