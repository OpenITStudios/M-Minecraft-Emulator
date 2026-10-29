import {test} from 'node:test';
import assert from 'node:assert/strict';
import {inflateRawSync} from 'node:zlib';
import {createMcpack} from './archive.mjs';
import {compileFiles, emissionAtDistance, gitBlobSha, molangLod, tuneParticle} from './tune.mjs';
import {LOD, MANIFEST, PARTICLES} from './catalog.mjs';

function example(mode = 'steady') {
  const emission = mode === 'steady'
    ? {'minecraft:emitter_rate_steady': {spawn_rate: {expression: 'Math.random(1, 4)', version: 0}, max_particles: 20}}
    : {'minecraft:emitter_rate_manual': {max_particles: 2000}};
  return {format_version: '1.26.10', particle_effect: {
    description: {identifier: 'minecraft:test_ambient', basic_render_parameters: {
      material: 'particles_alpha', texture: 'textures/particle/particles'}},
    components: {
      'minecraft:emitter_lifetime_expression': {},
      ...emission,
      'minecraft:particle_lifetime_expression': {max_lifetime: 5},
      'minecraft:particle_appearance_billboard': {size: [0.15, 0.15], uv: {texture_width: 128}},
      'minecraft:particle_motion_dynamic': {linear_acceleration: [0, 1, 0]},
    },
  }};
}
function fixture(mode='steady', data=example(mode)) {
  const raw = Buffer.from(JSON.stringify(data));
  const spec = {file:'test_ambient.json', identifier:'minecraft:test_ambient', format:'1.26.10', mode, blob:gitBlobSha(raw)};
  return {raw,spec};
}
function withoutEmissionFields(data, mode) {
  const copy=structuredClone(data);
  const fields=mode==='steady'?['spawn_rate','max_particles']:['max_particles'];
  for(const field of fields) delete copy.particle_effect.components[`minecraft:emitter_rate_${mode}`][field];
  return copy;
}

test('near/medium/far/very-far tiers and continuous interpolation are exact', () => {
  assert.deepEqual(LOD.rates,[0.75,0.50,0.25,0]);
  for(const [distance,expected] of [
    [0,0.75],[1,0.75],[16,0.75],[24,0.625],
    [32,0.5],[40,0.375],[48,0.25],[56,0.125],[64,0],[500,0],
  ]) assert.ok(Math.abs(emissionAtDistance(distance)-expected)<1e-12, String(distance));
  for(const distance of [0,4,16,20,32,44,50,63,64,128])
    assert.ok(emissionAtDistance(distance)>=0 && emissionAtDistance(distance)<=0.75);
  for(const distance of [-1,NaN,Infinity]) assert.throws(()=>emissionAtDistance(distance),/distance/);
  for(const invalid of [
    {...LOD,near:48,medium:32},
    {...LOD,veryFar:32},
    {...LOD,rates:[0.75,0.5,0.25,0.01]},
    {...LOD,rates:[0.75,0.5,0.5,0]},
  ]) assert.throws(()=>emissionAtDistance(16,invalid),/profile/);
});

test('steady particle tuning adjusts spawn rate AND live cap, preserving appearance', () => {
  const {raw,spec}=fixture();
  const before=JSON.parse(raw.toString());
  const after=tuneParticle(raw,spec);
  const rate=after.particle_effect.components['minecraft:emitter_rate_steady'].spawn_rate;
  assert.equal(rate.version,0);
  assert.equal(rate.expression, '(Math.random(1, 4)) * (0.75 * (1.0 - query.camera_distance_range_lerp(16, 64)))');
  assert.equal((rate.expression.match(/query\.camera_distance_range_lerp/g)||[]).length,1);
  assert.deepEqual(withoutEmissionFields(before,'steady'), withoutEmissionFields(after,'steady'));
  assert.equal(after.particle_effect.components['minecraft:emitter_rate_steady'].max_particles,
    '(20) * (0.75 * (1.0 - query.camera_distance_range_lerp(16, 64)))');
});

