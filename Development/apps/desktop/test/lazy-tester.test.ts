import {sameWorkspaceGeometry, type WorkspaceGeometry} from '../src/app/workspaceGeometry.js';
import {describe, expect, it} from 'vitest';
import {readFileSync} from 'node:fs';
import {lazyTester, parseRecordingStatus, recordingActive, parseSelectedClose, firstEmbeddedProfile, parseSession, parsePreviewReport, parseDwmPreviewInvalidation, dwmPreviewInvalidationApplies, parseProfileBinding, parseDataLocation, validFpsLimits, canRequestLaunch, parseNativeBackendSupport, parseStartupCapabilities, type ProfileSession, type ProfileSlot} from '../src/app/bridge/lazyTester.js';
import {previewViewport} from '../src/app/bridge/workspaceSurface.js';
import {canRequestProfileLaunch, projectSessionHealth, launcherCardGuidance, reconcileLaunchReceipts} from '../src/app/presentation/sessionHealth.js';
import {guidedRecoveryAllowed} from '../src/app/presentation/guidedRecovery.js';
import {userMessage, userFeedback, feedbackHeading, sessionConnectionLabel, updateRecoverySlots, recoveryLabels, type UserMessageKey} from '../src/app/presentation/sessionFeedback.js';
import {closeReviewActive, observationStillCurrent, previewLayoutStillCurrent, releasedSelectionStillCurrent, periodicInspectionInterval, sessionInspectionDeadline, nextBackgroundDeadline, nextPendingUiAction, sameProfileObservation, sameProfileLayout, sameSettingsObservation, sameAgentObservation, type PendingUiQueueState, type PendingUiAction, type CloseReview} from '../src/app/inspectionScheduling.js';

const stopped = () => ({schema: 1, storage: null, profiles: [1,2,3,4,5].map(slot => ({slot, state: 'STOPPED', process: null, issue: null}))});
const running = (slot: number) => ({slot, state: 'RUNNING', issue: null, process: {
  pid: slot + 100, createdFileTime: String(slot + 200), sessionId: 1, windowHandle: `0x${slot}`, responding: true,
}});
describe('Ordered desktop storage receipt admission',()=>{
  const cache={bytes:1,files:1,partial:false,canClear:true,issue:null};
  const gameSettings={hasSettings:true,backupAvailable:true,canReset:true,canRestore:true,issue:null};
  const fullClear={canClear:true,pending:false,confirmationToken:'a'.repeat(64),issue:null};
  const storage=[1,2,3,4,5].map(slot=>({slot,bytes:1,files:1,partial:false,disk:null,
    cache,downloadCache:cache,gameSettings,fullClear}));
  it('checks each mutation receipt against its own validated profile slot',()=>{
    for(const state of ['STARTING','RUNNING','NOT_RESPONDING']){
      const snapshot={...stopped(),profiles:stopped().profiles.map(p=>p.slot===3?{...running(3),state}:p)};
      const blocked={...storage[2]!,cache:{...cache,canClear:false},downloadCache:{...cache,canClear:false},
        gameSettings:{...gameSettings,canReset:false,canRestore:false},fullClear:{...fullClear,canClear:false}};
      const rows=storage.map(row=>row.slot===3?blocked:row);
      expect(parseSession({...snapshot,storage:rows}).storage?.[3]?.cache.canClear).toBe(true);
      for(const key of ['cache','downloadCache','gameSettings','fullClear'] as const){
        const unsafe=rows.map(row=>row.slot===3?{...row,[key]:storage[2]![key]}:row);
        expect(()=>parseSession({...snapshot,storage:unsafe})).toThrow();
      }
    }
  });
  it('binds selected storage to the requested slot and retains full roster admission',()=>{
    const snapshot=stopped();
    const selected={...snapshot,storage:[storage[2]!]};
    expect(parseSession(selected,3).storage).toEqual([storage[2]]);
    expect(()=>parseSession(selected)).toThrow('Invalid storage response');
    expect(()=>parseSession(selected,2)).toThrow('Invalid storage metadata');
    expect(()=>parseSession({...snapshot,storage},3)).toThrow('Invalid storage response');
    expect(()=>parseSession(snapshot,3)).toThrow('Missing selected storage response');
    expect(()=>parseSession({...snapshot,storage:new Array(1)},3)).toThrow('Invalid storage metadata');
    const active={...selected,profiles:snapshot.profiles.map(p=>p.slot===3?running(3):p)};
    expect(()=>parseSession(active,3)).toThrow('Invalid cache metadata');
    const otherActive={...selected,profiles:snapshot.profiles.map(p=>p.slot===1?running(1):p)};
    expect(parseSession(otherActive,3).storage?.[0]?.slot).toBe(3);
  });
  it('refuses sparse rosters, coerced states and mismatched storage order',()=>{
    const snapshot=stopped();
    expect(()=>parseSession({...snapshot,profiles:new Array(5)})).toThrow('Invalid profile response');
    expect(()=>parseSession({...snapshot,profiles:snapshot.profiles.map((p,i)=>i===0?{...p,state:['STOPPED']}:p)})).toThrow('Invalid profile response');
    expect(()=>parseSession({...snapshot,storage:new Array(5)})).toThrow('Invalid storage metadata');
    expect(()=>parseSession({...snapshot,storage:[...storage].reverse()})).toThrow('Invalid storage metadata');
  });
});

