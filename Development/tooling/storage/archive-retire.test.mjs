import test from 'node:test';
import assert from 'node:assert/strict';
import {existsSync, linkSync, mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname, join, resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {ARCHIVE, inspectArchivedCargo, retireArchivedCargo} from './archive-retire.mjs';

function fixture(fn){
  const root=mkdtempSync(join(tmpdir(),'lazy-archive-retire-'));
  try{return fn(root);}finally{rmSync(root,{recursive:true,force:true});}
}
function put(root,path,bytes='Cargo compiler artifact'){
  const target=resolve(root,path);
  mkdirSync(dirname(target),{recursive:true});
  writeFileSync(target,bytes);
  return target;
}
function ready(root){
  put(root,'workspace/builds/desktop/target/x86_64-pc-windows-msvc/debug/deps/current.rmeta','current');
  put(root,`${ARCHIVE}/debug/deps/obsolete.rmeta`,'obsolete');
  put(root,'workspace/toolchains/cargo/bin/cargo.exe','synthetic-cargo-not-executable');
  put(root,'Development/toolchain.json',JSON.stringify({rust:{version:'1.99.1'}}));
  put(root,'Development/apps/desktop/src-tauri/Cargo.toml','[package]\nname="synthetic"\n');
}

test('no historical compiler archive is a harmless no-op',()=>fixture(root=>{
  assert.equal(inspectArchivedCargo(root).status,'ABSENT');
  assert.equal(retireArchivedCargo(root).status,'ABSENT');
  assert.equal(existsSync(join(root,'workspace')),false);
}));

test('old Cargo archive with a verified replacement Windows target is reviewable',()=>fixture(root=>{
  ready(root);
  const data=inspectArchivedCargo(root);
  assert.equal(data.status,'REVIEWABLE');
  assert.equal(data.logicalBytes,String(Buffer.byteLength('obsolete')));
}));

test('retirement uses only pinned Cargo clean and the fixed historical target',()=>fixture(root=>{
  ready(root);
  let called=0;
  const result=retireArchivedCargo(root,{silent:true,execute:(binary,args,opts)=>{
    if(args[0]==='--version')return {status:0,stdout:'cargo 1.99.1 (fixture)'};
    called++;
    assert.equal(binary,resolve(root,'workspace/toolchains/cargo/bin/cargo.exe'));
    assert.deepEqual(args.slice(0,3),['clean','--locked','--manifest-path']);
    assert.equal(args.at(-1),resolve(root,ARCHIVE));
    assert.equal(args.at(-2),'--target-dir');
    assert.equal(opts.env.CARGO_TARGET_DIR,resolve(root,ARCHIVE));
    assert.equal(opts.env.CARGO_BUILD_BUILD_DIR,resolve(root,ARCHIVE));
    assert.equal(opts.shell,false);
    rmSync(resolve(root,ARCHIVE),{recursive:true,force:true}); // Synthetic fixture ONLY.
    return {status:0};
  }});
  assert.equal(called,1);
  assert.equal(result.status,'RETIRED');
  assert.equal(readFileSync(resolve(root,'workspace/builds/desktop/target/x86_64-pc-windows-msvc/debug/deps/current.rmeta'),'utf8'),'current');
}));

test('no verified current output blocks retired cache disposal',()=>fixture(root=>{
  ready(root);
  rmSync(resolve(root,'workspace/builds/desktop/target'),{recursive:true,force:true});
  let called=false;
  const result=retireArchivedCargo(root,{execute:()=>{called=true;return {status:0};}});
  assert.equal(result.status,'BLOCKED');
  assert.equal(called,false);
  assert.equal(existsSync(resolve(root,ARCHIVE)),true);
}));

test('unrecognized archive sibling is preserved and prevents cleaning',()=>fixture(root=>{
  ready(root);
  put(root,'workspace/archive/compiler-cache-20261026/custom-notes.txt','do not touch');
  const result=retireArchivedCargo(root);
  assert.equal(result.status,'BLOCKED');
  assert.equal(existsSync(resolve(root,ARCHIVE)),true);
}));

test('ordinary archive target containing a hardlinked file cannot be retired',()=>fixture(root=>{
  ready(root);
  const outside=put(root,'workspace/private/other-hardlink-owner','protected');
  linkSync(outside,resolve(root,`${ARCHIVE}/debug/deps/hardlink.bin`));
  const result=retireArchivedCargo(root);
  assert.equal(result.status,'BLOCKED');
  assert.equal(existsSync(outside),true);
}));

test('directory junction and symlink cannot escape to user paths',t=>fixture(root=>{
  ready(root);
  const outside=put(root,'Data/Launchers/Minecraft-1/world/secret','world');
  const link=resolve(root,`${ARCHIVE}/debug/deps/world-link`);
  try{symlinkSync(dirname(outside),link,process.platform==='win32'?'junction':'dir');}
  catch(error){if(process.platform==='win32'&&error.code==='EPERM'){t.skip('No junction permission');return;}throw error;}
  const result=retireArchivedCargo(root);
  assert.equal(result.status,'BLOCKED');
  assert.equal(readFileSync(outside,'utf8'),'world');
}));

test('failed Cargo clean does not pretend that storage was reclaimed',()=>fixture(root=>{
  ready(root);
  const result=retireArchivedCargo(root,{silent:true,execute:(_bin,args)=>
    args[0]==='--version'?{status:0,stdout:'cargo 1.99.1 (fixture)'}:{status:1}});
  assert.equal(result.status,'FAILED');
  assert.equal(existsSync(resolve(root,ARCHIVE)),true);
}));

test('successful subprocess exit without removal is only PARTIAL evidence',()=>fixture(root=>{
  ready(root);
  const result=retireArchivedCargo(root,{silent:true,execute:(_bin,args)=>
    args[0]==='--version'?{status:0,stdout:'cargo 1.99.1 (fixture)'}:{status:0}});
  assert.equal(result.status,'PARTIAL');
  assert.equal(existsSync(resolve(root,ARCHIVE)),true);
}));

test('different Cargo executable version is blocked before invoking clean',()=>fixture(root=>{
  ready(root);
  let cleanCalls=0;
  const result=retireArchivedCargo(root,{silent:true,execute:(_bin,args)=>{
    if(args[0]==='--version')return {status:0,stdout:'cargo 0.0.0 (untrusted)'};
    cleanCalls++;
    return {status:0};
  }});
  assert.equal(result.status,'BLOCKED');
  assert.equal(cleanCalls,0);
  assert.equal(existsSync(resolve(root,ARCHIVE)),true);
}));

test('missing pinned Cargo executable blocks mutation',()=>fixture(root=>{
  ready(root);
  rmSync(resolve(root,'workspace/toolchains/cargo/bin/cargo.exe'));
  assert.equal(retireArchivedCargo(root).status,'BLOCKED');
  assert.equal(existsSync(resolve(root,ARCHIVE)),true);
}));

test('direct --apply is rejected with no invocation or cache changes',()=>fixture(root=>{
  ready(root);
  const entry=fileURLToPath(new URL('./archive-retire.mjs',import.meta.url));
  const result=spawnSync(process.execPath,[entry,'--apply'],{encoding:'utf8'});
  assert.equal(result.status,2);
  assert.match(result.stderr,/No direct arguments accepted/);
  assert.equal(existsSync(resolve(root,ARCHIVE)),true);
}));
