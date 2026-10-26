import {describe, expect, it} from 'vitest';
import {firstEmbeddedProfile, nextProfile, parseSession, parsePreviewReport, parseProfileBinding, parseDataLocation, validFpsLimits, type ProfileSession} from '../src/app/bridge/lazyTester.js';
import {previewViewport} from '../src/app/bridge/workspaceSurface.js';

const stopped = () => ({schema: 1, storage: null, profiles: [1,2,3,4,5].map(slot => ({slot, state: 'STOPPED', process: null, issue: null}))});
const running = (slot: number) => ({slot, state: 'RUNNING', issue: null, process: {
  pid: slot + 100, createdFileTime: String(slot + 200), sessionId: 1, windowHandle: `0x${slot}`, responding: true,
}});
describe('Lazy Tester profile contract', () => {
  it('admits settings recovery only for its stopped launcher with consistent backup metadata',()=>{
    const gameSettings={hasSettings:true,backupAvailable:true,canReset:true,canRestore:true,issue:null};
    const storage=[1,2,3,4,5].map(slot=>({slot,bytes:100,files:2,partial:false,disk:null,
      cache:{bytes:0,files:0,partial:false,canClear:false,issue:null},gameSettings}));
    const snapshot={...stopped(),storage};
    expect(parseSession(snapshot).storage![3]!.gameSettings?.canRestore).toBe(true);
    for(const invalid of [{...gameSettings,hasSettings:false},{...gameSettings,backupAvailable:false},
      {...gameSettings,issue:'Locked'},{...gameSettings,canReset:'yes'},{...gameSettings,path:'../outside'},null]) {
      expect(()=>parseSession({...snapshot,storage:[{...storage[0]!,gameSettings:invalid},...storage.slice(1)]})).toThrow();
    }
    const active={...snapshot,profiles:[running(1),...snapshot.profiles.slice(1)]};
    expect(()=>parseSession(active)).toThrow();
    const gated={...active,storage:storage.map(item=>item.slot===1?{...item,gameSettings:{...gameSettings,canReset:false,canRestore:false,issue:'Close this Minecraft'}}:item)};
    expect(parseSession(gated).storage![3]!.gameSettings?.canReset).toBe(true);
    expect(parseSession(gated).storage![0]!.gameSettings?.canRestore).toBe(false);
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
  it('accepts exactly five High/Low selections and rejects invented graphics states',()=>{
    const settings={activeFps:144,backgroundFps:10,renderDistanceChunks:12,canApplyPreset:true,changedProfiles:null,graphicsPresets:['High','Low','High','Low','Low']};
    expect(parseSession({...stopped(),settings}).settings!.graphicsPresets).toEqual(settings.graphicsPresets);
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
  it('allows small preview viewports but refuses offscreen and invalid coordinate payloads',()=>{
    expect(previewViewport({x:10,y:450,width:145,height:80},1.5)).toEqual({x:10,y:450,width:145,height:80,pixelRatio:1.5});
    for(const rect of [{x:-1,y:450,width:145,height:80},{x:10,y:450,width:31,height:80},{x:10,y:450,width:145,height:NaN}])expect(()=>previewViewport(rect)).toThrow();
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
