import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { checkTracked, checkLinks } from './check-structure.mjs';

test('reject outputs and competing root owners while preserving native provenance', () => {
  assert.deepEqual(checkTracked(['README.md', 'workspace/README.md',
    'engine/isolation-owned/tools/dist/tool.exe', 'engine/isolation-candidates/target/reference.log']), []);
  for (const path of ['apps/desktop/.svelte-check/tsconfig.json', 'scratch.mjs', 'workspace/report.json', 'apps/desktop/dist/index.html',
    'engine/runtime-core/target/debug/core.exe', '.github/workflows/verify.yml',
    'apps/desktop/src-tauri/package-resources/lazy-tester/bootstrap-compat.dll',
    'apps/desktop/src-tauri/icons/icon.ico', 'apps/desktop/src-tauri/gen/schema.json',
    'apps/desktop/src-tauri/binaries/old.exe', 'release/setup.exe', 'develop/copy.ts']) {
    assert.ok(checkTracked([path]).length > 0, path);
  }
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
