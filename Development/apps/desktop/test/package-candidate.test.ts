import {describe,expect,it} from 'vitest';
import {execFileSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,readdirSync,rmSync,linkSync,lstatSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {assembleCandidate,sourceRevision,stageLocalApp,verifyDesktopStartup} from '../scripts/package-lazy-tester.mjs';

describe('Development source and distribution candidates',()=>{
  it('requires Explorer-parented Desktop UI evidence and rejects build-tool-only success',()=>{
    const ready={event:'desktop-startup-observation',sourceCommit:'a'.repeat(40),processId:42,parentProcessId:7,explorerParent:true,connected:true,issue:'',cards:5,enabled:5};
    expect(verifyDesktopStartup(JSON.stringify(ready),ready.sourceCommit)).toEqual(ready);
    for(const change of [{connected:false},{issue:'Windows error 2'},{enabled:0},{cards:0},{processId:0},{parentProcessId:0},{explorerParent:false},{sourceCommit:''},{sourceCommit:'b'.repeat(40)}])expect(()=>verifyDesktopStartup(JSON.stringify({...ready,...change}),ready.sourceCommit)).toThrow();
    for(const old of [{schema:1,configured:true,issue:null},{event:'startup-observation',processId:42,connection:'Connected',issue:'',cards:5,enabled:5,diagnosticOnly:true}])expect(()=>verifyDesktopStartup(JSON.stringify(old),ready.sourceCommit)).toThrow();
  });
  it('stages only app payloads, preserves the location and old app, and never touches Data',()=>{
    const root=mkdtempSync(join(tmpdir(),'lazy-local-app-'));
    try {
      const release=join(root,'build');
      for (const name of ['lazy-tester.exe','independent-bootstrap/bootstrap-compat.dll','independent-bootstrap/bootstrap-enroll.dll','distribution/lazy-tester-package.json']) {
        const path=join(release,name);mkdirSync(dirname(path),{recursive:true});
        writeFileSync(path,name.endsWith('.json')?'{}':'MZfixture');
      }
      writeFileSync(join(release,'private.bin'),'never copy');
      writeFileSync(join(release,'Lazy Tester.exe'),'MZlauncher');
      linkSync(join(release,'lazy-tester.exe'),join(release,'cargo-artifact.exe'));
      const first=stageLocalApp(root,release,'a'.repeat(40));
      expect(lstatSync(join(first.app,'lazy-tester.exe')).nlink).toBe(1);
      expect(readFileSync(join(root,'Lazy Tester.exe'),'utf8')).toBe('MZlauncher');
      expect(readdirSync(first.data)).toEqual([]);
      expect(readdirSync(first.app).sort()).toEqual(['build-info.json','distribution','independent-bootstrap','lazy-tester.exe']);
      writeFileSync(join(first.app,'data-location.json'),'{"schema":1,"path":"D:/My Data"}');
      writeFileSync(join(first.app,'performance.json'),'{"activeFps":60,"backgroundFps":5}');
      writeFileSync(join(first.app,'graphics.json'),'["High","Low","Low","Low","Low"]');
      writeFileSync(join(first.data,'world-fixture'),'preserve');
      writeFileSync(join(release,'lazy-tester.exe'),'MZupdated');
      const next=stageLocalApp(root,release,'b'.repeat(40));
      expect(readFileSync(join(next.app,'data-location.json'),'utf8')).toContain('D:/My Data');
      expect(readFileSync(join(next.app,'performance.json'),'utf8')).toContain('"activeFps":60');
      expect(readFileSync(join(next.app,'graphics.json'),'utf8')).toContain('"High"');
      expect(readFileSync(join(next.data,'world-fixture'),'utf8')).toBe('preserve');
      expect(readFileSync(join(next.backup!,'lazy-tester.exe'),'utf8')).toBe('MZfixture');
      writeFileSync(join(root,'Lazy Tester.exe'),'MZunrelated');
      expect(()=>stageLocalApp(root,release,'c'.repeat(40))).toThrow(/unrelated/);
      writeFileSync(join(root,'Lazy Tester.exe'),'MZlauncher');
      writeFileSync(join(release,'lazy-tester.exe'),'invalid');
      expect(()=>stageLocalApp(root,release,'c'.repeat(40))).toThrow(/executable/);
      expect(readFileSync(join(next.app,'lazy-tester.exe'),'utf8')).toBe('MZupdated');
    } finally {
      expect(dirname(resolve(root))).toBe(resolve(tmpdir()));rmSync(root,{recursive:true});
    }
  });
  it('exports only the installer and receipts, refuses reuse and never copies adjacent private data',()=>{
    const root=mkdtempSync(join(tmpdir(),'lazy-candidate-'));
    try {
      const build=join(root,'build');mkdirSync(build);
      const installer=join(build,'setup.exe');
      const bytes=Buffer.alloc(128);bytes.write('MZ');writeFileSync(installer,bytes);
      writeFileSync(join(build,'profile-binding-five.bin'),'private fixture');
      writeFileSync(join(build,'source.rs'),'developer source');
      const args=[root,installer,'a'.repeat(40),'0.1.0','20261026T070000000Z'] as const;
      const {output,manifest}=assembleCandidate(...args);
      expect(readdirSync(output).sort()).toEqual(['Lazy-Tester-0.1.0-windows-x86_64-setup.exe','SHA256SUMS.txt','package-manifest.json']);
      expect(readFileSync(join(output,manifest.installer.name))).toEqual(bytes);
      expect(readFileSync(join(output,'SHA256SUMS.txt'),'utf8')).toContain(manifest.installer.sha256);
      expect(JSON.parse(readFileSync(join(output,'package-manifest.json'),'utf8')).releaseReady).toBe(false);
      expect(()=>assembleCandidate(...args)).toThrow();
      expect(readFileSync(join(build,'profile-binding-five.bin'),'utf8')).toBe('private fixture');
      expect(()=>assembleCandidate(root,installer,'a'.repeat(40),'../bad','20261026T070000000Z')).toThrow(/identity/);
      writeFileSync(installer,Buffer.alloc(128));
      expect(()=>assembleCandidate(root,installer,'a'.repeat(40),'0.1.0','20261026T070001000Z')).toThrow(/executable/);
    } finally {
      expect(dirname(resolve(root))).toBe(resolve(tmpdir()));rmSync(root,{recursive:true});
    }
  });
  it('refuses changed source and other branches before packaging',()=>{
    const root=mkdtempSync(join(tmpdir(),'lazy-package-source-'));
    const git=(...args:string[])=>execFileSync('git',args,{cwd:root,encoding:'utf8'}).trim();
    try {
      git('init','--initial-branch=Local');
      writeFileSync(join(root,'README.md'),'source');git('add','README.md');
      git('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','-m','fixture');
      const sha=sourceRevision(root);expect(sha).toMatch(/^[a-f0-9]{40}$/);
      writeFileSync(join(root,'README.md'),'changed');expect(()=>sourceRevision(root)).toThrow(/clean source/);
      writeFileSync(join(root,'README.md'),'source');
      writeFileSync(join(root,'untracked.txt'),'draft');expect(()=>sourceRevision(root)).toThrow(/clean source/);
      rmSync(join(root,'untracked.txt'));git('checkout','-b','unrelated');
      expect(()=>sourceRevision(root)).toThrow(/Local/);
      git('checkout','--detach',sha);expect(sourceRevision(root)).toBe(sha);
    } finally {
      expect(dirname(resolve(root))).toBe(resolve(tmpdir()));rmSync(root,{recursive:true});
    }
  });
});
