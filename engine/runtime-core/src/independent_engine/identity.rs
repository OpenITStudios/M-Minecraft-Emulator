//! Immutable domain identities. Live ownership requires OS handle verification
//! by apps/desktop; these value objects by themselves are NOT process proof.

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum SessionSlot {
    Mc01,
    Mc02,
    Mc03,
    Mc04,
}

impl SessionSlot {
    pub const ALL: [Self; 4] = [Self::Mc01, Self::Mc02, Self::Mc03, Self::Mc04];

    pub const fn as_desktop_id(self) -> &'static str {
        match self {
            Self::Mc01 => "01",
            Self::Mc02 => "02",
            Self::Mc03 => "03",
            Self::Mc04 => "04",
        }
    }

    pub fn from_desktop_id(value: &str) -> Option<Self> {
        match value {
            "01" => Some(Self::Mc01),
            "02" => Some(Self::Mc02),
            "03" => Some(Self::Mc03),
            "04" => Some(Self::Mc04),
            _ => None,
        }
    }

    // This describes the product topology, not a provider/launch selection.
    pub const fn is_host_native(self) -> bool {
        matches!(self, Self::Mc01)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub created_file_time: u64,
}

impl ProcessIdentity {
    // Require PID plus the full unsigned FILETIME. No JavaScript floating-point
    // conversion, process-name lookup, window title or PID-only ownership.
    pub fn checked(pid: u32, created_file_time: u64) -> Option<Self> {
        if pid == 0 || created_file_time == 0 {
            None
        } else {
            Some(Self { pid, created_file_time })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_slots_match_existing_desktop_ids_without_aliases() {
        assert_eq!(SessionSlot::ALL.map(SessionSlot::as_desktop_id), ["01", "02", "03", "04"]);
        for slot in SessionSlot::ALL {
            assert_eq!(SessionSlot::from_desktop_id(slot.as_desktop_id()), Some(slot));
        }
        for bad in ["", "1", "05", "00", "02 ", "MC02", "02\n"] {
            assert!(SessionSlot::from_desktop_id(bad).is_none());
        }
        assert!(SessionSlot::Mc01.is_host_native());
        assert!(SessionSlot::ALL[1..].iter().all(|slot| !slot.is_host_native()));
    }

    #[test]
    fn process_identity_rejects_unpinned_and_zero_values() {
        assert!(ProcessIdentity::checked(0, 4).is_none());
        assert!(ProcessIdentity::checked(10, 0).is_none());
        let key = ProcessIdentity::checked(10, u64::MAX).unwrap();
        assert_eq!(key.created_file_time, u64::MAX);
        assert_ne!(key, ProcessIdentity::checked(10, u64::MAX - 1).unwrap());
    }
}
