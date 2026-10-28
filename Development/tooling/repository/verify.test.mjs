import test from 'node:test';
import assert from 'node:assert/strict';
import {traceFrontend,verifyDesktopWindowsDeps,verifyFrontendNativeIpcBoundary} from './verify.mjs';
test('frontend reachability rejects missing/escaping imports and unmounted alternate apps',()=>{
  assert.deepEqual(traceFrontend({'main.ts':"import App from './App.svelte';",'App.svelte':"import './style.css';",'style.css':''}),['main.ts','App.svelte','style.css']);
  assert.throws(()=>traceFrontend({'main.ts':"import './missing.js';"}),/Missing/);
  assert.throws(()=>traceFrontend({'main.ts':"import '../other.js';"}),/outside/);
  assert.throws(()=>traceFrontend({'main.ts':'','RetiredApp.svelte':''}),/Unmounted/);
  assert.throws(()=>traceFrontend({'main.ts':"import('./hidden.js')"}),/dynamic/);
});

test('Windows native bindings are target-scoped without losing existing capabilities',()=>{
  const valid = `[package]
name = "desktop"
[dependencies]
serde = "1"
tauri = "2"
[target.'cfg(windows)'.dependencies]
windows = { version = "0.62.2", features = ["Win32_Media_Audio"] }
windows-sys = { version = "0.61", features = ["Win32_Graphics_Dwm", "Win32_System_Registry"] }
[features]
default = []
`;
  assert.doesNotThrow(()=>verifyDesktopWindowsDeps(valid));
  assert.throws(()=>verifyDesktopWindowsDeps(valid.replace(
    '[dependencies]\nserde', '[dependencies]\nwindows = "0.62.2"\nserde'
  )),/unconditional: windows/);
  assert.throws(()=>verifyDesktopWindowsDeps(valid.replace(
    'windows-sys = { version = "0.61", features = ["Win32_Graphics_Dwm", "Win32_System_Registry"] }\n', ''
  )),/declared once under cfg\(windows\): windows-sys/);
  assert.throws(()=>verifyDesktopWindowsDeps(valid.replace(
    '[target.\'cfg(windows)\'.dependencies]', '[target.\'cfg(unix)\'.dependencies]'
  )),/Missing Cargo manifest section/);
  assert.throws(()=>verifyDesktopWindowsDeps(valid.replace(
    '"Win32_Graphics_Dwm", "Win32_System_Registry"', '"Win32_Foundation"'
  )),/Existing DWM and registry features/);
});

test('mounted native IPC is owned by typed bridges, not UI or Tauri window controls',()=>{
  const good={
    'main.ts': "import App from './LazyTesterApp.svelte';",
    'LazyTesterApp.svelte': "import {lazyTester} from './app/bridge/lazyTester.js';",
    'WindowTitlebar.svelte': "import {getCurrentWindow} from '@tauri-apps/api/window';",
    'app/bridge/invokeRuntime.ts': "import {invoke} from '@tauri-apps/api/core'; export function invokeRuntime<T>(command:string,args?:object){return invoke<T>(command,args);}",
    'app/bridge/lazyTester.ts': "import {invokeRuntime} from './invokeRuntime.js'; export const game=()=>invokeRuntime('lazy_tester_session');",
    'app/bridge/workspaceSurface.ts': "import {invokeRuntime} from './invokeRuntime.js'; export const view=()=>invokeRuntime('desktop_release_workspace');"
  };
  assert.doesNotThrow(()=>verifyFrontendNativeIpcBoundary(good));
  assert.throws(()=>verifyFrontendNativeIpcBoundary({...good,'LazyTesterApp.svelte':"invokeRuntime('lazy_tester_session')"}),/never direct game IPC/);
  assert.throws(()=>verifyFrontendNativeIpcBoundary({...good,'WindowTitlebar.svelte':"import {invoke} from '@tauri-apps/api/core';"}),/raw Tauri core/);
  assert.throws(()=>verifyFrontendNativeIpcBoundary({...good,'main.ts':"window.__TAURI_INTERNALS__.invoke('lazy_tester_session')"}),/reach internals/);
  assert.throws(()=>verifyFrontendNativeIpcBoundary({...good,'app/bridge/workspaceSurface.ts':"import {invoke} from '@tauri-apps/api/core';"}),/raw Tauri core/);
  assert.throws(()=>verifyFrontendNativeIpcBoundary({...good,'app/bridge/invokeRuntime.ts':"export function invokeRuntime() {}"}),/must be owned by invokeRuntime.ts/);
});
