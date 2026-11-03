import {describe, expect, it} from 'vitest';
import {
  blockCoordinates, chunkCoordinates, coordinateCopyText,
  coordinateDisplayXYZ, parseUnboundCoordinateSample,
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
