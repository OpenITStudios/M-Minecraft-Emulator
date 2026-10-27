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
  assert.ok(sources['main.ts'].includes('mount(LazyTesterApp, { target });'),'Lazy Tester must be the mounted app');
  assert.ok(!reached.some(path=>/ownedProviderApi|nativeFeasibilityApi|virtualClientsApi/.test(path)),'Retired provider bridge must not be active');
  const app=json('Development/apps/desktop/package.json'),manifest=json('Development/package.json'),tauri=json('Development/apps/desktop/src-tauri/tauri.conf.json');
  const version=read('Development/VERSION').trim();
  for(const actual of [manifest.version,app.version,tauri.version,
    /^version\s*=\s*"([^"]+)"/m.exec(read('Development/apps/desktop/src-tauri/Cargo.toml'))?.[1],
    /^version\s*=\s*"([^"]+)"/m.exec(read('Development/engine/runtime-core/Cargo.toml'))?.[1]])assert.equal(actual,version,'Version parity');
  checkDependencyLock(app,json('Development/apps/desktop/package-lock.json'));
  const toolchain=json('Development/toolchain.json');
  assert.equal(toolchain.node.version,read('Development/.node-version').trim());
  assert.equal(toolchain.policy.ciPolicy,'disabled');
  assert.equal(app.engines.node,'^'+toolchain.node.version);
  assert.equal(manifest.engines.node,app.engines.node);
  assert.equal(app.engines.npm,toolchain.npm.version);
  assert.equal(manifest.engines.npm,app.engines.npm);
  if(existsSync(resolve(root,'.github/workflows')))assert.equal(filesBelow(root,'.github/workflows').filter(path=>/\.ya?ml$/i.test(path)).length,0,'CI must remain off');
  const bootstrap=read('Development/apps/desktop/src-tauri/src/app_bootstrap.rs');
  for(const name of ['lazy_tester_session','lazy_tester_profile_binding','lazy_tester_present','lazy_tester_previews'])assert.ok(bootstrap.includes('commands::native_clients::'+name+','),name);
  assert.ok(!/commands::virtual_clients::|commands::setup_tools::|commands::window_arrangement::/.test(bootstrap),'Legacy handlers must remain unregistered');
  const invoked=bootstrap.split('.invoke_handler(tauri::generate_handler![')[1]?.split('])')[0]??'';
  const exposed=[...invoked.matchAll(/commands::native_clients::([a-z_]+),/g)].map(m=>m[1]);
  assert.deepEqual(exposed,[
    'lazy_tester_session','lazy_tester_close_one','lazy_tester_profile_binding',
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
  assert.ok(read('Development/apps/desktop/src-tauri/src/engine/instance_catalog.rs').includes('SANDBOXIE_RETIRED'),
    'Unapproved legacy game-launch routes must remain refused');
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
