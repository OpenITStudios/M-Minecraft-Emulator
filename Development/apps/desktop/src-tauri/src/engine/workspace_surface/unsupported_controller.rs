//! Non-Windows compile boundary for the existing Windows-only presentation.
//! This module owns no game process, helper, native window or recovery state.
//! It is not a macOS adapter: game presentation remains unsupported until an
//! independently reviewed backend can prove exact process/window ownership.
use super::{OwnedProcess, Viewport};
use std::io;

fn unavailable<T>() -> io::Result<T> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Native Minecraft presentation is not implemented on this platform",
    ))
}

fn known_slot(slot: u32) -> io::Result<()> {
    if (1..=5).contains(&slot) {
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::InvalidInput, "Unknown Minecraft profile"))
    }
}

pub fn is_active() -> bool {
    false // No native workspace lease can be created on this platform.
}

pub fn cancel_close() {
    // No native checkpoint or state exists to mutate.
}

pub fn present_retained(
    _manager: u64, _viewport: Viewport, _target: OwnedProcess,
    _session: String, _client: String,
) -> io::Result<bool> {
    unavailable() // Do not return Ok(false): that would invite a second capture.
}

pub fn present_verified(
    _manager: u64, _viewport: Viewport, _target: OwnedProcess,
    _session: String, _client: String,
) -> io::Result<()> {
    unavailable()
}

pub fn update(_manager: u64, _viewport: Option<Viewport>) -> io::Result<()> {
    unavailable()
}

pub fn suspend() -> io::Result<bool> {
    unavailable()
}

pub fn release() -> io::Result<()> {
    unavailable() // Never claim a native window was restored.
}

pub fn release_for_close() -> io::Result<()> {
    unavailable() // Never grant normal game close from missing window evidence.
}

pub fn release_target(_target: &OwnedProcess) -> io::Result<()> {
    unavailable()
}

pub fn release_managed_slot(slot: u32) -> Result<(), (bool, io::Error)> {
    // false = no native window mutation was even attempted.
    known_slot(slot).and_then(|_| unavailable()).map_err(|error| (false, error))
}

pub fn allow_game_close_after_failed_restore() -> io::Result<bool> {
    unavailable()
}

pub fn allow_managed_slot_close_after_failed_restore(slot: u32) -> io::Result<bool> {
    known_slot(slot)?;
    unavailable()
}

pub fn release_exited_managed_slot_checkpoint(slot: u32) -> io::Result<()> {
    known_slot(slot)?;
    unavailable()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate() -> OwnedProcess {
        OwnedProcess {
            pid: 42,
            created_file_time: "134000000000000001".into(),
            session_id: 1,
            window_handle: "0x123".into(),
            responding: true,
        }
    }

    fn viewport() -> Viewport {
        Viewport { x: 0.0, y: 0.0, width: 800.0, height: 450.0, pixel_ratio: 1.0 }
    }

    #[test]
    fn unsupported_platform_never_claims_window_or_process_authority() {
        assert!(!is_active());
        cancel_close();
        assert_eq!(
            present_retained(0, viewport(), candidate(), "session".into(), "01".into())
                .unwrap_err().kind(),
            io::ErrorKind::Unsupported,
        );
        assert_eq!(
            present_verified(0, viewport(), candidate(), "session".into(), "01".into())
                .unwrap_err().kind(),
            io::ErrorKind::Unsupported,
        );
        assert_eq!(release().unwrap_err().kind(), io::ErrorKind::Unsupported);
        assert_eq!(release_for_close().unwrap_err().kind(), io::ErrorKind::Unsupported);
        assert_eq!(suspend().unwrap_err().kind(), io::ErrorKind::Unsupported);
        assert_eq!(update(0, Some(viewport())).unwrap_err().kind(), io::ErrorKind::Unsupported);
        assert_eq!(release_target(&candidate()).unwrap_err().kind(), io::ErrorKind::Unsupported);
        assert_eq!(allow_game_close_after_failed_restore().unwrap_err().kind(), io::ErrorKind::Unsupported);
        assert_eq!(
            allow_managed_slot_close_after_failed_restore(1).unwrap_err().kind(),
            io::ErrorKind::Unsupported,
        );
        assert_eq!(
            release_exited_managed_slot_checkpoint(1).unwrap_err().kind(),
            io::ErrorKind::Unsupported,
        );
        let (mutated, error) = release_managed_slot(1).unwrap_err();
        assert!(!mutated);
        assert_eq!(error.kind(), io::ErrorKind::Unsupported);
        let (mutated, error) = release_managed_slot(0).unwrap_err();
        assert!(!mutated);
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}
