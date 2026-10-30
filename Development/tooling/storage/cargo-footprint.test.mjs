import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,symlinkSync,linkSync,rmSync,existsSync,statSync} from 'node:fs';
import {join,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {cargoBucket,inspectCargoFootprint,renderCargoFootprint,CARGO_ROOTS,activeSnapshot} from './cargo-footprint.mjs';

function fixture(run){const root=mkdtempSync(join(tmpdir(),'cargo-footprint-'));try{return run(root);}finally{rmSync(root,{recursive:true,force:true});}}
function put(root,name,size){const file=resolve(root,name);mkdirSync(resolve(file,'..'),{recursive:true});writeFileSync(file,Buffer.alloc(size,1));return file;}
const target='workspace/builds/desktop/target';
const win='x86_64-pc-windows-msvc';
const get=report=>report.rows[0];

test('classification uses known Cargo target, profile, and component',()=>{
  assert.equal(cargoBucket([win,'debug','incremental','a']),'Windows MSVC debug / incremental');
  assert.equal(cargoBucket([win,'release','deps','b']),'Windows MSVC release / deps');
  assert.equal(cargoBucket(['debug','build','x']),'Host-default debug / build');
  assert.equal(cargoBucket(['release','.fingerprint','x']),'Host-default release / .fingerprint');
  assert.equal(cargoBucket(['x86_64-unknown-linux-gnu','debug','deps','x']),'Other target or root / other');
  assert.equal(cargoBucket([win,'debug','random']),'Windows MSVC debug / other');
});

test('active and legacy targets measured separately with no double-counting',()=>fixture(root=>{
  put(root,`${target}/${win}/debug/incremental/a`,17);
  put(root,`${target}/${win}/debug/deps/b`,31);
  put(root,`${target}/${win}/release/build/c`,47);
  put(root,`${target}/debug/deps/d`,11);
  put(root,`${target}/something-new`,13);
  put(root,'Development/apps/desktop/src-tauri/target/debug/deps/x',19);
  put(root,'Development/engine/runtime-core/target/debug/deps/y',23);
  put(root,'Data/worlds/do-not-open',1000);
  put(root,'workspace/private/do-not-open',1200);
  const r=inspectCargoFootprint(root);
  assert.equal(r.complete,true);
  assert.equal(r.total,161n);
  assert.equal(r.rows[0].bytes,119n);
  assert.equal(r.rows[1].bytes,19n);
  assert.equal(r.rows[2].bytes,23n);
  assert.equal(r.rows[0].buckets.get('Windows MSVC debug / incremental').bytes,17n);
  assert.equal(r.rows[0].buckets.get('Windows MSVC debug / deps').bytes,31n);
  assert.equal(r.rows[0].buckets.get('Windows MSVC release / build').bytes,47n);
  assert.match(renderCargoFootprint(r),/coexist/);
  assert.doesNotMatch(renderCargoFootprint(r),/do-not-open|Data\/worlds|workspace\/private/);
}));

test('missing outputs do not create directories',()=>fixture(root=>{
  const r=inspectCargoFootprint(root);
  assert.equal(r.complete,true);
  assert.equal(r.total,0n);
  assert.ok(r.rows.every(x=>x.status==='ABSENT'));
  assert.equal(existsSync(resolve(root,'workspace')),false);
}));

test('category root ancestor symlink is PARTIAL without traversing destination',t=>fixture(root=>{
  const external=mkdtempSync(join(tmpdir(),'cargo-outside-'));
  try{
    put(external,'a',1000);
    mkdirSync(resolve(root,'workspace'),{recursive:true});
    try{symlinkSync(external,resolve(root,'workspace/builds'),process.platform==='win32'?'junction':'dir');}
    catch(e){if(process.platform==='win32'&&e.code==='EPERM'){t.skip('Junction requires privileges');return;}throw e;}
    const r=inspectCargoFootprint(root);
    assert.equal(get(r).status,'PARTIAL');
    assert.equal(get(r).bytes,0n);
    assert.equal(r.complete,false);
  }finally{rmSync(external,{recursive:true,force:true});}
}));

test('interior directory link never follows private file and forces lower bound',t=>fixture(root=>{
  const external=mkdtempSync(join(tmpdir(),'cargo-outside-'));
  try{
    put(external,'secret',1000);
    put(root,`${target}/debug/deps/normal`,5);
    try{symlinkSync(external,resolve(root,`${target}/debug/deps/private-link`),process.platform==='win32'?'junction':'dir');}
    catch(e){if(process.platform==='win32'&&e.code==='EPERM'){t.skip('Junction requires privileges');return;}throw e;}
    const r=inspectCargoFootprint(root);
    assert.equal(get(r).status,'PARTIAL');
    assert.equal(get(r).bytes,5n);
    assert.equal(get(r).links,1);
    assert.match(renderCargoFootprint(r),/LOWER BOUND/);
  }finally{rmSync(external,{recursive:true,force:true});}
}));

test('global entry cap is explicit and does not scan unchecked directories',()=>fixture(root=>{
  for(let i=0;i<20;i++)put(root,`${target}/debug/deps/file-${i}`,2);
  const r=inspectCargoFootprint(root,{maxEntries:2});
  assert.equal(r.complete,false);
  assert.equal(r.limited,true);
  assert.equal(get(r).status,'PARTIAL');
}));

test('limit parameters must be positive integers',()=>fixture(root=>{
  assert.throws(()=>inspectCargoFootprint(root,{maxEntries:0}),/positive/);
  assert.throws(()=>inspectCargoFootprint(root,{maxDurationMs:-1}),/positive/);
  assert.throws(()=>inspectCargoFootprint(root,{maxEntries:Infinity}),/positive/);
}));

test('hardlink is counted but never described as recoverable storage',()=>fixture(root=>{
  const name=put(root,`${target}/debug/deps/a`,9);
  linkSync(name,resolve(root,`${target}/debug/deps/b`));
  const r=inspectCargoFootprint(root);
  assert.equal(get(r).bytes,18n);
  assert.equal(get(r).hardlinkedFiles,2);
  assert.match(renderCargoFootprint(r),/NOT disk allocation/);
}));

test('CLI rejects apply and leaves target unmodified',()=>fixture(root=>{
  const input=put(root,`${target}/debug/a`,3);
  const before=statSync(input).mtimeMs;
  const cli=fileURLToPath(new URL('./cargo-footprint.mjs',import.meta.url));
  const result=spawnSync(process.execPath,[cli,'--apply'],{encoding:'utf8'});
  assert.equal(result.status,2);
  assert.match(result.stderr,/no --apply/);
  assert.equal(statSync(input).mtimeMs,before);
}));

test('only fixed Cargo directories are inspected, never game/account paths',()=>{
  assert.equal(CARGO_ROOTS.length,3);
  assert.ok(CARGO_ROOTS.every(p=>p.path.endsWith('/target')));
  assert.ok(CARGO_ROOTS.every(p=>!/(^Data\/|^App\/|workspace\/(private|runs|runtime))/.test(p.path)));
});

test('active snapshot exposes only aggregated bounded metadata without file paths',()=>fixture(root=>{
  put(root,`${target}/debug/deps/private-token-filename`,41);
  put(root,'Data/Launchers/one/account.json',200);
  const r=inspectCargoFootprint(root,{roots:[CARGO_ROOTS[0]]});
  const data=activeSnapshot(r);
  assert.deepEqual(data,{schema:1,scope:'active-cargo-target',complete:true,
    status:'OK',bytes:'41',files:1,hardlinkedFiles:0,visited:r.visited});
  assert.doesNotMatch(JSON.stringify(data),/private-token|account|Data|workspace|\\|\//);
  assert.throws(()=>activeSnapshot(inspectCargoFootprint(root)),/canonical Cargo target/);
}));

test('partial active scan reports incomplete truth without fabricated savings',()=>fixture(root=>{
  put(root,`${target}/debug/deps/a`,13);
  put(root,`${target}/debug/deps/b`,37);
  const report=inspectCargoFootprint(root,{roots:[CARGO_ROOTS[0]],maxEntries:2});
  const data=activeSnapshot(report);
  assert.equal(data.complete,false);
  assert.equal(data.status,'PARTIAL');
  assert.match(data.bytes,/^\d+$/);
}));

test('active JSON CLI never accepts cleanup or arbitrary arguments',()=>{
  const cli=fileURLToPath(new URL('./cargo-footprint.mjs',import.meta.url));
  const snapshot=spawnSync(process.execPath,[cli,'--active-json'],{encoding:'utf8'});
  assert.equal(snapshot.status,0);
  const result=JSON.parse(snapshot.stdout.trim());
  assert.equal(result.schema,1);
  assert.equal(result.scope,'active-cargo-target');
  assert.match(result.bytes,/^\d+$/);
  assert.equal(Object.keys(result).includes('path'),false);
  for(const input of ['--apply','--delete','--active-json --apply']){
    const denied=spawnSync(process.execPath,[cli,...input.split(' ')],{encoding:'utf8'});
    assert.equal(denied.status,2);
    assert.match(denied.stderr,/no --apply/);
  }
});
