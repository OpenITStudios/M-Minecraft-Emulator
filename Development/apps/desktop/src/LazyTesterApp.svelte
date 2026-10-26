<script lang="ts">
  import {onMount, tick} from 'svelte';
  import {listen} from '@tauri-apps/api/event';
  import {lazyTester, nextProfile, validFpsLimits, type ProfileSession, type ProfileSlot, type ProfileStorage, type PreviewStatus, type ProfileBinding, type DataLocation, type SettingsReport} from './app/bridge/lazyTester.js';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import './lazy-tester.css';

  let profiles: ProfileSession[] = [], storage: ProfileStorage[] | null = null;
  let binding: ProfileBinding | null = null;
  let dataLocation:DataLocation|null=null;
  let settings:SettingsReport|null=null, settingsNotice='';
  let applyingPreset=false;
  let savingPerformance=false;
  let activeFps:number|undefined=144,backgroundFps:number|undefined=10,performanceNotice='';
  $: performanceDirty=settings!==null&&(activeFps!==settings.activeFps||backgroundFps!==settings.backgroundFps);
  $: performanceValid=validFpsLimits(activeFps,backgroundFps);
  let clearSlot: ProfileSlot | null = null, storageNotice = '';
  let page: 'workspace' | 'launchers' | 'storage' | 'settings' = 'workspace';
  let active: ProfileSlot | null = null, requested: ProfileSlot | null = null, queued: ProfileSlot | null = null;
  let viewport: HTMLDivElement, observer: ResizeObserver | undefined;
  let busy = false, refreshing = false, resizing = false, available = false;
  let mounted = false, disposed = false, dirty = false, detached = false;
  let issue = '', connectionIssue = '', previewIssue = '', selectedIdentity = '';
  let previewStatuses: PreviewStatus[] = [];
  const images = new Map<ProfileSlot, HTMLElement>();
  const labels = {STOPPED: 'Not running', STARTING: 'Starting', RUNNING: 'Running', NOT_RESPONDING: 'Not responding', ATTENTION: 'Needs attention'};
  const identity = (profile?: ProfileSession) => profile?.process ? [profile.process.pid, profile.process.createdFileTime, profile.process.sessionId, profile.process.windowHandle].join('/') : '';
  $: primary = active ?? requested ?? profiles.find(profile => profile.state === 'RUNNING')?.slot ?? 1;
  $: others = profiles.filter(profile => profile.slot !== primary);
  $: runningCount = profiles.filter(profile => ['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(profile.state)).length;
  $: playableCount = profiles.filter(profile => profile.state === 'RUNNING').length;
  $: if (mounted && !busy && !refreshing && !resizing && queued !== null) void consumeSelection();
  $: if (mounted && !busy && !refreshing && !resizing && queued === null && dirty) void syncLayout();

  function image(node: HTMLElement, slot: ProfileSlot) {
    images.set(slot, node); observer?.observe(node); dirty = true;
    return {destroy() { observer?.unobserve(node); if (images.get(slot) === node) images.delete(slot); dirty = true; }};
  }
  function select(slot: ProfileSlot) { requested = null; queued = slot; }
  async function consumeSelection() {
    const slot = queued; queued = null;
    if (slot === null || !available || profiles.find(profile => profile.slot === slot)?.state !== 'RUNNING') return;
    busy = true; issue = '';
    try {
      page = 'workspace'; await tick();
      await lazyTester.present(slot, viewport.getBoundingClientRect());
      active = slot; selectedIdentity = identity(profiles.find(profile => profile.slot === slot));
      detached = false; requested = null; dirty = true;
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function refresh() {
    if (busy || refreshing || resizing || disposed) return;
    refreshing = true;
    try {
      const snapshot = page === 'settings' ? await lazyTester.settings() : await lazyTester.inspect();
      if (disposed) return;
      profiles = snapshot.profiles; available = true; connectionIssue = '';
      if(snapshot.settings){
        if(!performanceDirty){activeFps=snapshot.settings.activeFps;backgroundFps=snapshot.settings.backgroundFps;}
        settings=snapshot.settings;
      }
      if (active !== null && selectedIdentity !== identity(profiles.find(profile => profile.slot === active))) {
        await workspaceSurface.release(); active = null; selectedIdentity = '';
      }
      if (requested !== null) {
        const profile = profiles.find(profile => profile.slot === requested);
        if (profile?.state === 'RUNNING') queued = requested;
        else if (profile?.state === 'ATTENTION' || profile?.state === 'STOPPED') {
          issue = profile.issue ?? ('Minecraft - ' + requested + ' stopped before its window became available.'); requested = null;
        }
      } else if (active === null && !detached && page === 'workspace' && queued === null) {
        queued = profiles.find(profile => profile.state === 'RUNNING')?.slot ?? null;
      }
      dirty = true;
    } catch (error) { if (!disposed) { available = false; connectionIssue = String(error); dirty = true; } }
    finally { refreshing = false; }
  }
  async function launch(slot: ProfileSlot) {
    if (busy || refreshing || resizing || !binding?.configured) return;
    busy = true; issue = ''; requested = slot;
    try {
      profiles = (await lazyTester.launch(slot)).profiles; storage = null; available = true;
      detached = false; page = 'workspace'; dirty = true;
      if (requested === slot && profiles.find(profile => profile.slot === slot)?.state === 'RUNNING') queued = slot;
      else if (active !== null) {
        await tick();
        await lazyTester.present(active, viewport.getBoundingClientRect());
      }
    } catch (error) { issue = String(error); requested = null; }
    finally { busy = false; }
  }
  async function navigate(next: typeof page) {
    if (busy || refreshing || resizing || page === next) return;
    busy = true; issue = '';
    try {
      if (next !== 'workspace') await workspaceSurface.suspend();
      page = next; await tick();
      if (next === 'workspace' && active !== null) await lazyTester.present(active, viewport.getBoundingClientRect());
      dirty = true;
    } catch (error) { issue = String(error); }
    finally { busy = false; }
    if(page==='settings')void refresh();
  }
  async function applyPreset() {
    if(busy||refreshing||resizing||!available||!binding?.configured||!settings?.canApplyPreset||runningCount>0)return;
    busy=true;applyingPreset=true;issue='';settingsNotice='';
    try {
      const snapshot=await lazyTester.applyPreset();
      profiles=snapshot.profiles;settings=snapshot.settings;
      if(settings.changedProfiles===null)throw new Error('The preset update was not confirmed. Refresh settings before trying again.');
      settingsNotice=settings.changedProfiles===0?'All five profiles already use this preset.':'Graphics preset saved for all five profiles.';
    }catch(error){issue=String(error);settings=null;}
    finally{applyingPreset=false;busy=false;}
  }
  async function savePerformance() {
    if(busy||refreshing||resizing||!available||!settings||!performanceValid||!performanceDirty)return;
    busy=true;savingPerformance=true;issue='';performanceNotice='';
    try{
      const snapshot=await lazyTester.savePerformance(activeFps!,backgroundFps!);
      profiles=snapshot.profiles;settings=snapshot.settings;
      activeFps=settings.activeFps;backgroundFps=settings.backgroundFps;
      performanceNotice='Saved. New limits apply when each Minecraft profile next starts.';
    }catch(error){issue=String(error);}
    finally{savingPerformance=false;busy=false;}
  }
  async function release() {
    if (busy || refreshing || resizing) return;
    busy = true;
    try {
      await workspaceSurface.release(); active = null; selectedIdentity = ''; requested = null;
      detached = true; previewStatuses = []; dirty = true;
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function syncLayout() {
    if (busy || refreshing || resizing || !mounted) return;
    dirty = false; resizing = true;
    try {
      if (page !== 'workspace' || detached || !available || document.hidden) {
        await lazyTester.clearPreviews(); previewStatuses = []; return;
      }
      await tick();
      if (active !== null) await workspaceSurface.update(viewport.getBoundingClientRect());
      const tiles = others.flatMap(profile => {
        const element = images.get(profile.slot); if (!element) return [];
        const rect = element.getBoundingClientRect();
        return rect.x >= 0 && rect.y >= 0 && rect.width >= 32 && rect.height >= 24 &&
          rect.right <= window.innerWidth && rect.bottom <= window.innerHeight ? [{slot: profile.slot, rect}] : [];
      });
      previewStatuses = (await lazyTester.previews(viewport.getBoundingClientRect(), tiles)).tiles;
      previewIssue = '';
    } catch (error) { previewIssue = String(error); }
    finally { resizing = false; }
  }
  function step(direction: 1 | -1) {
    const next = nextProfile(profiles, active, direction); if (next !== null) select(next);
  }
  async function inspectStorage() {
    if (busy || refreshing || resizing) return;
    busy = true; issue = ''; clearSlot = null; storageNotice = '';
    try { const snapshot = await lazyTester.storage(); profiles = snapshot.profiles; storage = snapshot.storage; }
    catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function loadBinding(importProfiles = false) {
    if (busy || refreshing || resizing || disposed) return;
    busy = true; issue = '';
    try { binding = await lazyTester.binding(importProfiles); }
    catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function loadLocation(action:'Inspect'|'Open'|'Locate'|'Choose'='Inspect') {
    if(busy||refreshing||resizing||disposed)return;
    busy=true;issue='';
    try {
      dataLocation=await lazyTester.dataLocation(action);
      if((action==='Locate'||action==='Choose')&&dataLocation.path!==null)binding=await lazyTester.binding();
    } catch(error){issue=String(error);}finally{busy=false;}
  }
  async function openFolder(slot: ProfileSlot) {
    if (busy || refreshing || resizing) return;
    busy = true; issue = '';
    try { await lazyTester.openFolder(slot); }
    catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function clearCache(slot: ProfileSlot) {
    if (busy || refreshing || resizing || clearSlot !== slot || runningCount > 0) return;
    busy = true; issue = ''; storageNotice = ''; clearSlot = null;
    try {
      const snapshot = await lazyTester.clearCache(slot);
      profiles = snapshot.profiles; storage = snapshot.storage;
      storageNotice = 'Web cache for Minecraft - ' + slot + ' cleared.';
    } catch (error) {
      issue = String(error); storage = null;
    } finally { busy = false; }
  }
  onMount(() => {
    mounted = true; void (async () => { await loadBinding(); await loadLocation(); await refresh(); })();
    const interval = setInterval(() => { if (!document.hidden) void refresh(); }, 2000);
    observer = new ResizeObserver(() => { dirty = true; }); observer.observe(viewport, {box: 'device-pixel-content-box'});
    for (const node of images.values()) observer.observe(node);
    const changed = () => { dirty = true; };
    window.addEventListener('resize', changed); document.addEventListener('visibilitychange', changed);
    const stops: (() => void)[] = [];
    const retain = (stop: () => void) => { if (disposed) stop(); else stops.push(stop); };
    void listen<string>('workspace-surface-error', event => { issue = event.payload; }).then(retain).catch(error => { issue = String(error); });
    void listen<number>('lazy-tester-preview-select', event => {
      if ([1,2,3,4,5].includes(event.payload)) select(event.payload as ProfileSlot);
    }).then(retain).catch(error => { previewIssue = String(error); });
    return () => {
      mounted = false; disposed = true; clearInterval(interval); observer?.disconnect();
      window.removeEventListener('resize', changed); document.removeEventListener('visibilitychange', changed);
      stops.forEach(stop => stop()); void workspaceSurface.release().catch(() => {});
    };
  });
</script>

<svelte:head><title>Lazy Tester</title></svelte:head>
<div class="lt-app">
  <aside class="lt-sidebar">
    <a class="lt-brand" href="#workspace" on:click|preventDefault={() => navigate('workspace')}><span>LT</span> Lazy Tester</a>
    <p class="lt-caption">MINECRAFT EDUCATION</p>
    <nav aria-label="Main navigation">
      {#each [['workspace','Workspace'],['launchers','Launchers'],['storage','Storage'],['settings','Settings']] as [id,label]}
        <button class:chosen={page === id} disabled={busy || resizing} on:click={() => navigate(id as typeof page)}>{label}</button>
      {/each}
    </nav>
    <div class="lt-sidebar-bottom"><span class="lt-dot" class:waiting={!available}></span>{available ? 'Connected' : 'Connecting'}<p>{available ? runningCount + ' of 5 running' : 'Session status unavailable'}</p></div>
  </aside>
  <main>
    <header class="lt-page-header"><div><p class="lt-caption">MINECRAFT EDUCATION</p><h1>{page === 'workspace' ? 'Workspace' : page === 'launchers' ? 'Minecraft launchers' : page === 'storage' ? 'Storage' : 'Settings'}</h1></div><button class="lt-secondary" disabled={busy || refreshing || resizing} on:click={refresh}>Refresh</button></header>
    {#if issue || connectionIssue}<div class="lt-alert" role="alert"><strong>Something needs your attention</strong>{#if issue}<p>{issue}</p>{/if}{#if connectionIssue}<p>{connectionIssue}</p>{/if}<span>Your games and data have been preserved.</span></div>{/if}
    {#if !binding?.configured}
      <section class="lt-binding" aria-label="Profile setup">
        <div><strong>Profile setup required</strong><p>{binding?.issue ?? 'Checking profiles…'}</p></div>
        <div class="lt-actions"><button disabled={busy || refreshing || resizing || runningCount > 0} on:click={() => loadBinding(true)}>Locate previous launcher</button><button class="lt-secondary" disabled={busy || refreshing || resizing} on:click={() => loadBinding()}>Try again</button></div>
      </section>
    {/if}
    <section class="lt-workspace" hidden={page !== 'workspace'} aria-label="Minecraft workspace">
      <header class="lt-stage-header"><div><h2>{active ? 'Minecraft - ' + active : requested ? 'Starting Minecraft - ' + requested : 'Choose a Minecraft to start'}</h2><p>{active ? 'Click the game to play' : detached ? 'Game windows restored to the desktop.' : 'Open a profile from Launchers.'}</p></div>
        <div class="lt-actions"><button class="lt-secondary" disabled={busy || !available || !playableCount} on:click={() => step(-1)} aria-label="Previous Minecraft">←</button><button class="lt-secondary" disabled={busy || !available || !playableCount} on:click={() => step(1)}>Next →</button>{#if runningCount}<button class="lt-secondary" disabled={busy || resizing} on:click={release}>Restore windows</button>{/if}</div>
      </header>
      <div class="lt-viewport" bind:this={viewport}>{#if !active}<div class="lt-empty"><span>LT</span><h2>{requested ? 'Starting Minecraft - ' + requested : 'Choose a Minecraft'}</h2><p>{requested ? 'Waiting for the game window…' : 'Launch a profile to begin.'}</p>{#if !requested}<button on:click={() => navigate('launchers')}>Open launchers</button>{/if}</div>{/if}</div>
      <div class="lt-filmstrip" aria-label="Other Minecraft sessions">
        {#each others as profile (profile.slot)}
          <button class="lt-card" aria-label={'Play Minecraft - ' + profile.slot} disabled={busy || !available || profile.state !== 'RUNNING'} on:click={() => select(profile.slot)}>
            <span class="lt-card-picture" use:image={profile.slot} aria-hidden="true">{profile.slot}</span><strong>Minecraft - {profile.slot}</strong><small>{labels[profile.state]}</small>
            {#if profile.state === 'RUNNING' && previewStatuses.find(tile => tile.slot === profile.slot)?.reason}<small class="lt-preview-fallback" title={previewStatuses.find(tile => tile.slot === profile.slot)?.reason ?? ''}>Preview unavailable · click to play</small>{/if}
          </button>
        {/each}
      </div>
      <p class="lt-help">{resizing ? 'Updating layout…' : 'Select a preview to switch games.'}</p>
      {#if previewIssue}<details class="lt-preview-details"><summary>Preview details</summary><p>{previewIssue}</p></details>{/if}
    </section>
    {#if page === 'launchers'}
      <p class="lt-description">Choose which profiles to launch.</p>
      <div class="lt-profile-list">{#each profiles as profile (profile.slot)}<article><span class="lt-profile-number">{profile.slot}</span><div><h2>Minecraft - {profile.slot}</h2><p>{labels[profile.state]}</p>{#if profile.issue}<p class="lt-error">{profile.issue}</p>{/if}</div>{#if profile.state === 'RUNNING'}<button disabled={busy || !available} on:click={() => select(profile.slot)}>Show</button>{:else}<button disabled={busy || refreshing || resizing || !available || !binding?.configured || ['STARTING','NOT_RESPONDING'].includes(profile.state)} on:click={() => launch(profile.slot)}>{profile.state === 'STARTING' ? 'Starting…' : profile.state === 'NOT_RESPONDING' ? 'Not responding' : 'Launch'}</button>{/if}</article>{/each}</div>
    {:else if page === 'storage'}
      <section class="lt-panel">
        <h2>Data location</h2>
        <p class="lt-data-path">{dataLocation?.path ?? 'No data folder is connected.'}</p>
        <div class="lt-actions">
          <button class="lt-secondary" disabled={busy||refreshing||resizing||!dataLocation?.path} on:click={()=>loadLocation('Open')}>Open folder</button>
          <button class="lt-secondary" disabled={busy||refreshing||resizing||dataLocation?.canLocate===false} on:click={()=>loadLocation('Locate')}>Locate data</button>
          {#if !dataLocation?.path}<button disabled={busy||refreshing||resizing} on:click={()=>loadLocation('Choose')}>Choose data folder</button>{/if}
          <button class="lt-secondary" disabled={busy||refreshing||resizing} on:click={()=>loadLocation()}>Refresh location</button>
        </div>
        {#if dataLocation}<details><summary>Profile locations</summary>{#each dataLocation.profiles as profile}<p class="lt-data-path">Minecraft {profile.slot}: {profile.path}</p>{/each}<p class="lt-data-path">Default: {dataLocation.defaultPath}</p></details>{/if}
        <h2>Minecraft storage</h2>
        <p>View storage use for each profile.</p>
        <p><button disabled={busy || refreshing || resizing || !binding?.configured} on:click={inspectStorage}>{busy ? 'Working…' : 'Check storage'}</button></p>
        {#if storageNotice}<p role="status">{storageNotice}</p>{/if}
        {#if storage}
          {#each storage as item}
            <article class="lt-storage-profile">
              <h3>Minecraft - {item.slot}</h3>
              <div class="lt-storage-metrics">
                <div><span>Profile data</span><strong>{(item.bytes / 1048576).toFixed(1)} MB</strong><small>{item.files.toLocaleString('en-US')} file{item.partial ? ' · partial count' : ''}</small></div>
                <div><span>Available disk space</span><strong>{item.disk ? (item.disk.availableBytes / 1073741824).toFixed(1) + ' GB' : 'Unavailable'}</strong><small>{item.disk ? 'of ' + (item.disk.totalBytes / 1073741824).toFixed(1) + ' GB · shared drive' : 'Disk information unavailable'}</small></div>
                <div><span>Web cache</span><strong>{item.cache.partial ? 'Unavailable' : (item.cache.bytes / 1048576).toFixed(1) + ' MB'}</strong><small>Included in profile size</small></div>
              </div>
              {#if item.cache.issue}<p>{item.cache.issue}</p>{/if}
              <div class="lt-actions">
                <button class="lt-secondary" disabled={busy || refreshing || resizing} on:click={() => openFolder(item.slot)}>Open folder</button>
                <button class="lt-secondary" disabled={busy || refreshing || resizing || runningCount > 0 || !item.cache.canClear} on:click={() => clearSlot = item.slot}>Clear cache</button>
              </div>
              {#if clearSlot === item.slot && item.cache.canClear && runningCount === 0}
                <div class="lt-cache-confirm" role="group" aria-label={'Confirm cache cleanup for Minecraft - ' + item.slot}>
                  <p>Clear {(item.cache.bytes / 1048576).toFixed(1)} MB of web cache for Minecraft - {item.slot}? Accounts, worlds and settings will be kept. Web content may load more slowly next time.</p>
                  <div class="lt-actions"><button disabled={busy || refreshing || resizing} on:click={() => clearCache(item.slot)}>Clear web cache</button><button class="lt-secondary" disabled={busy} on:click={() => clearSlot = null}>Cancel</button></div>
                </div>
              {/if}
            </article>
          {/each}
        {/if}
        <p>Close all Minecraft sessions before clearing web cache.</p>
      </section>
    {:else if page === 'settings'}
      <section class="lt-panel" aria-label="Settings" aria-busy={applyingPreset||savingPerformance}>
        <h2>Performance</h2>
        {#if settings}
          <form on:submit|preventDefault={savePerformance}>
            <label class="lt-setting"><span>Active FPS limit</span><input type="number" min="1" max="1000" step="1" required bind:value={activeFps} on:input={()=>performanceNotice=''} disabled={busy} aria-describedby="fps-help" /></label>
            <label class="lt-setting"><span>Background FPS limit</span><input type="number" min="1" max={activeFps??1000} step="1" required bind:value={backgroundFps} on:input={()=>performanceNotice=''} disabled={busy} aria-describedby="fps-help" /></label>
            <p id="fps-help" class="lt-setting-help">Shared by all five profiles. Saved limits apply at the next launch; running games keep their current limits.</p>
            {#if !performanceValid}<p class="lt-error" role="alert">Enter whole numbers from 1 to 1000. Background FPS must not exceed active FPS.</p>{/if}
            <div class="lt-actions lt-setting-actions"><button type="submit" disabled={busy||refreshing||resizing||!available||!performanceDirty||!performanceValid}>{savingPerformance?'Saving…':'Save'}</button><button type="button" class="lt-secondary" disabled={busy} on:click={()=>{activeFps=144;backgroundFps=10;performanceNotice='';}}>Restore defaults</button></div>
            {#if performanceDirty}<p class="lt-setting-help">Unsaved changes</p>{/if}
            {#if performanceNotice}<p class="lt-setting-help" role="status">{performanceNotice}</p>{/if}
          </form>
          <p class="lt-setting-help">The in-game counter shows the actual frame rate.</p>
        {:else}<p>{refreshing?'Loading settings…':'Settings unavailable. Use Refresh to try again.'}</p>{/if}
        <h2>Graphics preset</h2>
        <div class="lt-setting"><span>Graphics mode</span><strong>Simple</strong></div>
        <div class="lt-setting"><span>Render distance</span><strong>{settings?settings.renderDistanceChunks+' chunks':'Unavailable'}</strong></div>
        <div class="lt-setting"><span>Visual effects</span><strong>Reduced</strong></div>
        <div class="lt-setting"><span>Simulation distance</span><strong>Unchanged</strong></div>
        <p class="lt-setting-help">Saved in each profile and applied again at launch. Close all Minecraft sessions to apply it now.</p>
        <div class="lt-actions lt-setting-actions"><button disabled={busy||refreshing||resizing||!available||!binding?.configured||!settings?.canApplyPreset||runningCount>0} on:click={applyPreset}>{applyingPreset?'Applying…':'Apply to all profiles'}</button></div>
        {#if settingsNotice}<p class="lt-setting-help" role="status">{settingsNotice}</p>{/if}
        {#if !binding?.configured}<p class="lt-setting-help">Complete profile setup before applying the preset.</p>{:else if settings&&!settings.canApplyPreset}<p class="lt-setting-help">Close all Minecraft sessions normally, then refresh settings.</p>{/if}
        <h2>Profile data</h2><p>View profile locations, storage use and cache controls in Storage.</p>
        <div class="lt-actions lt-setting-actions"><button class="lt-secondary" disabled={busy||refreshing||resizing} on:click={()=>navigate('storage')}>Manage storage</button></div>
        <h2>When Lazy Tester closes</h2><p>Minecraft keeps running and its windows return to the desktop.</p>
      </section>
    {/if}
  </main>
</div>
