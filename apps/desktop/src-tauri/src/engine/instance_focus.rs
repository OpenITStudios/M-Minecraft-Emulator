//! One-shot foreground diagnostic for the already verified Client 02 window.
//! Never changes focus, FPS, Minecraft settings, Windows process QoS or driver.
//! No timer, event hook, service, background worker or persistent data.
use serde::Serialize;
use std::io;

use super::{
    instance_catalog::{admit_slot_operation, SlotOperation},
    native_clients::{self, ProcessIdentityCandidate},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Client02ForegroundObservation {
    pub schema: u32,
    pub slot_id: &'static str,
    pub state: &'static str,
}

fn classify_foreground(verified_hwnd: u64, foreground_hwnd: Option<u64>) -> &'static str {
    match foreground_hwnd {
        None | Some(0) => "UNKNOWN",
        Some(current) if verified_hwnd != 0 && current == verified_hwnd => "FOREGROUND",
        Some(_) => "NOT_FOREGROUND",
    }
}

#[cfg(windows)]
pub fn inspect_client_02_foreground(
    target: ProcessIdentityCandidate,
) -> io::Result<Client02ForegroundObservation> {
    use windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    admit_slot_operation("02", SlotOperation::Inspect)?;
    // Reuse the same full and freshly validated Sandboxie image, PID/FILETIME
    // and HWND owner boundary already used for Focus and Place. Passing a
    // caller-supplied HWND or PID alone never grants an attributed slot.
    let mut state = "UNKNOWN";
    native_clients::with_verified_client_02_window(target, |verified| {
        let current = unsafe { GetForegroundWindow() };
        state = classify_foreground(
            verified as usize as u64,
            if current.is_null() { None } else { Some(current as usize as u64) },
        );
        Ok(())
    })?;
    Ok(Client02ForegroundObservation {
        schema: 1,
        slot_id: "02",
        state,
    })
}

#[cfg(not(windows))]
pub fn inspect_client_02_foreground(
    _target: ProcessIdentityCandidate,
) -> io::Result<Client02ForegroundObservation> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Foreground inspection requires an interactive Windows desktop",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn foreground_identity_matches_only_the_exact_verified_window() {
        assert_eq!(classify_foreground(0x200, Some(0x200)), "FOREGROUND");
        assert_eq!(classify_foreground(0x200, Some(0x300)), "NOT_FOREGROUND");
        assert_eq!(classify_foreground(0x200, None), "UNKNOWN");
        assert_eq!(classify_foreground(0x200, Some(0)), "UNKNOWN");
        assert_ne!(classify_foreground(0, Some(0)), "FOREGROUND");
    }

    #[test]
    fn snapshot_does_not_claim_fps_account_or_gameplay() {
        let result = Client02ForegroundObservation {
            schema: 1,
            slot_id: "02",
            state: classify_foreground(0x200, None),
        };
        let json = serde_json::to_value(result).unwrap();
        assert_eq!(json["state"], "UNKNOWN");
        assert_eq!(json.as_object().unwrap().len(), 3);
        assert!(json.get("fps").is_none());
        assert!(json.get("running").is_none());
        assert!(json.get("authenticated").is_none());
    }
}
