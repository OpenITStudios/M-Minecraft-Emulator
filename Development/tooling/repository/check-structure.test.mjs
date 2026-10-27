import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkTracked, checkLinks, checkPhysical } from './check-structure.mjs';

test('reject outputs and competing root owners while preserving native provenance', () => {
  assert.deepEqual(checkTracked(['README.md', 'workspace/README.md']), []);
  for (const path of ['Development/apps/desktop/.svelte-check/tsconfig.json', 'scratch.mjs', 'workspace/report.json', 'Development/apps/desktop/dist/index.html',
    'Development/engine/runtime-core/target/debug/core.exe', '.github/workflows/verify.yml',
    '.gitmodules', 'Development/engine/isolation-owned/tools/dist/tool.exe',
    'Development/engine/isolation-candidates/target/reference.log',
    'Development/engine/isolation-upstream/Sandboxie',
    'Development/apps/desktop/src-tauri/package-resources/lazy-tester/bootstrap-compat.dll',
    'Development/apps/desktop/src-tauri/icons/icon.ico', 'Development/apps/desktop/src-tauri/gen/schema.json',
    'Development/apps/desktop/src-tauri/binaries/old.exe', 'release/setup.exe', 'develop/copy.ts',
    'Data/profiles.json', 'App/data-location.json', 'Logs/session.log',
    'Development/copy/source.ts', 'Archive/apps/source.ts', 'apps/desktop/src/main.ts']) {
    assert.ok(checkTracked([path]).length > 0, path);
  }
});
test('physical layout rejects leftover source roots and root build metadata', () => {
  const root = mkdtempSync(join(tmpdir(), 'emulator-layout-'));
  try {
    for (const name of ['Development', 'Archive', 'App', 'Data', 'workspace']) mkdirSync(join(root, name));
    writeFileSync(join(root, '.git'), 'gitdir: linked-worktree');
    assert.deepEqual(checkPhysical(root), []);
    writeFileSync(join(root,'Lazy Tester.exe'),'MZfixture');
    assert.deepEqual(checkPhysical(root), []);
    assert.ok(checkTracked(['Lazy Tester.exe']).length);
    mkdirSync(join(root, 'apps'));
    writeFileSync(join(root, 'package.json'), '{}');
    assert.deepEqual(checkPhysical(root).sort(), [
      'Unreviewed physical root entry: apps', 'Unreviewed physical root entry: package.json',
    ]);
  } finally { rmSync(root, {recursive:true, force:true}); }
});
test('check relative document links, including fragments, without fetching external URLs', () => {
  const root = mkdtempSync(join(tmpdir(), 'emulator-links-'));
  try {
    mkdirSync(join(root, 'docs'));
    writeFileSync(join(root, 'docs', 'guide.md'), '[ok](../README.md#start) [web](https://example.com) [bad](missing.md)');
    writeFileSync(join(root, 'README.md'), '# start');
    assert.deepEqual(checkLinks(root, ['docs/guide.md']), ['Invalid internal link: docs/guide.md -> missing.md']);
  } finally { rmSync(root, {recursive: true, force: true}); }
});