test('numeric steady source is transformed without creating textures or extra components', () => {
  const data=example();data.particle_effect.components['minecraft:emitter_rate_steady'].spawn_rate=3;
  const {raw,spec}=fixture('steady',data);
  const after=tuneParticle(raw,spec);
  assert.match(after.particle_effect.components['minecraft:emitter_rate_steady'].spawn_rate,/^\(3\) \* \(0\.75/);
  assert.deepEqual(withoutEmissionFields(data,'steady'),withoutEmissionFields(after,'steady'));
});

test('manual ambient tuning scales only active particle cap, not event emission frequency', () => {
  const {raw,spec}=fixture('manual');
  const before=JSON.parse(raw.toString());
  const after=tuneParticle(raw,spec);
  const cap=after.particle_effect.components['minecraft:emitter_rate_manual'].max_particles;
  assert.equal(cap,'(2000) * (0.75 * (1.0 - query.camera_distance_range_lerp(16, 64)))');
  assert.deepEqual(withoutEmissionFields(before,'manual'), withoutEmissionFields(after,'manual'));
  assert.equal(after.particle_effect.components['minecraft:emitter_rate_steady'],undefined);
  assert.equal(after.particle_effect.components['minecraft:emitter_rate_manual'].spawn_rate,undefined);
});

test('wrong fingerprint, wrong mode, missing or ambiguous rate family, malformed MoLang fail closed', () => {
  const {raw,spec}=fixture();
  assert.throws(()=>tuneParticle(raw,{...spec,blob:'0'.repeat(40)}),/fingerprint/);
  assert.throws(()=>tuneParticle(raw,{...spec,identifier:'minecraft:other'}),/contract/);
  assert.throws(()=>tuneParticle(raw,{...spec,mode:'manual'}),/family/);
  for(const [mode,data] of [
    ['steady',(()=>{const v=example();v.particle_effect.components['minecraft:emitter_rate_instant']={num_particles:2};return v;})()],
    ['steady',(()=>{const v=example();delete v.particle_effect.components['minecraft:emitter_rate_steady'].max_particles;return v;})()],
    ['steady',(()=>{const v=example();v.particle_effect.components['minecraft:emitter_rate_steady'].spawn_rate={expression:'bad\nsyntax',version:0};return v;})()],
    ['steady',(()=>{const v=example();v.particle_effect.components['minecraft:emitter_rate_steady'].spawn_rate=-1;return v;})()],
    ['manual',(()=>{const v=example('manual');v.particle_effect.components['minecraft:emitter_rate_manual'].max_particles=0;return v;})()],
    ['manual',(()=>{const v=example('manual');v.particle_effect.components['minecraft:emitter_rate_manual'].max_particles=1.2;return v;})()],
  ]) {const q=fixture(mode,data);assert.throws(()=>tuneParticle(q.raw,q.spec));}
});

test('compiles a mix of steady/manual without unknown paths or Mojang texture files', () => {
  const steady=fixture();
  const manualData=example('manual');
  manualData.particle_effect.description.identifier='minecraft:manual_ambient';
  const manual=fixture('manual',manualData);
  const otherSpec={...manual.spec,file:'manual_ambient.json',identifier:'minecraft:manual_ambient'};
  const files=compileFiles(new Map([
    [steady.spec.file,steady.raw],['manual_ambient.json',manual.raw]
  ]),[steady.spec,otherSpec]);
  assert.deepEqual([...files.keys()],['manifest.json','particles/test_ambient.json','particles/manual_ambient.json']);
  assert.equal(JSON.parse(files.get('manifest.json')).header.name,MANIFEST.header.name);
  assert.deepEqual(JSON.parse(files.get('manifest.json')).header.version,[0,3,0]);
  assert.deepEqual(PARTICLES.reduce((n,p)=>{n[p.mode]=(n[p.mode]??0)+1;return n;},{}),{steady:3,manual:17});
  for(const path of files.keys()) assert.ok(path.endsWith('.json'));
  const zip=createMcpack(files);
  assert.equal(zip.readUInt32LE(0),0x04034b50);
  assert.equal(zip.readUInt32LE(zip.length-22),0x06054b50);
  assert.equal(zip.readUInt16LE(zip.length-14),3);
  assert.deepEqual(zip,createMcpack(files));
  assert.throws(()=>createMcpack(new Map([['manifest.json','x'],['../outside.json','x']])),/path/);
  assert.throws(()=>compileFiles(new Map(),[steady.spec]),/Missing pinned/);
  assert.throws(()=>compileFiles(new Map([[steady.spec.file,steady.raw]]),[steady.spec,steady.spec]),/Duplicate/);
  assert.throws(()=>compileFiles(new Map(),Array(32).fill(steady.spec)),/size/);
});

import {spawnSync} from 'node:child_process';
import {mkdtempSync, mkdirSync, writeFileSync, existsSync, rmSync} from 'node:fs';
import {join} from 'node:path';
import {tmpdir} from 'node:os';
import {fileURLToPath} from 'node:url';

test('builder never writes output on explicit-download refusal or untrusted local source', () => {
  const root=mkdtempSync(join(tmpdir(),'lazytester-particle-'));
  const script=fileURLToPath(new URL('./build.mjs',import.meta.url));
  try {
    const output=join(root,'opt-in.mcpack');
    const refused=spawnSync(process.execPath,[script,
      '--official','no','--output',output],{encoding:'utf8'});
    assert.equal(refused.status,1);
    assert.match(refused.stderr,/explicit --official yes/);
    assert.equal(existsSync(output),false);
    const source=join(root,'resource_pack');
    mkdirSync(join(source,'particles'),{recursive:true});
    writeFileSync(join(source,'particles','portal_reverse.json'),'x'.repeat(128*1024+1));
    const oversized=spawnSync(process.execPath,[script,
      '--source',source,'--output',output],{encoding:'utf8'});
    assert.equal(oversized.status,1);
    assert.match(oversized.stderr,/Unsupported source size/);
    assert.equal(existsSync(output),false);
  } finally {rmSync(root,{recursive:true,force:true});}
});

test('one camera-distance query replaces three while preserving all four tier values', () => {
  assert.equal(LOD.near,16);
  assert.equal(LOD.veryFar,64);
  assert.deepEqual(LOD.rates,[0.75,0.5,0.25,0]);
  assert.equal(PARTICLES.length,20);
  const expression=molangLod();
  assert.equal(expression, '0.75 * (1.0 - query.camera_distance_range_lerp(16, 64))');
  assert.equal((expression.match(/query\.camera_distance_range_lerp/g)||[]).length,1);
  for (let d=0;d<100;d+=0.25) {
    const clamp = Math.min(1,Math.max(0,(d-16)/48));
    assert.ok(Math.abs(emissionAtDistance(d)-.75*(1-clamp))<1e-12, String(d));
  }
  const custom={near:16,medium:24,far:40,veryFar:80,rates:[.75,.55,.27,0]};
  assert.equal((molangLod(custom).match(/query\.camera_distance_range_lerp/g)||[]).length,3);
});
test('all 20 catalog entries produce a deterministic, compressed 21-entry .mcpack', () => {
  const catalog=[],sources=new Map();
  for (const spec of PARTICLES) {
    const v=example(spec.mode);
    v.particle_effect.description.identifier=spec.identifier;
    const raw=Buffer.from(JSON.stringify(v));
    sources.set(spec.file,raw);
    catalog.push({...spec,blob:gitBlobSha(raw)});
  }
  const files=compileFiles(sources,catalog);
  assert.equal(files.size,21);
  assert.match(MANIFEST.header.description, /3 steady rates; 17 conditional manual caps/);
  assert.equal(new Set(PARTICLES.map(s=>s.identifier)).size,20);
  const zip=createMcpack(files);
  assert.deepEqual(zip,createMcpack(files));
  assert.equal(zip.readUInt16LE(zip.length-14),21);
  // All large JSON entries should use ZIP DEFLATE method 8, preserving bytes.
  assert.equal(zip.readUInt32LE(0),0x04034b50);
  assert.equal(zip.readUInt16LE(8),8);
  const nameLen=zip.readUInt16LE(26),packedLen=zip.readUInt32LE(18);
  const decoded=inflateRawSync(zip.subarray(30+nameLen,30+nameLen+packedLen));
  const filename=zip.subarray(30,30+nameLen).toString();
  assert.deepEqual(decoded,files.get(filename));
});
test('first unrecognized local source aborts before output even if later sources are missing',()=>{
  const root=mkdtempSync(join(tmpdir(),'lazytester-particle-sha-'));
  const script=fileURLToPath(new URL('./build.mjs',import.meta.url));
  try {
    const output=join(root,'no-output.mcpack'),source=join(root,'source');
    mkdirSync(join(source,'particles'),{recursive:true});
    writeFileSync(join(source,'particles','portal_reverse.json'),'{}');
    const result=spawnSync(process.execPath,[script,'--source',source,'--output',output],{encoding:'utf8'});
    assert.equal(result.status,1);
    assert.match(result.stderr,/fingerprint/);
    assert.equal(existsSync(output),false);
  }finally{rmSync(root,{recursive:true,force:true});}
});

test('builder bounds streamed official response even without Content-Length', async()=>{
  const original=globalThis.fetch;
  const root=mkdtempSync(join(tmpdir(),'lazytester-stream-'));
  try {
    let requested=0;
    globalThis.fetch=async()=>{
      requested++;
      return new Response(new ReadableStream({start(ctrl){
        ctrl.enqueue(new Uint8Array(100_000));
        ctrl.enqueue(new Uint8Array(100_000));
        ctrl.close();
      }}),{status:200});
    };
    const {build}=await import('./build.mjs');
    await assert.rejects(build(['--official','yes','--output',join(root,'rejected.mcpack')]),/Oversized vanilla source/);
    assert.equal(requested,1);
    assert.equal(existsSync(join(root,'rejected.mcpack')),false);
  }finally{globalThis.fetch=original;rmSync(root,{recursive:true,force:true});}
});

test('ZIP output refuses path traversal and all output entries are compressed JSON',()=>{
  const source=fixture(),data=compileFiles(new Map([[source.spec.file,source.raw]]),[source.spec]);
  const zip=createMcpack(data);
  assert.equal(zip.readUInt16LE(8),8);
  assert.throws(()=>createMcpack(new Map([['manifest.json','{}'],['../bad.json','{}']])),/Unsafe pack path/);
});
