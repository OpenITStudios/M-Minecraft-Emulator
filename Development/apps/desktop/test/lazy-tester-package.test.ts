import {describe,it,expect} from 'vitest';
import {createHash} from 'node:crypto';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,existsSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,dirname,resolve} from 'node:path';
import {verifyLazyTesterPackage} from '../scripts/check-lazy-tester-package.mjs';
import {reviewedDigests,stageNativePayload} from '../scripts/prepare-native-payload.mjs';
import {processRoles,roleMuiNames} from '../scripts/prepare-process-roles.mjs';

describe('Lazy Tester native package',()=>{
  it('keeps independent installer identity and excludes private bindings and legacy payloads',()=>{
    const {config,policy}=verifyLazyTesterPackage();
    expect(config.productName).toBe('Lazy Tester');
    expect(config.identifier).toBe('com.mivubi.lazytester');
    expect(config.bundle.externalBin).toEqual([]);
    const packaged=Object.values(config.bundle.resources).flatMap(name=>name==='en-US/'?roleMuiNames:[name]);
    expect([...packaged,'lazy-tester.exe',...processRoles.map(r=>r.file)].sort()).toEqual([...policy.requiredFiles].sort());
    expect(policy.migration.installerMayDeleteData).toBe(false);
    expect(policy.runtime.defaultDataRelativePath).toBe('Data');
    expect(policy.runtime.dataLocatorRelativePath).toBe('App/data-location.json');
    const hooks=readFileSync(new URL('../src-tauri/windows/lazy-tester-hooks.nsh',import.meta.url),'utf8');
    expect(hooks).toContain('MUI_CUSTOMFUNCTION_GUIINIT');
    expect(hooks).toContain('$PROFILE\\Lazy Tester\\App');
    expect(hooks).toContain('IfFileExists "$INSTDIR\\lazy-tester.exe" lazy_tester_layout_done 0');
    expect(hooks).toContain('IfFileExists "$INSTDIR\\data-location.json" lazy_tester_layout_done 0');
    const extra=structuredClone(JSON.parse(readFileSync(new URL('../src-tauri/tauri.lazy-tester.conf.json',import.meta.url),'utf8')));
    extra.bundle.resources['private/replay-profile-five.bin']='independent-bootstrap/replay-profile-five.bin';
    expect(()=>verifyLazyTesterPackage(extra)).toThrow(/Unexpected/);
    extra.bundle.resources=structuredClone(config.bundle.resources);
    extra.identifier=policy.migration.previousIdentifier;
    expect(()=>verifyLazyTesterPackage(extra)).toThrow(/Identity/);
  });
  it('derives hashes from the exact runtime policy and refuses absent or malformed review',()=>{
    const source=readFileSync(new URL('../src-tauri/src/engine/independent_bootstrap.rs',import.meta.url),'utf8');
    expect(reviewedDigests(source)).toEqual({
      'bootstrap-compat.dll':'e90b312b3f12915098a3571a647050d5e3a1d082a0f831242d92ffac5514653f',
      'bootstrap-enroll.dll':'51517528886483ba89f1b93b448c1ddd349f26f0e48e5322db434cbfe9c4de99',
    });
    expect(()=>reviewedDigests('const REVIEWED_EDUCATION_BOOTSTRAP: Option<PayloadDigests> = None;')).toThrow();
    expect(()=>reviewedDigests(source.replace(/enroller: \[\s*0x[0-9a-f]{2},/,'enroller: ['))).toThrow();
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
      expect(stageNativePayload(source,destination,expected)).toHaveLength(2);
      expect(existsSync(join(destination,'replay-profile-five.bin'))).toBe(false);
      expect(stageNativePayload(source,destination,expected)).toHaveLength(2);
      writeFileSync(join(source,'bootstrap-compat.dll'),Buffer.alloc(128,0));
      expect(()=>stageNativePayload(source,join(root,'refused'),expected)).toThrow(/reviewed/);
      expect(existsSync(join(root,'refused'))).toBe(false);
      expect(readFileSync(join(source,'replay-profile-five.bin'),'utf8')).toBe('private fixture');
      writeFileSync(join(source,'bootstrap-compat.dll'),Buffer.alloc(128,'bootstrap-compat.dll'.length));
      writeFileSync(join(destination,'bootstrap-enroll.dll'),Buffer.alloc(128,0));
      expect(()=>stageNativePayload(source,destination,expected)).toThrow(/Stale/);
    }finally{
      expect(dirname(resolve(root))).toBe(resolve(tmpdir()));
      rmSync(root,{recursive:true,force:false});
    }
  });
});

