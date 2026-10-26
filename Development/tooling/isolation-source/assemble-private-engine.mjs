// Private-exclusive core artifacts -> the EXISTING owned engine payload layout.
// Refuses to treat a modified fork report as an unmodified upstream build.
// No driver/service install and no Tauri backend switch.
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { createReadStream, existsSync, lstatSync, readFileSync, realpathSync } from 'node:fs';
import { resolve, join, relative, isAbsolute, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { assembleEngine } from './assemble-engine.mjs';
import { verifyManifest } from './validate-baseline.mjs';
import { SOURCE_FILES, verifyCommittedPrivateFork } from '../../engine/isolation-native/fork-overlay/apply-private-source.mjs';

const PIN = 'abd4c240031cfb46666daa74ca47ed4e48b561e4';
const SHA = /^[a-f0-9]{64}$/;
const SHA_COMMIT = /^[a-f0-9]{40}$/;
const SHA_TREE = /^[a-f0-9]{40}$/;
const STATUS = 'PRIVATE_CORE_BINARIES_BUILT_NOT_SIGNED_OR_RUN';
const refuse = (yes, message) => {
  if (!yes) throw new Error('Private core staging refused: ' + message);
};
/** One committed-source verifier shared with the Windows build. */
export function verifyPrivateForkCheckout(root, report) {
  const observed = verifyCommittedPrivateFork(root);
  refuse(observed.privateForkHead === report.privateForkHead &&
    observed.privateForkTree === report.privateForkTree &&
    observed.privatePatchSha256 === report.privatePatchSha256,
    'committed source identity differs from build report');
  return observed;
}
function safeFile(root, rel) {
  refuse(typeof rel === 'string' && /^[A-Za-z0-9._/-]+$/.test(rel) &&
    !rel.split('/').includes('..') && !rel.startsWith('/'), 'invalid source path');
  const file = resolve(root, ...rel.split('/'));
  const remainder = relative(root, file);
  refuse(remainder !== '..' && !remainder.startsWith('..'+sep) &&
    !isAbsolute(remainder), 'source escapes the fork checkout');
  let part = root;
  for (const component of rel.split('/')) {
    part = join(part, component);
    const stat = lstatSync(part);
    refuse(!stat.isSymbolicLink(), 'symlinked source artifact');
  }
  refuse(lstatSync(file).isFile(), 'source artifact not a file');
  return file;
}
async function sizeAndHash(path) {
  const hash = createHash('sha256');
  let size = 0;
  for await (const data of createReadStream(path)) {
    size += data.length;
    hash.update(data);
  }
  refuse(Number.isSafeInteger(size) && size > 0, 'source file size invalid');
  return {sizeBytes:size, sha256:hash.digest('hex')};
}
function checkReport(manifest, report) {
  // Same complete version-locked inventory as the upstream baseline builder.
  // No reduced private manifest may claim all core artifacts were built.
  verifyManifest(manifest);
  refuse(manifest?.upstream?.commit === PIN &&
    report?.schema === 1 && report.state === STATUS &&
    report.upstreamCommit === PIN && SHA_COMMIT.test(report.privateForkHead) &&
    report.privateForkHead !== PIN && SHA_TREE.test(report.privateForkTree) &&
    SHA.test(report.privatePatchSha256) &&
    report.sourceBuildMarker === 'MIVUBI_PRIVATE_ISOLATION_BUILD' &&
    JSON.stringify(report.sourceFilesModified) === JSON.stringify(SOURCE_FILES) &&
    report.driverSigningPolicyAccepted === false &&
    report.driverLoadVerified === false && report.installerReady === false &&
    report.canCoexistWithOfficialSandboxie === false &&
    report.microsoftSignInVerified === false &&
    report.minecraftGameplayVerified === false &&
    report.distributionApproved === false,
    'untrusted, incomplete or overstated private build provenance');
  const stages = manifest.buildStages;
  refuse(Array.isArray(report.buildStages) &&
    report.buildStages.length === stages.length &&
    report.buildStages.every((s,i)=>s.id===stages[i].id &&
      s.platform===stages[i].platform && s.exitCode===0),
    'native Win32/x64/driver compilation not proven');
  refuse(Array.isArray(report.artifacts) &&
    report.artifacts.length === manifest.requiredArtifacts.length &&
    report.artifacts.every((item,i)=>
      item.path===manifest.requiredArtifacts[i] &&
      SHA.test(item.sha256) && Number.isSafeInteger(item.sizeBytes) &&
      item.sizeBytes>0 && typeof item.authenticode === 'string'),
    'private build artifact list/order/hash invalid');
}
async function inspectPrivateArtifacts(root, manifest, report) {
  for (const row of report.artifacts) {
    const actual = await sizeAndHash(safeFile(root,row.path));
    refuse(actual.sha256===row.sha256 && actual.sizeBytes===row.sizeBytes,
      'private binary bytes do not match the compiled report: ' + row.path);
  }
  const resourceRows = [];
  for (const path of manifest.sourceResources) {
    const meta = await sizeAndHash(safeFile(root,path));
    resourceRows.push({path,kind:'source-resource',state:'PRESENT',...meta,
      authenticode:'NOT_EXECUTED'});
  }
  const symbols = [];
  for (const path of manifest.optionalSymbols) {
    const f = resolve(root,...path.split('/'));
    if (existsSync(f)) {
      const meta = await sizeAndHash(safeFile(root,path));
      symbols.push({path,kind:'optional-symbol',state:'PRESENT',...meta,
        authenticode:'NOT_EXECUTED'});
    } else {
      // Dangling symlink must not be silently reported as missing.
      try {
        const stat = lstatSync(f);
        refuse(!stat.isSymbolicLink(), 'dangling optional symbol symlink');
      } catch (error) {
        if (error.code !== 'ENOENT') throw error;
      }
      symbols.push({path,kind:'optional-symbol',state:'MISSING',
        sizeBytes:null,sha256:null,authenticode:'NOT_EXECUTED'});
    }
  }
  for (const path of manifest.upstream.sourceLicenseFiles) {
    safeFile(root,path);
  }
  return [
    ...report.artifacts.map(item=>({...item,kind:'required',state:'PRESENT'})),
    ...resourceRows,...symbols,
  ];
}

/** Generates a private package with its identity, never a false upstream one. */
export async function stagePrivateCore({
  sourceRoot, manifest, report, outputDirectory,
}, {checkSource = verifyPrivateForkCheckout} = {}) {
  refuse(typeof sourceRoot === 'string' && sourceRoot.trim(), 'source directory required');
  const root = realpathSync(resolve(sourceRoot));
  checkReport(manifest,report);
  const source = await checkSource(root,report);
  refuse(source?.privateForkHead === report.privateForkHead &&
    source?.privateForkTree === report.privateForkTree &&
    source?.privatePatchSha256 === report.privatePatchSha256,
    'committed source proof did not match build report');
  const data = await inspectPrivateArtifacts(root,manifest,report);
  const normalized = {
    compiledInThisInvocation:true,
    artifactResult:'BUILD_ARTIFACTS_COLLECTED',
    stageResults:report.buildStages.map(s=>({
      id:s.id,platform:s.platform,state:'COMPILED',
    })),
    artifacts:data,
  };
  const privateBuild = {
    kind:'PRIVATE_EXCLUSIVE_SOURCE',
    privateForkHead:report.privateForkHead,
    privateForkTree:report.privateForkTree,
    privatePatchSha256:report.privatePatchSha256,
    sourceBuildMarker:report.sourceBuildMarker,
    sourceFilesModified:[...SOURCE_FILES],
    canCoexistWithOfficialSandboxie:false,
  };
  // Reuse exactly the existing layout, atomic output and attribution owner.
  // Source and file-byte checks were performed above against the PRIVATE
  // report rather than claiming an unmodified upstream build.
  return assembleEngine({
    sourceRoot:root,manifest,report:normalized,outputDirectory,privateBuild,
  }, {
    checkSource:()=>undefined,
    checkFiles:()=>undefined,
  });
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    refuse(args.length===8 && args[0]==='--fork-root' &&
      args[2]==='--manifest' && args[4]==='--private-report' &&
      args[6]==='--output',
      'usage: assemble-private-engine.mjs --fork-root <private-checkout> --manifest <baseline-manifest> --private-report <private-build.json> --output <new-dir>');
    const json = p=>JSON.parse(readFileSync(resolve(p),'utf8').replace(/^\uFEFF/,''));
    const result = await stagePrivateCore({
      sourceRoot:args[1],manifest:json(args[3]),
      report:json(args[5]),outputDirectory:args[7],
    });
    process.stdout.write(JSON.stringify(result,null,2)+'\n');
  } catch (error) {
    process.stderr.write(error.message+'\n');
    process.exitCode=1;
  }
}
