import {describe, expect, it} from 'vitest';
import {nextProfile, parseSession, type ProfileSession} from '../src/app/bridge/lazyTester.js';

const stopped = () => ({schema: 1, storage: null, profiles: [1,2,3,4,5].map(slot => ({slot, state: 'STOPPED', process: null, issue: null}))});
const running = (slot: number) => ({slot, state: 'RUNNING', issue: null, process: {
  pid: slot + 100, createdFileTime: String(slot + 200), sessionId: 1, windowHandle: `0x${slot}`, responding: true,
}});
describe('Lazy Tester profile contract', () => {
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
