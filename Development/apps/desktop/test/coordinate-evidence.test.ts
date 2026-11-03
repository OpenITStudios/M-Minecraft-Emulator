import {describe, expect, it} from 'vitest';
import {
  assessUnverifiedCoordinateCandidate,
  assessUnverifiedCoordinateSequence,
} from '../src/app/coordinateEvidence.js';

const sample = () => ({
  schema: 1,
  position: {x: 123.456, y: 64, z: -45.125},
  rotation: {x: -12.5, y: 90},
  dimensionId: 'minecraft:overworld',
  gameTick: 100,
});

describe('source-only coordinate candidate provenance gate', () => {
  const process = () => ({
    pid: 1234, createdFileTime: '134000000000000001',
    sessionId: 1, windowHandle: '0x123', responding: true,
  });
  const hint = () => {
    const {responding: _unused, ...identity} = process();
    return identity;
  };
  const candidate = () => ({schema: 1, slot: 2, process: hint(), sample: sample()});
  const context = () => ({
    selectedSlot: 2 as const,
    profile: {slot: 2 as const, state: 'RUNNING' as const,
      process: process(), issue: null},
    receivedAtMonotonicMs: 1000, nowMonotonicMs: 1500,
  });

  it('always denies clipboard even for a fresh structurally matching candidate', () => {
    expect(assessUnverifiedCoordinateCandidate(candidate(), context()))
      .toEqual({status: 'PLAYER_WORLD_ORIGIN_UNVERIFIED', copyAllowed: false});
    expect(assessUnverifiedCoordinateCandidate(null, context()))
      .toEqual({status: 'NO_CANDIDATE', copyAllowed: false});
  });

  it('rejects cross-slot reports and PID/birth/session/HWND changes', () => {
    expect(assessUnverifiedCoordinateCandidate({...candidate(), slot: 3}, context()).status)
      .toBe('SLOT_MISMATCH');
    for (const changed of [
      {pid: 1235}, {createdFileTime: '134000000000000002'},
      {sessionId: 2}, {windowHandle: '0x124'},
    ]) {
      expect(assessUnverifiedCoordinateCandidate({
        ...candidate(), process: {...hint(), ...changed},
      }, context()).status).toBe('PROCESS_MISMATCH');
    }
    expect(assessUnverifiedCoordinateCandidate(candidate(), {
      ...context(), profile: {...context().profile, process: {...process(), responding: false}},
    }).status).toBe('NO_RESPONDING_CLIENT');
    expect(assessUnverifiedCoordinateCandidate(candidate(), {
      ...context(), profile: {...context().profile, state: 'STOPPED'},
    }).status).toBe('NO_RESPONDING_CLIENT');
    expect(assessUnverifiedCoordinateCandidate(candidate(), {
      ...context(), selectedSlot: 1,
    }).status).toBe('NO_RESPONDING_CLIENT');
  });

  it('rejects claims of verified source, world or player as extra fields', () => {
    for (const claim of [
      {sourceVerified: true}, {worldId: 'claimed'}, {playerId: 'claimed'},
      {origin: 'TRUSTED'}, {capturedAtMs: 1000},
    ]) {
      expect(assessUnverifiedCoordinateCandidate({
        ...candidate(), ...claim,
      }, context()).status).toBe('MALFORMED_CANDIDATE');
    }
    expect(assessUnverifiedCoordinateCandidate({
      ...candidate(), process: {...hint(), responding: true},
    }, context()).status).toBe('MALFORMED_CANDIDATE');
    expect(assessUnverifiedCoordinateCandidate({
      ...candidate(), sample: {...sample(), verified: true},
    }, context()).status).toBe('MALFORMED_CANDIDATE');
  });

  it('rejects missing, negative, old, non-finite or future receipt times', () => {
    for (const delta of [
      {receivedAtMonotonicMs: null},
      {receivedAtMonotonicMs: 0, nowMonotonicMs: 2001},
      {receivedAtMonotonicMs: -1},
      {receivedAtMonotonicMs: 2000, nowMonotonicMs: 1999},
      {receivedAtMonotonicMs: NaN},
      {nowMonotonicMs: Infinity},
    ]) expect(assessUnverifiedCoordinateCandidate(candidate(), {
      ...context(), ...delta,
    }).status).toBe('STALE_OR_UNTIMED');
    expect(assessUnverifiedCoordinateCandidate(candidate(), {
      ...context(), receivedAtMonotonicMs: 0, nowMonotonicMs: 2000,
    }).status).toBe('PLAYER_WORLD_ORIGIN_UNVERIFIED');
  });
});

describe('consecutive unverified coordinate samples', () => {
  const process = () => ({
    pid: 1234, createdFileTime: '134000000000000001',
    sessionId: 1, windowHandle: '0x123',
  });
  const frame = (tick: number) => ({
    schema: 1, slot: 2, process: process(),
    sample: {...sample(), gameTick: tick},
  });
  it('screens plausible continuation but never permits copy', () => {
    expect(assessUnverifiedCoordinateSequence(frame(100), frame(101), 10, 20))
      .toEqual({status: 'CONTINUOUS_BUT_UNVERIFIED', copyAllowed: false});
    expect(assessUnverifiedCoordinateSequence(null, frame(101), null, 20))
      .toEqual({status: 'NO_PREVIOUS_CANDIDATE', copyAllowed: false});
  });
  it('rejects slot and all process identity changes, including PID reuse', () => {
    expect(assessUnverifiedCoordinateSequence(
      frame(100), {...frame(101), slot: 3}, 10, 20,
    ).status).toBe('SLOT_CHANGED');
    for (const changed of [
      {pid: 4321}, {createdFileTime: '134000000000000002'},
      {sessionId: 2}, {windowHandle: '0x124'},
    ]) {
      expect(assessUnverifiedCoordinateSequence(
        frame(100), {...frame(101), process: {...process(), ...changed}}, 10, 20,
      ).status).toBe('PROCESS_REPLACED');
    }
  });
  it('rejects dimension changes, repeated/rewound ticks and malformed frames', () => {
    expect(assessUnverifiedCoordinateSequence(frame(100), {
      ...frame(101), sample: {...sample(), dimensionId: 'minecraft:nether', gameTick: 101},
    }, 10, 20).status).toBe('DIMENSION_CHANGED');
    for (const tick of [100, 99, 0]) {
      expect(assessUnverifiedCoordinateSequence(frame(100), frame(tick), 10, 20).status)
        .toBe('TICK_REPLAY_OR_RESET');
    }
    for (const bad of [
      {...frame(101), verified: true},
      {...frame(101), process: {...process(), playerName: 'spoof'}},
      {...frame(101), sample: {...sample(), gameTick: Infinity}},
      null,
    ]) expect(assessUnverifiedCoordinateSequence(frame(100), bad, 10, 20).status)
      .toBe('MALFORMED_SEQUENCE');
  });
  it('rejects invalid, reversed or excessive receiver-side time gaps', () => {
    for (const [before,after] of [
      [null, 20], [0, null], [-1, 10],
      [NaN, 10], [0, Infinity], [200, 199], [0, 2001],
    ]) {
      expect(assessUnverifiedCoordinateSequence(frame(100), frame(101), before, after).status)
        .toBe('RECEIPT_SEQUENCE_INVALID');
    }
    expect(assessUnverifiedCoordinateSequence(frame(100), frame(101), 0, 2000).status)
      .toBe('CONTINUOUS_BUT_UNVERIFIED');
  });
});
