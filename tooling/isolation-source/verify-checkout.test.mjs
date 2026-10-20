import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, rmSync, writeFileSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { spawnSync } from 'node:child_process';
import { verifyCheckout } from './verify-checkout.mjs';

function git(...args) {
  const r = spawnSync('git', args, { encoding: 'utf8' });
  assert.equal(r.status, 0, r.stderr);
  return r.stdout.trim();
}
function fixture() {
  const dir = mkdtempSync(join(tmpdir(), 'mivubi-pinned-checkout-'));
  git('init', '-b', 'main', dir);
  git('-C', dir, 'config', 'user.email', 'test@example.invalid');
  git('-C', dir, 'config', 'user.name', 'Isolation Tester');
  for (let i = 0; i < 3; i++) git('-C', dir, 'commit', '--allow-empty', '-m', `history ${i}`);
  const pin = git('-C', dir, 'rev-parse', 'HEAD');
  git('-C', dir, 'tag', 'v1.18.5');
  git('-C', dir, 'switch', '-c', 'isolation/baseline-v1.18.5');
  git('-C', dir, 'remote', 'add', 'upstream', 'https://github.com/sandboxie-plus/Sandboxie.git');
  return { dir, pin, cleanup: () => rmSync(dir, { recursive: true, force: true }) };
}
const checked = f => verifyCheckout(f.dir, { pin: f.pin });
test('accepts pinned full-history clean working branch without building', () => {
  const f = fixture(); try {
    const result = checked(f);
    assert.equal(result.status, 'PINNED_CHECKOUT_VERIFIED');
    assert.equal(result.historyCount, 3);
    assert.equal(result.driverInstalled, false);
  } finally { f.cleanup(); }
});
test('rejects changed or deleted upstream remote', () => {
  const f = fixture(); try {
    git('-C', f.dir, 'remote', 'set-url', 'upstream', 'https://example.invalid/other.git');
    assert.throws(() => checked(f), /upstream source URL changed/);
  } finally { f.cleanup(); }
});
test('rejects untracked changes even when HEAD is still pinned', () => {
  const f = fixture(); try {
    writeFileSync(join(f.dir, 'unexpected.txt'), 'artifact');
    assert.throws(() => checked(f), /tracked or untracked changes/);
  } finally { f.cleanup(); }
});
test('rejects detached or wrong branch without modifying source', () => {
  const f = fixture(); try {
    git('-C', f.dir, 'switch', '--detach');
    assert.throws(() => checked(f), /unexpected working branch/);
  } finally { f.cleanup(); }
});
test('rejects mismatched source pin/tag', () => {
  const f = fixture(); try {
    assert.throws(() => verifyCheckout(f.dir, { pin: 'f'.repeat(40) }), /unpinned HEAD/);
  } finally { f.cleanup(); }
});
test('rejects source path symlink', t => {
  const f = fixture(); const wrapper = join(tmpdir(), 'isolation-checkout-link-' + process.pid);
  try {
    symlinkSync(f.dir, wrapper, process.platform === 'win32' ? 'junction' : 'dir');
    assert.throws(() => verifyCheckout(wrapper, { pin: f.pin }), /real directory/);
  } finally { rmSync(wrapper, { force: true }); f.cleanup(); }
});
