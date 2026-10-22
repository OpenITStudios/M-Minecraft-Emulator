//! Independent in-memory copy-on-write namespace for synthetic file and
//! registry-value resources. No filesystem, HKCU, Win32 hooks or game writes.
//!
//! A base snapshot is explicitly test-supplied and immutable. Each MC02-04
//! session owns a separate overlay. Read merges private entries over the
//! snapshot; deletion leaves tombstones that mask the snapshot; mutations
//! never change base bytes. This is executable COW *domain logic*, not a
//! Windows interception layer or Minecraft-compatible pathname adapter.
//!
//! Names are deliberately ASCII-only and bounded for synthetic fixtures.
//! Arbitrary real-world Minecraft Unicode filenames, registry key metadata,
//! directory operations, alternate streams and Windows link behavior require
//! a separate validated native implementation before any production use.

use std::collections::{BTreeMap, BTreeSet};

use super::{ProcessIdentity, SessionSlot};

pub const MAX_OVERLAY_ITEM_BYTES: usize = 64 * 1024;
pub const MAX_OVERLAY_ITEMS: usize = 1024;
pub const MAX_OVERLAY_PRIVATE_BYTES: usize = 8 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum OverlaySurface {
    File,
    RegistryValue,
}

impl OverlaySurface {
    fn separator(self) -> char {
        match self {
            Self::File => '/',
            Self::RegistryValue => '\\',
        }
    }
}

/// A resource inside a *synthetic* namespace, never an OS-qualified path.
/// Case folding here covers ASCII fixture names only, not full Windows
/// Unicode case mappings. These keys MUST NOT be passed to native file APIs.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OverlayKey {
    surface: OverlaySurface,
    name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OverlayError {
    HostSlotForbidden,
    InvalidProcess,
    WrongOwnerOrProcess,
    InvalidName,
    EntryTooLarge,
    CapacityExceeded,
    AlreadyExists,
    MissingSource,
    AmbiguousPrivateProvenance,
    HostCopyIntoPrivateBoundary,
}

impl OverlayKey {
    pub fn parse(surface: OverlaySurface, input: &str) -> Result<Self, OverlayError> {
        if input.is_empty() || input.len() > 240 || !input.is_ascii() {
            return Err(OverlayError::InvalidName);
        }
        let separator = surface.separator();
        let parts: Vec<_> = input.split(separator).collect();
        if parts.len() > 32 || parts.iter().any(|component| !valid_component(component)) {
            return Err(OverlayError::InvalidName);
        }
        Ok(Self { surface, name: input.to_ascii_lowercase() })
    }

    pub fn surface(&self) -> OverlaySurface {
        self.surface
    }

    pub fn normalized_name(&self) -> &str {
        &self.name
    }

    fn at_or_below(&self, prefix: &Self) -> bool {
        if self.surface != prefix.surface {
            return false;
        }
        self.name == prefix.name
            || self.name.strip_prefix(&prefix.name).is_some_and(|rest| {
                rest.starts_with(self.surface.separator())
            })
    }
}

fn valid_component(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 || value == "." || value == ".."
        || value.ends_with('.') || value.ends_with(' ')
        || value.as_bytes().iter().any(|c| !c.is_ascii_alphanumeric()
            && !matches!(*c, b'-' | b'_' | b'.' | b' ' | b'$'))
    {
        return false;
    }
    // Treat dangerous DOS device basenames as invalid even with extensions.
    // No code here opens Windows filenames, but accepting these would make
    // later adapters ambiguous.
    let stem = value.split('.').next().unwrap_or("");
    let stem = stem.to_ascii_uppercase();
    !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL"
        | "CONIN$" | "CONOUT$")
        && !(stem.len() == 4 && (
            (stem.starts_with("COM") || stem.starts_with("LPT"))
                && stem.as_bytes()[3].is_ascii_digit()
        ))
}

/// Immutable test-supplied host view: there is no path or OS handle to
/// the actual desktop account, filesystem, registry or authentication broker.
#[derive(Clone, Debug, Default)]
pub struct SyntheticBaseView {
    entries: BTreeMap<OverlayKey, Vec<u8>>,
    bytes: usize,
}

impl SyntheticBaseView {
    pub fn insert_fixture(
        &mut self,
        key: OverlayKey,
        data: &[u8],
    ) -> Result<(), OverlayError> {
        if data.len() > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        if self.entries.contains_key(&key) {
            return Err(OverlayError::AlreadyExists);
        }
        // This synthetic File surface stores regular-file leaves only.
        // Refuse a fixture in which one leaf is also a parent directory,
        // regardless of insertion order. RegistryValue stays flattened.
        if key.surface == OverlaySurface::File
            && self.entries.keys().any(|existing| {
                existing.surface == OverlaySurface::File
                    && (existing.at_or_below(&key) || key.at_or_below(existing))
            })
        {
            return Err(OverlayError::AlreadyExists);
        }
        let future_bytes = self.bytes.checked_add(data.len())
            .ok_or(OverlayError::CapacityExceeded)?;
        if self.entries.len() >= MAX_OVERLAY_ITEMS
            || future_bytes > MAX_OVERLAY_PRIVATE_BYTES
        {
            return Err(OverlayError::CapacityExceeded);
        }
        self.bytes = future_bytes;
        self.entries.insert(key, data.to_vec());
        Ok(())
    }

    pub fn fixture_bytes(&self, key: &OverlayKey) -> Option<&[u8]> {
        self.entries.get(key).map(Vec::as_slice)
    }
}

/// A bound, in-memory overlay; there is no process-identity trust claim.
/// The caller must re-pin a real PID/FILETIME before using any future OS
/// adapter. A slot/proc mismatch fails before reading or altering state.
#[derive(Clone, Debug)]
pub struct SyntheticSlotOverlay {
    slot: SessionSlot,
    process: ProcessIdentity,
    private: BTreeMap<OverlayKey, Vec<u8>>,
    // An explicit host copy-up or partial update of host-derived bytes
    // cannot later be disguised as newly authored data through rename.
    host_derived: BTreeSet<OverlayKey>,
    tombstones: BTreeSet<OverlayKey>,
    // Explicit read boundary: fixture host values under these prefixes may
    // NEVER fall back, even when not deleted. Private values still override.
    private_only_prefixes: BTreeSet<OverlayKey>,
    private_bytes: usize,
}

impl SyntheticSlotOverlay {
    pub fn new(slot: SessionSlot, process: ProcessIdentity) -> Result<Self, OverlayError> {
        if slot.is_host_native() {
            return Err(OverlayError::HostSlotForbidden);
        }
        if process.pid == 0 || process.created_file_time == 0 {
            return Err(OverlayError::InvalidProcess);
        }
        Ok(Self {
            slot,
            process,
            private: BTreeMap::new(),
            host_derived: BTreeSet::new(),
            tombstones: BTreeSet::new(),
            private_only_prefixes: BTreeSet::new(),
            private_bytes: 0,
        })
    }

    fn check(&self, slot: SessionSlot, process: ProcessIdentity) -> Result<(), OverlayError> {
        if slot != self.slot || process != self.process {
            return Err(OverlayError::WrongOwnerOrProcess);
        }
        Ok(())
    }

    /// Shared bounded prefix-set admission. A wider mask can replace many
    /// narrower masks without consuming another quota entry. File/registry
    /// surfaces are distinct and cannot mask one another.
    fn preflight_prefix_insert(
        entries: &BTreeSet<OverlayKey>,
        prefix: &OverlayKey,
    ) -> Result<bool, OverlayError> {
        let descendants = entries.iter()
            .filter(|key| key.at_or_below(prefix))
            .count();
        let covered = entries.iter()
            .any(|key| key != prefix && prefix.at_or_below(key));
        let future = entries.len().checked_sub(descendants)
            .and_then(|remaining| remaining.checked_add(usize::from(!covered)))
            .ok_or(OverlayError::CapacityExceeded)?;
        if future > MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        Ok(covered)
    }

    /// Apply a preflighted prefix insertion with no remaining denial paths.
    fn install_prefix(
        entries: &mut BTreeSet<OverlayKey>,
        prefix: OverlayKey,
        covered: bool,
    ) {
        entries.retain(|key| !key.at_or_below(&prefix));
        if !covered {
            entries.insert(prefix);
        }
    }

    /// This synthetic tombstone policy is NOT a Windows ACL or file hook.
    fn preflight_prefix_mask(&self, prefix: &OverlayKey) -> Result<bool, OverlayError> {
        Self::preflight_prefix_insert(&self.tombstones, prefix)
    }

    fn install_prefix_mask(&mut self, prefix: OverlayKey, covered: bool) {
        Self::install_prefix(&mut self.tombstones, prefix, covered);
    }

