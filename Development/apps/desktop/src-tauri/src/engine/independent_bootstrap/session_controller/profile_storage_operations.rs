//! Storage admission, reports and mutations on the existing retained Sessions owner.
//! No separate actor, worker, profile registry or cached mutation admission.
use super::Sessions;
use super::super::{
    directory_id, directory_path, game_preset, invalid, open_directory,
    product_profiles, profile_storage, resource_pack_inventory, same_image, wide, SlotStorage,
};
use crate::engine::{
    native_clients_session::{ProfileStorage, SelectiveCategory, SelectiveInventory,
        SelectiveInventoryEntry, SelectiveReviewReceipt, SelectiveSelection},
    process_identity::OwnedProcess,
};
use std::{fs::File, io, ptr, time::Duration};
use virtual_clients_core::InspectionBudget;
use windows_sys::Win32::Storage::FileSystem::FILE_READ_ATTRIBUTES;

// All affected operations use the original pending-clear policy; feature-off
// builds still do not inspect a product cleanup journal here.
fn require_no_pending_profile_clear(_store: &SlotStorage) -> io::Result<()> {
    #[cfg(feature="lazy-tester-package")]
    if product_profiles::clear::pending(_store)? {
        return Err(invalid("Finish pending profile cleanup first"));
    }
    Ok(())
}

impl Sessions {
    pub(super) fn inspect_storage(&self, slots: std::ops::RangeInclusive<u32>) -> io::Result<Vec<ProfileStorage>> {
        // One cooperative traversal budget for this *entire* read-only request,
        // not two seconds independently for each of five profiles and caches.
        // Native OS calls and the separate clear-confirmation inspection can
        // still block; this is not a hard latency guarantee or a new worker.
        let budget = InspectionBudget::new(Duration::from_secs(2));
        let observed = super::super::super::inspect_native_workspace()?.processes;
        // One report-local observation of the original retained handles.
        // Do not carry this result into a later request or mutation.
        let ownership = self.owned_data_processes()
            .map(|owned| all_observed_processes_owned(&owned, &observed));
        slots
            .map(|slot| {
                let store = self.store(slot)?;
                let path = directory_path(&store.private)?;
                // Only read-only reports share this admission observation.
                // Cache deletion, profile clear, launch and settings writes each
                // recheck their own authority at the moment of mutation.
                let admission = report_data_admission(
                    self.running[slot as usize - 1].is_none(), &ownership);
                let replay_admission = || {
                    admission.as_ref().map(|_| ()).map_err(|error| {
                        io::Error::new(error.kind(), error.to_string())
                    })
                };
                let full_clear = product_profiles::clear::inspect(store, replay_admission());
                let mut result = ProfileStorage {
                    slot,
                    bytes: 0,
                    files: 0,
                    partial: false,
                    disk: profile_storage::disk_at_verified_path(&path).ok(),
                    cache: {
                        let mut cache = profile_storage::inspect(&store.private, admission.is_ok(), budget);
                        if !cache.partial {
                            if let Err(error) = &admission { cache.issue = Some(error.to_string()); }
                        }
                        cache
                    },
                    download_cache: Some(profile_storage::inspect_cache(&store.private, profile_storage::CacheTarget::Download, admission.is_ok(), budget)),
                    // The inventory reads saved world *pack declaration metadata*.
                    // Do not inspect a selected profile while its own Minecraft
                    // session is active or the admission result is uncertain.
                    resource_packs: admission.is_ok().then(||
                        resource_pack_inventory::inspect(&store.private, budget)),
                    game_settings: Some(game_preset::inspect_settings(&store.private, replay_admission())),
                    full_clear: Some(full_clear),
                };
                // A pending full clear is exclusive with either cache action.
                // Preserve observed size/partial metadata; only revoke admission.
                if result.full_clear.as_ref().is_some_and(|report| report.pending) {
                    for cache in std::iter::once(&mut result.cache).chain(result.download_cache.iter_mut()) {
                        cache.can_clear = false;
                        if !cache.partial {
                            cache.issue = Some("Finish pending profile cleanup first".into());
                        }
                    }
                }
                profile_storage::scan_profile_size(&store.private, &mut result, budget);
                Ok(result)
            })
            .collect()
    }

