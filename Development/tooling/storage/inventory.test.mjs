import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,symlinkSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {collectInventory,renderInventory,CATEGORIES} from './inventory.mjs';

function fixture(fn) {
  const root=mkdtempSync(join(tmpdir(),'lazy-storage-'));
  try {return fn(root);} finally {rmSync(root,{recursive:true,force:true});}
}
const write=(root,path,bytes)=>{
  const file=resolve(root,path);
  mkdirSync(resolve(file,'..'),{recursive:true});
  writeFileSync(file,Buffer.alloc(bytes));
};

test('scan categories are disjoint and never inspect Data or private runtime',()=>fixture(root=>{
  write(root,'workspace/builds/desktop/target/debug/a',11);
  write(root,'workspace/builds/extra/b',17);
  write(root,'workspace/archive/compiler-cache-20261026/a',19);
  write(root,'workspace/archive/layout-20261026/b',23);
  write(root,'Data/worlds/secret',101);
  write(root,'workspace/private/local-app-backups/secret',103);
  const result=collectInventory(root);
  const get=name=>result.rows.find(row=>row.name===name);
  assert.equal(get('Active Cargo target').bytes,11n);
  assert.equal(get('Other build outputs').bytes,17n);
  assert.equal(get('Archived compiler cache').bytes,19n);
  assert.equal(get('Other retained archives').bytes,23n);
  assert.equal(result.total,70n);
  assert.equal(result.complete,true);
  assert.doesNotMatch(renderInventory(result),/secret|worlds/);
}));

test('missing paths are absent rather than claimed measured, and report is read only',()=>fixture(root=>{
  const report=collectInventory(root);
  assert.equal(report.total,0n);
  assert.equal(report.complete,true);
  assert.ok(report.rows.every(row=>row.state==='missing'));
  assert.match(renderInventory(report),/Nothing was deleted/);
}));

test('symlink inside source tree is skipped without following outside',t=>fixture(root=>{
  const outside=mkdtempSync(join(tmpdir(),'lazy-storage-outside-'));
  try {
    write(outside,'restricted',1024);
    write(root,'workspace/builds/desktop/target/regular',3);
    try { symlinkSync(outside,join(root,'workspace/builds/desktop/target/external'),process.platform==='win32'?'junction':'dir'); }
    catch(error) { if(process.platform==='win32' && error.code==='EPERM'){t.skip('Junction privileges unavailable');return;}throw error; }
    const report=collectInventory(root);
    const active=report.rows.find(row=>row.name==='Active Cargo target');
    assert.equal(active.bytes,3n);
    assert.equal(active.links,1);
    assert.equal(active.state,'partial');
    assert.equal(report.complete,false);
  } finally {rmSync(outside,{recursive:true,force:true});}
}));

test('linked category ancestor is not traversed',t=>fixture(root=>{
  const outside=mkdtempSync(join(tmpdir(),'lazy-storage-outside-'));
  try {
    write(outside,'outsider',1024);
    mkdirSync(join(root,'workspace'),{recursive:true});
    try { symlinkSync(outside,join(root,'workspace/builds'),process.platform==='win32'?'junction':'dir'); }
    catch(error) { if(process.platform==='win32' && error.code==='EPERM'){t.skip('Junction privileges unavailable');return;}throw error; }
    const report=collectInventory(root);
    assert.equal(report.rows[0].bytes,0n);
    assert.equal(report.rows[0].state,'partial');
  }finally{rmSync(outside,{recursive:true,force:true});}
}));

test('entry cap produces explicit lower-bound partial, not a false complete total',()=>fixture(root=>{
  write(root,'workspace/cache/a',4);
  write(root,'workspace/cache/b',7);
  const report=collectInventory(root,{maxVisitedEntries:2});
  assert.equal(report.complete,false);
  assert.equal(report.limited,true);
  assert.match(renderInventory(report),/LOWER BOUND/);
}));

test('fixture retains no mutation functions and categories avoid protected paths',()=>{
  assert.equal(CATEGORIES.some(row=>/^Data\/|^workspace\/(private|runs|runtime|images|evidence)/.test(row.path)),false);
  assert.equal(new Set(CATEGORIES.map(row=>row.path)).size,CATEGORIES.length);
  for(const parent of CATEGORIES) for(const child of CATEGORIES) {
    if(child.path.startsWith(parent.path+'/')) {
      assert.ok(parent.exclude?.includes(child.path.slice(parent.path.length+1)),
        `${parent.name} must exclude nested ${child.name}`);
    }
  }
});
