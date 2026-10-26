import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, existsSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { bootstrapFork, verifyPinnedSeed } from './bootstrap-fork.mjs';

const cmd = (args) => {
  const r = spawnSync('git', args, {encoding:'utf8'});
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
};
function createRepo() {
  const parent = mkdtempSync(join(tmpdir(), 'mivubi-source-fork-'));
  const upstream = join(parent, 'upstream');
  cmd(['init', '-b', 'main', upstream]);
  cmd(['-C', upstream, 'config', 'user.email', 'test@example.invalid']);
  cmd(['-C', upstream, 'config', 'user.name', 'Isolation Test']);
  for (let i = 0; i < 3; i++) cmd(['-C', upstream, 'commit', '--allow-empty', '-m', `history ${i}`]);
  cmd(['-C', upstream, 'tag', 'v1.18.5']);
  const expectedCommit = cmd(['-C', upstream, 'rev-parse', 'HEAD']);
  return {parent, upstream, expectedCommit, destination:join(parent,'owned-engine'), cleanup:()=>rmSync(parent,{recursive:true,force:true})};
}
test('preserves actual Git history and pins an independent branch', () => {
  const f = createRepo(); try {
    const result = bootstrapFork({source:f.upstream,destination:f.destination,expectedCommit:f.expectedCommit});
    assert.equal(result.status, 'PINNED_HISTORY_PRESERVED');
    assert.equal(result.historyCount, 3);
    assert.equal(result.binaryBuilt, false);
    assert.equal(result.pushedToGitHub, false);
    assert.equal(cmd(['-C', f.destination, 'rev-parse', 'HEAD']),f.expectedCommit);
    assert.equal(cmd(['-C', f.destination, 'rev-parse', '--is-shallow-repository']),'false');
    assert.equal(cmd(['-C', f.destination, 'remote']), 'upstream');
    assert.equal(cmd(['-C', f.destination, 'branch', '--show-current']), 'isolation/baseline-v1.18.5');
  } finally { f.cleanup(); }
});
test('refuses mismatched pinned tag without leaving destination',()=>{
  const f=createRepo();try{
    assert.throws(()=>bootstrapFork({source:f.upstream,destination:f.destination,expectedCommit:'f'.repeat(40)}),/tag\/commit mismatch/);
    assert.equal(existsSync(f.destination),false);
  }finally{f.cleanup()}
});
test('refuses existing destination and preserves the original',()=>{
  const f=createRepo();try{
    assert.throws(()=>bootstrapFork({source:f.upstream,destination:f.upstream,expectedCommit:f.expectedCommit}),/Destination exists/);
    assert.equal(cmd(['-C',f.upstream,'rev-parse','HEAD']),f.expectedCommit);
  }finally{f.cleanup()}
});
test('refuses missing tag and cleans temporary checkout',()=>{
  const f=createRepo();try{
    assert.throws(()=>bootstrapFork({source:f.upstream,destination:f.destination,expectedCommit:f.expectedCommit,tag:'v7.8.9'}),/Git command failed/);
    assert.equal(existsSync(f.destination),false);
  }finally{f.cleanup()}
});

test('refuses dangling symlink as a destination', t => {
  const f = createRepo(); try {
    symlinkSync(join(f.parent,'nonexistent'),f.destination, process.platform === 'win32' ? 'junction' : 'dir');
    assert.throws(() => bootstrapFork({source:f.upstream,destination:f.destination,expectedCommit:f.expectedCommit}), /Destination exists/);
  } finally { f.cleanup(); }
});

test('accepts a pinned local seed and creates standalone full-history checkout', () => {
  const f = createRepo(); try {
    cmd(['-C', f.upstream, 'remote', 'add', 'origin', 'https://github.com/sandboxie-plus/Sandboxie.git']);
    const local = verifyPinnedSeed(f.upstream, { expectedCommit: f.expectedCommit });
    const result = bootstrapFork({source:local,destination:f.destination,expectedCommit:f.expectedCommit});
    assert.equal(result.historyCount,3);
    assert.equal(result.shallow,false);
    assert.equal(cmd(['-C',f.destination,'remote','get-url','upstream']), 'https://github.com/sandboxie-plus/Sandboxie.git');
  } finally {f.cleanup();}
});
test('rejects an alternate source URL even if the commit and tag match', () => {
  const f=createRepo(); try {
    cmd(['-C', f.upstream,'remote','add','origin','https://example.invalid/lookalike.git']);
    assert.throws(()=>verifyPinnedSeed(f.upstream,{expectedCommit:f.expectedCommit}),/upstream URL/);
    assert.equal(existsSync(f.destination), false);
  } finally { f.cleanup(); }
});
test('rejects modified local seed despite matching tag', () => {
  const f=createRepo(); try {
    cmd(['-C',f.upstream,'remote','add','origin','https://github.com/sandboxie-plus/Sandboxie.git']);
    writeFileSync(join(f.upstream,'unexpected.txt'),'local artifact');
    assert.throws(()=>verifyPinnedSeed(f.upstream,{expectedCommit:f.expectedCommit}),/uncommitted or untracked/);
    assert.equal(existsSync(f.destination),false);
  } finally { f.cleanup(); }
});
