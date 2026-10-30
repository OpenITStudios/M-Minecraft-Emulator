// Authored for later validation. Operator has NOT authorized local execution.
import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import {inspectPng,validateCuaExecutable,buildCuaWindowQuery} from './visual-capture.mjs';

const png1x1=Buffer.from(
  'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+j6YQAAAAASUVORK5CYII=',
  'base64');

test('Cua executable requires a precise absolute local binary path',()=>{
  assert.equal(validateCuaExecutable(path.resolve('bin','cua-driver.exe')),
    path.resolve('bin','cua-driver.exe'));
  for(const invalid of ['cua-driver.exe','./cua-driver.exe',
    path.resolve('bin','powershell.exe')]) {
    assert.throws(()=>validateCuaExecutable(invalid));
  }
});

test('Cua preview targets one exact window without modifying action snapshots',()=>{
  const query=buildCuaWindowQuery({pid:42,windowId:1234});
  assert.equal(query.pid,42);
  assert.equal(query.window_id,1234);
  assert.equal(query.display_only,true);
  assert.equal(query.include_accessibility_tree,false);
  assert.equal(query.include_screenshot,true);
  assert.equal(query.scope,undefined);
  assert.throws(()=>buildCuaWindowQuery({pid:0,windowId:1234}));
  assert.throws(()=>buildCuaWindowQuery({pid:42,windowId:0}));
});

test('PNG observation validates signature, dimensions and bounded source',()=>{
  const meta=inspectPng(png1x1);
  assert.equal(meta.width,1);
  assert.equal(meta.height,1);
  assert.match(meta.sha256,/^[a-f0-9]{64}$/);
  assert.throws(()=>inspectPng(Buffer.from('not-an-image')));
  const altered=Buffer.from(png1x1);
  altered.writeUInt32BE(99999,16);
  assert.throws(()=>inspectPng(altered),/dimensions/);
  assert.throws(()=>inspectPng(Buffer.alloc(6*1024*1024+1)));
});
