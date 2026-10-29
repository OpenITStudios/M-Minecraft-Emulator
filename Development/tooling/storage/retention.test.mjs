import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,symlinkSync,linkSync,rmSync,existsSync,statSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {planRetention,renderPlan} from './retention.mjs';

const NOW=Date.UTC(2026,9,30,12);
const dates=['2026-07-01T12:00:00.000Z','2026-08-01T12:00:00.000Z',
  '2026-09-01T12:00:00.000Z','2026-10-28T12:00:00.000Z'];
const archive='workspace/archive/compiler-cache-20261026/target/debug/object.o';

function fixture(fn){const root=mkdtempSync(join(tmpdir(),'lazy-retention-'));try{return fn(root);}finally{rmSync(root,{recursive:true,force:true});}}
function put(root,path,contents){const file=resolve(root,path);mkdirSync(resolve(file,'..'),{recursive:true});writeFileSync(file,contents);return file;}
function build(root,date,number,options={}) {
  const version='0.1.0';
  const commit=number.toString(16).padStart(40,'0');
  const stamp=new Date(date).toISOString().replace(/[-:.]/g,'');
  const folder=`workspace/packages/lazy-tester/${version}-${commit.slice(0,12)}-${stamp}`;
  const name=`Lazy-Tester-${version}-windows-x86_64-setup.exe`;
  const file=put(root,`${folder}/${name}`,Buffer.concat([Buffer.from('MZ'),Buffer.alloc(126,number)]));
  const bytes=readFileSync(file);
  const hash=createHash('sha256').update(bytes).digest('hex');
  const manifest={schema:1,product:'Lazy Tester',version,sourceCommit:commit,sourceBranch:'Local',
    classification:'DEVELOPMENT_CANDIDATE',releaseReady:false,
    buildCommand:'npm --prefix Development/apps/desktop run build:app',
    installer:{name,sha256:hash,bytes:bytes.length},
    signing:'NOT_VERIFIED',installedAcceptance:'NOT_EXECUTED',
    minecraftAcceptance:'NOT_EXECUTED',publication:'DISABLED',...options.manifest};
  put(root,`${folder}/package-manifest.json`,JSON.stringify(manifest)+'\n');
  put(root,`${folder}/SHA256SUMS.txt`,`${hash}  ${name}\n`);
  return {folder,name,filename:file,manifest};
}
function expectStatus(plan,name,status){const row=plan.packageRows.find(p=>p.name===name.split('/').at(-1));assert.ok(row,'candidate not found');assert.equal(row.status,status);return row;}

test('empty checkout yields no candidates and no created directories',()=>fixture(root=>{
  const report=planRetention(root,{nowMs:NOW});
  assert.equal(report.complete,true);
  assert.equal(report.packageStatus,'ABSENT');
  assert.equal(report.archive.status,'ABSENT');
  assert.equal(report.packageRows.length,0);
  assert.equal(existsSync(join(root,'workspace')),false);
  assert.match(renderPlan(report),/NO DELETE CAPABILITY/);
}));

test('latest three and all recent candidates protected; older fourth is review-only',()=>fixture(root=>{
  const fixtures=dates.map((date,i)=>build(root,date,i+1));
  const result=planRetention(root,{nowMs:NOW});
  assert.equal(result.complete,true);
  assert.equal(result.packageStatus,'OK');
  assert.equal(expectStatus(result,fixtures[0].folder,'REVIEW_ONLY').reason.includes('explicit approval'),true);
  for(const current of fixtures.slice(1))expectStatus(result,current.folder,'PRESERVE');
  assert.equal(result.destructiveCapability,false);
}));

test('even very old exports are preserved when fewer than three exist',()=>fixture(root=>{
  for(let i=0;i<2;i++)build(root,dates[i],i+1);
  const result=planRetention(root,{nowMs:NOW});
  assert.ok(result.packageRows.every(row=>row.status==='PRESERVE'));
}));

test('future-dated export is protected',()=>fixture(root=>{
  const result=build(root,'2027-01-01T00:00:00.000Z',1);
  const report=planRetention(root,{nowMs:NOW});
  assert.equal(expectStatus(report,result.folder,'PRESERVE').reason.includes('Future'),true);
}));

test('manifest with wrong commit and mismatched SHA is blocked',()=>fixture(root=>{
  const target=build(root,dates[0],1,{manifest:{sourceCommit:'f'.repeat(40)}});
  const result=planRetention(root,{nowMs:NOW});
  expectStatus(result,target.folder,'BLOCKED');
}));

test('missing completion manifest is never treated as candidate',()=>fixture(root=>{
  const target=build(root,dates[0],1);
  rmSync(resolve(root,target.folder,'package-manifest.json'));
  const result=planRetention(root,{nowMs:NOW});
  expectStatus(result,target.folder,'BLOCKED');
}));

test('unrecognized fourth file blocks an otherwise valid candidate',()=>fixture(root=>{
  const target=build(root,dates[0],1);
  put(root,`${target.folder}/untrusted.txt`,'no');
  const result=planRetention(root,{nowMs:NOW});
  expectStatus(result,target.folder,'BLOCKED');
}));

