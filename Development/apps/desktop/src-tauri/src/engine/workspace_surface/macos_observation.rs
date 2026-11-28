//! Read-only Quartz candidate-window observation. No ownership, capture,
//! focus, permission request, or native workspace admission.
use core_foundation::{
    array::{CFArray, CFArrayRef, CFArrayGetValueAtIndex},
    base::{CFType, TCFType},
    dictionary::{CFDictionary, CFDictionaryGetValue},
    number::CFNumber,
    string::CFString,
};
use std::collections::{HashMap, HashSet};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> CFArrayRef;
}

// CGWindow.h: on-screen only, exclude desktop elements, null window ID.
const WINDOW_LIST_OPTIONS: u32 = (1 << 0) | (1 << 4);
const MAX_WINDOWS: isize = 16_384;

fn integer(row: &CFDictionary, key: &CFString) -> Option<i64> {
    // SAFETY: the dictionary and key are live CF objects; a non-null
    // borrowed value remains alive while the dictionary owns it.
    let value = unsafe { CFDictionaryGetValue(row.as_concrete_TypeRef(), key.as_CFTypeRef()) };
    if value.is_null() {
        return None;
    }
    let value = unsafe { CFType::wrap_under_get_rule(value) };
    value.downcast::<CFNumber>()?.to_i64()
}

fn single_window_count(pids: &[libc::pid_t], rows: impl IntoIterator<Item = (libc::pid_t, u32, i64)>) -> Option<u32> {
    let mut counts: HashMap<_, u32> = pids.iter().copied().map(|pid| (pid, 0)).collect();
    if counts.len() != pids.len() || pids.iter().any(|pid| *pid <= 0) {
        return None;
    }
    let mut ids = HashSet::new();
    for (pid, window, layer) in rows {
        let Some(count) = counts.get_mut(&pid) else { continue };
        if window == 0 || !ids.insert(window) {
            return None;
        }
        if layer == 0 {
            *count = count.checked_add(1)?;
        }
    }
    u32::try_from(counts.values().filter(|&&count| count == 1).count()).ok()
}

pub(in crate::engine) fn observe_single_window_applications(pids: &[libc::pid_t]) -> Option<u32> {
    if pids.is_empty() {
        return Some(0); // No matching AppKit application to inspect.
    }
    // SAFETY: fixed documented arguments; non-null Copy result is owned.
    let raw = unsafe { CGWindowListCopyWindowInfo(WINDOW_LIST_OPTIONS, 0) };
    if raw.is_null() {
        return None; // No GUI session/window server, not a zero count.
    }
    let windows: CFArray = unsafe { CFArray::wrap_under_create_rule(raw) };
    if !(0..=MAX_WINDOWS).contains(&windows.len()) {
        return None;
    }
    let owner = CFString::new("kCGWindowOwnerPID");
    let number = CFString::new("kCGWindowNumber");
    let layer = CFString::new("kCGWindowLayer");
    let mut rows = Vec::new();
    for index in 0..windows.len() {
        // SAFETY: immutable array is retained, index is in range; Quartz
        // documents dictionary entries. Check the CF type before access.
        let raw = unsafe { CFArrayGetValueAtIndex(windows.as_concrete_TypeRef(), index) };
        if raw.is_null() {
            return None;
        }
        let entry = unsafe { CFType::wrap_under_get_rule(raw) };
        let row = entry.downcast::<CFDictionary>()?;
        let pid = libc::pid_t::try_from(integer(&row, &owner)?).ok()?;
        if !pids.contains(&pid) {
            continue; // Never read another app's window number or layer.
        }
        let id = u32::try_from(integer(&row, &number)?).ok()?;
        rows.push((pid, id, integer(&row, &layer)?));
    }
    single_window_count(pids, rows)
}

#[cfg(test)]
mod tests {
    use super::single_window_count;

    #[test]
    fn only_single_normal_layer_candidates_count() {
        assert_eq!(single_window_count(&[10, 11], [(10, 1, 0), (11, 2, 0), (11, 3, 5), (99, 4, 0)]), Some(2));
        assert_eq!(single_window_count(&[10, 11], [(10, 1, 0), (10, 2, 0)]), Some(0));
        assert_eq!(single_window_count(&[10, 11], [(10, 1, 0), (11, 2, 5)]), Some(1));
        assert_eq!(single_window_count(&[10], []), Some(0));
        assert_eq!(single_window_count(&[10, 11], [(10, 1, 0), (11, 1, 0)]), None);
        assert_eq!(single_window_count(&[10], [(10, 0, 0)]), None);
        assert_eq!(single_window_count(&[10, 10], []), None);
    }
}
