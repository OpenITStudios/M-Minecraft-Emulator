import {describe, expect, it} from 'vitest';
import {
  blockCoordinates, chunkCoordinates, coordinateCopyText,
  coordinateDisplayXYZ, parseUnboundCoordinateSample,
  parseUserPastedCoordinates, formatUserPastedCoordinates, displayUserPastedCoordinates,
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
describe('explicit user-paste XYZ fallback, never live game attribution', () => {
  it('accepts Bedrock-style plain and comma-separated coordinate text', () => {
    expect(parseUserPastedCoordinates(' 123.456 64 -45.125 ')).toEqual({
      origin: 'USER_PASTED_UNVERIFIED',
      position: {x: 123.456, y: 64, z: -45.125},
    });
    expect(parseUserPastedCoordinates('123.456, 64, -45.125').position)
      .toEqual({x: 123.456, y: 64, z: -45.125});
    expect(parseUserPastedCoordinates('1,2,3').position)
      .toEqual({x: 1, y: 2, z: 3});
    expect(parseUserPastedCoordinates(' +12\t-.5\t.25 ').position)
      .toEqual({x: 12, y: -0.5, z: 0.25});
  });
  it('formats XYZ, negative block floors and self teleport without fabricated rotation', () => {
    const input = '123.456 64 -45.125';
    expect(formatUserPastedCoordinates(input, 'XYZ')).toBe('123.456 64 -45.125');
    expect(formatUserPastedCoordinates(input, 'BLOCK')).toBe('123 64 -46');
    expect(formatUserPastedCoordinates(input, 'TELEPORT_POSITION_ONLY'))
      .toBe('/tp @s 123.456 64 -45.125');
    expect(displayUserPastedCoordinates(input)).toBe('123.456 / 64.000 / -45.125');
    expect(formatUserPastedCoordinates('0.0000001 -0.0000001 -0', 'XYZ'))
      .toBe('0.0000001 -0.0000001 0');
    expect(() => formatUserPastedCoordinates('0 0 0', 'TELEPORT' as never)).toThrow();
  });
  it('rejects malformed, unbounded, multiline and command-bearing text', () => {
    for (const raw of [
      null, 123, {}, [], '', '1 2', '1 2 3 4', '1,,2,3',
      '1, 2,', '1 2,3', '1,2 3', '1 2;3', '/tp @s 1 2 3', 'XYZ: 1 2 3',
      '1 2 3\nsay hi', '1 2 3\r\n', '1 2 3\0',
      '1e3 2 3', 'Infinity 2 3', 'NaN 2 3',
      '1000000001 0 0', '1 2 999999999999999999999',
      '1.234,5 6', '1 2 3'.padEnd(129, ' '),
    ]) expect(() => parseUserPastedCoordinates(raw)).toThrow();
  });
});

describe('manual copy output matches each current input and selected mode', () => {
  it('reformats changed input in exact XYZ, negative block XYZ and position-only teleport modes', () => {
    const modes = ['XYZ', 'BLOCK', 'TELEPORT_POSITION_ONLY'] as const;
    const inputs = [
      {raw: '123.456 64 -45.125', expected: [
        '123.456 64 -45.125', '123 64 -46', '/tp @s 123.456 64 -45.125',
      ]},
      {raw: '-0.01 70.999 -16.01', expected: [
        '-0.01 70.999 -16.01', '-1 70 -17', '/tp @s -0.01 70.999 -16.01',
      ]},
      {raw: '0.0000001 -0.0000001 -0', expected: [
        '0.0000001 -0.0000001 0', '0 -1 0', '/tp @s 0.0000001 -0.0000001 0',
      ]},
    ];
    for (const {raw, expected} of inputs) {
      modes.forEach((mode, i) => {
        expect(formatUserPastedCoordinates(raw, mode)).toBe(expected[i]);
      });
    }
  });

  it('refuses partial input and rejected values in every output mode', () => {
    for (const raw of ['12 34', '12 34 ', '12 34 invalid', '12 34 56 78',
      '1,2 3', '/tp @s 1 2 3', '1 2 3\n4']) {
      for (const mode of ['XYZ', 'BLOCK', 'TELEPORT_POSITION_ONLY'] as const) {
        expect(() => formatUserPastedCoordinates(raw, mode)).toThrow();
      }
    }
  });
});
