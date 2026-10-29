import {describe, expect, it} from 'vitest';
import {lazyTester, parseSelectedClose, firstEmbeddedProfile, nextProfile, parseSession, parsePreviewReport, parseDwmPreviewInvalidation, dwmPreviewInvalidationApplies, parseProfileBinding, parseDataLocation, validFpsLimits, parseNativeBackendSupport, type ProfileSession} from '../src/app/bridge/lazyTester.js';
import {previewViewport} from '../src/app/bridge/workspaceSurface.js';
import {projectSessionHealth} from '../src/app/presentation/sessionHealth.js';
import {userMessage, sessionConnectionLabel, updateRecoverySlots, recoveryLabels, type UserMessageKey} from '../src/app/presentation/sessionFeedback.js';
import {closeReviewActive, observationStillCurrent, previewLayoutStillCurrent, periodicInspectionDue, type CloseReview} from '../src/app/inspectionScheduling.js';

const stopped = () => ({schema: 1, storage: null, profiles: [1,2,3,4,5].map(slot => ({slot, state: 'STOPPED', process: null, issue: null}))});
const running = (slot: number) => ({slot, state: 'RUNNING', issue: null, process: {
  pid: slot + 100, createdFileTime: String(slot + 200), sessionId: 1, windowHandle: `0x${slot}`, responding: true,
}});
describe('Lazy Tester profile contract', () => {
  it('accepts only compiled-backend support, never user agent or gameplay guesses',()=>{
    expect(parseNativeBackendSupport('WindowsNative')).toBe('WindowsNative');
    expect(parseNativeBackendSupport('Unsupported')).toBe('Unsupported');
    for (const forged of [true,false,null,undefined,0,'macos','MacNative','Ready',{},['WindowsNative']]) {
      expect(()=>parseNativeBackendSupport(forged)).toThrow('Invalid native backend capability');
    }
  });
  it('throttles read-only idle settings/storage inspection without delaying live or recovery views', () => {
    for (const page of ['settings', 'storage'] as const) {
      for (const elapsed of [0, 1999, 2000, 9999]) {
        expect(periodicInspectionDue(page, 0, true, elapsed)).toBe(false);
      }
      expect(periodicInspectionDue(page, 0, true, 10000)).toBe(true);
      expect(periodicInspectionDue(page, 1, true, 0)).toBe(true);
      expect(periodicInspectionDue(page, 0, false, 0)).toBe(true);
    }
    for (const page of ['workspace', 'launchers'] as const) {
      expect(periodicInspectionDue(page, 0, true, 0)).toBe(true);
      expect(periodicInspectionDue(page, 0, false, 0)).toBe(true);
    }
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
  it('drops late preview layout observations on minimize, source swap or close review',()=>{
    const idle: CloseReview = {
      nativeExitReview:false,closeAllRunning:false,selectedReview:false,selectedCloseRunning:false,
    };
    const issued={observationEpoch:5,processIdentity:'2:RUNNING:pid100/windowA',primarySlot:1};
    const current={...issued,dirty:false,managerVisible:true,workspaceAvailable:true};
    expect(previewLayoutStillCurrent(issued,current,idle)).toBe(true);
    for(const changed of [
      {...current,observationEpoch:6},
      {...current,processIdentity:'2:RUNNING:pid101/windowB'},
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
  it('leaves separately opened games alone while another game remains playable here',()=>{
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
  it('rejects a forged texture-streaming profile before game IPC',async()=>{
    for(const slot of [0,-1,6,99] as number[]) {
      await expect(lazyTester.enableTextureStreaming(slot as 1)).rejects.toThrow('Unknown Minecraft profile');
    }
  });
  it('refuses a forged RAM workaround slot without contacting the native controller', async()=>{
    for (const slot of [0,6,-1,10] as number[]) {
      await expect(lazyTester.disableImprovedInputResponse(slot as 1)).rejects.toThrow('Unknown Minecraft profile');
    }
  });
  it('refuses a forged per-launcher close slot before invoking Tauri', async()=>{
    for (const slot of [0,6,-1,10] as number[]) {
      await expect(lazyTester.closeOne(slot as 1)).rejects.toThrow('Unknown Minecraft profile');
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
    expect(nextProfile(snapshot.profiles, null, 1)).toBeNull();
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
  it('cycles only playable profiles in stable order, including profile five', () => {
    const snapshot = stopped();
    for (const slot of [1,3,5]) Object.assign(snapshot.profiles[slot - 1]!, running(slot));
    const profiles = parseSession(snapshot).profiles;
    expect(nextProfile(profiles, 1, 1)).toBe(3);
    expect(nextProfile(profiles, 3, 1)).toBe(5);
    expect(nextProfile(profiles, 5, 1)).toBe(1);
    expect(nextProfile(profiles, 1, -1)).toBe(5);
    expect(nextProfile(profiles, null, -1)).toBe(5);
    expect(nextProfile(profiles.map(profile => ({...profile, state: 'STARTING'} as ProfileSession)), 1, 1)).toBeNull();
  });
});

describe('Lazy Tester user-facing feedback and recovery warnings',()=>{
  it('reports observed profiles without claiming a live controller or game sign-in',()=>{
    expect(sessionConnectionLabel(true,'WindowsNative',true,true)).toBe('Checking profiles…');
    expect(sessionConnectionLabel(false,'Unsupported',false,false)).toBe('Not supported');
    expect(sessionConnectionLabel(false,null,true,true)).toBe('Status unavailable');
    expect(sessionConnectionLabel(false,'WindowsNative',false,true)).toBe('Profiles unavailable');
    expect(sessionConnectionLabel(false,'WindowsNative',true,false)).toBe('Session status unavailable');
    expect(sessionConnectionLabel(false,'WindowsNative',true,true)).toBe('Profiles verified');
  });

  it('uses only authored context messages, never untrusted backend paths or prompt text',()=>{
    const hostile='C:\\Users\\private\\worlds [internal policy] PID 1234';
    const contexts:UserMessageKey[]=['backend','binding','connection','launch','selection',
      'workspace','location','storage','settings','performance','audio','close',
      'recovery','closeAll','preview','profile','pack','worldPack','cache','clearData',
      'gameSettings','folder','event','general'];
    for(const context of contexts){
      const message=userMessage(context);
      expect(message.length).toBeGreaterThan(12);
      expect(message).not.toContain(hostile);
      expect(message).not.toContain('C:\\');
      expect(message).not.toMatch(/PID|HWND|stack trace|IPC|internal policy/i);
    }
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
    separateWindows:new Set<1|2|3|4|5>(),
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
    expect(projectSessionHealth(sample({profiles,previewStatuses,separateWindows:new Set([2])}))).toEqual([]);
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
