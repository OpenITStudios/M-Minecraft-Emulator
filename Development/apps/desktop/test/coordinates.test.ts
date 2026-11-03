import {describe, expect, it} from 'vitest';
import {
  blockCoordinates, chunkCoordinates, coordinateCopyText,
  coordinateDisplayXYZ, parseUnboundCoordinateSample,
  assessUnverifiedCoordinateCandidate,
} from '../src/app/coordinates.js';

const sample = () => ({
  schema: 1,
  position: {x: 123.456, y: 64, z: -45.125},
  rotation: {x: -12.5, y: 90},
  dimensionId: 'minecraft:overworld',
  gameTick: 100,
});

describe('unbound F3 coordinate contract', () => {
  it('returns a detached, allowlisted value without inventing a client/world identity', () => {
    const payload = sample();
    const result = parseUnboundCoordinateSample(payload);
    expect(result).toEqual(payload);
    expect(result).not.toBe(payload);
    expect(result.position).not.toBe(payload.position);
    expect(result.rotation).not.toBe(payload.rotation);
    payload.position.x = 111;
    expect(result.position.x).toBe(123.456);
    expect(result).not.toHaveProperty('slot');
    expect(result).not.toHaveProperty('sourceVerified');
  });

  it('rejects forged provenance and arbitrary player or runtime metadata', () => {
    for (const extra of [
      {slot: 2}, {verified: true}, {worldId: 'world-a'},
      {playerName: 'someone'}, {pid: 100}, {source: 'TRUSTED'},
      {capturedAt: '2026-11-03T03:00:00Z'},
    ]) {
      expect(() => parseUnboundCoordinateSample({...sample(), ...extra})).toThrow();
    }
    expect(() => parseUnboundCoordinateSample({...sample(), position: {
      ...sample().position, playerId: 'someone',
    }})).toThrow();
    expect(() => parseUnboundCoordinateSample({...sample(), rotation: {
      ...sample().rotation, yaw: 90,
    }})).toThrow();
  });

  it('rejects malformed nested data, coerced numbers and non-finite coordinates', () => {
    for (const bad of [
      null, [], {}, { ...sample(), schema: 2 },
      {...sample(), dimensionId: 'Overworld'},
      {...sample(), dimensionId: '../world'},
      {...sample(), dimensionId: 'minecraft:overworld\nINJECT'},
      {...sample(), dimensionId: 'a:' + 'b'.repeat(96)},
      {...sample(), gameTick: -1},
      {...sample(), gameTick: 1.5},
      {...sample(), gameTick: Number.MAX_SAFE_INTEGER + 1},
      {...sample(), position: null},
      {...sample(), position: {x: '123', y: 64, z: 1}},
      {...sample(), position: {x: Infinity, y: 64, z: 1}},
      {...sample(), position: {x: 1e10, y: 64, z: 1}},
      {...sample(), position: {x: 0, y: NaN, z: 1}},
      {...sample(), rotation: {x: 0, y: '90'}},
    ]) expect(() => parseUnboundCoordinateSample(bad)).toThrow();
  });
});

describe('copy and presentation formatting, not game control', () => {
  it('keeps full numeric copy precision separate from three-decimal display', () => {
    expect(coordinateDisplayXYZ(sample())).toBe('123.456 / 64.000 / -45.125');
    expect(coordinateCopyText(sample(), 'XYZ')).toBe('123.456 64 -45.125');
    const tiny = {...sample(), position: {x: 1e-7, y: -1e-7, z: -0}};
    expect(coordinateCopyText(tiny, 'XYZ')).toBe('0.0000001 -0.0000001 0');
    expect(coordinateDisplayXYZ(tiny)).toBe('0.000 / 0.000 / 0.000');
    expect(coordinateCopyText({...sample(), position: {x: 1e9, y: 1e-6, z: 0}}, 'XYZ'))
      .toBe('1000000000 0.000001 0');
  });

  it('uses mathematical floor for block and chunk positions, especially negative axes', () => {
    expect(blockCoordinates(sample().position)).toEqual({x: 123, y: 64, z: -46});
    expect(chunkCoordinates(sample().position)).toEqual({x: 7, z: -3});
    expect(coordinateCopyText(sample(), 'BLOCK')).toBe('123 64 -46');
    expect(blockCoordinates({x: -0.01, y: -0.01, z: -16.01}))
      .toEqual({x: -1, y: -1, z: -17});
    expect(chunkCoordinates({x: -0.01, y: 0, z: -16.01}))
      .toEqual({x: -1, z: -2});
  });

  it('produces only a same-dimension self /tp text with yaw then pitch', () => {
    expect(coordinateCopyText(sample(), 'TELEPORT'))
      .toBe('/tp @s 123.456 64 -45.125 90 -12.5');
    expect(coordinateCopyText({...sample(), dimensionId:'custom:dimension/arena'}, 'TELEPORT'))
      .toBe('/tp @s 123.456 64 -45.125 90 -12.5');
    expect(() => coordinateCopyText(sample(), 'COMMAND' as never)).toThrow();
    expect(() => coordinateCopyText({...sample(), position: {x: NaN, y: 64, z: 1}}, 'XYZ'))
      .toThrow();
  });
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
