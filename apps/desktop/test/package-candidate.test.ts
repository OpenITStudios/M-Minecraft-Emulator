import {describe,expect,it} from 'vitest';
import {execFileSync} from 'node:child_process';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,readdirSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import {assembleCandidate,sourceRevision} from '../scripts/package-lazy-tester.mjs';

describe('Development source and distribution candidates',()=>{
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
