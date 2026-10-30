import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,symlinkSync,linkSync,rmSync,existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {pruneSupersededCandidates} from './package-recycle.mjs';

const NOW=Date.UTC(2026,9,30,12);
const PARENT='workspace/packages/lazy-tester';
function fixture(run){const root=mkdtempSync(join(tmpdir(),'lazy-package-gc-'));try{return run(root);}finally{rmSync(root,{recursive:true,force:true});}}
function put(root,rel,content){const dest=resolve(root,rel);mkdirSync(resolve(dest,'..'),{recursive:true});writeFileSync(dest,content);return dest;}
function create(root,i,opts={}) {
  const version='0.1.0',sha=i.toString(16).padStart(40,'0');
  const stamp=`202610${String(i+1).padStart(2,'0')}T120000000Z`;
  const name=`${version}-${sha.slice(0,12)}-${stamp}`;
  const folder=`${PARENT}/${name}`,exe=`Lazy-Tester-${version}-windows-x86_64-setup.exe`;
  const bytes=Buffer.concat([Buffer.from('MZ'),Buffer.alloc(180,i)]);
  const hash=createHash('sha256').update(bytes).digest('hex');
  const manifest={schema:1,product:'Lazy Tester',version,sourceCommit:sha,sourceBranch:'Local',
    classification:'DEVELOPMENT_CANDIDATE',releaseReady:false,
    installer:{name:exe,sha256:hash,bytes:bytes.length},signing:'NOT_VERIFIED',
    installedAcceptance:'NOT_EXECUTED',minecraftAcceptance:'NOT_EXECUTED',publication:'DISABLED'};
  put(root,`${folder}/${exe}`,bytes);
  put(root,`${folder}/SHA256SUMS.txt`,`${hash}  ${exe}\n`);
  put(root,`${folder}/package-manifest.json`,JSON.stringify(manifest)+'\n');
  return {name,folder,exe,manifest};
}
function plan(rows,status='OK') {
  const sorted=[...rows].sort((a,b)=>b.i-a.i);
  return () => ({packageStatus:status,packageRows:sorted.map((v,i)=>({
    name:v.obj.name,time:NOW-10_000*(i+1),status:i<2?'PRESERVE':'REVIEW_ONLY',
  }))});
}
function buildRows(root,n=4){return Array.from({length:n},(_,i)=>({i:i+1,obj:create(root,i+1)}));}
function options(rows,more={}){return {planner:plan(rows),nowMs:NOW,...more};}

test('after a verified build retain latest two, remove older complete exports, preserve private data',()=>fixture(root=>{
  const rows=buildRows(root);
  put(root,'Data/Launchers/Minecraft-1/world.json','secret');
  put(root,'workspace/private/local-app-backups/sentinel','private');
  put(root,'workspace/builds/desktop/target/debug/sentinel','compiler');
  const result=pruneSupersededCandidates(root,rows[3].obj.name,options(rows));
  assert.equal(result.status,'PRUNED');
  assert.equal(result.deleted.length,2);
  assert.ok(result.deleted.includes(rows[0].obj.name));
  assert.ok(result.deleted.includes(rows[1].obj.name));
  assert.ok(existsSync(resolve(root,rows[2].obj.folder)));
  assert.ok(existsSync(resolve(root,rows[3].obj.folder)));
  assert.equal(readFileSync(resolve(root,'Data/Launchers/Minecraft-1/world.json'),'utf8'),'secret');
  assert.equal(readFileSync(resolve(root,'workspace/private/local-app-backups/sentinel'),'utf8'),'private');
  assert.equal(readFileSync(resolve(root,'workspace/builds/desktop/target/debug/sentinel'),'utf8'),'compiler');
}));

test('only two candidates means no deletion',()=>fixture(root=>{
  const rows=buildRows(root,2);
  const result=pruneSupersededCandidates(root,rows[1].obj.name,options(rows));
  assert.equal(result.status,'NO_CHANGE');
  assert.deepEqual(result.deleted,[]);
}));

test('tampered newest installer blocks pruning older recoverable versions',()=>fixture(root=>{
  const rows=buildRows(root);
  const current=rows[3].obj;
  put(root,`${current.folder}/${current.exe}`,Buffer.alloc(182,8));
  const result=pruneSupersededCandidates(root,current.name,options(rows));
  assert.equal(result.status,'BLOCKED');
  assert.equal(result.deleted.length,0);
  assert.ok(existsSync(resolve(root,rows[0].obj.folder)));
}));

test('modified old candidate, extra file or incomplete manifest blocks pruning all',()=>fixture(root=>{
  const rows=buildRows(root);
  put(root,`${rows[0].obj.folder}/unexpected.txt`,'user');
  const result=pruneSupersededCandidates(root,rows[3].obj.name,options(rows));
  assert.equal(result.status,'BLOCKED');
  assert.ok(existsSync(resolve(root,rows[0].obj.folder)));
  assert.ok(existsSync(resolve(root,rows[1].obj.folder)));
}));

test('hardlinked installer is protected rather than removed',()=>fixture(root=>{
  const rows=buildRows(root);
  const outside=put(root,'workspace/scratch/marker','');
  rmSync(outside);
  linkSync(resolve(root,rows[0].obj.folder,rows[0].obj.exe),outside);
  const result=pruneSupersededCandidates(root,rows[3].obj.name,options(rows));
  assert.equal(result.status,'BLOCKED');
  assert.ok(existsSync(resolve(root,rows[0].obj.folder)));
}));

test('linked package root is not traversed during deletion',t=>fixture(root=>{
  const external=mkdtempSync(join(tmpdir(),'lazy-package-gc-outside-'));
  try{
    const rows=buildRows(root);
    const target=resolve(root,'workspace/packages');
    rmSync(target,{recursive:true,force:true});
    try{symlinkSync(external,target,process.platform==='win32'?'junction':'dir');}
    catch(e){if(process.platform==='win32'&&e.code==='EPERM'){t.skip('Windows symlink privileges absent');return;}throw e;}
    const result=pruneSupersededCandidates(root,rows[3].obj.name,options(rows));
    assert.equal(result.status,'BLOCKED');
    assert.equal(result.deleted.length,0);
  }finally{rmSync(external,{recursive:true,force:true});}
}));

test('incomplete package planning prevents every deletion',()=>fixture(root=>{
  const rows=buildRows(root);
  const result=pruneSupersededCandidates(root,rows[3].obj.name,{planner:plan(rows,'PARTIAL'),nowMs:NOW});
  assert.equal(result.status,'BLOCKED');
  assert.equal(result.deleted.length,0);
}));

test('non-latest candidate, invalid name and invalid batch size are refused',()=>fixture(root=>{
  const rows=buildRows(root);
  assert.equal(pruneSupersededCandidates(root,rows[1].obj.name,options(rows)).status,'BLOCKED');
  assert.throws(()=>pruneSupersededCandidates(root,'../../Data',options(rows)),/newly exported/);
  assert.throws(()=>pruneSupersededCandidates(root,rows[3].obj.name,options(rows,{maxDeletes:999})),/bounded/);
}));

test('prune is bounded per build and older leftovers are not misrepresented as freed',()=>fixture(root=>{
  const rows=buildRows(root,7);
  const result=pruneSupersededCandidates(root,rows[6].obj.name,options(rows,{maxDeletes:2}));
  assert.equal(result.status,'PRUNED');
  assert.equal(result.deleted.length,2);
  assert.equal(result.remaining,3);
  assert.ok(existsSync(resolve(root,rows[2].obj.folder)));
}));
