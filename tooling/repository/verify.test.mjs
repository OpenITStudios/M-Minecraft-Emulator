import test from 'node:test';
import assert from 'node:assert/strict';
import {traceFrontend} from './verify.mjs';
test('frontend reachability rejects missing/escaping imports and unmounted alternate apps',()=>{
  assert.deepEqual(traceFrontend({'main.ts':"import App from './App.svelte';",'App.svelte':"import './style.css';",'style.css':''}),['main.ts','App.svelte','style.css']);
  assert.throws(()=>traceFrontend({'main.ts':"import './missing.js';"}),/Missing/);
  assert.throws(()=>traceFrontend({'main.ts':"import '../other.js';"}),/outside/);
  assert.throws(()=>traceFrontend({'main.ts':'','RetiredApp.svelte':''}),/Unmounted/);
  assert.throws(()=>traceFrontend({'main.ts':"import('./hidden.js')"}),/dynamic/);
});

