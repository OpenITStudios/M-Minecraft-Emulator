// Authored-only regression cases. No Minecraft runtime or Add-On install.
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {formatPlayerDebugSnapshot} from './format-player-debug.mjs';

const sample = Object.freeze({
  location: {x: -385.25, y: 64.875, z: -60.1},
  rotation: {x: -12.5, y: 180},
  dimensionId: 'minecraft:overworld',
});
test('projects real player-supplied XYZ, negative floor, chunk and facing', () => {
  const actual = formatPlayerDebugSnapshot(sample);
  assert.deepEqual(actual.lines, [
    'Minecraft Education | Debug',
    'XYZ: -385.25 / 64.88 / -60.10',
    'Block: -386 / 64 / -61',
    'Chunk: -25 / -4',
    'Within chunk: 14 / 3',
    'Facing: North (-Z)',
    'Rotation: yaw 180.00 / pitch -12.50',
    'Dimension: Overworld',
  ]);
  assert.equal(actual.copyCoordinatesText, '-386 64 -61');
  assert.ok(Object.isFrozen(actual) && Object.isFrozen(actual.lines));
});
test('cardinal directions follow Bedrock yaw rather than Java assumptions', () => {
  for (const [yaw, label] of [
    [0, 'South (+Z)'],[90, 'West (-X)'],
    [180,'North (-Z)'],[-90,'East (+X)'],[359,'South (+Z)'],
  ]) {
    const result = formatPlayerDebugSnapshot({...sample,rotation:{x:0,y:yaw}});
    assert.equal(result.lines[5], 'Facing: '+label);
  }
});
test('handles negative chunk boundaries and known dimensions', () => {
  const edge = formatPlayerDebugSnapshot({
    location:{x:-16.001,y:-1,z:16},
    rotation:{x:0,y:90},
    dimensionId:'minecraft:nether',
  });
  assert.equal(edge.lines[2],'Block: -17 / -1 / 16');
  assert.equal(edge.lines[3],'Chunk: -2 / 1');
  assert.equal(edge.lines[4],'Within chunk: 15 / 0');
  assert.equal(edge.lines[7],'Dimension: Nether');
  assert.equal(edge.copyCoordinatesText,'-17 -1 16');
  const end = formatPlayerDebugSnapshot({...sample,dimensionId:'minecraft:the_end'});
  assert.equal(end.lines[7],'Dimension: The End');
});
test('fails closed on absent/invalid, unbounded or non-player context', () => {
  const invalid = [
    {...sample,location:{x:NaN,y:0,z:0}},
    {...sample,location:{x:Infinity,y:0,z:0}},
    {...sample,location:{x:'12',y:0,z:0}},
    {...sample,location:{x:30_000_001,y:0,z:0}},
    {...sample,rotation:{x:95,y:0}},
    {...sample,rotation:{x:0,y:NaN}},
    {...sample,dimensionId:'minecraft:unknown'},
    {...sample,dimensionId:'\u00a7cForged'},
    {...sample,location:null},
    {...sample,rotation:null},
  ];
  for (const value of invalid) {
    assert.throws(() => formatPlayerDebugSnapshot(value), TypeError);
  }
});
test('projection has no F3 key listener, overlay renderer or clipboard capability', () => {
  const sampleResult = formatPlayerDebugSnapshot(sample);
  assert.deepEqual(Object.keys(sampleResult), ['lines','copyCoordinatesText']);
  assert.equal(sampleResult.lines.some(line => /fps|tps|clipboard/i.test(line)),false);
});
