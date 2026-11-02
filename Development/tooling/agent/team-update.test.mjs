// Source-authored synthetic fixtures. DO NOT EXECUTE without separate approval.
// They do not require an actual team ZIP, Windows, Minecraft or network.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {deflateRawSync} from 'node:zlib';
import {mkdtemp,writeFile,rm,readFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {openTeamZip,sha256} from './team-update-zip.mjs';
import {extractTeamWire,inspectTeamSourceZips} from './team-update.mjs';

const REPO='https://github.com/achmadawdi/minecraft-bedrock-agent-skills.git';
const COMMIT='a'.repeat(40);
const wireA=[
  'export const ANALYTICS_EVENT_ID = "mivubi:get_analytics";',
  'export const ANALYTICS_MARKER = "[ANALYTICS] ";',
  'export const ANALYTICS_MAX_LINE_BYTES = 24 * 1024;',
  'export const ANALYTICS_LINES_PER_TICK = 8;','',
].join('\n');
const wireG=[
  'export const GAME_RESULTS_EVENT_ID = "daigon:get_all_scoring";',
  'export const GAME_RESULTS_MARKER = "[GAME_RESULTS] ";',
  'export const GAME_RESULTS_MAX_LINE_BYTES = 24 * 1024;',
  'export const SCORED_RECORD_TYPES: readonly string[] = ["level_score", "gameplay_result"];','',
].join('\n');
const runtime={
  'analytics/contract.ts':wireA,
  'analytics/analytics-export.ts':'// original export module\n',
  'analytics/module.json':'{"name":"analytics"}\n',
  'game-results/game-results-contract.ts':wireG,
  'game-results/game-results-export.ts':'// original score export\n',
  'game-results/module.json':'{"name":"game-results"}\n',
};

function crc32(data){
  let n=0xffffffff;
  for(const byte of data){
    n^=byte;
    for(let i=0;i<8;i++)n=(n>>>1)^((n&1)?0xedb88320:0);
  }
  return (n^0xffffffff)>>>0;
}
function makeZip(entries,{deflate=true}={}){
  const locals=[],centers=[];let offset=0;
  for(const [name,content] of Object.entries(entries)){
    const filename=Buffer.from(name);
    const raw=Buffer.from(content);
    const packed=deflate?deflateRawSync(raw):raw;
    const method=deflate?8:0;
    const header=Buffer.alloc(30),center=Buffer.alloc(46);
    header.writeUInt32LE(0x04034b50,0);
    header.writeUInt16LE(20,4);
    header.writeUInt16LE(method,8);
    header.writeUInt32LE(crc32(raw),14);
    header.writeUInt32LE(packed.length,18);
    header.writeUInt32LE(raw.length,22);
    header.writeUInt16LE(filename.length,26);
    center.writeUInt32LE(0x02014b50,0);
    center.writeUInt16LE(20,4);
    center.writeUInt16LE(20,6);
    center.writeUInt16LE(method,10);
    center.writeUInt32LE(crc32(raw),16);
    center.writeUInt32LE(packed.length,20);
    center.writeUInt32LE(raw.length,24);
    center.writeUInt16LE(filename.length,28);
    center.writeUInt32LE(offset,42);
    locals.push(header,filename,packed);
    centers.push(center,filename);
    offset+=header.length+filename.length+packed.length;
  }
  const central=Buffer.concat(centers),eocd=Buffer.alloc(22);
  eocd.writeUInt32LE(0x06054b50,0);
  eocd.writeUInt16LE(centers.length/2,8);
  eocd.writeUInt16LE(centers.length/2,10);
  eocd.writeUInt32LE(central.length,12);
  eocd.writeUInt32LE(offset,16);
  return Buffer.concat([...locals,central,eocd]);
}
function synthesize({
  teamChanged={},probeChanged={},teamExtra={},tag='v0.35.0',
  corruptLock=false,foreignProvider=false,
}={}){
  const teamFiles={...runtime,...teamChanged,...teamExtra};
  const probeFiles={...runtime,...probeChanged};
  const team={},probe={};
  for(const [rel,value] of Object.entries(teamFiles))
    team['team-v36/modules/'+rel]=value;
  for(const [rel,value] of Object.entries(probeFiles))
    probe['probe-v36/packs/BP/scripts/modules/'+rel]=value;
  const locked={modules:{}};
  for(const mod of ['analytics','game-results']){
    const files={};
    for(const [rel,value] of Object.entries(probeFiles)){
      if(!rel.startsWith(mod+'/'))continue;
      files['packs/BP/scripts/modules/'+rel]=sha256(Buffer.from(value));
    }
    locked.modules[mod]={
      repo:REPO,tag,commit:COMMIT,source:'git',files,
    };
  }
  if(foreignProvider)locked.modules.analytics.repo=
    'https://github.com/example/other-modules.git';
  if(corruptLock){
    locked.modules.analytics.files[
      'packs/BP/scripts/modules/analytics/analytics-export.ts'
    ]='f'.repeat(64);
  }
  probe['probe-v36/packs/BP/manifest.json']=JSON.stringify({
    format_version:2,header:{min_engine_version:[1,26,30]},
    dependencies:[{module_name:'@minecraft/server',version:'2.8.0'}],
  });
  probe['probe-v36/package.json']=JSON.stringify({
    name:'synthetic-skills-probe',
    devDependencies:{'@minecraft/server':'2.8.0'},
  });
  probe['probe-v36/modules-lock.json']=JSON.stringify(locked);
  probe['probe-v36/skills-lock.json']=JSON.stringify({
    repo:REPO,tag,commit:COMMIT,source:'git',targets:['.agents/skills'],
  });
  probe['probe-v36/packs/BP/scripts/probe/runner/runner.ts']=
    'if(event.id==="probe:list")return;\n'+
    'if(event.id==="probe:run")return;\n'+
    'if(event.id==="probe:report")return;\n';
  return {team,probe};
}
const emptyBaseline={
  schema:1,purpose:'SYNTHETIC_PIN',reviewState:'APPROVED_SOURCE_REVIEW',
  provenance:{
    moduleRepo:REPO,upstreamTag:'v0.35.0',upstreamCommit:COMMIT,
    teamArchiveSha256:'0'.repeat(64),probeArchiveSha256:'0'.repeat(64),
  },
  wire:extractTeamWire(Buffer.from(wireA),Buffer.from(wireG)),
  // Synthetic baseline deliberately pins non-contract files so the
  // first generated candidate is reviewable, not a core-schema drift.
  teamRequiredHashes:{
    'analytics/analytics-export.ts':'0'.repeat(64),
    'game-results/game-results-export.ts':'0'.repeat(64),
  },
  probeRequiredHashes:{
    'modules-lock.json':'0'.repeat(64),
    'skills-lock.json':'0'.repeat(64),
    'packs/BP/scripts/probe/runner/runner.ts':'0'.repeat(64),
  },
  probeRuntime:{minEngine:[1,26,30],serverModule:'2.8.0'},
  runtimeTrees:{team:{},probe:{}},
};
async function fixture(files,work){
  const teamPath=path.join(work,'team.zip');
  const probePath=path.join(work,'probe.zip');
  await writeFile(teamPath,makeZip(files.team));
  await writeFile(probePath,makeZip(files.probe));
  return {teamPath,probePath};
}
async function withFolder(fn){
  const dir=await mkdtemp(path.join(tmpdir(),'lazy-team-zip-'));
  try{await fn(dir);}finally{await rm(dir,{recursive:true,force:true});}
}
test('reads ZIP STORE/DEFLATE without extracting and checks CRC',async()=>{
  await withFolder(async root=>{
    const file=path.join(root,'sample.zip');
    await writeFile(file,makeZip({'repo/modules/test.ts':'content\n'}));
    const zip=await openTeamZip(file);
    try{
      assert.equal((await zip.readMember('repo/modules/test.ts')).toString(),'content\n');
      assert.match(await zip.archiveDigest(),/^[a-f0-9]{64}$/);
      await zip.verifyStable();
    }finally{await zip.close();}
    await writeFile(file,makeZip({'repo/modules/test.ts':'content\n'},{deflate:false}));
    const stored=await openTeamZip(file);
    try{assert.equal((await stored.readMember('repo/modules/test.ts')).toString(),'content\n');}
    finally{await stored.close();}
  });
});
test('traversal and corrupted source member are rejected',async()=>{
  await withFolder(async root=>{
    const file=path.join(root,'bad.zip');
    await writeFile(file,makeZip({'repo/../secret.js':'value'}));
    await assert.rejects(()=>openTeamZip(file));
    const valid=makeZip({'team/contract.ts':'hello'});
    const needle=Buffer.from('hello');
    const at=valid.indexOf(Buffer.from('team/contract.ts'))+
      Buffer.byteLength('team/contract.ts');
    assert.ok(at>0);
    // Different raw byte / checksum from central directory must fail.
    await writeFile(file,makeZip({'team/contract.ts':'hello'},{deflate:false}));
    const stable=await openTeamZip(file);
    try{
      const actual=await readFile(file);
      const i=actual.indexOf(needle);
      actual[i]=0x7a;
      await writeFile(file,actual);
      await assert.rejects(()=>stable.readMember('team/contract.ts'));
    }finally{await stable.close();}
  });
});
test('team module/probe locks from identical synthetic snapshots are reviewable',async()=>{
  await withFolder(async root=>{
    const paths=await fixture(synthesize(),root);
    const initial=await inspectTeamSourceZips(paths,emptyBaseline);
    assert.equal(initial.status,'REVIEW_REQUIRED');
    assert.equal(initial.compatibility.moduleLock,'SELF_CONSISTENT');
    assert.equal(initial.compatibility.installedTeamModules,'BYTE_MATCHED');
    assert.equal(initial.compatibility.wire,'PINNED_WIRE_MATCH');
    assert.equal(initial.updatesApplied,false);
    assert.equal(initial.codexUpload,false);
    const approved={...initial.proposedBaseline,
      reviewState:'APPROVED_SOURCE_REVIEW'};
    const same=await inspectTeamSourceZips(paths,approved);
    assert.equal(same.status,'PINNED_SNAPSHOTS_MATCH');
    assert.equal(same.reviewRequired,false);
    assert.equal('proposedBaseline' in same,false);
  });
});
test('team code change before Skills Probe sync blocks the update',async()=>{
  await withFolder(async root=>{
    const paths=await fixture(synthesize({
      teamChanged:{'analytics/analytics-export.ts':'// updated team code\n'},
    }),root);
    const result=await inspectTeamSourceZips(paths,emptyBaseline);
    assert.equal(result.status,'BLOCKED_TEAM_PROBE_DESYNC');
    assert.equal(result.changes.divergentModules.modules.includes('analytics'),true);
    assert.equal(result.compatibility.moduleLock,'SELF_CONSISTENT');
    assert.equal(result.updatesApplied,false);
  });
});
test('contract drift blocks even if team and probe copied matching code',async()=>{
  await withFolder(async root=>{
    const changed=wireA.replace('mivubi:get_analytics','mivubi:future_export');
    const paths=await fixture(synthesize({
      teamChanged:{'analytics/contract.ts':changed},
      probeChanged:{'analytics/contract.ts':changed},
    }),root);
    const result=await inspectTeamSourceZips(paths,emptyBaseline);
    assert.equal(result.status,'BLOCKED_WIRE_CONTRACT_CHANGE');
    assert.equal(result.compatibility.wire,'CHANGED_REVIEW_REQUIRED');
    assert.equal(result.compatibility.moduleLock,'SELF_CONSISTENT');
  });
});
test('documentation-only team ZIP change gets review, not code-desync claim',async()=>{
  await withFolder(async root=>{
    const paths=await fixture(synthesize({teamExtra:{
      'analytics/README.md':'updated docs\n',
    }}),root);
    const v=await inspectTeamSourceZips(paths,emptyBaseline);
    assert.equal(v.status,'REVIEW_REQUIRED');
    assert.equal(v.compatibility.moduleLock,'SELF_CONSISTENT');
    assert.equal(v.compatibility.installedTeamModules,'BYTE_MATCHED');
    assert.equal(v.changes.divergentModules.count,0);
  });
});
test('zip members with huge uncompressed size are blocked without extraction',async()=>{
  await withFolder(async root=>{
    const file=path.join(root,'huge.zip');
    await writeFile(file,makeZip({'test/big.ts':'x'.repeat(131073)}));
    const zip=await openTeamZip(file);
    try{await assert.rejects(()=>zip.readMember('test/big.ts'));}
    finally{await zip.close();}
  });
});

test('a staged candidate is never accepted as an approved baseline',async()=>{
  await withFolder(async root=>{
    const paths=await fixture(synthesize(),root);
    const first=await inspectTeamSourceZips(paths,emptyBaseline);
    assert.equal(first.proposedBaseline.reviewState,'CANDIDATE_NOT_APPROVED');
    await assert.rejects(()=>inspectTeamSourceZips(
      paths,first.proposedBaseline),/INVALID_LOCAL_SOURCE_BASELINE/);
  });
});

test('an installed Skills Probe module with an invalid lock never passes',async()=>{
  await withFolder(async root=>{
    const paths=await fixture(synthesize({corruptLock:true}),root);
    const outcome=await inspectTeamSourceZips(paths,emptyBaseline);
    assert.equal(outcome.status,'BLOCKED_PROBE_MODULE_LOCK');
    assert.equal(outcome.compatibility.moduleLock,'INCONSISTENT');
    assert.ok(outcome.changes.badModuleLocks.modules.includes('analytics'));
    assert.equal('proposedBaseline' in outcome,false);
  });
});
test('contract source hash change requires review even when wire values are equal',async()=>{
  await withFolder(async root=>{
    const original=await fixture(synthesize(),root);
    const initial=await inspectTeamSourceZips(original,emptyBaseline);
    const base={...initial.proposedBaseline,
      reviewState:'APPROVED_SOURCE_REVIEW',
      teamRequiredHashes:{
        ...initial.proposedBaseline.teamRequiredHashes,
        'analytics/contract.ts':sha256(Buffer.from(wireA)),
      }};
    const bumped=wireA+'// schema contract changed but literals unchanged\n';
    const next=await fixture(synthesize({
      teamChanged:{'analytics/contract.ts':bumped},
      probeChanged:{'analytics/contract.ts':bumped},
    }),root);
    const outcome=await inspectTeamSourceZips(next,base);
    assert.equal(outcome.compatibility.wire,'PINNED_WIRE_MATCH');
    assert.equal(outcome.status,'BLOCKED_CONTRACT_SOURCE_REVIEW');
    assert.equal('proposedBaseline' in outcome,false);
  });
});

test('Education Script API dependency version change is a separate blocking gate',async()=>{
  await withFolder(async root=>{
    const paths=await fixture(synthesize(),root);
    const prior=await inspectTeamSourceZips(paths,emptyBaseline);
    const old={...prior.proposedBaseline,
      reviewState:'APPROVED_SOURCE_REVIEW'};
    const source=synthesize();
    const manifest=JSON.parse(source.probe['probe-v36/packs/BP/manifest.json']);
    manifest.dependencies[0].version='2.9.0';
    source.probe['probe-v36/packs/BP/manifest.json']=JSON.stringify(manifest);
    const next=await fixture(source,root);
    const result=await inspectTeamSourceZips(next,old);
    assert.equal(result.status,'BLOCKED_EDUCATION_API_CHANGE');
    assert.equal(result.compatibility.educationTarget,'CHANGED_REVIEW_REQUIRED');
    assert.equal(result.compatibility.wire,'PINNED_WIRE_MATCH');
    assert.equal('proposedBaseline' in result,false);
  });
});

test('team-owned module cannot silently switch to an external provider',async()=>{
  await withFolder(async root=>{
    const paths=await fixture(synthesize({foreignProvider:true}),root);
    const result=await inspectTeamSourceZips(paths,emptyBaseline);
    assert.equal(result.status,'BLOCKED_PROBE_MODULE_LOCK');
    assert.equal(result.compatibility.moduleLock,'INCONSISTENT');
    assert.ok(result.changes.badModuleLocks.modules.includes('analytics'));
  });
});
