//! Stable captions for verified owned Minecraft root windows.
use super::{invalid, MAX_EDUCATION_SLOTS};
use std::io;

// Current slots are single-digit; fail compilation if that contract expands.
const _: () = assert!(MAX_EDUCATION_SLOTS <= 9);

fn slot_caption(slot: u32) -> io::Result<[u16; 14]> {
    if !(1..=MAX_EDUCATION_SLOTS).contains(&slot) {
        return Err(invalid("Invalid Minecraft window slot"));
    }
    let mut caption = (*b"Minecraft - 0\0").map(u16::from);
    caption[12] = u16::from(b'0') + slot as u16;
    Ok(caption)
}

pub(super) fn label_slot_window(
    hwnd: windows_sys::Win32::Foundation::HWND,
    pid: u32,
    slot: u32,
) -> io::Result<()> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowTextW, IsHungAppWindow, SendMessageTimeoutW, SMTO_ABORTIFHUNG, SMTO_BLOCK, SMTO_ERRORONEXIT,
        WM_SETTEXT,
    };
    let caption = slot_caption(slot)?;
    super::super::verify_live_client_02_window(hwnd, pid)?;
    let mut current = [0u16; 32];
    let length = unsafe { GetWindowTextW(hwnd, current.as_mut_ptr(), current.len() as i32) };
    if length > 0 && current[..length as usize] == caption[..caption.len() - 1] {
        return Ok(());
    }
    // System-message marshalling, bounded wait; never inject another game hook for presentation.
    super::super::verify_live_client_02_window(hwnd, pid)?;
    // Keep the unchanged-caption fast path and skip only the costly write
    // when an actual repair is needed but the verified window is hung.
    // The caller still observes responsiveness AFTER this operation.
    if unsafe { IsHungAppWindow(hwnd) } != 0 {
        return Err(io::Error::new(io::ErrorKind::WouldBlock, "Minecraft window is not responding"));
    }
    let mut result = 0;
    if unsafe {
        SendMessageTimeoutW(
            hwnd,
            WM_SETTEXT,
            0,
            caption.as_ptr() as isize,
            SMTO_ABORTIFHUNG | SMTO_BLOCK | SMTO_ERRORONEXIT,
            200,
            &mut result,
        )
    } == 0
        || result == 0
    {
        return Err(io::Error::last_os_error());
    }
    super::super::verify_live_client_02_window(hwnd, pid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::wide;
    use std::{ffi::OsStr, ptr};
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;

    #[test]
    fn slot_captions_use_stable_numbers_and_reject_foreign_windows() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, GetWindowTextW, SetWindowTextW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
        };
        for slot in 1..=MAX_EDUCATION_SLOTS {
            assert_eq!(
                String::from_utf16_lossy(&slot_caption(slot).unwrap()),
                format!("Minecraft - {slot}\0")
            );
        }
        assert!(slot_caption(0).is_err());
        assert!(slot_caption(MAX_EDUCATION_SLOTS + 1).is_err());
        assert!(slot_caption(u32::MAX).is_err());
        let class = wide(OsStr::new("STATIC")).unwrap();
        let title = wide(OsStr::new("Owned caption control")).unwrap();
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                class.as_ptr(),
                title.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                0,
                0,
                240,
                80,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        };
        assert!(!hwnd.is_null());
        let outcome = std::panic::catch_unwind(|| {
            assert!(
                label_slot_window(hwnd, unsafe { GetCurrentProcessId() }.wrapping_add(1), 1)
                    .is_err()
            );
            label_slot_window(hwnd, unsafe { GetCurrentProcessId() }, 2).unwrap();
            let mut current = [0u16; 32];
            let reset = wide(OsStr::new("Minecraft Education")).unwrap();
            assert_ne!(unsafe { SetWindowTextW(hwnd, reset.as_ptr()) }, 0);
            label_slot_window(hwnd, unsafe { GetCurrentProcessId() }, 2).unwrap();
            let length = unsafe { GetWindowTextW(hwnd, current.as_mut_ptr(), 32) };
            assert_eq!(
                String::from_utf16_lossy(&current[..length as usize]),
                "Minecraft - 2"
            );
            label_slot_window(hwnd, unsafe { GetCurrentProcessId() }, 2).unwrap();
        });
        assert_ne!(unsafe { DestroyWindow(hwnd) }, 0);
        assert!(label_slot_window(hwnd, unsafe { GetCurrentProcessId() }, 1).is_err());
        if let Err(error) = outcome {
            std::panic::resume_unwind(error);
        }
    }
}
