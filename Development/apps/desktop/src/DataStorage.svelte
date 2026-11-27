<script lang="ts">
  // Visual state only: all cache/settings/full-clear reviews and native
  // admission remain in LazyTesterApp. The three form bindings target its state.
  import {tick} from 'svelte';
  import SelectiveCleanup from './SelectiveCleanup.svelte';
  import type {SelectiveCategory, SelectiveInventory, SelectiveSelection, SelectiveReviewReceipt} from './app/bridge/lazyTester.js';
  import type {DataLocation, ProfileSession, ProfileSlot, ProfileStorage} from './app/bridge/lazyTester.js';
  import {userMessage} from './app/presentation/sessionFeedback.js';
  import {formatStorageBytes, formatStorageExtent} from './app/presentation/storageSize.js';

  type StorageSettingsAction = 'Reset' | 'Restore';

  export let profiles: ProfileSession[];
  export let selectedSlot: ProfileSlot;
  export let selectedProfile: ProfileSession | undefined;
  export let selectedStorage: ProfileStorage | undefined;
  export let selectedPath: string | undefined;
  export let labels: Record<ProfileSession['state'], string>;
  export let storage: ProfileStorage[] | null;
  export let storageReportReceivedAt: string;
  export let storageNotice: string;
  export let busy: boolean;
  export let refreshing: boolean;
  export let resizing: boolean;
  export let available: boolean;
  export let bindingConfigured: boolean;
  export let advanced: boolean;
  export let clearSlot: ProfileSlot | null;
  export let recovery: {slot: ProfileSlot; action: StorageSettingsAction} | null;
  export let fullReview: {slot: ProfileSlot; token: string; pending: boolean} | null;
  export let fullName: string;
  export let fullAck: boolean;
  export let clearingData: boolean;
  export let dataLocation: DataLocation | null;
  export let locationIssue: string;
  export let runningCount: number;

  export let onChooseSlot: (slot: ProfileSlot) => void;
  export let onInspectSelective: (slot: ProfileSlot, category: SelectiveCategory) => Promise<SelectiveInventory>;
  export let onReviewSelective: (slot: ProfileSlot, selections: SelectiveSelection[]) => Promise<SelectiveReviewReceipt>;
  export let onConfirmWorlds: (slot: ProfileSlot, receipt: SelectiveReviewReceipt) => Promise<void>;
  let selectiveSlot: ProfileSlot | null = null;
  let maintenanceOpen = false;
  export let nativeClosePending = false;
  export let selectiveOpen = false;
  $: selectiveOpen = selectiveSlot !== null;
  let selectiveTrigger: HTMLButtonElement | undefined;
  $: if (selectiveSlot !== null && (selectiveSlot !== selectedSlot || !available || !bindingConfigured || selectedProfile?.state !== 'STOPPED' || !selectedStorage || selectedStorage.fullClear?.pending === true || nativeClosePending || clearSlot !== null || recovery !== null || fullReview !== null)) closeSelective();
  function closeSelective(): void {
    selectiveSlot = null;
    void tick().then(() => {
      if(selectiveTrigger?.isConnected && !selectiveTrigger.disabled) selectiveTrigger.focus({preventScroll:true});
      else if(storageOverviewHeading?.isConnected) storageOverviewHeading.focus({preventScroll:true});
    });
  }
  export let onReviewCache: () => void;
  export let onClearCache: (slot: ProfileSlot) => void;
  export let onCancelCacheReview: () => void;
  export let onOpenFolder: (slot: ProfileSlot) => void;
  export let onReviewSettings: (action: StorageSettingsAction) => void;
  export let onRecoverSettings: () => void;
  export let onCancelSettingsReview: () => void;
  export let onReviewFullClear: () => void;
  export let onClearAllData: () => void;
  export let onCancelFullReview: () => void;
  export let onLocateData: () => void;
  export let onChooseData: () => void;
  // Presentation-only keyboard context. The original parent and native engine
  // retain confirmation admission, data ownership and every mutation.
  let reviewTrigger: HTMLButtonElement | null = null;
  let storageOverviewHeading: HTMLHeadingElement | undefined;
  let cacheReviewElement: HTMLElement | undefined;
  let settingsReviewElement: HTMLElement | undefined;
  let fullReviewElement: HTMLFormElement | undefined;
  let cacheCancelButton: HTMLButtonElement | undefined;
  let settingsCancelButton: HTMLButtonElement | undefined;

  // Keep one storage review in place until native confirmation or Cancel.
  $: storageReviewOpen = clearSlot !== null || recovery !== null || fullReview !== null || selectiveSlot !== null;

  // Presentation-only input feedback; native data-clear admission is unchanged.
  $: typedFullClearNameIncorrect = fullReview !== null && fullName.length > 0 &&
    fullName !== `Minecraft - ${fullReview.slot}`;

  $: if(clearSlot!==null) maintenanceOpen=true;

  function showCacheReview(trigger: HTMLButtonElement): void {
    if (storageReviewOpen || busy || refreshing || resizing) return;
    reviewTrigger = trigger;
    const slot = selectedSlot;
    onReviewCache();
    void tick().then(() => {
      if (clearSlot === slot && cacheCancelButton?.isConnected && !cacheCancelButton.disabled)
        cacheCancelButton.focus({preventScroll: true});
    });
  }

  function showSettingsReview(action: StorageSettingsAction, trigger: HTMLButtonElement): void {
    if (storageReviewOpen || busy || refreshing || resizing) return;
    reviewTrigger = trigger;
    const slot = selectedSlot;
    onReviewSettings(action);
    void tick().then(() => {
      if (recovery?.slot === slot && recovery.action === action &&
          settingsCancelButton?.isConnected && !settingsCancelButton.disabled)
        settingsCancelButton.focus({preventScroll: true});
    });
  }

  function showFullReview(trigger: HTMLButtonElement): void {
    if (storageReviewOpen || busy || refreshing || resizing) return;
    reviewTrigger = trigger;
    // The parent already focuses the typed identity confirmation field.
    onReviewFullClear();
  }

  // Cache and settings confirmations are immediately removed by the parent
  // once their validated request starts. Reanchor ONLY actual WebView focus
  // from that review to the persistent measurement heading before removal.
  // The native operation/denial still runs through the original callback.
  function commitCacheReview(slot: ProfileSlot): void {
    if (document.hasFocus() && cacheReviewElement?.contains(document.activeElement))
      storageOverviewHeading?.focus({preventScroll:true});
    onClearCache(slot);
  }
  function commitSettingsReview(): void {
    if (document.hasFocus() && settingsReviewElement?.contains(document.activeElement))
      storageOverviewHeading?.focus({preventScroll:true});
    onRecoverSettings();
  }

  function cancelReview(cancel: () => void): void {
    const trigger = reviewTrigger;
    reviewTrigger = null;
    cancel();
    void tick().then(() => {
      if (trigger?.isConnected && !trigger.disabled) trigger.focus({preventScroll: true});
    });
  }

  function onReviewEscape(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || busy || refreshing || resizing ||
        !(event.target instanceof Node)) return;
    const target = event.target;
    const cancel = fullReview?.slot === selectedSlot && fullReviewElement?.contains(target)
      ? onCancelFullReview
      : recovery?.slot === selectedSlot && settingsReviewElement?.contains(target)
        ? onCancelSettingsReview
        : clearSlot === selectedSlot && cacheReviewElement?.contains(target)
          ? onCancelCacheReview
          : null;
    if (!cancel) return;
    event.preventDefault();
    event.stopPropagation();
    cancelReview(cancel);
  }
