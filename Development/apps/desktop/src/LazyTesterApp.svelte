<script lang="ts">
  import {onMount, tick} from 'svelte';
  import {listen} from '@tauri-apps/api/event';
  import {lazyTester, firstEmbeddedProfile, validFpsLimits, type AgentActivityReport, type ProfileSession, type ProfileSlot, type ProfileStorage, type PreviewStatus, parseDwmPreviewInvalidation, dwmPreviewInvalidationApplies, type DwmPreviewInvalidation, type ProfileBinding, type DataLocation, type SettingsReport, type NativeBackendSupport, profileSlots, recordingActive, type RecordingReceipt} from './app/bridge/lazyTester.js';
  import {workspaceSurface} from './app/bridge/workspaceSurface.js';
  import './lazytester.css';
  import WindowTitlebar from './WindowTitlebar.svelte';
  import LauncherRail from './LauncherRail.svelte';
  import AgentActivityNotice from './AgentActivityNotice.svelte';
  import {presentAgentActivity} from './app/presentation/agentActivity.js';
  import GeneralSettings from './GeneralSettings.svelte';
  import DataStorage from './DataStorage.svelte';
  import {canRequestProfileLaunch, projectSessionHealth, reconcileLaunchReceipts, type LaunchReceiptState, type SessionHealthAction} from './app/presentation/sessionHealth.js';
  import {guidedRecoveryAllowed} from './app/presentation/guidedRecovery.js';
    import {userMessage, userFeedback, feedbackHeading, sessionConnectionLabel, updateRecoverySlots, recoveryLabels, type UserMessageKey} from './app/presentation/sessionFeedback.js';
  import {observeDisplayScale, sameWorkspaceGeometry, type WorkspaceGeometry} from './app/workspaceGeometry.js';
  import type {StorageReview, StorageReviewKind, StorageSettingsAction} from './app/storageReview.js';
  import {closeReviewActive, observationStillCurrent, previewLayoutStillCurrent, releasedSelectionStillCurrent, periodicInspectionInterval, passiveIdleAllowed, sessionInspectionDeadline, nextBackgroundDeadline, nextPendingUiAction, sameProfileObservation, sameProfileLayout, sameSettingsObservation, sameAgentObservation, agentNeedsFastInspection, type CloseReview} from './app/inspectionScheduling.js';

  let profiles: ProfileSession[] = [], storage: ProfileStorage[] | null = null;
  let binding: ProfileBinding | null = null;
  let nativeBackend: NativeBackendSupport | null = null;
  let bindingIssue = '', locationIssue = '';
  let dataLocation:DataLocation|null=null;
  let settings:SettingsReport|null=null, settingsNotice='';
  let queuedAudio:'Keep'|'Mute'|null=null;
  let selectedSlot:ProfileSlot=1;
  let advanced=false;
  $: selectedStorage=storage?.find(item=>item.slot===selectedSlot);
  $: selectedProfile=profiles.find(item=>item.slot===selectedSlot);
  $: selectedPath=dataLocation?.profiles.find(item=>item.slot===selectedSlot)?.path;
  async function chooseSlot(slot:ProfileSlot){
    // A profile switch cannot dismiss a review or race a native operation.
    if (slot===selectedSlot || activeOperation !== null || selectiveOpen ||
        refreshing || resizing || disposed || !profileSlots.includes(slot) ||
        closeReviewActive(reviewState()) || storageReview!==null) return;
    selectedSlot=slot;
    // Never display another slot's receipt while the selected read is pending.
    settingsNotice='';storageNotice='';storage=null;
    if(page!=='storage'||nativeBackend!=='WindowsNative'||!binding?.configured)return;
    const issued=observationEpoch;
    activeOperation='refreshStorage';issueKey=null;
    try {
      await inspectSelectedStorage(issued);
    } catch(error) {
      if(isCurrentObservation(issued)){issueKey='storage';storage=null;}
    } finally {activeOperation=null;}
  }
  let queuedPerformance:{active:number;background:number}|null=null;
  let activeFps:number|undefined=144,backgroundFps:number|undefined=10,performanceNotice='';
  $: performanceDirty=settings!==null&&(activeFps!==settings.activeFps||backgroundFps!==settings.backgroundFps);
  $: performanceValid=validFpsLimits(activeFps,backgroundFps);
  let storageReview: StorageReview | null = null, storageNotice = '';
  let fullName='',fullAck=false;
  type Page = 'workspace' | 'storage' | 'settings';
  let page: Page = 'workspace';
  let queuedPage: Page | null = null;
  // Ephemeral route feedback only; not another session or settings owner.
  let loadingPage: Page | null = null;
  let pageHeading: HTMLHeadingElement | undefined;
  let headerRefreshButton: HTMLButtonElement | undefined;
  // Keyboard-only Refresh focus continuity, not a new status or native action.
  let headerRefreshFocusPage: Page | null = null;
  let initializing = true, storageReportReceivedAt = '';
  // A clock time alone is ambiguous after an overnight/open-window session.
  // The timestamp describes the last UI-received storage report, not a live
  // native filesystem measurement or a verified game/world state.
  function storageReportReceivedTime(): string {
    return new Date().toLocaleString('en-US',{dateStyle:'medium',timeStyle:'short'});
  }
  let active: ProfileSlot | null = null, requested: ProfileSlot | null = null, queued: ProfileSlot | null = null;
  let viewport: HTMLDivElement, stageSpace: HTMLDivElement, observer: ResizeObserver | undefined;
  let workspaceLayout: 'Bottom' | 'Right' = 'Bottom';
  let layoutPersistenceIssue = '';
  const workspaceLayoutKey = 'lazytester.workspace-layout';
  function chooseWorkspaceLayout(value: 'Bottom' | 'Right') {
    if (page !== 'settings' || navigationReviewOpen) return;
    workspaceLayout = value;
    dirty = true;
    try { window.localStorage.setItem(workspaceLayoutKey, value); layoutPersistenceIssue = ''; }
    catch { layoutPersistenceIssue = 'Layout applied for this session only. Preference could not be saved.'; }
  }
  // One synchronous admission owner for mutually exclusive UI actions.
  // Reactive flags below are presentation only; handlers read activeOperation.
  type UiOperation = 'recording' | 'screenshot' | 'select' | 'launch' |
    'navigate' | 'refreshStorage' | 'saveStoragePreferences' | 'saveAudio' |
    'savePerformance' | 'closeOne' | 'binding' | 'dataLocation' | 'openFolder' |
    'confirmSelectedData' | 'clearDownloadCache' | 'clearCache' |
    'recoverSettings' | 'clearData';
  let activeOperation: UiOperation | null = null;
  $: busy = activeOperation !== null;
  $: savingPerformance = activeOperation === 'savePerformance';
  $: screenshotBusy = activeOperation === 'screenshot';
  $: clearingData = activeOperation === 'clearData';
  let refreshing = false, resizing = false, available = false;
  let screenshotAvailable=false;
  let recordingAvailable=false, recordingStatusKnown=false, recordingStatusPending=false, recordingWatch=false;
  let recordingSlot:ProfileSlot=1;
  let recordings:RecordingReceipt[]=[];
  let recordingNotice:{slot:ProfileSlot;message:string}|null=null;
  $: activeRecordingSlots=new Set(recordings.filter(recordingActive).map(r=>r.slot));
  $: selectedRecording=recordings.find(r=>r.slot===recordingSlot);
  $: recordingTarget=profiles.find(p=>p.slot===recordingSlot&&p.state==='RUNNING')?.process;
  // A status read failure does not remove the explicit, idempotent
  // per-slot Stop command. Never authorize a new start on unknown state.
  $: canStopRecording=recordingWatch&&(!recordingStatusKnown||
    recordingActive(selectedRecording)&&selectedRecording?.state!=='Stopping');
  $: canStartRecording=recordingStatusKnown&&!recordingActive(selectedRecording)&&page==='workspace'&&
    available&&!!binding?.configured&&recordingTarget?.responding===true;
  async function refreshRecordingStatus(){
    if(!recordingAvailable||recordingStatusPending||disposed)return;
    recordingStatusPending=true;
    lastRecordingInspectionStartedAt=performance.now();
    try{
      const result=await lazyTester.recordingStatus();
      if(disposed)return;
      recordings=result;recordingStatusKnown=true;recordingWatch=result.some(recordingActive);
    }catch{
      if(!disposed){recordingStatusKnown=false;recordingWatch=true;}
    }finally{recordingStatusPending=false;}
  }
  async function recordingAction(){
    if(!recordingAvailable||activeOperation !== null||refreshing||resizing||recordingStatusPending||
      closeReviewPending||navigationReviewOpen||(!recordingStatusKnown&&!recordingWatch))return;
    const slot=recordingSlot;
    const receipt=recordings.find(r=>r.slot===slot);
    const target=profiles.find(p=>p.slot===slot&&p.state==='RUNNING')?.process;
    if(recordingStatusKnown&&receipt?.state==='Stopping')return;
    const stop=!recordingStatusKnown||recordingActive(receipt);
    if(!stop&&(!target?.responding||page!=='workspace'||!available||!binding?.configured))return;
    activeOperation = 'recording';recordingNotice=null;recordingWatch=true;
    try{
      if(stop)await lazyTester.stopRecording(slot);
      else if(target)await lazyTester.startRecording(slot,target);
    }catch{
      if(!disposed)recordingNotice={slot,message:stop?'Stop request failed':'Recording start failed'};
    }finally{
      await refreshRecordingStatus();
      activeOperation = null;
    }
  }
  function recordingLabel(slot:ProfileSlot,receipts:RecordingReceipt[],known:boolean,notice:typeof recordingNotice){
    const receipt=receipts.find(r=>r.slot===slot);
    const state=known?(receipt?.state??'Idle'):'Status unavailable';
    const recovery=known&&receipt?.recoveryFile?' · Finalized file retained in save folder: '+receipt.recoveryFile:'';
    const detail=notice?.slot===slot?' · '+notice.message:'';
    return 'MC0'+slot+' · '+state+recovery+detail;
  }
  let screenshotNotice:{slot:ProfileSlot;message:string}|null=null;
  async function saveScreenshot(){
    const slot=active;
    const target=profiles.find(p=>p.slot===slot&&p.state==='RUNNING')?.process;
    if(!screenshotAvailable||slot===null||!target?.responding||activeOperation !== null||refreshing||resizing||
      closeReviewPending||navigationReviewOpen||detached||page!=='workspace')return;
    activeOperation = 'screenshot';screenshotNotice=null;
    try{
      const receipt=await lazyTester.screenshot(slot,target);
      if(receipt&&!disposed)screenshotNotice={slot,message:`Screenshot saved · ${receipt.width} × ${receipt.height}`};
    }catch{
      if(!disposed)screenshotNotice={slot,message:'Screenshot unavailable. No successful save was confirmed.'};
    }finally{activeOperation = null;}
  }
  let manualRefreshRequested=false, manualRefreshing=false;
  // Each observation uses its own last-start deadline; both share one timer.
  let lastInspectionStartedAt = Number.NEGATIVE_INFINITY;
  let lastSettingsObservedAt = Number.NEGATIVE_INFINITY;
  // One coalesced hint for the existing read lane, not a native receipt.
  let sessionObservationHint = false;
  let lastRecordingInspectionStartedAt = Number.NEGATIVE_INFINITY;
  let managerVisible = false;
  let observationTimer: number | undefined;
  let queuedLaunch:ProfileSlot|null=null;
  let mounted = false, disposed = false, dirty = false, detached = false;
  // Alert context is authored, not a retained raw backend exception.
  let issueKey: UserMessageKey | null = null, connectionIssueKey: UserMessageKey | null = null;
  let previewIssue = '', selectedIdentity = '';
  // Optional native controller mirror; no new timer or MCP process.
  let agentActivity:AgentActivityReport|null=null;
  $: activityView=presentAgentActivity(agentActivity);
  // Display state only; the retained Rust controller still owns game lifetime.
  let closeInProgress = false, closeFailure = '';
  // Presentation-only close reviews; never another process or session owner.
  let nativeExitReview = false, observationEpoch = 0;
  let closeOneSlot: ProfileSlot | null = null, closingOneSlot: ProfileSlot | null = null;
  // UI-only keyboard focus references, never another game or window owner.
  let closeOneTrigger: HTMLButtonElement | null = null;
  let closeCancelButton: HTMLButtonElement | undefined;
  let closeOneReviewElement: HTMLElement | undefined;
  // UI warning only; native presentation authority remains in workspace_surface.
  let uncertainWindowSlots = new Set<ProfileSlot>();
  // UI receipt uncertainty only; native session admission remains authoritative.
  let launchFailures: ReadonlyMap<ProfileSlot, LaunchReceiptState> = new Map();
  function recordLaunchFailure(slot: ProfileSlot, state: LaunchReceiptState) {
    if (launchFailures.get(slot) !== state)
      launchFailures = new Map(launchFailures).set(slot,state);
  }
  function clearLaunchFailure(slot: ProfileSlot) {
    if (!launchFailures.has(slot)) return;
    const next = new Map(launchFailures);
    next.delete(slot);
    launchFailures = next;
  }
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
  // Page changes must not silently dispose a visible CloseOne or Storage
  // review. The user explicitly cancels it before navigating elsewhere.
  let selectiveOpen = false;
  $: navigationReviewOpen = selectiveOpen || closeReviewPending || clearingData || storageReview !== null;
  const images = new Map<ProfileSlot, HTMLElement>();
  const labels = {STOPPED: 'Stopped', STARTING: 'Starting…', RUNNING: 'Running', NOT_RESPONDING: 'Not Responding', ATTENTION: 'Needs Attention'};
  const identity = (profile?: ProfileSession) => profile?.process ? [profile.process.pid, profile.process.createdFileTime, profile.process.sessionId, profile.process.windowHandle].join('/') : '';
  $: primary = active ?? requested ?? profiles.find(profile => profile.state === 'RUNNING')?.slot ?? 1;
  $: others = profiles.filter(profile => profile.slot !== primary);
  $: runningCount = profiles.filter(profile => ['STARTING', 'RUNNING', 'NOT_RESPONDING'].includes(profile.state)).length;
  // Displayed observations are narrower than occupied sessions used for native safety gates.
  $: observedRunningCount = profiles.filter(profile => profile.state === 'RUNNING').length;
  $: startingCount = profiles.filter(profile => profile.state === 'STARTING').length;
  // Display-only health projection from the existing verified observations.
  // No new monitoring timer, native request or admission state.
  $: healthIssues = projectSessionHealth({
    profiles, previewStatuses, launchFailures, unresolvedRecovery: uncertainWindowSlots,
    primarySlot: primary,
    observed: nativeBackend === 'WindowsNative' && !initializing && available && !!binding?.configured,
    previewsVisible: page === 'workspace' && !detached,
  });
  $: attentionCount = healthIssues.length + uncertainWindowSlots.size;
  // Only fully observed, healthy Play may slow reconciliation. A selected
  // native binding and registered background previews provide event hints,
  // not proof that the game is still alive or responsive.
  $: workspaceObservationHealthy = profiles.length === 5 &&
    requested === null && queued === null && queuedLaunch === null &&
    !detached && attentionCount === 0 && issueKey === null && connectionIssueKey === null &&
    previewIssue === '' && !closeFailure;
  $: workspaceSettledIdle = workspaceObservationHealthy && active === null && agentActivity === null &&
    profiles.every(profile => profile.state === 'STOPPED' && profile.process === null && profile.issue === null);
  $: workspaceSettledLive = workspaceObservationHealthy && active !== null && !agentNeedsFastInspection(agentActivity) &&
    selectedIdentity !== '' && selectedIdentity === identity(profiles.find(profile => profile.slot === active)) &&
    profiles.some(profile => profile.slot === active && profile.state === 'RUNNING') &&
    profiles.every(profile => profile.issue === null &&
      (profile.state === 'STOPPED' && profile.process === null ||
       profile.state === 'RUNNING' && profile.process?.responding === true &&
       (profile.slot === active || previewStatuses.some(status => status.slot === profile.slot && status.registered))));
  // All reactive dependencies are explicit. A blocked lane has no deadline;
  // resuming uses its last start, not a fresh full delay after each UI event.
  $: sessionObservationEnabled = mounted && !disposed && managerVisible &&
    nativeBackend === 'WindowsNative' && !closeReviewPending && !!binding?.configured &&
    !selectiveOpen && storageReview===null &&
    activeOperation === null && !refreshing && !resizing && !initializing;
  $: recordingObservationEnabled = mounted && !disposed && managerVisible &&
    recordingAvailable && recordingWatch && !recordingStatusPending &&
    activeOperation === null && !closeInProgress && !nativeExitReview;
  $: sessionDeadline = sessionObservationEnabled
    ? sessionInspectionDeadline(lastInspectionStartedAt,
        periodicInspectionInterval(page, runningCount, available, workspaceSettledIdle, workspaceSettledLive),
        sessionObservationHint)
    : null;
  $: recordingDeadline = recordingObservationEnabled ? lastRecordingInspectionStartedAt + 2000 : null;
  $: scheduleBackgroundObservation(sessionDeadline, recordingDeadline);
  // Normal Play is already inspected automatically. Keep explicit Refresh
  // for exceptional states, settings/storage and an in-flight manual action.
  $: showHeaderRefresh = nativeBackend==='WindowsNative' &&
    (page!=='workspace' || closeOneSlot!==null || closingOneSlot!==null || manualRefreshing || manualRefreshRequested ||
      (!initializing && (!available || !binding?.configured || issueKey!==null ||
        connectionIssueKey!==null || previewIssue!=='' || attentionCount>0 || !!closeFailure)));
  // Local display failures reuse the bounded workspace feedback region.
  // Connection and restoration uncertainty retain their prominent global warnings.
  $: workspaceActionFeedback = page==='workspace' && nativeBackend==='WindowsNative' &&
    active!==null && !detached && available && !!binding?.configured &&
    connectionIssueKey===null && uncertainWindowSlots.size===0 && !closeFailure &&
    (issueKey==='selection' || issueKey==='preview');
  // Keep complete last-native-observation text in DOM while only the visual
  // line ellipsizes on narrow windows. This is not gameplay readiness.
  $: gamesStatusSummary = available && binding?.configured
    ? observedRunningCount + ' of 5 running' +
      (startingCount > 0 ? ' · ' + startingCount + ' starting' : '') +
      (attentionCount > 0 ? ' · ' + attentionCount + ' need attention' : '')
    : 'Session status unavailable';
  // UI-only admission: pending selections and launches must finish before
  // another manual recovery action can replace their queued destination.
  $: guidedRecoveryBlocked = nativeBackend !== 'WindowsNative' || page !== 'workspace' ||
    disposed || !mounted || initializing || !available || !binding?.configured ||
    busy || refreshing || resizing || manualRefreshing || manualRefreshRequested ||
    closeReviewPending || queuedPage !== null || queuedLaunch !== null ||
    queued !== null || requested !== null;
  // One passive scheduling decision per Svelte update. All payloads,
  // mutations and native operations remain owned by this component.
  // Re-evaluate the current queue instead of caching a decision in state.
  $: {
    const nextAction = nextPendingUiAction({
      mounted, nativeBackend, closeInProgress, closeReviewPending,
      selectedReviewGeometryOnly: closeOneSlot!==null && closingOneSlot===null &&
        !nativeExitReview && page==='workspace' && !detached && available,
      initializing, busy: activeOperation !== null, refreshing, resizing,
      queuedAudio: queuedAudio !== null,
      queuedPerformance: queuedPerformance !== null,
      queuedPage: queuedPage !== null,
      manualRefreshRequested,
      queuedLaunch: queuedLaunch !== null,
      queuedSelection: queued !== null,
      dirty,
    });
    switch (nextAction) {
      case 'audio': {
        const mode = queuedAudio;
        if (mode !== null) {queuedAudio = null; void saveAudio(mode);}
        break;
      }
      case 'performance': {
        const limits = queuedPerformance;
        if (limits !== null) {queuedPerformance = null; void savePerformance(limits);}
        break;
      }
      case 'navigation':
        void consumeNavigation();
        break;
      case 'manualRefresh':
        manualRefreshRequested = false;
        void refreshPage();
        break;
      case 'launch': {
        const slot = queuedLaunch;
        if (slot !== null) {queuedLaunch = null; void launch(slot);}
        break;
      }
      case 'selection':
        void consumeSelection();
        break;
      case 'layout':
        void syncLayout();
        break;
    }
  }

  function image(node: HTMLElement, slot: ProfileSlot) {
    images.set(slot, node); observer?.observe(node); dirty = true;
    return {destroy() { observer?.unobserve(node); if (images.get(slot) === node) images.delete(slot); dirty = true; }};
  }
  // Rebuild the same read-only projection at click time: reactive UI rendering
  // can lag a just-observed process change by one microtask. No new IPC or
  // process authority is introduced here.
  function requestGuidedRecovery(slot: ProfileSlot, action: SessionHealthAction) {
    const recommended = projectSessionHealth({
      profiles, previewStatuses, launchFailures, unresolvedRecovery: uncertainWindowSlots,
      primarySlot: primary,
      observed: nativeBackend === 'WindowsNative' && !initializing &&
        available && !!binding?.configured,
      previewsVisible: page === 'workspace' && !detached,
    }).find(item => item.slot === slot);
    if (!guidedRecoveryAllowed(recommended, profiles.find(item => item.slot === slot),
      slot, action, {
        blocked: guidedRecoveryBlocked || closeReviewActive(reviewState()),
        unresolvedRecovery: uncertainWindowSlots, 
        primarySlot: primary,
      })) return;
    if (action === 'open') {
      select(slot); // Original manual Play path; Rust rechecks the HWND.
    } else {
      manualRefreshRequested = true; // Original manual Refresh path.
    }
  }
  function select(slot: ProfileSlot) {
    // Only explicit Cancel ends a game/storage confirmation. Card selection
    // must not silently discard a pending review or replace navigation.
    if (closeReviewActive(reviewState()) || navigationReviewOpen || activeOperation !== null ||
        queuedPage !== null) return;
    requested = null; queued = slot;
  }
  async function consumeSelection() {
    const slot = queued; queued = null;
    if (slot === null || !available || profiles.find(profile => profile.slot === slot)?.state !== 'RUNNING') return;
    activeOperation = 'select'; issueKey=null;
    try {
      page = 'workspace'; await tick();
      await presentCurrentViewport(slot);
      active = slot; selectedIdentity = identity(profiles.find(profile => profile.slot === slot));
      detached = false; requested = null; dirty = true;
    } catch (error) { issueKey='selection'; dirty = true; }
    finally { activeOperation = null; }
  }
  function isCurrentObservation(issued: number): boolean {
    return !disposed && observationStillCurrent(issued, observationEpoch, reviewState());
  }
  async function refresh(passiveIdle = false) {
    if (nativeBackend !== 'WindowsNative' || closeReviewActive(reviewState()) || activeOperation !== null || refreshing || resizing || disposed) return;
    const issued = observationEpoch;
    refreshing = true;
    lastInspectionStartedAt = performance.now();
    sessionObservationHint = false;
    try {
      const snapshot = page === 'settings' || page === 'storage'
        ? (passiveIdle ? await lazyTester.settingsIdle() : await lazyTester.settings())
        : passiveIdle ? await lazyTester.inspectIdle() : await lazyTester.inspect();
      // Null is a cheap negative hint, not a new status or readiness receipt.
      // A close review starting mid-request invalidates this old snapshot.
      if (!isCurrentObservation(issued) || snapshot===null) return;
      const profilesChanged = !sameProfileObservation(profiles, snapshot.profiles);
      // An issue-only change updates feedback without reapplying geometry.
      const layoutChanged = !available || (profilesChanged &&
        !sameProfileLayout(profiles, snapshot.profiles));
      if (profilesChanged) profiles = snapshot.profiles;
      available = true; connectionIssueKey=null;
      const nextFailures = reconcileLaunchReceipts(launchFailures, snapshot.profiles);
      if (nextFailures !== launchFailures) launchFailures = nextFailures;
      const incomingActivity = snapshot.agentActivity ?? null;
      if (!sameAgentObservation(agentActivity, incomingActivity)) agentActivity = incomingActivity;
      // STOPPED only confirms a session observation, not restoration of
      // the original HWND/helper checkpoint. Only a positive selected-close
      // presentation receipt below may clear the uncertainty warning.
      if(snapshot.settings){
        observeSettings(snapshot.settings);
      }
      if (active !== null && selectedIdentity !== identity(profiles.find(profile => profile.slot === active))) {
        const released = {slot: active, identity: selectedIdentity};
        await workspaceSurface.release();
        // Release may finish AFTER a CloseOne review starts and increments
        // observationEpoch. Its completed native side effect must still
        // retire the previous UI binding. Never clear a newer selection.
        if (releasedSelectionStillCurrent(released,{slot: active, identity: selectedIdentity})) {
          active = null; selectedIdentity = '';
          previewStatuses = []; dirty = true;
        }
        if (!observationStillCurrent(issued, observationEpoch, reviewState())) return;
      }
      if (requested !== null) {
        const profile = profiles.find(profile => profile.slot === requested);
        if (profile?.state === 'RUNNING') {
          if (page === 'workspace' && queuedPage === null) queued = requested;
          requested = null;
        }
        else if (profile?.state === 'ATTENTION' || profile?.state === 'STOPPED'
          || (profile?.state === 'NOT_RESPONDING' && profile.issue)) {
          recordLaunchFailure(requested,'observed'); requested = null;
        }
      } else if (active === null && !detached && page === 'workspace' && queued === null) {
        queued = firstEmbeddedProfile(profiles,uncertainWindowSlots);
      }
      if(layoutChanged) {
        previewStatuses=[];pendingPreviewInvalidations.clear();dirty=true;
      }
    } catch (error) {
      if (isCurrentObservation(issued)) {
        if (available) dirty = true;
        available = false;agentActivity=null;previewStatuses=[];pendingPreviewInvalidations.clear();
        connectionIssueKey='connection';
      }
    } finally { refreshing = false; }
  }
  async function launch(slot: ProfileSlot) {
    // Read the current observed slot again; queued or stale card clicks do
    // not grant permission to launch an ATTENTION/uncertain profile.
    if (closeInProgress || navigationReviewOpen || activeOperation !== null || !available || !binding?.configured ||
        !canRequestProfileLaunch(profiles.find(profile => profile.slot === slot),launchFailures)) return;
    if(refreshing||resizing){queuedLaunch=slot;return;}
    activeOperation = 'launch'; issueKey=null; requested = slot;
    clearLaunchFailure(slot);
    try {
      publishProfiles((await lazyTester.launch(slot)).profiles); storage = null; available = true;
      detached = false; page = 'workspace'; dirty = true;
      if (requested === slot && profiles.find(profile => profile.slot === slot)?.state === 'RUNNING') queued = slot;
      else if (active !== null) {
        await tick();
        try { await presentCurrentViewport(active); }
        catch { issueKey='selection'; }
      }
    } catch (error) { recordLaunchFailure(slot,'unverified'); requested = null; }
    finally { activeOperation = null; }
  }
  function navigate(next: typeof page) {
    // Settings and storage require the Windows native session owner.
    if (nativeBackend !== 'WindowsNative' || closeReviewActive(reviewState()) ||
        selectiveOpen || activeOperation === 'clearData' || storageReview !== null) return;
    closeOneTrigger=null;
    // A queued Refresh belongs to its original page; moving to another
    // page cancels only the undispatched UI request, never a native action.
    if (next !== page) {
      manualRefreshRequested = false;
      headerRefreshFocusPage = null;
    }
    queuedPage = next; queued = null;queuedLaunch=null;
  }
  async function consumeNavigation() {
    const next = queuedPage; queuedPage = null;
    if (next === null || page === next) return;
    // Navigation reads share the original close-review epoch with Inspect.
    // A completed native suspend/present is not itself a UI or game receipt.
    const issued = observationEpoch;
    const current = () => isCurrentObservation(issued);
    activeOperation = 'navigate'; loadingPage = next; issueKey=null;
    try {
      if (next !== 'workspace') await workspaceSurface.suspend();
      if (!current()) return;
      page = next; await tick();
      if (!current()) return;
      // Focus content only for Settings/Storage navigation. Do not steal game focus in Play.
      if (next !== 'workspace' && pageHeading?.isConnected)
        pageHeading.focus({preventScroll:true});
      if (next === 'workspace' && active !== null) {
        await presentCurrentViewport(active);
        if (!current()) return;
      }
      if(next==='settings'||next==='storage') {
        lastInspectionStartedAt=performance.now();
        sessionObservationHint=false;
        const snapshot=await lazyTester.settings();
        if (!current()) return;
        publishProfiles(snapshot.profiles);observeSettings(snapshot.settings);available=true;
      }
      if(next==='storage') {
        try {
          const location=await lazyTester.dataLocation();
          if (!current()) return;
          dataLocation=location;locationIssue='';
        } catch(error) {
          if (!current()) return;
          locationIssue=userMessage('location');dataLocation=null;storage=null;binding=null;
        }
      }
      if(next==='storage'&&binding?.configured) {
        storage=null;
        if (!await inspectSelectedStorage(issued)) return;
      }
      if (current()) dirty = true;
    } catch (error) {
      if (current()) {issueKey='workspace';dirty=true;}
    }
    finally { loadingPage = null; activeOperation = null; }
  }
  function requestHeaderRefresh(event: MouseEvent): void {
    if (navigationReviewOpen) return;
    // A keyboard-triggered click (detail=0) would disable its own button.
    // Move focus first to the existing page heading; a user Tab/Shift+Tab
    // choice from there always supersedes our eventual restoration.
    if (event.detail === 0 && document.hasFocus() &&
        document.activeElement === event.currentTarget && pageHeading?.isConnected) {
      headerRefreshFocusPage = page;
      pageHeading.focus({preventScroll:true});
    }
    manualRefreshRequested=true;
  }
  async function refreshPage() {
    const keyboardOrigin = headerRefreshFocusPage;
    headerRefreshFocusPage = null;
    if (nativeBackend !== 'WindowsNative' || navigationReviewOpen ||
        activeOperation !== null || refreshing || resizing) return;
    const issued = observationEpoch;
    const current = () => isCurrentObservation(issued);
    manualRefreshing=true;
    try {
    issueKey=null;
    if (page === 'storage') {
      activeOperation = 'refreshStorage';
      try {
        if(!binding?.configured)await checkBinding();
        if (!current()) return;
        try {
          const location=await lazyTester.dataLocation();
          if (!current()) return;
          dataLocation=location;locationIssue='';
        } catch(error) {
          if (!current()) return;
          locationIssue=userMessage('location');dataLocation=null;storage=null;binding=null;
          throw error;
        }
        if(!binding?.configured){
          // A readable DataLocation is not proof of verified profile handles.
          // Do not present a previous storage report or success receipt as refreshed.
          // Preserve the independently observed game session; do not retry/mutate it.
          storage=null;storageNotice='';issueKey='binding';
          return;
        }
        if (!await inspectSelectedStorage(issued)) return;
      }catch(error){
        if (current()) {issueKey='storage';storage=null;}
      }
      finally{activeOperation = null;}
    } else {
      // Only explicit Refresh of a known preview failure requests another
      // DWM layout. Periodic session inspection and worker errors never
      // schedule additional native preview IPC.
      const previewFailed = previewIssue !== '' ||
        pendingPreviewInvalidations.size > 0 ||
        previewStatuses.some(status => !status.registered && status.reason !== null);
      if (!binding?.configured) await loadBinding();
      if (!current()) return;
      await refresh();
      if (!current()) return;
      if (previewFailed && page === 'workspace' && available &&
          !detached) dirty = true;
    }
    }finally{
      manualRefreshing=false;
      if (keyboardOrigin !== null) {
        await tick();
        if (!disposed && document.hasFocus() && page === keyboardOrigin &&
            document.activeElement === pageHeading &&
            headerRefreshButton?.isConnected && !headerRefreshButton.disabled)
          headerRefreshButton.focus({preventScroll:true});
      }
    }
  }
  async function saveWebCachePreference(enabled:boolean):Promise<void> {
    if(activeOperation !== null||refreshing||resizing||!available||navigationReviewOpen||settings?.storagePreferences===undefined)return;
    activeOperation = 'saveStoragePreferences';storageNotice='';
    try {
      const snapshot=await lazyTester.saveStoragePreferences({...settings.storagePreferences,autoClearWebCache:enabled});
      publishSettings(snapshot.settings);publishProfiles(snapshot.profiles);
      storageNotice='Web cache preference saved.';
    } catch {
      // An uncertain save must not be represented as the old or new value.
      if(settings)settings={...settings,storagePreferences:undefined};
      storageNotice='Web cache preference could not be confirmed. Refresh settings.';
    } finally {activeOperation = null;}
  }
  async function saveAudio(mode:'Keep'|'Mute') {
    if(activeOperation !== null||!available||!settings?.backgroundAudio)return;
    if(refreshing||resizing){queuedAudio=mode;return;}
    activeOperation = 'saveAudio';issueKey=null;settingsNotice='';
    try {const snapshot=await lazyTester.saveAudio(mode);publishSettings(snapshot.settings);publishProfiles(snapshot.profiles);settingsNotice='Background audio saved and applied to managed games.';}
    catch(error){issueKey='audio';}
    finally{activeOperation = null;}
  }
  function requestPerformanceSave() {
    if(activeOperation !== null||queuedPerformance!==null||!available||!settings||!performanceValid||!performanceDirty)return;
    queuedPerformance={active:activeFps!,background:backgroundFps!};
  }
  async function savePerformance(limits:{active:number;background:number}) {
    if(!available||!settings){issueKey='performanceBlocked';return;}
    activeOperation = 'savePerformance';issueKey=null;performanceNotice='';
    try{
      const snapshot=await lazyTester.savePerformance(limits.active,limits.background);
      publishProfiles(snapshot.profiles);publishSettings(snapshot.settings);
      activeFps=snapshot.settings.activeFps;backgroundFps=snapshot.settings.backgroundFps;
      performanceNotice='FPS limits saved. Applies when each game next starts.';
    }catch(error){issueKey='performance';}
    finally{activeOperation = null;}
  }
  function cancelCloseOne() {
    if (closeOneSlot===null) return;
    const trigger=closeOneTrigger;
    closeOneSlot=null;closeOneTrigger=null;
    // A review invalidates in-flight layout reports. Reconcile the latest
    // viewport on Cancel even when ResizeObserver did not detect a size change.
    dirty=true;
    // Restore keyboard context without scrolling the fixed five-card rail.
    void tick().then(()=>{
      if (!disposed && trigger?.isConnected && !trigger.disabled) trigger.focus({preventScroll:true});
    });
  }
  function closeReviewKeydown(event: KeyboardEvent) {
    // Scoped to the focused review: Escape elsewhere belongs to its actual
    // control, not to an unrelated Minecraft CloseOne confirmation.
    if (closeOneSlot===null || event.key!=='Escape' ||
        !(event.target instanceof Node) ||
        !closeOneReviewElement?.contains(event.target)) return;
    event.preventDefault();
    event.stopPropagation();
    cancelCloseOne();
  }
  function requestCloseOne(slot: ProfileSlot, trigger: HTMLButtonElement) {
    if (closeInProgress || nativeExitReview || activeOperation !== null || closingOneSlot !== null ||
        closeOneSlot !== null || activeRecordingSlots.has(slot)) return;
    const profile=profiles.find(item=>item.slot===slot);
    if (!profile || !['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state)) return;
    // Keep the user's confirmation available while an old Inspect finishes,
    // but never allow that result to overwrite state after review began.
    observationEpoch++;
    // A confirmation supersedes any uncommitted UI intent. Do not replay
    // another slot's queued launch/selection after the user presses Cancel.
    queuedLaunch=null;queued=null;queuedPage=null;
    closeOneTrigger=trigger;closeOneSlot=slot;
    // Freeze lifecycle/preview admission during review; any pending geometry
    // is reconciled by the existing scheduler on Cancel or close completion.
    // Cancel is the safe initial keyboard choice. Do not reselect/launch games.
    void tick().then(()=>{
      if (!disposed && closeOneSlot===slot && closeCancelButton?.isConnected) closeCancelButton.focus();
    });
  }
  // Read-only reconciliation never confirms native window restoration.
  async function reconcileCloseObservation() {
    try {
      publishProfiles((await lazyTester.inspect()).profiles);
      available = true; connectionIssueKey = null;
    } catch {
      available = false; connectionIssueKey = 'connection';
    }
  }
  function markClosePresentationUncertain(slot: ProfileSlot) {
    uncertainWindowSlots = updateRecoverySlots(uncertainWindowSlots, slot, false);
    issueKey = 'recovery';
    if (active === slot) {
      active = null; selectedIdentity = ''; requested = null; queued = null;
      detached = true; previewStatuses = [];
    }
  }
  async function confirmCloseOne() {
    const slot=closeOneSlot;
    if (slot===null || closeInProgress || nativeExitReview || activeOperation !== null ||
        refreshing || resizing || closingOneSlot!==null || activeRecordingSlots.has(slot)) return;
    // A committed CloseOne review disappears immediately. Before its focused
    // button is removed, anchor keyboard context to the SAME stable MC slot
    // group, not to body, another Minecraft process or a new Tab stop.
    const entry = closeOneTrigger?.closest<HTMLElement>('.lazytester-launcher-entry');
    if (document.hasFocus() && closeOneReviewElement?.contains(document.activeElement) &&
        entry?.isConnected)
      entry.focus({preventScroll:true});
    closeOneSlot=null;closeOneTrigger=null; closingOneSlot=slot; activeOperation = 'closeOne'; issueKey=null;
    try {
      const report=await lazyTester.closeOne(slot);
      if (!report.presentationReleased) {
        // Release was attempted but did not confirm restoration. Do not
        // pretend the previous Play surface still owns the original HWND.
        markClosePresentationUncertain(slot);
        await reconcileCloseObservation();
        return;
      }
      uncertainWindowSlots = updateRecoverySlots(uncertainWindowSlots, slot, true);
      // Only a confirmed native restoration receipt permits a normal
      // selected close transition and automatic selection of another peer.
      const wasActive=active===slot;
      if (wasActive) {
        active=null;selectedIdentity='';requested=null;queued=null;
      }
      if (report.snapshot) {
        publishProfiles(report.snapshot.profiles);available=true;connectionIssueKey=null;storage=null;
        if (wasActive) {
          // Existing background games may still be parked offscreen with
          // WS_EX_TOOLWINDOW. Promote another verified responsive peer into
          // Play instead of leaving it inaccessible without a preview.
          queued=firstEmbeddedProfile(profiles,uncertainWindowSlots);
          detached=queued===null;
        }
      } else {
        issueKey='close';
        await reconcileCloseObservation();
        if (wasActive) {
          // Never automatically reselect the game whose exit outcome is
          // uncertain; a different responding peer can still be presented.
          const excluded=new Set<ProfileSlot>([...uncertainWindowSlots,slot]);
          queued=available?firstEmbeddedProfile(profiles,excluded):null;
          detached=queued===null;
        }
      }
    } catch (error) {
      // A transport failure or invalid receipt cannot establish whether the
      // Rust command reached native window restoration. The UI must stop
      // claiming an embedded selected HWND; no extra close/retry is sent.
      markClosePresentationUncertain(slot);
      // Read-only reconciliation cannot certify window restoration and must
      // not clear uncertainWindowSlots, even when the game has stopped.
      await reconcileCloseObservation();
    } finally {
      closingOneSlot=null;activeOperation = null;dirty=true;
    }
  }

  function presentCurrentViewport(slot:ProfileSlot):Promise<void> {
    const rect=viewport.getBoundingClientRect();
    const pixelRatio=window.devicePixelRatio;
    return lazyTester.present(slot,rect,pixelRatio);
  }

  function captureWorkspaceGeometry(): WorkspaceGeometry {
    // Only observed RUNNING background processes can supply preview targets.
    const tiles = others.filter(profile=>profile.state==='RUNNING'&&!!profile.process).flatMap(profile => {
      const element = images.get(profile.slot); if (!element) return [];
      const rect = element.getBoundingClientRect();
      return rect.x >= 0 && rect.y >= 0 && rect.width >= 32 && rect.height >= 24 &&
        rect.right <= window.innerWidth && rect.bottom <= window.innerHeight ? [{slot: profile.slot, rect}] : [];
    });
    return {viewport:viewport.getBoundingClientRect(),tiles,pixelRatio:window.devicePixelRatio};
  }

  async function syncSelectedReviewGeometry() {
    // Clear stale preview destinations even without a selected game.
    // Update only an existing selected workspace lease; never adopt another game.
    if (closeOneSlot===null || closingOneSlot!==null || nativeExitReview ||
        closeInProgress || page!=='workspace' || detached || !available) return;
    const epoch = observationEpoch;
    dirty=false; resizing=true;
    try {
      await tick();
      if (epoch!==observationEpoch || closeOneSlot===null || nativeExitReview ||
          closeInProgress || document.hidden || disposed) return;
      const geometry=captureWorkspaceGeometry();
      // Old DWM destinations cannot follow new CSS coordinates during review.
      // Clear presentation only; do not adopt sources or change game lifecycle.
      await lazyTester.clearPreviews();
      previewStatuses=[];pendingPreviewInvalidations.clear();
      if (epoch!==observationEpoch || closeOneSlot===null || nativeExitReview ||
          closeInProgress || document.hidden || disposed) return;
      if (!sameWorkspaceGeometry(geometry,captureWorkspaceGeometry())) {
        dirty=true;
        return;
      }
      if (active !== null) {
        await workspaceSurface.update(geometry.viewport,geometry.pixelRatio);
        if (!disposed && !sameWorkspaceGeometry(geometry,captureWorkspaceGeometry())) dirty=true;
      }
    } catch {
      // Report through the existing feedback owner after review ends.
      previewIssue=userMessage('preview');
    } finally { resizing=false; }
  }

  async function syncLayout() {
    if (nativeBackend !== 'WindowsNative' || activeOperation !== null || refreshing || resizing || !mounted) return;
    if (closeReviewActive(reviewState())) {
      await syncSelectedReviewGeometry();
      return;
    }
    // This UI snapshot grants no native process/window authority. The
    // original selected slot and process set must still match on return.
    // Profile observations are replaced as complete arrays, never mutated
    // in place by the UI. Retain the original observation across awaits.
    const issued = {
      observationEpoch,
      profiles,
      primarySlot: primary,
    };
    const layoutCurrent = () => previewLayoutStillCurrent(issued,{
      observationEpoch,
      profiles,
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
      const geometry = captureWorkspaceGeometry();
      const geometryCurrent = () => layoutCurrent() &&
        sameWorkspaceGeometry(geometry,captureWorkspaceGeometry());
      if (active !== null) {
        await workspaceSurface.update(geometry.viewport,geometry.pixelRatio);
        if (!geometryCurrent()) {discardOutdated();return;}
      }
      const response = (await lazyTester.previews(geometry.viewport,geometry.tiles,geometry.pixelRatio)).tiles;
      if (!geometryCurrent()) {discardOutdated();return;}
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
    } catch (error) { previewStatuses = [];previewIssue = userMessage('preview'); }
    finally { resizing = false; }
  }
  // Reuse the validated full-field comparison for action receipts too.
  // Keep caller-specific status, timestamps and recovery effects independent.
  function publishProfiles(incoming: ProfileSession[]) {
    if (!sameProfileObservation(profiles, incoming)) profiles = incoming;
  }
  function publishSettings(incoming: SettingsReport) {
    lastSettingsObservedAt = performance.now();
    if (!sameSettingsObservation(settings, incoming)) settings = incoming;
  }
  function observeSettings(incoming: SettingsReport) {
    // Only passive observation adopts FPS values when the user has no draft.
    if (!performanceDirty) {
      activeFps = incoming.activeFps; backgroundFps = incoming.backgroundFps;
    }
    publishSettings(incoming);
  }
  function applyStorageSnapshot(snapshot: {profiles: ProfileSession[]; storage: ProfileStorage[] | null}) {
    publishProfiles(snapshot.profiles);
    storage = snapshot.storage;
    storageReportReceivedAt = storageReportReceivedTime();
  }
  // The three explicit Storage read paths share one request/publication fence.
  // Callers keep the existing operation admission and page-specific feedback.
  async function inspectSelectedStorage(issued:number):Promise<boolean> {
    const slot=selectedSlot;
    const snapshot=await lazyTester.inspectProfileStorage(slot);
    if(!isCurrentObservation(issued)||selectedSlot!==slot)return false;
    applyStorageSnapshot(snapshot);
    available=true;connectionIssueKey=null;
    return true;
  }
  async function loadBinding(importProfiles = false) {
    if (nativeBackend !== 'WindowsNative' || activeOperation !== null || refreshing || resizing || disposed) return;
    activeOperation = 'binding'; issueKey=null;
    try { await checkBinding(importProfiles); }
    finally { activeOperation = null; }
  }
  async function initializeLaunchers() {
    initializing=true;
    let diagnostic=false;
    try {
      // Read non-mutating capabilities from Rust. Unsupported platforms do
      // not retry Windows profile discovery or request game/session IPC.
      try {
        const capabilities = await lazyTester.startupCapabilities();
        if (disposed) return;
        nativeBackend = capabilities.backend;
        diagnostic = capabilities.diagnostic;
        screenshotAvailable = capabilities.screenshot;
        recordingAvailable = capabilities.recording;
      } catch (error) {
        if (!disposed) {binding=null;bindingIssue=userMessage('backend');}
        return;
      }
      if (disposed) return;
      if (nativeBackend === 'Unsupported') {
        binding=null;
        bindingIssue='Minecraft Education native session management is not implemented on this platform. No game, account or profile data was accessed.';
        queuedPage=null;queuedLaunch=null;queued=null;manualRefreshRequested=false;dirty=false;
        return;
      }
      // Capture status is needed only when the native approval gate is open.
      if(recordingAvailable) void refreshRecordingStatus();
      // Initialization may prepare or recover existing data. Its failure
      // is not proof that repeating it is safe or useful. Retry connection
      // remains an explicit user action; do one automatic attempt.
      await loadBinding();
      // Session observation is independent of binding success. Never
      // reopen all profile stores through DataLocation just to enter Play.
      if(!disposed)await refresh();
    }finally{
      if(!disposed){
        initializing=false;
        if (diagnostic) {
          await tick();
          if (!disposed) void lazyTester.reportStartup({
            connection:sessionConnectionLabel(initializing,nativeBackend,!!binding?.configured,available),
            issue:document.querySelector('.lazytester-binding p')?.textContent??'',
            cards:document.querySelectorAll('.lazytester-launcher-card').length,
            enabled:document.querySelectorAll('.lazytester-launcher-card:not(:disabled)').length,
          }).catch(()=>console.warn('Startup observation unavailable'));
        }
      }
    }
  }
  async function checkBinding(importProfiles = false) {
    try {
      const result = await lazyTester.binding(importProfiles);
      if (!disposed) { binding = result; bindingIssue = ''; }
    } catch (error) {
      if (!disposed) { binding = null; bindingIssue = userMessage('binding'); }
    }
  }
  async function loadLocation(action:'Inspect'|'Open'|'Locate'|'Choose'='Inspect') {
    if(activeOperation !== null||refreshing||resizing||disposed||navigationReviewOpen||closeReviewActive(reviewState()))return;
    if(action==='Open'&&(!available||!binding?.configured||!dataLocation?.path))return;
    const issued = observationEpoch;
    activeOperation = 'dataLocation';issueKey=null;locationIssue='';
    try {
      const previousPath = dataLocation?.path;
      const location=await lazyTester.dataLocation(action);
      // Opening Explorer may finish during a native exit review; do not
      // publish its old observation. Locate/Choose retain mutation receipts.
      if(disposed||(action==='Open'&&!isCurrentObservation(issued)))return;
      dataLocation=location;
      if (dataLocation.path !== previousPath) { storage = null; cancelStorageReview('cache'); storageNotice = ''; }
      if((action==='Locate'||action==='Choose')&&dataLocation.path!==null)await checkBinding();
    } catch(error){
      if(disposed||(action==='Open'&&!isCurrentObservation(issued)))return;
      locationIssue=userMessage('location');
      if(action==='Inspect'){
        dataLocation=null;storage=null;cancelStorageReview('cache');storageNotice='';
        binding=null;if(!bindingIssue)bindingIssue=locationIssue;
      }
    }finally{activeOperation = null;}
  }
  async function openFolder(slot: ProfileSlot) {
    if (activeOperation !== null || refreshing || resizing) return;
    activeOperation = 'openFolder'; issueKey=null;
    try { await lazyTester.openFolder(slot); }
    catch (error) { issueKey='folder'; }
    finally { activeOperation = null; }
  }
  async function confirmWorlds(slot: ProfileSlot, receipt: import('./app/bridge/lazyTester.js').SelectiveReviewReceipt): Promise<void> {
    if(activeOperation !== null||refreshing||resizing||!available||!binding?.configured||!selectiveOpen||slot!==selectedSlot||selectedProfile?.state!=='STOPPED'||closeReviewPending||!selectedStorage||selectedStorage.fullClear?.pending===true)throw new Error('Selected data cleanup is unavailable');
    activeOperation = 'confirmSelectedData';storageNotice='';
    try {
      const snapshot=receipt.items.every(item=>item.category==='Worlds')
        ?await lazyTester.confirmSelectedWorlds(slot,receipt):await lazyTester.confirmSelectedPacks(slot,receipt);
      applyStorageSnapshot(snapshot);
      storageNotice='Selected data deleted.';
    } catch(error) {
      storage=null;
      storageNotice='Cleanup was refused or its result is uncertain. Refresh session and storage.';
      throw error;
    } finally {activeOperation = null;}
  }
  async function confirmDownloadCache(slot:ProfileSlot):Promise<void> {
    const report=storage?.find(item=>item.slot===slot)?.downloadCache;
    if(activeOperation !== null||refreshing||resizing||!available||!binding?.configured||!selectiveOpen||
      slot!==selectedSlot||selectedProfile?.state!=='STOPPED'||closeReviewPending||
      selectedStorage?.fullClear?.pending===true||!report?.canClear||report.partial||
      report.issue!==null||!report.files)throw new Error('Download cache cleanup is unavailable');
    activeOperation = 'clearDownloadCache';storageNotice='';
    try {
      const snapshot=await lazyTester.clearDownloadCache(slot);
      applyStorageSnapshot(snapshot);
      storageNotice='Download cache cleared.';
    } catch(error) {
      storage=null;
      storageNotice='Download cache cleanup was refused or its result is uncertain. Refresh session and storage.';
      throw error;
    } finally {activeOperation = null;}
  }
  function cancelStorageReview(kind: StorageReviewKind) {
    if (storageReview?.kind === kind) storageReview = null;
  }
  function reviewCache() {
    if (navigationReviewOpen || storageReview !== null || activeOperation !== null || refreshing || resizing) return;
    storageReview = {kind:'cache',slot:selectedSlot};
  }
  async function clearCache(slot: ProfileSlot) {
    if (activeOperation !== null || refreshing || resizing || storageReview?.kind !== 'cache' ||
        storageReview.slot !== slot || !storage?.find(item=>item.slot===slot)?.cache.canClear) return;
    activeOperation = 'clearCache'; issueKey=null; storageNotice = ''; storageReview = null;
    try {
      const snapshot = await lazyTester.clearCache(slot);
      applyStorageSnapshot(snapshot);
      storageNotice = 'Web cache for Minecraft - ' + slot + ' cleared.';
    } catch (error) {
      issueKey='cache'; storage = null;
    } finally { activeOperation = null; }
  }
  async function recoverSettings() {
    const pending=storageReview;
    if(pending?.kind!=='settings'||activeOperation !== null||refreshing||resizing||pending.slot!==selectedSlot)return;
    const status=selectedStorage?.gameSettings;
    const allowed=pending.action==='Reset'?status?.canReset:status?.canRestore;
    if(!allowed)return;
    activeOperation = 'recoverSettings';issueKey=null;storageNotice='';storageReview=null;
    try {
      const snapshot=await lazyTester.recoverSettings(pending.slot,pending.action);
      applyStorageSnapshot(snapshot);
      storageNotice=`Minecraft - ${pending.slot}: settings ${pending.action==='Reset'?'reset':'restored'}. The global optimized graphics preset applies at next launch. Previous settings remain available through Restore.`;
    } catch(error) {issueKey='settings';storage=null;}
    finally {activeOperation = null;}
  }
  function reviewFullClear() {
    const status=selectedStorage?.fullClear;
    if(activeOperation !== null||navigationReviewOpen||storageReview!==null||!status?.canClear||!status.confirmationToken)return;
    fullName='';fullAck=false;
    storageReview={kind:'full',slot:selectedSlot,token:status.confirmationToken,pending:status.pending};
    void tick().then(()=>document.getElementById('full-clear-name')?.focus());
  }
  function reviewSettings(action:StorageSettingsAction) {
    if(navigationReviewOpen||storageReview!==null||activeOperation !== null||refreshing||resizing||!available)return;
    storageReview={kind:'settings',slot:selectedSlot,action};
  }
  async function clearAllData() {
    const pending=storageReview,status=selectedStorage?.fullClear;
    if(pending?.kind!=='full'||activeOperation !== null||refreshing||resizing||!fullAck||pending.slot!==selectedSlot||fullName!==`Minecraft - ${pending.slot}`||
      !status?.canClear||status.confirmationToken!==pending.token)return;
    // The exact typed-name/ack/token admission above remains authoritative.
    // This is only a focus handoff from the disappearing confirmation form
    // to its persistent warning heading; no extra deletion is requested.
    const deletionReview = document.querySelector('.lazytester-delete-confirm');
    if (document.hasFocus() && document.activeElement instanceof Node &&
        deletionReview?.contains(document.activeElement))
      document.getElementById('lazytester-clear-data-heading')?.focus({preventScroll:true});
    activeOperation = 'clearData';issueKey=null;storageNotice='';storageReview=null;
    try {
      const snapshot=await lazyTester.clearData(pending.slot,pending.token,fullName);
      applyStorageSnapshot(snapshot);
      storageNotice=`Minecraft - ${pending.slot}: local data cleared. Next launch uses an empty profile. Other launchers, Minecraft installation and app preferences stay.`;
    } catch(error) {issueKey='clearData';storage=null;}
    finally {activeOperation = null;}
  }
  function clearBackgroundObservation() {
    if (observationTimer !== undefined) window.clearTimeout(observationTimer);
    observationTimer = undefined;
  }
  function scheduleBackgroundObservation(sessionDue: number | null, recordingDue: number | null) {
    clearBackgroundObservation();
    const due = nextBackgroundDeadline(sessionDue, recordingDue);
    if (due === null || disposed) return;
    const timer = window.setTimeout(() => {
      if (observationTimer !== timer || disposed) return;
      observationTimer = undefined;
      // Visibility can change before its event is delivered. Publish the
      // observed hidden state so the next visible event rearms this deadline.
      if (document.hidden) { managerVisible = false; return; }
      const now = performance.now();
      // A timer that fires early must not lose either original deadline.
      if (now < due) {
        scheduleBackgroundObservation(sessionDue, recordingDue);
        return;
      }
      // As before, capture status is an independent bounded read. Native
      // admission and the existing action dispatcher remain authoritative.
      if (recordingDue !== null && recordingDue <= now && recordingObservationEnabled)
        void refreshRecordingStatus();
      if (sessionDue !== null && sessionDue <= now && sessionObservationEnabled &&
          !closeReviewActive(reviewState()) && binding?.configured && !selectiveOpen && storageReview===null)
        void refresh(passiveIdleAllowed(page, available, workspaceSettledIdle,
          sessionObservationHint, settings, now - lastSettingsObservedAt));
    }, Math.max(0, Math.ceil(due - performance.now())));
    observationTimer = timer;
  }
  onMount(() => {
    try { workspaceLayout = window.localStorage.getItem(workspaceLayoutKey) === 'Right' ? 'Right' : 'Bottom'; }
    catch { layoutPersistenceIssue = 'Saved layout unavailable. Using Bottom.'; }
    managerVisible = !document.hidden;
    mounted = true;
    // Geometry-only changes outside Play cannot acquire a new preview.
    // Preserve explicit cleanup, visibility transitions and failed recovery.
    observer = new ResizeObserver(() => {
      if (page==='workspace' || previewIssue!=='') dirty = true;
    });
    try { observer.observe(viewport, {box: 'device-pixel-content-box'}); }
    catch { observer.observe(viewport); } // Older WebView fallback; DPR listener below remains active.
    observer.observe(stageSpace);
    for (const node of images.values()) observer.observe(node);
    const changed = (event?: Event) => {
      // Hidden/minimized WebViews cannot claim current DWM visibility.
      // Losing focus while the manager remains shown is not a hide.
      const visible = !document.hidden;
      const visibilityChanged = managerVisible !== visible;
      managerVisible = visible;
      if (visibilityChanged && visible) sessionObservationHint = true;
      if (!managerVisible) {
        previewStatuses=[];pendingPreviewInvalidations.clear();
      }
      if (page==='workspace' || previewIssue!=='' || visibilityChanged ||
          event?.type==='visibilitychange') dirty = true;
    };
    window.addEventListener('resize', changed); document.addEventListener('visibilitychange', changed);
    const returnedFocus = () => {
      if (available && managerVisible)
        sessionObservationHint=true;
    };
    window.addEventListener('focus', returnedFocus);
    const stopDisplayScale=observeDisplayScale(window,changed);
    // Main can scroll even when native targets retain their original size.
    const workspaceScrolled = (event: Event) => {
      if (page==='workspace' && event.target instanceof Element && event.target.contains(viewport)) dirty=true;
    };
    document.addEventListener('scroll', workspaceScrolled, true);
    const stops: (() => void)[] = [];
    const retain = (stop: () => void) => { if (disposed) stop(); else stops.push(stop); };
    void listen<boolean>('lazy-tester-close-review', event => {
      // UI scheduling hint only; OS confirmation still owns the exit decision.
      if (event.payload && !nativeExitReview) observationEpoch++;
      nativeExitReview = event.payload;
      if (!nativeExitReview) dirty = true;
    }).then(retain).catch(error => { issueKey='event'; });
    void listen<boolean>('lazy-tester-closing', event => {
      closeInProgress = event.payload;
      if (closeInProgress) {
        observationEpoch++;
        closeOneSlot=null;closeOneTrigger=null;
        queuedLaunch=null;
        queued=null;
      }
    }).then(retain).catch(error => { issueKey='event'; });
    void listen<string>('lazy-tester-close-rejected', event => {
      // Resource admission refused before restoring or closing any game.
      // Keep the verified embedded selection, previews and queued work.
      closeInProgress = false;
      nativeExitReview = false;
      issueKey='closeAll';
    }).then(retain).catch(error => { issueKey='event'; });
    void listen<string>('lazy-tester-close-error', event => {
      // The native close attempt already released/restored workspace leases.
      // Never imply the game is still embedded or automatically reacquire it.
      closeInProgress = false;
      nativeExitReview = false;
      closeFailure = userMessage('closeAll');
      active = null; selectedIdentity = ''; requested = null; queued = null; queuedLaunch = null;
      previewStatuses = []; detached = true; page = 'workspace'; dirty = true;
      void refresh(); // Observation only; does not start or capture Minecraft.
    }).then(retain).catch(error => { issueKey='event'; });
    void listen<string>('workspace-surface-error', event => {
      try {
        const notice=parseDwmPreviewInvalidation(event.payload);
        if(!notice) {issueKey='preview';return;}
        if (disposed || nativeBackend!=='WindowsNative' ||
          !dwmPreviewInvalidationApplies(notice,profiles.find(p=>p.slot===notice.slot))) return;
        previewInvalidationEpoch++;
        pendingPreviewInvalidations.set(notice.slot,{...notice,epoch:previewInvalidationEpoch});
        // Hint only the existing session read lane. A stale/unmatched
        // notice is rejected above; unchanged native identity never relayouts.
        sessionObservationHint=true;
        // Preserve peer statuses and avoid direct preview RPC/layout work.
        previewStatuses=previewStatuses.map(status=>status.slot===notice.slot?
          {...status,registered:false,reason:notice.reason}:status);
      } catch (error) {issueKey='preview';}
    }).then(retain).catch(error => { issueKey='event'; });
    void listen<number>('lazy-tester-preview-select', event => {
      if ([1,2,3,4,5].includes(event.payload)) select(event.payload as ProfileSlot);
    }).then(retain).catch(error => { previewIssue = userMessage('preview'); });
    // Register the mounted observation and cleanup paths before starting
    // the asynchronous native initialization.
    void initializeLaunchers();
    return () => {
      mounted = false; disposed = true; clearBackgroundObservation(); observer?.disconnect();stopDisplayScale();
      window.removeEventListener('resize', changed); document.removeEventListener('visibilitychange', changed);
      window.removeEventListener('focus', returnedFocus);
      document.removeEventListener('scroll', workspaceScrolled, true);
      stops.forEach(stop => stop());pendingPreviewInvalidations.clear();
      // Unsupported diagnostic UI holds no native game-window lease.
      if (nativeBackend === 'WindowsNative') void workspaceSurface.release().catch(() => {});
    };
  });
</script>
<svelte:head><title>LazyTester</title></svelte:head>
<svelte:window on:keydown={closeReviewKeydown}/>
<div class="lazytester-app">
  <WindowTitlebar backendSupport={nativeBackend} />
  <header class="lazytester-topbar">
    {#if page==='workspace'}
      <div class="lazytester-brand" aria-label="LazyTester"><span class="lazytester-brand-mark" aria-hidden="true"></span><span class="lazytester-brand-name">LazyTester</span></div>
    {/if}
    <nav aria-label="Main navigation">
      {#if recordingAvailable}
        <div class="lazytester-recording-controls" role="group" aria-label="Minecraft recording">
          <select aria-label="Recording Minecraft profile" bind:value={recordingSlot}
            disabled={busy||closeReviewPending} on:change={()=>recordingNotice=null}>
            {#each profileSlots as slot}
              <option value={slot}>{recordingLabel(slot,recordings,recordingStatusKnown,recordingNotice)}</option>
            {/each}
          </select>
          <button class="lazytester-secondary" type="button"
            aria-label={(canStopRecording?'Stop recording Minecraft - ':'Start recording Minecraft - ')+recordingSlot}
            title={recordingNotice?.slot===recordingSlot?recordingNotice.message:recordingLabel(recordingSlot,recordings,recordingStatusKnown,null)}
            disabled={busy||refreshing||resizing||recordingStatusPending||closeReviewPending||navigationReviewOpen||
              (!canStopRecording&&!canStartRecording)}
            on:click={()=>void recordingAction()}>{recordingStatusKnown&&selectedRecording?.state==='Stopping'?'Stopping…':canStopRecording?'Stop':'Record'}</button>
          <span class="lazytester-visually-hidden" role="status">{recordingLabel(recordingSlot,recordings,recordingStatusKnown,recordingNotice)}</span>
        </div>
      {/if}
      {#if page==='workspace'}
        <button class="lazytester-secondary lazytester-settings-button" disabled={navigationReviewOpen||nativeBackend!=='WindowsNative'} on:click={()=>navigate('settings')}>Settings</button>
      {:else}
        <button class="lazytester-secondary" disabled={navigationReviewOpen} on:click={()=>navigate('workspace')}>Back</button><span class="lazytester-settings-title">Settings</span>
      {/if}
    </nav>
    {#if page==='workspace'&&available&&activityView}
      <AgentActivityNotice view={activityView}/>
    {/if}
    {#if initializing || nativeBackend !== 'WindowsNative' || !binding?.configured || !available}
    <span class="lazytester-connection" role="status" title={sessionConnectionLabel(initializing,nativeBackend,!!binding?.configured,available)}><span class="lazytester-dot" class:waiting={!available||!binding?.configured}></span>{sessionConnectionLabel(initializing,nativeBackend,!!binding?.configured,available)}</span>
    {/if}
  </header>
  <div class="lazytester-layout" class:lazytester-settings-layout={page!=='workspace'} inert={closeInProgress}>
    {#if page!=='workspace'}<aside class="lazytester-sidebar"><nav aria-label="Settings navigation"><button disabled={navigationReviewOpen} class:chosen={page==='settings'} aria-current={page==='settings'?'page':undefined} on:click={()=>navigate('settings')}>General</button><button disabled={navigationReviewOpen} class:chosen={page==='storage'} aria-current={page==='storage'?'page':undefined} on:click={()=>navigate('storage')}>Data Storage</button></nav></aside>{/if}
    <main class={page==='workspace'?'lazytester-play-main':undefined}>
      <header class="lazytester-page-header" class:lazytester-play-header={page==='workspace'}>
        <div><h1 bind:this={pageHeading} tabindex="-1">{page==='workspace'?(detached?(uncertainWindowSlots.size > 0?'Window recovery unconfirmed':'Workspace'):active?'Minecraft '+active:initializing?'Preparing your launchers':nativeBackend==='Unsupported'?'Minecraft backend unavailable':!binding?.configured?'Launcher connection unavailable':!available?'Session status unavailable':requested?'Starting Minecraft - '+requested:'Workspace'):page==='storage'?'Data Storage':'General'}</h1>{#if page==='workspace'}<p hidden={active!==null&&!detached}>{detached?(uncertainWindowSlots.size > 0?'Session state and native window restoration must be verified separately.':'Select a Minecraft launcher.'):active?'Select another Minecraft launcher to switch.':binding?.configured?'Each launcher keeps its own data.':'Existing data will be preserved.'}</p>{/if}</div>
        <div class="lazytester-header-actions">
          {#if page==='workspace' && nativeBackend==='WindowsNative'}<span class="lazytester-games-count" title={gamesStatusSummary}>{gamesStatusSummary}</span>{/if}
          {#if page==='workspace'&&screenshotAvailable&&active!==null&&!detached}
          <button class="lazytester-secondary" disabled={busy||refreshing||resizing||closeReviewPending||navigationReviewOpen}
            on:click={()=>void saveScreenshot()}>{screenshotBusy?'Saving…':'Screenshot'}</button>
          {/if}
          {#if showHeaderRefresh}
          <button class="lazytester-secondary" bind:this={headerRefreshButton}
            disabled={busy||manualRefreshing||manualRefreshRequested||navigationReviewOpen||nativeBackend!=='WindowsNative'}
            on:click={requestHeaderRefresh}>{manualRefreshing||manualRefreshRequested?'Refreshing…':'Refresh'}</button>
          {/if}
        </div>
      </header>
      {#if loadingPage===page && page!=='workspace'}
        <p class="lazytester-page-loading" role="status" aria-live="polite">{page==='storage'?'Checking saved profile storage…':'Checking current settings…'}</p>
      {/if}
      {#snippet actionFeedback(compact: boolean)}
        <div class="lazytester-alert" class:lazytester-workspace-action-feedback={compact} role="alert" tabindex="0" aria-label="Action needs attention">
          {#if issueKey!==null}
            <div class="lazytester-alert-item">
              <strong>{feedbackHeading(issueKey)}</strong>
              <p>{userMessage(issueKey)}</p>
            </div>
          {/if}
          {#if connectionIssueKey!==null}
            <div class="lazytester-alert-item">
              <strong>{feedbackHeading(connectionIssueKey)}</strong>
              <p>{userMessage(connectionIssueKey)}</p>
            </div>
          {/if}
          <div class="lazytester-alert-footer">
            {#if nativeBackend==='WindowsNative'}
              <button class="lazytester-secondary lazytester-alert-refresh" type="button"
                disabled={busy||manualRefreshing||manualRefreshRequested||navigationReviewOpen||nativeBackend!=='WindowsNative'}
                title="Checks current status only; does not repeat the failed action."
                on:click={requestHeaderRefresh}>Refresh status</button>
            {/if}
            <details class="lazytester-feedback-reference">
              <summary>Error codes</summary>
              {#if issueKey!==null}<p>{userFeedback(issueKey).area} · {userFeedback(issueKey).code}</p>{/if}
              {#if connectionIssueKey!==null}<p>{userFeedback(connectionIssueKey).area} · {userFeedback(connectionIssueKey).code}</p>{/if}
            </details>
          </div>
        </div>
      {/snippet}
      <div class="lazytester-global-feedback" class:lazytester-play-feedback={page==='workspace'}>
      {#if (issueKey!==null||connectionIssueKey!==null) && !workspaceActionFeedback}
        {@render actionFeedback(false)}
      {/if}
      {#if uncertainWindowSlots.size > 0}
        <section class="lazytester-exit-recovery" role="status" tabindex="0" aria-label="Window recovery unconfirmed">
          <h2>Window recovery needs attention</h2>
          <p>{recoveryLabels(uncertainWindowSlots)}: Window restoration unconfirmed. Check game windows before closing again. Refresh checks status only.</p>
        </section>
      {/if}
      {#if closeFailure}
        <section class="lazytester-exit-recovery" role="status" tabindex="0" aria-label="Exit recovery">
          <h2>Exit was not completed</h2>
          <p>Some games may still be running. Other sessions were preserved. Check open dialogs or select a responsive game below.</p>
          <div class="lazytester-actions"><button class="lazytester-secondary" on:click={()=>{closeFailure='';void refresh();}}>Dismiss and refresh</button></div>
          
        </section>
      {/if}

      {#if nativeBackend===null&&!initializing}
        <section class="lazytester-binding" tabindex="0" aria-label="Backend capability unavailable">
          <div><strong>Backend capability could not be verified</strong><p>{bindingIssue||'Native support check failed. No Minecraft operation was attempted.'}</p></div>
          <div class="lazytester-actions"><button disabled={busy} on:click={()=>void initializeLaunchers()}>Retry backend check</button></div>
        </section>
      {:else if nativeBackend==='Unsupported'&&!initializing}
        <section class="lazytester-binding" tabindex="0" aria-label="Unsupported platform">
          <div><strong>Minecraft backend unavailable</strong><p>{bindingIssue}</p></div>
        </section>
      {:else if !binding?.configured&&!initializing}<section class="lazytester-binding" tabindex="0" aria-label="Launcher recovery"><div><strong>Existing launcher connection could not be verified</strong><p>{bindingIssue||userMessage('binding')}</p></div><div class="lazytester-actions"><button disabled={busy||manualRefreshing||manualRefreshRequested} on:click={()=>manualRefreshRequested=true}>Retry connection</button><button class="lazytester-secondary" on:click={()=>navigate('storage')}>Data Storage</button></div></section>{/if}
      </div>
      <section class="lazytester-workspace" class:lazytester-workspace-right={workspaceLayout==='Right' && nativeBackend==='WindowsNative'} hidden={page!=='workspace'} aria-label="Minecraft play area">
        {#if nativeBackend === 'WindowsNative'}
        <div class="lazytester-games-heading lazytester-launcher-action-strip">
        {#if closeOneSlot!==null}
          <section class="lazytester-launcher-close-review" aria-label="Confirm one Minecraft close" bind:this={closeOneReviewElement}>
            <div class="lazytester-launcher-close-copy"><h3>Close Minecraft - {closeOneSlot}?</h3>
            <p>Save your work first. Only this game closes; other games keep running.</p></div>
            <div class="lazytester-actions">
              <button disabled={busy||refreshing||resizing||closeInProgress||nativeExitReview} on:click={()=>void confirmCloseOne()}>Close Minecraft - {closeOneSlot}</button>
              <button class="lazytester-secondary" bind:this={closeCancelButton} disabled={busy||closeInProgress||nativeExitReview} on:click={cancelCloseOne}>Cancel</button>
            </div>
          </section>
        {:else if closingOneSlot!==null}<p role="status">Waiting for Minecraft - {closingOneSlot} to close normally…</p>
        {:else if workspaceActionFeedback}
          {@render actionFeedback(true)}
        {:else if previewIssue}
          <div class="lazytester-workspace-preview-feedback" role="status" tabindex="0">
            <strong>Preview unavailable</strong><span>{previewIssue}</span>
          </div>
        {:else if screenshotNotice&&screenshotNotice.slot===active}
          <p role="status">{screenshotNotice.message} <button class="lazytester-secondary" on:click={()=>screenshotNotice=null}>Dismiss</button></p>
        {/if}
        </div>
        {/if}
        <div class="lazytester-workspace-grid">
        <div class="lazytester-stage-space" bind:this={stageSpace}>
        <div class="lazytester-viewport" class:lazytester-desktop={detached} bind:this={viewport}>
          {#if detached}<div class="lazytester-empty" role="region" aria-label="Minecraft workspace status" tabindex="0"><h2>{uncertainWindowSlots.size > 0?'Window restoration not verified':closeFailure?'Exit incomplete — sessions preserved':'Choose a Minecraft to play'}</h2><p>{uncertainWindowSlots.size > 0?'Affected launchers: '+recoveryLabels(uncertainWindowSlots)+'. Refresh checks game status, not window restoration.':closeFailure?'Some games may still be running. Check their windows before returning to Play.':'Select a running launcher or launch a stopped game.'}</p><p>{available&&binding?.configured?observedRunningCount+' of 5 running':'Session status unavailable'}</p></div>{:else if !active}<div class="lazytester-empty" role="region" aria-label="Minecraft workspace status" tabindex="0"><span class="lazytester-empty-mark" aria-hidden="true"></span><h2>{initializing?'Preparing launchers…':nativeBackend==='Unsupported'?'Minecraft backend unavailable':!binding?.configured?'Connection needs attention':!available?'Session status unavailable':requested?'Starting Minecraft - '+requested:'Ready when you are'}</h2><p>{initializing?'Checking existing profiles. No setup needed.':nativeBackend==='Unsupported'?'Minecraft session management is not supported on this OS.':!binding?.configured?'Retry connection. Accounts and worlds are preserved.':!available?'Use Refresh to reconnect.':requested?'Waiting for the game window…':'Select a launcher to start Minecraft.'}</p></div>{/if}
        </div>
        </div>
        {#if nativeBackend === 'WindowsNative'}
        <LauncherRail {profiles} {labels} {healthIssues} {active} {primary} {detached}
          unresolvedRecovery={uncertainWindowSlots} {launchFailures}
          {available} bindingConfigured={!!binding?.configured} {busy}
          {closeInProgress} {nativeExitReview} {guidedRecoveryBlocked} {activeRecordingSlots}
          reviewPending={closeReviewPending}
          observePreview={image} onSelect={select} onLaunch={launch}
          onRequestClose={requestCloseOne} onGuidedRecovery={requestGuidedRecovery}/>
        {/if}
        </div>
      </section>
      {#if page==='settings'}
        <GeneralSettings {settings} {workspaceLayout} {layoutPersistenceIssue} onChooseWorkspaceLayout={chooseWorkspaceLayout}
          bind:activeFps bind:backgroundFps
          {busy} {available} {refreshing} {resizing}
          performanceQueued={queuedPerformance!==null}
          audioQueued={queuedAudio!==null} {savingPerformance}
          {performanceDirty} {performanceValid}
          {settingsNotice} {performanceNotice}
          onRequestPerformanceSave={requestPerformanceSave} onSaveAudio={saveAudio}/>
      {:else if page==='storage'}
        <DataStorage {profiles} {selectedSlot} {selectedProfile} {selectedStorage} {selectedPath}
          {labels} {storage} {storageReportReceivedAt} {storageNotice}
          {busy} {refreshing} {resizing} {available}
          bindingConfigured={!!binding?.configured}
          bind:advanced bind:fullName bind:fullAck
          {storageReview} {clearingData}
          {dataLocation} {locationIssue} {runningCount}
          onChooseSlot={chooseSlot}
          bind:selectiveOpen nativeClosePending={closeReviewPending}
          autoClearWebCache={settings?.storagePreferences?.autoClearWebCache} onSaveWebCachePreference={saveWebCachePreference}
          onInspectSelective={lazyTester.selectiveInventory}
          onReviewSelective={lazyTester.reviewSelectiveInventory}
          onConfirmWorlds={confirmWorlds} onConfirmDownloadCache={confirmDownloadCache}
          onReviewCache={reviewCache} onClearCache={clearCache}
          onOpenFolder={openFolder} onReviewSettings={reviewSettings}
          onRecoverSettings={recoverSettings}
          onReviewFullClear={reviewFullClear} onClearAllData={clearAllData}
          onCancelReview={cancelStorageReview}
          onOpenData={() => loadLocation('Open')}
          onLocateData={() => loadLocation('Locate')}
          onChooseData={() => loadLocation('Choose')}/>
      {/if}
    </main>
  </div>
</div>
