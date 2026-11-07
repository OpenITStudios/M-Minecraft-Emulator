<script lang="ts">
  // Visual state only: all cache/settings/full-clear reviews and native
  // admission remain in LazyTesterApp. The three form bindings target its state.
  import {tick} from 'svelte';
  import type {DataLocation, ProfileSession, ProfileSlot, ProfileStorage} from './app/bridge/lazyTester.js';
  import {userMessage} from './app/presentation/sessionFeedback.js';
  import {formatStorageBytes, formatStorageExtent} from './app/presentation/storageSize.js';

  type StorageSettingsAction = 'Reset' | 'Restore' | 'DisableInputResponse' |
    'EnableTextureStreaming' | 'DisableTextureStreaming';

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
  $: storageReviewOpen = clearSlot !== null || recovery !== null || fullReview !== null;

  // Presentation-only input feedback; native data-clear admission is unchanged.
  $: typedFullClearNameIncorrect = fullReview !== null && fullName.length > 0 &&
    fullName !== `Minecraft - ${fullReview.slot}`;

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

        <div class="lt-settings-content">
          <p>Profile files, cache and settings. Worlds are included; individual backups are not managed here.</p><div class="lt-launcher-heading"><select value={selectedSlot} disabled={busy||clearSlot!==null||recovery!==null||fullReview!==null} aria-label="Choose Minecraft launcher" aria-describedby={clearSlot!==null||recovery!==null||fullReview!==null?'lt-storage-review-guidance':undefined} on:change={event=>onChooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>MC0{p.slot}</option>{/each}</select><span>{!available?'Status unavailable':selectedProfile?labels[selectedProfile.state]:'Unavailable'}</span></div>
          {#if clearSlot!==null||recovery!==null||fullReview!==null}<p id="lt-storage-review-guidance" class="lt-help" role="status">Finish or cancel this review before switching profiles.</p>{/if}
          {#if storageNotice}<p role="status">{storageNotice}</p>{/if}
          {#if storage===null}
            <p class="lt-storage-feedback" role="status">Storage unavailable. Refresh to inspect again. Unknown does not mean empty.</p>
          {:else if !selectedStorage}
            <p class="lt-storage-feedback" role="status">No storage report for this profile. Refresh before changing data.</p>
          {:else if !available}
            <p class="lt-storage-feedback" role="status">Session status unavailable. Showing last reported sizes, not live usage.</p>
          {/if}
          <section class="lt-group" aria-labelledby="lt-storage-overview-section">
            <h2 id="lt-storage-overview-section" bind:this={storageOverviewHeading} tabindex="-1">Storage overview</h2>
            <div class="lt-setting"><h3>Profile data</h3>
              <strong>{selectedStorage ? formatStorageExtent(selectedStorage.bytes,selectedStorage.partial) : 'Not measured'}</strong>
            </div>
            <div class="lt-setting"><h3>Web cache</h3>
              <span>{selectedStorage ? formatStorageExtent(selectedStorage.cache.bytes,selectedStorage.cache.partial) : 'Not measured'}</span>
            </div>
            <div class="lt-setting"><h3>Available disk space</h3>
              <span>{selectedStorage?.disk ? formatStorageBytes(selectedStorage.disk.availableBytes) : 'Not measured'}</span>
            </div>
            <p class="lt-help">{storage
              ? 'Last storage report received: '+(storageReportReceivedAt||'time unavailable')+'. Not live. Includes cache and world files, but not per-world sizes. Partial values are minimums.'
              : 'Refresh to inspect storage. Not measured does not mean empty.'}</p>
          </section>
          <section class="lt-setting"><div><h3>Clear cache</h3><p>Remove temporary web cache. Accounts, worlds and settings stay.</p>{#if selectedStorage?.cache.issue}<p>{userMessage('cache')}</p>{:else if selectedStorage&&!selectedStorage.cache.canClear}<p class="lt-help">Cache cleanup unavailable. Refresh to check again.</p>{/if}</div><button class="lt-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.cache.canClear} on:click={event=>showCacheReview(event.currentTarget)}>Clear cache…</button></section>
          {#if clearSlot===selectedSlot}<section class="lt-cache-confirm" aria-label="Confirm cache cleanup" bind:this={cacheReviewElement}><h3>Clear web cache for Minecraft - {selectedSlot}?</h3><p>Worlds, accounts, settings and other launchers stay.</p><div class="lt-actions"><button disabled={busy||refreshing||resizing||!selectedStorage?.cache.canClear} on:click={()=>commitCacheReview(selectedSlot)}>Clear web cache</button><button class="lt-secondary" bind:this={cacheCancelButton} on:click={()=>cancelReview(onCancelCacheReview)}>Cancel</button></div></section>{/if}
          <section class="lt-setting"><div><h3>Data folder</h3><p class="lt-data-path">{selectedPath??'Location unavailable.'}</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!bindingConfigured} on:click={()=>onOpenFolder(selectedSlot)}>Open folder</button></section>
          <details class="lt-location-options"><summary>Find existing profile data</summary><p class="lt-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lt-error">{locationIssue}</p>{/if}<div class="lt-actions"><button class="lt-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||runningCount>0||dataLocation?.canLocate===false} on:click={onLocateData}>Locate existing data</button>{#if dataLocation&&!dataLocation.path}<button disabled={busy||refreshing||resizing||storageReviewOpen} on:click={onChooseData}>Choose new data folder</button>{/if}</div><p>Connecting data does not move or delete accounts or worlds.</p></details>
          <details class="lt-storage-packs">
            <summary><strong>Resource pack details</strong><small>{selectedStorage?.resourcePacks?(selectedStorage.resourcePacks.partial?'Incomplete':selectedStorage.resourcePacks.manifested+' on disk'):'Not inspected'} · read-only</small></summary>
            <div class="lt-storage-packs-body">
              <div class="lt-setting"><div><h3>Resource pack inventory</h3><p>Counts pack folders with manifests, not packs loaded in the game.</p></div><span>{selectedStorage?.resourcePacks?(selectedStorage.resourcePacks.partial?'Incomplete':selectedStorage.resourcePacks.manifested+' on disk'):'Not inspected'}</span></div>
          {#if selectedStorage?.resourcePacks?.issue}<p class="lt-help" role="status">{userMessage('pack')}</p>{/if}
          {#if selectedStorage?.resourcePacks?.worldDeclarations}
            <div class="lt-setting"><div><h3>Saved world pack declarations</h3><p>Counts saved-world pack references, not packs active in game.</p></div><span>{selectedStorage.resourcePacks.worldDeclarations.partial?'Incomplete':selectedStorage.resourcePacks.worldDeclarations.packReferences+' references · '+selectedStorage.resourcePacks.worldDeclarations.worldsWithReferences+' of '+selectedStorage.resourcePacks.worldDeclarations.worldsScanned+' saved worlds'}</span></div>
            {#if selectedStorage.resourcePacks.worldDeclarations.issue}<p class="lt-help" role="status">{userMessage('worldPack')}</p>{/if}
          {:else if selectedProfile?.state!=='STOPPED'}
            <p class="lt-help">Pack inspection unavailable while the game runs or profile access is unverified.</p>
          {/if}
          <p class="lt-help">No packs are changed. Active packs and memory usage are unknown. Manage packs in Minecraft.</p>
            </div>
          </details>
          <details class="lt-advanced" bind:open={advanced}>
            <summary aria-disabled={recovery!==null} title={recovery!==null?'Finish or cancel this recovery review first.':undefined} on:click={event=>{if(recovery!==null)event.preventDefault();}}><span><strong>Advanced recovery</strong><small>Reset and restore saved settings for this launcher</small></span><span aria-hidden="true">{advanced?'−':'+'}</span></summary>
            <div class="lt-advanced-body">
              {#if selectedStorage?.gameSettings?.textureStreamingEnabled===false}
                <p class="lt-help" role="status">Texture Streaming is off. Memory savings in Education are unverified.</p>
                <div class="lt-setting"><div><h3>Reduce texture memory</h3><p>Enable for this stopped launcher. May change texture loading. Settings are backed up; restart required.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.gameSettings?.canReset||selectedProfile?.state!=='STOPPED'} on:click={event=>showSettingsReview('EnableTextureStreaming',event.currentTarget)}>Review texture streaming…</button></div>
              {:else if selectedStorage?.gameSettings?.textureStreamingEnabled===true}
                <p class="lt-help">Texture Streaming is on. Memory savings in Education are unverified.</p>
                <div class="lt-setting"><div><h3>Disable Texture Streaming</h3><p>Disable for this stopped launcher. May increase GPU memory use. Settings are backed up; restart required.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.gameSettings?.canReset||selectedProfile?.state!=='STOPPED'} on:click={event=>showSettingsReview('DisableTextureStreaming',event.currentTarget)}>Review disable Texture Streaming…</button></div>
              {:else}
                <p class="lt-help">Texture Streaming state unknown. No change will be made.</p>
              {/if}
              {#if selectedStorage?.gameSettings?.improvedInputResponseEnabled===true}
                <p class="lt-help" role="status">Improved Input Response is on. Older versions reported high RAM use; impact here is unverified.</p>
                <div class="lt-setting"><div><h3>Input-response setting</h3><p>Disable for this stopped launcher. May change input latency. Settings are backed up; restart required.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.gameSettings?.canReset||selectedProfile?.state!=='STOPPED'} on:click={event=>showSettingsReview('DisableInputResponse',event.currentTarget)}>Review input setting…</button></div>
              {:else if selectedStorage?.gameSettings?.improvedInputResponseEnabled===false}
                <p class="lt-help">Improved Input Response is off. RAM savings are unverified.</p>
              {:else}
                <p class="lt-help">Improved Input Response state unknown. No change assumed.</p>
              {/if}
              <div class="lt-setting"><div><h3>Reset game settings</h3><p>Back up and reset settings for next launch. Accounts, worlds and cache stay.</p><p class="lt-help">Preserve Visuals does not overwrite Minecraft graphics options.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.gameSettings?.canReset} on:click={event=>showSettingsReview('Reset',event.currentTarget)}>Reset settings…</button></div>
              <div class="lt-setting"><div><h3>Restore settings</h3><p>{selectedStorage?.gameSettings?.backupAvailable?'Restore the latest verified backup. Current settings are backed up first.':'No verified backup yet. Reset creates one.'}</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.gameSettings?.canRestore} on:click={event=>showSettingsReview('Restore',event.currentTarget)}>Restore settings…</button></div>
              {#if selectedStorage?.gameSettings?.issue}<p class="lt-help">{userMessage('gameSettings')}</p>{:else if !selectedStorage?.gameSettings}<p class="lt-help">Refresh to check recovery options.</p>{:else if !selectedStorage.gameSettings.canReset && !selectedStorage.gameSettings.canRestore}<p class="lt-help">Recovery unavailable. Close this game normally, then Refresh.</p>{/if}
              {#if recovery?.slot===selectedSlot}
                <section class="lt-cache-confirm" aria-label="Confirm game settings change" bind:this={settingsReviewElement}><h3>{recovery.action==='EnableTextureStreaming'?'Enable Texture Streaming':recovery.action==='DisableTextureStreaming'?'Disable Texture Streaming':recovery.action==='DisableInputResponse'?'Disable Improved Input Response':recovery.action+' settings'} for Minecraft - {recovery.slot}?</h3><p>{recovery.action==='EnableTextureStreaming'?'Enables Texture Streaming for this launcher. May reduce GPU memory, but Education results are unverified. The previous Restore point is replaced with a backup of current settings. Restart required. Accounts, worlds and other launchers stay.':recovery.action==='DisableTextureStreaming'?'Disables Texture Streaming for this launcher. May increase GPU memory use. Current settings are backed up; restart required. Accounts, worlds and other launchers stay.':recovery.action==='DisableInputResponse'?'Disables Improved Input Response for this launcher. Input latency may change; RAM savings are unverified. The previous Restore point is replaced with a backup of current settings. Restart required. Accounts, worlds and other launchers stay.':'Only this launcher’s game settings change. Accounts, worlds, cache and other launchers are kept. A backup is saved first.'}</p><div class="lt-actions"><button disabled={busy||refreshing||resizing||!(recovery.action==='EnableTextureStreaming'?selectedStorage?.gameSettings?.canReset&&selectedStorage?.gameSettings?.textureStreamingEnabled===false&&selectedProfile?.state==='STOPPED':recovery.action==='DisableTextureStreaming'?selectedStorage?.gameSettings?.canReset&&selectedStorage?.gameSettings?.textureStreamingEnabled===true&&selectedProfile?.state==='STOPPED':recovery.action==='DisableInputResponse'?selectedStorage?.gameSettings?.canReset&&selectedStorage?.gameSettings?.improvedInputResponseEnabled===true&&selectedProfile?.state==='STOPPED':recovery.action==='Reset'?selectedStorage?.gameSettings?.canReset:selectedStorage?.gameSettings?.canRestore)} on:click={commitSettingsReview}>{recovery.action==='EnableTextureStreaming'?'Enable saved option':recovery.action==='DisableTextureStreaming'?'Disable saved option':recovery.action==='DisableInputResponse'?'Disable saved option':recovery.action+' settings'}</button><button class="lt-secondary" bind:this={settingsCancelButton} disabled={busy} on:click={()=>cancelReview(onCancelSettingsReview)}>Cancel</button></div></section>
              {/if}
            </div>
          </details>
          {#if clearingData}<p class="lt-alert" role="status">Clearing profile data… Keep LazyTester open.</p>{/if}
          <section class="lt-danger-zone" aria-labelledby="lt-clear-data-heading"><div><h3 id="lt-clear-data-heading" tabindex="-1">Clear all data</h3><p>Permanently deletes this launcher's local accounts, worlds, cache, settings and backups. Minecraft and app preferences stay.</p><p>Export worlds first: no recovery backup is made. Windows sign-in may remain. External backups stay.</p>{#if selectedStorage?.fullClear?.pending}<p>Cleanup is pending. This launcher cannot start until it finishes.</p>{/if}{#if selectedStorage?.fullClear?.issue}<p>{userMessage('clearData')}</p>{:else if !selectedStorage?.fullClear}<p>Refresh storage to check availability.</p>{/if}</div><button class="lt-secondary lt-danger" disabled={busy||refreshing||resizing||storageReviewOpen||!available||!selectedStorage?.fullClear?.canClear} on:click={event=>showFullReview(event.currentTarget)}>{selectedStorage?.fullClear?.pending?'Finish clearing data…':'Clear all data…'}</button></section>
          {#if fullReview?.slot===selectedSlot}
            <form class="lt-cache-confirm lt-delete-confirm" aria-label="Confirm full launcher data deletion" bind:this={fullReviewElement} on:submit|preventDefault={onClearAllData}>
              <h3>{fullReview.pending?'Finish clearing':'Clear all data for'} Minecraft - {fullReview.slot}?</h3>
              <p>Permanently deletes this launcher's local accounts, worlds, cache, settings and backups. Other launchers and Windows sign-in stay.</p>
              <label for="full-clear-name">Type Minecraft - {fullReview.slot} to confirm</label>
              <input id="full-clear-name" type="text" maxlength="32" autocomplete="off" bind:value={fullName} disabled={busy} aria-invalid={typedFullClearNameIncorrect} aria-describedby={typedFullClearNameIncorrect?'lt-full-clear-hint lt-full-clear-error':'lt-full-clear-hint'}/>
              <p id="lt-full-clear-hint" class="lt-help">Enter the exact profile name, not your account name.</p>
              {#if typedFullClearNameIncorrect}<p id="lt-full-clear-error" class="lt-field-error" role="status">The name does not match Minecraft - {fullReview.slot}. No deletion has been requested.</p>{/if}
              <label class="lt-delete-ack"><input type="checkbox" bind:checked={fullAck} disabled={busy} aria-describedby="lt-full-clear-ack-hint"/> I understand this cannot be undone and have exported any worlds I need.</label>
              <p id="lt-full-clear-ack-hint" class="lt-help">Required before permanent deletion.</p>
              <div class="lt-actions"><button type="submit" class="lt-danger" disabled={busy||refreshing||resizing||!fullAck||fullName!==`Minecraft - ${fullReview.slot}`||!selectedStorage?.fullClear?.canClear||selectedStorage?.fullClear?.confirmationToken!==fullReview.token}>Permanently clear data</button><button type="button" class="lt-secondary" disabled={busy} on:click={()=>cancelReview(onCancelFullReview)}>Cancel</button></div>
            </form>
          {/if}
        </div>
