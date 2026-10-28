// Source/layout verification only. Runtime and installer acceptance are separate.
import assert from 'node:assert/strict';
import {existsSync,readFileSync,readdirSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {resolve,dirname,posix} from 'node:path';
import {fileURLToPath} from 'node:url';
import {checkTracked,checkLinks} from './check-structure.mjs';
import {checkDependencyLock} from './check-dependency-lock.mjs';
import {verifyLazyTesterPackage} from '../../apps/desktop/scripts/check-lazy-tester-package.mjs';
import {verifyNativePreviewPackage} from '../../apps/desktop/scripts/check-native-preview-package.mjs';

// Static Cargo manifest boundary only. This does not compile the Mac target:
// Win32 session/window code still requires its own future platform adapter.
export function verifyDesktopWindowsDeps(manifest) {
  const lines = manifest.split(/\r?\n/);
  const entries = heading => {
    const start = lines.indexOf('[' + heading + ']');
    assert.ok(start >= 0, 'Missing Cargo manifest section: ' + heading);
    const block = [];
    for (let i = start + 1; i < lines.length && !/^\[[^\]]+\]$/.test(lines[i].trim()); i++) {
      const row = lines[i].trim();
      if (/^[a-zA-Z][\w-]*\s*=/.test(row)) block.push(row);
    }
    return block;
  };
  const common = entries('dependencies');
  const native = entries("target.'cfg(windows)'.dependencies");
  const key = entry => entry.split('=', 1)[0].trim();
  for (const name of ['windows', 'windows-sys']) {
    assert.ok(!common.some(entry => key(entry) === name),
      'Windows-only desktop dependency must not be unconditional: ' + name);
    assert.equal(native.filter(entry => key(entry) === name).length, 1,
      'Windows desktop dependency must be declared once under cfg(windows): ' + name);
  }
  assert.ok(native.some(entry => key(entry) === 'windows' && entry.includes('Win32_Media_Audio')),
    'Native audio COM binding must remain available on Windows');
  assert.ok(native.some(entry => key(entry) === 'windows-sys' &&
    entry.includes('Win32_Graphics_Dwm') && entry.includes('Win32_System_Registry')),
    'Existing DWM and registry features must remain available on Windows');
}