test('symlink replacing a receipt is blocked without following private content',t=>fixture(root=>{
  const target=build(root,dates[0],1);
  const outside=put(root,'Data/private/world.json','{"sensitive":true}');
  const receipt=resolve(root,target.folder,'package-manifest.json');
  rmSync(receipt);
  try{symlinkSync(outside,receipt);}catch(e){if(process.platform==='win32'&&e.code==='EPERM'){t.skip('Insufficient Windows symlink privileges');return;}throw e;}
  const plan=planRetention(root,{nowMs:NOW});
  expectStatus(plan,target.folder,'BLOCKED');
  assert.doesNotMatch(renderPlan(plan),/sensitive|private\/world/);
}));

test('hardlinked installer is blocked, not counted as recoverable bytes',()=>fixture(root=>{
  const candidate=build(root,dates[0],1);
  const outside=resolve(root,'workspace/scratch/hardlink');
  mkdirSync(resolve(outside,'..'),{recursive:true});
  linkSync(candidate.filename,outside);
  expectStatus(planRetention(root,{nowMs:NOW}),candidate.folder,'BLOCKED');
}));

test('linked package parent is blocked and never scanned',t=>fixture(root=>{
  const outside=mkdtempSync(join(tmpdir(),'lazy-retention-outside-'));
  try{
    const parent=resolve(root,'workspace/packages');
    mkdirSync(resolve(parent,'..'),{recursive:true});
    try{symlinkSync(outside,parent,process.platform==='win32'?'junction':'dir');}
    catch(e){if(process.platform==='win32'&&e.code==='EPERM'){t.skip('Insufficient Windows junction privileges');return;}throw e;}
    const result=planRetention(root,{nowMs:NOW});
    assert.equal(result.packageStatus,'BLOCKED');
    assert.equal(result.complete,false);
  }finally{rmSync(outside,{recursive:true,force:true});}
}));

test('bounded discovery suppresses review when package list is incomplete',()=>fixture(root=>{
  dates.forEach((date,i)=>build(root,date,i+1));
  const result=planRetention(root,{nowMs:NOW,maxDirectories:2});
  assert.equal(result.packageStatus,'PARTIAL');
  assert.equal(result.complete,false);
  assert.ok(result.packageRows.every(row=>row.status!=='REVIEW_ONLY'));
}));

test('archived compiler cache reports logical bytes but is never cleanup-eligible',()=>fixture(root=>{
  put(root,archive,Buffer.alloc(31));
  put(root,'Data/Launchers/Minecraft-1/world',Buffer.alloc(1024));
  put(root,'workspace/private/local-app-backups/payload',Buffer.alloc(2048));
  const result=planRetention(root,{nowMs:NOW});
  assert.equal(result.archive.status,'REVIEW_ONLY');
  assert.equal(result.archive.bytes,31n);
  assert.equal(result.packageRows.length,0);
  assert.doesNotMatch(renderPlan(result),/Data\/Launchers|local-app-backups/);
}));

test('linked archival contents force BLOCKED instead of REVIEW_ONLY',t=>fixture(root=>{
  const outside=mkdtempSync(join(tmpdir(),'lazy-retention-protected-'));
  try{
    put(root,archive,Buffer.alloc(3));
    const link=resolve(root,'workspace/archive/compiler-cache-20261026/protected');
    try{symlinkSync(outside,link,process.platform==='win32'?'junction':'dir');}
    catch(e){if(process.platform==='win32'&&e.code==='EPERM'){t.skip('Insufficient Windows junction privileges');return;}throw e;}
    const result=planRetention(root,{nowMs:NOW});
    assert.equal(result.archive.status,'BLOCKED');
    assert.equal(result.complete,false);
  }finally{rmSync(outside,{recursive:true,force:true});}
}));

test('retention planner has no mutating filesystem imports or user path traversal',()=>{
  const src=readFileSync(fileURLToPath(new URL('./retention.mjs',import.meta.url)),'utf8');
  assert.doesNotMatch(src,/\b(?:rmSync|rmdirSync|unlinkSync|renameSync|writeFileSync|mkdirSync|execFileSync|spawnSync)\b/);
  assert.doesNotMatch(src,/\b(?:Data\/|workspace\/private|workspace\/runs|App\/).*\.remove\b/);
  assert.match(src,/destructiveCapability:false/);
});

test('--apply and all unexpected modes are rejected with no modified cache',()=>fixture(root=>{
  const candidate=build(root,dates[0],1);
  const before=statSync(candidate.filename).mtimeMs;
  const exe=resolve(fileURLToPath(new URL('./retention.mjs',import.meta.url)));
  const denied=spawnSync(process.execPath,[exe,'--apply'],{encoding:'utf8'});
  assert.equal(denied.status,2);
  assert.match(denied.stderr,/no --apply mode/);
  assert.ok(existsSync(candidate.filename));
  assert.equal(statSync(candidate.filename).mtimeMs,before);
}));

test('corrupt neighbor blocks all retention recommendations, including older valid candidates',()=>fixture(root=>{
  dates.forEach((date,i)=>build(root,date,i+1));
  put(root,'workspace/packages/lazy-tester/unexpected/unknown.bin','x');
  const plan=planRetention(root,{nowMs:NOW});
  assert.equal(plan.packageStatus,'BLOCKED');
  assert.equal(plan.complete,false);
  assert.ok(plan.packageRows.every(p=>p.status==='BLOCKED'));
  assert.doesNotMatch(renderPlan(plan),/unexpected\/unknown\.bin/);
}));
