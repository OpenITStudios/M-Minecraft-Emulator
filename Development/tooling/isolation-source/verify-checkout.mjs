import { spawnSync } from 'node:child_process';
import { lstatSync, realpathSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { sameGitDirectory } from './git-directory-identity.mjs';

const UPSTREAM = 'https://github.com/sandboxie-plus/Sandboxie.git';
const PIN = 'abd4c240031cfb46666daa74ca47ed4e48b561e4';
const TAG = 'v1.18.5';
const BRANCH = 'isolation/baseline-v1.18.5';

function requireFact(value, description) {
  if (!value) throw new Error('Isolation source checkout rejected: ' + description);
}

function samePath(a, b) {
  const left = resolve(a), right = resolve(b);
  return process.platform === 'win32' ? left.toLowerCase() === right.toLowerCase() : left === right;
}

function git(root, ...args) {
  const result = spawnSync('git', ['-C', root, ...args], {
    encoding: 'utf8', timeout: 15000, maxBuffer: 1024 * 1024, windowsHide: true,
  });
  requireFact(!result.error && result.status === 0,
    'Git inspection failed for ' + args[0]);
  return result.stdout.trim();
}

// Independent read-only verification before compiling native binaries. An
// existing directory is NOT assumed to be the bootstrap script's output.
export function verifyCheckout(path, { upstream = UPSTREAM, pin = PIN,
  tag = TAG, branch = BRANCH } = {}) {
  requireFact(typeof path === 'string' && path.trim(), 'checkout path missing');
  const root = resolve(path);
  let entry;
  try { entry = lstatSync(root); } catch { throw new Error('Isolation source checkout rejected: directory missing'); }
  requireFact(entry.isDirectory() && !entry.isSymbolicLink(), 'checkout must be a real directory');
  requireFact(samePath(realpathSync(root), root), 'checkout contains redirected parent path');
  requireFact(sameGitDirectory(root,git(root, 'rev-parse', '--show-toplevel')),
    'nested worktree or non-root path');
  requireFact(git(root, 'rev-parse', '--is-shallow-repository') === 'false',
    'shallow history');
  requireFact(git(root, 'rev-parse', 'HEAD') === pin, 'unpinned HEAD');
  requireFact(git(root, 'rev-parse', '--verify', `refs/tags/${tag}^{commit}`) === pin,
    'tag does not match pinned commit');
  requireFact(git(root, 'branch', '--show-current') === branch,
    'unexpected working branch');
  requireFact(git(root, 'config', '--local', '--get', 'remote.upstream.url') === upstream,
    'upstream source URL changed');
  requireFact(git(root, 'status', '--porcelain=v1', '--untracked-files=all') === '',
    'checkout has tracked or untracked changes');
  const count = Number(git(root, 'rev-list', '--count', 'HEAD'));
  requireFact(Number.isSafeInteger(count) && count > 0, 'no verifiable history');
  return {
    status: 'PINNED_CHECKOUT_VERIFIED', commit: pin, tag, branch,
    historyCount: count, shallow: false,
    sourceBuild: false, driverInstalled: false, minecraftTested: false,
  };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const args = process.argv.slice(2);
    requireFact(args.length === 2 && args[0] === '--source-root',
      'usage: node verify-checkout.mjs --source-root <pinned checkout>');
    process.stdout.write(JSON.stringify(verifyCheckout(args[1]), null, 2) + '\n');
  } catch (error) {
    process.stderr.write(error.message + '\n');
    process.exitCode = 1;
  }
}