    /// A visible *file leaf* cannot also be the parent directory of a
    /// destination in a real filesystem. Check only fixture File surface;
    /// flattened RegistryValue names are not OS registry tree metadata.
    fn visible_file_ancestor(
        &self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: &OverlayKey,
    ) -> Result<bool, OverlayError> {
        if key.surface != OverlaySurface::File {
            return Ok(false);
        }
        for (index, _) in key.name.match_indices('/') {
            let ancestor = OverlayKey {
                surface: OverlaySurface::File,
                name: key.name[..index].to_owned(),
            };
            if self.read(host, slot, process, &ancestor)?.is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn visible_file_descendants(
        &self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: &OverlayKey,
    ) -> Result<bool, OverlayError> {
        if key.surface != OverlaySurface::File {
            return Ok(false);
        }
        Ok(self.list(host, slot, process, key)?
            .iter().any(|candidate| candidate != key))
    }

    /// A synthetic file leaf cannot coexist with a visible parent file or
    /// child files. The host snapshot is essential: private-only metadata
    /// and tombstones may hide otherwise-colliding host leaves.
    fn preflight_file_leaf(
        &self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: &OverlayKey,
    ) -> Result<(), OverlayError> {
        if self.visible_file_ancestor(host, slot, process, key)?
            || self.visible_file_descendants(host, slot, process, key)?
        {
            return Err(OverlayError::AlreadyExists);
        }
        Ok(())
    }

    fn masked(&self, key: &OverlayKey) -> bool {
        self.tombstones.iter().any(|deleted| key.at_or_below(deleted))
    }

    fn host_fallback_denied(&self, key: &OverlayKey) -> bool {
        self.private_only_prefixes.iter().any(|prefix| key.at_or_below(prefix))
    }

    /// Require all reads under a synthetic directory/registry prefix to
    /// originate from this slot's PRIVATE overlay. This is a diagnostic
    /// policy method, never a filesystem ACL or Windows hook.
    pub fn make_private_only(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
        prefix: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        // A strict ancestor already enforces this boundary. Do not consume
        // quota or refuse a redundant rule merely because this slot later
        // authored its own private data beneath that existing boundary.
        if self.private_only_prefixes.iter()
            .any(|existing| prefix.at_or_below(existing))
        {
            return Ok(());
        }
        // A NEW/wider rule installed after private bytes exist cannot
        // establish their provenance, even if a synthetic tag looks safe.
        // Reject before coalescing smaller existing rules.
        if self.private.keys().any(|key| key.at_or_below(&prefix)) {
            return Err(OverlayError::AmbiguousPrivateProvenance);
        }
        let covered = Self::preflight_prefix_insert(
            &self.private_only_prefixes, &prefix,
        )?;
        Self::install_prefix(&mut self.private_only_prefixes, prefix, covered);
        Ok(())
    }

    /// Private value first, then optionally visible fixture base; never
    /// reads the real Windows host OS, regardless of the configured prefix.
    pub fn read<'a>(
        &'a self,
        host: &'a SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: &OverlayKey,
    ) -> Result<Option<&'a [u8]>, OverlayError> {
        self.check(slot, process)?;
        if let Some(value) = self.private.get(key) {
            return Ok(Some(value.as_slice()));
        }
        if self.masked(key) || self.host_fallback_denied(key) {
            return Ok(None);
        }
        Ok(host.fixture_bytes(key))
    }

    /// A full replacement is always made in private memory, even when the
    /// immutable host fixture contains the same name. No host write exists.
    pub fn write(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: OverlayKey,
        data: &[u8],
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if data.len() > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        // Require the exact immutable baseline used by other COW methods,
        // so overwrites cannot silently create a file/directory collision.
        // Do NOT require the baseline's value to be copied: full writes
        // remain freshly authored and clear host-derived provenance.
        self.preflight_file_leaf(host, slot, process, &key)?;
        let previous = self.private.get(&key).map_or(0, Vec::len);
        if !self.private.contains_key(&key) && self.private.len() >= MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        let total = self.private_bytes.checked_sub(previous)
            .and_then(|remaining| remaining.checked_add(data.len()))
            .ok_or(OverlayError::CapacityExceeded)?;
        if total > MAX_OVERLAY_PRIVATE_BYTES {
            return Err(OverlayError::CapacityExceeded);
        }
        self.private_bytes = total;
        // A full replacement has newly authored content, not a copy-up.
        self.host_derived.remove(&key);
        // A write after a subtree delete must NOT remove its prefix
        // tombstone: that would resurrect unrelated host descendants.
        // Reads always prefer private bytes over matching tombstones.
        self.private.insert(key, data.to_vec());
        Ok(())
    }

    /// Materialize one visible immutable host fixture leaf into this slot.
    /// Reject missing/masked host data or any quota excess before mutation.
    /// An existing private value is a no-op, with its provenance unchanged.
    /// Copy source bytes ONCE: write() would clone the payload again and
    /// clear the host-derived tag, which is wrong for a copy-up.
    pub fn copy_up(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: &OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if self.private.contains_key(key) {
            return Ok(());
        }
        let visible = self.read(host, slot, process, key)?
            .ok_or(OverlayError::MissingSource)?;
        self.preflight_file_leaf(host, slot, process, key)?;
        if visible.len() > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        if self.private.len() >= MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        let total = self.private_bytes.checked_add(visible.len())
            .ok_or(OverlayError::CapacityExceeded)?;
        if total > MAX_OVERLAY_PRIVATE_BYTES {
            return Err(OverlayError::CapacityExceeded);
        }
        // No fallible quota or source checks remain after this one copy.
        let owned = visible.to_vec();
        self.private.insert(key.clone(), owned);
        self.host_derived.insert(key.clone());
        self.private_bytes = total;
        Ok(())
    }

    /// Partial file writes reuse an already-owned private Vec in place.
    /// Synthetic host fallback is copied at most once; holes stay zero-filled.
    /// All logical size/item/quota denial happens before private mutation.
    /// Registry values have no offset operation.
    pub fn write_at(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: OverlayKey,
        offset: usize,
        data: &[u8],
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if key.surface != OverlaySurface::File {
            return Err(OverlayError::InvalidName);
        }
        let end = offset.checked_add(data.len())
            .ok_or(OverlayError::EntryTooLarge)?;
        if end > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        // A zero-byte write cannot extend an existing file or implicitly
        // create a private file. Do not copy up host bytes for a no-op;
        // explicit write() creates an empty private value if needed.
        if data.is_empty() {
            return Ok(());
        }
        self.preflight_file_leaf(host, slot, process, &key)?;
        let private = self.private.contains_key(&key);
        let original = self.read(host, slot, process, &key)?;
        let future_len = original.map_or(0, <[u8]>::len).max(end);
        let derived = self.host_derived.contains(&key)
            || (!private && original.is_some());
        if !private && self.private.len() >= MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        let previous = self.private.get(&key).map_or(0, Vec::len);
        let future_bytes = self.private_bytes.checked_sub(previous)
            .and_then(|rest| rest.checked_add(future_len))
            .ok_or(OverlayError::CapacityExceeded)?;
        if future_bytes > MAX_OVERLAY_PRIVATE_BYTES {
            return Err(OverlayError::CapacityExceeded);
        }
        // Finish all potentially rejecting reads and calculations before
        // mutating state. The new Vec is allocated only for a base fallback
        // or an absent file, not for a previously private value.
        let host_copy = if private {
            None
        } else {
            Some(original.map_or_else(Vec::new, |bytes| bytes.to_vec()))
        };
        if private {
            let owned = self.private.get_mut(&key)
                .expect("preflighted private file");
            owned.resize(future_len, 0);
            owned[offset..end].copy_from_slice(data);
        } else {
            let mut owned = host_copy.expect("preflighted new private file");
            owned.resize(future_len, 0);
            owned[offset..end].copy_from_slice(data);
            self.private.insert(key.clone(), owned);
        }
        self.private_bytes = future_bytes;
        if derived {
            self.host_derived.insert(key);
        }
        Ok(())
    }

    /// Truncation reuses an existing private allocation. Copy-on-write
    /// host fallback remains tagged as host-derived even when shortened.
    pub fn truncate_file(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: OverlayKey,
        length: usize,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if key.surface != OverlaySurface::File {
            return Err(OverlayError::InvalidName);
        }
        if length > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        let private = self.private.contains_key(&key);
        let original = self.read(host, slot, process, &key)?
            .ok_or(OverlayError::MissingSource)?;
        self.preflight_file_leaf(host, slot, process, &key)?;
        let derived = self.host_derived.contains(&key) || !private;
        if !private && self.private.len() >= MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        let previous = self.private.get(&key).map_or(0, Vec::len);
        let future_bytes = self.private_bytes.checked_sub(previous)
            .and_then(|rest| rest.checked_add(length))
            .ok_or(OverlayError::CapacityExceeded)?;
        if future_bytes > MAX_OVERLAY_PRIVATE_BYTES {
            return Err(OverlayError::CapacityExceeded);
        }
        let host_copy = if private { None } else { Some(original.to_vec()) };
        if private {
            self.private.get_mut(&key)
                .expect("preflighted private file")
                .resize(length, 0);
        } else {
            let mut owned = host_copy.expect("preflighted host fallback");
            owned.resize(length, 0);
            self.private.insert(key.clone(), owned);
        }
        self.private_bytes = future_bytes;
        if derived {
            self.host_derived.insert(key);
        }
        Ok(())
    }

    /// Delete an exact private leaf and mask fixture host fallback.
    /// A pre-existing ancestor mask needs no extra tombstone capacity.
    pub fn delete(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
        key: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        let covered = self.preflight_prefix_mask(&key)?;
        if let Some(value) = self.private.remove(&key) {
            self.private_bytes -= value.len();
        }
        self.host_derived.remove(&key);
        self.install_prefix_mask(key, covered);
        Ok(())
    }