    pub(super) fn open_folder(&self, slot: u32) -> io::Result<()> {
        let store = self.store(slot)?;
        let path = directory_path(&store.private)?;
        // Keep the validated directory pinned while opening its canonical location.
        let pinned = open_directory(&path, FILE_READ_ATTRIBUTES)?;
        if !same_image(&directory_id(&pinned)?, &directory_id(&store.private)?) {
            return Err(invalid("Profile directory changed"));
        }
        let path = wide(path.as_os_str())?;
        let outcome = unsafe {
            windows_sys::Win32::UI::Shell::ShellExecuteW(
                ptr::null_mut(),
                windows_sys::w!("open"),
                path.as_ptr(),
                ptr::null(),
                ptr::null(),
                1,
            )
        };
        if outcome as isize <= 32 {
            return Err(io::Error::other(
                "Windows could not open the profile folder",
            ));
        }
        Ok(())
    }

    // Every profile mutation takes a fresh observation inside the sole actor.
    // Read-only storage reports must never substitute their cached admission.
    fn admit_profile_mutation(&mut self, slot: u32) -> io::Result<()> {
        self.reap();
        let observed = super::super::super::inspect_native_workspace()?.processes;
        self.data_admission(slot, &observed)
    }

    // Run only from the existing idle maintenance lane, after queued requests
    // have priority. Never delay a close reply or retry an uncertain deletion.
    pub(super) fn maintain_storage(&mut self) {
        if self.exiting { self.pending_storage_maintenance = [false; 5]; return; }
        let Some(index) = self.pending_storage_maintenance.iter().position(|pending| *pending) else { return; };
        self.pending_storage_maintenance[index] = false;
        // Missing preferences preserve the established default; corrupt/unreadable preferences fail closed.
        let Ok(preferences)=crate::engine::manager_settings::load_storage_preferences() else {return;};
        if !preferences.has_maintenance() {return;}
        let slot = index as u32 + 1;
        let Ok(workspace) = super::super::super::inspect_native_workspace() else { return; };
        if self.data_admission(slot, &workspace.processes).is_err() { return; }
        let Ok(store) = self.store(slot) else { return; };
        if require_no_pending_profile_clear(store).is_err() { return; }
        // Best effort: a locked file, incomplete preflight, or lost admission
        // is not a game exit failure and never triggers a retry loop.
        // Do not retry or continue to another target after an uncertain failure.
        if preferences.auto_clear_web_cache && profile_storage::clear(&store.private,true).is_err() {return;}
        if preferences.auto_clear_download_cache && profile_storage::clear_cache_target(&store.private,profile_storage::CacheTarget::Download,true).is_err() {return;}
        if preferences.auto_clear_old_logs {let _=profile_storage::clear_old_logs(&store.private,true);}
    }

    pub(super) fn selective_inventory(&mut self, slot: u32, category: SelectiveCategory) -> io::Result<SelectiveInventory> {
        self.admit_profile_mutation(slot)?;
        let store = self.store(slot)?;
        require_no_pending_profile_clear(store)?;
        let inventory = resource_pack_inventory::individual_inventory(&store.private, category,
            InspectionBudget::new(Duration::from_secs(2)))?;
        let items = if inventory.is_empty() {
            Vec::new()
        } else {
            let profile = directory_id(&store.private)?;
            inventory.into_iter().map(|item| {
                let identity = resource_pack_inventory::inventory_identity(&profile, slot, &item);
                SelectiveInventoryEntry { root: item.root.into(), name: item.name, identity }
            }).collect()
        };
        Ok(SelectiveInventory { slot, category, items })
    }

