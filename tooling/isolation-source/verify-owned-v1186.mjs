/**
 * Read-only proof that Local owns the exact full Sandboxie v1.18.6 source
 * plus thirteen native MIVUBI changes and four exact build-path fixes.
 */
import {spawnSync} from 'node:child_process';
import {resolve, dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {
  SOURCE_FILES, buildPrivateSourcePatch, verifyExactPrivateSourceBytes,
} from '../../engine/isolation-native/fork-overlay/apply-private-source.mjs';
import {CREDENTIAL_FILE,STORAGE_FILE,applyOwnedCredentialIsolation,applyOwnedProtectedStorageIsolation} from './owned-credential-policy.mjs';
export const NATIVE_FILES=Object.freeze([...SOURCE_FILES,CREDENTIAL_FILE,STORAGE_FILE]);

export const V1186_SOURCE_TREE = 'f690477c07a40810f479f53973334ceae7e0ab13';
export const V1186_FILE_COUNT = 1627;
export const V1186_CANDIDATE = 'engine/isolation-candidates/sandboxie-v1.18.6';
export const V1186_OWNED = 'engine/isolation-owned';
const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
export const BUILD_FILES = Object.freeze([
  'Sandboxie/core/low/LowLevel.vcxproj',
  'Sandboxie/msgs/Parse.vcxproj',
  'Sandboxie/core/dll/SboxDll.vcxproj',
  'Sandboxie/core/drv/SboxDrv.vcxproj',
]);

/** Only quote compiler paths; no compiler flags, source or driver policy changes. */
export function applyOwnedBuildPathFix(path, original) {
  let text = original.replace(/\r\n/g, '\n');
  const counts = new Map([[BUILD_FILES[0], [1, 3]],
    [BUILD_FILES[2], [2, 12]], [BUILD_FILES[3], [4, 8]]]);
  const changes = counts.has(path) ? [
    [String.raw`-Fo$(Configuration)\%(Filename).obj %(FullPath)`,
      String.raw`-Fo"$(Configuration)\%(Filename).obj" "%(FullPath)"`, counts.get(path)[0]],
    [String.raw`-Fo$(Platform)\$(Configuration)\%(Filename).obj %(FullPath)`,
      String.raw`-Fo"$(Platform)\$(Configuration)\%(Filename).obj" "%(FullPath)"`, counts.get(path)[1]],
  ] : path === BUILD_FILES[1] ? [[
    String.raw`mc /h $(MSBuildProjectDirectory)\$(Configuration) /r $(MSBuildProjectDirectory)\$(Configuration) -u $(MSBuildProjectDirectory)\msgs.mc`,
    String.raw`mc /h "$(MSBuildProjectDirectory)\$(Configuration)" /r "$(MSBuildProjectDirectory)\$(Configuration)" -u "$(MSBuildProjectDirectory)\msgs.mc"`, 1,
  ]] : [];
  if (path === BUILD_FILES[2]) changes.push([
    String.raw`lib /def:%(FullPath) /out:$(SolutionDir)Bin\$(PlatformName)\$(Configuration)\NtCRT.lib`,
    String.raw`lib /def:"%(FullPath)" /out:"$(SolutionDir)Bin\$(PlatformName)\$(Configuration)\NtCRT.lib"`, 8,
  ]);
  requireFact(changes.length > 0, 'unknown build patch path');
  for (const [before, after, count] of changes) {
    requireFact(text.split(before).length - 1 === count, 'build patch anchor drift: ' + path);
    text = text.replaceAll(before, () => after);
  }
  return text;
}

function requireFact(ok, reason) {
  if (!ok) throw Error('MIVUBI v1.18.6 source refused: ' + reason);
}
function git(root, args) {
  const child = spawnSync('git', ['-C', root, ...args], {
    windowsHide: true, maxBuffer: 12 * 1024 * 1024, timeout: 120000,
  });
  requireFact(!child.error && child.status === 0,
    'git ' + args[0] + ' failed: ' +
    (child.stderr?.toString('utf8') || child.error?.message || '').slice(0,350));
  return child.stdout;
}
function gitText(root, args) { return git(root, args).toString('utf8').trim(); }

function lsOwnedTree(root, path) {
  const rows = git(root, ['ls-tree', '-r', '-z', 'HEAD', '--', path]);
  const map = new Map();
  for (const record of rows.toString('utf8').split('\0')) {
    if (!record) continue;
    const divider = record.indexOf('\t');
    requireFact(divider > 0, 'invalid git tree entry');
    const [mode, kind, sha] = record.slice(0, divider).split(' ');
    const fullPath = record.slice(divider + 1);
    requireFact(fullPath.startsWith(path + '/') && kind === 'blob' &&
      (mode === '100644' || mode === '100755') && /^[a-f0-9]{40}$/.test(sha),
      'foreign tree object ' + fullPath);
    const relative = fullPath.slice(path.length + 1);
    requireFact(!map.has(relative), 'duplicate Git path ' + relative);
    map.set(relative, {mode, sha});
  }
  return map;
}

/** Pure tree-diff gate; unit tests cover missing, alien and modified paths. */
export function assessOwnedV1186(candidate, owned, allowed = [...NATIVE_FILES, ...BUILD_FILES],
  expectedCount = V1186_FILE_COUNT) {
  requireFact(candidate instanceof Map && owned instanceof Map,
    'Git tree maps required');
  requireFact(candidate.size === expectedCount && owned.size === expectedCount,
    'source tree incomplete or has extra files');
  requireFact(new Set(allowed).size === allowed.length, 'duplicate native patch path');
  const changed = [];
  for (const [path, upstream] of candidate) {
    const current = owned.get(path);
    requireFact(current && current.mode === upstream.mode,
      'missing or changed file mode: ' + path);
    requireFact(/^[a-f0-9]{40}$/.test(upstream.sha) &&
      /^[a-f0-9]{40}$/.test(current.sha), 'invalid Git blob SHA: ' + path);
    if (current.sha !== upstream.sha) changed.push(path);
  }
  requireFact(owned.size === candidate.size, 'unapproved owned source entry');
  const expected = [...allowed].sort();
  requireFact(JSON.stringify(changed.sort()) === JSON.stringify(expected),
    'native source delta is not the exact approved source/build closure');
  return {sourceFiles: candidate.size, nativeModifiedFiles: changed.length,
    nativePaths: changed, sourceTreeComplete: true, nativeBuildVerified: false,
    signedDriverLoaded: false, minecraftGameplayVerified: false};
}

export function verifyOwnedV1186(repoRoot = ROOT) {
  const root = resolve(repoRoot);
  requireFact(gitText(root, ['branch', '--show-current']) === 'Local',
    'verification requires development branch Local');
  const head = gitText(root, ['rev-parse', 'HEAD']);
  const upstreamTree = gitText(root, ['rev-parse', 'HEAD:' + V1186_CANDIDATE]);
  requireFact(upstreamTree === V1186_SOURCE_TREE,
    'upstream source candidate must remain byte-exact and unmodified');
  const candidate = lsOwnedTree(root, V1186_CANDIDATE);
  const owned = lsOwnedTree(root, V1186_OWNED);
  const summary = assessOwnedV1186(candidate, owned);
  const original = {};
  const modified = {};
  for (const path of SOURCE_FILES) {
    original[path] = git(root, ['cat-file', 'blob', candidate.get(path).sha])
      .toString('utf8');
    modified[path] = git(root, ['cat-file', 'blob', owned.get(path).sha])
      .toString('utf8');
  }
  // Reconstruct the exact reviewed native transformation from pristine
  // upstream bytes. No trust is delegated to filenames or self-reported hashes.
  const patch = buildPrivateSourcePatch(original);
  requireFact((patch.match(/^@@ /gm) || []).length === 24,
    'native source patch closure changed');
  verifyExactPrivateSourceBytes(original, modified);
  const credentials=git(root,['cat-file','blob',candidate.get(CREDENTIAL_FILE).sha]).toString('utf8');
  const isolated=git(root,['cat-file','blob',owned.get(CREDENTIAL_FILE).sha]).toString('utf8');
  requireFact(isolated.replace(/\r\n/g,'\n')===applyOwnedCredentialIsolation(credentials),
    'credential source differs from reviewed boxed-only policy');
  const storage=git(root,['cat-file','blob',candidate.get(STORAGE_FILE).sha]).toString('utf8');
  const boxedStorage=git(root,['cat-file','blob',owned.get(STORAGE_FILE).sha]).toString('utf8');
  requireFact(boxedStorage.replace(/\r\n/g,'\n')===applyOwnedProtectedStorageIsolation(storage),
    'protected storage source differs from reviewed private policy');
  for (const path of BUILD_FILES) {
    const pristine = git(root, ['cat-file', 'blob', candidate.get(path).sha]).toString('utf8');
    const current = git(root, ['cat-file', 'blob', owned.get(path).sha]).toString('utf8');
    requireFact(current.replace(/\r\n/g, '\n') === applyOwnedBuildPathFix(path, pristine),
      'build project differs from reviewed path quoting: ' + path);
  }
  return {state: 'MIVUBI_V1186_NATIVE_SOURCE_PATCHED_NOT_BUILT', head,
    upstreamTree, ownedTree: gitText(root, ['rev-parse', 'HEAD:' + V1186_OWNED]),
    ...summary, totalModifiedFiles: summary.nativeModifiedFiles,
    nativeModifiedFiles: NATIVE_FILES.length, nativeBuildModifiedFiles: BUILD_FILES.length,
    nativePaths: summary.nativePaths.filter(path => NATIVE_FILES.includes(path)),
    nativeBuildPaths: summary.nativePaths.filter(path => BUILD_FILES.includes(path)),
    activeProviderChanged: false, installationPermitted: false};
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { process.stdout.write(JSON.stringify(verifyOwnedV1186(), null, 2) + '\n'); }
  catch (error) { process.stderr.write(error.message + '\n'); process.exitCode = 1; }
}
