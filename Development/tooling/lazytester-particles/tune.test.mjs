import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createMcpack} from './archive.mjs';
import {compileFiles, gitBlobSha, tuneParticle} from './tune.mjs';
import {MANIFEST, PARTICLES} from './catalog.mjs';

const fixture = () => ({
  format_version: '1.26.10', particle_effect: {
    description: {identifier: 'minecraft:example', basic_render_parameters: {
      material: 'particles_alpha', texture: 'textures/particle/particles'}},
    components: {
      'minecraft:emitter_lifetime_expression': {},
      'minecraft:emitter_rate_steady': {spawn_rate: {expression: 'Math.random(1, 4)', version: 0}, max_particles: 20},
      'minecraft:particle_lifetime_expression': {max_lifetime: 5},
      'minecraft:particle_appearance_billboard': {size: [0.15, 0.15], uv: {texture_width: 128}},
    },
  },
});
function source(original = fixture()) {
  const raw = Buffer.from(JSON.stringify(original));
  const spec = {file: 'example.json', identifier: 'minecraft:example', format: '1.26.10', blob: gitBlobSha(raw)};
  return {raw, spec};
}
test('pinned vanilla rate changes only the spawn expression, keeps visual and lifetime fields', () => {
  const {raw,spec} = source();
  const before = JSON.parse(raw.toString());
  const after = tuneParticle(raw,spec);
  const old = before.particle_effect.components['minecraft:emitter_rate_steady'];
  const next = after.particle_effect.components['minecraft:emitter_rate_steady'];
  assert.equal(old.spawn_rate.expression, 'Math.random(1, 4)');
  assert.deepEqual(next, {spawn_rate: {expression: '(Math.random(1, 4)) * (1.0 - 0.20 * query.camera_distance_range_lerp(16, 48))', version: 0}, max_particles: 20});
  delete before.particle_effect.components['minecraft:emitter_rate_steady'];
  delete after.particle_effect.components['minecraft:emitter_rate_steady'];
  assert.deepEqual(after, before);
});
test('numeric spawn rates remain numeric in source but are wrapped as MoLang', () => {
  const f=fixture(); f.particle_effect.components['minecraft:emitter_rate_steady'].spawn_rate=3;
  const {raw,spec}=source(f);
  assert.equal(tuneParticle(raw,spec).particle_effect.components['minecraft:emitter_rate_steady'].spawn_rate,
    '(3) * (1.0 - 0.20 * query.camera_distance_range_lerp(16, 48))');
});
test('outdated, malformed, instant and manual sources fail closed', () => {
  const {raw,spec}=source();
  assert.throws(()=>tuneParticle(raw,{...spec,blob:'0'.repeat(40)}),/fingerprint/);
  for(const effect of ['minecraft:emitter_rate_instant','minecraft:emitter_rate_manual']) {
    const f=fixture();f.particle_effect.components[effect]={max_particles:4};
    const v=source(f);assert.throws(()=>tuneParticle(v.raw,v.spec),/steady/);
  }
  const f=fixture();f.particle_effect.components['minecraft:emitter_rate_steady'].spawn_rate='\n';
  const v=source(f);assert.throws(()=>tuneParticle(v.raw,v.spec),/spawn expression/);
  assert.throws(()=>tuneParticle(raw,spec,{near:40,far:15,maximumReduction:0.2}),/bounds/);
  assert.throws(()=>tuneParticle(raw,spec,{near:16,far:48,maximumReduction:0.5}),/bounds/);
});
test('compilation creates only a manifest and the explicitly selected particle JSON', () => {
  const {raw,spec}=source();const files=compileFiles(new Map([['example.json',raw]]),[spec]);
  assert.deepEqual([...files.keys()],['manifest.json','particles/example.json']);
  assert.equal(JSON.parse(files.get('manifest.json').toString()).header.name,MANIFEST.header.name);
  assert.equal(PARTICLES.length,3);
  const zip=createMcpack(files);
  assert.equal(zip.readUInt32LE(0),0x04034b50);
  assert.equal(zip.readUInt32LE(zip.length-22),0x06054b50);
  assert.equal(zip.readUInt16LE(zip.length-14),2);
  assert.deepEqual(zip,createMcpack(files));
  assert.throws(()=>createMcpack(new Map([['manifest.json','x'],['../outside.json','x']])),/path/);
});
