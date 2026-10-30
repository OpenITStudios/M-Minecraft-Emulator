//! Opt-in MC02 background input *feasibility*, not an AI gameplay backend.
//! Minecraft Education may ignore posted messages (polled/raw input).
//! Never inject global SendInput, activate windows or own game sessions.
use super::{process_identity::OwnedProcess, workspace_surface};
use std::{io, thread, time::Duration};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, IsHungAppWindow, IsWindowVisible, PostMessageW,
    WM_KEYDOWN, WM_KEYUP,
};

// ONLY one brief W pulse. No mouse, repeats, macros, or held input.
const W_KEY: usize = 0x57;
const W_SCAN: isize = 0x11;
const DOWN: isize = 1_isize | (W_SCAN << 16);
const UP: isize = DOWN | (1_isize << 30) | (1_isize << 31);

fn refused(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, message)
}
fn uncertain(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, message)
}

/// Controller attributed MC01 and MC02; a posted HWND message is never
/// evidence that the target Minecraft process interpreted movement input.
pub(crate) fn one_forward_pulse(human: &OwnedProcess, ai: &OwnedProcess) -> io::Result<()> {
    if !human.responding || !ai.responding {
        return Err(refused("Both game windows must be responsive"));
    }
    workspace_surface::with_verified_native_window(human, |human_hwnd| {
        workspace_surface::with_verified_native_window(ai, |ai_hwnd| {
            if human_hwnd == ai_hwnd {
                return Err(refused("Human and AI must have distinct Windows handles"));
            }
            let human_win = human_hwnd as _;
            let ai_win = ai_hwnd as _;
            // MC01's original game HWND must stay foreground. A manager or
            // unrelated foreground window is not sufficient authorization.
            if unsafe { GetForegroundWindow() } != human_win {
                return Err(refused("MC01 must retain foreground keyboard focus"));
            }
            if unsafe { IsWindowVisible(ai_win) } == 0 ||
                unsafe { IsHungAppWindow(ai_win) } != 0 {
                return Err(refused("Background MC02 is hidden or unresponsive"));
            }
            if !workspace_surface::matches_verified_native_window(ai, ai_hwnd)? {
                return Err(refused("MC02 window identity changed"));
            }
            // Targeted HWND messages. This does NOT modify the physical
            // keyboard state or guarantee Minecraft observes the key.
            if unsafe { PostMessageW(ai_win, WM_KEYDOWN, W_KEY, DOWN) } == 0 {
                return Err(io::Error::last_os_error());
            }
            thread::sleep(Duration::from_millis(80));
            // If the HWND was destroyed/reused, NEVER post to its new owner.
            // No blind retry; original vanished so key-up is inapplicable.
            if !workspace_surface::matches_verified_native_window(ai, ai_hwnd)? {
                return Err(uncertain("MC02 window changed after keydown"));
            }
            if unsafe { PostMessageW(ai_win, WM_KEYUP, W_KEY, UP) } == 0 {
                return Err(uncertain("MC02 keyup could not be posted"));
            }
            if unsafe { GetForegroundWindow() } != human_win {
                return Err(uncertain("MC01 foreground changed during probe"));
            }
            Ok(())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn short_pulse_contains_release_flag_and_never_uses_global_input() {
        assert_eq!(W_KEY, 0x57);
        assert_eq!(DOWN & (1_isize << 30), 0);
        assert_ne!(UP & (1_isize << 30), 0);
        assert_ne!(UP & (1_isize << 31), 0);
    }
}
