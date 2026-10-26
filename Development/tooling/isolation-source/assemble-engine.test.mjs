import test from 'node:test';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { existsSync, mkdtempSync, mkdirSync, readFileSync, writeFileSync, rmSync, symlinkSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { tmpdir } from 'node:os';
import { assembleEngine } from './assemble-engine.mjs';

function makeFixture() {
  const base = mkdtempSync(join(tmpdir(), 'mivubi-engine-payload-'));
  const root = join(base, 'sandboxie-source');
  mkdirSync(root);
  const output = join(base, 'owned-payload');
  const items = [
    ['Sandboxie/Bin/x64/SbieRelease/SbieDrv.sys', 'required'],
    ['Sandboxie/Bin/x64/SbieRelease/SbieSvc.exe', 'required'],
    ['Sandboxie/Bin/x64/SbieRelease/SbieDll.dll', 'required'],
    ['Sandboxie/Bin/x64/SbieRelease/Start.exe', 'required'],
    ['Sandboxie/Bin/Win32/SbieRelease/SbieSvc.exe', 'required'],
    ['Sandboxie/Bin/Win32/SbieRelease/SbieDll.dll', 'required'],
    ['Sandboxie/install/Templates.ini', 'source-resource'],
    ['Sandboxie/Bin/x64/SbieRelease/SbieDll.pdb', 'optional-symbol'],
  ];
  const missingPdb = items[7][0];
  const artifacts = items.map(([path, kind]) => {
    if (path === missingPdb) return {
      path, kind, state: 'MISSING', sha256: null, sizeBytes: null, authenticode: 'NOT_EXECUTED',
    };
    const bytes = Buffer.from(`test-artifact:${path}`);
    const filename = join(root, ...path.split('/'));
    mkdirSync(dirname(filename), { recursive: true });
    writeFileSync(filename, bytes);
    return { path, kind, state: 'PRESENT', sizeBytes: bytes.length,
      sha256: createHash('sha256').update(bytes).digest('hex'),
      authenticode: 'NotSigned' };
  });
  for (const path of ['LICENSE.Plus', 'Sandboxie/COPYING']) {
    const name = join(root, ...path.split('/'));
    mkdirSync(dirname(name), { recursive: true });
    writeFileSync(name, `Test attribution ${path}\n`);
  }
  const manifest = {
    upstream: { repository: 'sandboxie-plus/Sandboxie', tag: 'v1.18.5',
      commit: 'abd4c240031cfb46666daa74ca47ed4e48b561e4',
      sourceLicenseFiles: ['LICENSE.Plus', 'Sandboxie/COPYING'] },
  };
  const report = {
    compiledInThisInvocation: true, artifactResult: 'BUILD_ARTIFACTS_COLLECTED',
    stageResults: [{ id: 'x64-core', platform: 'x64', state: 'COMPILED' }], artifacts,
  };
  const checks = {
    checkSource: () => ({ status: 'PINNED_CHECKOUT_VERIFIED' }),
    checkFiles: async () => ({ verification: 'FILE_CONTENTS_MATCH_REPORTED_HASHES' }),
  };
  return { base, root, output, manifest, report, checks,
    clean: () => rmSync(base, { recursive: true, force: true }) };
}
const run = f => assembleEngine({ sourceRoot: f.root, outputDirectory: f.output,
  manifest: f.manifest, report: f.report }, f.checks);

test('stages required x64, 32-bit companions, resources, notices, provenance without install', async () => {
  const f = makeFixture();
  try {
    const result = await run(f);
    assert.equal(result.state, 'ARTIFACTS_STAGED_NOT_INSTALLABLE');
    assert.equal(result.driverInstalled, false);
    assert.ok(existsSync(join(f.output, 'SbieDrv.sys')));
    assert.ok(existsSync(join(f.output, 'SbieSvc.exe')));
    assert.ok(existsSync(join(f.output, '32', 'SbieDll.dll')));
    assert.ok(existsSync(join(f.output, 'Templates.ini')));
    assert.ok(existsSync(join(f.output, 'LICENSES', 'Sandboxie', 'COPYING')));
    assert.ok(existsSync(join(f.output, 'LICENSES', 'LICENSE.Plus')));
    assert.ok(!existsSync(join(f.output, '_debug', 'x64', 'SbieDll.pdb')));
    const provenance = JSON.parse(readFileSync(join(f.output, 'engine-provenance.json'), 'utf8'));
    assert.equal(provenance.upstreamCommit, f.manifest.upstream.commit);
    assert.equal(provenance.sandboxieExternalDependencyRemoved, false);
    assert.equal(provenance.minecraftGameplayVerified, false);
    assert.equal(provenance.artifacts.length, 9);
  } finally { f.clean(); }
});

test('refuses incomplete build and produces no output', async () => {
  const f = makeFixture();
  try {
    f.report.compiledInThisInvocation = false;
    await assert.rejects(run(f), /a successful native compilation/);
    assert.equal(existsSync(f.output), false);
  } finally { f.clean(); }
});

test('refuses a payload when copied file does not match report hash', async () => {
  const f = makeFixture();
  try {
    f.report.artifacts[0].sha256 = 'a'.repeat(64);
    await assert.rejects(run(f), /copied artifact hash mismatch/);
    assert.equal(existsSync(f.output), false);
  } finally { f.clean(); }
});

test('refuses pre-existing output and preserves its content', async () => {
  const f = makeFixture();
  try {
    mkdirSync(f.output);
    writeFileSync(join(f.output, 'keep.txt'), 'do not touch');
    await assert.rejects(run(f), /already exists/);
    assert.equal(readFileSync(join(f.output, 'keep.txt'), 'utf8'), 'do not touch');
  } finally { f.clean(); }
});

test('refuses missing GPL license text and cleans temporary staging', async () => {
  const f = makeFixture();
  try {
    rmSync(join(f.root, 'Sandboxie', 'COPYING'));
    await assert.rejects(run(f), /source artifact not available/);
    assert.equal(existsSync(f.output), false);
  } finally { f.clean(); }
});

test('rejects symlinked required executable despite mocked preflight', async t => {
  const f = makeFixture();
  try {
    const original = join(f.root, 'Sandboxie/Bin/x64/SbieRelease/SbieDrv.sys');
    rmSync(original);
    try { symlinkSync(join(f.base, 'not-found.sys'), original); }
    catch (e) { if (process.platform === 'win32' && e.code === 'EPERM') { t.skip('symlink restricted'); return; } throw e; }
    await assert.rejects(run(f), /symlinked source path/);
    assert.equal(existsSync(f.output), false);
  } finally { f.clean(); }
});

test('rejects source and output that overlap', async () => {
  const f = makeFixture();
  try {
    await assert.rejects(assembleEngine({ sourceRoot: f.root, outputDirectory: join(f.root, 'payload'),
      manifest: f.manifest, report: f.report }, f.checks), /cannot overlap/);
    assert.equal(existsSync(join(f.root, 'payload')), false);
  } finally { f.clean(); }
});
