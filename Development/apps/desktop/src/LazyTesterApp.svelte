<script lang="ts">
  import {onMount, tick} from 'svelte';
  import {listen} from '@tauri-apps/api/event';
  import {lazyTester, firstEmbeddedProfile, nextProfile, validFpsLimits, type GraphicsPreset, type ProfileSession, type ProfileSlot, type ProfileStorage, type PreviewStatus, parseDwmPreviewInvalidation, dwmPreviewInvalidationApplies, type DwmPreviewInvalidation, type ProfileBinding, type DataLocation, type SettingsReport, type NativeBackendSupport} from './app/bridge/lazyTester.js';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import './lazy-tester.css';
  import WindowTitlebar from './WindowTitlebar.svelte';
  import {closeReviewActive, observationStillCurrent, previewLayoutStillCurrent, type CloseReview} from './app/inspectionScheduling.js';

  let profiles: ProfileSession[] = [], storage: ProfileStorage[] | null = null;
  let binding: ProfileBinding | null = null;
  let nativeBackend: NativeBackendSupport | null = null;
  let bindingIssue = '', locationIssue = '';
  let dataLocation:DataLocation|null=null;
  let settings:SettingsReport|null=null, settingsNotice='';
  let applyingPreset=false;
  let queuedPreset:{slot:ProfileSlot;preset:GraphicsPreset}|null=null;
  let queuedAudio:'Keep'|'Mute'|null=null;
  let selectedSlot:ProfileSlot=1, graphicsPreset:GraphicsPreset='Low';
  let advanced=false;
  $: selectedStorage=storage?.find(item=>item.slot===selectedSlot);
  $: selectedProfile=profiles.find(item=>item.slot===selectedSlot);
  $: selectedPath=dataLocation?.profiles.find(item=>item.slot===selectedSlot)?.path;
  $: graphicsDirty=!!settings?.graphicsPresets&&graphicsPreset!==settings.graphicsPresets[selectedSlot-1];
  function chooseSlot(slot:ProfileSlot){selectedSlot=slot;clearSlot=null;recovery=null;fullReview=null;settingsNotice='';graphicsPreset=settings?.graphicsPresets?.[slot-1]??'Low';}
  let savingPerformance=false;
  let queuedPerformance:{active:number;background:number}|null=null;
  let activeFps:number|undefined=120,backgroundFps:number|undefined=5,performanceNotice='';
  $: performanceDirty=settings!==null&&(activeFps!==settings.activeFps||backgroundFps!==settings.backgroundFps);
  $: performanceValid=validFpsLimits(activeFps,backgroundFps);
  let clearSlot: ProfileSlot | null = null, storageNotice = '';
  let recovery:{slot:ProfileSlot;action:'Reset'|'Restore'}|null=null;
  let fullReview:{slot:ProfileSlot;token:string;pending:boolean}|null=null,fullName='',fullAck=false,clearingData=false;
  type Page = 'workspace' | 'launchers' | 'storage' | 'settings';
  let page: Page = 'workspace';
  let queuedPage: Page | null = null;
  let initializing = true, checkingStorage = false, storageCheckedAt = '';
  let active: ProfileSlot | null = null, requested: ProfileSlot | null = null, queued: ProfileSlot | null = null;
  let viewport: HTMLDivElement, stageSpace: HTMLDivElement, observer: ResizeObserver | undefined;
  let busy = false, refreshing = false, resizing = false, available = false;
  let manualRefreshRequested=false, manualRefreshing=false;
  let queuedLaunch:ProfileSlot|null=null;
  let mounted = false, disposed = false, dirty = false, detached = false;
  let separateSlots = new Set<ProfileSlot>();
  let issue = '', connectionIssue = '', previewIssue = '', selectedIdentity = '';
  // Display state only; the retained Rust controller still owns game lifetime.
  let closeInProgress = false, closeFailure = '';
  // Presentation-only close reviews; never another process or session owner.
  let nativeExitReview = false, observationEpoch = 0;
  let closeOneSlot: ProfileSlot | null = null, closingOneSlot: ProfileSlot | null = null;
  // UI warning only; native presentation authority remains in workspace_surface.
  let uncertainWindowSlot: ProfileSlot | null = null;
  let previewStatuses: PreviewStatus[] = [];
  // Read-only UI observations. A DWM failure does not own the game or
  // request an IPC/layout retry. Epochs prevent an old in-flight layout
  // response from restoring a thumbnail after asynchronous invalidation.
  let previewInvalidationEpoch = 0;
  const pendingPreviewInvalidations = new Map<ProfileSlot, DwmPreviewInvalidation & {epoch:number}>();
  let closeReviewPending = false;
  function reviewState(): CloseReview {
    return {
      nativeExitReview,
      closeAllRunning: closeInProgress,
      selectedReview: closeOneSlot !== null,
      selectedCloseRunning: closingOneSlot !== null,
    };
  }
  // Explicit Svelte dependencies so any close review suspends queued
  // background UI work; it resumes only after a cancel or terminal receipt.
  $: closeReviewPending = closeReviewActive({
    nativeExitReview, closeAllRunning: closeInProgress,
    selectedReview: closeOneSlot !== null,
    selectedCloseRunning: closingOneSlot !== null,
  });
  const images = new Map<ProfileSlot, HTMLElement>();
  const labels = {STOPPED: 'Not running', STARTING: 'Starting', RUNNING: 'Running', NOT_RESPONDING: 'Not responding', ATTENTION: 'Needs attention'};
  const identity = (profile?: ProfileSession) => profile?.process ? [profile.process.pid, profile.process.createdFileTime, profile.process.sessionId, profile.process.windowHandle].join('/') : '';
  const layoutIdentity=(items:ProfileSession[])=>items.map(p=>p.slot+':'+p.state+':'+identity(p)+':'+(p.process?.responding??false)).join('|');
  $: primary = active ?? requested ?? profiles.find(profile => profile.state === 'RUNNING')?.slot ?? 1;
  $: others = profiles.filter(profile => profile.slot !== primary);
  $: runningCount = profiles.filter(profile => ['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(profile.state)).length;
  $: playableCount = profiles.filter(profile => profile.state === 'RUNNING').length;
  $: if (mounted && nativeBackend === 'WindowsNative' && !closeInProgress && !closeReviewPending && !initializing && !busy && !refreshing && !resizing && queuedAudio!==null) {const mode=queuedAudio;queuedAudio=null;void saveAudio(mode);}
  $: if (mounted && nativeBackend === 'WindowsNative' && !closeInProgress && !closeReviewPending && !initializing && !busy && !refreshing && !resizing && queuedAudio===null && queuedPerformance!==null) {const limits=queuedPerformance;queuedPerformance=null;void savePerformance(limits);}
  $: if (mounted && nativeBackend === 'WindowsNative' && !closeInProgress && !closeReviewPending && !initializing && !busy && !refreshing && !resizing && queuedAudio===null && queuedPerformance===null && queuedPreset!==null) {const request=queuedPreset;queuedPreset=null;void applyPreset(request);}
  $: if (mounted && nativeBackend === 'WindowsNative' && !closeInProgress && !closeReviewPending && !initializing && !busy && !refreshing && !resizing && queuedAudio===null && queuedPerformance===null && queuedPreset===null && queuedPage !== null) void consumeNavigation();
  $: if (mounted && nativeBackend === 'WindowsNative' && !closeInProgress && !closeReviewPending && !initializing && !busy && !refreshing && !resizing && queuedPerformance===null && queuedPreset===null && queuedPage===null && manualRefreshRequested) {manualRefreshRequested=false;void refreshPage();}
  $: if (mounted && nativeBackend === 'WindowsNative' && !closeInProgress && !closeReviewPending && !initializing && !busy && !refreshing && !resizing && queuedPerformance===null && queuedPreset===null && queuedPage===null && queuedLaunch!==null) {const slot=queuedLaunch;queuedLaunch=null;void launch(slot);}
  $: if (mounted && nativeBackend === 'WindowsNative' && !closeInProgress && !closeReviewPending && !busy && !refreshing && !resizing && queuedAudio===null && queuedPerformance===null && queuedPreset===null && queuedPage === null && queued !== null) void consumeSelection();
  $: if (mounted && nativeBackend === 'WindowsNative' && !closeInProgress && !closeReviewPending && !busy && !refreshing && !resizing && queuedAudio===null && queuedPerformance===null && queuedPreset===null && queuedPage === null && queued === null && dirty) void syncLayout();

  function image(node: HTMLElement, slot: ProfileSlot) {
    images.set(slot, node); observer?.observe(node); dirty = true;
    return {destroy() { observer?.unobserve(node); if (images.get(slot) === node) images.delete(slot); dirty = true; }};
  }
  function select(slot: ProfileSlot) { closeOneSlot=null; requested = null; queued = slot; }
  async function consumeSelection() {
    const slot = queued; queued = null;
    if (slot === null || !available || profiles.find(profile => profile.slot === slot)?.state !== 'RUNNING') return;
    busy = true; issue = '';
    try {
      page = 'workspace'; await tick();
      await lazyTester.present(slot, viewport.getBoundingClientRect());
      active = slot; selectedIdentity = identity(profiles.find(profile => profile.slot === slot));
      detached = false; requested = null; dirty = true;
      separateSlots = new Set([...separateSlots].filter(value=>value!==slot));
    } catch (error) { issue = String(error); dirty = true; }
    finally { busy = false; }
  }
  async function refresh() {
    if (nativeBackend !== 'WindowsNative' || closeReviewActive(reviewState()) || busy || refreshing || resizing || disposed) return;
    const issued = observationEpoch;
    refreshing = true;
    try {
      const snapshot = page === 'settings' ? await lazyTester.settings() : await lazyTester.inspect();
      // A close review starting mid-request invalidates this old snapshot.
      if (disposed || !observationStillCurrent(issued, observationEpoch, reviewState())) return;
      const layoutChanged=!available||layoutIdentity(profiles)!==layoutIdentity(snapshot.profiles);
      profiles = snapshot.profiles; available = true; connectionIssue = '';
      // STOPPED only confirms a session observation, not restoration of
      // the original HWND/helper checkpoint. Only a positive selected-close
      // presentation receipt below may clear the uncertainty warning.
      if(snapshot.settings){
        if(!performanceDirty){activeFps=snapshot.settings.activeFps;backgroundFps=snapshot.settings.backgroundFps;}
        settings=snapshot.settings;
      }
      if (active !== null && selectedIdentity !== identity(profiles.find(profile => profile.slot === active))) {
        await workspaceSurface.release();
        if (!observationStillCurrent(issued, observationEpoch, reviewState())) return;
        active = null; selectedIdentity = '';
      }
      if (requested !== null) {
        const profile = profiles.find(profile => profile.slot === requested);
        if (profile?.state === 'RUNNING') {
          if (page === 'workspace' && queuedPage === null) queued = requested;
          requested = null;
        }
        else if (profile?.state === 'ATTENTION' || profile?.state === 'STOPPED'
          || (profile?.state === 'NOT_RESPONDING' && profile.issue)) {
          issue = profile.issue ?? ('Minecraft - ' + requested + ' stopped before its window became available.'); requested = null;
        }
      } else if (active === null && !detached && page === 'workspace' && queued === null) {
        queued = firstEmbeddedProfile(profiles,separateSlots);
      }
      if(layoutChanged) {
        previewStatuses=[];pendingPreviewInvalidations.clear();dirty=true;
      }
    } catch (error) {
      if (!disposed && observationStillCurrent(issued, observationEpoch, reviewState())) {
        if (available) dirty = true;
        available = false;previewStatuses=[];pendingPreviewInvalidations.clear();
        connectionIssue = String(error);
      }
    } finally { refreshing = false; }
  }
  async function launch(slot: ProfileSlot) {
    if (closeInProgress || busy || !binding?.configured) return;
    if(refreshing||resizing){queuedLaunch=slot;return;}
    busy = true; issue = ''; requested = slot;
    try {
      profiles = (await lazyTester.launch(slot)).profiles; storage = null; available = true;
      detached = false; page = 'workspace'; dirty = true;
      if (requested === slot && profiles.find(profile => profile.slot === slot)?.state === 'RUNNING') queued = slot;
      else if (active !== null) {
        await tick();
        await lazyTester.present(active, viewport.getBoundingClientRect());
      }
    } catch (error) { issue = String(error); requested = null; dirty = true; }
    finally { busy = false; }
  }
  function navigate(next: typeof page) {
    // Settings and storage require the Windows native session owner.
    if (nativeBackend !== 'WindowsNative') return;
    closeOneSlot=null;recovery=null;clearSlot=null;fullReview=null;
    if(next==='launchers')next='workspace';
    queuedPage = next; queued = null;queuedLaunch=null;
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
    } catch (error) { issue = String(error); dirty = true; }
    finally { busy = false; }
  }
  async function refreshPage() {
    if (nativeBackend !== 'WindowsNative' || busy || refreshing || resizing) return;
    manualRefreshing=true;
    try {
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
      // Only explicit Refresh of a known preview failure requests another
      // DWM layout. Periodic session inspection and worker errors never
      // schedule additional native preview IPC.
      const previewFailed = previewIssue !== '' ||
        pendingPreviewInvalidations.size > 0 ||
        previewStatuses.some(status => !status.registered && status.reason !== null);
      if (!binding?.configured) await loadBinding();
      await refresh();
      if (previewFailed && page === 'workspace' && available &&
          !detached && !disposed) dirty = true;
    }
    }finally{manualRefreshing=false;}
  }
  async function saveAudio(mode:'Keep'|'Mute') {
    if(busy||!settings?.backgroundAudio)return;
    if(refreshing||resizing){queuedAudio=mode;return;}
    busy=true;issue='';settingsNotice='';
    try {const snapshot=await lazyTester.saveAudio(mode);settings=snapshot.settings;profiles=snapshot.profiles;settingsNotice='Background audio saved. Applies to managed Minecraft sessions immediately.';}
    catch(error){issue=String(error);}
    finally{busy=false;}
  }
  function requestPreset() {
    if(busy||queuedPreset!==null||!available||!binding?.configured||!settings?.canApplyPreset||!graphicsDirty||runningCount>0)return;
    queuedPreset={slot:selectedSlot,preset:graphicsPreset};
  }
  async function applyPreset(request:{slot:ProfileSlot;preset:GraphicsPreset}) {
    if(!available||!binding?.configured||!settings?.canApplyPreset||runningCount>0){issue='Graphics preset could not be applied. Close Minecraft sessions and refresh, then try again.';return;}
    busy=true;applyingPreset=true;issue='';settingsNotice='';
    try {
      const snapshot=await lazyTester.applyGraphics(request.slot,request.preset);
      profiles=snapshot.profiles;settings=snapshot.settings;
      if(settings.changedProfiles===null)throw new Error('The preset update was not confirmed. Refresh settings before trying again.');
      settingsNotice='Minecraft - '+request.slot+': '+request.preset+' saved and applied. This preset is reapplied at launch.';
    }catch(error){issue=String(error);settings=null;}
    finally{applyingPreset=false;busy=false;}
  }
  function requestPerformanceSave() {
    if(busy||queuedPerformance!==null||!available||!settings||!performanceValid||!performanceDirty)return;
    queuedPerformance={active:activeFps!,background:backgroundFps!};
  }
  async function savePerformance(limits:{active:number;background:number}) {
    if(!available||!settings){issue='Frame limits could not be saved. Refresh and try again.';return;}
    busy=true;savingPerformance=true;issue='';performanceNotice='';
    try{
      const snapshot=await lazyTester.savePerformance(limits.active,limits.background);
      profiles=snapshot.profiles;settings=snapshot.settings;
      activeFps=settings.activeFps;backgroundFps=settings.backgroundFps;
      performanceNotice='Saved. New limits apply when each Minecraft profile next starts.';
    }catch(error){issue=String(error);}
    finally{savingPerformance=false;busy=false;}
  }
  function requestCloseOne(slot: ProfileSlot) {
    if (closeInProgress || nativeExitReview || busy || closingOneSlot !== null) return;
    const profile=profiles.find(item=>item.slot===slot);
    if (!profile || !['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state)) return;
    // Keep the user's confirmation available while an old Inspect finishes,
    // but never allow that result to overwrite state after review began.
    observationEpoch++;
    if (queuedLaunch===slot) queuedLaunch=null;
    if (queued===slot) queued=null;
    closeOneSlot=slot;
  }
  async function confirmCloseOne() {
    const slot=closeOneSlot;
    if (slot===null || closeInProgress || nativeExitReview || busy ||
        refreshing || resizing || closingOneSlot!==null) return;
    closeOneSlot=null; closingOneSlot=slot; busy=true; issue='';
    try {
      const report=await lazyTester.closeOne(slot);
      if (!report.presentationReleased) {
        // Release was attempted but did not confirm restoration. Do not
        // pretend the previous Play surface still owns the original HWND.
        uncertainWindowSlot = slot;
        issue = `Minecraft - ${slot}: ${report.issue??'Window recovery or game exit has not been confirmed.'}`;
        if (active===slot) {
          active=null;selectedIdentity='';requested=null;queued=null;detached=true;
          previewStatuses=[];
        }
        try {profiles=(await lazyTester.inspect()).profiles;available=true;connectionIssue='';}
        catch(refreshError){available=false;connectionIssue=String(refreshError);}
        return;
      }
      if (uncertainWindowSlot===slot) uncertainWindowSlot=null;
      // Only a confirmed native restoration receipt permits a normal
      // selected close transition and automatic selection of another peer.
      const wasActive=active===slot;
      if (wasActive) {
        active=null;selectedIdentity='';requested=null;queued=null;
        separateSlots=new Set([...separateSlots].filter(value=>value!==slot));
      }
      if (report.snapshot) {
        profiles=report.snapshot.profiles;available=true;connectionIssue='';storage=null;
        if (wasActive) {
          // Existing background games may still be parked offscreen with
          // WS_EX_TOOLWINDOW. Promote another verified responsive peer into
          // Play instead of leaving it inaccessible without a preview.
          queued=firstEmbeddedProfile(profiles,separateSlots);
          detached=queued===null;
        }
      } else {
        issue=`Minecraft - ${slot}: Selected close was not confirmed. ${report.issue??'Refresh to inspect the current session.'}`;
        try {profiles=(await lazyTester.inspect()).profiles;available=true;connectionIssue='';}
        catch(refreshError){available=false;connectionIssue=String(refreshError);}
        if (wasActive) {
          // Never automatically reselect the game whose exit outcome is
          // uncertain; a different responding peer can still be presented.
          const excluded=new Set<ProfileSlot>([...separateSlots,slot]);
          queued=available?firstEmbeddedProfile(profiles,excluded):null;
          detached=queued===null;
        }
      }
    } catch (error) {
      // A transport failure or invalid receipt cannot establish whether the
      // Rust command reached native window restoration. The UI must stop
      // claiming an embedded selected HWND; no extra close/retry is sent.
      uncertainWindowSlot = slot;
      issue=`Minecraft - ${slot}: Close outcome and window restoration were not confirmed. ${String(error)}`;
      if (active===slot) {
        active=null;selectedIdentity='';requested=null;queued=null;detached=true;
        previewStatuses=[];
      }
      // Read-only reconciliation cannot certify window restoration and must
      // not clear uncertainWindowSlot, even when the game has stopped.
      try {profiles=(await lazyTester.inspect()).profiles;available=true;connectionIssue='';}
      catch(refreshError){available=false;connectionIssue=String(refreshError);}
    } finally {
      closingOneSlot=null;busy=false;dirty=true;
    }
  }

  async function release() {
    if (busy || refreshing || resizing) return;
    const slot=active;
    const target=profiles.find(profile=>profile.slot===slot)?.process;
    if(slot===null||!target)return;
    busy = true;
    try {
      await lazyTester.clearPreviews();
      await workspaceSurface.releaseTarget(target); active = null; selectedIdentity = ''; requested = null; queued = null;
      separateSlots=new Set([...separateSlots,slot]);
      queued=firstEmbeddedProfile(profiles,separateSlots);
      detached = queued===null; previewStatuses = []; dirty = true;
    } catch (error) { issue = String(error); }
    finally { busy = false; }
  }
  async function syncLayout() {
    if (nativeBackend !== 'WindowsNative' || closeReviewActive(reviewState()) || busy || refreshing || resizing || !mounted) return;
    // This UI snapshot grants no native process/window authority. The
    // original selected slot and process set must still match on return.
    const issued = {
      observationEpoch,
      processIdentity: layoutIdentity(profiles),
      primarySlot: primary,
    };
    const layoutCurrent = () => previewLayoutStillCurrent(issued,{
      observationEpoch,
      processIdentity: layoutIdentity(profiles),
      primarySlot: primary,
      dirty,
      managerVisible: !document.hidden,
      workspaceAvailable: mounted && !disposed && nativeBackend === 'WindowsNative' &&
        available && page === 'workspace' && !detached,
    },reviewState());
    const discardOutdated = () => {
      previewStatuses = [];
      // Only the existing dirty/ResizeObserver admission may schedule a
      // later valid layout. No direct retry/IPC from a stale response.
      if (mounted && !disposed && nativeBackend === 'WindowsNative' &&
          page === 'workspace' && !detached) dirty = true;
    };
    const invalidationAtStart = previewInvalidationEpoch;
    dirty = false; resizing = true;
    try {
      if (page !== 'workspace' || detached || !available || document.hidden) {
        await lazyTester.clearPreviews(); previewStatuses = [];pendingPreviewInvalidations.clear();previewIssue=''; return;
      }
      await tick();
      if (!layoutCurrent()) {discardOutdated();return;}
      if (active !== null) await workspaceSurface.update(viewport.getBoundingClientRect());
      if (!layoutCurrent()) {discardOutdated();return;}
      // DWM monitoring tiles require an observed RUNNING process, not
      // merely an existing launcher profile or stale session state.
      const tiles = others.filter(profile=>profile.state==='RUNNING'&&!!profile.process&&!separateSlots.has(profile.slot)).flatMap(profile => {
        const element = images.get(profile.slot); if (!element) return [];
        const rect = element.getBoundingClientRect();
        return rect.x >= 0 && rect.y >= 0 && rect.width >= 32 && rect.height >= 24 &&
          rect.right <= window.innerWidth && rect.bottom <= window.innerHeight ? [{slot: profile.slot, rect}] : [];
      });
      if (!layoutCurrent()) {discardOutdated();return;}
      const response = (await lazyTester.previews(viewport.getBoundingClientRect(), tiles)).tiles;
      if (!layoutCurrent()) {discardOutdated();return;}
      // An invalidation received AFTER this layout began is newer evidence
      // than its eventual PreviewReport. Never let that older response
      // re-mark MC02 as registered. Only another genuine layout can clear it.
      previewStatuses = response.map(status => {
        const notice = pendingPreviewInvalidations.get(status.slot);
        if(notice && notice.epoch > invalidationAtStart &&
          dwmPreviewInvalidationApplies(notice,profiles.find(p=>p.slot===status.slot))) {
          return {...status,registered:false,reason:notice.reason};
        }
        return status;
      });
      for(const [slot, notice] of pendingPreviewInvalidations) {
        if(notice.epoch <= invalidationAtStart ||
          !dwmPreviewInvalidationApplies(notice,profiles.find(p=>p.slot===slot))) {
          pendingPreviewInvalidations.delete(slot);
        }
      }
      previewIssue = '';
    } catch (error) { previewStatuses = [];previewIssue = String(error); }
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
    busy = true; checkingStorage = true; issue = ''; clearSlot = null; recovery=null; fullReview=null; storageNotice = '';
    try {
      const snapshot = await lazyTester.storage(); profiles = snapshot.profiles; storage = snapshot.storage;
      available = true; connectionIssue = '';
      storageCheckedAt = new Date().toLocaleTimeString('en-US');
    }
    catch (error) { issue = String(error); storage = null; }
    finally { checkingStorage = false; busy = false; }
  }
  async function loadBinding(importProfiles = false) {
    if (nativeBackend !== 'WindowsNative' || busy || refreshing || resizing || disposed) return;
    busy = true; issue = '';
    try { await checkBinding(importProfiles); }
    finally { busy = false; }
  }
  async function initializeLaunchers() {
    initializing=true;
    try {
      // Read a non-mutating capability from Rust. Unsupported platforms do
      // not retry Windows profile discovery or request game/session IPC.
      try {
        nativeBackend = await lazyTester.backendSupport();
      } catch (error) {
        if (!disposed) {binding=null;bindingIssue='Native backend capability could not be verified: '+String(error);}
        return;
      }
      if (disposed) return;
      if (nativeBackend === 'Unsupported') {
        binding=null;
        bindingIssue='Minecraft Education native session management is not implemented on this platform. No game, account or profile data was accessed.';
        queuedPage=null;queuedLaunch=null;queued=null;manualRefreshRequested=false;dirty=false;
        return;
      }
      // Retry the canonical Windows initializer before offering recovery.
      for(let attempt=0;attempt<3&&!disposed;attempt++){
        if(attempt)await new Promise(resolve=>setTimeout(resolve,attempt*1000));
        if(disposed)return;
        await loadBinding();
        if(binding?.configured){
          await loadLocation();
          if(binding?.configured){await refresh();return;}
        }
      }
      if(!disposed)await refresh();
    }finally{
      if(!disposed){
        initializing=false;await tick();
        void lazyTester.reportStartup({
          connection:document.querySelector('.lt-connection')?.textContent?.trim()??'',
          issue:document.querySelector('.lt-binding p')?.textContent??'',
          cards:document.querySelectorAll('.lt-card').length,
          enabled:document.querySelectorAll('.lt-card:not(:disabled)').length,
        }).catch(error=>console.warn('Startup observation unavailable:',String(error)));
      }
    }
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
    if (busy || refreshing || resizing || clearSlot !== slot || !storage?.find(item=>item.slot===slot)?.cache.canClear) return;
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
  async function recoverSettings() {
    const pending=recovery;
    if(!pending||busy||refreshing||resizing||pending.slot!==selectedSlot)return;
    const status=selectedStorage?.gameSettings;
    if(!(pending.action==='Reset'?status?.canReset:status?.canRestore))return;
    busy=true;issue='';storageNotice='';recovery=null;
    try {
      const snapshot=await lazyTester.recoverSettings(pending.slot,pending.action);
      profiles=snapshot.profiles;storage=snapshot.storage;
      storageCheckedAt=new Date().toLocaleTimeString('en-US');
      storageNotice=`Minecraft - ${pending.slot}: settings ${pending.action==='Reset'?'reset':'restored'}. Changes apply at the next launch; your saved graphics preset is reapplied. Previous settings are available through Restore settings.`;
    } catch(error) {issue=String(error);storage=null;}
    finally {busy=false;}
  }
  function reviewFullClear() {
    const status=selectedStorage?.fullClear;
    if(busy||!status?.canClear||!status.confirmationToken)return;
    recovery=null;clearSlot=null;fullName='';fullAck=false;
    fullReview={slot:selectedSlot,token:status.confirmationToken,pending:status.pending};
    void tick().then(()=>document.getElementById('full-clear-name')?.focus());
  }
  function reviewSettings(action:'Reset'|'Restore') {
    fullReview=null;clearSlot=null;recovery={slot:selectedSlot,action};
  }
  async function clearAllData() {
    const pending=fullReview,status=selectedStorage?.fullClear;
    if(!pending||busy||refreshing||resizing||!fullAck||pending.slot!==selectedSlot||fullName!==`Minecraft - ${pending.slot}`||
      !status?.canClear||status.confirmationToken!==pending.token)return;
    busy=true;clearingData=true;issue='';storageNotice='';fullReview=null;
    try {
      const snapshot=await lazyTester.clearData(pending.slot,pending.token,fullName);
      profiles=snapshot.profiles;storage=snapshot.storage;
      storageCheckedAt=new Date().toLocaleTimeString('en-US');
      storageNotice=`Minecraft - ${pending.slot}: local data cleared. Its next launch uses an empty profile. Other launchers, the Minecraft installation and app preferences are kept.`;
    } catch(error) {issue=String(error);storage=null;}
    finally {busy=false;clearingData=false;}
  }
  onMount(() => {
    mounted = true; void initializeLaunchers();
    // Keep a review stable while the user decides; the backend rechecks admission on action.
    const interval = setInterval(() => {
      // A Mac diagnostic shell still has its WebView lifecycle, but it must
      // never schedule Windows game inspection from the periodic observer.
      if (nativeBackend === 'WindowsNative' && !closeReviewActive(reviewState()) && !document.hidden && binding?.configured &&
          clearSlot===null && recovery===null && fullReview===null) void refresh();
    }, 2000);
    observer = new ResizeObserver(() => { dirty = true; }); observer.observe(viewport, {box: 'device-pixel-content-box'});
    observer.observe(stageSpace);
    for (const node of images.values()) observer.observe(node);
    const changed = () => {
      // Hidden/minimized WebViews cannot claim current DWM visibility.
      // Losing focus while the manager remains shown is not a hide.
      if (document.hidden) {
        previewStatuses=[];pendingPreviewInvalidations.clear();
      }
      dirty = true;
    };
    window.addEventListener('resize', changed); document.addEventListener('visibilitychange', changed);
    const stops: (() => void)[] = [];
    const retain = (stop: () => void) => { if (disposed) stop(); else stops.push(stop); };
    void listen<boolean>('lazy-tester-close-review', event => {
      // UI scheduling hint only; OS confirmation still owns the exit decision.
      if (event.payload && !nativeExitReview) observationEpoch++;
      nativeExitReview = event.payload;
      if (!nativeExitReview) dirty = true;
    }).then(retain).catch(error => { issue = String(error); });
    void listen<boolean>('lazy-tester-closing', event => {
      closeInProgress = event.payload;
      if (closeInProgress) {
        observationEpoch++;
        closeOneSlot=null;
        queuedLaunch=null;
        queued=null;
      }
    }).then(retain).catch(error => { issue = String(error); });
    void listen<string>('lazy-tester-close-rejected', event => {
      // Resource admission refused before restoring or closing any game.
      // Keep the verified embedded selection, previews and queued work.
      closeInProgress = false;
      nativeExitReview = false;
      issue = event.payload;
    }).then(retain).catch(error => { issue = String(error); });
    void listen<string>('lazy-tester-close-error', event => {
      // The native close attempt already released/restored workspace leases.
      // Never imply the game is still embedded or automatically reacquire it.
      closeInProgress = false;
      nativeExitReview = false;
      closeFailure = event.payload;
      active = null; selectedIdentity = ''; requested = null; queued = null; queuedLaunch = null;
      separateSlots = new Set<ProfileSlot>();
      previewStatuses = []; detached = true; page = 'workspace'; dirty = true;
      void refresh(); // Observation only; does not start or capture Minecraft.
    }).then(retain).catch(error => { issue = String(error); });
    void listen<string>('workspace-surface-error', event => {
      try {
        const notice=parseDwmPreviewInvalidation(event.payload);
        if(!notice) {issue=event.payload;return;}
        if (disposed || nativeBackend!=='WindowsNative' ||
          !dwmPreviewInvalidationApplies(notice,profiles.find(p=>p.slot===notice.slot))) return;
        previewInvalidationEpoch++;
        pendingPreviewInvalidations.set(notice.slot,{...notice,epoch:previewInvalidationEpoch});
        // Preserve peer statuses and avoid new preview RPC/layout work.
        previewStatuses=previewStatuses.map(status=>status.slot===notice.slot?
          {...status,registered:false,reason:notice.reason}:status);
      } catch (error) {issue=String(error);}
    }).then(retain).catch(error => { issue = String(error); });
    void listen<number>('lazy-tester-preview-select', event => {
      if ([1,2,3,4,5].includes(event.payload)) select(event.payload as ProfileSlot);
    }).then(retain).catch(error => { previewIssue = String(error); });
    return () => {
      mounted = false; disposed = true; clearInterval(interval); observer?.disconnect();
      window.removeEventListener('resize', changed); document.removeEventListener('visibilitychange', changed);
      stops.forEach(stop => stop());pendingPreviewInvalidations.clear();
      // Unsupported diagnostic UI holds no native game-window lease.
      if (nativeBackend === 'WindowsNative') void workspaceSurface.release().catch(() => {});
    };
  });
</script>
<svelte:head><title>Lazy Tester</title></svelte:head>
<div class="lt-app">
  <WindowTitlebar backendSupport={nativeBackend} />
  <header class="lt-topbar">
    <a class="lt-brand" href="#play" on:click|preventDefault={() => navigate('workspace')}><span>LT</span>Lazy Tester</a>
    <nav aria-label="Main navigation"><button disabled={closeInProgress} class:chosen={page==='workspace'} aria-current={page==='workspace'?'page':undefined} on:click={()=>navigate('workspace')}>Play</button><button disabled={closeInProgress||nativeBackend!=='WindowsNative'} class:chosen={page!=='workspace'} aria-current={page!=='workspace'?'page':undefined} on:click={()=>navigate('settings')}>Settings</button></nav>
    <span class="lt-connection" role="status"><span class="lt-dot" class:waiting={!available||!binding?.configured}></span>{initializing?'Preparing launchers…':nativeBackend==='Unsupported'?'Game backend unavailable':!binding?.configured?'Connection unavailable':available?'Connected':'Unavailable'}</span>
  </header>
  <div class="lt-layout" class:lt-settings-layout={page!=='workspace'} inert={closeInProgress}>
    {#if page!=='workspace'}<aside class="lt-sidebar"><h2>Settings</h2><nav aria-label="Settings navigation"><button class:chosen={page==='settings'} aria-current={page==='settings'?'page':undefined} on:click={()=>navigate('settings')}>General</button><button class:chosen={page==='storage'} aria-current={page==='storage'?'page':undefined} on:click={()=>navigate('storage')}>Data Storage</button></nav></aside>{/if}
    <main>
      <header class="lt-page-header" class:lt-play-header={page==='workspace'}>
        <div><h1>{page==='workspace'?(detached?(uncertainWindowSlot!==null?'Window recovery unconfirmed':'Separate windows'):active?'Minecraft - '+active:requested?'Starting Minecraft - '+requested:binding?.configured?'Choose a Minecraft below':initializing?'Preparing your launchers':nativeBackend==='Unsupported'?'Minecraft backend unavailable':'Launcher connection unavailable'):page==='storage'?'Data Storage':'General'}</h1>{#if page==='workspace'}<p>{detached?(uncertainWindowSlot!==null?'Session state and native window restoration must be verified separately.':'Your games keep running on the desktop.'):active?'Click the game to play.':binding?.configured?'Each launcher keeps its own data.':'Existing data will be preserved.'}</p>{/if}</div>
        <div class="lt-header-actions"><button class="lt-secondary" disabled={busy||manualRefreshing||manualRefreshRequested||nativeBackend!=='WindowsNative'} on:click={()=>manualRefreshRequested=true}>{manualRefreshing||manualRefreshRequested?'Refreshing…':'Refresh'}</button>{#if page==='workspace'&&active!==null}<button class="lt-secondary" disabled={busy||refreshing||resizing} title={'Open only Minecraft - '+active+' in its own window'} on:click={release}>Open Minecraft - {active} in a window</button>{/if}</div>
      </header>
      {#if issue||connectionIssue}<div class="lt-alert" role="alert">{#if issue}<p>{issue}</p>{/if}{#if connectionIssue}<p>{connectionIssue}</p>{/if}</div>{/if}
      {#if uncertainWindowSlot!==null}
        <section class="lt-exit-recovery" role="status" aria-label="Window recovery unconfirmed">
          <h2>Minecraft - {uncertainWindowSlot} window recovery is unverified</h2>
          <p>A STOPPED game process does not prove its original native window checkpoint was restored. Refresh updates session observations only; do not assume this window is available. Other verified games are preserved.</p>
        </section>
      {/if}
      {#if closeFailure}
        <section class="lt-exit-recovery" role="status" aria-label="Exit recovery">
          <h2>Exit was not completed</h2>
          <p>Some Minecraft sessions may still be running. Your remaining sessions were preserved. Choose a responsive Minecraft below to return to Play, or finish an open game dialog normally.</p>
          <div class="lt-actions"><button class="lt-secondary" on:click={()=>{closeFailure='';void refresh();}}>Dismiss and refresh</button></div>
          <details><summary>Technical details</summary><p>{closeFailure}</p></details>
        </section>
      {/if}

      {#if nativeBackend===null&&!initializing}
        <section class="lt-binding" aria-label="Backend capability unavailable">
          <div><strong>Backend capability could not be verified</strong><p>{bindingIssue||'Native support check failed. No Minecraft operation was attempted.'}</p></div>
          <div class="lt-actions"><button disabled={busy} on:click={()=>void initializeLaunchers()}>Retry backend check</button></div>
        </section>
      {:else if nativeBackend==='Unsupported'&&!initializing}
        <section class="lt-binding" aria-label="Unsupported platform">
          <div><strong>Minecraft backend unavailable</strong><p>{bindingIssue}</p></div>
        </section>
      {:else if !binding?.configured&&!initializing}<section class="lt-binding" aria-label="Launcher recovery"><div><strong>Existing launcher connection could not be verified</strong><p>{bindingIssue||binding?.issue||'Automatic preparation could not finish. Existing data is preserved.'}</p></div><div class="lt-actions"><button disabled={busy||manualRefreshing||manualRefreshRequested} on:click={()=>manualRefreshRequested=true}>Retry connection</button><button class="lt-secondary" on:click={()=>navigate('storage')}>Data Storage</button></div></section>{/if}
      <section class="lt-workspace" hidden={page!=='workspace'} aria-label="Minecraft play area">
        <div class="lt-stage-space" bind:this={stageSpace}>
        <div class="lt-viewport" class:lt-desktop={detached} bind:this={viewport}>
          {#if detached}<div class="lt-empty"><h2>{uncertainWindowSlot!==null?'Window restoration not verified':closeFailure?'Exit incomplete — sessions preserved':'Games are in separate windows'}</h2><p>{uncertainWindowSlot!==null?'Minecraft - '+uncertainWindowSlot+' has an unverified native recovery checkpoint. Refresh checks game process state, not whether window restoration succeeded.':closeFailure?'Some Minecraft sessions may still be running and native window recovery is not guaranteed. Review each session before returning to Play.':'Use the Windows taskbar, or choose a game below to open it in Play.'}</p><p>{runningCount} of 5 running</p></div>{:else if !active}<div class="lt-empty"><span>LT</span><h2>{initializing?'Preparing launchers…':nativeBackend==='Unsupported'?'Minecraft backend unavailable':!binding?.configured?'Connection needs attention':!available?'Session status unavailable':requested?'Starting Minecraft - '+requested:'Ready when you are'}</h2><p>{initializing?'Reconnecting saved data automatically. No setup is needed.':nativeBackend==='Unsupported'?'Native Minecraft sessions are not supported by this application build on this OS.':!binding?.configured?'Retry the connection. Your existing accounts and worlds are preserved.':!available?'Use Refresh to reconnect.':requested?'Waiting for the game window…':'Select Launch below to start Minecraft.'}</p></div>{/if}
        </div>
        </div>
        {#if nativeBackend === 'WindowsNative'}
        <div class="lt-games-heading"><span>Your games</span><span>{runningCount} of 5 running</span></div>
        <div class="lt-filmstrip" aria-label="Minecraft launchers">
          {#each profiles as profile (profile.slot)}
            {@const previewFailure = previewStatuses.find(status => status.slot === profile.slot && !status.registered && status.reason !== null)}
            <div class="lt-launcher-entry">
              <button class="lt-card" class:selected={active===profile.slot&&!detached} aria-pressed={active===profile.slot&&!detached} aria-label={(profile.state==='RUNNING'?'Open ':'Launch ')+'Minecraft - '+profile.slot} disabled={busy||!available||!binding?.configured||['STARTING','NOT_RESPONDING'].includes(profile.state)} on:click={()=>profile.state==='RUNNING'?select(profile.slot):launch(profile.slot)}>
                <strong>Minecraft - {profile.slot}{active===profile.slot&&!detached?' ✓':''}</strong>
                <span class="lt-card-picture" use:image={profile.slot} aria-hidden="true">{profile.state==='RUNNING'?profile.slot:profile.state==='STARTING'?'Starting…':profile.state==='NOT_RESPONDING'?'Unavailable':profile.state==='ATTENTION'?'Needs attention':'▷ Launch'}</span>
                <small>{active===profile.slot&&!detached?'Playing here':profile.state==='RUNNING'&&separateSlots.has(profile.slot)?'Separate window':profile.state==='RUNNING'?'Running in background':labels[profile.state]}</small>
                {#if profile.issue}<small class="lt-error">{profile.issue}</small>{/if}
              </button>
              {#if profile.state==='RUNNING' && !detached && profile.slot!==primary && !separateSlots.has(profile.slot) && previewFailure}
                <small class="lt-error lt-preview-slot-issue" role="status" aria-label={'Minecraft - '+profile.slot+' preview unavailable'}>Preview unavailable: {previewFailure.reason}</small>
              {/if}
              {#if ['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state)}
                <button class="lt-secondary lt-close-launcher" aria-label={'Close Minecraft - '+profile.slot+' normally'} disabled={busy||closeInProgress||nativeExitReview||!available||!binding?.configured} on:click={()=>requestCloseOne(profile.slot)}>Close game</button>
              {/if}
            </div>
          {/each}
        </div>
        {#if closeOneSlot!==null}
          <section class="lt-single-close-confirm" aria-label="Confirm one Minecraft close">
            <h3>Close Minecraft - {closeOneSlot}?</h3>
            <p>Only this Minecraft session will be asked to exit normally. Other games will keep running. Save any work in Minecraft before continuing.</p>
            <div class="lt-actions">
              <button disabled={busy||refreshing||resizing||closeInProgress||nativeExitReview} on:click={()=>void confirmCloseOne()}>Close Minecraft - {closeOneSlot}</button>
              <button class="lt-secondary" disabled={busy||closeInProgress||nativeExitReview} on:click={()=>closeOneSlot=null}>Cancel</button>
            </div>
          </section>
        {/if}
        {#if closingOneSlot!==null}<p role="status">Waiting for Minecraft - {closingOneSlot} to close normally…</p>{/if}
        <p class="lt-help">{!binding?.configured?'Saved launcher data is checked automatically.':detached?'Select a running game to bring it into Play.':'Click a game to switch. Other games keep running.'}</p>
        {#if previewIssue}<details class="lt-preview-details"><summary>Preview unavailable</summary><p>{previewIssue}</p></details>{/if}
        {/if}
      </section>
      {#if page==='settings'}
        <div class="lt-settings-content">
          <section class="lt-group"><h2>Graphics</h2><label class="lt-target">Applies to <select value={selectedSlot} disabled={busy||queuedPreset!==null} on:change={event=>chooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>Minecraft - {p.slot}</option>{/each}</select></label>
            <div class="lt-setting"><div><h3><label for="graphics-preset">Graphics preset</label></h3><p>High enables visual effects. Low reduces them. Both use 12 chunks; simulation distance is unchanged.</p></div><select id="graphics-preset" bind:value={graphicsPreset} disabled={busy||queuedPreset!==null||!settings?.graphicsPresets}><option>High</option><option>Low</option></select></div>
            <div class="lt-actions"><button disabled={busy||queuedPreset!==null||!available||!binding?.configured||!settings?.canApplyPreset||!graphicsDirty||runningCount>0} on:click={requestPreset}>{applyingPreset||queuedPreset!==null?'Applying…':'Apply preset'}</button></div>
            <p class="lt-help">Close all Minecraft sessions before applying. The saved preset is reapplied when this launcher starts.</p>{#if settings&&!settings.graphicsPresets}<p class="lt-error">The running controller needs an update before High/Low can be selected. Existing games are preserved.</p>{/if}{#if settingsNotice}<p role="status">{settingsNotice}</p>{/if}
          </section>
          <section class="lt-group"><h2>Frame Limits <small>All launchers</small></h2>{#if settings}<form on:submit|preventDefault={requestPerformanceSave}>
            <div class="lt-setting"><div><h3><label for="active-fps">While playing</label></h3><p>The game receiving keyboard and mouse input.</p></div><div class="lt-fps"><input id="active-fps" type="number" min="5" max="1000" required bind:value={activeFps} disabled={busy||queuedPerformance!==null}/> FPS</div></div>
            <div class="lt-setting"><div><h3><label for="background-fps">In the background</label></h3><p>Other games keep running at this limit.</p></div><div class="lt-fps"><input id="background-fps" type="number" min="5" max={activeFps??1000} required bind:value={backgroundFps} disabled={busy||queuedPerformance!==null}/> FPS</div></div>
            <p class="lt-help">Saved limits apply at the next launch. Running games keep their current limits.</p>{#if !performanceValid}<p class="lt-error" role="alert">Use whole numbers from 5 to 1000. Background FPS cannot exceed active FPS.</p>{/if}
            <div class="lt-actions"><button type="submit" disabled={busy||queuedPerformance!==null||!available||!performanceDirty||!performanceValid}>{savingPerformance||queuedPerformance!==null?'Saving…':'Save changes'}</button>{#if performanceDirty}<button type="button" class="lt-secondary" disabled={busy||queuedPerformance!==null} on:click={()=>{activeFps=settings!.activeFps;backgroundFps=settings!.backgroundFps;}}>Discard</button>{/if}<button type="button" class="lt-secondary" disabled={busy||queuedPerformance!==null} on:click={()=>{activeFps=120;backgroundFps=5;}}>Use 120 / 5</button></div>{#if performanceNotice}<p role="status">{performanceNotice}</p>{/if}
          </form>{:else}<p>Settings unavailable. Refresh to try again.</p>{/if}</section>
          <section class="lt-group"><h2>App behavior</h2><div class="lt-setting"><div><h3>When Lazy Tester closes</h3><p>All Minecraft sessions started by Lazy Tester close normally first. Lazy Tester waits for them to finish before exiting.</p></div></div><p class="lt-help">If Minecraft is waiting for a dialog or has not finished closing, Lazy Tester stays open. Other Minecraft applications are left alone.</p><div class="lt-setting"><div><h3>Background game audio</h3><p>Mute games that are not receiving keyboard and mouse input. Existing user mute and volume settings are kept.</p>{#if settings?.audioIssue}<p class="lt-error" role="status">{settings.audioIssue}</p>{/if}{#if !settings?.backgroundAudio}<p>Update the running controller to enable audio control.</p>{/if}</div><label class="lt-audio-toggle"><input type="checkbox" checked={settings?.backgroundAudio==='Mute'} disabled={busy||queuedAudio!==null||!settings?.backgroundAudio} aria-label="Mute background games" on:change={event=>saveAudio(event.currentTarget.checked?'Mute':'Keep')}/>Mute background games</label></div></section>
          <section class="lt-group"><h2>Installation & data</h2><h3>Minecraft Education</h3><p>The controller verifies the registered game executable before launch. Moving data does not move Minecraft.</p><h3>Launcher data location</h3><p class="lt-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lt-error">{locationIssue}</p>{/if}<div class="lt-actions"><button class="lt-secondary" disabled={busy||refreshing||resizing||!dataLocation?.path} on:click={()=>loadLocation('Open')}>Open data folder</button><button class="lt-secondary" on:click={()=>navigate('storage')}>Data Storage →</button></div></section>
        </div>
      {:else if page==='storage'}
        <div class="lt-settings-content">
          <p>Worlds, cache and settings for each launcher.</p><div class="lt-launcher-heading"><select value={selectedSlot} disabled={busy} aria-label="Choose Minecraft launcher" on:change={event=>chooseSlot(Number(event.currentTarget.value) as ProfileSlot)}>{#each profiles as p}<option value={p.slot}>Minecraft - {p.slot}</option>{/each}</select><span>{selectedProfile?labels[selectedProfile.state]:'Unavailable'}</span></div>
          {#if storageNotice}<p role="status">{storageNotice}</p>{/if}
          <section class="lt-group"><div class="lt-setting"><h3>Profile data</h3><strong>{selectedStorage?(selectedStorage.partial?'At least ':'')+(selectedStorage.bytes/1048576).toFixed(1)+' MB':'Not measured'}</strong></div><div class="lt-setting"><h3>Web cache</h3><span>{selectedStorage&&!selectedStorage.cache.partial?(selectedStorage.cache.bytes/1048576).toFixed(1)+' MB':'Unavailable'}</span></div><div class="lt-setting"><h3>Available disk space</h3><span>{selectedStorage?.disk?(selectedStorage.disk.availableBytes/1073741824).toFixed(1)+' GB':'Unavailable'}</span></div><p class="lt-help">{storage?'Last measured at '+storageCheckedAt+'. Web cache is included in profile data.':'Use Refresh to measure storage. World sizes are not separately classified.'}</p></section>
          <section class="lt-setting"><div><h3>Clear cache</h3><p>Remove recognized web cache. Accounts, worlds and game settings are kept.</p>{#if selectedStorage?.cache.issue}<p>{selectedStorage.cache.issue}</p>{/if}</div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.cache.canClear} on:click={()=>{fullReview=null;recovery=null;clearSlot=selectedSlot;}}>Clear cache…</button></section>
          {#if clearSlot===selectedSlot}<section class="lt-cache-confirm" aria-label="Confirm cache cleanup"><h3>Clear web cache for Minecraft - {selectedSlot}?</h3><p>Worlds, accounts and settings are kept. Other launchers are unchanged.</p><div class="lt-actions"><button disabled={busy||refreshing||resizing||!selectedStorage?.cache.canClear} on:click={()=>clearCache(selectedSlot)}>Clear web cache</button><button class="lt-secondary" on:click={()=>clearSlot=null}>Cancel</button></div></section>{/if}
          <section class="lt-setting"><div><h3>Data folder</h3><p class="lt-data-path">{selectedPath??'Location unavailable.'}</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!binding?.configured} on:click={()=>openFolder(selectedSlot)}>Open folder</button></section>
          <details class="lt-advanced" bind:open={advanced}>
            <summary><span><strong>Advanced options</strong><small>Reset and recovery for this launcher</small></span><span aria-hidden="true">{advanced?'−':'+'}</span></summary>
            <div class="lt-advanced-body">
              <div class="lt-setting"><div><h3>Reset game settings</h3><p>Back up this launcher's settings, then reset them for the next start. Accounts, worlds and cache are kept.</p><p class="lt-help">The saved High/Low graphics preset is reapplied at launch.</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.gameSettings?.canReset} on:click={()=>reviewSettings('Reset')}>Reset settings…</button></div>
              <div class="lt-setting"><div><h3>Restore settings</h3><p>{selectedStorage?.gameSettings?.backupAvailable?'Restore the latest verified settings backup for this launcher. Current settings are backed up first.':'No verified settings backup yet. Reset creates one.'}</p></div><button class="lt-secondary" disabled={busy||refreshing||resizing||!available||!selectedStorage?.gameSettings?.canRestore} on:click={()=>reviewSettings('Restore')}>Restore settings…</button></div>
              {#if selectedStorage?.gameSettings?.issue}<p class="lt-help">{selectedStorage.gameSettings.issue}</p>{:else if !selectedStorage?.gameSettings}<p class="lt-help">Refresh storage to check settings recovery availability.</p>{/if}
              {#if recovery?.slot===selectedSlot}
                <section class="lt-cache-confirm" aria-label="Confirm game settings change"><h3>{recovery.action} settings for Minecraft - {recovery.slot}?</h3><p>Only this launcher's game settings change. Accounts, worlds, cache and other launchers are kept. A backup is saved first.</p><div class="lt-actions"><button disabled={busy||refreshing||resizing||!(recovery.action==='Reset'?selectedStorage?.gameSettings?.canReset:selectedStorage?.gameSettings?.canRestore)} on:click={recoverSettings}>{recovery.action} settings</button><button class="lt-secondary" disabled={busy} on:click={()=>recovery=null}>Cancel</button></div></section>
              {/if}
            </div>
          </details>
          {#if clearingData}<p class="lt-alert" role="status">Clearing launcher data… Keep Lazy Tester open while the verified cleanup finishes.</p>{/if}
          <section class="lt-danger-zone"><div><h3>Clear all data</h3><p>Permanently remove this launcher's stored account data, worlds, cache, game settings and backups. Keep the Minecraft installation and app preferences.</p><p>Windows-managed sign-in data is not cleared, so a previous account may still be offered. Export any worlds you want to keep before continuing; no recovery backup is created. Exported or external backups are kept.</p>{#if selectedStorage?.fullClear?.pending}<p>A previous clear needs to finish. This launcher cannot start until its old data is removed.</p>{/if}{#if selectedStorage?.fullClear?.issue}<p>{selectedStorage.fullClear.issue}</p>{:else if !selectedStorage?.fullClear}<p>Refresh storage to check availability.</p>{/if}</div><button class="lt-secondary lt-danger" disabled={busy||refreshing||resizing||!available||!selectedStorage?.fullClear?.canClear} on:click={reviewFullClear}>{selectedStorage?.fullClear?.pending?'Finish clearing data…':'Clear all data…'}</button></section>
          {#if fullReview?.slot===selectedSlot}
            <form class="lt-cache-confirm lt-delete-confirm" aria-label="Confirm full launcher data deletion" on:submit|preventDefault={clearAllData}>
              <h3>{fullReview.pending?'Finish clearing':'Clear all data for'} Minecraft - {fullReview.slot}?</h3>
              <p>This permanently removes this launcher's stored account data, worlds, cache, settings and backups. Other launchers and Windows-managed sign-in data are kept.</p>
              <label for="full-clear-name">Type Minecraft - {fullReview.slot} to confirm</label>
              <input id="full-clear-name" type="text" maxlength="32" autocomplete="off" bind:value={fullName} disabled={busy}/>
              <label class="lt-delete-ack"><input type="checkbox" bind:checked={fullAck} disabled={busy}/> I understand this cannot be undone and have exported any worlds I need.</label>
              <div class="lt-actions"><button type="submit" class="lt-danger" disabled={busy||refreshing||resizing||!fullAck||fullName!==`Minecraft - ${fullReview.slot}`||!selectedStorage?.fullClear?.canClear||selectedStorage?.fullClear?.confirmationToken!==fullReview.token}>Permanently clear data</button><button type="button" class="lt-secondary" disabled={busy} on:click={()=>fullReview=null}>Cancel</button></div>
            </form>
          {/if}
          <details class="lt-location-options"><summary>Connect or locate launcher data</summary><p class="lt-data-path">{dataLocation?.path??'No data folder connected.'}</p>{#if locationIssue}<p class="lt-error">{locationIssue}</p>{/if}<div class="lt-actions"><button class="lt-secondary" disabled={busy||refreshing||resizing||runningCount>0||dataLocation?.canLocate===false} on:click={()=>loadLocation('Locate')}>Locate existing data</button>{#if dataLocation&&!dataLocation.path}<button disabled={busy||refreshing||resizing} on:click={()=>loadLocation('Choose')}>Choose new data folder</button>{/if}</div><p>Connecting data does not migrate or delete existing accounts and worlds.</p></details>
        </div>
      {/if}
    </main>
  </div>
</div>
