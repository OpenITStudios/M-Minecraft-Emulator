//! Tiny identity parsing shared by eligible client windows.
//! This validates format and nonzero values only; it is never enough to
//! establish Sandboxie membership, an Education signature, or gameplay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowIdentity {
    pub pid: u32,
    pub created_file_time: u64,
    pub hwnd: u64,
}

pub fn parse_window_identity(
    pid: u32,
    created_file_time: Option<&str>,
    window_handle: Option<&str>,
) -> Option<WindowIdentity> {
    if pid == 0 {
        return None;
    }
    let created = created_file_time?.parse::<u64>().ok()?;
    let hwnd = u64::from_str_radix(window_handle?.strip_prefix("0x")?, 16).ok()?;
    if created == 0 || hwnd == 0 {
        return None;
    }
    Some(WindowIdentity {
        pid,
        created_file_time: created,
        hwnd,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_full_width_windows_filetime_without_lossy_float_conversion() {
        assert_eq!(
            parse_window_identity(42, Some("133400000000000000"), Some("0x100")),
            Some(WindowIdentity { pid: 42, created_file_time: 133400000000000000, hwnd: 256 })
        );
        assert_eq!(parse_window_identity(42, Some("18446744073709551615"), Some("0xffffffffffffffff"))
            .map(|key| key.created_file_time), Some(u64::MAX));
    }
    #[test]
    fn refuses_missing_zero_reused_or_malformed_identity_pieces() {
        for (pid, created, handle) in [
            (0, Some("10"), Some("0x2")),
            (1, None, Some("0x2")),
            (1, Some("0"), Some("0x2")),
            (1, Some("-10"), Some("0x2")),
            (1, Some("10.5"), Some("0x2")),
            (1, Some("18446744073709551616"), Some("0x2")),
            (1, Some("10"), None),
            (1, Some("10"), Some("0x0")),
            (1, Some("10"), Some("2")),
            (1, Some("10"), Some("0x")),
            (1, Some("10"), Some("0xzz")),
        ] {
            assert_eq!(parse_window_identity(pid, created, handle), None);
        }
    }
}
