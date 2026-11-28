//! Shared five-profile lifecycle policy for the active native controller.
//! Pure metadata/selection only; the platform owner retains all process,
//! window, storage and authorization decisions. Not the historical four-slot catalog.
use std::io;

/// Current product roster; a valid slot is not permission to act.
pub const fn valid_profile_slot(slot: u32) -> bool {
    slot >= 1 && slot <= 5
}

// Project only facts already collected by the retained native owner. A
// missing window or uncertain observation must not relabel an owned live
// process as a stopped, relaunchable profile. No OS handle or state here.
pub fn profile_session_state(
    retained: bool,
    responding: Option<bool>,
    has_issue: bool,
    storage_available: bool,
) -> &'static str {
    if retained {
        match responding {
            Some(true) => "RUNNING",
            Some(false) => "NOT_RESPONDING",
            None if has_issue => "NOT_RESPONDING",
            None => "STARTING",
        }
    } else if has_issue || !storage_available {
        "ATTENTION"
    } else {
        "STOPPED"
    }
}

// Select exactly one slot or the complete roster. Selection is not close
// permission: the native owner still verifies lifetime and normal exit.
pub fn normal_close_mask(slot:Option<u32>)->io::Result<[bool;5]> {
    if slot.is_some_and(|value|!valid_profile_slot(value)) {
        return Err(io::Error::new(io::ErrorKind::InvalidInput,"Unknown Minecraft profile"));
    }
    Ok(std::array::from_fn(|index|slot.is_none_or(|value|value==index as u32+1)))
}