    fn resolve_selective_review(&mut self, slot: u32, selections: &[SelectiveSelection], budget: InspectionBudget) -> io::Result<resource_pack_inventory::SelectiveReview> {
        self.admit_profile_mutation(slot)?;
        let store = self.store(slot)?;
        require_no_pending_profile_clear(store)?;
        use resource_pack_inventory::ReviewSelection;
        let native: Vec<_> = selections.iter().map(|s| ReviewSelection {
            category: s.category,
            root: s.root.clone(), name: s.name.clone(),
        }).collect();
        let review = resource_pack_inventory::review_selection(&store.private, &native,
            budget)?;
        // Revalidate the retained profile once after the review, not once per item.
        // Every item still binds the same verified profile, slot and directory ID.
        let profile_identity = directory_id(&store.private)?;
        if !same_image(&profile_identity, &review.profile_identity) {
            return Err(invalid("Profile identity changed; refresh selection"));
        }
        for (selection, requested) in native.iter().zip(selections) {
            let item = review.items.iter().find(|i| i.category == selection.category &&
                i.root == selection.root.as_str() && i.name == selection.name)
                .ok_or_else(|| invalid("Selection no longer matches inventory"))?;
            if resource_pack_inventory::inventory_identity(&profile_identity, slot, item) != requested.identity {
                return Err(invalid("Inventory changed; refresh selection"));
            }
        }
        if budget.exhausted() { return Err(invalid("Selective review deadline exceeded")); }
        Ok(review)
    }

    pub(super) fn review_selective_inventory(&mut self, slot: u32, selections: Vec<SelectiveSelection>) -> io::Result<SelectiveReviewReceipt> {
        let review = self.resolve_selective_review(slot, &selections, InspectionBudget::new(Duration::from_secs(2)))?;
        let confirmation_token = if review.warnings.is_empty() { Some(review.confirmation_token(slot)?) } else { None };
        Ok(SelectiveReviewReceipt { slot, bytes: review.bytes, files: review.files, items: selections, warnings: review.warnings, confirmation_token })
    }

