import test from 'node:test';
import assert from 'node:assert/strict';
import {traceFrontend,verifyDesktopWindowsDeps,verifyFrontendNativeIpcBoundary,
  verifyNativeGraphicsExperimentBoundary} from './verify.mjs';
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

test('native graphics allocator canaries must not activate without a pinned opt-in',()=>{
  const gfx='BOOTSTRAP_GRAPHICS_WARM_POOLS_EXPERIMENT';
  const upload='BOOTSTRAP_UPLOAD_RETENTION_EXPERIMENT';
  const cmake=[
    'option('+gfx+' "Legacy heap" OFF)',
    'option('+upload+' "Upload heap" OFF)',
    'if('+gfx+' AND '+upload+')',
    '  message(FATAL_ERROR "Heap canaries must remain separate")','endif()',
    'if('+gfx+')','  target_compile_definitions(bootstrap-compat PRIVATE '+gfx+'=1)','endif()',
    'if('+upload+')','  target_compile_definitions(bootstrap-compat PRIVATE '+upload+'=1)','endif()',
    'if('+upload+')','  target_compile_definitions(graphics-warm-pools-control PRIVATE '+upload+'=1)','endif()',
  ].join('\n');
  const sites=[
    '0xbc22120, 691,','0xbc22490, 402,','0xbc22660, 402,',
    '0xbc22840, 2363,','0xbc24d10, 2905,','0xbc24570, 172,'
  ];
  const source=[
    '#if defined('+gfx+') || defined('+upload+')',
    '#if defined('+gfx+') && defined('+upload+')','#error Cannot combine experiments','#endif',
    'if (nt->FileHeader.TimeDateStamp != 0x6a63e832 || nt->OptionalHeader.SizeOfImage != 0x1255c000) return false;',
    ...sites.map(site=>'if (!digest(algorithm, '+site+' "pinned")) return false;'),
    'if (knownCode() && attach()) installed.store(true, std::memory_order_release);',
    '#else','namespace graphics_warm_pools { void consider() noexcept {} }','#endif'
  ].join('\n');
  assert.doesNotThrow(()=>verifyNativeGraphicsExperimentBoundary(cmake,source));
  for(const modified of [
    cmake.replace('option('+gfx+' "Legacy heap" OFF)','option('+gfx+' "Legacy heap" ON)'),
    cmake.replace('option('+upload+' "Upload heap" OFF)','option('+upload+' "Upload heap" ON)'),
    cmake.replace(' AND '+upload,''),
    cmake.replace('if('+gfx+')','if('+upload+')'),
    cmake+'\ntarget_compile_definitions(bootstrap-compat PRIVATE '+gfx+'=1)',
    cmake.replace('target_compile_definitions(bootstrap-compat PRIVATE '+upload+'=1)',
      'target_compile_definitions(graphics-warm-pools-control PRIVATE '+upload+'=1)'),
  ]) assert.throws(()=>verifyNativeGraphicsExperimentBoundary(modified,source));
  for(const modified of [
    source.replace('if (knownCode() && attach())','if (attach())'),
    source.replace('SizeOfImage != 0x1255c000','SizeOfImage == 0x1255c000'),
    source.replace('digest(algorithm,','unused(algorithm,'),
    source.replace('#error Cannot combine experiments',''),
    source.replace('#if defined('+gfx+') || defined('+upload+')','#if 1'),
    source.replace('namespace graphics_warm_pools { void consider() noexcept {} }',
      'namespace graphics_warm_pools { void consider() noexcept { attach(); } }'),
  ]) assert.throws(()=>verifyNativeGraphicsExperimentBoundary(cmake,modified));
});
