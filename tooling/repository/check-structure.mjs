import { existsSync, readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { dirname, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const rootFiles = new Set(['.editorconfig', '.gitattributes', '.gitignore', '.gitmodules',
  '.node-version', 'AGENTS.md', 'CONTEXT.md', 'CONTRIBUTING.md', 'DEV.cmd',
  'GITHUB_RULES.md', 'README.md', 'RUN.cmd', 'SECURITY.md', 'VERSION', 'package.json', 'toolchain.json']);
const roots = new Set(['.agents', 'apps', 'docs', 'engine', 'experiments', 'planning', 'tooling', 'workspace']);
// Upstream resources and tools are deliberate provenance inputs, including binaries.
const upstream = /^(?:engine\/isolation-owned\/|engine\/isolation-candidates\/)/;
export function checkTracked(paths) {
  const errors = [];
  for (const path of paths) {
    if (!path.includes('/')) {
      if (!rootFiles.has(path)) errors.push('Unreviewed root owner: ' + path);
    } else if (!roots.has(path.split('/')[0])) errors.push('Unreviewed directory owner: ' + path);
    if (path.startsWith('workspace/') && path !== 'workspace/README.md') errors.push('Tracked workspace output: ' + path);
    if (/^apps\/desktop\/src-tauri\/(?:icons|gen|binaries|package-resources)\//.test(path)) {
      errors.push('Tracked desktop staging output: ' + path);
    }
    if (!upstream.test(path) && /(?:^|\/)(?:node_modules|target|dist|coverage|test-results|\.svelte-check)\/|\.(?:log|tmp)$/.test(path)) {
      errors.push('Tracked generated output: ' + path);
    }
    if (/^\.github\/workflows\//.test(path)) errors.push('GitHub Actions must stay disabled: ' + path);
  }
  return errors;
}
export function checkLinks(root, paths) {
  const errors = [];
  for (const path of paths.filter(p => p.endsWith('.md') &&
    !upstream.test(p) && !p.startsWith('planning/history/'))) {
    const text = readFileSync(resolve(root, path), 'utf8');
    if (/lifecycle: HISTORICAL/.test(text)) continue;
    for (const match of text.matchAll(/!?\[[^\]\n]*\]\(([^)\s]+)\)/g)) {
      const target = match[1].replace(/^<|>$/g, '').split('#')[0];
      if (!target || /^[a-z]+:|^\//i.test(target)) continue;
      const destination = resolve(root, dirname(path), decodeURIComponent(target));
      const local = relative(root, destination);
      if (local === '..' || local.startsWith('..' + sep) || !existsSync(destination)) {
        errors.push('Invalid internal link: ' + path + ' -> ' + target);
      }
    }
  }
  return errors;
}
export function verifyStructure(root) {
  const paths = execFileSync('git', ['ls-files', '-z'], {cwd: root, encoding: 'utf8'}).split('\0').filter(Boolean);
  const errors = [...checkTracked(paths), ...checkLinks(root, paths)];
  for (const path of ['apps/desktop/package.json', 'apps/desktop/package-lock.json',
    'apps/desktop/src-tauri/Cargo.toml', 'apps/desktop/src-tauri/Cargo.lock',
    'engine/runtime-core/Cargo.toml', 'engine/runtime-core/Cargo.lock',
    'tooling/repository', 'tooling/dependencies', 'docs/system', 'planning/development.md']) {
    if (!existsSync(resolve(root, path))) errors.push('Missing canonical owner: ' + path);
  }
  const packageJson = JSON.parse(readFileSync(resolve(root, 'package.json'), 'utf8'));
  for (const command of Object.values(packageJson.scripts)) {
    for (const match of command.matchAll(/(?:node (?!-)|--prefix |--manifest-path[ =])([^\s&]+)/g)) {
      if (!existsSync(resolve(root, match[1]))) errors.push('Missing command path: ' + match[1]);
    }
  }
  assert.deepEqual(errors, [], errors.join('\n'));
  console.log('Repository structure: PASS (' + paths.length + ' tracked files; upstream resources preserved)');
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  verifyStructure(resolve(dirname(fileURLToPath(import.meta.url)), '../..'));
}
