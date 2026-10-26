/**
 * Import ALL tracked files from the authentic pinned Sandboxie tag into the
 * MIVUBI application repository as a writable Git subtree snapshot.
 *
 * This does NOT install, load, compile or distribute a driver. The full source
 * remains subject to its ORIGINAL component licenses and attributions.
 *
 * CLI: node Development/tooling/isolation-source/import-full-sandboxie.mjs \
 *   --seed Development/engine/isolation-upstream/Sandboxie --expect-head <Local SHA>
 */
import { spawnSync } from 'node:child_process';
import { existsSync, lstatSync, realpathSync, mkdirSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { dirname, resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { verifyPinnedSeed } from './bootstrap-fork.mjs';
import { sameGitDirectory } from './git-directory-identity.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const UPSTREAM = 'https://github.com/sandboxie-plus/Sandboxie.git';
const PIN = 'abd4c240031cfb46666daa74ca47ed4e48b561e4';
const TAG = 'v1.18.5';
export const OWNED_PATH = 'Development/engine/isolation-owned';

function refuse(condition, message) {
  if (!condition) throw new Error('Full Sandboxie source import refused: ' + message);
}
function git(root, args, timeout = 120000) {
  const r = spawnSync('git', ['-C', root, ...args], {
    encoding: 'utf8', windowsHide: true, timeout, maxBuffer: 8 * 1024 * 1024,
  });
  refuse(!r.error && r.status === 0,
    `Git ${args[0]} failed: ${(r.error?.message || r.stderr || '').slice(0, 700)}`);
  return (r.stdout || '').trim();
}
function ordinaryDirectory(path) {
  let s;
  try { s = lstatSync(path); } catch { return false; }
  return s.isDirectory() && !s.isSymbolicLink() &&
    realpathSync(path) === resolve(path);
}

/** A one-time source COPY into the app's own Git tree, not a Git submodule.
 * Test-only overrides permit synthetic disposable Git fixtures. The CLI has
 * no options to override the authentic tag, root, prefix or upstream URL.
 */
export function importFullSandboxie({
  repoRoot, seedRoot, expectedHead, upstreamCommit = PIN, tag = TAG,
  upstreamUrl = UPSTREAM,
}) {
  refuse(typeof expectedHead === 'string' && /^[a-f0-9]{40}$/.test(expectedHead),
    'an exact parent Local SHA is mandatory');
  const root = resolve(repoRoot), seed = resolve(root, seedRoot);
  refuse(ordinaryDirectory(root), 'application Git root must be an ordinary directory');
  refuse(sameGitDirectory(root,git(root, ['rev-parse', '--show-toplevel'])),
    'must target the application repository root');
  refuse(git(root, ['branch', '--show-current']) === 'Local',
    'only the Local branch may receive the owned source');
  refuse(git(root, ['rev-parse', 'HEAD']) === expectedHead,
    'Local moved since the import was approved');
  refuse(git(root, ['status', '--porcelain=v1', '--untracked-files=all']) === '',
    'the application working tree or index is dirty');
  const owned = join(root, ...OWNED_PATH.split('/'));
  const ownedTracked = git(root, ['ls-tree', '-r', '--name-only', 'HEAD', '--', OWNED_PATH]);
  // An interrupted, already-committed copy is resumed without overwriting
  // any existing Git blob. Never interpret an untracked directory as a seed.
  if (existsSync(owned)) {
    refuse(ordinaryDirectory(owned), 'owned source must be an ordinary directory');
    refuse(ownedTracked !== '', 'owned directory already exists without committed source');
    refuse(ordinaryDirectory(seed), 'pinned source seed is missing or redirected');
    verifyPinnedSeed(seed, { expectedCommit: upstreamCommit, tag, upstreamUrl });
    return completePartialImport({root, seed, upstreamCommit, ownedTracked});
  }
  refuse(ownedTracked === '', 'owned source is already present in committed Git history');
  // The seed is a complete separate checkout, normally the initialized
  // pinned submodule. Never import from an official installed Sandboxie box.
  refuse(ordinaryDirectory(seed), 'pinned source seed is missing or redirected');
  verifyPinnedSeed(seed, { expectedCommit: upstreamCommit, tag,
    upstreamUrl });
  const sourceTree = git(seed, ['rev-parse', `${upstreamCommit}^{tree}`]);
  const tracked = git(seed, ['ls-tree', '-r', '--name-only', upstreamCommit]);
  for (const required of [
    'LICENSE.Plus', 'LICENSE.Classic', 'Sandboxie/COPYING',
    'SandboxiePlus/QSbieAPI/LICENSE', 'Sandboxie/core/drv/box.c',
    'Sandboxie/core/svc/DriverAssist.cpp', 'Sandboxie/core/dll/sbieapi.c',
    'Sandboxie/apps/start/start.cpp', 'SandboxiePlus/SandMan',
  ]) {
    // SandMan is a directory prefix; all other entries are tracked files.
    refuse(required === 'SandboxiePlus/SandMan' ?
      tracked.split('\n').some(p => p.startsWith(required + '/')) :
      tracked.split('\n').includes(required),
    'source seed lacks expected pinned component: ' + required);
  }
  // Squash avoids inflating the app repository with decades of unrelated
  // upstream commits, but the entire pinned file tree is copied byte-for-byte.
  // The original commit and the git subtree provenance remain recorded.
  git(root, ['subtree', 'add', '--prefix=' + OWNED_PATH, '--squash',
    seed, upstreamCommit, '-m',
    'feat(engine): import complete pinned Sandboxie source tree'], 600000);
  const importedTree = git(root, ['rev-parse', `HEAD:${OWNED_PATH}`]);
  refuse(importedTree === sourceTree,
    'imported source tree differs from the authentic pinned upstream tree');
  refuse(git(root, ['status', '--porcelain=v1', '--untracked-files=all']) === '',
    'subtree import did not leave a clean repository');
  return {
    state: 'OWNED_SANDBOXIE_FULL_SOURCE_IMPORTED_NOT_BUILT',
    upstreamCommit, sourceTree, importedTree, ownedPath: OWNED_PATH,
    importedFileCount: tracked.split('\n').filter(Boolean).length,
    appCommit: git(root, ['rev-parse', 'HEAD']),
    independentSourceFiles: true, privateEngineCompiled: false,
    driverInstalled: false, minecraftValidated: false,
  };
}


/**
 * Resume the genuine pinned source tree after a partial direct GitHub import.
 * Existing files are NEVER overwritten. Validate every committed blob/mode
 * against the pinned Git tree before creating any missing file.
 *
 * Writes are performed only in a clean Local checkout. The staged subtree
 * MUST equal the upstream tree exactly before the commit may occur.
 */
function completePartialImport({root, seed, upstreamCommit, ownedTracked}) {
  const sourceTree = git(seed, ['rev-parse', upstreamCommit + '^{tree}']);
  const list = (where, args) => git(where, args).split('\n').filter(Boolean).map(line => {
    const match = /^([0-7]{6}) blob ([a-f0-9]{40})\t(.+)$/.exec(line);
    refuse(match, 'unexpected tree entry (symlink/submodule or unparsable path)');
    const [, mode, sha, path] = match;
    refuse(mode === '100644' || mode === '100755', 'unsupported source mode');
    refuse(!path.startsWith('/') && !path.includes('\\') &&
      path.split('/').every(part => part && part !== '.' && part !== '..'),
      'unsafe source path');
    return {mode, sha, path};
  });
  const upstream = list(seed, ['ls-tree', '-r', upstreamCommit]);
  const original = list(root, ['ls-tree', '-r', 'HEAD', '--', OWNED_PATH]).map(f => {
    refuse(f.path.startsWith(OWNED_PATH + '/'), 'unexpected owned path');
    return {...f, path:f.path.slice(OWNED_PATH.length + 1)};
  });
  refuse(original.length === ownedTracked.split('\n').filter(Boolean).length,
    'unexpected existing source count');
  const expected = new Map(upstream.map(f => [f.path, f]));
  refuse(expected.size === upstream.length && upstream.length > 0,
    'incomplete or duplicate upstream file paths');
  for (const current of original) {
    const upstreamFile = expected.get(current.path);
    refuse(upstreamFile && upstreamFile.mode === current.mode &&
      upstreamFile.sha === current.sha,
      'existing owned source differs from upstream: ' + current.path);
  }
  const present = new Set(original.map(f => f.path));
  const missing = upstream.filter(f => !present.has(f.path));
  if (missing.length === 0) {
    refuse(git(root, ['rev-parse', 'HEAD:' + OWNED_PATH]) === sourceTree,
      'all files exist but owned tree differs from upstream');
    return {state:'OWNED_SANDBOXIE_FULL_SOURCE_IMPORTED_NOT_BUILT',
      upstreamCommit, sourceTree, importedTree:sourceTree,
      importedFileCount:upstream.length, newlyImportedFileCount:0,
      ownedPath:OWNED_PATH, appCommit:git(root, ['rev-parse', 'HEAD']),
      driverInstalled:false, minecraftValidated:false};
  }
  const ownRoot = join(root, ...OWNED_PATH.split('/'));
  for (const file of missing) {
    const dest = join(ownRoot, ...file.path.split('/'));
    refuse(!existsSync(dest), 'refusing to overwrite a source file: ' + file.path);
    // cat-file copies original blob bytes; text and binary are equivalent.
    const result = spawnSync('git', ['-C', seed, 'cat-file', 'blob', file.sha],
      {encoding:null, windowsHide:true, timeout:120000, maxBuffer:16*1024*1024});
    refuse(!result.error && result.status === 0, 'failed to read seed blob: ' + file.path);
    const bytes = result.stdout;
    const digest = createHash('sha1').update(
      Buffer.from('blob ' + bytes.length + '\0')).update(bytes).digest('hex');
    refuse(digest === file.sha, 'seed blob hash mismatch: ' + file.path);
    mkdirSync(dirname(dest), {recursive:true});
    writeFileSync(dest, bytes, {flag:'wx', mode:file.mode === '100755' ? 0o755 : 0o644});
  }
  // Index the new files but do not permit Git attributes or Windows EOL
  // conversion to change the pinned source. Exact staged Git tree is the gate.
  git(root, ['add', '--', OWNED_PATH], 600000);
  for (const file of missing.filter(f => f.mode === '100755')) {
    git(root, ['update-index', '--chmod=+x', '--', OWNED_PATH + '/' + file.path]);
  }
  const stagedRoot = git(root, ['write-tree'], 600000);
  const importedTree = git(root, ['rev-parse', stagedRoot + ':' + OWNED_PATH]);
  refuse(importedTree === sourceTree,
    'staged full-source tree differs from the pinned upstream (no commit made)');
  git(root, ['diff', '--cached', '--check']);
  git(root, ['commit', '-m',
    'feat(engine): complete pinned Sandboxie source ownership'], 600000);
  refuse(git(root, ['rev-parse', 'HEAD:' + OWNED_PATH]) === sourceTree,
    'committed source tree differs from the pinned upstream');
  refuse(git(root, ['status', '--porcelain=v1', '--untracked-files=all']) === '',
    'repository not clean after source import');
  return {state:'OWNED_SANDBOXIE_FULL_SOURCE_IMPORTED_NOT_BUILT',
    upstreamCommit, sourceTree, importedTree,
    importedFileCount:upstream.length, newlyImportedFileCount:missing.length,
    ownedPath:OWNED_PATH, appCommit:git(root, ['rev-parse', 'HEAD']),
    driverInstalled:false, minecraftValidated:false};
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (args.length !== 4 || args[0] !== '--seed' ||
        args[2] !== '--expect-head') {
      throw new Error('Usage: node import-full-sandboxie.mjs --seed <pinned-source-checkout> --expect-head <Local-SHA>');
    }
    const result = importFullSandboxie({ repoRoot: ROOT,
      seedRoot: args[1], expectedHead: args[3] });
    process.stdout.write(JSON.stringify(result, null, 2) + '\n');
  } catch (error) {
    process.stderr.write(error.message + '\n');
    process.exitCode = 1;
  }
}