    /// Hide a complete directory/subkey prefix in the synthetic base view.
    /// Coalesce redundant descendant tombstones BEFORE quota admission;
    /// old private descendants are removed, while other slots stay intact.
    pub fn delete_tree(
        &mut self,
        slot: SessionSlot,
        process: ProcessIdentity,
        prefix: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        let covered = self.preflight_prefix_mask(&prefix)?;
        let old: Vec<_> = self.private.keys()
            .filter(|key| key.at_or_below(&prefix))
            .cloned()
            .collect();
        for key in old {
            if let Some(bytes) = self.private.remove(&key) {
                self.private_bytes -= bytes.len();
            }
            self.host_derived.remove(&key);
        }
        self.install_prefix_mask(prefix, covered);
        Ok(())
    }

    /// Transactional synthetic rename with checked quotas, no whole-state
    /// clone, and no OS host mutation. The old key is tombstoned in this slot.
    pub fn rename(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        old: &OverlayKey,
        new: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if old == &new {
            return if self.read(host, slot, process, old)?.is_some() {
                Ok(())
            } else {
                Err(OverlayError::MissingSource)
            };
        }
        if old.surface != new.surface {
            return Err(OverlayError::InvalidName);
        }
        // A host-backed item must not cross into a private-only region
        // via rename without explicit, independently authorized import.
        // There is no import capability in this synthetic model.
        if (!self.private.contains_key(old) || self.host_derived.contains(old))
            && self.host_fallback_denied(&new)
        {
            return Err(OverlayError::HostCopyIntoPrivateBoundary);
        }
        // Preflight every fallible constraint BEFORE changing this overlay.
        // Renaming one item must not clone up to 8 MiB of unrelated data.
        let size = self.read(host, slot, process, old)?
            .ok_or(OverlayError::MissingSource)?.len();
        // Refuse file/value renames into their own ancestors/descendants.
        // The flat fixture namespace must not manufacture a leaf beneath
        // itself simply because tombstones allow private reads to bypass it.
        if old.at_or_below(&new) || new.at_or_below(old) {
            return Err(OverlayError::InvalidName);
        }
        if self.read(host, slot, process, &new)?.is_some()
            || self.visible_file_descendants(host, slot, process, &new)?
            || self.visible_file_ancestor(host, slot, process, &new)?
        {
            return Err(OverlayError::AlreadyExists);
        }
        // A file leaf with visible children would be an ambiguous source;
        // refuse moving only the leaf while leaving its descendants behind.
        if self.visible_file_descendants(host, slot, process, old)? {
            return Err(OverlayError::InvalidName);
        }
        let private_source = self.private.contains_key(old);
        if size > MAX_OVERLAY_ITEM_BYTES {
            return Err(OverlayError::EntryTooLarge);
        }
        // An existing tombstone needs no extra slot; otherwise deletion
        // must have room to suppress the original host fallback.
        let covered = self.preflight_prefix_mask(old)?;
        let future_items = self.private.len() - usize::from(private_source) + 1;
        if future_items > MAX_OVERLAY_ITEMS {
            return Err(OverlayError::CapacityExceeded);
        }
        let previous = if private_source { size } else { 0 };
        let future_bytes = self.private_bytes.checked_sub(previous)
            .and_then(|remaining| remaining.checked_add(size))
            .ok_or(OverlayError::CapacityExceeded)?;
        if future_bytes > MAX_OVERLAY_PRIVATE_BYTES {
            return Err(OverlayError::CapacityExceeded);
        }
        let derived = self.host_derived.contains(old) || !private_source;
        // A host fallback may copy ONE bounded fixture value; failures
        // must happen before any private map or tombstone is changed.
        let host_copy = if private_source {
            None
        } else {
            Some(host.fixture_bytes(old).ok_or(OverlayError::MissingSource)?.to_vec())
        };
        // No logical Result/error path remains after the first mutation.
        // An owned private buffer is transferred without allocating a copy.
        let moved = if private_source {
            self.private.remove(old).expect("preflighted private COW source")
        } else {
            host_copy.expect("preflighted synthetic host source")
        };
        self.private_bytes = future_bytes;
        self.host_derived.remove(old);
        self.install_prefix_mask(old.clone(), covered);
        self.private.insert(new.clone(), moved);
        if derived {
            self.host_derived.insert(new);
        }
        Ok(())
    }

    /// Move every currently visible synthetic leaf beneath a file-directory
    /// or registry-key prefix. Host/base data is copied at most once per
    /// visible leaf; existing private Vec allocations are TRANSFERRED.
    ///
    /// This is deliberately NOT an OS directory/registry rename: names are
    /// bounded ASCII fixture keys, there are no Windows directories, handles,
    /// ACLs, timestamps, links, hive metadata or Minecraft data. Prefix
    /// policies remain tied to their original names. The destination subtree
    /// must be empty in the merged view; hidden host entries never import.
    ///
    /// Every recoverable failure is checked before changing private maps,
    /// host-derived tags or tombstones. One source-prefix tombstone masks all
    /// original host descendants, including descendants not moved.
    pub fn rename_tree(
        &mut self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        from: &OverlayKey,
        to: OverlayKey,
    ) -> Result<(), OverlayError> {
        self.check(slot, process)?;
        if from.surface != to.surface {
            return Err(OverlayError::InvalidName);
        }
        let sources = self.list(host, slot, process, from)?;
        if sources.is_empty() {
            return Err(OverlayError::MissingSource);
        }
        if from == &to {
            return Ok(());
        }
        // A subtree cannot move into itself or its ancestor. Keeping source
        // and destination disjoint makes the final mutation infallible.
        if from.at_or_below(&to) || to.at_or_below(from) {
            return Err(OverlayError::InvalidName);
        }
        // This File surface models leaves, not directory metadata. A leaf
        // at the source prefix AND separate children would be ambiguous.
        if from.surface == OverlaySurface::File
            && sources.len() > 1 && sources.iter().any(|key| key == from)
        {
            return Err(OverlayError::InvalidName);
        }
        // Reject an existing destination SUBTREE or a visible file leaf
        // in its ancestor path. Neither condition can be a legal directory
        // rename target even when the destination prefix is currently empty.
        if !self.list(host, slot, process, &to)?.is_empty()
            || self.visible_file_ancestor(host, slot, process, &to)?
        {
            return Err(OverlayError::AlreadyExists);
        }

        let mut planned = Vec::with_capacity(sources.len());
        let mut private_sources = 0usize;
        let mut removed_bytes = 0usize;
        let mut incoming_bytes = 0usize;
        for original in sources {
            // Each enumerated key is at/below 'from', so this is an ASCII
            // component-bound suffix, never a raw/untrusted Windows path.
            let suffix = &original.name[from.name.len()..];
            let destination =
                OverlayKey::parse(to.surface, &format!("{}{}", to.name, suffix))?;
            let value = self.read(host, slot, process, &original)?
                .ok_or(OverlayError::MissingSource)?;
            if value.len() > MAX_OVERLAY_ITEM_BYTES {
                return Err(OverlayError::EntryTooLarge);
            }
            let private = self.private.contains_key(&original);
            let derived = self.host_derived.contains(&original) || !private;
            if derived && self.host_fallback_denied(&destination) {
                return Err(OverlayError::HostCopyIntoPrivateBoundary);
            }
            incoming_bytes = incoming_bytes.checked_add(value.len())
                .ok_or(OverlayError::CapacityExceeded)?;
            if private {
                private_sources += 1;
                removed_bytes = removed_bytes.checked_add(value.len())
                    .ok_or(OverlayError::CapacityExceeded)?;
            }
            planned.push((original, destination, private, derived));
        }

        let future_items = self.private.len().checked_sub(private_sources)
            .and_then(|left| left.checked_add(planned.len()))
            .ok_or(OverlayError::CapacityExceeded)?;
        let future_bytes = self.private_bytes.checked_sub(removed_bytes)
            .and_then(|left| left.checked_add(incoming_bytes))
            .ok_or(OverlayError::CapacityExceeded)?;
        if future_items > MAX_OVERLAY_ITEMS
            || future_bytes > MAX_OVERLAY_PRIVATE_BYTES
        {
            return Err(OverlayError::CapacityExceeded);
        }

        // Reuse the exact same quota/masking semantics as deletion and
        // leaf rename; a full tombstone set may still have reusable masks.
        let covered = self.preflight_prefix_mask(from)?;

        // Materialize ONLY the visible host-backed leaves, after quota and
        // provenance checks. All private-owned buffers remain un-copied.
        // There are no remaining Result-returning operations after this.
        let host_values: Vec<Option<Vec<u8>>> = planned.iter().map(
            |(source, _, private, _)| {
                if *private {
                    None
                } else {
                    Some(host.fixture_bytes(source)
                        .expect("preflighted visible synthetic base leaf")
                        .to_vec())
                }
            },
        ).collect();

        for ((source, destination, private, derived), host_value)
            in planned.into_iter().zip(host_values)
        {
            let bytes = if private {
                self.private.remove(&source)
                    .expect("preflighted private subtree leaf")
            } else {
                host_value.expect("preflighted host subtree leaf")
            };
            self.host_derived.remove(&source);
            self.private.insert(destination.clone(), bytes);
            if derived {
                self.host_derived.insert(destination);
            }
        }
        self.install_prefix_mask(from.clone(), covered);
        self.private_bytes = future_bytes;
        Ok(())
    }

