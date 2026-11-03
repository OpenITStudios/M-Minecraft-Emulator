import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync, unlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { beginPreflight, finishPreflight, beginQuick, finishQuick } from './session.mjs';

const commit = 'a'.repeat(40);
function fixture(t) {
  const root = mkdtempSync(join(tmpdir(), 'emulator-localtest-'));
  mkdirSync(join(root, 'workspace'));
  t.after(() => rmSync(root, { recursive: true, force: true }));
  return root;
}
const manifest = (folder) => JSON.parse(readFileSync(join(folder, 'manifest.json'), 'utf8'));

test('records exact source revision without pretending to prove Minecraft', (t) => {
  const root = fixture(t);
  const now = new Date('2026-10-13T05:00:00.000Z');
  const folder = beginPreflight(root, commit, now);
  const current = manifest(folder);
  assert.equal(current.source.commit, commit);
  assert.equal(current.status, 'RUNNING');
  assert.equal(current.windowsGameplay, 'NOT_EXECUTED');
  assert.equal(current.evidence.length, 0);
  const end = finishPreflight(root, folder, 'PASS', 'complete', new Date('2026-10-13T05:01:00Z'));
  assert.equal(end.status, 'PASS');
  assert.equal(end.sourceVerification, 'PASS');
  assert.equal(end.packageVerification, 'NOT_EXECUTED');
  assert.equal(end.windowsGameplay, 'NOT_EXECUTED');
  assert.throws(() => finishPreflight(root, folder, 'FAIL', 'verify-source'), /already been finalized/);
});

test('records failed source stage and preserves earlier evidence attempts', (t) => {
  const root = fixture(t);
  const first = beginPreflight(root, commit);
  const second = beginPreflight(root, commit);
  assert.notEqual(first, second);
  finishPreflight(root, first, 'FAIL', 'setup');
  assert.equal(manifest(first).sourceVerification, 'INCOMPLETE_OR_FAILED');
  assert.equal(manifest(second).status, 'RUNNING');
});

test('rejects forged revisions, result claims, wrong folders, and time travel', (t) => {
  const root = fixture(t);
  assert.throws(() => beginPreflight(root, 'abc'), /complete, lowercase/);
  const folder = beginPreflight(root, commit, new Date('2026-10-13T05:00:00.000Z'));
  assert.throws(() => finishPreflight(root, folder, 'PASS', 'verify-source'), /Invalid preflight/);
  assert.throws(() => finishPreflight(root, folder, 'FAIL', 'complete'), /Invalid preflight/);
  assert.throws(() => finishPreflight(root, join(root, 'workspace'), 'PASS', 'complete'), /immediate preflight/);
  assert.throws(() => finishPreflight(root, folder, 'PASS', 'complete', new Date('2026-10-12')), /predates/);
});

test('refuses symlinked storage instead of escaping to unrelated directories', (t) => {
  const root = fixture(t);
  const external = mkdtempSync(join(tmpdir(), 'emulator-external-'));
  t.after(() => rmSync(external, { recursive: true, force: true }));
  symlinkSync(external, join(root, 'workspace', 'runs'), process.platform === 'win32' ? 'junction' : 'dir');
  assert.throws(() => beginPreflight(root, commit), /real directories/);
});

test('rejects a modified commit identity rather than granting source PASS', (t) => {
  const root = fixture(t);
  const folder = beginPreflight(root, commit);
  const changed = manifest(folder);
  changed.source.commit = 'b'.repeat(40);
  writeFileSync(join(folder, 'manifest.json'), JSON.stringify(changed));
  assert.throws(() => finishPreflight(root, folder, 'PASS', 'complete'), /manifest identity/);
  assert.equal(manifest(folder).status, 'RUNNING');
});

test('exclusive finalization prevents a competing process from recording PASS', (t) => {
  const root = fixture(t);
  const folder = beginPreflight(root, commit);
  writeFileSync(join(folder, '.finalize.lock'), 'another finalizer is active', { flag: 'wx' });
  assert.throws(() => finishPreflight(root, folder, 'PASS', 'complete'), /EEXIST/);
  assert.equal(manifest(folder).status, 'RUNNING');
  unlinkSync(join(folder, '.finalize.lock'));
  finishPreflight(root, folder, 'PASS', 'complete');
  assert.equal(manifest(folder).status, 'PASS');
});

test('quick source receipt measures elapsed time without upgrading full source or Minecraft acceptance', (t) => {
  const root = fixture(t);
  const start = new Date('2026-11-03T13:10:00Z');
  const run = beginQuick(root, commit, start);
  const initial = manifest(run);
  assert.equal(initial.scenario,'SOURCE_QUICK');
  assert.equal(initial.quickSourceScreen,'NOT_EXECUTED');
  assert.equal(initial.sourceVerification,'NOT_EXECUTED');
  assert.equal(initial.windowsGameplay,'NOT_EXECUTED');
  const result = finishQuick(root,run,'PASS','complete',new Date('2026-11-03T13:10:06.500Z'));
  assert.equal(result.quickSourceScreen,'PASS');
  assert.equal(result.durationMilliseconds,6500);
  assert.equal(result.sourceVerification,'NOT_EXECUTED');
  assert.equal(result.packageVerification,'NOT_EXECUTED');
  assert.equal(result.windowsGameplay,'NOT_EXECUTED');
  assert.throws(()=>finishQuick(root,run,'PASS','complete'),/already been finalized/);
  assert.throws(()=>finishPreflight(root,run,'PASS','complete'),/preflight run directory/);
});

test('failed quick screen is not a source PASS and rejects fake stage or reversed time', (t) => {
  const root=fixture(t);
  const start=new Date('2026-11-03T13:10:00Z');
  const run=beginQuick(root,commit,start);
  assert.throws(()=>finishQuick(root,run,'PASS','verify-quick'),/Invalid quick/);
  assert.throws(()=>finishQuick(root,run,'FAIL','setup'),/Invalid quick/);
  assert.throws(()=>finishQuick(root,run,'PASS','complete',new Date('2026-11-03T13:09:59Z')), /predates/);
  assert.throws(()=>finishPreflight(root,run,'FAIL','setup'),/preflight run directory/);
  const result=finishQuick(root,run,'FAIL','verify-quick',new Date('2026-11-03T13:10:05Z'));
  assert.equal(result.quickSourceScreen,'INCOMPLETE_OR_FAILED');
  assert.equal(result.sourceVerification,'NOT_EXECUTED');
  assert.equal(result.lastStage,'verify-quick');
});

test('quick receipt refuses modified source identity and symlinked run storage', (t) => {
  const root=fixture(t);
  const run=beginQuick(root,commit);
  const record=manifest(run);
  record.source.commit='b'.repeat(40);
  writeFileSync(join(run,'manifest.json'),JSON.stringify(record));
  assert.throws(()=>finishQuick(root,run,'PASS','complete'),/manifest identity/);
});
