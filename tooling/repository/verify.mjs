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
    'docs/system/implementation-map.md','docs/system/legacy-retirement.md','VERSION',
    'apps/desktop/src-tauri/Cargo.lock','engine/runtime-core/Cargo.lock'])assert.ok(existsSync(resolve(root,path)),path);
  const sourcePaths=filesBelow(root,'apps/desktop/src');
  assert.ok(sourcePaths.every(path=>/\.(?:ts|svelte|css)$/.test(path)),'Unexpected binary/private frontend material');
  const sources=Object.fromEntries(sourcePaths.map(path=>[path.slice('apps/desktop/src/'.length),read(path)]));
  const reached=traceFrontend(sources);
  assert.ok(sources['main.ts'].includes('mount(LazyTesterApp, { target });'),'Lazy Tester must be the mounted app');
  assert.ok(!reached.some(path=>/ownedProviderApi|nativeFeasibilityApi|virtualClientsApi/.test(path)),'Retired provider bridge must not be active');
  const app=json('apps/desktop/package.json'),manifest=json('package.json'),tauri=json('apps/desktop/src-tauri/tauri.conf.json');
  const version=read('VERSION').trim();
  for(const actual of [manifest.version,app.version,tauri.version,
    /^version\s*=\s*"([^"]+)"/m.exec(read('apps/desktop/src-tauri/Cargo.toml'))?.[1],
    /^version\s*=\s*"([^"]+)"/m.exec(read('engine/runtime-core/Cargo.toml'))?.[1]])assert.equal(actual,version,'Version parity');
  checkDependencyLock(app,json('apps/desktop/package-lock.json'));
  const toolchain=json('toolchain.json');
  assert.equal(toolchain.node.version,read('.node-version').trim());
  assert.equal(toolchain.policy.ciPolicy,'disabled');
  assert.equal(app.engines.node,'^'+toolchain.node.version);
  assert.equal(manifest.engines.node,app.engines.node);
  assert.equal(app.engines.npm,toolchain.npm.version);
  assert.equal(manifest.engines.npm,app.engines.npm);
  if(existsSync(resolve(root,'.github/workflows')))assert.equal(filesBelow(root,'.github/workflows').filter(path=>/\.ya?ml$/i.test(path)).length,0,'CI must remain off');
  const bootstrap=read('apps/desktop/src-tauri/src/app_bootstrap.rs');
  for(const name of ['lazy_tester_session','lazy_tester_profile_binding','lazy_tester_present','lazy_tester_previews'])assert.ok(bootstrap.includes('commands::native_clients::'+name+','),name);
  assert.ok(!/commands::virtual_clients::|commands::setup_tools::|commands::window_arrangement::/.test(bootstrap),'Legacy handlers must remain unregistered');
  const commands=read('apps/desktop/src-tauri/src/commands/mod.rs');
  assert.ok(!/^\s*pub mod (virtual_clients|setup_tools|window_arrangement);/m.test(commands));
  const vm=read('engine/runtime-core/src/command.rs');
  const paused=vm.match(/const PAUSED_VM_COMMANDS:[\s\S]*?= &\[([\s\S]*?)\];/)?.[1]??'';
  for(const command of ['provision','reprovision','start','start-client','stop','restart','reset','launch-minecraft'])assert.ok(paused.includes('"'+command+'"'),'VM pause gate missing: '+command);
  const pauseGate=vm.indexOf('if legacy_operation_paused(command, args)');
  assert.ok(pauseGate>=0&&vm.indexOf('let app = VirtualClients;')>pauseGate,'VM mutations must be refused before provider discovery');
  assert.ok(read('apps/desktop/src-tauri/src/engine/instance_catalog.rs').includes('SANDBOXIE_RETIRED'),'Sandboxie new-session refusal must remain');
  assert.ok(read('apps/desktop/scripts/prepare-package-binaries.mjs').includes('const LEGACY_PACKAGE_RELEASE_APPROVED = false;'),'Legacy package hold must remain');
  verifyNativePreviewPackage();verifyLazyTesterPackage();
  const tracked=execFileSync('git',['ls-files','-z'],{cwd:root,encoding:'utf8'}).split('\0').filter(Boolean);
  assert.deepEqual(checkTracked(tracked),[]);
  assert.ok(!tracked.some(path=>/(?:replay-profile-(?:pair|five)|profile-binding-five)\.bin$/.test(path)),'Private bindings must never be tracked');
  for(const path of ['workspace/private/profile.json','workspace/packages/installer.exe','workspace/runtime/replay-profile-five.bin',
    'Data/profiles.json','App/data-location.json','Logs/session.log']) {
    execFileSync('git',['check-ignore','--no-index','-q','--',path],{cwd:root});
  }
  if(g0)assert.equal(execFileSync('git',['ls-tree','-r','--name-only','HEAD','--','apps/desktop/src-tauri/Cargo.lock'],{cwd:root,encoding:'utf8'}).trim(),'apps/desktop/src-tauri/Cargo.lock');
  assert.deepEqual(checkLinks(root,filesBelow(root,'docs').filter(path=>path.endsWith('.md'))),[]);
  const handoff=read('planning/development.md');
  assert.ok(handoff.startsWith('# Current Development Handoff')&&handoff.length<8000,'Keep one concise handoff');
  console.log('Lazy Tester repository contract: PASS ('+reached.length+' reachable frontend files; source/layout evidence only)');
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url))verifyRepository(resolve(dirname(fileURLToPath(import.meta.url)),'../..'),process.argv.includes('--g0'));
