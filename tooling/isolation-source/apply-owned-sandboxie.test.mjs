import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,copyFileSync,rmSync,readFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {spawnSync} from 'node:child_process';
import {pathToFileURL} from 'node:url';
function git(root,...args){const r=spawnSync('git',['-C',root,...args],{encoding:'utf8'});assert.equal(r.status,0,r.stderr);return r.stdout.trim()}
function put(root,p,content){const name=join(root,p);mkdirSync(dirname(name),{recursive:true});writeFileSync(name,content)}
function makeFixture(){
 const folder=mkdtempSync(join(tmpdir(),'mivubi-owned-git-'));
 const app=join(folder,'app'),source=join(folder,'source');mkdirSync(app);mkdirSync(source);
 git(source,'init','-b','master');git(source,'config','core.autocrlf','false');git(source,'config','user.name','Fixture');git(source,'config','user.email','fixture@example.invalid');
 git(source,'remote','add','origin','https://github.com/sandboxie-plus/Sandboxie.git');
 for (const p of ['LICENSE.Plus','LICENSE.Classic','Sandboxie/COPYING',
 'SandboxiePlus/QSbieAPI/LICENSE','Sandboxie/core/drv/box.c',
 'Sandboxie/core/svc/DriverAssist.cpp','Sandboxie/core/dll/sbieapi.c',
 'Sandboxie/apps/start/start.cpp','SandboxiePlus/SandMan/main.cpp']) put(source,p,'original '+p+'\n');
 git(source,'add','.');git(source,'commit','-m','Fixture upstream tag');git(source,'tag','v1.18.5');
 const pin=git(source,'rev-parse','HEAD');
 git(app,'init','-b','Local');git(app,'config','core.autocrlf','false');git(app,'config','user.name','Fixture');git(app,'config','user.email','fixture@example.invalid');
 for(const s of ['bootstrap-fork.mjs','import-full-sandboxie.mjs','apply-owned-sandboxie.mjs','git-directory-identity.mjs']){
 const dst=join(app,'tooling/isolation-source',s);mkdirSync(dirname(dst),{recursive:true});copyFileSync(new URL('./'+s,import.meta.url),dst);
 }
 put(app,'engine/isolation-native/fork-overlay/apply-private-source.mjs',`
export const SOURCE_FILES = ['Sandboxie/core/drv/box.c'];
const path = SOURCE_FILES[0], needle='original Sandboxie/core/drv/box.c';
export function buildPrivateSourcePatch(values){
 if(values[path]!==needle+'\\n')throw Error('unexpected source');
 return 'diff --git a/'+path+' b/'+path+'\\n--- a/'+path+'\\n+++ b/'+path+'\\n@@ -1,1 +1,2 @@\\n '+needle+'\\n+// MIVUBI OWN MODIFICATION\\n';
}
export function buildExpectedPrivateSources(values){return {[path]:values[path]+'// MIVUBI OWN MODIFICATION\\n'}}
export function verifyExactPrivateSourceBytes(up,edited){if(edited[path]!==buildExpectedPrivateSources(up)[path])throw Error('modified source differs')}
`);
 put(app,'README.md','MIVUBI app preserved\n');git(app,'add','.');git(app,'commit','-m','App baseline');
 const initial=git(app,'rev-parse','HEAD');
 return {folder,app,source,pin,initial};
}
test('full tagged subtree import then direct native change is independently editable',async()=>{
 const f=makeFixture();try{
  const imports=await import(pathToFileURL(join(f.app,'tooling/isolation-source/import-full-sandboxie.mjs')).href);
  const build=await import(pathToFileURL(join(f.app,'tooling/isolation-source/apply-owned-sandboxie.mjs')).href);
  const imported=imports.importFullSandboxie({repoRoot:f.app,seedRoot:f.source,expectedHead:f.initial,upstreamCommit:f.pin});
  assert.equal(imported.importedTree,git(f.source,'rev-parse','HEAD^{tree}'));
  const applied=build.applyOwnedSandboxie({repoRoot:f.app,seedRoot:f.source,expectedHead:imported.appCommit,upstreamCommit:f.pin});
  assert.equal(applied.modifiedSourceCount,1);
  assert.equal(applied.driverInstalled,false);
  assert.equal(readFileSync(join(f.app,'engine/isolation-owned/Sandboxie/core/drv/box.c'),'utf8'),
    'original Sandboxie/core/drv/box.c\n// MIVUBI OWN MODIFICATION\n');
  assert.equal(readFileSync(join(f.source,'Sandboxie/core/drv/box.c'),'utf8'),
    'original Sandboxie/core/drv/box.c\n');
  assert.equal(git(f.app,'status','--porcelain'),'');
  const changed=git(f.app,'diff','--name-only',imported.appCommit,applied.appCommit);
  assert.equal(changed,'engine/isolation-owned/Sandboxie/core/drv/box.c');
  assert.equal(git(f.app,'show',`${f.initial}:README.md`),'MIVUBI app preserved');
 } finally {rmSync(f.folder,{recursive:true,force:true})}
});
test('never reapply patch to modified source or apply without pinned snapshot',async()=>{
 const f=makeFixture();try{
  const imports=await import(pathToFileURL(join(f.app,'tooling/isolation-source/import-full-sandboxie.mjs')).href);
  const apply=await import(pathToFileURL(join(f.app,'tooling/isolation-source/apply-owned-sandboxie.mjs')).href);
  assert.throws(()=>apply.applyOwnedSandboxie({repoRoot:f.app,seedRoot:f.source,expectedHead:f.initial,upstreamCommit:f.pin}),
  /full owned source must be imported first/);
  const imported=imports.importFullSandboxie({repoRoot:f.app,seedRoot:f.source,expectedHead:f.initial,upstreamCommit:f.pin});
  const applied=apply.applyOwnedSandboxie({repoRoot:f.app,seedRoot:f.source,expectedHead:imported.appCommit,upstreamCommit:f.pin});
  assert.throws(()=>apply.applyOwnedSandboxie({repoRoot:f.app,seedRoot:f.source,expectedHead:applied.appCommit,upstreamCommit:f.pin}),
   /not the complete unmodified pinned Sandboxie snapshot/);
 }finally{rmSync(f.folder,{recursive:true,force:true})}
});
