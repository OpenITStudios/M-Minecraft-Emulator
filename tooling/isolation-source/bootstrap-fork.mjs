import { spawnSync } from 'node:child_process';
import { existsSync, lstatSync, mkdirSync, mkdtempSync, renameSync, rmSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { sameGitDirectory } from './git-directory-identity.mjs';

const UPSTREAM = 'https://github.com/sandboxie-plus/Sandboxie.git';
const PIN = 'abd4c240031cfb46666daa74ca47ed4e48b561e4';
const TAG = 'v1.18.5';
const BRANCH = 'isolation/baseline-v1.18.5';

function git(args, options = {}) {
  const result = spawnSync('git', args, {
    encoding: 'utf8', timeout: 600000, maxBuffer: 2 * 1024 * 1024,
    windowsHide: true, ...options,
  });
  if (result.error) throw new Error(`Git invocation failed: ${result.error.message}`);
  if (result.status !== 0) throw new Error(`Git command failed (${args.join(' ')}): ${(result.stderr || '').trim().slice(0, 700)}`);
  return result.stdout.trim();
}

function destinationOccupied(path) {
  try { lstatSync(path); return true; }
  catch (error) { if (error.code === 'ENOENT') return false; throw error; }
}

// A checked-out submodule can seed a full independent local Git clone, without
// needing another network fetch. This does not make a GitHub remote fork.
export function verifyPinnedSeed(seed, { expectedCommit = PIN, upstreamUrl = UPSTREAM, tag = TAG } = {}) {
  const root = resolve(seed);
  if (!existsSync(root) || !lstatSync(root).isDirectory() || lstatSync(root).isSymbolicLink()) {
    throw new Error('Pinned upstream seed must be a real directory');
  }
  const run = (...args) => git(['-C', root, ...args]);
  if (!sameGitDirectory(root,run('rev-parse', '--show-toplevel')) ||
      run('rev-parse', '--is-shallow-repository') !== 'false' ||
      run('rev-parse', 'HEAD') !== expectedCommit ||
      run('rev-parse', '--verify', `refs/tags/${tag}^{commit}`) !== expectedCommit) {
    throw new Error('Seed is not the complete pinned Sandboxie source');
  }
  const origin = run('config', '--get', 'remote.origin.url');
  if (origin !== upstreamUrl) throw new Error('Seed upstream URL does not match approved repository');
  if (run('status', '--porcelain=v1', '--untracked-files=all') !== '') {
    throw new Error('Seed contains uncommitted or untracked changes');
  }
  return root;
}

// Dependency-injectable source/pin exist for tests ONLY. The CLI defaults
// to the reviewed upstream SHA, tag and official GitHub repository.
export function bootstrapFork({ destination, source = UPSTREAM, expectedCommit = PIN,
  tag = TAG, branch = BRANCH } = {}) {
  if (typeof destination !== 'string' || !destination.trim()) throw new Error('Destination required');
  if (!/^[a-f0-9]{40}$/.test(expectedCommit) || !/^v\d+\.\d+\.\d+$/.test(tag) ||
      !/^isolation\/[a-zA-Z0-9._-]+$/.test(branch)) throw new Error('Invalid source identity arguments');
  if (typeof source !== 'string' || !source) throw new Error('Source required');
  const target = resolve(destination);
  const parent = dirname(target);
  if (!existsSync(parent) || !lstatSync(parent).isDirectory() || lstatSync(parent).isSymbolicLink()) {
    throw new Error('Destination parent must be an existing, ordinary directory');
  }
  if (destinationOccupied(target)) throw new Error('Destination exists; refusing to overwrite');
  // mkdtemp creates a private sibling on the same filesystem for atomic promotion.
  const staging = mkdtempSync(join(parent, '.mivubi-isolation-bootstrap-'));
  const checkout = join(staging, 'checkout');
  const hooks = join(staging, 'hooks-disabled');
  mkdirSync(hooks);
  const options = ['-c', `core.hooksPath=${hooks}`];
  try {
    git([...options, 'clone', '--no-local', '--no-single-branch', '--', source, checkout]);
    const at = (...args) => git([...options, '-C', checkout, ...args]);
    const actualTag = at('rev-parse', '--verify', `refs/tags/${tag}^{commit}`);
    if (actualTag !== expectedCommit) throw new Error('Pinned upstream tag/commit mismatch');
    if (at('rev-parse', '--is-shallow-repository') !== 'false') throw new Error('Shallow clone not accepted');
    at('switch', '--create', branch, expectedCommit);
    at('remote', 'rename', 'origin', 'upstream');
    // The resulting standalone source always records the real upstream URL,
    // including when cloned from a previously authenticated local submodule.
    at('remote', 'set-url', 'upstream', UPSTREAM);
    if (at('rev-parse', 'HEAD') !== expectedCommit) throw new Error('Working branch does not match pinned commit');
    if (at('status', '--porcelain=v1', '--untracked-files=all') !== '') throw new Error('Unexpected dirty checkout');
    const historyCount = Number(at('rev-list', '--count', 'HEAD'));
    if (!Number.isSafeInteger(historyCount) || historyCount < 1) throw new Error('Unverifiable Git history');
    // Directory must still be absent; never rename over existing user data.
    if (destinationOccupied(target)) throw new Error('Destination appeared during clone; refusing overwrite');
    renameSync(checkout, target);
    return { status: 'PINNED_HISTORY_PRESERVED', checkout: target,
      upstreamCommit: expectedCommit, upstreamTag: tag, branch, historyCount,
      shallow: false, pushedToGitHub: false, binaryBuilt: false, driverInstalled: false };
  } finally {
    rmSync(staging, { recursive: true, force: true });
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    if (!((args.length === 2 || args.length === 4) && args[0] === '--destination') ||
        (args.length === 4 && args[2] !== '--local-seed')) {
      throw new Error('Usage: node bootstrap-fork.mjs --destination <new-directory> [--local-seed <pinned-submodule>]');
    }
    const source = args.length === 4 ? verifyPinnedSeed(args[3]) : UPSTREAM;
    const result = bootstrapFork({ destination: args[1], source });
    process.stdout.write(JSON.stringify(result, null, 2) + '\n');
  } catch (error) {
    process.stderr.write(error.message + '\n');
    process.exitCode = 1;
  }
}
