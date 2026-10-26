<script lang="ts">
  import {onMount, tick} from 'svelte';
  import {listen} from '@tauri-apps/api/event';
  import {lazyTester, nextProfile, validFpsLimits, type GraphicsPreset, type ProfileSession, type ProfileSlot, type ProfileStorage, type PreviewStatus, type ProfileBinding, type DataLocation, type SettingsReport} from './app/bridge/lazyTester.js';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import './lazy-tester.css';

  let profiles: ProfileSession[] = [], storage: ProfileStorage[] | null = null;
  let binding: ProfileBinding | null = null;
  let bindingIssue = '', locationIssue = '';
  let dataLocation:DataLocation|null=null;
  let settings:SettingsReport|null=null, settingsNotice='';
  let applyingPreset=false;
  let selectedSlot:ProfileSlot=1, graphicsPreset:GraphicsPreset='Low';
  let advanced=false;
  $: selectedStorage=storage?.find(item=>item.slot===selectedSlot);
  $: selectedProfile=profiles.find(item=>item.slot===selectedSlot);
  $: selectedPath=dataLocation?.profiles.find(item=>item.slot===selectedSlot)?.path;
  $: graphicsDirty=!!settings?.graphicsPresets&&graphicsPreset!==settings.graphicsPresets[selectedSlot-1];
  function chooseSlot(slot:ProfileSlot){selectedSlot=slot;clearSlot=null;settingsNotice='';graphicsPreset=settings?.graphicsPresets?.[slot-1]??'Low';}
  let savingPerformance=false;
  let activeFps:number|undefined=144,backgroundFps:number|undefined=10,performanceNotice='';
  $: performanceDirty=settings!==null&&(activeFps!==settings.activeFps||backgroundFps!==settings.backgroundFps);
  $: performanceValid=validFpsLimits(activeFps,backgroundFps);
  let clearSlot: ProfileSlot | null = null, storageNotice = '';
  type Page = 'workspace' | 'launchers' | 'storage' | 'settings';
  let page: Page = 'workspace';
  let queuedPage: Page | null = null;
  let initializing = true, checkingStorage = false, storageCheckedAt = '';
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
  $: if (mounted && !initializing && !busy && !refreshing && !resizing && queuedPage !== null) void consumeNavigation();
  $: if (mounted && !busy && !refreshing && !resizing && queuedPage === null && queued !== null) void consumeSelection();
  $: if (mounted && !busy && !refreshing && !resizing && queuedPage === null && queued === null && dirty) void syncLayout();

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
        if (profile?.state === 'RUNNING') {
          if (page === 'workspace' && queuedPage === null) queued = requested;
          requested = null;
        }
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
  function navigate(next: typeof page) {
    if(next==='launchers')next='workspace';
    queuedPage = next; queued = null;
  }
  async function consumeNavigation() {
    const next = queuedPage; queuedPage = null;
    if (next === null || page === next) return;
    busy = true; issue = '';
    try {
      if (next !== 'workspace') await workspaceSurface.suspend();
      page = next; await tick();
      if (next === 'workspace' && active !== null) await lazyTester.present(active, viewport.getBoundingClientRect());
      if(next==='settings') {
        const snapshot=await lazyTester.settings();profiles=snapshot.profiles;settings=snapshot.settings;available=true;
        if(!performanceDirty){activeFps=settings.activeFps;backgroundFps=settings.backgroundFps;}
        graphicsPreset=settings.graphicsPresets?.[selectedSlot-1]??'Low';
      }
      if(next==='storage'||next==='settings') {
        try {dataLocation=await lazyTester.dataLocation();locationIssue='';}
        catch(error){locationIssue=String(error);dataLocation=null;storage=null;binding=null;}
      }
      if(next==='storage'&&binding?.configured) {
        const snapshot=await lazyTester.storage();profiles=snapshot.profiles;storage=snapshot.storage;
        storageCheckedAt=new Date().toLocaleTimeString('en-US');
      }
      dirty = true;
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function refreshPage() {
    if (busy || refreshing || resizing) return;
    issue = '';
    if (page === 'storage') {
      busy=true;checkingStorage=true;
      try {
        if(!binding?.configured)await checkBinding();
        try{dataLocation=await lazyTester.dataLocation();locationIssue='';}
        catch(error){locationIssue=String(error);dataLocation=null;storage=null;binding=null;throw error;}
        if(binding?.configured){const snapshot=await lazyTester.storage();profiles=snapshot.profiles;storage=snapshot.storage;storageCheckedAt=new Date().toLocaleTimeString('en-US');available=true;connectionIssue='';}
      }catch(error){issue=String(error);storage=null;}
      finally{checkingStorage=false;busy=false;}
    } else {
      if (!binding?.configured) await loadBinding();
      await refresh();
    }
  }
  async function applyPreset() {
    if(busy||refreshing||resizing||!available||!binding?.configured||!settings?.canApplyPreset||runningCount>0)return;
    busy=true;applyingPreset=true;issue='';settingsNotice='';
    try {
      const snapshot=await lazyTester.applyGraphics(selectedSlot,graphicsPreset);
      profiles=snapshot.profiles;settings=snapshot.settings;
      if(settings.changedProfiles===null)throw new Error('The preset update was not confirmed. Refresh settings before trying again.');
      settingsNotice='Minecraft - '+selectedSlot+': '+graphicsPreset+' saved and applied. This preset is reapplied at launch.';
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
      await workspaceSurface.release(); active = null; selectedIdentity = ''; requested = null; queued = null;
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
  function returnToWorkspace() {
    const slot = profiles.find(profile => profile.state === 'RUNNING')?.slot;
    if (slot) select(slot);
    else { detached = false; dirty = true; }
  }
  async function inspectStorage() {
    if (busy || refreshing || resizing) return;
    busy = true; checkingStorage = true; issue = ''; clearSlot = null; storageNotice = '';
    try {
      const snapshot = await lazyTester.storage(); profiles = snapshot.profiles; storage = snapshot.storage;
      available = true; connectionIssue = '';
      storageCheckedAt = new Date().toLocaleTimeString('en-US');
    }
    catch (error) { issue = String(error); storage = null; }
    finally { checkingStorage = false; busy = false; }
  }
  async function loadBinding(importProfiles = false) {
    if (busy || refreshing || resizing || disposed) return;
    busy = true; issue = '';
    try { await checkBinding(importProfiles); }
    finally { busy = false; }
  }
  async function checkBinding(importProfiles = false) {
    try { binding = await lazyTester.binding(importProfiles); bindingIssue = ''; }
    catch (error) { binding = null; bindingIssue = String(error); }
  }
  async function loadLocation(action:'Inspect'|'Open'|'Locate'|'Choose'='Inspect') {
    if(busy||refreshing||resizing||disposed)return;
    busy=true;issue='';locationIssue='';
    try {
      const previousPath = dataLocation?.path;
      dataLocation=await lazyTester.dataLocation(action);
      if (dataLocation.path !== previousPath) { storage = null; clearSlot = null; storageNotice = ''; }
      if((action==='Locate'||action==='Choose')&&dataLocation.path!==null)await checkBinding();
    } catch(error){
      locationIssue=String(error);
      if(action==='Inspect'){
        dataLocation=null;storage=null;clearSlot=null;storageNotice='';
        binding=null;if(!bindingIssue)bindingIssue=locationIssue;
      }
    }finally{busy=false;}
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
      storageCheckedAt = new Date().toLocaleTimeString('en-US');
      storageNotice = 'Web cache for Minecraft - ' + slot + ' cleared.';
    } catch (error) {
      issue = String(error); storage = null;
    } finally { busy = false; }
  }
  onMount(() => {
    mounted = true; void (async () => {
      try { await loadBinding(); await loadLocation(); await refresh(); }
      finally { initializing = false; }
    })();
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
  <header class="lt-topbar">
    <a class="lt-brand" href="#play" on:click|preventDefault={() => navigate('workspace')}><span>LT</span>Lazy Tester</a>
    <nav aria-label="Main navigation"><button class:chosen={page==='workspace'} aria-current={page==='workspace'?'page':undefined} on:click={()=>navigate('workspace')}>Play</button><button class:chosen={page!=='workspace'} aria-current={page!=='workspace'?'page':undefined} on:click={()=>navigate('settings')}>Settings</button></nav>
    <span class="lt-connection" role="status"><span class="lt-dot" class:waiting={!available}></span>{available?'Connected':initializing?'Connecting…':'Unavailable'}</span>
  </header>
  <div class="lt-layout" class:lt-settings-layout={page!=='workspace'}>
    {#if page!=='workspace'}<aside class="lt-sidebar"><h2>Settings</h2><nav aria-label="Settings navigation"><button class:chosen={page==='settings'} aria-current={page==='settings'?'page':undefined} on:click={()=>navigate('settings')}>General</button><button class:chosen={page==='storage'} aria-current={page==='storage'?'page':undefined} on:click={()=>navigate('storage')}>Data Storage</button></nav></aside>{/if}
    <main>
      <header class="lt-page-header"><h1>{page==='workspace'?'Play':page==='storage'?'Data Storage':'General'}</h1><button class="lt-secondary" disabled={busy||refreshing||resizing} on:click={refreshPage}>{refreshing||checkingStorage?'Refreshing…':'Refresh'}</button></header>
      {#if issue||connectionIssue}<div class="lt-alert" role="alert">{#if issue}<p>{issue}</p>{/if}{#if connectionIssue}<p>{connectionIssue}</p>{/if}</div>{/if}
      {#if !binding?.configured}<section class="lt-binding"><div><strong>{initializing?'Checking profiles…':'Profile setup required'}</strong><p>{bindingIssue||binding?.issue||'Connect your existing launcher data to continue.'}</p></div>{#if !initializing}<div class="lt-actions"><button on:click={()=>navigate('storage')}>Manage data</button><button class="lt-secondary" disabled={busy||runningCount>0} on:click={()=>loadBinding(true)}>Locate previous launcher</button></div>{/if}</section>{/if}
      <section class="lt-workspace" hidden={page!=='workspace'} aria-label="Minecraft play area">
        <header class="lt-stage-header"><div><h2>{detached?'Separate windows':active?'Minecraft - '+active:requested?'Starting Minecraft - '+requested:'Choose a Minecraft below'}</h2><p>{detached?'Your games keep running on the desktop.':active?'Click the game to play.':'Each launcher keeps its own data.'}</p></div>{#if detached}<button class="lt-secondary" disabled={busy||refreshing||resizing||!available} on:click={returnToWorkspace}>Back to Play</button>{:else if runningCount}<button class="lt-secondary" disabled={busy||refreshing||resizing} on:click={release}>Open in separate windows</button>{/if}</header>
        <div class="lt-viewport" class:lt-desktop={detached} bind:this={viewport}>
          {#if detached}<div class="lt-empty"><h2>Games are in separate windows</h2><p>Use the Windows taskbar, or choose a game below to open it in Play.</p><p>{runningCount} of 5 running</p></div>{:else if !active}<div class="lt-empty"><span>LT</span><h2>{initializing?'Getting ready…':!available?'Session status unavailable':requested?'Starting Minecraft - '+requested:'Ready when you are'}</h2><p>{!available?'Use Refresh to reconnect.':requested?'Waiting for the game window…':'Select Launch below to start Minecraft.'}</p></div>{/if}
        </div>
        <div class="lt-games-heading"><span>Your games</span><span>{runningCount} of 5 running</span></div>
        <div class="lt-filmstrip" aria-label="Minecraft launchers">
          {#each profiles as profile (profile.slot)}
            <button class="lt-card" class:selected={active===profile.slot&&!detached} aria-pressed={active===profile.slot&&!detached} aria-label={(profile.state==='RUNNING'?'Open ':'Launch ')+'Minecraft - '+profile.slot} disabled={busy||refreshing||resizing||!available||!binding?.configured||['STARTING','NOT_RESPONDING'].includes(profile.state)} on:click={()=>profile.state==='RUNNING'?select(profile.slot):launch(profile.slot)}>
              <strong>Minecraft - {profile.slot}{active===profile.slot&&!detached?' ✓':''}</strong>
              <span class="lt-card-picture" use:image={profile.slot} aria-hidden="true">{profile.state==='RUNNING'?profile.slot:profile.state==='STARTING'?'Starting…':'▷ Launch'}</span>
              <small>{active===profile.slot&&!detached?'Selected':labels[profile.state]}</small>
              {#if profile.issue}<small class="lt-error">{profile.issue}</small>{/if}
            </button>
          {/each}
        </div>
        <p class="lt-help">{resizing?'Updating display…':detached?'Select a running game to bring it into Play.':'Click a game to switch. Other games keep running.'}</p>
        {#if previewIssue}<details class="lt-preview-details"><summary>Preview unavailable</summary><p>{previewIssue}</p></details>{/if}
      </section>
      {#if page==='settings'}
        <div class="lt-settings-content">
          <section class="lt-group"><h2>Graphics</h2><label class="lt-target">Applies to <select value={selectedSlot} on:change={event=>chooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>Minecraft - {p.slot}</option>{/each}</select></label>
            <div class="lt-setting"><div><h3><label for="graphics-preset">Graphics preset</label></h3><p>High enables visual effects. Low reduces them. Both use 12 chunks; simulation distance is unchanged.</p></div><select id="graphics-preset" bind:value={graphicsPreset} disabled={busy||!settings?.graphicsPresets}><option>High</option><option>Low</option></select></div>
            <div class="lt-actions"><button disabled={busy||refreshing||resizing||!available||!binding?.configured||!settings?.canApplyPreset||!graphicsDirty||runningCount>0} on:click={applyPreset}>{applyingPreset?'Applying…':'Apply preset'}</button></div>
            <p class="lt-help">Close all Minecraft sessions before applying. The saved preset is reapplied when this launcher starts.</p>{#if settings&&!settings.graphicsPresets}<p class="lt-error">The running controller needs an update before High/Low can be selected. Existing games are preserved.</p>{/if}{#if settingsNotice}<p role="status">{settingsNotice}</p>{/if}
          </section>
          <section class="lt-group"><h2>Frame Limits <small>All launchers</small></h2>{#if settings}<form on:submit|preventDefault={savePerformance}>
            <div class="lt-setting"><div><h3><label for="active-fps">While playing</label></h3><p>The game receiving keyboard and mouse input.</p></div><div class="lt-fps"><input id="active-fps" type="number" min="1" max="1000" required bind:value={activeFps} disabled={busy}/> FPS</div></div>
            <div class="lt-setting"><div><h3><label for="background-fps">In the background</label></h3><p>Other games keep running at this limit.</p></div><div class="lt-fps"><input id="background-fps" type="number" min="1" max={activeFps??1000} required bind:value={backgroundFps} disabled={busy}/> FPS</div></div>
            <p class="lt-help">Saved limits apply at the next launch. Running games keep their current limits.</p>{#if !performanceValid}<p class="lt-error" role="alert">Use whole numbers from 1 to 1000. Background FPS cannot exceed active FPS.</p>{/if}
            <div class="lt-actions"><button type="submit" disabled={busy||refreshing||resizing||!available||!performanceDirty||!performanceValid}>{savingPerformance?'Saving…':'Save changes'}</button>{#if performanceDirty}<button type="button" class="lt-secondary" disabled={busy} on:click={()=>{activeFps=settings!.activeFps;backgroundFps=settings!.backgroundFps;}}>Discard</button>{/if}<button type="button" class="lt-secondary" disabled={busy} on:click={()=>{activeFps=144;backgroundFps=10;}}>Use 144 / 10</button></div>{#if performanceNotice}<p role="status">{performanceNotice}</p>{/if}
          </form>{:else}<p>Settings unavailable. Refresh to try again.</p>{/if}</section>
          <section class="lt-group"><h2>App behavior</h2><div class="lt-setting"><div><h3>When Lazy Tester closes</h3><p>Minecraft stays open and returns to its own desktop window. Closing the manager never force-closes your games.</p></div></div><p class="lt-help">Automatic game shutdown is not available yet.</p><div class="lt-setting"><div><h3>Background game audio</h3><p>Mute control requires verified per-game audio support.</p></div><select disabled aria-label="Background game audio"><option>Keep audio</option></select></div></section>
          <section class="lt-group"><h2>Installation & data</h2><h3>Minecraft Education</h3><p>The controller verifies the registered game executable before launch. Moving data does not move Minecraft.</p><h3>Launcher data location</h3><p class="lt-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lt-error">{locationIssue}</p>{/if}<div class="lt-actions"><button class="lt-secondary" disabled={busy||!dataLocation?.path} on:click={()=>loadLocation('Open')}>Open data folder</button><button class="lt-secondary" on:click={()=>navigate('storage')}>Data Storage →</button></div></section>
        </div>
      {:else if page==='storage'}
        <div class="lt-settings-content">
          <p>Worlds, cache and settings for each launcher.</p><div class="lt-launcher-heading"><select value={selectedSlot} aria-label="Choose Minecraft launcher" on:change={event=>chooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>Minecraft - {p.slot}</option>{/each}</select><span>{selectedProfile?labels[selectedProfile.state]:'Unavailable'}</span></div>
          {#if storageNotice}<p role="status">{storageNotice}</p>{/if}
          <section class="lt-group"><div class="lt-setting"><h3>Profile data</h3><strong>{selectedStorage?(selectedStorage.partial?'At least ':'')+(selectedStorage.bytes/1048576).toFixed(1)+' MB':'Not measured'}</strong></div><div class="lt-setting"><h3>Web cache</h3><span>{selectedStorage&&!selectedStorage.cache.partial?(selectedStorage.cache.bytes/1048576).toFixed(1)+' MB':'Unavailable'}</span></div><div class="lt-setting"><h3>Available disk space</h3><span>{selectedStorage?.disk?(selectedStorage.disk.availableBytes/1073741824).toFixed(1)+' GB':'Unavailable'}</span></div><p class="lt-help">{storage?'Last measured at '+storageCheckedAt+'. Web cache is included in profile data.':'Use Refresh to measure storage. World sizes are not separately classified.'}</p></section>
          <section class="lt-setting"><div><h3>Clear cache</h3><p>Remove recognized web cache. Accounts, worlds and game settings are kept.</p>{#if runningCount>0}<p>Close all Minecraft sessions before cleanup.</p>{/if}{#if selectedStorage?.cache.issue}<p>{selectedStorage.cache.issue}</p>{/if}</div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||runningCount>0||!selectedStorage?.cache.canClear} on:click={()=>clearSlot=selectedSlot}>Clear cache…</button></section>
          {#if clearSlot===selectedSlot}<section class="lt-cache-confirm" aria-label="Confirm cache cleanup"><h3>Clear web cache for Minecraft - {selectedSlot}?</h3><p>Worlds, accounts and settings are kept. Other launchers are unchanged.</p><div class="lt-actions"><button disabled={busy||runningCount>0||!selectedStorage?.cache.canClear} on:click={()=>clearCache(selectedSlot)}>Clear web cache</button><button class="lt-secondary" on:click={()=>clearSlot=null}>Cancel</button></div></section>{/if}
          <section class="lt-setting"><div><h3>Data folder</h3><p class="lt-data-path">{selectedPath??'Location unavailable.'}</p></div><button class="lt-secondary" disabled={busy||!available||!binding?.configured} on:click={()=>openFolder(selectedSlot)}>Open folder</button></section>
          <details class="lt-advanced" bind:open={advanced}><summary><span><strong>Advanced options</strong><small>Reset and recovery for this launcher</small></span><span aria-hidden="true">{advanced?'−':'+'}</span></summary><div class="lt-advanced-body"><div class="lt-setting"><div><h3>Reset game settings</h3><p>Requires a verified settings backup and restore path. Existing settings are preserved.</p></div><button class="lt-secondary" disabled>Not available</button></div><div class="lt-setting"><div><h3>Restore settings</h3><p>No verified restore operation is available in this version.</p></div><button class="lt-secondary" disabled>Restore…</button></div></div></details>
          <section class="lt-danger-zone"><div><h3>Clear all data</h3><p>Intended to remove accounts, worlds, cache, settings and launcher backups, leaving the Minecraft installation.</p><p>Unavailable until isolated account cleanup and world export are verified. No data will be deleted.</p></div><button class="lt-secondary lt-danger" disabled>Clear all data…</button></section>
          <details class="lt-location-options"><summary>Connect or locate launcher data</summary><p class="lt-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lt-error">{locationIssue}</p>{/if}<div class="lt-actions"><button class="lt-secondary" disabled={busy||runningCount>0||dataLocation?.canLocate===false} on:click={()=>loadLocation('Locate')}>Locate existing data</button>{#if dataLocation&&!dataLocation.path}<button disabled={busy} on:click={()=>loadLocation('Choose')}>Choose new data folder</button>{/if}</div><p>Connecting data does not migrate or delete existing accounts and worlds.</p></details>
        </div>
      {/if}
    </main>
  </div>
</div>
