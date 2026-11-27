import {lstatSync,readdirSync,readFileSync} from 'node:fs';
import {dirname,resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {mergePatch} from './check-native-preview-package.mjs';
import {processRoles,roleMuiNames} from './prepare-process-roles.mjs';
const root=resolve(fileURLToPath(new URL('..',import.meta.url)));
const read=path=>readFileSync(resolve(root,path),'utf8');
// The compiled UI is not confidential, but original source and debugging
// artifacts must not be embedded by Tauri. Inspect only the generated dist.
export function verifyFrontendPayload(directory=resolve(root,'dist')) {
  const fail=reason=>{throw new Error('LAZY_TESTER_FRONTEND_REFUSED: '+reason);};
  for(let path=resolve(directory);;path=dirname(path)){
    const info=lstatSync(path);
    if(!info.isDirectory()||info.isSymbolicLink())fail('Expected ordinary output directories');
    if(dirname(path)===path)break;
  }
  const entries=readdirSync(directory).sort();
  if(entries.join(',')!=='assets,index.html')fail('Expected only index.html and assets');
  const assets=resolve(directory,'assets');
  const assetInfo=lstatSync(assets);
  if(!assetInfo.isDirectory()||assetInfo.isSymbolicLink())fail('Expected ordinary assets directory');
  const names=readdirSync(assets);
  if(!names.length||names.length>64)fail('Unexpected asset count');
  let js=false,css=false;
  const inspect=(file)=>{
    const info=lstatSync(file);
    if(!info.isFile()||info.isSymbolicLink()||info.nlink!==1||info.size<1||info.size>16*1024*1024)
      fail('Expected bounded ordinary generated file');
    const body=readFileSync(file,'utf8');
    if(/(?:\/\/[#@]|\/\*[#@])\s*sourceMappingURL\s*=/.test(body))
      fail('Source map directive is not allowed');
  };
  inspect(resolve(directory,'index.html'));
  for(const name of names){
    if(!/^[A-Za-z0-9_-]+-[A-Za-z0-9_-]{8,}\.(?:js|css)$/.test(name)&&
       !/^app-icon-[A-Za-z0-9_-]{8,}\.svg$/.test(name))fail('Unexpected generated asset');
    inspect(resolve(assets,name));
    js ||= name.endsWith('.js'); css ||= name.endsWith('.css');
  }
  if(!js||!css)fail('Compiled UI JavaScript and CSS are required');
}

// Source-layout witness only. The native resolver and its Windows fixtures
// remain the authorities for actual catalog, identity and data admission.
export function verifyNativeProfileLayout(profiles) {
  const canonicalFirst = [
    'let (root, missing) = resolve_location(home, &locator_path(home))?;',
    'if !missing { return Ok((root, false, false)); }',
    'let (root, missing) = resolve_location(home, &home.join("App").join(LOCATOR))?;',
    'Ok((root, missing, !missing))',
  ].join('\n    ');
  if (!profiles.includes('const LOCATOR: &str = "data-location.json";') ||
      !profiles.includes('const CATALOG: &str = "profiles.json";') ||
      !profiles.includes('home.join("Data/Settings").join(LOCATOR)') ||
      !profiles.includes(canonicalFirst) ||
      !profiles.includes('root.join(format!("Launchers/Minecraft-{slot}"))') ||
      !profiles.includes('let profiles = if catalog.schema == 2 {\n        "Launchers"\n    } else {\n        "Profiles"\n    };')) {
    throw new Error('LAZY_TESTER_PACKAGE_REFUSED: Native private profile owner differs from package policy');
  }
}
// Static source witness only: actual NSIS execution/upgrade is a separate gate.
export function verifyInstallerLayout(hooks) {
  const layout=hooks.slice(hooks.indexOf('!macro LAZY_TESTER_APP_LAYOUT'),hooks.indexOf('!macro LAZY_TESTER_REQUIRE_IDLE'));
  const registered=layout.indexOf('ReadRegStr $0 HKCU "Software\\MIVUBI-STD\\Lazy Tester" ""');
  const preserve=layout.indexOf('StrCmp $1 $0 lazy_tester_layout_done');
  const refuse=layout.indexOf('Abort',preserve);
  const append=layout.indexOf('StrCpy $INSTDIR "$INSTDIR\\App"');
  if(registered<0||preserve<=registered||refuse<=preserve||append<=refuse||
     !layout.includes('GetFullPathName $1 "$INSTDIR"')||
     !layout.includes('GetFullPathName $0 "$0"'))
    throw new Error('LAZY_TESTER_PACKAGE_REFUSED: Registered install location must be preserved before new layout selection');
}
export function verifyCandidateReleasePolicy(release) {
  if(release?.publicationEnabled!==false||release.requiresAuthenticodeBeforePublication!==true)
    throw new Error('LAZY_TESTER_PACKAGE_REFUSED: Candidate-only publication hold and Authenticode requirement must remain enabled');
}
export function verifyLazyTesterPackage(patch=JSON.parse(read('src-tauri/tauri.lazy-tester.conf.json'))) {
  const base=JSON.parse(read('src-tauri/tauri.conf.json'));
  const policy=JSON.parse(read('../../engine/distribution/lazy-tester-package.json'));
  verifyCandidateReleasePolicy(policy.release);
  const config=mergePatch(base,patch);
  const fail=(ok,reason)=>{if(!ok)throw new Error('LAZY_TESTER_PACKAGE_REFUSED: '+reason);};
  fail(config.productName===policy.productName&&config.identifier===policy.identifier&&config.mainBinaryName===policy.mainBinaryName,'Identity mismatch');
  fail(config.identifier!==base.identifier&&config.identifier!==policy.migration.previousIdentifier,'A new installer must not overwrite legacy product identity');
  fail(config.bundle.windows.nsis.installMode==='currentUser'&&config.bundle.windows.nsis.startMenuFolder==='Lazy Tester','Per-user native installation required');
  fail(config.bundle.windows.nsis.installerHooks==='windows/lazy-tester-hooks.nsh','Native preservation hooks required');
  fail(!policy.migration.installerMayMigrateData&&!policy.migration.installerMayDeleteData&&!policy.migration.installerMayStopMinecraft&&!policy.runtime.profileBindingIncluded,'Runtime data must remain outside the package');
  const allowed={
    'package-resources/lazy-tester/bootstrap-compat.dll':'independent-bootstrap/bootstrap-compat.dll',
    'package-resources/lazy-tester/bootstrap-enroll.dll':'independent-bootstrap/bootstrap-enroll.dll',
    '../../../engine/distribution/lazy-tester-package.json':'distribution/lazy-tester-package.json',
    'package-resources/lazy-tester/en-US/':'en-US/',
  };
  fail(JSON.stringify(Object.entries(config.bundle.resources).sort())===JSON.stringify(Object.entries(allowed).sort()),'Unexpected or missing native payload');
  fail(config.build.beforeBundleCommand==='node scripts/prepare-process-roles.mjs','Process role metadata hook required');
  fail(config.build.frontendDist==='../dist'&&config.build.beforeBuildCommand==='npm run build:frontend',
    'Only the checked compiled frontend may be embedded');
  fail(Object.keys(base.bundle.resources).length===0&&Object.values(patch.bundle.resources).every(value=>value!==null),'Shared bundle must remain payload-free; overlay may contain only native assets');
  fail(Array.isArray(config.bundle.externalBin)&&config.bundle.externalBin.length===0,'Legacy sidecars forbidden');
  fail(JSON.stringify(config.app.security)===JSON.stringify(base.app.security)&&!Object.hasOwn(config,'plugins'),'Desktop security must remain unchanged');
  // Canonical schema 2 and historical flat/schema 1 paths must stay distinct.
  // This gate observes source only; it never migrates or recreates user data.
  const runtime=policy.runtime;
  fail(runtime.dataLocatorRelativePath==='Data/Settings/data-location.json' &&
    runtime.legacyDataLocatorRelativePath==='App/data-location.json' &&
    runtime.profileCatalogRelativePath==='Data/profiles.json' &&
    runtime.newProfileDirectories==='Data/Launchers/Minecraft-1..5' &&
    runtime.legacyProfileDirectories==='Data/Profiles/Minecraft-1..5',
    'Canonical and legacy data paths must stay distinct');
  verifyNativeProfileLayout(read('src-tauri/src/engine/independent_bootstrap/product_profiles.rs'));
  const hooks=read('src-tauri/windows/lazy-tester-hooks.nsh');
  verifyInstallerLayout(hooks);
  fail(hooks.includes('OpenMutexW')&&hooks.includes('Local\\MivubiEducationStartupObservation')&&hooks.includes('NSIS_HOOK_PREINSTALL')&&hooks.includes('NSIS_HOOK_PREUNINSTALL')&&hooks.includes('StrCpy $DeleteAppDataCheckboxState 0'),'Idle/data preservation hooks missing');
  fail(hooks.includes('MUI_CUSTOMFUNCTION_GUIINIT')&&hooks.includes('$PROFILE\\Lazy Tester\\App')&&hooks.includes('!insertmacro LAZY_TESTER_APP_LAYOUT')&&runtime.defaultDataRelativePath==='Data'&&runtime.applicationSubdirectory==='App'&&
    hooks.includes('IfFileExists "$INSTDIR\\data-location.json" lazy_tester_layout_done 0'),
    'Visible App/Data and historical flat-install preservation hooks required');
  fail(!/^\s*(Exec|ExecWait|RMDir|Delete|DeleteRegKey|DeleteRegValue)\b/im.test(hooks),'Installer may not run tools or delete data');
  const scripts=JSON.parse(read('package.json')).scripts;
  fail(scripts['build:frontend']==='vite build && node scripts/check-lazy-tester-package.mjs --frontend',
    'Frontend build must verify generated output before native compilation');
  const installedResources=Object.values(allowed).flatMap(name=>name==='en-US/'?roleMuiNames:[name]);
  fail(JSON.stringify([...installedResources,'lazy-tester.exe',...processRoles.map(role=>role.file)].sort())===JSON.stringify([...policy.requiredFiles].sort()),
    'Runtime file allowlist differs from distribution contract');
  fail(scripts['build:app']==='npm run build:lazy-tester'&&scripts['build:lazy-tester'].includes('prepare:lazy-payload')&&scripts['build:lazy-tester'].includes('check:lazy-package')&&scripts['build:lazy-tester'].includes('--features lazy-tester-package')&&scripts['build:lazy-tester'].includes('--config src-tauri/tauri.lazy-tester.conf.json')&&scripts['build:lazy-tester'].includes('-- --locked'),'Native build route required');
  fail(scripts['dev:app']==='npm run dev:lazy-tester'&&scripts['dev:lazy-tester']===scripts['build:lazy-tester'].replace('tauri build --target x86_64-pc-windows-msvc --bundles nsis','tauri dev --target x86_64-pc-windows-msvc'),'Development must use the same reviewed native configuration, feature and payload as the product build');
  return {config,policy};
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  if(process.argv.length===3&&process.argv[2]==='--frontend'){
    verifyFrontendPayload();
    process.stdout.write('Compiled frontend output admitted; original source files and maps excluded by layout check\n');
  }else if(process.argv.length===2){
    verifyLazyTesterPackage();process.stdout.write('Lazy Tester native installer source contract verified; profile bindings excluded\n');
  }else throw new Error('Unsupported package check arguments');
}

