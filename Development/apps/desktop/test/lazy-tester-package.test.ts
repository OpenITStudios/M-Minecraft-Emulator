import {describe,it,expect} from 'vitest';
import {createHash} from 'node:crypto';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,existsSync,rmSync,linkSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname,resolve} from 'node:path';
import {verifyLazyTesterPackage,verifyNativeProfileLayout,verifyFrontendPayload,verifyInstallerLayout,verifyCandidateReleasePolicy} from '../scripts/check-lazy-tester-package.mjs';
import {reviewedDigests,readReviewedDigests,stageNativePayload,verifyNativePayload} from '../scripts/prepare-native-payload.mjs';
import {processRoles,roleMuiNames,verifyRoleResources} from '../scripts/prepare-process-roles.mjs';

describe('Lazy Tester native package',()=>{
  it('validates native DLL admission before startup can mutate product profiles',()=>{
    // Source-order fixture only; no DLL, Windows registry, profile or game I/O.
    const source=readFileSync(new URL('../src-tauri/src/engine/independent_bootstrap/session_controller.rs',import.meta.url),'utf8');
    const start=source.indexOf('fn prepare() -> io::Result<Self> {');
    expect(start).toBeGreaterThanOrEqual(0);
    const end=source.indexOf('fn reap(&mut self)',start);
    expect(end).toBeGreaterThan(start);
    const startup=source.slice(start,end);
    const native=startup.indexOf('drop(BootstrapLibrary::reviewed()?);');
    const profiles=startup.indexOf('registered_slots::<5>()?');
    expect(native).toBeGreaterThanOrEqual(0);
    expect(profiles).toBeGreaterThan(native);
  });
  it('keeps independent installer identity and excludes private bindings and legacy payloads',()=>{
    const {config,policy}=verifyLazyTesterPackage();
    expect(config.productName).toBe('Lazy Tester');
    expect(config.identifier).toBe('com.mivubi.lazytester');
    expect(config.bundle.externalBin).toEqual([]);
    const packaged=Object.values(config.bundle.resources).flatMap(name=>name==='en-US/'?roleMuiNames:[name]);
    expect([...packaged,'lazy-tester.exe',...processRoles.map(r=>r.file)].sort()).toEqual([...policy.requiredFiles].sort());
    expect(policy.migration.installerMayDeleteData).toBe(false);
    expect(policy.runtime.defaultDataRelativePath).toBe('Data');
    expect(policy.runtime.dataLocatorRelativePath).toBe('Data/Settings/data-location.json');
    expect(policy.runtime.legacyDataLocatorRelativePath).toBe('App/data-location.json');
    expect(policy.runtime.newProfileDirectories).toBe('Data/Launchers/Minecraft-1..5');
    expect(policy.runtime.legacyProfileDirectories).toBe('Data/Profiles/Minecraft-1..5');
    expect(policy.runtime.profileCatalogRelativePath).toBe('Data/profiles.json');
    const hooks=readFileSync(new URL('../src-tauri/windows/lazy-tester-hooks.nsh',import.meta.url),'utf8');
    expect(hooks).toContain('MUI_CUSTOMFUNCTION_GUIINIT');
    expect(hooks).toContain('$PROFILE\\Lazy Tester\\App');
    expect(hooks).toContain('IfFileExists "$INSTDIR\\lazy-tester.exe" lazy_tester_layout_done 0');
    expect(hooks).toContain('IfFileExists "$INSTDIR\\data-location.json" lazy_tester_layout_done 0');
    const extra=structuredClone(JSON.parse(readFileSync(new URL('../src-tauri/tauri.lazy-tester.conf.json',import.meta.url),'utf8')));
    extra.bundle.resources['private/replay-profile-five.bin']='independent-bootstrap/replay-profile-five.bin';
    expect(()=>verifyLazyTesterPackage(extra)).toThrow(/Unexpected/);
    extra.bundle.resources=structuredClone(config.bundle.resources);
    extra.bundle.resources['package-resources/lazy-tester/']='';
    expect(()=>verifyLazyTesterPackage(extra)).toThrow(/Unexpected/);
    extra.bundle.resources=structuredClone(config.bundle.resources);
    extra.build.frontendDist='../src';
    expect(()=>verifyLazyTesterPackage(extra)).toThrow(/compiled frontend/);
    delete extra.build.frontendDist;
    extra.identifier=policy.migration.previousIdentifier;
    expect(()=>verifyLazyTesterPackage(extra)).toThrow(/Identity/);
  });
  it('preserves registered flat installs even after their executable is removed',()=>{
    const hooks=readFileSync(new URL('../src-tauri/windows/lazy-tester-hooks.nsh',import.meta.url),'utf8');
    expect(()=>verifyInstallerLayout(hooks)).not.toThrow();
    const changed=hooks.replace('StrCmp $1 $0 lazy_tester_layout_done','StrCmp $1 $0 0');
    expect(changed).not.toBe(hooks);
    expect(()=>verifyInstallerLayout(changed)).toThrow(/Registered install location/);
  });
  it('admits an empty locale before compilation and requires the exact complete set before bundling',()=>{
    const dir=mkdtempSync(join(tmpdir(),'lazy-role-fixture-'));
    try{
      const locale=join(dir,'en-US');
      expect(()=>verifyRoleResources(locale)).not.toThrow();
      mkdirSync(locale);
      expect(()=>verifyRoleResources(locale)).not.toThrow();
      expect(()=>verifyRoleResources(locale,true)).toThrow(/Complete role resources/);
      for(const name of roleMuiNames)writeFileSync(join(dir,name),'fixture');
      expect(()=>verifyRoleResources(locale,true)).not.toThrow();
      const extra=join(locale,'source.rs');writeFileSync(extra,'preserve');
      expect(()=>verifyRoleResources(locale)).toThrow(/Unexpected/);
      expect(()=>verifyRoleResources(locale,true)).toThrow(/Unexpected/);
      expect(readFileSync(extra,'utf8')).toBe('preserve');rmSync(extra);
      const missing=join(dir,roleMuiNames[0]);rmSync(missing);
      expect(()=>verifyRoleResources(locale)).not.toThrow();
      expect(()=>verifyRoleResources(locale,true)).toThrow(/Complete role resources/);
      mkdirSync(missing);
      expect(()=>verifyRoleResources(locale)).toThrow(/ordinary file/);
    }finally{rmSync(dir,{recursive:true});}
  });
  it('admits only the generated frontend layout and refuses extra files or embedded source maps',()=>{
    const dir=mkdtempSync(join(tmpdir(),'lazy-frontend-fixture-'));
    try{
      const assets=join(dir,'assets');mkdirSync(assets);
      writeFileSync(join(dir,'index.html'),'<html></html>');
      const js=join(assets,'index-Ab12_cd3.js');
      writeFileSync(js,'console.log("fixture");');
      writeFileSync(join(assets,'index-Ab12_cd3.css'),'body{}');
      writeFileSync(join(assets,'app-icon-Ab12_cd3.svg'),'<svg/>');
      expect(()=>verifyFrontendPayload(dir)).not.toThrow();
      for(const name of ['source.ts','index-Ab12_cd3.js.map','private.json','debug.pdb']){
        const extra=join(assets,name);writeFileSync(extra,'preserve');
        expect(()=>verifyFrontendPayload(dir)).toThrow(/Unexpected generated asset/);
        expect(readFileSync(extra,'utf8')).toBe('preserve');rmSync(extra);
      }
      const extra=join(dir,'Development');mkdirSync(extra);
      expect(()=>verifyFrontendPayload(dir)).toThrow(/only index.html and assets/);
      rmSync(extra,{recursive:true});
      writeFileSync(js,'console.log("fixture");\n//# sourceMappingURL=data:application/json;base64,AAAA');
      expect(()=>verifyFrontendPayload(dir)).toThrow(/Source map directive/);
      writeFileSync(js,'console.log("fixture");');
      rmSync(join(assets,'index-Ab12_cd3.css'));
      expect(()=>verifyFrontendPayload(dir)).toThrow(/JavaScript and CSS/);
    }finally{rmSync(dir,{recursive:true});}
  });
  it('recognizes the current canonical-first profile owner and refuses conflicting layout witnesses',()=>{
    const source=readFileSync(new URL('../src-tauri/src/engine/independent_bootstrap/product_profiles.rs',import.meta.url),'utf8');
    expect(()=>verifyNativeProfileLayout(source)).not.toThrow();
    const canonical='let (root, missing) = resolve_location(home, &locator_path(home))?;';
    const legacy='let (root, missing) = resolve_location(home, &home.join("App").join(LOCATOR))?;';
    const inverted=source.replace(canonical,'__CANONICAL_FIXTURE__').replace(legacy,canonical).replace('__CANONICAL_FIXTURE__',legacy);
    for(const changed of [
      inverted,
      source.replace('home.join("Data/Settings").join(LOCATOR)','home.join("App").join(LOCATOR)'),
      source.replace('if !missing { return Ok((root, false, false)); }','if missing { return Ok((root, false, false)); }'),
      source.replace('let profiles = if catalog.schema == 2 {','let profiles = if catalog.schema == 1 {'),
      source.replace('const CATALOG: &str = "profiles.json";','const CATALOG: &str = "other.json";'),
    ]) {
      expect(changed).not.toBe(source);
      expect(()=>verifyNativeProfileLayout(changed)).toThrow(/profile owner differs/);
    }
  });
  it('keeps candidate publication disabled and requires Authenticode before publication',()=>{
    expect(()=>verifyCandidateReleasePolicy({publicationEnabled:false,requiresAuthenticodeBeforePublication:true})).not.toThrow();
    for(const release of [{publicationEnabled:true,requiresAuthenticodeBeforePublication:true},
      {publicationEnabled:false,requiresAuthenticodeBeforePublication:false},{publicationEnabled:false},{}])
      expect(()=>verifyCandidateReleasePolicy(release)).toThrow(/publication hold/);
  });
  it('derives hashes from the exact runtime policy and refuses absent or malformed review',()=>{
    const source=readFileSync(new URL('../src-tauri/src/engine/independent_bootstrap/reviewed_bootstrap_payload.rs',import.meta.url),'utf8');
    expect(source).toContain('const REVIEWED_EDUCATION_BOOTSTRAP: Option<PayloadDigests> = None;');
    expect(()=>reviewedDigests(source)).toThrow(/unavailable/); // v4 payload is not reviewed yet.
    expect(()=>readReviewedDigests()).toThrow(/unavailable/);
    const packaging=readFileSync(new URL('../scripts/package-lazy-tester.mjs',import.meta.url),'utf8');
    const staging=readFileSync(new URL('../scripts/prepare-native-payload.mjs',import.meta.url),'utf8');
    const readiness=readFileSync(new URL('../../../tooling/windows-toolchain/local-readiness.ps1',import.meta.url),'utf8');
    expect(packaging).toContain('const expected=readReviewedDigests();');
    expect(staging).toContain('const policy = readReviewedDigests();');
    expect(readiness).toContain('independent_bootstrap/reviewed_bootstrap_payload.rs');
    for(const consumer of [packaging,staging,readiness])
      expect(consumer).not.toContain('engine/independent_bootstrap.rs');
    const fixture='const REVIEWED_EDUCATION_BOOTSTRAP: Option<PayloadDigests> = Some(PayloadDigests {enroller: ['+
      Array(32).fill('0x11').join(',')+'], compatibility: ['+Array(32).fill('0x22').join(',')+']});';
    expect(reviewedDigests(fixture)).toEqual({
      'bootstrap-compat.dll':'22'.repeat(32),
      'bootstrap-enroll.dll':'11'.repeat(32),
    });
    expect(()=>reviewedDigests(fixture.replace('0x11,',''))).toThrow(/Invalid/);
  });
  it('stages exactly two verified DLLs, refuses changed bytes and never copies adjacent profiles',()=>{
    const root=mkdtempSync(join(tmpdir(),'lazy-package-fixture-'));
    try {
      const source=join(root,'source'),destination=join(root,'output');mkdirSync(source);
      const expected:Record<string,string>={};
      for(const name of ['bootstrap-compat.dll','bootstrap-enroll.dll']){
        const bytes=Buffer.alloc(128,name.length);writeFileSync(join(source,name),bytes);
        expected[name]=createHash('sha256').update(bytes).digest('hex');
      }
      writeFileSync(join(source,'replay-profile-five.bin'),'private fixture');
      const receipts=stageNativePayload(source,destination,expected);
      expect(receipts).toHaveLength(2);
      expect(verifyNativePayload(destination,expected)).toEqual(receipts);
      expect(existsSync(join(destination,'replay-profile-five.bin'))).toBe(false);
      expect(stageNativePayload(source,destination,expected)).toHaveLength(2);
      const linked=join(root,'linked');mkdirSync(linked);
      linkSync(join(source,'bootstrap-enroll.dll'),join(linked,'bootstrap-enroll.dll'));
      // Even exact reviewed bytes must not be signed through a shared file.
      expect(()=>stageNativePayload(source,linked,expected)).toThrow(/Stale/);
      expect(existsSync(join(linked,'bootstrap-compat.dll'))).toBe(false);
      linkSync(join(destination,'bootstrap-compat.dll'),join(root,'external-alias.dll'));
      expect(()=>verifyNativePayload(destination,expected)).toThrow(/Unsupported native DLL/);
      rmSync(join(root,'external-alias.dll'));
      writeFileSync(join(source,'bootstrap-compat.dll'),Buffer.alloc(128,0));
      expect(()=>stageNativePayload(source,join(root,'refused'),expected)).toThrow(/reviewed/);
      expect(existsSync(join(root,'refused'))).toBe(false);
      expect(readFileSync(join(source,'replay-profile-five.bin'),'utf8')).toBe('private fixture');
      writeFileSync(join(source,'bootstrap-compat.dll'),Buffer.alloc(128,'bootstrap-compat.dll'.length));
      writeFileSync(join(destination,'bootstrap-enroll.dll'),Buffer.alloc(128,0));
      expect(()=>stageNativePayload(source,destination,expected)).toThrow(/Stale/);
      // A mutation after staging (including signing) must block candidate export.
      expect(()=>verifyNativePayload(destination,expected)).toThrow(/reviewed/);
      expect(readFileSync(join(destination,'bootstrap-enroll.dll'))).toEqual(Buffer.alloc(128,0));
    }finally{
      expect(dirname(resolve(root))).toBe(resolve(tmpdir()));
      rmSync(root,{recursive:true,force:false});
    }
  });
});