describe('Selective inventory wire contract',()=>{
  const item={root:'minecraftWorlds',name:'one',identity:'a'.repeat(64)};
  const inventory={slot:1,category:'Worlds',items:[item]};
  const review={slot:1,items:[{...item,category:'Worlds'}],bytes:7,files:1,warnings:[],confirmationToken:'b'.repeat(64)};
  it('accepts explicit read-only receipts and preserves old snapshots',()=>{
    expect(parseSession(stopped()).selectiveInventory).toBeUndefined();
    expect(parseSession({...stopped(),selectiveInventory:inventory}).selectiveInventory).toEqual(inventory);
    expect(parseSession({...stopped(),selectiveReview:review}).selectiveReview).toEqual(review);
  });
  it('rejects duplicate identity, unsafe names and unverified pack confirmation',()=>{
    expect(()=>parseSession({...stopped(),selectiveInventory:{...inventory,items:[item,{...item,name:'two'}]}})).toThrow();
    expect(()=>parseSession({...stopped(),selectiveReview:{...review,items:[{...review.items[0],name:'../outside'}]}})).toThrow();
    expect(()=>parseSession({...stopped(),selectiveReview:{...review,items:[{...item,category:'ResourcePacks',root:'resource_packs'}]}})).toThrow();
    expect(()=>parseSession({...stopped(),profiles:[running(1),...stopped().profiles.slice(1)],selectiveInventory:inventory})).toThrow();
  });
});
describe('Absent-controller Settings observation',()=>{
  it('reuses the admitted empty inventory for read-only Settings but retains fresh save observation',()=>{
    const source=readFileSync(new URL('../src-tauri/src/engine/native_clients_session.rs',import.meta.url),'utf8');
    const fallback=source.split('fn transact(request: SessionRequest)')[1]!
      .split('// A report or Testing correlation')[0]!;
    const gate=fallback.indexOf('require_absent_controller_idle(observation.processes.len())?;');
    const save=fallback.indexOf('super::manager_settings::save_audio(mode)?;');
    const observation=fallback.indexOf('let idle = matches!(request, SessionRequest::InspectSettings)\n                || super::native_clients::inspect_native_workspace()?.processes.is_empty();');
    expect(gate).toBeGreaterThan(-1);
    expect(save).toBeGreaterThan(gate);
    expect(observation).toBeGreaterThan(save);
    expect(fallback).toContain('snapshot.settings = Some(SettingsReport::policy(idle, None)?);');
  });
});
describe('Passive idle observation boundary',()=>{
  it('allows a negative hint only on the healthy idle scheduler, not on a full receipt',()=>{
    const app=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const bridge=readFileSync(new URL('../src/app/bridge/lazyTester.ts',import.meta.url),'utf8');
    const commands=readFileSync(new URL('../src-tauri/src/commands/native_clients.rs',import.meta.url),'utf8');
    expect(app).toContain("void refresh(page==='workspace' && available && workspaceSettledIdle && !sessionObservationHint)");
    expect(app).toContain('if (!isCurrentObservation(issued) || snapshot===null) return;');
    expect(bridge).toContain("inspect: async () => parseSession(await invokeRuntime<unknown>('lazy_tester_session', {request: {operation: 'Inspect'}}))");
    expect(bridge).toContain('return value===null ? null : parseSession(value);');
    expect(()=>parseSession(null)).toThrow();
    expect(commands).toContain('Passive idle hint is restricted to Inspect');
    expect(commands).toContain('!crate::engine::native_clients_session::passive_idle_needs_inspect()?');
    expect(commands).toContain('crate::engine::native_clients_session::request(request).map(Some)');
  });
});
describe('Storage on-demand boundary', () => {
  it('inspects only selected storage on demand, never in startup or periodic observation', () => {
    const app=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const refresh=app.split('async function refresh(passiveIdle = false)')[1]!.split('async function launch(')[0]!;
    const selection=app.split('function chooseSlot(')[1]!.split('let queuedPerformance:')[0]!;
    expect(refresh).toContain("page === 'settings' || page === 'storage'");
    expect(refresh).toContain('? await lazyTester.settings()');
    expect(refresh).toContain('passiveIdle ? await lazyTester.inspectIdle() : await lazyTester.inspect()');
    expect(refresh).not.toContain('lazyTester.storage(');
    expect(selection).toContain('lazyTester.storage(slot)');
    expect(selection).toContain('storage=null');
    expect(selection).toContain('!isCurrentObservation(issued)||selectedSlot!==slot');
    expect(app.match(/lazyTester\.storage\(/g)).toHaveLength(3);
    expect(app).toContain("if(next==='storage'&&binding?.configured)");
    const bridge=readFileSync(new URL('../src/app/bridge/lazyTester.ts',import.meta.url),'utf8');
    expect(bridge).toContain('value.storage.length !== (selectedStorageSlot === undefined ? 5 : 1)');
    expect(bridge).toContain("operation: 'InspectProfileStorage', slot");
  });
});
describe('Desktop operation ownership', () => {
  it('uses a synchronous operation tag and derives display-only activity flags', () => {
    const app=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const script=app.split('</script>')[0]!;
    expect(script).toContain('let activeOperation: UiOperation | null = null');
    expect(script).toContain('$: busy = activeOperation !== null');
    expect(script).toContain('initializing, busy: activeOperation !== null, refreshing, resizing,');
    expect(script).toContain("$: savingPerformance = activeOperation === 'savePerformance'");
    expect(script).toContain("$: screenshotBusy = activeOperation === 'screenshot'");
    expect(script).toContain("$: clearingData = activeOperation === 'clearData'");
    expect(script).not.toMatch(/\bbusy\s*=\s*(true|false)\b/);
    for (const action of ['savePerformance','screenshot','clearData','launch','closeOne','navigate']) {
      expect(script).toContain("activeOperation = '"+action+"'");
    }
    const screenshot=script.slice(script.indexOf('async function saveScreenshot()'),script.indexOf('let manualRefreshRequested'));
    expect(screenshot).toContain('activeOperation !== null');
    expect(screenshot).toContain('finally{activeOperation = null;}');
  });
  it('shares settings publication while leaving FPS drafts to their existing owners', () => {
    const app=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    expect(app).toContain('if (!sameSettingsObservation(settings, incoming)) settings = incoming;');
    expect(app).toContain('observeSettings(snapshot.settings)');
    expect(app).toContain('if (!performanceDirty) {');
    expect(app).toContain('publishProfiles(snapshot.profiles);publishSettings(snapshot.settings);');
    expect(app).toContain('activeFps=snapshot.settings.activeFps;backgroundFps=snapshot.settings.backgroundFps;');
    expect(app).not.toContain('settings=snapshot.settings');
  });
});
describe('Lazy Tester profile contract', () => {
  it('compares all profile observation fields before suppressing redundant UI updates', () => {
    const base = parseSession({...stopped(), profiles: [running(1), ...stopped().profiles.slice(1)]}).profiles;
    const copy = () => base.map(profile => ({...profile, process: profile.process && {...profile.process}}));
    expect(sameProfileObservation(base, copy())).toBe(true);
    expect(sameProfileObservation(base, [])).toBe(false);
    expect(sameProfileObservation(base, [...base].reverse())).toBe(false);
    for (const change of [
      {state: 'NOT_RESPONDING' as const}, {issue: 'Native issue'}, {process: null},
    ]) {
      const next = copy();
      Object.assign(next[0]!, change);
      expect(sameProfileObservation(base, next)).toBe(false);
    }
    for (const change of [
      {pid: 999}, {createdFileTime: '999'}, {sessionId: 2},
      {windowHandle: '0x99'}, {responding: false},
    ]) {
      const next = copy();
      Object.assign(next[0]!.process!, change);
      expect(sameProfileObservation(base, next)).toBe(false);
    }
    const next = copy();
    next[1]!.process = {...base[0]!.process!};
    expect(sameProfileObservation(base, next)).toBe(false);
  });
  it('compares layout identity directly while keeping issue-only changes in feedback', () => {
    const base = parseSession({...stopped(), profiles: [running(1), ...stopped().profiles.slice(1)]}).profiles;
    const copy = () => base.map(profile => ({...profile, process: profile.process && {...profile.process}}));
    expect(sameProfileLayout(base, copy())).toBe(true);
    expect(sameProfileLayout(base, [])).toBe(false);
    expect(sameProfileLayout(base, [...base].reverse())).toBe(false);
    const issueOnly = copy();
    issueOnly[0]!.issue = 'Native issue';
    expect(sameProfileLayout(base, issueOnly)).toBe(true);
    expect(sameProfileObservation(base, issueOnly)).toBe(false);
    for (const change of [{state: 'NOT_RESPONDING' as const}, {process: null}]) {
      const next = copy();
      Object.assign(next[0]!, change);
      expect(sameProfileLayout(base, next)).toBe(false);
    }
    for (const change of [{pid: 999}, {createdFileTime: '999'}, {sessionId: 2},
      {windowHandle: '0x99'}, {responding: false}]) {
      const next = copy();
      Object.assign(next[0]!.process!, change);
      expect(sameProfileLayout(base, next)).toBe(false);
    }
  });
  it('preserves changed settings receipts, optional fields and ordered presets', () => {
    const base = parseSession({...stopped(), settings: {
      activeFps:144, backgroundFps:10, renderDistanceChunks:12, canApplyPreset:true,
      changedProfiles:null, backgroundAudio:'Keep', graphicsPresets:['Low','High','Low','Low','Low'],
    }}).settings!;
    expect(sameSettingsObservation(null, base)).toBe(false);
    expect(sameSettingsObservation(base, {...base, graphicsPresets:[...base.graphicsPresets!]})).toBe(true);
    for (const change of [
      {activeFps:120}, {backgroundFps:5}, {renderDistanceChunks:8},
      {canApplyPreset:false}, {changedProfiles:0}, {backgroundAudio:'Mute' as const},
      {audioIssue:'Native audio unavailable'}, {graphicsPresets:undefined},
      {graphicsPresets:[...base.graphicsPresets!].reverse()},
    ]) expect(sameSettingsObservation(base, {...base, ...change})).toBe(false);
  });
  it('never suppresses changed agent timestamps, staleness or missing reports', () => {
    const base = {
      schema:1 as const, source:'CURRENT_MCP_PROCESS_MEMORY_ONLY' as const,
      snapshotAt:'2026-01-01T00:00:00.000Z', displayHint:'ATTENTION' as const,
      desktopControl:'NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER' as const,
      minecraftInputEffect:'NOT_VERIFIED' as const, mapGameplayAcceptance:'NOT_TESTED' as const,
      items:[{kind:'AI_INPUT' as const, slot:1 as const, status:'NATIVE_RELEASE_UNCONFIRMED',
        importance:'ATTENTION' as const, evidence:'LOCAL_MCP_RECOVERY_HOLD'}], stale:false,
    };
    expect(sameAgentObservation(null, null)).toBe(true);
    expect(sameAgentObservation(base, structuredClone(base))).toBe(true);
    expect(sameAgentObservation(base, null)).toBe(false);
    expect(sameAgentObservation(base, {...base, stale:true})).toBe(false);
    expect(sameAgentObservation(base, {...base, snapshotAt:'2026-01-01T00:00:01.000Z'})).toBe(false);
    expect(sameAgentObservation(base, {...base, items:[]})).toBe(false);
  });
  it('rejects reversed native inventory rather than trusting unverified order',()=>{
    const snapshot=stopped();
    expect(()=>parseSession({...snapshot,profiles:[...snapshot.profiles].reverse()}))
      .toThrow('Invalid profile response');
  });
  it('accepts only compiled-backend support, never user agent or gameplay guesses',()=>{
    expect(parseNativeBackendSupport('WindowsNative')).toBe('WindowsNative');
    expect(parseNativeBackendSupport('Unsupported')).toBe('Unsupported');
    for (const forged of [true,false,null,undefined,0,'macos','MacNative','Ready',{},['WindowsNative']]) {
      expect(()=>parseNativeBackendSupport(forged)).toThrow('Invalid native backend capability');
    }
  });
  it('accepts one exact startup capability report and rejects capture in unsupported or diagnostic mode',()=>{
    const normal={backend:'WindowsNative',diagnostic:false,screenshot:false,recording:false};
    expect(parseStartupCapabilities(normal)).toEqual(normal);
    expect(parseStartupCapabilities({...normal,screenshot:true,recording:true})).toEqual({...normal,screenshot:true,recording:true});
    expect(parseStartupCapabilities({...normal,diagnostic:true}).diagnostic).toBe(true);
    expect(parseStartupCapabilities({...normal,backend:'Unsupported'}).backend).toBe('Unsupported');
    for(const value of ['WindowsNative',null,{...normal,recording:1},{...normal,extra:true},
      {...normal,diagnostic:true,screenshot:true},{...normal,backend:'Unsupported',recording:true},
      {...normal,backend:'Unsupported',diagnostic:true}])expect(()=>parseStartupCapabilities(value)).toThrow();
  });
  it('schedules 10s idle, 5s settled live Play and 2s recovery deadlines', () => {
    for (const page of ['settings', 'storage'] as const) {
      expect(periodicInspectionInterval(page, 0, true)).toBe(10_000);
      expect(periodicInspectionInterval(page, 1, true)).toBe(2000);
      expect(periodicInspectionInterval(page, 0, false)).toBe(2000);
    }
    expect(periodicInspectionInterval('workspace', 0, true)).toBe(2000);
    expect(periodicInspectionInterval('workspace', 0, true, true)).toBe(10_000);
    expect(periodicInspectionInterval('workspace', 1, true, true)).toBe(2000);
    expect(periodicInspectionInterval('workspace', 0, false, true)).toBe(2000);
    expect(periodicInspectionInterval('workspace', 1, true, false, true)).toBe(5000);
    expect(periodicInspectionInterval('workspace', 5, true, false, true)).toBe(5000);
    expect(periodicInspectionInterval('workspace', 1, false, false, true)).toBe(2000);
    expect(periodicInspectionInterval('settings', 1, true, false, true)).toBe(2000);
    expect(periodicInspectionInterval('storage', 1, true, false, true)).toBe(2000);
  });
  it('coalesces hints against the last read start, not a new timer or deadline',()=>{
    const started=1200;
    expect(sessionInspectionDeadline(started,5000,false)).toBe(6200);
    expect(sessionInspectionDeadline(started,5000,true)).toBe(3200);
    expect(sessionInspectionDeadline(started,10000,true)).toBe(3200);
    expect(sessionInspectionDeadline(started,2000,true)).toBe(3200);
    expect(sessionInspectionDeadline(Number.NEGATIVE_INFINITY,5000,true)).toBe(Number.NEGATIVE_INFINITY);
  });
  it('uses one earliest deadline without accelerating the other lane', () => {
    const start=1200;
    const session=start+periodicInspectionInterval('workspace',0,true,true);
    expect(session).toBe(11200);
    expect(nextBackgroundDeadline(session,3200)).toBe(3200);
    expect(nextBackgroundDeadline(session,5200)).toBe(5200);
    expect(nextBackgroundDeadline(session,null)).toBe(11200);
    expect(nextBackgroundDeadline(null,3200)).toBe(3200);
    expect(nextBackgroundDeadline(null,null)).toBeNull();
    expect(nextBackgroundDeadline(11200,13200)).toBe(11200);
    expect(nextBackgroundDeadline(Number.NEGATIVE_INFINITY,null)).toBe(Number.NEGATIVE_INFINITY);
  });
  it('shares the observation clocks, pauses blocked lanes, and cancels the one timer on teardown', () => {
    const mounted = readFileSync(new URL('../src/LazyTesterApp.svelte', import.meta.url), 'utf8');
    expect(mounted).toMatch(/async function refresh\(passiveIdle = false\)[\s\S]*?refreshing = true;\s*lastInspectionStartedAt = performance\.now\(\);/);
    expect(mounted).toContain('sessionInspectionDeadline(lastInspectionStartedAt,');
    expect(mounted).toContain('periodicInspectionInterval(page, runningCount, available, workspaceSettledIdle, workspaceSettledLive)');
    expect(mounted).toContain("selectedIdentity !== '' && selectedIdentity === identity(profiles.find(profile => profile.slot === active))");
    expect(mounted).toContain("profile.slot === active || previewStatuses.some(status => status.slot === profile.slot && status.registered)");
    expect(mounted).toContain('lastInspectionStartedAt = performance.now();\n    sessionObservationHint = false;');
    expect(mounted).toContain('if (visibilityChanged && visible) sessionObservationHint = true;');
    expect(mounted).toContain("window.removeEventListener('focus', returnedFocus);");
    const invalidation=mounted.split("void listen<string>('workspace-surface-error'")[1]!.split("void listen<number>")[0]!;
    expect(invalidation.indexOf('!dwmPreviewInvalidationApplies')).toBeLessThan(invalidation.indexOf('sessionObservationHint=true;'));
    expect(invalidation).not.toContain('void refresh()');
    expect(mounted).toContain('lastRecordingInspectionStartedAt + 2000');
    expect(mounted).toContain('sessionObservationEnabled = mounted && !disposed && managerVisible');
    expect(mounted).toContain('recordingObservationEnabled = mounted && !disposed && managerVisible');
    expect(mounted).toContain('!closeReviewPending && !!binding?.configured');
    expect(mounted).toContain('!selectiveOpen && storageReview===null');
    expect(mounted).toContain('recordingAvailable && recordingWatch && !recordingStatusPending');
    expect(mounted).toContain('const due = nextBackgroundDeadline(sessionDue, recordingDue)');
    expect(mounted).toContain('if (due === null || disposed) return;');
    expect(mounted).toContain('if (observationTimer !== timer || disposed) return;');
    expect(mounted).toContain('if (document.hidden) { managerVisible = false; return; }');
    expect(mounted).toContain('sessionDue <= now && sessionObservationEnabled');
    expect(mounted).toContain('recordingDue <= now && recordingObservationEnabled');
    expect(mounted).toContain('managerVisible = !document.hidden;');
    expect(mounted).toContain('disposed = true; clearBackgroundObservation();');
    expect(mounted.match(/window\.setTimeout\(/g)).toHaveLength(1);
    expect(mounted).not.toContain('setInterval(');
    expect(mounted).not.toContain('lastAutomaticInspectionAt');
  });
  it('suspends automatic inspection through every close review and drops stale results', () => {
    const idle: CloseReview = {
      nativeExitReview:false, closeAllRunning:false,
      selectedReview:false, selectedCloseRunning:false,
    };
    expect(closeReviewActive(idle)).toBe(false);
    expect(observationStillCurrent(12,12,idle)).toBe(true);
    expect(observationStillCurrent(12,13,idle)).toBe(false);
    for (const phase of Object.keys(idle) as (keyof CloseReview)[]) {
      const review = {...idle,[phase]:true};
      expect(closeReviewActive(review)).toBe(true);
      expect(observationStillCurrent(12,12,review)).toBe(false);
    }
    // Canceling the dialog does not resurrect the old in-flight inspection.
    expect(observationStillCurrent(12,13,idle)).toBe(false);
  });
  it('retires only the released selection despite a newer close-review epoch',()=>{
    // The native Release already succeeded. An unrelated close-review must
    // not resurrect that old embedded HWND; a replacement remains untouched.
    const old={slot:2,identity:'102/134000000000000002/1/0x22'};
    expect(releasedSelectionStillCurrent(old,{...old})).toBe(true);
    expect(releasedSelectionStillCurrent(old,{slot:2,identity:'102/134000000000000003/1/0x22'})).toBe(false);
    expect(releasedSelectionStillCurrent(old,{slot:3,identity:old.identity})).toBe(false);
    expect(releasedSelectionStillCurrent(old,{slot:null,identity:''})).toBe(false);
    expect(releasedSelectionStillCurrent({slot:null,identity:''},{slot:null,identity:''})).toBe(false);
    const source=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const begin=source.indexOf('const released = {slot: active, identity: selectedIdentity};');
    const cleanup=source.indexOf('releasedSelectionStillCurrent(released,{slot: active, identity: selectedIdentity})',begin);
    const invalidation=source.indexOf('if (!observationStillCurrent(issued, observationEpoch, reviewState())) return;',cleanup);
    expect(begin).toBeGreaterThanOrEqual(0);
    expect(cleanup).toBeGreaterThan(begin);
    expect(invalidation).toBeGreaterThan(cleanup);
  });
  it('reconciles invalidated geometry when selected close review is cancelled',()=>{
    const source=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const cancel=source.slice(source.indexOf('function cancelCloseOne()'),source.indexOf('function closeReviewKeydown('));
    expect(cancel).toContain('closeOneSlot=null;closeOneTrigger=null;');
    expect(cancel).toContain('dirty=true;');
  });
  it('drops late preview layout observations on minimize, source swap or close review',()=>{
    const idle: CloseReview = {
      nativeExitReview:false,closeAllRunning:false,selectedReview:false,selectedCloseRunning:false,
    };
    const profiles=parseSession({...stopped(),profiles:[running(1),running(2),...stopped().profiles.slice(2)]}).profiles;
    const issued={observationEpoch:5,profiles,primarySlot:1};
    const current={...issued,dirty:false,managerVisible:true,workspaceAvailable:true};
    const nextProfiles=profiles.map(profile=>profile.slot===2
      ? {...profile,process:{...profile.process!,pid:999}} : profile);
    const issueOnly=profiles.map(profile=>profile.slot===2
      ? {...profile,issue:'Native issue'} : profile);
    expect(previewLayoutStillCurrent(issued,{...current,profiles:issueOnly},idle)).toBe(true);
    expect(previewLayoutStillCurrent(issued,current,idle)).toBe(true);
    for(const changed of [
      {...current,observationEpoch:6},
      {...current,profiles:nextProfiles},
      {...current,primarySlot:2},
      {...current,dirty:true},
      {...current,managerVisible:false},
      {...current,workspaceAvailable:false},
    ])expect(previewLayoutStillCurrent(issued,changed,idle)).toBe(false);
    for(const phase of Object.keys(idle) as (keyof CloseReview)[]) {
      expect(previewLayoutStillCurrent(issued,current,{...idle,[phase]:true})).toBe(false);
    }
    // A merely unfocused but still-visible manager retains DWM observation.
    expect(previewLayoutStillCurrent(issued,{...current,managerVisible:true},idle)).toBe(true);
  });
  it('validates optional background audio support and reports native failures',()=>{
    const settings={activeFps:120,backgroundFps:5,renderDistanceChunks:12,canApplyPreset:true,changedProfiles:null};
    expect(parseSession({...stopped(),settings}).settings?.backgroundAudio).toBeUndefined();
    for(const backgroundAudio of ['Keep','Mute'])expect(parseSession({...stopped(),settings:{...settings,backgroundAudio}}).settings?.backgroundAudio).toBe(backgroundAudio);
    expect(parseSession({...stopped(),settings:{...settings,backgroundAudio:'Mute',audioIssue:'Device disconnected'}}).settings?.audioIssue).toBe('Device disconnected');
    for(const invalid of [{backgroundAudio:'Other'},{backgroundAudio:null},{audioIssue:'unsupported'},{backgroundAudio:'Mute',audioIssue:7}])expect(()=>parseSession({...stopped(),settings:{...settings,...invalid}})).toThrow();
  });
  it('validates full-clear identity confirmation and refuses active or inconsistent targets',()=>{
    const fullClear={canClear:true,pending:false,confirmationToken:'a'.repeat(64),issue:null};
    const storage=[1,2,3,4,5].map(slot=>({slot,bytes:100,files:2,partial:false,disk:null,
      cache:{bytes:0,files:0,partial:false,canClear:false,issue:null},fullClear}));
    const snapshot={...stopped(),storage};
    expect(parseSession(snapshot).storage![3]!.fullClear?.canClear).toBe(true);
    for(const invalid of [{...fullClear,confirmationToken:null},{...fullClear,confirmationToken:'../outside'},
      {...fullClear,issue:'Locked'},{...fullClear,pending:'yes'},{...fullClear,path:'outside'},null]) {
      expect(()=>parseSession({...snapshot,storage:[{...storage[0]!,fullClear:invalid},...storage.slice(1)]})).toThrow();
    }
    const active={...snapshot,profiles:[running(1),...snapshot.profiles.slice(1)]};
    expect(()=>parseSession(active)).toThrow();
    const gated={...active,storage:storage.map(item=>item.slot===1?{...item,fullClear:{...fullClear,canClear:false,confirmationToken:null,issue:'Close this Minecraft'}}:item)};
    expect(parseSession(gated).storage![0]!.fullClear?.confirmationToken).toBeNull();
    expect(parseSession(gated).storage![3]!.fullClear?.canClear).toBe(true);
    expect(parseSession({...snapshot,storage:storage.map(item=>({...item,fullClear:{...fullClear,pending:true}}))}).storage![0]!.fullClear?.pending).toBe(true);
  });
  it('admits settings recovery only for its stopped launcher with consistent backup metadata',()=>{
    const gameSettings={hasSettings:true,backupAvailable:true,canReset:true,canRestore:true,issue:null};
    const storage=[1,2,3,4,5].map(slot=>({slot,bytes:100,files:2,partial:false,disk:null,
      cache:{bytes:0,files:0,partial:false,canClear:false,issue:null},gameSettings}));
    const snapshot={...stopped(),storage};
    expect(parseSession(snapshot).storage![3]!.gameSettings?.canRestore).toBe(true);
    const withEnabled={...snapshot,storage:storage.map(item=>({...item,gameSettings:{...gameSettings,improvedInputResponseEnabled:true}}))};
    expect(parseSession(withEnabled).storage![0]!.gameSettings?.improvedInputResponseEnabled).toBe(true);
    const withDisabled={...snapshot,storage:storage.map(item=>({...item,gameSettings:{...gameSettings,improvedInputResponseEnabled:false}}))};
    expect(parseSession(withDisabled).storage![0]!.gameSettings?.improvedInputResponseEnabled).toBe(false);
    expect(parseSession(snapshot).storage![0]!.gameSettings?.improvedInputResponseEnabled).toBeUndefined();
    const textureOff={...snapshot,storage:storage.map(item=>({...item,gameSettings:{...gameSettings,textureStreamingEnabled:false}}))};
    const textureOn={...snapshot,storage:storage.map(item=>({...item,gameSettings:{...gameSettings,textureStreamingEnabled:true}}))};
    expect(parseSession(textureOff).storage![0]!.gameSettings?.textureStreamingEnabled).toBe(false);
    expect(parseSession(textureOn).storage![0]!.gameSettings?.textureStreamingEnabled).toBe(true);
    expect(parseSession(snapshot).storage![0]!.gameSettings?.textureStreamingEnabled).toBeUndefined();
    for(const invalid of [{...gameSettings,hasSettings:false},{...gameSettings,backupAvailable:false},
      {...gameSettings,issue:'Locked'},{...gameSettings,canReset:'yes'},{...gameSettings,path:'../outside'},
      {...gameSettings,improvedInputResponseEnabled:'yes'},
      {...gameSettings,improvedInputResponseEnabled:null},
      {...gameSettings,textureStreamingEnabled:null},
      {...gameSettings,textureStreamingEnabled:'yes'},
      {...gameSettings,hasSettings:false,canReset:false,textureStreamingEnabled:true},
      {...gameSettings,canReset:false,canRestore:false,issue:'Profile running',textureStreamingEnabled:false},
      {...gameSettings,hasSettings:false,canReset:false,improvedInputResponseEnabled:true},
      {...gameSettings,canReset:false,canRestore:false,issue:'Profile running',improvedInputResponseEnabled:true},
      null]) {
      expect(()=>parseSession({...snapshot,storage:[{...storage[0]!,gameSettings:invalid},...storage.slice(1)]})).toThrow();
    }
    const active={...snapshot,profiles:[running(1),...snapshot.profiles.slice(1)]};
    expect(()=>parseSession(active)).toThrow();
    const gated={...active,storage:storage.map(item=>item.slot===1?{...item,gameSettings:{...gameSettings,canReset:false,canRestore:false,issue:'Close this Minecraft'}}:item)};
    expect(parseSession(gated).storage![3]!.gameSettings?.canReset).toBe(true);
    expect(parseSession(gated).storage![0]!.gameSettings?.canRestore).toBe(false);
  });
  it('accepts optional bounded read-only resource pack inventory without claiming activation',()=>{
    const resourcePacks={manifested:2,partial:false,issue:null};
    const row=(status:unknown)=>[1,2,3,4,5].map(slot=>({
      slot,bytes:100,files:2,partial:false,disk:null,
      cache:{bytes:0,files:0,partial:false,canClear:false,issue:null},
      resourcePacks:status,
    }));
    const profile={...stopped(),storage:row(resourcePacks)};
    expect(parseSession(profile).storage![0]?.resourcePacks?.manifested).toBe(2);
    const declarations={worldsScanned:4,worldsWithReferences:2,packReferences:3,partial:false,issue:null};
    const withWorlds={...resourcePacks,worldDeclarations:declarations};
    expect(parseSession({...stopped(),storage:row(withWorlds)}).storage![0]?.resourcePacks?.worldDeclarations).toEqual(declarations);
    // Source-inspection uncertainty is not an authoritative zero-reference claim.
    const uncertain={worldsScanned:0,worldsWithReferences:0,packReferences:0,
      partial:true,issue:'World metadata not readable'};
    expect(parseSession({...stopped(),storage:row({...resourcePacks,worldDeclarations:uncertain})})
      .storage![0]?.resourcePacks?.worldDeclarations?.partial).toBe(true);
    for(const invalidWorlds of [
      {...declarations,worldsScanned:129}, {...declarations,worldsScanned:-1},
      {...declarations,packReferences:257}, {...declarations,packReferences:0},
      {...declarations,worldsWithReferences:5}, {...declarations,worldsScanned:1},
      {...declarations,partial:true}, {...declarations,issue:'Mismatch'},
      {...uncertain,packReferences:1}, {...declarations,activeInGame:true}, null,
    ]) expect(()=>parseSession({...stopped(),storage:row({...resourcePacks,worldDeclarations:invalidWorlds})})).toThrow();
    const partial={manifested:0,partial:true,issue:'Resource pack inventory is partial'};
    expect(parseSession({...stopped(),storage:row(partial)}).storage![0]?.resourcePacks?.partial).toBe(true);
    const older={...stopped(),storage:row(resourcePacks).map(({resourcePacks,...rest})=>rest)};
    expect(parseSession(older).storage![0]?.resourcePacks).toBeUndefined();
    for(const bad of [
      {...resourcePacks,manifested:-1}, {...resourcePacks,manifested:129},
      {...resourcePacks,manifested:1.5}, {...resourcePacks,partial:'yes'},
      {...resourcePacks,issue:'Unexpected warning'},
      {manifested:1,partial:true,issue:'Uncertain'},
      {manifested:0,partial:true,issue:null},
      {...resourcePacks,activeInWorld:true},null,
    ]) expect(()=>parseSession({...stopped(),storage:row(bad)})).toThrow();
  });
  it('excludes unresolved or closing profiles from automatic selection',()=>{
    const snapshot=stopped();
    for(const slot of [1,2])Object.assign(snapshot.profiles[slot-1]!,running(slot));
    const profiles=parseSession(snapshot).profiles;
    expect(firstEmbeddedProfile(profiles,new Set([1]))).toBe(2);
    expect(firstEmbeddedProfile(profiles,new Set([1,2]))).toBeNull();
    expect(firstEmbeddedProfile(profiles,new Set([2]))).toBe(1);
    expect(profiles.filter(profile=>profile.state==='RUNNING')).toHaveLength(2);
  });
  it('accepts Preserve Visuals and legacy five-profile High/Low preferences, rejects invented states',()=>{
    const settings={activeFps:144,backgroundFps:10,renderDistanceChunks:12,canApplyPreset:true,changedProfiles:null,graphicsPresets:['Preserve','Low','High','Preserve','Low']};
    expect(parseSession({...stopped(),settings}).settings!.graphicsPresets).toEqual(settings.graphicsPresets);
    expect(parseSession({...stopped(),settings:{...settings,graphicsPresets:['High','Low','High','Low','Low']}}).settings!.graphicsPresets).toEqual(['High','Low','High','Low','Low']);
    for(const graphicsPresets of [['High'],['High','Low','Low','Low','Auto'],null])expect(()=>parseSession({...stopped(),settings:{...settings,graphicsPresets}})).toThrow();
  });
  it('validates configured Settings policy and refuses a successful preset update while games are active',()=>{
    const settings={activeFps:144,backgroundFps:10,renderDistanceChunks:12,canApplyPreset:true,changedProfiles:null};
    expect(parseSession({...stopped(),settings}).settings).toEqual(settings);
    expect(parseSession({...stopped(),settings:{...settings,changedProfiles:0}}).settings!.changedProfiles).toBe(0);
    expect(parseSession({...stopped(),settings:{...settings,changedProfiles:5}}).settings!.changedProfiles).toBe(5);
    expect(parseSession({...stopped(),settings:{...settings,activeFps:60,backgroundFps:5}}).settings!.activeFps).toBe(60);
    expect(parseSession({...stopped(),settings:{...settings,activeFps:120,backgroundFps:5}}).settings!.backgroundFps).toBe(5);
    // Previously saved preferences remain readable, but cannot be newly saved below 5 FPS.
    expect(parseSession({...stopped(),settings:{...settings,backgroundFps:1}}).settings!.backgroundFps).toBe(1);
    for(const invalid of [{...settings,activeFps:0},{...settings,activeFps:1001},{...settings,backgroundFps:145},
      {...settings,renderDistanceChunks:8},{...settings,changedProfiles:-1},{...settings,changedProfiles:6},
      {...settings,changedProfiles:0.5},{...settings,measuredFps:144},
      {...settings,canApplyPreset:false,changedProfiles:1}])expect(()=>parseSession({...stopped(),settings:invalid})).toThrow();
    const active={...stopped(),profiles:[running(1),...stopped().profiles.slice(1)],settings};
    expect(()=>parseSession(active)).toThrow();
    expect(parseSession({...active,settings:{...settings,canApplyPreset:false}}).settings!.canApplyPreset).toBe(false);
  });
  it('accepts bounded custom FPS pairs and rejects missing, fractional and inverted limits',()=>{
    for(const [active,background] of [[120,5],[144,10],[60,5],[5,5],[1000,1000]])expect(validFpsLimits(active,background)).toBe(true);
    for(const [active,background] of [[1,1],[4,4],[120,4],[0,1],[1001,10],[60,61],[60,0],[60.5,5],[NaN,5],[undefined,5],['60',5]])expect(validFpsLimits(active,background)).toBe(false);
  });
  it('validates persisted data locations without inferring account readiness',()=>{
    const value={schema:1,defaultPath:'D:\\Lazy Tester\\Data',path:'D:\\Lazy Tester\\Data',canLocate:true,
      profiles:[1,2,3,4,5].map(slot=>({slot,path:`D:\\Lazy Tester\\Data\\Profiles\\Minecraft-${slot}`}))};
    expect(parseDataLocation(value).profiles).toHaveLength(5);
    expect(parseDataLocation({...value,path:null,profiles:[]}).path).toBeNull();
    for(const invalid of [{...value,path:null},{...value,profiles:value.profiles.slice(1)},
      {...value,defaultPath:''},{...value,authenticated:true},{...value,path:'bad\npath'}])expect(()=>parseDataLocation(invalid)).toThrow();
  });
  it('keeps missing or invalid binding separate from launch/game/account readiness', () => {
    expect(parseProfileBinding({schema:1,required:true,configured:false,issue:'Select existing binding'}).configured).toBe(false);
    expect(parseProfileBinding({schema:1,required:true,configured:true,issue:null}).configured).toBe(true);
    expect(parseProfileBinding({schema:1,required:false,configured:true,issue:null}).required).toBe(false);
    for(const value of [
      {schema:1,required:true,configured:true,issue:'Invalid'},
      {schema:1,required:false,configured:false,issue:null},
      {schema:1,required:true,configured:true,issue:null,authenticated:true},
      {schema:2,required:true,configured:false,issue:null},
    ]) expect(()=>parseProfileBinding(value)).toThrow();
  });
  it('validates quota-aware disk space and refuses unsafe cache cleanup status', () => {
    const storage = [1,2,3,4,5].map(slot => ({slot, bytes: 100, files: 2, partial: false,
      disk: {availableBytes: 20, totalBytes: 1000},
      cache: {bytes: 50, files: 1, partial: false, canClear: true, issue: null}}));
    const snapshot = {...stopped(), storage};
    expect(parseSession(snapshot).storage![4]!.cache.canClear).toBe(true);
    expect(parseSession({...snapshot, storage: storage.map(item => ({...item, disk: null}))}).storage![0]!.disk).toBeNull();
    const invalid = [
      {...storage[0]!, disk: {availableBytes: 1001, totalBytes: 1000}},
      {...storage[0]!, disk: {availableBytes: -1, totalBytes: 1000}},
      {...storage[0]!, cache: {...storage[0]!.cache, partial: true}},
      {...storage[0]!, cache: {...storage[0]!.cache, issue: 'Locked'}},
      {...storage[0]!, cache: {...storage[0]!.cache, files: 0}},
      {...storage[0]!, cache: {...storage[0]!.cache, files: 2049}},
      {...storage[0]!, cache: {...storage[0]!.cache, path: 'untrusted'}},
    ];
    for (const item of invalid) expect(() => parseSession({...snapshot, storage: [item, ...storage.slice(1)]})).toThrow();
    const active = {...snapshot, profiles: snapshot.profiles.map(profile => profile.slot === 2 ? running(2) : profile)};
    expect(() => parseSession(active)).toThrow();
    const scoped = {...active, storage: storage.map(item => item.slot === 2 ?
      {...item, cache: {...item.cache, canClear:false, issue:'Close this Minecraft'}} : item)};
    expect(parseSession(scoped).storage![3]!.cache.canClear).toBe(true);
    expect(parseSession(scoped).storage![1]!.cache.canClear).toBe(false);
    expect(parseSession({...active, storage: storage.map(item => ({...item, cache: {...item.cache, canClear: false, issue: 'Close Minecraft'}}))}).storage).toHaveLength(5);
  });
  it('keeps pending full clear exclusive with both cache actions for that launcher',()=>{
    const eligible={bytes:50,files:1,partial:false,canClear:true,issue:null};
    const blocked={...eligible,canClear:false,issue:'Finish pending profile cleanup first'};
    const fullClear={canClear:true,pending:true,confirmationToken:'a'.repeat(64),issue:null};
    const storage=[1,2,3,4,5].map(slot=>({slot,bytes:100,files:2,partial:false,disk:null,
      cache:eligible,downloadCache:eligible,fullClear:{...fullClear,pending:false}}));
    const pending={...storage[0]!,fullClear,cache:blocked,downloadCache:blocked};
    const snapshot={...stopped(),storage:[pending,...storage.slice(1)]};
    const report=parseSession(snapshot).storage!;
    expect(report[0]!.fullClear?.pending).toBe(true);
    expect(report[0]!.cache.canClear).toBe(false);
    expect(report[0]!.downloadCache?.canClear).toBe(false);
    expect(report[1]!.cache.canClear).toBe(true);
    expect(report[1]!.downloadCache?.canClear).toBe(true);
    for(const invalid of [{...pending,cache:eligible},{...pending,downloadCache:eligible}]) {
      expect(()=>parseSession({...snapshot,storage:[invalid,...storage.slice(1)]})).toThrow(/Pending full clear/);
    }
    // Legacy reports may omit download cache without granting new admission.
    const {downloadCache: _omitted,...legacy}=pending;
    expect(parseSession({...snapshot,storage:[legacy,...storage.slice(1)]}).storage![0]!.downloadCache).toBeUndefined();
  });
  it('validates exactly the requested preview slots and keeps unavailable previews explicit',()=>{
    const report={schema:1,tiles:[{slot:2,registered:true,reason:null},{slot:5,registered:false,reason:'Window not available'}]};
    expect(parsePreviewReport(report,[2,5])).toEqual(report);
    expect(()=>parsePreviewReport(report,[2,3])).toThrow();
    expect(()=>parsePreviewReport({...report,tiles:[report.tiles[0],report.tiles[0]]},[2,5])).toThrow();
    expect(()=>parsePreviewReport({schema:1,tiles:[{slot:2,registered:true,reason:'failed'}]},[2])).toThrow();
    expect(()=>parsePreviewReport({schema:1,tiles:[{slot:2,registered:true,reason:null,path:'other'}]},[2])).toThrow();
  });
  it('scopes asynchronous DWM refresh invalidation to the exact original process',()=>{
    const process=running(2).process;
    const event='dwm-preview-invalidated:'+JSON.stringify({schema:1,slot:2,process,reason:'Native preview refresh unverified'});
    const notice=parseDwmPreviewInvalidation(event);
    expect(notice).not.toBeNull();
    expect(dwmPreviewInvalidationApplies(notice!,running(2) as ProfileSession)).toBe(true);
    expect(dwmPreviewInvalidationApplies(notice!,running(3) as ProfileSession)).toBe(false);
    const replacement={...running(2),process:{...process!,createdFileTime:'301'}};
    expect(dwmPreviewInvalidationApplies(notice!,replacement as ProfileSession)).toBe(false);
    const stoppedProfile={...running(2),state:'STOPPED'};
    expect(dwmPreviewInvalidationApplies(notice!,stoppedProfile as ProfileSession)).toBe(false);
    expect(parseDwmPreviewInvalidation('Normal workspace failure')).toBeNull();
    for(const malformed of [
      'dwm-preview-invalidated:bad-json',
      'dwm-preview-invalidated:'+JSON.stringify({schema:1,slot:0,process,reason:'bad'}),
      'dwm-preview-invalidated:'+JSON.stringify({schema:1,slot:2,process,reason:''}),
      'dwm-preview-invalidated:'+JSON.stringify({schema:1,slot:2,process:{...process,pid:0},reason:'bad'}),
      'dwm-preview-invalidated:'+JSON.stringify({schema:1,slot:2,process,reason:'bad',success:true}),
    ])expect(()=>parseDwmPreviewInvalidation(malformed)).toThrow();
  });
  it('allows small preview viewports but refuses offscreen and invalid coordinate payloads',()=>{
    expect(previewViewport({x:10,y:450,width:145,height:80},1.5)).toEqual({x:10,y:450,width:145,height:80,pixelRatio:1.5});
    for(const rect of [{x:-1,y:450,width:145,height:80},{x:10,y:450,width:31,height:80},{x:10,y:450,width:145,height:NaN}])expect(()=>previewViewport(rect)).toThrow();
  });
  it('distinguishes post-restoration pending exit from preflight failures',()=>{
    const snapshot=stopped();
    expect(parseSelectedClose({schema:1,presentationReleased:true,snapshot,issue:null}).snapshot?.profiles).toHaveLength(5);
    expect(parseSelectedClose({schema:1,presentationReleased:true,snapshot:null,issue:'Minecraft - 2 still running'}).issue).toContain('still running');
    expect(parseSelectedClose({schema:1,presentationReleased:false,snapshot:null,issue:'Native window restore not confirmed'}))
      .toMatchObject({presentationReleased:false,snapshot:null});
    for(const invalid of [
      {schema:1,presentationReleased:false,snapshot,issue:null},
      {schema:1,presentationReleased:true,snapshot:null,issue:null},
      {schema:1,presentationReleased:true,snapshot,issue:'contradictory'},
      {schema:1,presentationReleased:false,snapshot,issue:null},
      {schema:1,presentationReleased:false,snapshot,issue:'Cannot claim success'},
      {schema:1,presentationReleased:true,snapshot:null,issue:''},
      {schema:1,presentationReleased:true,snapshot,issue:null,process:123},
    ])expect(()=>parseSelectedClose(invalid)).toThrow();
  });
  it('does not expose retired individual performance methods',()=>{
    for(const name of ['applyPreset','applyGraphics','enableTextureStreaming','disableTextureStreaming','disableImprovedInputResponse'])
      expect(Object.hasOwn(lazyTester,name)).toBe(false);
  });
  it('refuses a forged per-launcher close slot before invoking Tauri', async()=>{
    for (const slot of [0,6,-1,10] as number[]) {
      await expect(lazyTester.closeOne(slot as 1)).rejects.toThrow('Unknown Minecraft profile');
    }
  });
  it('preserves manual retry for stopped ATTENTION but blocks active or uncertain owners',()=>{
    const clean=parseSession(stopped()).profiles[0]!;
    expect(canRequestLaunch(clean)).toBe(true);
    // Native may retain a stopped slot after a nonzero game exit. A manual
    // retry is valid only if its original controller admits the attempt.
    expect(canRequestLaunch({...clean,state:'ATTENTION',issue:'Minecraft exited with code 1'})).toBe(true);
    expect(canRequestLaunch({...clean,state:'ATTENTION',issue:'Profile storage unavailable'})).toBe(true);
    expect(canRequestLaunch({...clean,state:'ATTENTION',issue:null})).toBe(true);
    for(const invalid of [
      undefined,
      {...clean,state:'STOPPED',issue:'Saved profile may be incomplete'},
      {...clean,state:'STARTING',issue:null},
      {...clean,state:'NOT_RESPONDING',issue:'Previous exit still pending'},
      {...clean,state:'RUNNING',process:running(1).process},
      {...clean,state:'STOPPED',process:running(1).process},
      {...clean,state:'ATTENTION',process:running(1).process},
    ]){
      expect(canRequestLaunch(invalid as ProfileSession|undefined)).toBe(false);
    }
  });
  it('keeps a windowless normal-exit survivor non-launchable in the existing protocol', () => {
    const original = stopped();
    const pending = {
      ...original,
      profiles: original.profiles.map(profile => profile.slot === 2
        ? {...profile, state:'NOT_RESPONDING', issue:'Normal exit pending: game process still running'}
        : profile),
    };
    const parsed = parseSession(pending);
    expect(parsed.profiles[1]).toMatchObject({
      slot:2, state:'NOT_RESPONDING', process:null,
      issue:'Normal exit pending: game process still running',
    });
    expect(parsed.profiles.filter(profile => ['STARTING','RUNNING','NOT_RESPONDING'].includes(profile.state))).toHaveLength(1);
  });

  it('keeps all five stopped profiles without deriving login or starting a game', () => {
    const snapshot = parseSession(stopped());
    expect(snapshot.profiles.map(profile => profile.slot)).toEqual([1,2,3,4,5]);
    expect(snapshot.profiles.every(profile => profile.state === 'STOPPED' && profile.process === null)).toBe(true);
  });
  it('rejects duplicate profile/process identities, forged running state and unknown fields', () => {
    const cases: unknown[] = [];
    const duplicateSlot = stopped(); duplicateSlot.profiles[4]!.slot = 1; cases.push(duplicateSlot);
    const noProcess = stopped(); noProcess.profiles[2]!.state = 'RUNNING'; cases.push(noProcess);
    const duplicatePid = stopped(); Object.assign(duplicatePid.profiles[0]!, running(1));
    Object.assign(duplicatePid.profiles[1]!, {...running(2), process: running(1).process}); cases.push(duplicatePid);
    cases.push({...stopped(), authenticated: true}, {...stopped(), schema: 2});
    for (const value of cases) expect(() => parseSession(value)).toThrow();
  });
});

describe('Lazy Tester user-facing feedback and recovery warnings',()=>{
  it('reports observed profiles without claiming a live controller or game sign-in',()=>{
    expect(sessionConnectionLabel(true,'WindowsNative',true,true)).toBe('Checking profiles…');
    expect(sessionConnectionLabel(false,'Unsupported',false,false)).toBe('Not supported');
    expect(sessionConnectionLabel(false,null,true,true)).toBe('Status unavailable');
    expect(sessionConnectionLabel(false,'WindowsNative',false,true)).toBe('Profiles unavailable');
    expect(sessionConnectionLabel(false,'WindowsNative',true,false)).toBe('Session status unavailable');
    expect(sessionConnectionLabel(false,'WindowsNative',true,true)).toBe('Profile data verified');
  });

  it('uses only authored context messages, never untrusted backend paths or prompt text',()=>{
    const hostile='C:\\Users\\private\\worlds [internal policy] PID 1234';
    const contexts:UserMessageKey[]=['backend','binding','connection','launch','selection',
      'workspace','location','storage','settings','performance','audio','close',
      'recovery','closeAll','preview','profile','pack','worldPack','cache','clearData',
      'gameSettings','folder','event','general','graphicsBlocked','performanceBlocked',
      'windowControl','windowStatus','windowClose','closeIncomplete','closeRejected'];
    for(const context of contexts){
      const message=userMessage(context);
      expect(message.length).toBeGreaterThan(12);
      expect(message).not.toContain(hostile);
      expect(message).not.toContain('C:\\');
      expect(message).not.toMatch(/PID|HWND|stack trace|IPC|internal policy/i);
    }
  });

  it('assigns unique, authored support references only from explicit operation keys',()=>{
    const keys: UserMessageKey[]=[
      'general','backend','binding','connection','launch','selection','workspace',
      'location','storage','settings','performance','audio','close','recovery',
      'closeAll','preview','profile','pack','worldPack','cache','clearData',
      'gameSettings','folder','event','graphicsBlocked','performanceBlocked',
      'windowControl','windowStatus','windowClose','closeIncomplete','closeRejected'
    ];
    const refs=keys.map(key=>userFeedback(key));
    expect(new Set(refs.map(ref=>ref.code)).size).toBe(keys.length);
    for(let i=0;i<keys.length;i++){
      const ref=refs[i]!;
      expect(ref.message).toBe(userMessage(keys[i]!));
      expect(ref.code).toMatch(/^LT-(APP|CON|PRO|GAM|DIS|SET|STO)-[0-9]{2}$/);
      expect(['Application','Connection','Launcher','Game','Display','Settings','Storage']).toContain(ref.area);
      expect(JSON.stringify(ref)).not.toMatch(/C:\\\\|PID|HWND|stack trace|internal policy/i);
    }
    expect(userFeedback('connection')).toMatchObject({area:'Connection',code:'LT-CON-01'});
    expect(userFeedback('graphicsBlocked').message).toBe(
      'Close all Minecraft sessions, then Refresh before saving graphics.');
    expect(userFeedback('performanceBlocked').message).toBe(
      'Refresh before saving FPS limits.');
  });

  it('uses specific authored error titles without assuming a failed action retried',()=>{
    expect(feedbackHeading('binding')).toBe('Launcher profiles not verified');
    expect(feedbackHeading('launch')).toBe('Game launch not confirmed');
    expect(feedbackHeading('preview')).toBe('Game display needs attention');
    expect(feedbackHeading('clearData')).toBe('Data cleanup not confirmed');
    expect(feedbackHeading('storage')).toBe('Storage information incomplete');
    expect(feedbackHeading('windowClose')).toBe('App exit not confirmed');
    const keys:UserMessageKey[]=['general','backend','binding','connection','launch',
      'selection','workspace','location','storage','settings','performance','audio',
      'close','recovery','closeAll','preview','profile','pack','worldPack','cache',
      'clearData','gameSettings','folder','event','graphicsBlocked',
      'performanceBlocked','windowControl','windowStatus','windowClose',
      'closeIncomplete','closeRejected'];
    for(const key of keys){
      const title=feedbackHeading(key);
      expect(title.length).toBeGreaterThan(10);
      expect(title).not.toMatch(/PID|HWND|stack trace|IPC|internal policy|C:\\\\/i);
    }
  });

  it('keeps authored errors concise without losing safety guidance',()=>{
    const keys: UserMessageKey[]=['general','backend','binding','connection','launch',
      'selection','workspace','location','storage','settings','performance','audio','close',
      'recovery','closeAll','preview','profile','pack','worldPack','cache','clearData',
      'gameSettings','folder','event','graphicsBlocked','performanceBlocked',
      'windowControl','windowStatus','windowClose','closeIncomplete','closeRejected'];
    for(const key of keys){
      const message=userMessage(key);
      expect(message.length).toBeLessThanOrEqual(80);
      expect(message).not.toMatch(/PID|HWND|stack trace|internal policy/i);
    }
    expect(userMessage('launch')).toContain('Check Minecraft version support');
    expect(userMessage('launch')).not.toContain('may still be starting');
    expect(userMessage('recovery')).toContain('not window restoration');
    expect(userMessage('clearData')).toContain('unfinished');
    expect(userMessage('binding')).toContain('No game was launched');
  });

  it('retains distinct unconfirmed windows until each gets its own positive receipt',()=>{
    let slots=new Set<1|2|3|4|5>();
    slots=updateRecoverySlots(slots,3,false);
    slots=updateRecoverySlots(slots,2,false);
    expect(recoveryLabels(slots)).toBe('Minecraft - 2, Minecraft - 3');
    slots=updateRecoverySlots(slots,3,true);
    expect(recoveryLabels(slots)).toBe('Minecraft - 2');
    slots=updateRecoverySlots(slots,3,true);
    expect(recoveryLabels(slots)).toBe('Minecraft - 2');
    expect(recoveryLabels(updateRecoverySlots(slots,2,true))).toBe('');
  });
});

describe('Lazy Tester minimal session health projection',()=>{
  const normal=[1,2,3,4,5].map(slot=>({slot,state:'STOPPED',process:null,issue:null})) as ProfileSession[];
  const sample=(overrides:Partial<Parameters<typeof projectSessionHealth>[0]>={})=>({
    profiles:normal,
    previewStatuses:[],
    unresolvedRecovery:new Set<1|2|3|4|5>(),
    
    primarySlot:1 as const,
    observed:true,previewsVisible:true,...overrides,
  });
  it('stays invisible for normal, idle, unavailable or partial observations',()=>{
    expect(projectSessionHealth(sample())).toEqual([]);
    const active=normal.map(profile=>profile.slot===2?running(2) as ProfileSession:profile);
    expect(projectSessionHealth(sample({profiles:active}))).toEqual([]);
    const stale=active.map(profile=>profile.slot===2?
      {...profile,state:'ATTENTION' as const,process:null,issue:'private backend message'}:profile);
    expect(projectSessionHealth(sample({profiles:stale,observed:false}))).toEqual([]);
    expect(projectSessionHealth(sample({profiles:active,
      previewStatuses:[{slot:2,registered:false,reason:null}]}))).toEqual([]);
  });
  it('reuses unchanged launch receipts and copies only on an observed transition',()=>{
    const empty = new Map<ProfileSlot,'unverified'|'observed'>();
    expect(reconcileLaunchReceipts(empty, normal)).toBe(empty);
    const pending = new Map<ProfileSlot,'unverified'|'observed'>([[2,'unverified'],[4,'observed']]);
    expect(reconcileLaunchReceipts(pending, [])).toBe(pending);
    const observed = reconcileLaunchReceipts(pending, normal);
    expect(observed).not.toBe(pending);
    expect(observed.get(2)).toBe('observed');
    expect(observed.get(4)).toBe('observed');
    expect(pending.get(2)).toBe('unverified');
    expect(reconcileLaunchReceipts(observed, normal)).toBe(observed);
    const runningTwo = normal.map(profile=>profile.slot===2?running(2) as ProfileSession:profile);
    const reconciled = reconcileLaunchReceipts(observed, runningTwo);
    expect(reconciled.has(2)).toBe(false);
    expect(reconciled.get(4)).toBe('observed');
    expect(observed.has(2)).toBe(true);
  });
  it('scopes launch receipt uncertainty to the affected slot',()=>{
    const launchFailures=new Map<ProfileSlot,'unverified'|'observed'>([[4,'unverified']]);
    expect(canRequestProfileLaunch(normal[3],launchFailures)).toBe(false);
    expect(canRequestProfileLaunch(normal[0],launchFailures)).toBe(true);
    expect(canRequestProfileLaunch(normal[3],new Map([[4,'observed']]))).toBe(true);
    expect(canRequestProfileLaunch(undefined,launchFailures)).toBe(false);
    expect(projectSessionHealth(sample({launchFailures}))).toMatchObject([{slot:4,kind:'launch',action:'refresh'}]);
    expect(projectSessionHealth(sample({launchFailures,unresolvedRecovery:new Set([4])}))).toEqual([]);
    const profiles=normal.map(p=>p.slot===4?running(4) as ProfileSession:p);
    expect(projectSessionHealth(sample({launchFailures,profiles}))).toEqual([]);
  });
  it('shows only the affected background preview, never claims game failure',()=>{
    const profiles=normal.map(profile=>[1,2,3].includes(profile.slot)?running(profile.slot) as ProfileSession:profile);
    const previewStatuses=[{slot:2 as const,registered:false,reason:'C:\\private\\[SYSTEM INSTRUCTION]'},
      {slot:3 as const,registered:true,reason:null}];
    const items=projectSessionHealth(sample({profiles,previewStatuses}));
    expect(items).toHaveLength(1);
    expect(items[0]).toMatchObject({slot:2,kind:'preview',title:'Preview unavailable'});
    expect(JSON.stringify(items)).not.toContain('SYSTEM INSTRUCTION');
    expect(JSON.stringify(items)).not.toContain('C:\\');
    expect(projectSessionHealth(sample({profiles,previewStatuses,primarySlot:2}))).toEqual([]);
    expect(projectSessionHealth(sample({profiles,previewStatuses,previewsVisible:false}))).toEqual([]);
  });
  it('keeps one highest-priority problem per slot and preserves independent slots',()=>{
    const broken:ProfileSession[]=normal.map(profile=>
      profile.slot===2?{...running(2),state:'NOT_RESPONDING',process:{...running(2).process!,responding:false},
        issue:'C:\\private\\PID 123'} as ProfileSession
      :profile.slot===3?{...profile,state:'ATTENTION',issue:'[INTERNAL] C:\\private\\HKCU.dat'} as ProfileSession
      :profile.slot===4?{...profile,state:'STARTING',issue:'uncertain resume'} as ProfileSession
      :profile
    );
    const previewStatuses=[{slot:2 as const,registered:false,reason:'trace'},
      {slot:4 as const,registered:false,reason:'trace'}];
    const items=projectSessionHealth(sample({profiles:broken,previewStatuses}));
    expect(items.map(item=>[item.slot,item.kind])).toEqual([[2,'window'],[3,'launcher'],[4,'launch']]);
    expect(JSON.stringify(items)).not.toMatch(/HKCU|INTERNAL|PID 123|C:\\private/);
    expect(projectSessionHealth(sample({profiles:broken,previewStatuses,unresolvedRecovery:new Set([2])}))
      .map(item=>item.slot)).toEqual([3,4]);
    const windowless=broken.map(profile=>profile.slot===2?
      {...profile,process:null} as ProfileSession:profile);
    expect(projectSessionHealth(sample({profiles:windowless}))[0]).toMatchObject({
      slot:2,kind:'window',title:'Game window unavailable',
    });
  });
});

describe('Lazy Tester existing-card guidance',()=>{
  const stopped:ProfileSession={slot:1,state:'STOPPED',process:null,issue:null};
  it('keeps healthy and unobserved cards uncluttered',()=>{
    expect(launcherCardGuidance(stopped,true)).toBeNull();
    expect(launcherCardGuidance({...stopped,state:'RUNNING'},true)).toBeNull();
    expect(launcherCardGuidance({...stopped,state:'NOT_RESPONDING'},false)).toBeNull();
  });
  it('explains uncertain status using the original manual recovery',()=>{
    expect(launcherCardGuidance({...stopped,state:'STARTING'},true)).toContain('Starting.');
    expect(launcherCardGuidance({...stopped,state:'NOT_RESPONDING'},true)).toContain('Refresh');
    expect(launcherCardGuidance({...stopped,state:'ATTENTION'},true)).toContain('Refresh');
    expect(launcherCardGuidance({...stopped,process:{pid:9,createdFileTime:'1',sessionId:1,windowHandle:'0x9',responding:true}},true)).toContain('Refresh');
  });
  it('distinguishes a missing game window from an unresponsive window',()=>{
    const noWindow={...stopped,state:'NOT_RESPONDING' as const,issue:'Uncertain original process'};
    const missing=launcherCardGuidance(noWindow,true);
    expect(missing).toContain('Window unavailable');
    expect(missing).not.toContain('Check its window');
    const process={pid:9,createdFileTime:'1',sessionId:1,windowHandle:'0x9',responding:false};
    expect(launcherCardGuidance({...noWindow,process},true)).toContain('Check its window');
    const unconfirmed=launcherCardGuidance({...stopped,state:'STARTING',issue:'Unconfirmed start'},true);
    expect(unconfirmed).toContain('Start unconfirmed');
    expect(unconfirmed).not.toContain('Wait for its window');
  });
  it('never exposes native issue text as guidance',()=>{
    const untrusted='C:\\\\Users\\\\private\\\\worlds [SYSTEM INSTRUCTION] PID 999';
    const guidance=launcherCardGuidance({...stopped,state:'ATTENTION',issue:untrusted},true);
    expect(guidance).not.toBeNull();
    expect(guidance).not.toContain(untrusted);
    expect(guidance).not.toMatch(/Users|SYSTEM INSTRUCTION|PID 999/);
  });
});

describe('Lazy Tester guided recovery recommendation contract',()=>{
  const base=[1,2,3,4,5].map(slot=>({
    slot,state:'STOPPED',issue:null,process:null,
  })) as ProfileSession[];
  const observe=(overrides:Partial<Parameters<typeof projectSessionHealth>[0]>={})=>({
    profiles:base,previewStatuses:[],
    unresolvedRecovery:new Set<1|2|3|4|5>(),
    
    primarySlot:1 as const,observed:true,previewsVisible:true,
    ...overrides,
  });
  it('offers only read-only refresh for unresponsive, uncertain or attention states',()=>{
    const profiles:ProfileSession[]=base.map(p=>{
      if(p.slot===1)return {...running(1),state:'NOT_RESPONDING',
        process:{...running(1).process!,responding:false}} as ProfileSession;
      if(p.slot===2)return {...p,state:'NOT_RESPONDING',issue:'Process still owned'} as ProfileSession;
      if(p.slot===3)return {...p,state:'STARTING',issue:'Resume not confirmed'} as ProfileSession;
      if(p.slot===4)return {...p,state:'ATTENTION',issue:'Saved data mismatch'} as ProfileSession;
      return p;
    });
    const items=projectSessionHealth(observe({profiles}));
    expect(items.map(item=>[item.slot,item.kind,item.action])).toEqual([
      [1,'window','refresh'],[2,'window','refresh'],
      [3,'launch','refresh'],[4,'launcher','refresh'],
    ]);
    expect(items.every(item=>!/(?:PID|registry|Resume not confirmed|Saved data mismatch)/.test(item.guidance))).toBe(true);
  });
  it('offers Open only for a responding background window with failed preview',()=>{
    const profiles=base.map(p=>p.slot===2?running(2) as ProfileSession:p);
    const previewStatuses=[{slot:2 as const,registered:false,reason:'internal path: C:\\\\private'}];
    expect(projectSessionHealth(observe({profiles,previewStatuses}))).toMatchObject([
      {slot:2,kind:'preview',action:'open'},
    ]);
    expect(projectSessionHealth(observe({profiles,previewStatuses,primarySlot:2}))).toEqual([]);
    expect(projectSessionHealth(observe({profiles,previewStatuses,previewsVisible:false}))).toEqual([]);
    const unresponsive=profiles.map(p=>p.slot===2?
      {...p,state:'NOT_RESPONDING' as const,process:{...p.process!,responding:false}}:p);
    expect(projectSessionHealth(observe({profiles:unresponsive,previewStatuses}))[0]?.action).toBe('refresh');
    expect(projectSessionHealth(observe({profiles,previewStatuses,unresolvedRecovery:new Set([2])}))).toEqual([]);
    expect(projectSessionHealth(observe({profiles,previewStatuses,observed:false}))).toEqual([]);
  });
});

describe('Guided Recovery pending-work and stale-action hardening',()=>{
  const profile:ProfileSession={
    slot:2,state:'RUNNING',issue:null,
    process:{pid:102,createdFileTime:'1302',sessionId:1,windowHandle:'0x22',responding:true},
  };
  const issue={
    slot:2 as const,kind:'preview' as const,title:'Preview unavailable',
    guidance:'The preview failed.',action:'open' as const,
  };
  const gate={
    blocked:false,unresolvedRecovery:new Set<1|2|3|4|5>(),
    primarySlot:1 as const,
  };
  it('admits only a current matching slot and action, never an obsolete suggestion',()=>{
    expect(guidedRecoveryAllowed(issue,profile,2,'open',gate)).toBe(true);
    expect(guidedRecoveryAllowed(issue,profile,2,'refresh',gate)).toBe(false);
    expect(guidedRecoveryAllowed(issue,{...profile,slot:3},2,'open',gate)).toBe(false);
    expect(guidedRecoveryAllowed(undefined,profile,2,'open',gate)).toBe(false);
    expect(guidedRecoveryAllowed(issue,profile,2,'open',{...gate,blocked:true})).toBe(false);
    expect(guidedRecoveryAllowed(issue,profile,2,'open',
      {...gate,unresolvedRecovery:new Set([2])})).toBe(false);
    expect(guidedRecoveryAllowed(issue,profile,2,'open',
      {...gate,primarySlot:2})).toBe(false);
    expect(guidedRecoveryAllowed(issue,{...profile,state:'NOT_RESPONDING'},2,'open',gate)).toBe(false);
    expect(guidedRecoveryAllowed(issue,{...profile,process:{...profile.process!,responding:false}},2,'open',gate)).toBe(false);
    expect(guidedRecoveryAllowed(issue,{...profile,process:null},2,'open',gate)).toBe(false);
  });
  it('keeps Refresh as a read-only recommendation with the same pending-work guard',()=>{
    const check={...issue,kind:'window' as const,action:'refresh' as const};
    const blockedProfile={...profile,state:'NOT_RESPONDING' as const,process:null};
    expect(guidedRecoveryAllowed(check,blockedProfile,2,'refresh',gate)).toBe(true);
    expect(guidedRecoveryAllowed(check,blockedProfile,2,'refresh',{...gate,blocked:true})).toBe(false);
    expect(guidedRecoveryAllowed(check,blockedProfile,2,'refresh',
      {...gate,unresolvedRecovery:new Set([2])})).toBe(false);
    expect(guidedRecoveryAllowed(check,blockedProfile,2,'open',gate)).toBe(false);
  });
});

describe('Lazy Tester queue scheduling parity',()=>{
  const idle:PendingUiQueueState={
    mounted:true,nativeBackend:'WindowsNative',closeInProgress:false,
    closeReviewPending:false,initializing:false,busy:false,refreshing:false,
    resizing:false,queuedAudio:false,queuedPerformance:false,
    queuedPage:false,manualRefreshRequested:false,
    queuedLaunch:false,queuedSelection:false,dirty:false,
  };
  // The remaining seven Svelte reactive admission clauses, preserved here as
  // independent regression truth, not as a second runtime dispatcher.
  function previousFirst(s:PendingUiQueueState):PendingUiAction|null {
    const common=s.mounted&&s.nativeBackend==='WindowsNative'&&!s.closeInProgress&&
      !s.closeReviewPending&&!s.busy&&!s.refreshing&&!s.resizing;
    const strict=common&&!s.initializing;
    const eligible=[
      strict&&s.queuedAudio?'audio':null,
      strict&&!s.queuedAudio&&s.queuedPerformance?'performance':null,
      strict&&!s.queuedAudio&&!s.queuedPerformance&&s.queuedPage?'navigation':null,
      strict&&!s.queuedPerformance&&!s.queuedPage&&s.manualRefreshRequested?'manualRefresh':null,
      strict&&!s.queuedPerformance&&!s.queuedPage&&s.queuedLaunch?'launch':null,
      common&&!s.queuedAudio&&!s.queuedPerformance&&!s.queuedPage&&s.queuedSelection?'selection':null,
      common&&!s.queuedAudio&&!s.queuedPerformance&&!s.queuedPage&&!s.queuedSelection&&s.dirty?'layout':null,
    ];
    return (eligible.find((item):item is PendingUiAction=>item!==null)??null);
  }

  it('preserves first-eligible lane and priority for all 128 queue combinations',()=>{
    const queueKeys=[
      'queuedAudio','queuedPerformance','queuedPage',
      'manualRefreshRequested','queuedLaunch','queuedSelection','dirty',
    ] as const;
    for(let flags=0;flags<128;flags++){
      const mask=Object.fromEntries(queueKeys.map((key,index)=>[key,!!(flags&(1<<index))]));
      for(const variants of [
        {},{initializing:true},{busy:true},{refreshing:true},
        {closeReviewPending:true},{closeInProgress:true},{resizing:true},
        {nativeBackend:'Unsupported' as const},{mounted:false},
      ]){
        const state={...idle,...mask,...variants};
        expect(nextPendingUiAction(state)).toBe(state.initializing ? null : previousFirst(state));
      }
    }
  });
  it('clears stale preview destinations before review-only placement',()=>{
    const source=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const review=source.slice(source.indexOf('async function syncSelectedReviewGeometry()'),source.indexOf('async function syncLayout()'));
    expect(review).toContain('await lazyTester.clearPreviews()');
    expect(review.indexOf('await lazyTester.clearPreviews()')).toBeLessThan(review.indexOf('await workspaceSurface.update('));
    expect(review).toContain('if (!sameWorkspaceGeometry(geometry,captureWorkspaceGeometry()))');
    expect(review.indexOf('if (!sameWorkspaceGeometry(geometry,captureWorkspaceGeometry()))')).toBeLessThan(review.indexOf('await workspaceSurface.update('));
    expect(review).not.toContain('lazyTester.previews(');
    expect(review).not.toContain('lazyTester.inspect(');
  });
  it('allows only retained geometry during an eligible selected close review',()=>{
    const review={...idle,closeReviewPending:true,selectedReviewGeometryOnly:true,
      queuedLaunch:true,queuedSelection:true,manualRefreshRequested:true,dirty:true};
    expect(nextPendingUiAction(review)).toBe('layout');
    expect(nextPendingUiAction({...review,dirty:false})).toBeNull();
    expect(nextPendingUiAction({...review,selectedReviewGeometryOnly:false})).toBeNull();
    expect(nextPendingUiAction({...review,closeInProgress:true})).toBeNull();
    expect(nextPendingUiAction({...review,resizing:true})).toBeNull();
  });
  it('keeps layout and selection out of initial profile/session inspection',()=>{
    expect(nextPendingUiAction({...idle,manualRefreshRequested:true,queuedLaunch:true})).toBe('manualRefresh');
    expect(nextPendingUiAction({...idle,queuedAudio:true,manualRefreshRequested:true})).toBe('audio');
    expect(nextPendingUiAction({...idle,queuedSelection:true,dirty:true})).toBe('selection');
    expect(nextPendingUiAction({...idle,initializing:true,queuedSelection:true})).toBeNull();
    expect(nextPendingUiAction({...idle,initializing:true,dirty:true})).toBeNull();
    expect(nextPendingUiAction({...idle,initializing:false,dirty:true})).toBe('layout');
    expect(nextPendingUiAction({...idle,initializing:true,queuedAudio:true})).toBeNull();
    expect(nextPendingUiAction({...idle,queuedAudio:true,closeReviewPending:true,queuedSelection:true})).toBeNull();
  });
});

describe('Workspace geometry identity',()=>{
  it('does not request layout solely for non-Play geometry changes',()=>{
    const source=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    const mount=source.split('onMount(() => {')[1]!.split('const stopDisplayScale=')[0]!;
    expect(mount).toContain("if (page==='workspace' || previewIssue!=='') dirty = true;");
    expect(mount).toContain("if (page==='workspace' || previewIssue!=='' || visibilityChanged ||\n          event?.type==='visibilitychange') dirty = true;");
    expect(source).toContain("if (next !== 'workspace') await workspaceSurface.suspend();");
    expect(source).toContain('if (current()) dirty = true;');
  });
  it('passes the captured DPI to both native layout bridge calls',()=>{
    const source=readFileSync(new URL('../src/LazyTesterApp.svelte',import.meta.url),'utf8');
    expect(source).toContain('workspaceSurface.update(geometry.viewport,geometry.pixelRatio)');
    expect(source).toContain('lazyTester.previews(geometry.viewport,[...geometry.tiles],geometry.pixelRatio)');
    const bridge=readFileSync(new URL('../src/app/bridge/lazyTester.ts',import.meta.url),'utf8');
    expect(bridge).toContain('previewViewport(tile.rect,pixelRatio)');
    expect(bridge).toContain('viewport:workspaceViewport(rect,pixelRatio),tiles:placements');
  });
  const rect=(x=0,y=0,width=640,height=360)=>({x,y,width,height} as DOMRect);
  const baseline:WorkspaceGeometry={viewport:rect(),tiles:[{slot:2,rect:rect(20,400,160,90)}],pixelRatio:1};
  it('rejects position-only changes, DPI changes and changed preview targets',()=>{
    expect(sameWorkspaceGeometry(baseline,{...baseline,viewport:rect()})).toBe(true);
    expect(sameWorkspaceGeometry(baseline,{...baseline,viewport:rect(1)})).toBe(false);
    expect(sameWorkspaceGeometry(baseline,{...baseline,tiles:[{slot:2,rect:rect(20,401,160,90)}]})).toBe(false);
    expect(sameWorkspaceGeometry(baseline,{...baseline,pixelRatio:1.25})).toBe(false);
    expect(sameWorkspaceGeometry(baseline,{...baseline,tiles:[]})).toBe(false);
  });
});

describe('Recording receipt contract',()=>{
  it('accepts independent slots without interpreting a launch as a recording',()=>{
    const receipts=parseRecordingStatus([{slot:1,state:'Starting',frames:0,recoveryFile:null},{slot:3,state:'Recording',frames:7,recoveryFile:null},{slot:5,state:'Saved',frames:120,recoveryFile:null}]);
    expect(receipts.filter(recordingActive).map(r=>r.slot)).toEqual([1,3]);
    expect(recordingActive(undefined)).toBe(false);
    expect(recordingActive({slot:2,state:'Stopping',frames:0,recoveryFile:null})).toBe(true);
  });
  it('exposes only a terminal, generated recovery filename, not a path',()=>{
    const result=parseRecordingStatus([{slot:2,state:'Failed',frames:9,recoveryFile:'.lazy-tester-video-abc123.part'}]);
    expect(result[0]?.recoveryFile).toBe('.lazy-tester-video-abc123.part');
    expect(recordingActive(result[0])).toBe(false);
  });
  it('refuses duplicate, invalid or unconfirmed recording receipts',()=>{
    const good={slot:1,state:'Recording',frames:1,recoveryFile:null};
    for(const value of [[good,good],[{...good,slot:6}],[{...good,frames:0}],
      [{...good,state:'Saved',frames:0}],[{...good,state:'Starting'}],
      [{...good,frames:-1}],[{...good,state:'Success'}],[{...good,state:['Recording']}],[{...good,path:'private.mkv'}],[{...good,recoveryFile:'.lazy-tester-video-abc.part'}],[{...good,state:'Failed',recoveryFile:'../private.mkv'}],[{...good,state:'Failed',frames:0,recoveryFile:'.lazy-tester-video-abc.part'}],null]){
      expect(()=>parseRecordingStatus(value)).toThrow();
    }
  });
});