    /// Deterministic merged enumeration, delimited by directory/key-prefix
    /// boundaries. Never combines different surfaces or another slot's state.
    pub fn list(
        &self,
        host: &SyntheticBaseView,
        slot: SessionSlot,
        process: ProcessIdentity,
        prefix: &OverlayKey,
    ) -> Result<Vec<OverlayKey>, OverlayError> {
        self.check(slot, process)?;
        let mut seen = BTreeSet::new();
        for key in host.entries.keys() {
            if key.at_or_below(prefix)
                && !self.masked(key) && !self.host_fallback_denied(key)
            {
                seen.insert(key.clone());
            }
        }
        for key in self.private.keys() {
            if key.at_or_below(prefix) {
                seen.insert(key.clone());
            }
        }
        Ok(seen.into_iter().collect())
    }

    pub fn private_byte_count(&self) -> usize {
        self.private_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(pid: u32) -> ProcessIdentity {
        ProcessIdentity::checked(pid, 133400000000000000 + u64::from(pid)).unwrap()
    }
    fn file(value: &str) -> OverlayKey {
        OverlayKey::parse(OverlaySurface::File, value).unwrap()
    }
    fn reg(value: &str) -> OverlayKey {
        OverlayKey::parse(OverlaySurface::RegistryValue, value).unwrap()
    }
    fn setup() -> (SyntheticBaseView, SyntheticSlotOverlay, SyntheticSlotOverlay) {
        let mut base = SyntheticBaseView::default();
        base.insert_fixture(file("world/data.bin"), b"original").unwrap();
        base.insert_fixture(file("world/other.bin"), b"second").unwrap();
        base.insert_fixture(reg("Software\\Game\\Setting"), b"base-setting").unwrap();
        (base,
         SyntheticSlotOverlay::new(SessionSlot::Mc02, id(22)).unwrap(),
         SyntheticSlotOverlay::new(SessionSlot::Mc03, id(33)).unwrap())
    }

    #[test]
    fn fixture_paths_are_case_insensitive_but_reject_os_path_escape() {
        assert_eq!(file("World/DATA.bin"), file("world/data.BIN"));
        assert_eq!(reg("Software\\GAME\\Setting"), reg("software\\game\\setting"));
        for (kind, bad) in [
            (OverlaySurface::File, "../host"),
            (OverlaySurface::File, "/absolute"),
            (OverlaySurface::File, "world//file"),
            (OverlaySurface::File, "world\\backslash"),
            (OverlaySurface::File, "world/NUL.txt"),
            (OverlaySurface::File, "world/com1.log"),
            (OverlaySurface::File, "world/bad:stream"),
            (OverlaySurface::RegistryValue, "HKCU\\..\\outside"),
            (OverlaySurface::RegistryValue, "Software\\A/B"),
            (OverlaySurface::RegistryValue, "\\HKCU"),
            (OverlaySurface::RegistryValue, "Software\\\\Value"),
            (OverlaySurface::File, "world/é"),
        ] {
            assert_eq!(OverlayKey::parse(kind, bad), Err(OverlayError::InvalidName));
        }
        assert!(OverlayKey::parse(OverlaySurface::File, "world/.cache").is_ok());
    }

    #[test]
    fn private_copy_on_write_masks_host_without_mutating_base() {
        let (base, mut box_a, box_b) = setup();
        let key = file("world/data.bin");
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), Some(b"original".as_slice()));
        box_a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        assert_eq!(box_a.private_byte_count(), b"original".len());
        box_a.write(&base, SessionSlot::Mc02, id(22), key.clone(), b"changed").unwrap();
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), Some(b"changed".as_slice()));
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
        assert_eq!(box_b.read(&base, SessionSlot::Mc03, id(33), &key).unwrap(), Some(b"original".as_slice()));
        box_a.delete(SessionSlot::Mc02, id(22), key.clone()).unwrap();
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), None);
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
    }

    #[test]
    fn registry_values_are_independent_of_file_names_and_peer_boxes() {
        let (base, mut a, b) = setup();
        let registry = reg("SOFTWARE\\Game\\Setting");
        a.write(&base, SessionSlot::Mc02, id(22), registry.clone(), b"a-private").unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &registry).unwrap(), Some(b"a-private".as_slice()));
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33), &registry).unwrap(), Some(b"base-setting".as_slice()));
        a.delete(SessionSlot::Mc02, id(22), registry.clone()).unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &registry).unwrap(), None);
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33), &registry).unwrap(), Some(b"base-setting".as_slice()));
    }

    #[test]
    fn directory_tombstones_hide_host_descendants_but_allow_new_private_children() {
        let (base, mut box_a, box_b) = setup();
        box_a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &file("world/data.bin")).unwrap(), None);
        box_a.write(&base, SessionSlot::Mc02, id(22), file("world/new.bin"), b"new").unwrap();
        let names = box_a.list(&base, SessionSlot::Mc02, id(22), &file("world")).unwrap();
        assert_eq!(names, vec![file("world/new.bin")]);
        assert_eq!(box_b.list(&base, SessionSlot::Mc03, id(33), &file("world")).unwrap().len(), 2);
        assert!(box_a.list(&base, SessionSlot::Mc02, id(22), &file("worl")).unwrap().is_empty());
    }

    #[test]
    fn pid_reuse_cross_slot_and_mc01_must_never_touch_a_box() {
        let (base, mut box_a, _) = setup();
        let key = file("world/data.bin");
        for (slot, process) in [
            (SessionSlot::Mc03, id(22)),
            (SessionSlot::Mc02, id(33)),
            (SessionSlot::Mc02, id(23)),
        ] {
            assert_eq!(box_a.read(&base, slot, process, &key), Err(OverlayError::WrongOwnerOrProcess));
            assert_eq!(box_a.write(&base, slot, process, key.clone(), b"poison"), Err(OverlayError::WrongOwnerOrProcess));
        }
        assert_eq!(SyntheticSlotOverlay::new(SessionSlot::Mc01, id(1)).unwrap_err(), OverlayError::HostSlotForbidden);
        assert_eq!(SyntheticSlotOverlay::new(SessionSlot::Mc02,
            ProcessIdentity { pid: 0, created_file_time: 1 }).unwrap_err(), OverlayError::InvalidProcess);
        assert_eq!(box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), Some(b"original".as_slice()));
    }

    #[test]
    fn rename_is_atomic_and_never_changes_original_host_entries() {
        let (base, mut a, _) = setup();
        let old = file("world/data.bin");
        a.rename(&base, SessionSlot::Mc02, id(22), &old, file("world/renamed.bin")).unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &old).unwrap(), None);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &file("world/renamed.bin")).unwrap(), Some(b"original".as_slice()));
        assert_eq!(base.fixture_bytes(&old), Some(b"original".as_slice()));
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22), &file("world/renamed.bin"), file("world/other.bin")), Err(OverlayError::AlreadyExists));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &file("world/renamed.bin")).unwrap(), Some(b"original".as_slice()));
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22), &file("world/renamed.bin"), reg("Software\\Game\\New")), Err(OverlayError::InvalidName));
    }

    #[test]
    fn rename_at_item_and_tombstone_capacity_moves_bytes_atomically() {
        let (base, mut a, _) = setup();
        for n in 0..MAX_OVERLAY_ITEMS {
            a.write(&base, SessionSlot::Mc02, id(22),
                file(&format!("quota/item_{n:04}.bin")), &[(n & 0xff) as u8]).unwrap();
        }
        let old = file("quota/item_0000.bin");
        let destination = file("quota/moved.bin");
        let original_buffer = a.private.get(&old).unwrap().as_ptr();
        let count_before = a.private_byte_count();
        a.rename(&base, SessionSlot::Mc02, id(22), &old, destination.clone()).unwrap();
        assert_eq!(a.private.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(a.private_byte_count(), count_before);
        assert_eq!(a.private.get(&destination).unwrap().as_ptr(), original_buffer);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &old).unwrap(), None);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &destination).unwrap(),
            Some(&[0][..]));

        // Host source may not be copied when the destination's quota is full.
        let host_source = file("world/data.bin");
        let host_destination = file("quota/from_host.bin");
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &host_source, host_destination.clone()), Err(OverlayError::CapacityExceeded));
        assert_eq!(a.private_byte_count(), count_before);
        assert_eq!(a.private.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &host_source).unwrap(),
            Some(b"original".as_slice()));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &host_destination).unwrap(), None);

        // Full tombstones must reject even a small rename without changing
        // its still-visible host source or creating a partial destination.
        let mut tombstoned = SyntheticSlotOverlay::new(SessionSlot::Mc04, id(44)).unwrap();
        for n in 0..MAX_OVERLAY_ITEMS {
            tombstoned.delete(SessionSlot::Mc04, id(44),
                file(&format!("removed/item_{n:04}.bin"))).unwrap();
        }
        let failed_destination = file("world/attempted.bin");
        assert_eq!(tombstoned.rename(&base, SessionSlot::Mc04, id(44),
            &host_source, failed_destination.clone()), Err(OverlayError::CapacityExceeded));
        assert_eq!(tombstoned.private.len(), 0);
        assert_eq!(tombstoned.tombstones.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(tombstoned.private_byte_count(), 0);
        assert_eq!(tombstoned.read(&base, SessionSlot::Mc04, id(44), &host_source).unwrap(),
            Some(b"original".as_slice()));
        assert_eq!(tombstoned.read(&base, SessionSlot::Mc04, id(44),
            &failed_destination).unwrap(), None);
    }

    #[test]
    fn subtree_delete_then_private_root_replacement_does_not_resurrect_host() {
        let (base, mut box_a, _) = setup();
        box_a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        box_a.write(&base, SessionSlot::Mc02, id(22), file("world"), b"private").unwrap();
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &file("world")).unwrap(),
            Some(b"private".as_slice()),
        );
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &file("world/data.bin")).unwrap(),
            None,
        );
    }

    #[test]
    fn partial_writes_and_truncation_copy_host_bytes_into_private_view() {
        let (base, mut box_a, box_b) = setup();
        let key = file("world/data.bin");
        box_a.write_at(&base, SessionSlot::Mc02, id(22), key.clone(), 3, b"XY").unwrap();
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"oriXYnal".as_slice()),
        );
        box_a.truncate_file(&base, SessionSlot::Mc02, id(22), key.clone(), 3).unwrap();
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"ori".as_slice()),
        );
        assert_eq!(
            box_b.read(&base, SessionSlot::Mc03, id(33), &key).unwrap(),
            Some(b"original".as_slice()),
        );
        box_a.write_at(&base, SessionSlot::Mc02, id(22), file("world/sparse.bin"), 3, b"x").unwrap();
        assert_eq!(
            box_a.read(&base, SessionSlot::Mc02, id(22), &file("world/sparse.bin")).unwrap(),
            Some(&[0, 0, 0, b'x'][..]),
        );
        assert_eq!(box_a.write_at(
            &base, SessionSlot::Mc02, id(22), key.clone(), usize::MAX, b"x",
        ), Err(OverlayError::EntryTooLarge));
        assert_eq!(box_a.truncate_file(
            &base, SessionSlot::Mc02, id(22), reg("Software\\Game\\Setting"), 1,
        ), Err(OverlayError::InvalidName));
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
    }

    #[test]
    fn private_only_policies_block_host_fallback_but_allow_own_private_values() {
        let (base, mut a, b) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("world")).unwrap();
        let key = file("WORLD/data.bin");
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), None);
        assert_eq!(a.copy_up(&base, SessionSlot::Mc02, id(22), &key),
            Err(OverlayError::MissingSource));
        assert_eq!(a.list(&base, SessionSlot::Mc02, id(22), &file("world")).unwrap().len(), 0);
        a.write(&base, SessionSlot::Mc02, id(22), key.clone(), b"private").unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"private".as_slice()));
        assert_eq!(a.list(&base, SessionSlot::Mc02, id(22), &file("world")).unwrap(),
            vec![file("world/data.bin")]);
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33), &key).unwrap(),
            Some(b"original".as_slice()));
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
    }

    #[test]
    fn private_only_file_policy_does_not_change_registry_namespace() {
        let (base, mut a, _) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22), &reg("Software\\Game\\Setting")).unwrap(),
            Some(b"base-setting".as_slice()),
        );
        a.make_private_only(SessionSlot::Mc02, id(22), reg("Software\\Game")).unwrap();
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22), &reg("Software\\Game\\Setting")).unwrap(),
            None,
        );
        assert_eq!(
            a.make_private_only(SessionSlot::Mc03, id(33), reg("Software\\Other")),
            Err(OverlayError::WrongOwnerOrProcess),
        );
    }

    #[test]
    fn late_private_only_rule_cannot_relabel_existing_base_copies() {
        let (base, mut a, _) = setup();
        let key = file("world/data.bin");
        a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        assert_eq!(
            a.make_private_only(SessionSlot::Mc02, id(22), file("world")),
            Err(OverlayError::AmbiguousPrivateProvenance),
        );
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"original".as_slice()),
        );
    }

    #[test]
    fn host_rename_cannot_smuggle_bytes_across_private_only_boundary() {
        let (base, mut a, _) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("isolated")).unwrap();
        assert_eq!(
            a.rename(&base, SessionSlot::Mc02, id(22),
                &file("world/data.bin"), file("isolated/copied.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary),
        );
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22),
                &file("isolated/copied.bin")).unwrap(),
            None,
        );
        assert_eq!(base.fixture_bytes(&file("world/data.bin")),
            Some(b"original".as_slice()));
        a.write(&base, SessionSlot::Mc02, id(22), file("world/private.bin"), b"mine").unwrap();
        a.rename(&base, SessionSlot::Mc02, id(22),
            &file("world/private.bin"), file("isolated/renamed.bin")).unwrap();
        assert_eq!(
            a.read(&base, SessionSlot::Mc02, id(22),
                &file("isolated/renamed.bin")).unwrap(),
            Some(b"mine".as_slice()),
        );
    }

    #[test]
    fn copied_host_bytes_cannot_be_renamed_into_private_only_prefix() {
        let (base, mut a, _) = setup();
        let key = file("world/data.bin");
        a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        a.make_private_only(SessionSlot::Mc02, id(22), file("isolated")).unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("isolated/copied.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"original".as_slice()));
        a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("world/moved.bin")).unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &file("world/moved.bin"), file("isolated/copied.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
        // An actual newly authored full overwrite clears synthetic provenance.
        a.write(&base, SessionSlot::Mc02, id(22), file("world/moved.bin"), b"new").unwrap();
        a.rename(&base, SessionSlot::Mc02, id(22),
            &file("world/moved.bin"), file("isolated/new.bin")).unwrap();
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
    }

    #[test]
    fn host_derived_partial_write_remains_tagged_until_full_replacement() {
        let (base, mut a, _) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("isolated")).unwrap();
        let key = file("world/data.bin");
        a.write_at(&base, SessionSlot::Mc02, id(22), key.clone(), 1, b"x").unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("isolated/no.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
        a.truncate_file(&base, SessionSlot::Mc02, id(22), key.clone(), 2).unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("isolated/no.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
    }

    #[test]
    fn subtree_rename_preserves_copy_on_write_and_moves_private_vectors() {
        let (base, mut a, b) = setup();
        let own = file("world/my-new.bin");
        let changed = file("world/data.bin");
        a.write_at(&base, SessionSlot::Mc02, id(22), changed.clone(), 0, b"X").unwrap();
        a.write(&base, SessionSlot::Mc02, id(22), own.clone(), b"mine").unwrap();
        let original_private_allocation = a.private.get(&own).unwrap().as_ptr();
        a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &file("world"), file("archive/mc02")).unwrap();
        let destination = file("archive/mc02/my-new.bin");
        assert_eq!(a.private.get(&destination).unwrap().as_ptr(),
            original_private_allocation);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("archive/mc02/data.bin")).unwrap(), Some(b"Xriginal".as_slice()));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("archive/mc02/other.bin")).unwrap(), Some(b"second".as_slice()));
        assert!(a.host_derived.contains(&file("archive/mc02/data.bin")));
        assert!(a.host_derived.contains(&file("archive/mc02/other.bin")));
        assert!(!a.host_derived.contains(&destination));
        assert_eq!(a.private_byte_count(), b"Xriginal".len() + b"second".len() + 4);
        assert!(a.list(&base, SessionSlot::Mc02, id(22),
            &file("world")).unwrap().is_empty());
        assert_eq!(base.fixture_bytes(&changed), Some(b"original".as_slice()));
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33),
            &changed).unwrap(), Some(b"original".as_slice()));
        assert!(a.tombstones.contains(&file("world")));
    }

    #[test]
    fn subtree_rename_refuses_overlap_collision_and_invalid_paths_atomically() {
        let (mut base, mut a, _) = setup();
        base.insert_fixture(file("destination/existing.bin"), b"exists").unwrap();
        a.write(&base, SessionSlot::Mc02, id(22), file("world/new.bin"), b"own").unwrap();
        let old_private = a.private.clone();
        let old_tags = a.host_derived.clone();
        let old_tombstones = a.tombstones.clone();
        let old_bytes = a.private_byte_count();
        let too_deep = file(&(0..32).map(|_| "a").collect::<Vec<_>>().join("/"));
        for (from, target, expected) in [
            (file("world"), file("destination"), OverlayError::AlreadyExists),
            (file("world"), file("world/child"), OverlayError::InvalidName),
            (file("world/child"), file("world"), OverlayError::MissingSource),
            (file("world"), too_deep, OverlayError::InvalidName),
            (file("world"), reg("software\\world"), OverlayError::InvalidName),
        ] {
            assert_eq!(a.rename_tree(&base, SessionSlot::Mc02, id(22),
                &from, target), Err(expected));
            assert_eq!(a.private, old_private);
            assert_eq!(a.host_derived, old_tags);
            assert_eq!(a.tombstones, old_tombstones);
            assert_eq!(a.private_byte_count(), old_bytes);
        }
        assert_eq!(a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &file("world"), file("world")), Ok(()));
        assert_eq!(a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &file("absent"), file("absent")), Err(OverlayError::MissingSource));
        assert_eq!(a.rename_tree(&base, SessionSlot::Mc03, id(33),
            &file("world"), file("destination")), Err(OverlayError::WrongOwnerOrProcess));
    }

    #[test]
    fn subtree_rename_observes_item_byte_and_tombstone_quotas() {
        let (base, _, _) = setup();
        let mut items = SyntheticSlotOverlay::new(SessionSlot::Mc04, id(44)).unwrap();
        for n in 0..MAX_OVERLAY_ITEMS {
            items.write(&base, SessionSlot::Mc04, id(44),
                file(&format!("quota/key_{n:04}")), b"x").unwrap();
        }
        assert_eq!(items.rename_tree(&base, SessionSlot::Mc04, id(44),
            &file("world"), file("archive")),
            Err(OverlayError::CapacityExceeded));
        assert_eq!(items.private.len(), MAX_OVERLAY_ITEMS);
        assert!(items.tombstones.is_empty());
        assert_eq!(items.private_byte_count(), MAX_OVERLAY_ITEMS);

        let mut bytes = SyntheticSlotOverlay::new(SessionSlot::Mc04, id(44)).unwrap();
        let max_item = vec![7u8; MAX_OVERLAY_ITEM_BYTES];
        for n in 0..MAX_OVERLAY_PRIVATE_BYTES / MAX_OVERLAY_ITEM_BYTES {
            bytes.write(&base, SessionSlot::Mc04, id(44),
                file(&format!("size/key_{n:04}")), &max_item).unwrap();
        }
        assert_eq!(bytes.rename_tree(&base, SessionSlot::Mc04, id(44),
            &file("world"), file("archive")),
            Err(OverlayError::CapacityExceeded));
        assert_eq!(bytes.private_byte_count(), MAX_OVERLAY_PRIVATE_BYTES);
        assert_eq!(bytes.private.len(), MAX_OVERLAY_PRIVATE_BYTES / MAX_OVERLAY_ITEM_BYTES);
        assert!(bytes.tombstones.is_empty());

        let mut tombstoned = SyntheticSlotOverlay::new(SessionSlot::Mc04, id(44)).unwrap();
        for n in 0..MAX_OVERLAY_ITEMS {
            tombstoned.delete(SessionSlot::Mc04, id(44),
                file(&format!("removed/leaf_{n:04}"))).unwrap();
        }
        assert_eq!(tombstoned.rename_tree(&base, SessionSlot::Mc04, id(44),
            &file("world"), file("archive")),
            Err(OverlayError::CapacityExceeded));
        assert_eq!(tombstoned.tombstones.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(tombstoned.private_byte_count(), 0);
        assert_eq!(tombstoned.read(&base, SessionSlot::Mc04, id(44),
            &file("world/data.bin")).unwrap(), Some(b"original".as_slice()));
    }

    #[test]
    fn subtree_rename_coalesces_deletes_without_resurrecting_host() {
        let (base, mut a, _) = setup();
        a.delete(SessionSlot::Mc02, id(22), file("world/other.bin")).unwrap();
        a.delete(SessionSlot::Mc02, id(22), file("world/deleted.bin")).unwrap();
        a.write(&base, SessionSlot::Mc02, id(22), file("world/private.bin"), b"private").unwrap();
        a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &file("world"), file("archive")).unwrap();
        assert!(a.tombstones.contains(&file("world")));
        assert!(!a.tombstones.contains(&file("world/other.bin")));
        assert!(!a.tombstones.contains(&file("world/deleted.bin")));
        assert!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/other.bin")).unwrap().is_none());
        assert!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("archive/other.bin")).unwrap().is_none());
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("archive/data.bin")).unwrap(), Some(b"original".as_slice()));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("archive/private.bin")).unwrap(), Some(b"private".as_slice()));
        // Exact-prefix tombstone and private data inside it are both legal:
        // the new prefix is masked from host, not from private leaves.
        a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &file("archive"), file("renamed")).unwrap();
        assert!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("archive/data.bin")).unwrap().is_none());
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("renamed/private.bin")).unwrap(), Some(b"private".as_slice()));
    }

    #[test]
    fn subtree_rename_preserves_host_provenance_into_private_only_namespace() {
        let (base, mut a, _) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("private")).unwrap();
        assert_eq!(a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &file("world"), file("private/world")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
        assert!(a.tombstones.is_empty());
        assert_eq!(a.private_byte_count(), 0);
        a.write(&base, SessionSlot::Mc02, id(22), file("own/profile.bin"), b"new").unwrap();
        a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &file("own"), file("private/own")).unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("private/own/profile.bin")).unwrap(), Some(b"new".as_slice()));
        assert!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/data.bin")).unwrap().is_some());
    }

    #[test]
    fn full_tombstone_capacity_can_coalesce_descendant_deletes() {
        let (base, mut a, b) = setup();
        for n in 0..MAX_OVERLAY_ITEMS {
            a.delete(SessionSlot::Mc02, id(22),
                file(&format!("world/deleted_{n:04}.bin"))).unwrap();
        }
        a.write(&base, SessionSlot::Mc02, id(22),
            file("world/new-private.bin"), b"private").unwrap();
        assert_eq!(a.tombstones.len(), MAX_OVERLAY_ITEMS);
        a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(a.tombstones.len(), 1);
        assert!(a.tombstones.contains(&file("world")));
        assert!(a.private.is_empty());
        assert_eq!(a.private_byte_count(), 0);
        a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(a.tombstones.len(), 1);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/data.bin")).unwrap(), None);
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33),
            &file("world/data.bin")).unwrap(), Some(b"original".as_slice()));
    }

    #[test]
    fn full_tombstone_capacity_reuses_parent_mask_for_leaf_delete_and_rename() {
        let (base, mut a, _) = setup();
        for n in 0..MAX_OVERLAY_ITEMS - 1 {
            a.delete(SessionSlot::Mc02, id(22),
                file(&format!("other/item_{n:04}.bin"))).unwrap();
        }
        a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(a.tombstones.len(), MAX_OVERLAY_ITEMS);
        a.write(&base, SessionSlot::Mc02, id(22), file("world/keep.bin"), b"hello").unwrap();
        a.rename(&base, SessionSlot::Mc02, id(22),
            &file("world/keep.bin"), file("archive/keep.bin")).unwrap();
        assert_eq!(a.tombstones.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("archive/keep.bin")).unwrap(), Some(b"hello".as_slice()));
        a.write(&base, SessionSlot::Mc02, id(22), file("world/remove.bin"), b"temp").unwrap();
        a.delete(SessionSlot::Mc02, id(22), file("world/remove.bin")).unwrap();
        assert_eq!(a.tombstones.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/remove.bin")).unwrap(), None);
        let before = a.private.clone();
        assert_eq!(a.delete(SessionSlot::Mc02, id(22), file("different/new.bin")),
            Err(OverlayError::CapacityExceeded));
        assert_eq!(a.private, before);
        assert_eq!(a.tombstones.len(), MAX_OVERLAY_ITEMS);
    }

    #[test]
    fn private_partial_write_and_truncate_keep_allocation_and_provenance() {
        let (base, mut a, _) = setup();
        let key = file("world/custom.bin");
        a.write(&base, SessionSlot::Mc02, id(22), key.clone(), b"abcdefgh").unwrap();
        let ptr = a.private.get(&key).unwrap().as_ptr();
        a.write_at(&base, SessionSlot::Mc02, id(22), key.clone(),
            2, b"XY").unwrap();
        assert_eq!(a.private.get(&key).unwrap().as_ptr(), ptr);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &key).unwrap(), Some(b"abXYefgh".as_slice()));
        a.truncate_file(&base, SessionSlot::Mc02, id(22),
            key.clone(), 4).unwrap();
        assert_eq!(a.private.get(&key).unwrap().as_ptr(), ptr);
        assert_eq!(a.private_byte_count(), 4);
        assert!(!a.host_derived.contains(&key));
        let host_key = file("world/data.bin");
        a.write_at(&base, SessionSlot::Mc02, id(22), host_key.clone(),
            0, b"Z").unwrap();
        assert!(a.host_derived.contains(&host_key));
        a.truncate_file(&base, SessionSlot::Mc02, id(22),
            host_key.clone(), 2).unwrap();
        assert!(a.host_derived.contains(&host_key));
        assert_eq!(base.fixture_bytes(&host_key), Some(b"original".as_slice()));
        a.write(&base, SessionSlot::Mc02, id(22), host_key.clone(), b"new").unwrap();
        assert!(!a.host_derived.contains(&host_key));
    }

    #[test]
    fn partial_write_and_truncate_quota_denials_leave_private_state_unchanged() {
        let (base, mut a, _) = setup();
        let max_item = vec![5u8; MAX_OVERLAY_ITEM_BYTES];
        for n in 0..MAX_OVERLAY_PRIVATE_BYTES / MAX_OVERLAY_ITEM_BYTES {
            a.write(&base, SessionSlot::Mc02, id(22),
                file(&format!("quota/item_{n:04}.bin")), &max_item).unwrap();
        }
        let existing = file("quota/item_0000.bin");
        let new = file("quota/new.bin");
        let ptr = a.private.get(&existing).unwrap().as_ptr();
        let before = a.private_byte_count();
        assert_eq!(a.write_at(&base, SessionSlot::Mc02, id(22),
            new.clone(), 0, b"x"), Err(OverlayError::CapacityExceeded));
        assert_eq!(a.truncate_file(&base, SessionSlot::Mc02, id(22),
            file("world/data.bin"), 1), Err(OverlayError::CapacityExceeded));
        assert_eq!(a.write_at(&base, SessionSlot::Mc02, id(22),
            existing.clone(), MAX_OVERLAY_ITEM_BYTES, b"x"),
            Err(OverlayError::EntryTooLarge));
        assert_eq!(a.truncate_file(&base, SessionSlot::Mc02, id(22),
            existing.clone(), MAX_OVERLAY_ITEM_BYTES + 1),
            Err(OverlayError::EntryTooLarge));
        assert_eq!(a.private_byte_count(), before);
        assert!(a.private.get(&new).is_none());
        assert_eq!(a.private.get(&existing).unwrap().as_ptr(), ptr);
        assert_eq!(a.private.get(&existing).unwrap().as_slice(), max_item.as_slice());
        assert!(a.host_derived.is_empty());
        a.write_at(&base, SessionSlot::Mc02, id(22),
            existing.clone(), 0, b"Q").unwrap();
        assert_eq!(a.private.get(&existing).unwrap().as_ptr(), ptr);
        assert_eq!(a.private_byte_count(), before);
        assert_eq!(a.private.get(&existing).unwrap()[0], b'Q');
    }

    #[test]
    fn tombstone_compaction_preserves_registry_and_unrelated_masks() {
        let (base, mut a, _) = setup();
        a.delete(SessionSlot::Mc02, id(22), file("world/one.bin")).unwrap();
        a.delete(SessionSlot::Mc02, id(22), file("world/two.bin")).unwrap();
        a.delete(SessionSlot::Mc02, id(22), reg("Software\\Game\\Setting")).unwrap();
        a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(a.tombstones.len(), 2);
        assert!(a.tombstones.contains(&file("world")));
        assert!(a.tombstones.contains(&reg("Software\\Game\\Setting")));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &reg("Software\\Game\\Setting")).unwrap(), None);
        a.write(&base, SessionSlot::Mc02, id(22), file("world/new.bin"), b"own").unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/new.bin")).unwrap(), Some(b"own".as_slice()));
    }

    #[test]
    fn private_only_rules_compact_full_child_set_without_hiding_other_surfaces() {
        let (mut base, mut a, b) = setup();
        base.insert_fixture(file("worldwide/data.bin"), b"unrelated").unwrap();
        for i in 0..MAX_OVERLAY_ITEMS {
            a.make_private_only(SessionSlot::Mc02, id(22),
                file(&format!("world/branch_{i:04}"))).unwrap();
        }
        assert_eq!(a.private_only_prefixes.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/data.bin")).unwrap(), Some(b"original".as_slice()));
        a.make_private_only(SessionSlot::Mc02, id(22), file("world")).unwrap();
        assert_eq!(a.private_only_prefixes.len(), 1);
        assert!(a.private_only_prefixes.contains(&file("world")));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/data.bin")).unwrap(), None);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("worldwide/data.bin")).unwrap(), Some(b"unrelated".as_slice()));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &reg("Software\\Game\\Setting")).unwrap(), Some(b"base-setting".as_slice()));
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33),
            &file("world/data.bin")).unwrap(), Some(b"original".as_slice()));
    }

    #[test]
    fn redundant_private_only_child_at_capacity_is_noop_even_with_private_data() {
        let (base, mut a, _) = setup();
        a.make_private_only(SessionSlot::Mc02, id(22), file("vault")).unwrap();
        for i in 0..MAX_OVERLAY_ITEMS - 1 {
            a.make_private_only(SessionSlot::Mc02, id(22),
                file(&format!("elsewhere/rule_{i:04}"))).unwrap();
        }
        let key = file("vault/own.bin");
        a.write(&base, SessionSlot::Mc02, id(22), key.clone(), b"owned").unwrap();
        let old = a.private_only_prefixes.clone();
        assert_eq!(old.len(), MAX_OVERLAY_ITEMS);
        a.make_private_only(SessionSlot::Mc02, id(22), key.clone()).unwrap();
        a.make_private_only(SessionSlot::Mc02, id(22), file("vault")).unwrap();
        assert_eq!(a.private_only_prefixes, old);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"owned".as_slice()));
        assert_eq!(a.make_private_only(SessionSlot::Mc02, id(22),
            file("new-boundary")), Err(OverlayError::CapacityExceeded));
        assert_eq!(a.private_only_prefixes, old);
        assert_eq!(a.make_private_only(SessionSlot::Mc02, id(22),
            file("elsewhere")), Ok(()));
        assert_eq!(a.private_only_prefixes.len(), 2);
        assert!(a.private_only_prefixes.contains(&file("vault")));
        assert!(a.private_only_prefixes.contains(&file("elsewhere")));
    }

    #[test]
    fn wider_private_only_policy_rejects_preexisting_private_bytes_atomically() {
        let (base, mut a, _) = setup();
        let rule = file("world/hidden");
        let own = file("world/own.bin");
        a.make_private_only(SessionSlot::Mc02, id(22), rule.clone()).unwrap();
        a.write(&base, SessionSlot::Mc02, id(22), own.clone(), b"owned").unwrap();
        let before = a.private_only_prefixes.clone();
        assert_eq!(a.make_private_only(SessionSlot::Mc02, id(22),
            file("world")), Err(OverlayError::AmbiguousPrivateProvenance));
        assert_eq!(a.private_only_prefixes, before);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &own).unwrap(),
            Some(b"owned".as_slice()));
        assert!(a.host_derived.is_empty());
    }

    #[test]
    fn leaf_rename_rejects_hierarchical_collisions_without_changing_state() {
        let (base, mut a, _) = setup();
        let from = file("world/other.bin");
        a.write(&base, SessionSlot::Mc02, id(22), file("archive/descendant.bin"),
            b"private").unwrap();
        a.write(&base, SessionSlot::Mc02, id(22), file("own"), b"private").unwrap();
        // Production writes must reject this contradictory file shape.
        assert_eq!(a.write(&base, SessionSlot::Mc02, id(22),
            file("own/child"), b"child"), Err(OverlayError::AlreadyExists));
        // Deliberately corrupt the private test fixture to exercise rename's
        // independent fail-closed check against pre-existing invalid state.
        a.private.insert(file("own/child"), b"child".to_vec());
        a.private_bytes += b"child".len();
        let existing = a.private.clone();
        let bytes = a.private_byte_count();
        let tags = a.host_derived.clone();
        let masks = a.tombstones.clone();
        for (old, new, expected) in [
            (from.clone(), file("world/data.bin/child"), OverlayError::AlreadyExists),
            (from.clone(), file("archive"), OverlayError::AlreadyExists),
            (from.clone(), file("world"), OverlayError::InvalidName),
            (file("world/data.bin"), file("world/data.bin/child"),
                OverlayError::InvalidName),
            (file("own"), file("separate"), OverlayError::InvalidName),
            (file("own/child"), file("own"), OverlayError::InvalidName),
        ] {
            assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
                &old, new), Err(expected));
            assert_eq!(a.private, existing);
            assert_eq!(a.private_byte_count(), bytes);
            assert_eq!(a.host_derived, tags);
            assert_eq!(a.tombstones, masks);
        }
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &from).unwrap(), Some(b"second".as_slice()));
    }

    #[test]
    fn subtree_rename_rejects_file_ancestors_and_mixed_leaf_directory_shape() {
        let (base, mut a, _) = setup();
        let from = file("move");
        a.write(&base, SessionSlot::Mc02, id(22), file("move/one.bin"),
            b"one").unwrap();
        assert_eq!(a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &from, file("world/data.bin/subdir")),
            Err(OverlayError::AlreadyExists));
        assert_eq!(a.write(&base, SessionSlot::Mc02, id(22),
            from.clone(), b"leaf"), Err(OverlayError::AlreadyExists));
        // Corrupt a disposable synthetic test fixture to verify rename_tree
        // itself still rejects a mixed file-leaf + directory shape.
        a.private.insert(from.clone(), b"leaf".to_vec());
        a.private_bytes += b"leaf".len();
        let original = a.private.clone();
        let masks = a.tombstones.clone();
        assert_eq!(a.rename_tree(&base, SessionSlot::Mc02, id(22),
            &from, file("archive/new")),
            Err(OverlayError::InvalidName));
        assert_eq!(a.private, original);
        assert_eq!(a.tombstones, masks);
        assert_eq!(a.private_byte_count(), b"one".len() + b"leaf".len());
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/data.bin")).unwrap(), Some(b"original".as_slice()));
    }

    #[test]
    fn copy_up_is_idempotent_and_tracks_file_registry_host_provenance() {
        let (base, mut a, b) = setup();
        let key = file("world/data.bin");
        a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        let ptr = a.private.get(&key).unwrap().as_ptr();
        a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        assert_eq!(a.private.get(&key).unwrap().as_ptr(), ptr);
        assert_eq!(a.private_byte_count(), b"original".len());
        assert!(a.host_derived.contains(&key));
        let reg_key = reg("Software\\Game\\Setting");
        a.copy_up(&base, SessionSlot::Mc02, id(22), &reg_key).unwrap();
        assert!(a.host_derived.contains(&reg_key));
        assert_eq!(a.private_byte_count(), b"original".len() + b"base-setting".len());
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33), &reg_key).unwrap(),
            Some(b"base-setting".as_slice()));
        assert_eq!(base.fixture_bytes(&key), Some(b"original".as_slice()));
        a.make_private_only(SessionSlot::Mc02, id(22), file("vault")).unwrap();
        assert_eq!(a.rename(&base, SessionSlot::Mc02, id(22),
            &key, file("vault/copied.bin")),
            Err(OverlayError::HostCopyIntoPrivateBoundary));
        a.write(&base, SessionSlot::Mc02, id(22), key.clone(), b"fresh").unwrap();
        assert!(!a.host_derived.contains(&key));
        a.copy_up(&base, SessionSlot::Mc02, id(22), &key).unwrap();
        assert!(!a.host_derived.contains(&key));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(),
            Some(b"fresh".as_slice()));
    }

    #[test]
    fn copy_up_quota_and_hidden_host_denials_leave_state_unchanged() {
        let (base, mut a, _) = setup();
        let host_key = file("world/data.bin");
        a.delete(SessionSlot::Mc02, id(22), host_key.clone()).unwrap();
        assert_eq!(a.copy_up(&base, SessionSlot::Mc02, id(22), &host_key),
            Err(OverlayError::MissingSource));
        assert!(a.private.is_empty());
        assert!(a.host_derived.is_empty());
        assert_eq!(a.private_byte_count(), 0);
        assert!(a.tombstones.contains(&host_key));
        let mut full_items = SyntheticSlotOverlay::new(SessionSlot::Mc04, id(44)).unwrap();
        for n in 0..MAX_OVERLAY_ITEMS {
            full_items.write(&base, SessionSlot::Mc04, id(44),
                file(&format!("quota/item_{n:04}")), b"x").unwrap();
        }
        let ptr = full_items.private.get(&file("quota/item_0000")).unwrap().as_ptr();
        assert_eq!(full_items.copy_up(&base, SessionSlot::Mc04, id(44), &host_key),
            Err(OverlayError::CapacityExceeded));
        assert_eq!(full_items.private.len(), MAX_OVERLAY_ITEMS);
        assert_eq!(full_items.private_byte_count(), MAX_OVERLAY_ITEMS);
        assert_eq!(full_items.private.get(&file("quota/item_0000")).unwrap().as_ptr(), ptr);
        assert!(full_items.host_derived.is_empty());
        let mut full_bytes = SyntheticSlotOverlay::new(SessionSlot::Mc04, id(44)).unwrap();
        let data = vec![8u8; MAX_OVERLAY_ITEM_BYTES];
        for n in 0..MAX_OVERLAY_PRIVATE_BYTES / MAX_OVERLAY_ITEM_BYTES {
            full_bytes.write(&base, SessionSlot::Mc04, id(44),
                file(&format!("quota/data_{n:04}")), &data).unwrap();
        }
        assert_eq!(full_bytes.copy_up(&base, SessionSlot::Mc04, id(44), &host_key),
            Err(OverlayError::CapacityExceeded));
        assert_eq!(full_bytes.private_byte_count(), MAX_OVERLAY_PRIVATE_BYTES);
        assert!(full_bytes.host_derived.is_empty());
        assert_eq!(base.fixture_bytes(&host_key), Some(b"original".as_slice()));
    }

    #[test]
    fn empty_partial_write_never_grows_or_materializes_host_data() {
        let (base, mut a, b) = setup();
        let host_key = file("world/data.bin");
        let absent = file("world/absent.bin");
        a.write_at(&base, SessionSlot::Mc02, id(22),
            host_key.clone(), MAX_OVERLAY_ITEM_BYTES, b"").unwrap();
        a.write_at(&base, SessionSlot::Mc02, id(22),
            absent.clone(), 0, b"").unwrap();
        assert!(a.private.is_empty());
        assert!(a.host_derived.is_empty());
        assert_eq!(a.private_byte_count(), 0);
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &host_key).unwrap(), Some(b"original".as_slice()));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &absent).unwrap(), None);
        let key = file("world/owned.bin");
        a.write(&base, SessionSlot::Mc02, id(22), key.clone(), b"private").unwrap();
        let ptr = a.private.get(&key).unwrap().as_ptr();
        a.write_at(&base, SessionSlot::Mc02, id(22),
            key.clone(), MAX_OVERLAY_ITEM_BYTES, b"").unwrap();
        assert_eq!(a.private.get(&key).unwrap().as_ptr(), ptr);
        assert_eq!(a.private.get(&key).unwrap().as_slice(), b"private");
        assert_eq!(a.private_byte_count(), b"private".len());
        assert_eq!(a.write_at(&base, SessionSlot::Mc02, id(22),
            absent.clone(), MAX_OVERLAY_ITEM_BYTES + 1, b""),
            Err(OverlayError::EntryTooLarge));
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33), &host_key).unwrap(),
            Some(b"original".as_slice()));
    }

    #[test]
    fn baseline_file_insertion_rejects_parent_child_collisions_in_both_orders() {
        let mut host = SyntheticBaseView::default();
        host.insert_fixture(file("tree/leaf.bin"), b"leaf").unwrap();
        let initial_bytes = host.bytes;
        assert_eq!(host.insert_fixture(file("tree"), b"parent"),
            Err(OverlayError::AlreadyExists));
        assert_eq!(host.insert_fixture(file("tree/leaf.bin/child"), b"child"),
            Err(OverlayError::AlreadyExists));
        assert_eq!(host.bytes, initial_bytes);
        assert_eq!(host.entries.len(), 1);
        host.insert_fixture(file("tree/sibling.bin"), b"ok").unwrap();

        let mut reverse = SyntheticBaseView::default();
        reverse.insert_fixture(file("file"), b"parent").unwrap();
        assert_eq!(reverse.insert_fixture(file("file/child"), b"x"),
            Err(OverlayError::AlreadyExists));
        assert_eq!(reverse.entries.len(), 1);
        // RegistryValue names intentionally stay flat synthetic values.
        reverse.insert_fixture(reg("Software\\Game"), b"one").unwrap();
        reverse.insert_fixture(reg("Software\\Game\\Value"), b"two").unwrap();
        assert_eq!(reverse.fixture_bytes(&reg("Software\\Game\\Value")),
            Some(b"two".as_slice()));
    }

    #[test]
    fn full_write_refuses_visible_host_or_private_file_ancestry_atomically() {
        let (base, mut a, b) = setup();
        let initial = a.private.clone();
        assert_eq!(a.write(&base, SessionSlot::Mc02, id(22),
            file("world"), b"file"), Err(OverlayError::AlreadyExists));
        assert_eq!(a.write(&base, SessionSlot::Mc02, id(22),
            file("world/data.bin/child"), b"x"), Err(OverlayError::AlreadyExists));
        assert_eq!(a.private, initial);
        assert_eq!(a.private_byte_count(), 0);
        assert!(a.host_derived.is_empty());
        assert!(a.tombstones.is_empty());

        a.write(&base, SessionSlot::Mc02, id(22),
            file("independent/root"), b"private").unwrap();
        let ptr = a.private.get(&file("independent/root")).unwrap().as_ptr();
        assert_eq!(a.write(&base, SessionSlot::Mc02, id(22),
            file("independent/root/child"), b"x"), Err(OverlayError::AlreadyExists));
        assert_eq!(a.private.get(&file("independent/root")).unwrap().as_ptr(), ptr);
        assert_eq!(a.private_byte_count(), b"private".len());
        assert_eq!(b.read(&base, SessionSlot::Mc03, id(33),
            &file("world/data.bin")).unwrap(), Some(b"original".as_slice()));

        // A prefix tombstone legitimately hides the OLD host descendants.
        a.delete_tree(SessionSlot::Mc02, id(22), file("world")).unwrap();
        a.write(&base, SessionSlot::Mc02, id(22), file("world"), b"replacement").unwrap();
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world")).unwrap(), Some(b"replacement".as_slice()));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22),
            &file("world/data.bin")).unwrap(), None);
    }

    #[test]
    fn host_bound_partial_write_and_copy_up_cannot_bypass_file_parent() {
        let (base, mut a, _) = setup();
        a.write(&base, SessionSlot::Mc02, id(22),
            file("scratch"), b"parent").unwrap();
        assert_eq!(a.write_at(&base, SessionSlot::Mc02, id(22),
            file("scratch/child"), 0, b"x"), Err(OverlayError::AlreadyExists));
        assert_eq!(a.truncate_file(&base, SessionSlot::Mc02, id(22),
            file("scratch/child"), 1), Err(OverlayError::MissingSource));
        assert!(a.private.get(&file("scratch/child")).is_none());
        assert_eq!(a.private_byte_count(), b"parent".len());

        // Deliberately insert a pre-existing corrupt private file parent;
        // public full writes are not allowed to create this state.
        a.private.insert(file("world"), b"synthetic-corruption".to_vec());
        a.private_bytes += b"synthetic-corruption".len();
        assert_eq!(a.copy_up(&base, SessionSlot::Mc02, id(22),
            &file("world/data.bin")), Err(OverlayError::AlreadyExists));
        assert!(!a.host_derived.contains(&file("world/data.bin")));
        assert!(a.private.get(&file("world/data.bin")).is_none());
        assert_eq!(base.fixture_bytes(&file("world/data.bin")),
            Some(b"original".as_slice()));
    }

    #[test]
    fn quotas_and_repeated_replacements_preserve_private_state_on_failure() {
        let (base, mut a, _) = setup();
        let key = file("world/new.bin");
        a.write(&base, SessionSlot::Mc02, id(22), key.clone(), b"small").unwrap();
        assert_eq!(a.write(&base, SessionSlot::Mc02, id(22), key.clone(), &vec![1; MAX_OVERLAY_ITEM_BYTES + 1]), Err(OverlayError::EntryTooLarge));
        assert_eq!(a.read(&base, SessionSlot::Mc02, id(22), &key).unwrap(), Some(b"small".as_slice()));
        assert_eq!(a.private_byte_count(), 5);
    }
}
