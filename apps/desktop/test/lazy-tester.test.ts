import {describe, expect, it} from 'vitest';
import {nextProfile, parseSession, parsePreviewReport, type ProfileSession} from '../src/app/bridge/lazyTester.js';
import {previewViewport} from '../src/app/bridge/workspaceSurface.js';

const stopped = () => ({schema: 1, storage: null, profiles: [1,2,3,4,5].map(slot => ({slot, state: 'STOPPED', process: null, issue: null}))});
const running = (slot: number) => ({slot, state: 'RUNNING', issue: null, process: {
  pid: slot + 100, createdFileTime: String(slot + 200), sessionId: 1, windowHandle: `0x${slot}`, responding: true,
}});
describe('Lazy Tester profile contract', () => {
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
