import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {checkDependencyLock} from './check-dependency-lock.mjs';
const manifest=JSON.parse(readFileSync(new URL('../../apps/desktop/package.json',import.meta.url)));
const lock=JSON.parse(readFileSync(new URL('../../apps/desktop/package-lock.json',import.meta.url)));
test('real npm lock accepts independent Tauri patch releases and valid undici range',()=>checkDependencyLock(manifest,lock));
test('reject stale declarations, root extras, missing platform integrity and out-of-range undici',()=>{
  for(const mutate of [
    l=>{l.packages[''].devDependencies.vite='0.0.0';},
    l=>{l.packages[''].dependencies.extra='1.0.0';},
    l=>{l.packages[''].engines.node='24.x';},
    l=>{delete l.packages['node_modules/@tauri-apps/cli-win32-x64-msvc'].integrity;},
    l=>{l.packages['node_modules/undici-types'].version='7.18.2';},
  ]) {const copy=structuredClone(lock);mutate(copy);assert.throws(()=>checkDependencyLock(manifest,copy));}
});
