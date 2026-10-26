import { createHash } from 'node:crypto';
import {
  copyFileSync, createReadStream, lstatSync, mkdirSync, mkdtempSync,
  readFileSync, realpathSync, renameSync, rmSync, statSync, writeFileSync,
} from 'node:fs';
import { basename, dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyCheckout } from './verify-checkout.mjs';
import { verifyArtifactFiles } from './verify-artifact-files.mjs';

const UPSTREAM_COMMIT = 'abd4c240031cfb46666daa74ca47ed4e48b561e4';

function refuseUnless(value, message) {
  if (!value) throw new Error(`Owned engine assembly refused: ${message}`);
}
function entryExists(path) {
  try { lstatSync(path); return true; }
  catch (error) { if (error.code === 'ENOENT') return false; throw error; }
}
function isWithin(base, candidate) {
  const rel = relative(base, candidate);
  return rel === '' || (rel !== '..' && !rel.startsWith(`..${sep}`) && !isAbsolute(rel));
}
async function hash(path) {
  const digest = createHash('sha256');
  for await (const part of createReadStream(path)) digest.update(part);
  return digest.digest('hex');
}
function validatePathComponent(root, relativePath) {
  const target = resolve(root, ...relativePath.split('/'));
  refuseUnless(isWithin(root, target), `unsafe source path ${relativePath}`);
  let path = root;
  for (const part of relativePath.split('/')) {
    path = join(path, part);
    if (!entryExists(path)) break;
    refuseUnless(!lstatSync(path).isSymbolicLink(), `symlinked source path ${relativePath}`);
  }
  return target;
}
function targetPathForArtifact(row) {
  const win32Prefix = 'Sandboxie/Bin/Win32/SbieRelease/';
  const x64Prefix = 'Sandboxie/Bin/x64/SbieRelease/';
  const resourcePrefix = 'Sandboxie/install/';
  if (row.kind === 'required' && row.path.startsWith(x64Prefix)) {
    return basename(row.path);
  }
  if (row.kind === 'required' && row.path.startsWith(win32Prefix)) {
    return `32/${basename(row.path)}`;
  }
  if (row.kind === 'source-resource' && row.path.startsWith(resourcePrefix)) {
    return basename(row.path);
  }
  if (row.kind === 'optional-symbol' && row.path.startsWith(x64Prefix)) {
    return `_debug/x64/${basename(row.path)}`;
  }
  if (row.kind === 'optional-symbol' && row.path.startsWith(win32Prefix)) {
    return `_debug/Win32/${basename(row.path)}`;
  }
  throw new Error(`Owned engine assembly refused: unmapped artifact ${row.path}`);
}

/**
 * Assembles a PRIVATE, noninstalling, unmodified upstream-runtime payload.
 * Dependencies are injectable solely for disposable fixture tests; CLI always
 * invokes the actual checkout and artifact-byte verifiers.
 */
