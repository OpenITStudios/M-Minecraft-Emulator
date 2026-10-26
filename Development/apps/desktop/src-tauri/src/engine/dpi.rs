use std::io;

#[cfg(target_os = "windows")]
pub fn configure() -> io::Result<()> {
    use windows_sys::Win32::UI::HiDpi::{
        SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
    };
    let ok = unsafe { SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
    if ok == 0 {
        let error = io::Error::last_os_error();
        // Windows may report access denied when a manifest or host already set
        // an equivalent DPI context. Do not downgrade an existing context.
        if error.raw_os_error() != Some(5) {
            return Err(error);
        }
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn configure() -> io::Result<()> {
    Ok(())
}