</script>
<svelte:window on:keydown={onReviewEscape}/>

        <div class="lazytester-settings-content">
          <section class="lazytester-group" aria-labelledby="lazytester-global-storage-heading">
            <div class="lazytester-profile-management-heading">
              <h2 id="lazytester-global-storage-heading">Global Settings</h2>
              <span class="lazytester-help">All launchers</span>
            </div>
            <div class="lazytester-settings-panel">
              <div class="lazytester-setting">
                <div><h3>Auto-clear Download Cache</h3><p>Unavailable until the download cache location is verified.</p></div>
                <label class="lazytester-audio-toggle"><input type="checkbox" role="switch" aria-label="Auto-clear Download Cache" checked={false} disabled/></label>
              </div>
              <div class="lazytester-setting">
                <div><h3>Auto-clear Old Logs</h3><p>Keep logs for 7 days. Unavailable until log locations are verified.</p></div>
                <label class="lazytester-audio-toggle"><input type="checkbox" role="switch" aria-label="Auto-clear Old Logs" checked={false} disabled/></label>
              </div>
              <div class="lazytester-actions lazytester-settings-panel-footer lazytester-global-storage-footer">
                <p class="lazytester-help">Verified web cache is eligible for automatic cleanup after a successful game exit. Additional cleanup settings are not available yet.</p>
                <button class="lazytester-secondary" disabled>Save Changes</button>
              </div>
            </div>
          </section>
          <section class="lazytester-group" aria-labelledby="lazytester-storage-overview-section">
            <div class="lazytester-profile-management-heading">
              <h2 id="lazytester-storage-overview-section" bind:this={storageOverviewHeading} tabindex="-1">Profile Management</h2>
              <div class="lazytester-launcher-heading"><select value={selectedSlot} disabled={busy||storageReviewOpen} aria-label="Choose Minecraft launcher" aria-describedby={storageReviewOpen?'lazytester-storage-review-guidance':undefined} on:change={event=>onChooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>Minecraft {p.slot}</option>{/each}</select><span>{!available?'Status unavailable':selectedProfile?labels[selectedProfile.state]:'Unavailable'}</span></div>
            </div>
          {#if storageReviewOpen}<p id="lazytester-storage-review-guidance" class="lazytester-help" role="status">Finish or cancel this review before switching profiles.</p>{/if}
          {#if storageNotice}<p role="status">{storageNotice}</p>{/if}
          {#if storage===null}
            <p class="lazytester-storage-feedback" role="status">Storage unavailable. Refresh to inspect again. Unknown does not mean empty.</p>
          {:else if !selectedStorage}
            <p class="lazytester-storage-feedback" role="status">No storage report for this profile. Refresh before changing data.</p>
          {:else if !available}
            <p class="lazytester-storage-feedback" role="status">Session status unavailable. Showing last reported sizes, not live usage.</p>
          {/if}
          <div class="lazytester-settings-panel">
            <div class="lazytester-setting"><div><h3>Profile Folder</h3><p>{selectedStorage ? formatStorageExtent(selectedStorage.bytes,selectedStorage.partial) : 'Not measured'}</p><p class="lazytester-data-path">{selectedPath??'Location unavailable.'}</p></div><button class="lazytester-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!bindingConfigured} on:click={()=>onOpenFolder(selectedSlot)}>Open Folder</button></div>
            <div class="lazytester-setting"><div><h3>Selective Cleanup</h3><p>Review individual worlds and packs.</p></div><button class="lazytester-secondary" bind:this={selectiveTrigger} disabled={busy||refreshing||resizing||storageReviewOpen||!available||!bindingConfigured||selectedProfile?.state!=='STOPPED'||!selectedStorage||selectedStorage.fullClear?.pending===true} on:click={()=>selectiveSlot=selectedSlot}>Select Data</button></div>
          <section class="lazytester-setting lazytester-profile-clear" aria-labelledby="lazytester-clear-data-heading"><div><h3 id="lazytester-clear-data-heading" tabindex="-1">Clear All Data</h3><p>Permanently deletes this launcher's local accounts, worlds, cache, settings and backups. Minecraft and app preferences stay.</p><p>Export worlds first: no recovery backup is made. Windows sign-in may remain. External backups stay.</p>{#if selectedStorage?.fullClear?.pending}<p>Cleanup is pending. This launcher cannot start until it finishes.</p>{/if}{#if selectedStorage?.fullClear?.issue}<p>{userMessage('clearData')}</p>{:else if !selectedStorage?.fullClear}<p>Refresh storage to check availability.</p>{/if}</div><button class="lazytester-secondary lazytester-danger" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.fullClear?.canClear} on:click={event=>showFullReview(event.currentTarget)}>{selectedStorage?.fullClear?.pending?'Finish clearing data…':'Clear All…'}</button></section>
          </div>
          <details class="lazytester-storage-maintenance" bind:open={maintenanceOpen}><summary aria-disabled={clearSlot!==null} on:click={event=>{if(clearSlot!==null)event.preventDefault();}}>Storage details and maintenance</summary><div class="lazytester-settings-panel">
            <div class="lazytester-setting"><h3>Web cache</h3>
              <span>{selectedStorage ? formatStorageExtent(selectedStorage.cache.bytes,selectedStorage.cache.partial) : 'Not measured'}</span>
            </div>
            <div class="lazytester-setting"><h3>Available disk space</h3>
              <span>{selectedStorage?.disk ? formatStorageBytes(selectedStorage.disk.availableBytes) : 'Not measured'}</span>
            </div>
            <p class="lazytester-help">{storage
              ? 'Last storage report received: '+(storageReportReceivedAt||'time unavailable')+'. Includes cache and worlds. Partial values are minimums, not live usage.'
              : 'Refresh to inspect storage. Not measured does not mean empty.'}</p>

          <div class="lazytester-setting"><div><h3>Clear cache</h3><p>Remove temporary web cache. Accounts, worlds and settings stay.</p>{#if selectedStorage?.cache.issue}<p>{userMessage('cache')}</p>{:else if selectedStorage&&!selectedStorage.cache.canClear}<p class="lazytester-help">Cache cleanup unavailable. Refresh to check again.</p>{/if}</div><button class="lazytester-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.cache.canClear} on:click={event=>showCacheReview(event.currentTarget)}>Clear cache…</button></div>
          {#if clearSlot===selectedSlot}<section class="lazytester-cache-confirm" aria-label="Confirm cache cleanup" bind:this={cacheReviewElement}><h3>Clear web cache for Minecraft - {selectedSlot}?</h3><p>Worlds, accounts, settings and other launchers stay.</p><div class="lazytester-actions"><button disabled={busy||refreshing||resizing||!selectedStorage?.cache.canClear} on:click={()=>commitCacheReview(selectedSlot)}>Clear web cache</button><button class="lazytester-secondary" bind:this={cacheCancelButton} on:click={()=>cancelReview(onCancelCacheReview)}>Cancel</button></div></section>{/if}
          </div></details>
          <details class="lazytester-location-options" open={!bindingConfigured || !dataLocation?.path}><summary>Find existing profile data</summary><p class="lazytester-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lazytester-error">{locationIssue}</p>{/if}<div class="lazytester-actions"><button class="lazytester-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||runningCount>0||dataLocation?.canLocate===false} on:click={onLocateData}>Locate existing data</button>{#if dataLocation&&!dataLocation.path}<button disabled={busy||refreshing||resizing||storageReviewOpen} on:click={onChooseData}>Choose new data folder</button>{/if}</div><p>Connecting data does not move or delete accounts or worlds.</p></details>
          <details class="lazytester-storage-packs">
            <summary><strong>Resource pack details</strong></summary>
            <div class="lazytester-storage-packs-body">
              <div class="lazytester-setting"><div><h3>Resource pack inventory</h3><p>Counts pack folders with manifests, not packs loaded in the game.</p></div><span>{selectedStorage?.resourcePacks?(selectedStorage.resourcePacks.partial?'Incomplete':selectedStorage.resourcePacks.manifested+' on disk'):'Not inspected'}</span></div>
          {#if selectedStorage?.resourcePacks?.issue}<p class="lazytester-help" role="status">{userMessage('pack')}</p>{/if}
          {#if selectedStorage?.resourcePacks?.worldDeclarations}
            <div class="lazytester-setting"><div><h3>Saved world pack declarations</h3><p>Counts saved-world pack references, not packs active in game.</p></div><span>{selectedStorage.resourcePacks.worldDeclarations.partial?'Incomplete':selectedStorage.resourcePacks.worldDeclarations.packReferences+' references · '+selectedStorage.resourcePacks.worldDeclarations.worldsWithReferences+' of '+selectedStorage.resourcePacks.worldDeclarations.worldsScanned+' saved worlds'}</span></div>
            {#if selectedStorage.resourcePacks.worldDeclarations.issue}<p class="lazytester-help" role="status">{userMessage('worldPack')}</p>{/if}
          {:else if selectedProfile?.state!=='STOPPED'}
            <p class="lazytester-help">Pack inspection unavailable while the game runs or profile access is unverified.</p>
          {/if}
          <p class="lazytester-help">No packs are changed. Active packs and memory usage are unknown. Manage packs in Minecraft.</p>
            </div>
          </details>
          <details class="lazytester-advanced" bind:open={advanced}>
            <summary aria-disabled={recovery!==null} title={recovery!==null?'Finish or cancel this recovery review first.':undefined} on:click={event=>{if(recovery!==null)event.preventDefault();}}><span><strong>Advanced recovery</strong><small>Saved settings backup and restore</small></span><span aria-hidden="true">{advanced?'−':'+'}</span></summary>
            <div class="lazytester-advanced-body">
              <div class="lazytester-setting"><div><h3>Reset game settings</h3><p>Back up and reset settings for next launch. Accounts, worlds and cache stay.</p></div><button class="lazytester-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.gameSettings?.canReset} on:click={event=>showSettingsReview('Reset',event.currentTarget)}>Reset settings…</button></div>
              <div class="lazytester-setting"><div><h3>Restore settings</h3><p>{selectedStorage?.gameSettings?.backupAvailable?'Restore the latest verified backup. Current settings are backed up first.':'No verified backup yet. Reset creates one.'}</p></div><button class="lazytester-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.gameSettings?.canRestore} on:click={event=>showSettingsReview('Restore',event.currentTarget)}>Restore settings…</button></div>
              {#if selectedStorage?.gameSettings?.issue}<p class="lazytester-help">{userMessage('gameSettings')}</p>{:else if !selectedStorage?.gameSettings}<p class="lazytester-help">Refresh to check recovery options.</p>{:else if !selectedStorage.gameSettings.canReset && !selectedStorage.gameSettings.canRestore}<p class="lazytester-help">Recovery unavailable. Close this game normally, then Refresh.</p>{/if}
              {#if recovery?.slot===selectedSlot}
                <section class="lazytester-cache-confirm" aria-label="Confirm game settings change" bind:this={settingsReviewElement}><h3>{recovery.action} settings for Minecraft - {recovery.slot}?</h3><p>Only this launcher’s game settings change. Accounts, worlds, cache and other launchers are kept. A backup is saved first.</p><div class="lazytester-actions"><button disabled={busy||refreshing||resizing||!(recovery.action==='Reset'?selectedStorage?.gameSettings?.canReset:selectedStorage?.gameSettings?.canRestore)} on:click={commitSettingsReview}>{recovery.action} settings</button><button class="lazytester-secondary" bind:this={settingsCancelButton} disabled={busy} on:click={()=>cancelReview(onCancelSettingsReview)}>Cancel</button></div></section>
              {/if}
            </div>
          </details>
          {#if clearingData}<p class="lazytester-alert" role="status">Clearing profile data… Keep LazyTester open.</p>{/if}
          {#if fullReview?.slot===selectedSlot}
            <form class="lazytester-cache-confirm lazytester-delete-confirm" aria-label="Confirm full launcher data deletion" bind:this={fullReviewElement} on:submit|preventDefault={onClearAllData}>
              <h3>{fullReview.pending?'Finish clearing':'Clear all data for'} Minecraft - {fullReview.slot}?</h3>
              <p>Permanently deletes this launcher's local accounts, worlds, cache, settings and backups. Other launchers and Windows sign-in stay.</p>
              <label for="full-clear-name">Type Minecraft - {fullReview.slot} to confirm</label>
              <input id="full-clear-name" type="text" maxlength="32" autocomplete="off" bind:value={fullName} disabled={busy} aria-invalid={typedFullClearNameIncorrect} aria-describedby={typedFullClearNameIncorrect?'lazytester-full-clear-hint lazytester-full-clear-error':'lazytester-full-clear-hint'}/>
              <p id="lazytester-full-clear-hint" class="lazytester-help">Enter the exact profile name, not your account name.</p>
              {#if typedFullClearNameIncorrect}<p id="lazytester-full-clear-error" class="lazytester-field-error" role="status">The name does not match Minecraft - {fullReview.slot}. No deletion has been requested.</p>{/if}
              <label class="lazytester-delete-ack"><input type="checkbox" bind:checked={fullAck} disabled={busy} aria-describedby="lazytester-full-clear-ack-hint"/> I understand this cannot be undone and have exported any worlds I need.</label>
              <p id="lazytester-full-clear-ack-hint" class="lazytester-help">Required before permanent deletion.</p>
              <div class="lazytester-actions"><button type="submit" class="lazytester-danger" disabled={busy||refreshing||resizing||!fullAck||fullName!==`Minecraft - ${fullReview.slot}`||!selectedStorage?.fullClear?.canClear||selectedStorage?.fullClear?.confirmationToken!==fullReview.token}>Permanently clear data</button><button type="button" class="lazytester-secondary" disabled={busy} on:click={()=>cancelReview(onCancelFullReview)}>Cancel</button></div>
            </form>
          {/if}
          </section>
        </div>

{#if selectiveSlot!==null}
  <SelectiveCleanup slot={selectiveSlot} inspect={onInspectSelective} review={onReviewSelective} confirmWorlds={onConfirmWorlds} onClose={closeSelective}/>
{/if}