// All mounted frontend source is already traced from main.ts. The Tauri
// window/event APIs may be used by the shell; game, profile and presentation
// commands must enter only through the two typed bridges and one invoke owner.
// This is a source-policy check, not an actual macOS IPC/runtime test.
export function verifyFrontendNativeIpcBoundary(sources) {
  const typedBridges = new Set(['app/bridge/lazyTester.ts', 'app/bridge/workspaceSurface.ts']);
  const runtime = sources['app/bridge/invokeRuntime.ts'];
  assert.ok(typeof runtime === 'string' &&
    /import\s*\{\s*invoke\s*\}\s*from\s*["']@tauri-apps\/api\/core["']/.test(runtime) &&
    /\breturn\s+invoke(?:<[^>\n]+>)?\s*\(/.test(runtime),
    'Tauri core invocation must be owned by invokeRuntime.ts');
  for (const [path, source] of Object.entries(sources)) {
    if (!/\.(?:ts|svelte)$/.test(path) || path === 'app/bridge/invokeRuntime.ts') continue;
    assert.ok(!/@tauri-apps\/api\/core|__TAURI_INTERNALS__/.test(source),
      'Frontend must not import raw Tauri core or reach internals: ' + path);
    if (typedBridges.has(path)) {
      assert.ok(/from\s*["']\.\/invokeRuntime\.js["']/.test(source),
        'Typed native bridge must import the reviewed runtime owner: ' + path);
    } else {
      assert.ok(!/\b(?:invokeRuntime|invoke)(?:<[^>\n]+>)?\s*\(/.test(source) &&
        !/\bfrom\s*["'][^"']*invokeRuntime(?:\.js)?["']/.test(source),
        'Mounted UI must use typed native bridges, never direct game IPC: ' + path);
    }
  }
}

export function traceFrontend(sources,entry='main.ts') {
  const reached=new Set();
  function visit(path) {
    if(reached.has(path))return;
    assert.ok(Object.hasOwn(sources,path),'Missing frontend import: '+path);
    reached.add(path);
    assert.ok(!/\bimport\s*\(/.test(sources[path]),'Review dynamic frontend import: '+path);
    for(const match of sources[path].matchAll(/(?:\bfrom\s*|\bimport\s*)["']([^"']+)["']/g)) {
      const specifier=match[1];
      if(!specifier.startsWith('.'))continue;
      let target=posix.normalize(posix.join(posix.dirname(path),specifier));
      assert.ok(!target.startsWith('../'),'Frontend imports outside its owner: '+path);
      if(target.endsWith('.js'))target=target.slice(0,-3)+'.ts';
      visit(target);
    }
  }
  visit(entry);
  for(const path of Object.keys(sources).filter(path=>path.endsWith('.d.ts')))visit(path);
  assert.deepEqual(Object.keys(sources).filter(path=>!reached.has(path)),[],'Unmounted/dead frontend source must be archived in Git history');
  return [...reached];
}
function filesBelow(root,path) {
  return readdirSync(resolve(root,path),{withFileTypes:true}).flatMap(entry=>{
    assert.ok(!entry.isSymbolicLink(),'Source symlink requires review: '+path+'/'+entry.name);
    const name=path+'/'+entry.name;
    return entry.isDirectory()?filesBelow(root,name):[name];
  });
}
export function verifyRepository(root,g0=false) {
  const read=path=>readFileSync(resolve(root,path),'utf8');
  const json=path=>JSON.parse(read(path));
  for(const path of ['AGENTS.md','GITHUB_RULES.md','CONTEXT.md','planning/development.md',
    'docs/system/implementation-map.md','docs/system/legacy-retirement.md','Development/VERSION',
    'Development/apps/desktop/src-tauri/Cargo.lock','Development/engine/runtime-core/Cargo.lock'])assert.ok(existsSync(resolve(root,path)),path);
  const sourcePaths=filesBelow(root,'Development/apps/desktop/src');
  assert.ok(sourcePaths.every(path=>/\.(?:ts|svelte|css)$/.test(path)),'Unexpected binary/private frontend material');
  const sources=Object.fromEntries(sourcePaths.map(path=>[path.slice('Development/apps/desktop/src/'.length),read(path)]));
  const reached=traceFrontend(sources);
  verifyFrontendNativeIpcBoundary(sources);
  assert.ok(sources['main.ts'].includes('mount(LazyTesterApp, { target });'),'Lazy Tester must be the mounted app');
  assert.ok(!reached.some(path=>/ownedProviderApi|nativeFeasibilityApi|virtualClientsApi/.test(path)),'Retired provider bridge must not be active');
  const app=json('Development/apps/desktop/package.json'),manifest=json('Development/package.json'),tauri=json('Development/apps/desktop/src-tauri/tauri.conf.json');
  const version=read('Development/VERSION').trim();
  for(const actual of [manifest.version,app.version,tauri.version,
    /^version\s*=\s*"([^"]+)"/m.exec(read('Development/apps/desktop/src-tauri/Cargo.toml'))?.[1],
    /^version\s*=\s*"([^"]+)"/m.exec(read('Development/engine/runtime-core/Cargo.toml'))?.[1]])assert.equal(actual,version,'Version parity');
  checkDependencyLock(app,json('Development/apps/desktop/package-lock.json'));
  verifyDesktopWindowsDeps(read('Development/apps/desktop/src-tauri/Cargo.toml'));
  const toolchain=json('Development/toolchain.json');
  assert.equal(toolchain.node.version,read('Development/.node-version').trim());
  assert.equal(toolchain.policy.ciPolicy,'disabled');
  assert.equal(app.engines.node,'^'+toolchain.node.version);
  assert.equal(manifest.engines.node,app.engines.node);
  assert.equal(app.engines.npm,toolchain.npm.version);
  assert.equal(manifest.engines.npm,app.engines.npm);
  if(existsSync(resolve(root,'.github/workflows')))assert.equal(filesBelow(root,'.github/workflows').filter(path=>/\.ya?ml$/i.test(path)).length,0,'CI must remain off');
  const bootstrap=read('Development/apps/desktop/src-tauri/src/app_bootstrap.rs');
  assert.ok(bootstrap.includes('.on_window_event(|window, event| {\n            // The close/restore handshake is Windows game authority only.')
    && /#\[cfg\(windows\)\]\s*\{\s*if let tauri::WindowEvent::CloseRequested/.test(bootstrap)
    && /#\[cfg\(not\(windows\)\)\]\s*let _ = \(window, event\);/.test(bootstrap)
    && /#\[cfg\(windows\)\]\s*let _ = crate::engine::workspace_surface::release\(\);/.test(bootstrap)
    && /#\[cfg\(windows\)\]\s*crate::engine::instance_focus_events::shutdown\(\);/.test(bootstrap),
    'Non-Windows Tauri exit must not send CloseAll or invoke native workspace recovery');
  for(const name of ['lazy_tester_backend_support','lazy_tester_session','lazy_tester_profile_binding','lazy_tester_present','lazy_tester_previews'])assert.ok(bootstrap.includes('commands::native_clients::'+name+','),name);
  assert.ok(!/commands::virtual_clients::|commands::setup_tools::|commands::window_arrangement::/.test(bootstrap),'Legacy handlers must remain unregistered');
  assert.ok(/#\[cfg\(not\(windows\)\)\][\s\S]*?app\.get_webview_window\("main"\)\.is_none\(\)/.test(bootstrap),
    'Non-Windows startup must verify the Tauri-created main window');
  assert.ok(bootstrap.includes('cfg(all(windows, feature="lazy-tester-package"))'),
    'Windows product data/WebView override must not disable non-Windows main window creation');
  const buildScript=read('Development/apps/desktop/src-tauri/build.rs');
  assert.ok(buildScript.includes('CARGO_CFG_TARGET_OS') && buildScript.includes('CARGO_FEATURE_LAZY_TESTER_PACKAGE'),
    'Native Windows-only product packaging must reject unsupported OS targets');
  const mountedUI=read('Development/apps/desktop/src/LazyTesterApp.svelte');
  const titlebar=read('Development/apps/desktop/src/WindowTitlebar.svelte');
  assert.ok(mountedUI.includes("if (nativeBackend === 'WindowsNative' && !closeReviewActive(reviewState()) && !document.hidden && binding?.configured") &&
    mountedUI.includes('<WindowTitlebar backendSupport={nativeBackend} />') &&
    mountedUI.includes("if (nativeBackend !== 'WindowsNative' || closeReviewActive(reviewState())") &&
    mountedUI.includes("if (nativeBackend !== 'WindowsNative') return;") &&
    mountedUI.includes("if (nativeBackend === 'WindowsNative') void workspaceSurface.release()") &&
    mountedUI.includes("{#if nativeBackend === 'WindowsNative'}") &&
    titlebar.includes("title={backendSupport === 'WindowsNative' ?"),
    'Unsupported diagnostic UI must not schedule game IPC or advertise CloseAll');
  const nativeSource=read('Development/apps/desktop/src-tauri/src/engine/native_clients.rs');
  assert.ok(nativeSource.includes('pub enum NativeBackendSupport') &&
    nativeSource.includes('cfg!(all(windows, target_arch = "x86_64"))'),
    'Native backend support must be an implementation capability, not a game readiness claim');
  const invoked=bootstrap.split('.invoke_handler(tauri::generate_handler![')[1]?.split('])')[0]??'';
  const exposed=[...invoked.matchAll(/commands::native_clients::([a-z_]+),/g)].map(m=>m[1]);
  assert.deepEqual(exposed,[
    'lazy_tester_backend_support','lazy_tester_session','lazy_tester_close_one','lazy_tester_profile_binding',
    'lazy_tester_data_location','lazy_tester_present','lazy_tester_previews',
    'lazy_tester_clear_previews','desktop_update_workspace',
    'desktop_release_workspace','desktop_suspend_workspace',
  ],'Only mounted Lazy Tester handlers may be exposed; retired provider IPC forbidden');

  for(const path of [
    'Development/apps/desktop/src-tauri/src/engine/owned_isolation.rs',
    'Development/apps/desktop/src-tauri/src/engine/virtual_inventory.rs']) {
    assert.ok(!existsSync(resolve(root,path)),'Retired fork inspector or Sandboxie inventory returned: '+path);
  }
  for(const path of [
    'Development/apps/desktop/src-tauri/src/commands/virtual_clients.rs',
    'Development/apps/desktop/src-tauri/src/commands/setup_tools.rs',
    'Development/apps/desktop/src-tauri/src/commands/window_arrangement.rs',
    'Development/apps/desktop/src-tauri/src/engine/screen_overlay.rs',
    'Development/apps/desktop/src-tauri/src/engine/setup_tools.rs',
    'Development/apps/desktop/src-tauri/src/engine/window_arrangement.rs']) {
    assert.ok(!existsSync(resolve(root,path)),'Unreachable VM desktop source returned: '+path);
  }
  const commands=read('Development/apps/desktop/src-tauri/src/commands/mod.rs');
  assert.ok(!/^\s*pub mod (virtual_clients|setup_tools|window_arrangement);/m.test(commands));
  const core=read('Development/engine/runtime-core/src/lib.rs');
  assert.ok(core.includes('pub mod independent_engine;')&&core.includes('mod process;'),
    'Native-only Rust engine must retain inspection contracts and bounded process runner');
  assert.ok(!/mod (?:runtime|guest|provider|command|doctor|profile|update);/.test(core),
    'Retired VM runtime and provider modules must not be reactivated');
  for(const path of ['Development/engine/runtime-core/src/command.rs',
    'Development/engine/runtime-core/src/runtime.rs',
    'Development/engine/runtime-core/src/provider/vmx.rs',
    'Development/engine/runtime-core/src/bin/virtual-guest-agent.rs']) {
    assert.ok(!existsSync(resolve(root,path)),'Retired VM source returned: '+path);
  }
  for (const path of [
    'Development/apps/desktop/src-tauri/src/engine/owned_provider.rs',
    'Development/apps/desktop/src-tauri/src/engine/instance_catalog.rs',
    'Development/apps/desktop/src-tauri/src/engine/instance_discovery.rs',
    'Development/apps/desktop/src-tauri/src/engine/instance_focus.rs',
    'Development/apps/desktop/src-tauri/src/engine/legacy-targets.json',
    'Development/tooling/isolation-source',
    'Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1',
    'Development/tooling/acceptance/windows/collect-virtual-boxes.ps1'
  ]) assert.ok(!existsSync(resolve(root,path)), 'Retired Sandboxie runtime returned: '+path);
  const ownerModules=read('Development/apps/desktop/src-tauri/src/engine/mod.rs');
  // Source-only guard: the retained Windows single-instance marker requires
  // LOCALAPPDATA/APPDATA. An unsupported OS must reach the Tauri diagnostic
  // window without touching that Windows path or adopting game authority.
  assert.ok(/#\[cfg\(windows\)\]\s*pub mod app_instance;/.test(ownerModules)
    && /#\[cfg\(windows\)\]\s*use crate::engine::app_instance;/.test(bootstrap)
    && /#\[cfg\(windows\)\]\s*let _instance_lease = match app_instance::acquire\(\)/.test(bootstrap),
    'Windows instance lease must not block unsupported-platform startup');
  const surface=read('Development/apps/desktop/src-tauri/src/engine/workspace_surface.rs');
  const unsupportedRole=surface.match(/#\[cfg\(not\(windows\)\)\]\s*pub fn maybe_run_helper\(\) -> bool \{([^}]*)\}/)?.[1];
  assert.ok(/#\[cfg\(windows\)\]\s*pub fn maybe_run_helper\(\) -> bool/.test(surface)
    && unsupportedRole
    && unsupportedRole.includes('args_os().nth(1).as_deref()')
    && unsupportedRole.includes('Some(std::ffi::OsStr::new("--workspace-surface"))')
    && !/native::|process_roles|Command::|read_line|spawn\(/.test(unsupportedRole),
    'Unsupported helper role must not start recovery, process, IPC or UI fallback');
  const unavailableSurface=read('Development/apps/desktop/src-tauri/src/engine/workspace_surface/unsupported_controller.rs');
  const windowsController=read('Development/apps/desktop/src-tauri/src/engine/workspace_surface/controller.rs');
  const windowsAdapter=read('Development/apps/desktop/src-tauri/src/engine/workspace_surface/native.rs');
  const previewApi=read('Development/apps/desktop/src-tauri/src/engine/workspace_surface/previews.rs');
  const windowsPreview=read('Development/apps/desktop/src-tauri/src/engine/workspace_surface/previews/windows.rs');
  assert.ok(/#\[cfg\(windows\)\]\s*#\[path = "workspace_surface\/controller\.rs"\]\s*mod controller;/.test(surface)
    && /#\[cfg\(not\(windows\)\)\]\s*#\[path = "workspace_surface\/unsupported_controller\.rs"\]\s*mod controller;/.test(surface)
    && windowsController.includes('pub fn release_for_close() -> io::Result<()>')
    && /#\[cfg\(windows\)\]\s*#\[path = "workspace_surface\/native\.rs"\]\s*mod native;/.test(surface)
    && windowsAdapter.includes('pub(super) struct ProcessHandle')
    && /#\[cfg\(windows\)\]\s*mod windows;/.test(previewApi)
    && windowsPreview.includes('DwmRegisterThumbnail')
    && windowsPreview.includes('mpsc::sync_channel(1)'),
    'Windows controller, adapter and bounded DWM worker must stay behind Win32-only module boundaries');
  assert.ok(unavailableSurface.includes('io::ErrorKind::Unsupported')
    && unavailableSurface.includes('pub fn is_active() -> bool')
    && unavailableSurface.includes('false // No native workspace lease')
    && !/windows(_sys)?::|std::os::windows|\.spawn\(/.test(unavailableSurface),
    'Non-Windows workspace must not start helpers, import Win32, or claim active leases');
  assert.ok(ownerModules.includes('pub mod process_identity;') &&
    !ownerModules.includes('pub mod owned_provider;'), 'Native process identity must be standalone');
  const gameSource=read('Development/apps/desktop/src-tauri/src/engine/native_clients.rs');
  assert.ok(!/OfficialSandboxieBackend|request_sandbox_client_02|collect-sandboxie-feasibility/.test(gameSource),
    'Legacy Sandboxie Client02 source must remain retired');
  // Windows developer preview status must never become a non-Windows
  // Minecraft profile-ready signal before a real backend is implemented.
  const nonWindowsBinding = gameSource.match(/#\[cfg\(not\(windows\)\)\]\s*pub fn profile_binding\([^)]*\) -> io::Result<ProfileBinding> \{([^}]*)\}/)?.[1];
  const nonWindowsLocation = gameSource.match(/#\[cfg\(not\(windows\)\)\]\s*pub fn data_location\([^)]*\) -> io::Result<DataLocation> \{([^}]*)\}/)?.[1];
  assert.ok(nonWindowsBinding && nonWindowsLocation &&
    /Err\(unsupported_native_profiles\(\)\)/.test(nonWindowsBinding) &&
    /Err\(unsupported_native_profiles\(\)\)/.test(nonWindowsLocation) &&
    /#\[cfg\(not\(windows\)\)\]\s*fn unsupported_native_profiles\(\) -> io::Error \{[\s\S]*?io::ErrorKind::Unsupported/.test(gameSource),
    'Non-Windows profile binding and storage must fail closed, never claim Ready');
  const launcher=read('Development/apps/desktop/src-tauri/src/main.rs');
  assert.ok(!launcher.includes('maybe_run_profile_helper'), 'Retired fork helper role must not start');
  assert.ok(read('Development/apps/desktop/scripts/prepare-package-binaries.mjs').includes('const LEGACY_PACKAGE_RELEASE_APPROVED = false;'),'Legacy package hold must remain');
  verifyNativePreviewPackage();verifyLazyTesterPackage();
  const tracked=execFileSync('git',['ls-files','-z'],{cwd:root,encoding:'utf8'}).split('\0').filter(Boolean);
  assert.deepEqual(checkTracked(tracked),[]);
  for(const path of ['Development/engine/guest/windows', 'Development/engine/isolation-native',
    'Development/apps/desktop/src-tauri/tauri.package.conf.json',
    'Development/apps/desktop/src-tauri/windows/installer-hooks.nsh']) {
    assert.ok(!existsSync(resolve(root,path)),'Retired Windows VM/source route returned: '+path);
  }
  assert.ok(!tracked.some(path=>path==='.gitmodules'||/^Development\/engine\/isolation-(?:owned|candidates|upstream)(?:\/|$)/.test(path)),
    'Retired Sandboxie vendor source or submodule must not return');
  assert.ok(!tracked.some(path=>/(?:replay-profile-(?:pair|five)|profile-binding-five)\.bin$/.test(path)),'Private bindings must never be tracked');
  for(const path of ['workspace/private/profile.json','workspace/packages/installer.exe','workspace/runtime/replay-profile-five.bin',
    'Data/profiles.json','App/data-location.json','Logs/session.log']) {
    execFileSync('git',['check-ignore','--no-index','-q','--',path],{cwd:root});
  }
  if(g0)assert.equal(execFileSync('git',['ls-tree','-r','--name-only','HEAD','--','Development/apps/desktop/src-tauri/Cargo.lock'],{cwd:root,encoding:'utf8'}).trim(),'Development/apps/desktop/src-tauri/Cargo.lock');
  assert.deepEqual(checkLinks(root,filesBelow(root,'docs').filter(path=>path.endsWith('.md'))),[]);
  const handoff=read('planning/development.md');
  assert.ok(handoff.startsWith('# Current Development Handoff')&&handoff.length<8000,'Keep one concise handoff');
  console.log('Lazy Tester repository contract: PASS ('+reached.length+' reachable frontend files; source/layout evidence only)');
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url))verifyRepository(resolve(dirname(fileURLToPath(import.meta.url)),'../../..'),process.argv.includes('--g0'));
