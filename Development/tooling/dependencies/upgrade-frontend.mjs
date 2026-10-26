#!/usr/bin/env node
// Resolve the coupled frontend group from official stable registry metadata.
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const desktop = resolve(root, 'apps/desktop');
const npmCli = process.env.npm_execpath || resolve(dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
const toolchain = JSON.parse(readFileSync(resolve(root, 'toolchain.json'), 'utf8'));
const args = process.argv.slice(2);
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url) && (args.some(arg => arg !== '--apply') || args.length > 1)) throw Error('Only --apply is supported');
const apply = args.includes('--apply');
const run = (command, argv, cwd = root, capture = false) => {
  const result = spawnSync(command === 'npm' ? process.execPath : command, command === 'npm' ? [npmCli, ...argv] : argv, {
    cwd, encoding: 'utf8', stdio: capture ? 'pipe' : 'inherit',
    windowsHide: true,
  });
  if (result.error || result.status !== 0) throw Error(result.error?.message || `${command} failed (${result.status}): ${result.stderr ?? ''}`);
  return result.stdout?.trim();
};
if (apply) {
  if (process.versions.node !== toolchain.node.version ||
      run('npm', ['--version'], root, true) !== toolchain.npm.version) {
    throw Error('Use the exact Node/npm versions in toolchain.json');
  }
  if (run('git', ['branch', '--show-current'], root, true) !== 'Local' ||
      run('git', ['status', '--porcelain', '--untracked-files=normal'], root, true)) {
    throw Error('Apply requires a clean Local checkout');
  }
}
export async function stableRelease(name, fetchMetadata = fetch) {
  const response = await fetchMetadata('https://registry.npmjs.org/' + encodeURIComponent(name) + '/latest');
  if (!response.ok) throw Error('Registry metadata failed: ' + name + ' (' + response.status + ')');
  const metadata = await response.json();
  if (!/^\d+\.\d+\.\d+$/.test(metadata.version)) throw Error('Stable release required: ' + name);
  console.log(name, metadata.version, JSON.stringify({engines: metadata.engines, peers: metadata.peerDependencies}));
  return name + '@' + metadata.version;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
const requested = await Promise.all(['vite', '@sveltejs/vite-plugin-svelte', 'vitest'].map(name => stableRelease(name)));
if (!apply) {
  console.log('Dry-run only. --apply resolves this group with npm and checks the resulting application.');
} else {
  run('npm', ['install', '--engine-strict', '--save-dev', '--save-exact', '--no-audit', '--no-fund', ...requested], desktop);
  run('npm', ['ci', '--engine-strict', '--no-audit', '--no-fund'], desktop);
  run(process.execPath, ['tooling/repository/check-dependency-lock.mjs']);
  run('npm', ['ls', '--all'], desktop);
  for (const command of ['typecheck', 'test', 'build:frontend']) run('npm', ['run', command], desktop);
  console.log('Focused checks complete. Review the diff and security audit before committing.');
}

}
