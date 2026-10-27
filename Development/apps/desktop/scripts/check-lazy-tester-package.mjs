import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
import {mergePatch} from './check-native-preview-package.mjs';
const root=resolve(fileURLToPath(new URL('..',import.meta.url)));
const read=path=>readFileSync(resolve(root,path),'utf8');
export function verifyLazyTesterPackage(patch=JSON.parse(read('src-tauri/tauri.lazy-tester.conf.json'))) {
  const base=JSON.parse(read('src-tauri/tauri.conf.json'));
  const policy=JSON.parse(read('../../engine/distribution/lazy-tester-package.json'));
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
  fail(Object.keys(base.bundle.resources).every(key=>patch.bundle.resources[key]===null),'Every legacy resource must be removed');
  fail(Array.isArray(config.bundle.externalBin)&&config.bundle.externalBin.length===0,'Legacy sidecars forbidden');
  fail(JSON.stringify(config.app.security)===JSON.stringify(base.app.security)&&!Object.hasOwn(config,'plugins'),'Desktop security must remain unchanged');
  const hooks=read('src-tauri/windows/lazy-tester-hooks.nsh');
  fail(hooks.includes('OpenMutexW')&&hooks.includes('Local\\MivubiEducationStartupObservation')&&hooks.includes('NSIS_HOOK_PREINSTALL')&&hooks.includes('NSIS_HOOK_PREUNINSTALL')&&hooks.includes('StrCpy $DeleteAppDataCheckboxState 0'),'Idle/data preservation hooks missing');
  fail(hooks.includes('MUI_CUSTOMFUNCTION_GUIINIT')&&hooks.includes('$PROFILE\\Lazy Tester\\App')&&hooks.includes('!insertmacro LAZY_TESTER_APP_LAYOUT')&&policy.runtime.defaultDataRelativePath==='Data'&&policy.runtime.applicationSubdirectory==='App','Visible App/Data installation layout required');
  fail(!/^\s*(Exec|ExecWait|RMDir|Delete|DeleteRegKey|DeleteRegValue)\b/im.test(hooks),'Installer may not run tools or delete data');
  const scripts=JSON.parse(read('package.json')).scripts;
  fail(scripts['build:app']==='npm run build:lazy-tester'&&scripts['build:lazy-tester'].includes('prepare:lazy-payload')&&scripts['build:lazy-tester'].includes('check:lazy-package')&&scripts['build:lazy-tester'].includes('--features lazy-tester-package')&&scripts['build:lazy-tester'].includes('--config src-tauri/tauri.lazy-tester.conf.json')&&scripts['build:lazy-tester'].includes('-- --locked'),'Native build route required');
  fail(scripts['dev:app']==='npm run dev:lazy-tester'&&scripts['dev:lazy-tester']===scripts['build:lazy-tester'].replace('tauri build --target x86_64-pc-windows-msvc --bundles nsis','tauri dev --target x86_64-pc-windows-msvc'),'Development must use the same reviewed native configuration, feature and payload as the product build');
  return {config,policy};
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url)){
  verifyLazyTesterPackage();process.stdout.write('Lazy Tester native installer source contract verified; profile bindings excluded\n');
}