    // Integrated through a fail-closed session request: mutation admission stays held until native
    // directory sharing/disposition evidence and wire consumer review exist.
    // The fingerprint is not authorization; repeat the original admission,
    // selection identity review and exclusive preflight before disposition.
    pub(super) fn confirm_selected_data(&mut self, slot: u32, selections: &[SelectiveSelection],
        confirmation_token: &str, packs: bool) -> io::Result<()> {
        if selections.is_empty() || selections.len() > 128 ||
            selections.iter().any(|s| (s.category != SelectiveCategory::Worlds) != packs) ||
            confirmation_token.len() != 64 ||
            !confirmation_token.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)) {
            return Err(invalid("Invalid selected-data confirmation"));
        }
        let budget = InspectionBudget::new(Duration::from_secs(2));
        let review = self.resolve_selective_review(slot, selections, budget)?;
        if review.confirmation_token(slot)? != confirmation_token {
            return Err(invalid("Selected data changed; review again"));
        }
        self.admit_profile_mutation(slot)?;
        let store = self.store(slot)?;
        require_no_pending_profile_clear(store)?;
        let plan = if packs { profile_storage::preflight_selected_packs(&store.private,&review,budget)? }
            else { profile_storage::preflight_selected_worlds(&store.private,&review,budget)? };
        // Keep every deletion candidate pinned while checking native
        // ownership once more; never call a path-based fallback.
        let observed = super::super::super::inspect_native_workspace()?.processes;
        self.data_admission(slot, &observed)?;
        if budget.exhausted() { return Err(invalid("Selected data confirmation deadline exceeded")); }
        plan.dispose()
    }

    pub(super) fn clear_cache_target(&mut self, slot: u32, target: profile_storage::CacheTarget) -> io::Result<()> {
        self.admit_profile_mutation(slot)?;
        require_no_pending_profile_clear(self.store(slot)?)?;
        // Preserve the existing Web-only cancellation of queued maintenance.
        // The separately held Download route did not cancel it.
        if matches!(target, profile_storage::CacheTarget::Web) {
            self.pending_storage_maintenance[slot as usize - 1] = false;
        }
        profile_storage::clear_cache_target(&self.store(slot)?.private, target, true)
    }

    pub(super) fn change_game_settings(
        &mut self, slot: u32, change: fn(&File) -> io::Result<bool>, selected_storage: Option<bool>,
    ) -> io::Result<Vec<ProfileStorage>> {
        self.admit_profile_mutation(slot)?;
        change(&self.store(slot)?.private)?;
        self.inspect_storage(mutation_storage_slots(slot, selected_storage))
    }

    fn owned_data_processes(&self) -> io::Result<Vec<(u32, u64)>> {
        if self.exiting { return Err(invalid("Minecraft sessions are closing; wait before changing launcher data")); }
        self.running.iter().flatten().map(|started| {
            let child = &started.children[0];
            child.identity()?;
            Ok((child.pid, child.created))
        }).collect()
    }

    fn data_admission(&self, slot: u32, observed: &[OwnedProcess]) -> io::Result<()> {
        self.store(slot)?;
        // Mutations always obtain fresh retained-handle observations.
        let owned = self.owned_data_processes()?;
        require_data_idle(self.running[slot as usize - 1].is_none(), &owned, observed)
    }

    pub(super) fn clear_data(&mut self,slot:u32,token:&str)->io::Result<()> {
        self.admit_profile_mutation(slot)?;
        let index=slot as usize-1;
        self.pending_storage_maintenance[index] = false;
        let store=self.store(slot)?;
        if product_profiles::clear::confirmation(store)?!=token {
            return Err(invalid("Launcher data changed after confirmation; review it again"));
        }
        let logical=store.logical.try_clone()?;
        let root=product_profiles::clear::location(store)?;
        let store=self.stores[index].take().ok_or_else(||invalid("Inactive profile storage is unavailable"))?;
        let result=product_profiles::clear::run(store,token);
        match product_profiles::clear::reopen(logical,&root,slot) {
            Ok(store)=>self.stores[index]=Some(store),
            Err(error)=>{self.issues[index]=Some(format!("Full clear needs recovery: {error}"));return Err(error);}
        }
        let store=self.stores[index].as_ref().unwrap();
        let pending=product_profiles::clear::pending(store)?;
        if !pending && (result.is_ok() || product_profiles::clear::confirmation(store)?!=token) {
            self.issues[index]=None;self.windows[index]=None;
            return Ok(());
        }
        result.map_err(|error|io::Error::new(error.kind(),format!("Full clear did not finish. Refresh Data Storage to review its recovery status. {error}")))
    }
}

// Reply scope only: mutation admission and the five-profile roster are unchanged.
pub(super) fn mutation_storage_slots(slot: u32, selected_storage: Option<bool>) -> std::ops::RangeInclusive<u32> {
    if selected_storage == Some(true) { slot..=slot } else { 1..=5 }
}

// Reuse one observation for the read-only report, retaining per-slot admission
// and propagating a failed native identity check to every report entry.
pub(super) fn all_observed_processes_owned(owned: &[(u32,u64)], observed: &[OwnedProcess]) -> bool {
    observed.iter().all(|process| {
        let birth = process.created_file_time.parse::<u64>().ok();
        owned.iter().any(|(pid,expected_birth)|
            process.pid == *pid && birth == Some(*expected_birth))
    })
}

pub(super) fn report_data_admission(selected_idle: bool, ownership: &io::Result<bool>) -> io::Result<()> {
    let all_owned = *ownership.as_ref().map_err(|error| io::Error::new(error.kind(), error.to_string()))?;
    require_profile_data_idle(selected_idle, || all_owned)
}

pub(super) fn require_data_idle(selected_idle: bool, owned: &[(u32,u64)], observed: &[OwnedProcess]) -> io::Result<()> {
    require_profile_data_idle(selected_idle, || all_observed_processes_owned(owned, observed))
}

fn require_profile_data_idle(selected_idle: bool, all_owned: impl FnOnce() -> bool) -> io::Result<()> {
    if !selected_idle { return Err(invalid("Close this Minecraft session normally before changing its launcher data.")); }
    if !all_owned() {
        return Err(invalid("An unmanaged Minecraft session is open. Close it normally before changing launcher data; managed sessions in other launchers may stay open."));
    }
    Ok(())
}