export async function assembleEngine(
  { sourceRoot, manifest, report, outputDirectory, privateBuild = null },
  { checkSource = verifyCheckout, checkFiles = verifyArtifactFiles } = {},
) {
  refuseUnless(typeof sourceRoot === 'string' && sourceRoot.trim(), 'missing source checkout');
  refuseUnless(typeof outputDirectory === 'string' && outputDirectory.trim(), 'missing output directory');
  refuseUnless(manifest?.upstream?.commit === UPSTREAM_COMMIT, 'unexpected upstream revision');
  if (privateBuild !== null) {
    refuseUnless(privateBuild?.kind === 'PRIVATE_EXCLUSIVE_SOURCE' &&
      typeof privateBuild.privateForkHead === 'string' &&
      /^[a-f0-9]{40}$/.test(privateBuild.privateForkHead) &&
      privateBuild.privateForkHead !== UPSTREAM_COMMIT &&
      /^[a-f0-9]{40}$/.test(privateBuild.privateForkTree) &&
      /^[a-f0-9]{64}$/.test(privateBuild.privatePatchSha256) &&
      privateBuild.canCoexistWithOfficialSandboxie === false &&
      privateBuild.sourceBuildMarker === 'MIVUBI_PRIVATE_ISOLATION_BUILD' &&
      Array.isArray(privateBuild.sourceFilesModified) &&
      privateBuild.sourceFilesModified.length === 11,
      'invalid or coinstallable private fork provenance');
  }
  const root = realpathSync(resolve(sourceRoot));
  refuseUnless(lstatSync(root).isDirectory(), 'source is not a directory');
  const target = resolve(outputDirectory);
  const parent = dirname(target);
  refuseUnless(entryExists(parent) && lstatSync(parent).isDirectory() &&
    !lstatSync(parent).isSymbolicLink() &&
    (process.platform === 'win32' ? realpathSync(parent).toLowerCase() === parent.toLowerCase() : realpathSync(parent) === parent), 'output parent must be an ordinary directory');
  refuseUnless(!isWithin(root, target) && !isWithin(target, root),
    'output and source directories cannot overlap');
  refuseUnless(!entryExists(target), 'output directory already exists');

  // Both verifiers run before any output exists. The artifact verifier checks
  // source bytes against the report; every copied file is hashed again below.
  checkSource(root);
  await checkFiles(manifest, report, root);
  refuseUnless(report.compiledInThisInvocation === true &&
    report.artifactResult === 'BUILD_ARTIFACTS_COLLECTED',
    'a successful native compilation and complete artifact evidence are required');
  const stage = mkdtempSync(join(parent, '.mivubi-engine-payload-'));
  const payload = join(stage, 'payload');
  mkdirSync(payload);
  try {
    const files = [];
    const used = new Set();
    const copyChecked = async (sourceRelative, outputRelative, expectedHash = null) => {
      refuseUnless(!used.has(outputRelative), `colliding payload path ${outputRelative}`);
      used.add(outputRelative);
      const original = validatePathComponent(root, sourceRelative);
      refuseUnless(entryExists(original) && lstatSync(original).isFile(),
        `source artifact not available: ${sourceRelative}`);
      const dest = resolve(payload, ...outputRelative.split('/'));
      refuseUnless(isWithin(payload, dest), `unsafe output path ${outputRelative}`);
      mkdirSync(dirname(dest), { recursive: true });
      copyFileSync(original, dest);
      const digest = await hash(dest);
      refuseUnless(expectedHash === null || digest === expectedHash,
        `copied artifact hash mismatch: ${sourceRelative}`);
      files.push({ path: outputRelative, source: sourceRelative,
        sizeBytes: statSync(dest).size, sha256: digest });
    };
    for (const row of report.artifacts) {
      if (row.state === 'MISSING') {
        refuseUnless(row.kind === 'optional-symbol',
          `missing required artifact ${row.path}`);
        continue;
      }
      await copyChecked(row.path, targetPathForArtifact(row), row.sha256);
    }
    // Attribution is retained for all selected upstream components and tools.
    const licensePaths = manifest.upstream.sourceLicenseFiles;
    refuseUnless(Array.isArray(licensePaths) && licensePaths.includes('Sandboxie/COPYING') &&
      licensePaths.includes('LICENSE.Plus'), 'required upstream license notices missing');
    for (const license of licensePaths) {
      refuseUnless(/^[A-Za-z0-9_.\/-]+$/.test(license) &&
        !license.split('/').includes('..'), `unsafe license path ${license}`);
      await copyChecked(license, `LICENSES/${license}`);
    }
    const provenance = {
      schema: 1, state: privateBuild === null
        ? 'ARTIFACTS_STAGED_NOT_INSTALLABLE' : 'PRIVATE_CORE_STAGED_NOT_INSTALLABLE',
      ...(privateBuild === null ? {} : {
        privateCoreBuild: true,
        privateForkHead: privateBuild.privateForkHead,
        privateForkTree: privateBuild.privateForkTree,
        privatePatchSha256: privateBuild.privatePatchSha256,
        sourceBuildMarker: privateBuild.sourceBuildMarker,
        sourceFilesModified: privateBuild.sourceFilesModified,
        canCoexistWithOfficialSandboxie: false,
      }),
      upstreamRepository: manifest.upstream.repository,
      upstreamTag: manifest.upstream.tag,
      upstreamCommit: UPSTREAM_COMMIT,
      buildStages: report.stageResults.map(s => ({
        id: s.id, platform: s.platform, state: s.state,
      })),
      artifacts: files,
      windowsKernelDriverLoaded: false, minecraftGameplayVerified: false,
      sandboxieExternalDependencyRemoved: false, publicDistributionApproved: false,
    };
    writeFileSync(join(payload, 'engine-provenance.json'),
      JSON.stringify(provenance, null, 2) + '\n', { encoding: 'utf8', flag: 'wx' });
    writeFileSync(join(payload, 'NOT-INSTALLABLE.txt'),
      (privateBuild === null
        ? 'Unmodified upstream candidate, not a signed or tested engine.'
        : 'Private exclusive fork candidate, NOT installable or co-installable with official Sandboxie.') +
      ' Do not install SbieDrv.sys or replace Client 02. See LICENSES and engine-provenance.json.\n',
      { encoding: 'utf8', flag: 'wx' });
    // Never replace a populated destination, including dangling symlinks.
    refuseUnless(!entryExists(target), 'output appeared during staging');
    renameSync(payload, target);
    return { state: provenance.state, destination: target,
      artifactCount: files.length, upstreamCommit: UPSTREAM_COMMIT,
      driverInstalled: false, gameplayVerified: false };
  } finally {
    // Only our own private staging directory may be deleted.
    rmSync(stage, { recursive: true, force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    refuseUnless(args.length === 9 && args[0] === '--assemble' &&
      args[1] === '--source-root' && args[3] === '--manifest' &&
      args[5] === '--report' && args[7] === '--output',
      'usage: node assemble-engine.mjs --assemble --source-root <checkout> --manifest <manifest.json> --report <report.json> --output <new-directory>');
    const read = p => JSON.parse(readFileSync(resolve(p), 'utf8').replace(/^\uFEFF/, ''));
    const result = await assembleEngine({ sourceRoot: args[2], manifest: read(args[4]),
      report: read(args[6]), outputDirectory: args[8] });
    process.stdout.write(JSON.stringify(result, null, 2) + '\n');
  } catch (error) {
    process.stderr.write(error.message + '\n');
    process.exitCode = 1;
  }
}
