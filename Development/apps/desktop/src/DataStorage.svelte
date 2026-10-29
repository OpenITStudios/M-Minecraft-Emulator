<script lang="ts">
  // Visual state only: all cache/settings/full-clear reviews and native
  // admission remain in LazyTesterApp. The three form bindings target its state.
  import type {DataLocation, ProfileSession, ProfileSlot, ProfileStorage} from './app/bridge/lazyTester.js';
  import {userMessage} from './app/presentation/sessionFeedback.js';

  type StorageSettingsAction = 'Reset' | 'Restore' | 'DisableInputResponse' |
    'EnableTextureStreaming' | 'DisableTextureStreaming';

  export let profiles: ProfileSession[];
  export let selectedSlot: ProfileSlot;
  export let selectedProfile: ProfileSession | undefined;
  export let selectedStorage: ProfileStorage | undefined;
  export let selectedPath: string | undefined;
  export let labels: Record<ProfileSession['state'], string>;
  export let storage: ProfileStorage[] | null;
  export let storageCheckedAt: string;
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
</script>

        <div class="lt-settings-content">
          <p>Worlds, cache and settings for each launcher.</p><div class="lt-launcher-heading"><select value={selectedSlot} disabled={busy} aria-label="Choose Minecraft launcher" on:change={event=>onChooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>Minecraft - {p.slot}</option>{/each}</select><span>{selectedProfile?labels[selectedProfile.state]:'Unavailable'}</span></div>
          {#if storageNotice}<p role="status">{storageNotice}</p>{/if}
          <section class="lt-group"><div class="lt-setting"><h3>Profile data</h3><strong>{selectedStorage?(selectedStorage.partial?'At least ':'')+(selectedStorage.bytes/1048576).toFixed(1)+' MB':'Not measured'}</strong></div><div class="lt-setting"><h3>Web cache</h3><span>{selectedStorage&&!selectedStorage.cache.partial?(selectedStorage.cache.bytes/1048576).toFixed(1)+' MB':'Unavailable'}</span></div><div class="lt-setting"><div><h3>Resource pack inventory</h3><p>Manifest-containing folders under resource_packs and development_resource_packs. This is not a list of loaded or active textures.</p></div><span>{selectedStorage?.resourcePacks?(selectedStorage.resourcePacks.partial?'Incomplete':selectedStorage.resourcePacks.manifested+' on disk'):'Not inspected'}</span></div>
          {#if selectedStorage?.resourcePacks?.issue}<p class="lt-help" role="status">{userMessage('pack')}</p>{/if}
          {#if selectedStorage?.resourcePacks?.worldDeclarations}
            <div class="lt-setting"><div><h3>Saved world pack declarations</h3><p>Read-only counts from world_resource_packs.json in this launcher's saved worlds. They show declared references, not which packs loaded in the running game.</p></div><span>{selectedStorage.resourcePacks.worldDeclarations.partial?'Incomplete':selectedStorage.resourcePacks.worldDeclarations.packReferences+' references · '+selectedStorage.resourcePacks.worldDeclarations.worldsWithReferences+' of '+selectedStorage.resourcePacks.worldDeclarations.worldsScanned+' saved worlds'}</span></div>
            {#if selectedStorage.resourcePacks.worldDeclarations.issue}<p class="lt-help" role="status">{userMessage('worldPack')}</p>{/if}
          {:else if selectedProfile?.state!=='STOPPED'}
            <p class="lt-help">Resource-pack inspection is deferred while the selected game is active or its profile access cannot be verified.</p>
          {/if}
          <p class="lt-help">No packs are automatically disabled or removed. Global packs, currently loaded textures, GPU memory and reclaimable RAM remain unknown. Manage resource packs through Minecraft Education.</p><div class="lt-setting"><h3>Available disk space</h3><span>{selectedStorage?.disk?(selectedStorage.disk.availableBytes/1073741824).toFixed(1)+' GB':'Unavailable'}</span></div><p class="lt-help">{storage?'Last measured at '+storageCheckedAt+'. Web cache is included in profile data.':'Use Refresh to measure storage. World sizes are not separately classified.'}</p></section>
          <section class="lt-setting"><div><h3>Clear cache</h3><p>Remove recognized web cache. Accounts, worlds and game settings are kept.</p>{#if selectedStorage?.cache.issue}<p>{userMessage('cache')}</p>{/if}</div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.cache.canClear} on:click={onReviewCache}>Clear cache…</button></section>
          {#if clearSlot===selectedSlot}<section class="lt-cache-confirm" aria-label="Confirm cache cleanup"><h3>Clear web cache for Minecraft - {selectedSlot}?</h3><p>Worlds, accounts and settings are kept. Other launchers are unchanged.</p><div class="lt-actions"><button disabled={busy||refreshing||resizing||!selectedStorage?.cache.canClear} on:click={()=>onClearCache(selectedSlot)}>Clear web cache</button><button class="lt-secondary" on:click={onCancelCacheReview}>Cancel</button></div></section>{/if}
          <section class="lt-setting"><div><h3>Data folder</h3><p class="lt-data-path">{selectedPath??'Location unavailable.'}</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!bindingConfigured} on:click={()=>onOpenFolder(selectedSlot)}>Open folder</button></section>
          <details class="lt-advanced" bind:open={advanced}>
            <summary><span><strong>Advanced options</strong><small>Reset and recovery for this launcher</small></span><span aria-hidden="true">{advanced?'−':'+'}</span></summary>
            <div class="lt-advanced-body">
              {#if selectedStorage?.gameSettings?.textureStreamingEnabled===false}
                <p class="lt-help" role="status">Texture Streaming is recorded as disabled in this launcher. Bedrock 26.30 can use it to reduce unnecessary GPU texture memory on supported PC devices; Education 26.32 support and system RAM savings are not verified.</p>
                <div class="lt-setting"><div><h3>Reduce texture memory</h3><p>Enable the existing saved Texture Streaming setting for this stopped launcher only. This may change when textures are loaded; a normal Minecraft restart is required. Current options are backed up for Restore.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.gameSettings?.canReset||selectedProfile?.state!=='STOPPED'} on:click={()=>onReviewSettings('EnableTextureStreaming')}>Review texture streaming…</button></div>
              {:else if selectedStorage?.gameSettings?.textureStreamingEnabled===true}
                <p class="lt-help">Texture Streaming is recorded as enabled. Actual GPU/system memory savings are not measured, and Education 26.32 runtime support has not been confirmed.</p>
                <div class="lt-setting"><div><h3>Disable Texture Streaming</h3><p>Disable only this existing saved Texture Streaming option for the stopped launcher. This may increase GPU texture memory use. The previous options are backed up, and Minecraft must restart normally.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.gameSettings?.canReset||selectedProfile?.state!=='STOPPED'} on:click={()=>onReviewSettings('DisableTextureStreaming')}>Review disable Texture Streaming…</button></div>
              {:else}
                <p class="lt-help">Texture Streaming: unavailable or unknown for this profile. Lazy Tester will not invent or force an unsupported option.</p>
              {/if}
              {#if selectedStorage?.gameSettings?.improvedInputResponseEnabled===true}
                <p class="lt-help" role="status">Improved Input Response is enabled in this launcher's saved options. Older Minecraft Education versions had reports of excessive RAM with it enabled. Impact on Education 26.32 is unverified; no setting changes automatically.</p>
                <div class="lt-setting"><div><h3>Potential RAM leak workaround</h3><p>Review disabling this saved setting for this launcher only. It may change input latency. Minecraft must be stopped; the existing settings backup and Restore action can undo it.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.gameSettings?.canReset||selectedProfile?.state!=='STOPPED'} on:click={()=>onReviewSettings('DisableInputResponse')}>Review workaround…</button></div>
              {:else if selectedStorage?.gameSettings?.improvedInputResponseEnabled===false}
                <p class="lt-help">Improved Input Response is recorded as disabled. This does not establish RAM savings or rule out other leaks.</p>
              {:else}
                <p class="lt-help">Improved Input Response: unknown or not recorded in this profile's settings. No default is assumed.</p>
              {/if}
              <div class="lt-setting"><div><h3>Reset game settings</h3><p>Back up this launcher's settings, then reset them for the next start. Accounts, worlds and cache are kept.</p><p class="lt-help">The saved graphics preference applies at launch; Preserve Visuals never overwrites Minecraft graphics options.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.gameSettings?.canReset} on:click={()=>onReviewSettings('Reset')}>Reset settings…</button></div>
              <div class="lt-setting"><div><h3>Restore settings</h3><p>{selectedStorage?.gameSettings?.backupAvailable?'Restore the latest verified settings backup for this launcher. Current settings are backed up first.':'No verified settings backup yet. Reset creates one.'}</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.gameSettings?.canRestore} on:click={()=>onReviewSettings('Restore')}>Restore settings…</button></div>
              {#if selectedStorage?.gameSettings?.issue}<p class="lt-help">{userMessage('gameSettings')}</p>{:else if !selectedStorage?.gameSettings}<p class="lt-help">Refresh storage to check settings recovery availability.</p>{/if}
              {#if recovery?.slot===selectedSlot}
                <section class="lt-cache-confirm" aria-label="Confirm game settings change"><h3>{recovery.action==='EnableTextureStreaming'?'Enable Texture Streaming':recovery.action==='DisableTextureStreaming'?'Disable Texture Streaming':recovery.action==='DisableInputResponse'?'Disable Improved Input Response':recovery.action+' settings'} for Minecraft - {recovery.slot}?</h3><p>{recovery.action==='EnableTextureStreaming'?'Only this launcher’s explicitly disabled Texture Streaming option will be enabled. This may reduce GPU texture memory on supported builds, but actual Education 26.32 and system RAM effects are unknown. The most recent Restore backup point will be replaced with a backup of current settings. Restart Minecraft normally after applying; accounts, worlds and other launchers are unchanged.':recovery.action==='DisableTextureStreaming'?'Only this launcher’s explicitly enabled Texture Streaming option will be disabled. This may increase GPU memory consumption, and no performance improvement is assumed. Current settings are backed up before changing this one key; a normal Minecraft restart is required. Accounts, worlds, and other launchers are preserved.':recovery.action==='DisableInputResponse'?'Only this launcher’s explicitly enabled option is changed. Input latency may change and RAM savings on Education 26.32 are unverified. The existing Restore backup point will be replaced with a backup of current settings. Restart Minecraft normally after applying. Accounts, worlds, and other launchers are kept.':'Only this launcher’s game settings change. Accounts, worlds, cache and other launchers are kept. A backup is saved first.'}</p><div class="lt-actions"><button disabled={busy||refreshing||resizing||!(recovery.action==='EnableTextureStreaming'?selectedStorage?.gameSettings?.canReset&&selectedStorage?.gameSettings?.textureStreamingEnabled===false&&selectedProfile?.state==='STOPPED':recovery.action==='DisableTextureStreaming'?selectedStorage?.gameSettings?.canReset&&selectedStorage?.gameSettings?.textureStreamingEnabled===true&&selectedProfile?.state==='STOPPED':recovery.action==='DisableInputResponse'?selectedStorage?.gameSettings?.canReset&&selectedStorage?.gameSettings?.improvedInputResponseEnabled===true&&selectedProfile?.state==='STOPPED':recovery.action==='Reset'?selectedStorage?.gameSettings?.canReset:selectedStorage?.gameSettings?.canRestore)} on:click={onRecoverSettings}>{recovery.action==='EnableTextureStreaming'?'Enable saved option':recovery.action==='DisableTextureStreaming'?'Disable saved option':recovery.action==='DisableInputResponse'?'Disable saved option':recovery.action+' settings'}</button><button class="lt-secondary" disabled={busy} on:click={onCancelSettingsReview}>Cancel</button></div></section>
              {/if}
            </div>
          </details>
          {#if clearingData}<p class="lt-alert" role="status">Clearing launcher data… Keep Lazy Tester open while the verified cleanup finishes.</p>{/if}
          <section class="lt-danger-zone"><div><h3>Clear all data</h3><p>Permanently remove this launcher's stored account data, worlds, cache, game settings and backups. Keep the Minecraft installation and app preferences.</p><p>Windows-managed sign-in data is not cleared, so a previous account may still be offered. Export any worlds you want to keep before continuing; no recovery backup is created. Exported or external backups are kept.</p>{#if selectedStorage?.fullClear?.pending}<p>A previous clear needs to finish. This launcher cannot start until its old data is removed.</p>{/if}{#if selectedStorage?.fullClear?.issue}<p>{userMessage('clearData')}</p>{:else if !selectedStorage?.fullClear}<p>Refresh storage to check availability.</p>{/if}</div><button class="lt-secondary lt-danger" disabled={busy||refreshing||resizing||!available||!selectedStorage?.fullClear?.canClear} on:click={onReviewFullClear}>{selectedStorage?.fullClear?.pending?'Finish clearing data…':'Clear all data…'}</button></section>
          {#if fullReview?.slot===selectedSlot}
            <form class="lt-cache-confirm lt-delete-confirm" aria-label="Confirm full launcher data deletion" on:submit|preventDefault={onClearAllData}>
              <h3>{fullReview.pending?'Finish clearing':'Clear all data for'} Minecraft - {fullReview.slot}?</h3>
              <p>This permanently removes this launcher's stored account data, worlds, cache, settings and backups. Other launchers and Windows-managed sign-in data are kept.</p>
              <label for="full-clear-name">Type Minecraft - {fullReview.slot} to confirm</label>
              <input id="full-clear-name" type="text" maxlength="32" autocomplete="off" bind:value={fullName} disabled={busy}/>
              <label class="lt-delete-ack"><input type="checkbox" bind:checked={fullAck} disabled={busy}/> I understand this cannot be undone and have exported any worlds I need.</label>
              <div class="lt-actions"><button type="submit" class="lt-danger" disabled={busy||refreshing||resizing||!fullAck||fullName!==`Minecraft - ${fullReview.slot}`||!selectedStorage?.fullClear?.canClear||selectedStorage?.fullClear?.confirmationToken!==fullReview.token}>Permanently clear data</button><button type="button" class="lt-secondary" disabled={busy} on:click={onCancelFullReview}>Cancel</button></div>
            </form>
          {/if}
          <details class="lt-location-options"><summary>Connect or locate launcher data</summary><p class="lt-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lt-error">{locationIssue}</p>{/if}<div class="lt-actions"><button class="lt-secondary" disabled={busy||refreshing||resizing||runningCount>0||dataLocation?.canLocate===false} on:click={onLocateData}>Locate existing data</button>{#if dataLocation&&!dataLocation.path}<button disabled={busy||refreshing||resizing} on:click={onChooseData}>Choose new data folder</button>{/if}</div><p>Connecting data does not migrate or delete existing accounts and worlds.</p></details>
        </div>
