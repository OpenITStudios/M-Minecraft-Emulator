import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname} from 'node:path';
import {spawnSync} from 'node:child_process';
import {importFullSandboxie,OWNED_PATH} from './import-full-sandboxie.mjs';
function git(root,...args) {
 const r=spawnSync('git',['-C',root,...args],{encoding:'utf8'});
 assert.equal(r.status,0,`git ${args.join(' ')}: ${r.stderr}`);
 return r.stdout.trim();
}
function makeFixture(){
 const dir=mkdtempSync(join(tmpdir(),'mivubi-vendor-fixture-'));
 const seed=join(dir,'source'),repo=join(dir,'app');mkdirSync(seed);mkdirSync(repo);
 git(seed,'init','-b','master');git(seed,'config','user.name','Fixture');git(seed,'config','user.email','f@example.invalid');
 git(seed,'remote','add','origin','https://github.com/sandboxie-plus/Sandboxie.git');
 for(const file of ['LICENSE.Plus','LICENSE.Classic','Sandboxie/COPYING','SandboxiePlus/QSbieAPI/LICENSE','Sandboxie/core/drv/box.c','Sandboxie/core/svc/DriverAssist.cpp','Sandboxie/core/dll/sbieapi.c','Sandboxie/apps/start/start.cpp','SandboxiePlus/SandMan/Main.cpp','SandboxiePlus/qt-extra/resource.dat']){
  const p=join(seed,file);mkdirSync(dirname(p),{recursive:true});writeFileSync(p,`original ${file}\n`);
 }
 git(seed,'add','.');git(seed,'commit','-m','Pinned upstream fixture');git(seed,'tag','v1.18.5');
 const pin=git(seed,'rev-parse','HEAD');
 git(repo,'init','-b','Local');git(repo,'config','user.name','Fixture');git(repo,'config','user.email','f@example.invalid');
 writeFileSync(join(repo,'README.md'),'MIVUBI application\n');git(repo,'add','.');git(repo,'commit','-m','App baseline');
 const head=git(repo,'rev-parse','HEAD');return {dir,seed,repo,pin,head};
}
test('imports every original upstream tracked source byte as owned editable tree',()=>{
 const f=makeFixture();try{
  const r=importFullSandboxie({repoRoot:f.repo,seedRoot:f.seed,expectedHead:f.head,upstreamCommit:f.pin});
  assert.equal(r.state,'OWNED_SANDBOXIE_FULL_SOURCE_IMPORTED_NOT_BUILT');
  assert.equal(r.sourceTree,r.importedTree);
  assert.equal(r.importedFileCount,10);
  assert.equal(git(f.repo,'rev-parse',`HEAD:${OWNED_PATH}`),git(f.seed,'rev-parse','HEAD^{tree}'));
  // Git verifies imported tree/blob identity; compare text with Windows worktree EOL normalized.
  assert.equal(readFileSync(join(f.repo,OWNED_PATH,'SandboxiePlus/SandMan/Main.cpp'),'utf8').replace(/\r\n/g,'\n'),'original SandboxiePlus/SandMan/Main.cpp\n');
  assert.equal(git(f.repo,'branch','--show-current'),'Local');
  assert.equal(git(f.repo,'show',`${f.head}:README.md`),'MIVUBI application');
  assert.equal(git(f.repo,'status','--porcelain'),'');
  const ownedFile=join(f.repo,OWNED_PATH,'Sandboxie/core/drv/box.c');
  writeFileSync(ownedFile, 'MIVUBI own modification\n');
  assert.equal(readFileSync(join(f.seed,'Sandboxie/core/drv/box.c'),'utf8'),'original Sandboxie/core/drv/box.c\n');
 } finally {rmSync(f.dir,{recursive:true,force:true})}
});
test('refuses dirty Local, bad branch, existing owner directory, or changed HEAD before copying',()=>{
 const f=makeFixture();try{
  writeFileSync(join(f.repo,'stray.txt'),'must not overwrite');
  assert.throws(()=>importFullSandboxie({repoRoot:f.repo,seedRoot:f.seed,expectedHead:f.head,upstreamCommit:f.pin}),/working tree or index is dirty/);
  rmSync(join(f.repo,'stray.txt'));
  assert.throws(()=>importFullSandboxie({repoRoot:f.repo,seedRoot:f.seed,expectedHead:'a'.repeat(40),upstreamCommit:f.pin}),/Local moved/);
  git(f.repo,'checkout','-b','main');
  assert.throws(()=>importFullSandboxie({repoRoot:f.repo,seedRoot:f.seed,expectedHead:f.head,upstreamCommit:f.pin}),/only the Local branch/);
  git(f.repo,'checkout','Local');
  mkdirSync(join(f.repo,OWNED_PATH),{recursive:true});
  assert.throws(()=>importFullSandboxie({repoRoot:f.repo,seedRoot:f.seed,expectedHead:f.head,upstreamCommit:f.pin}),/already exists/);
 } finally {rmSync(f.dir,{recursive:true,force:true})}
});

test('resumes 1-file owned source without replacing it and reaches exact upstream Git tree',()=>{
 const f=makeFixture();try{
  const path='Sandboxie/core/drv/box.c';
  const full=join(f.repo,OWNED_PATH,path);
  mkdirSync(dirname(full),{recursive:true});
  writeFileSync(full,'original '+path+'\n');
  git(f.repo,'add','.');git(f.repo,'commit','-m','Partial pinned source');
  const before=git(f.repo,'rev-parse','HEAD');
  const r=importFullSandboxie({repoRoot:f.repo,seedRoot:f.seed,expectedHead:before,upstreamCommit:f.pin});
  assert.equal(r.importedFileCount,10);
  assert.equal(r.newlyImportedFileCount,9);
  assert.equal(r.importedTree,git(f.seed,'rev-parse','HEAD^{tree}'));
  assert.equal(git(f.repo,'show',before+':'+OWNED_PATH+'/'+path),'original '+path);
  const repeat=importFullSandboxie({repoRoot:f.repo,seedRoot:f.seed,expectedHead:r.appCommit,upstreamCommit:f.pin});
  assert.equal(repeat.newlyImportedFileCount,0);
  assert.equal(repeat.appCommit,r.appCommit);
 } finally {rmSync(f.dir,{recursive:true,force:true})}
});
test('refuses mismatched or unexpected committed owned files before filling gaps',()=>{
 const f=makeFixture();try{
  const path=join(f.repo,OWNED_PATH,'Sandboxie/core/drv/box.c');
  mkdirSync(dirname(path),{recursive:true});writeFileSync(path,'CORRUPT\n');
  git(f.repo,'add','.');git(f.repo,'commit','-m','Wrong partial source');
  const head=git(f.repo,'rev-parse','HEAD');
  assert.throws(()=>importFullSandboxie({repoRoot:f.repo,seedRoot:f.seed,
    expectedHead:head,upstreamCommit:f.pin}),/existing owned source differs/);
  assert.equal(git(f.repo,'rev-parse','HEAD'),head);
  assert.equal(git(f.repo,'status','--porcelain'),'');
 }finally{rmSync(f.dir,{recursive:true,force:true})}
});
