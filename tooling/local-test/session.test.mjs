import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, symlinkSync, writeFileSync, unlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { beginPreflight, finishPreflight } from './session.mjs';

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
