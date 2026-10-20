// MIVUBI v1.18.6: one build report -> one immutable, NOT-INSTALLABLE engine payload.
// This is byte packaging, NOT Windows service registration, signing, or activation.
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import {
  copyFileSync, createReadStream, lstatSync, mkdirSync, mkdtempSync,
  readFileSync, realpathSync, renameSync, rmSync, statSync, writeFileSync,
} from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const PROFILE = 'engine/isolation-native/owned-runtime/build-profile-v1186.json';
const STATE = 'NATIVE_BUILD_ARTIFACTS_CAPTURED_NOT_INSTALLABLE';
const PAYLOAD_STATE = 'MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE';
const MARKER = 'MIVUBI-owned v1.18.6 Sandboxie-derived core. NOT installable or co-installable with official Sandboxie. Do not install SbieDrv.sys or replace MinecraftClient02. See LICENSES and engine-provenance.json.\n';
const sha40 = /^[a-f0-9]{40}$/;
const sha64 = /^[a-f0-9]{64}$/;
const demand = (condition, message) => {
  if (!condition) throw Error('Owned v1.18.6 assembly refused: ' + message);
};
const exists = (path) => {
  try { lstatSync(path); return true; }
  catch (e) { if (e.code === 'ENOENT') return false; throw e; }
};
const inside = (root, path) => {
  const remainder = relative(root, path);
  return remainder === '' || (remainder !== '..' &&
    !remainder.startsWith('..' + sep) && !isAbsolute(remainder));
};
function git(root, ...args) {
  const result = spawnSync('git', ['-C', root, ...args], {
    encoding: 'utf8', timeout: 120000, maxBuffer: 1024 * 1024, windowsHide: true,
  });
  demand(!result.error && result.status === 0,
    'git proof unavailable: ' + (result.error?.message || result.stderr || args[0]).slice(0,240));
  return result.stdout.trim();
}
function ordinary(root, relativePath) {
  demand(typeof relativePath === 'string' &&
    /^[A-Za-z0-9._/-]+$/.test(relativePath) &&
    !relativePath.startsWith('/') && !relativePath.split('/').some(x => !x || x === '.' || x === '..'),
    'unsafe source or payload path');
  const path = resolve(root, ...relativePath.split('/'));
  demand(inside(root, path), 'path escapes source/payload root');
  let node = root;
  for (const segment of relativePath.split('/')) {
    node = join(node, segment);
    const stat = lstatSync(node);
    demand(!stat.isSymbolicLink(), 'symlinked native artifact or parent');
  }
  demand(lstatSync(path).isFile(), 'required source path is not a regular file');
  return path;
}
async function checksum(path) {
  const digest = createHash('sha256');
  for await (const data of createReadStream(path)) digest.update(data);
  return digest.digest('hex');
}
// The nine shipped configuration/license files must match Git HEAD, not just
// a working-tree status snapshot taken before copying. Git SHA-1 blob IDs
// include the content length prefix; file SHA-1 alone is NOT equivalent.
async function gitBlobId(path) {
  const size = statSync(path).size;
  const digest = createHash('sha1').update('blob ' + size + '\0');
  for await (const data of createReadStream(path)) digest.update(data);
  return digest.digest('hex');
}
function committedResources(root, paths, readGit) {
  const prefix = 'engine/isolation-owned/';
  const expected = new Set(paths);
  const rows = readGit(root, 'ls-tree', '-r', '--full-tree', 'HEAD', '--',
    ...paths.map(path => prefix + path));
  const blobs = new Map();
  for (const line of rows.split('\n')) {
    if (!line) continue;
    const match = /^(100644|100755) blob ([a-f0-9]{40})\t(.+)$/.exec(line);
    demand(match && match[3].startsWith(prefix), 'unrecognized tracked resource entry');
    const path = match[3].slice(prefix.length);
    demand(expected.has(path) && !blobs.has(path), 'unexpected or duplicate tracked resource');
    blobs.set(path, match[2]);
  }
  demand(blobs.size === expected.size, 'required resources must exist in committed Git HEAD');
  return blobs;
}
function destinationFor(path) {
  if (path.startsWith('Sandboxie/Bin/Win32/SbieRelease/'))
    return '32/' + path.split('/').at(-1);
  if (path.startsWith('Sandboxie/Bin/x64/SbieRelease/'))
    return path.split('/').at(-1);
  if (path.startsWith('Sandboxie/install/'))
    return path.split('/').at(-1);
  if (path === 'LICENSE.Plus' || path === 'LICENSE.Classic' ||
      path === 'Sandboxie/COPYING' || path === 'SandboxiePlus/QSbieAPI/LICENSE')
    return 'LICENSES/' + path;
  throw Error('Owned v1.18.6 assembly refused: unrecognized source ' + path);
}
function requireProfile(profile) {
  demand(profile?.sourceVersion === 'v1.18.6' &&
    profile.sourceCommit === 'cdba7096240b93d79c5373aa81a75a1c5c71b883' &&
    profile.pristineSourceTree === 'f690477c07a40810f479f53973334ceae7e0ab13' &&
    profile.canonicalEditableSource === 'engine/isolation-owned' &&
    profile.privateDriverIdentity === '5.73.6-MIVU' &&
    profile.driverAbi === '0x57230' &&
    profile.approvedNativeModifiedFiles === 11 &&
    profile.enginePayloadState === PAYLOAD_STATE &&
    profile.flags?.canCoexistWithOfficialSandboxie === false &&
    profile.flags?.driverBuilt === false &&
    profile.buildStages?.length === 3 &&
    profile.requiredArtifacts?.length === 16 &&
    profile.sourceResources?.length === 5,
    'incorrect or incomplete owned source profile');
  const expectedStages = [
    ['win32-support', 'Sandboxie/SandboxDll.sln', 'Win32'],
    ['x64-user-core', 'Sandboxie/Sandbox.sln', 'x64'],
    ['x64-wdk-kernel', 'Sandboxie/SandboxDrv.sln', 'x64'],
  ];
  demand(profile.buildStages.every((row,i) =>
    JSON.stringify([row.id,row.sln,row.platform]) === JSON.stringify(expectedStages[i])),
    'noncanonical native build stage sequence');
  demand(new Set(profile.requiredArtifacts).size === 16 &&
    profile.requiredArtifacts.every(p =>
      /^Sandboxie\/Bin\/(Win32|x64)\/SbieRelease\/[A-Za-z0-9._-]+$/.test(p)),
    'invalid binary inventory');
  demand(JSON.stringify(profile.sourceResources) === JSON.stringify([
    'Sandboxie/install/Templates.ini',
    'Sandboxie/install/SbieSettings.ini',
    'Sandboxie/install/Manifest0.txt',
    'Sandboxie/install/Manifest1.txt',
    'Sandboxie/install/Manifest2.txt',
  ]), 'unreviewed resource inventory');
  demand(JSON.stringify(profile.originalLicensesMustBeRetained) === JSON.stringify([
    'LICENSE.Classic', 'LICENSE.Plus', 'Sandboxie/COPYING',
    'SandboxiePlus/QSbieAPI/LICENSE',
  ]), 'incomplete original license inventory');
}
function requireReport(report, profile, head, ownedTree) {
  demand(report?.schema === 1 && report.state === STATE &&
    report.sourceVersion === 'v1.18.6' &&
    report.repoHead === head && sha40.test(head) &&
    report.ownedTree === ownedTree && sha40.test(ownedTree) &&
    report.upstreamTree === profile.pristineSourceTree &&
    report.nativeModifiedFiles === 11 &&
    report.compiler && sha64.test(report.compiler.sha256) &&
    typeof report.compiler.fileVersion === 'string' &&
    !report.failureReason &&
    report.driverBuilt === true && report.driverSigned === false &&
    report.driverLoaded === false && report.installPermitted === false &&
    report.activeProviderChanged === false && report.minecraftGameplayVerified === false,
    'build report is incomplete or claims unsafe runtime activation');
  demand(report.buildStages?.length === 3 &&
    report.buildStages.every((s,i) =>
      s.stage === profile.buildStages[i].id &&
      s.platform === profile.buildStages[i].platform &&
      s.exitCode === 0 && typeof s.stdoutLog === 'string' &&
      typeof s.stderrLog === 'string' && !s.launchFailure),
    'native Win32/x64/WDK build stages not completed');
  demand(report.artifacts?.length === 16 &&
    report.artifacts.every((item,i) =>
      item.path === profile.requiredArtifacts[i] &&
      Number.isSafeInteger(item.bytes) && item.bytes > 0 &&
      sha64.test(item.sha256)),
    'native artifact paths, lengths or hashes invalid');
}
export async function stageOwnedV1186({
  repository = REPO, buildReport, outputDirectory,
}, { verifySource = null, readGit = git } = {}) {
  demand(typeof buildReport === 'string' && buildReport.length > 0 &&
    typeof outputDirectory === 'string' && outputDirectory.length > 0,
    'provide report path and new payload output directory');
  const root = realpathSync(resolve(repository));
  demand(lstatSync(root).isDirectory(), 'repository root invalid');
  const profile = JSON.parse(readFileSync(ordinary(root,PROFILE),'utf8'));
  requireProfile(profile);
  const sourceRoot = ordinarySource(root, 'engine/isolation-owned');
  const target = resolve(outputDirectory);
  const parent = dirname(target);
  demand(exists(parent) && lstatSync(parent).isDirectory() &&
    !lstatSync(parent).isSymbolicLink() && realpathSync(parent) === parent,
    'payload destination parent must be a real directory');
  const packagesRoot = join(root, 'workspace', 'packages');
  demand(inside(packagesRoot, target) && target !== packagesRoot &&
    !inside(sourceRoot, target) && !inside(target, sourceRoot),
    'payload must be a new child of workspace/packages');
  demand(!exists(target), 'destination already exists');
  demand(readGit(root, 'branch', '--show-current') === 'Local',
    'only reviewed Local branch is supported');
  const head = readGit(root,'rev-parse','HEAD');
  const ownedTree = readGit(root,'rev-parse','HEAD:engine/isolation-owned');
  demand(readGit(root,'rev-parse','HEAD:engine/isolation-candidates/sandboxie-v1.18.6') ===
    profile.pristineSourceTree, 'pinned upstream candidate changed');
  // The source verifier attests committed Git blobs, but this assembler
  // copies resources, licenses and compiled outputs from the working tree.
  // Never bind uncommitted edits to a verified HEAD. Allow ignored native
  // binaries from the completed build; those have separate report hashes.
  demand(readGit(root,'status','--porcelain=v1','--untracked-files=no','--',
    'engine/isolation-owned', PROFILE) === '',
    'tracked engine source/profile differs from verified Git HEAD');
  if (verifySource) {
    const result = verifySource(root);
    demand(result?.head === head && result?.ownedTree === ownedTree &&
      result?.nativeModifiedFiles === 11, 'modified source proof differs');
  } else {
    const output = spawnSync(process.execPath,
      [join(root,'tooling/isolation-source/verify-owned-v1186.mjs')],
      {cwd:root,encoding:'utf8',timeout:120000,maxBuffer:1024*1024});
    demand(!output.error && output.status === 0,
      'exact owned native source verification failed');
    const proof = JSON.parse(output.stdout);
    demand(proof.head === head && proof.ownedTree === ownedTree &&
      proof.nativeModifiedFiles === 11, 'source verifier mismatch');
  }
  // Read the actual report bytes and bind their SHA to package provenance.
  const reportPath = resolve(buildReport);
  const buildsRoot = join(root, 'workspace', 'builds');
  demand(inside(buildsRoot, reportPath) && reportPath !== buildsRoot &&
    realpathSync(dirname(reportPath)) === dirname(reportPath),
    'build report must remain within real workspace/builds');
  demand(exists(reportPath) && !lstatSync(reportPath).isSymbolicLink() &&
    lstatSync(reportPath).isFile() && statSync(reportPath).size <= 256*1024,
    'report must be an ordinary bounded file');
  const reportBytes = readFileSync(reportPath);
  demand(reportBytes.length <= 256*1024, 'report grew while being read');
  const report = JSON.parse(reportBytes.toString('utf8').replace(/^\uFEFF/,''));
  requireReport(report,profile,head,ownedTree);
  const reportHash = createHash('sha256').update(reportBytes).digest('hex');
  // Verify ALL inputs before touching the selected output directory.
  const resources = [...profile.sourceResources, ...profile.originalLicensesMustBeRetained];
  const input = [...profile.requiredArtifacts, ...resources];
  demand(new Set(input.map(destinationFor)).size === input.length,
    'payload destination collision');
  // One Git tree lookup for all nine tracked resources; no extra per-file
  // subprocesses or second source registry. The final copied bytes are
  // checked against these pinned IDs after the copy completes.
  const resourceBlobs = committedResources(root, resources, readGit);
  for (const rel of input) {
    const file = ordinary(sourceRoot, rel);
    if (profile.requiredArtifacts.includes(rel)) {
      const row = report.artifacts[profile.requiredArtifacts.indexOf(rel)];
      demand(statSync(file).size === row.bytes &&
        await checksum(file) === row.sha256,
        'native output bytes differ from actual build report: ' + rel);
    }
  }
  const temporary = mkdtempSync(join(parent,'.mivubi-owned-v1186-'));
  const payload = join(temporary,'payload');
  mkdirSync(payload);
  try {
    const artifacts = [];
    for (const rel of input) {
      const original = ordinary(sourceRoot,rel);
      const dest = join(payload,...destinationFor(rel).split('/'));
      mkdirSync(dirname(dest),{recursive:true});
      if (resourceBlobs.has(rel)) {
        demand(statSync(original).size <= 8 * 1024 * 1024, 'resource exceeds bounded size');
        let bytes = readFileSync(original);
        demand(bytes.length <= 8 * 1024 * 1024, 'resource exceeds bounded size');
        const blob = data => createHash('sha1').update('blob ' + data.length + '\0').update(data).digest('hex');
        // Git autocrlf may expand checkout text. Only accept normalization
        // when its resulting bytes match the exact committed resource blob.
        if (blob(bytes) !== resourceBlobs.get(rel)) {
          bytes = Buffer.from(bytes.toString('latin1').replace(/\r\n/g, '\n'), 'latin1');
        }
        demand(blob(bytes) === resourceBlobs.get(rel), 'copied resource differs from committed Git HEAD: ' + rel);
        writeFileSync(dest, bytes);
      } else {
        copyFileSync(original,dest);
      }
      const meta = {path:destinationFor(rel),source:rel,
        sizeBytes:statSync(dest).size, sha256:await checksum(dest)};
      const expected = report.artifacts.find(x => x.path === rel);
      demand(!expected || (meta.sha256 === expected.sha256 &&
        meta.sizeBytes === expected.bytes), 'copied binary changed');
      if (resourceBlobs.has(rel)) {
        demand(await gitBlobId(dest) === resourceBlobs.get(rel),
          'copied resource differs from committed Git HEAD: ' + rel);
      }
      artifacts.push(meta);
    }
    const provenance = {
      schema:2, state:PAYLOAD_STATE,
      sourceVersion:'v1.18.6',
      upstreamRepository:'sandboxie-plus/Sandboxie',
      upstreamTag:'v1.18.6', upstreamCommit:profile.sourceCommit,
      upstreamTree:profile.pristineSourceTree, ownedRepoHead:head,
      ownedTree, nativeModifiedFiles:11,
      buildReportSha256:reportHash, compilerSha256:report.compiler.sha256,
      canCoexistWithOfficialSandboxie:false,
      buildStages:profile.buildStages.map(s=>({id:s.id,platform:s.platform,state:'COMPILED'})),
      artifacts,
      windowsKernelDriverLoaded:false,
      minecraftGameplayVerified:false,
      sandboxieExternalDependencyRemoved:false,
      publicDistributionApproved:false,
    };
    writeFileSync(join(payload,'engine-provenance.json'),
      JSON.stringify(provenance,null,2)+'\n',{flag:'wx'});
    writeFileSync(join(payload,'NOT-INSTALLABLE.txt'),MARKER,{flag:'wx'});
    demand(!exists(target), 'payload destination appeared while copying');
    renameSync(payload,target);
    return {state:PAYLOAD_STATE,artifactCount:artifacts.length,
      directory:target,providerSelected:false,installPermitted:false,
      driverLoaded:false,gameplayVerified:false};
  } finally {
    rmSync(temporary,{recursive:true,force:true});
  }
}
function ordinarySource(root, relativePath) {
  const source = resolve(root,relativePath);
  demand(inside(root,source), 'source outside repository');
  let step = root;
  for (const part of relativePath.split('/')) {
    step = join(step,part);
    demand(!lstatSync(step).isSymbolicLink(), 'symlinked source root');
  }
  demand(lstatSync(source).isDirectory(), 'owned source missing');
  return source;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args=process.argv.slice(2);
    demand(args.length === 4 && args[0] === '--build-report' && args[2] === '--output',
      'usage: node assemble-owned-v1186.mjs --build-report <build-result.json> --output <new-absolute-directory>');
    const result=await stageOwnedV1186({buildReport:args[1],outputDirectory:args[3]});
    process.stdout.write(JSON.stringify(result,null,2)+'\n');
  } catch (error) {
    process.stderr.write(error.message+'\n');
    process.exitCode=1;
  }
}
