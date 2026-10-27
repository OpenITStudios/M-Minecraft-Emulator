import { existsSync, lstatSync, readFileSync, readdirSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { dirname, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import assert from 'node:assert/strict';

const rootFiles = new Set(['.editorconfig', '.gitattributes', '.gitignore',
  'AGENTS.md', 'CONTEXT.md', 'CONTRIBUTING.md', 'DEV.cmd',
  'GITHUB_RULES.md', 'README.md', 'RUN.cmd', 'SECURITY.md']);
const roots = new Set(['.agents', 'Development', 'Archive', 'docs', 'planning', 'workspace']);
const development = new Set(['apps', 'engine', 'tooling', 'README.md', 'package.json', 'toolchain.json', 'VERSION', '.node-version']);
// Vendor forks and upstream gitlinks are retired from the active source tree.
const retired = /^(?:\.gitmodules$|Development\/engine\/isolation-(?:owned|candidates|upstream)(?:\/|$))/;
export function checkTracked(paths) {
  const errors = [];
  for (const path of paths) {
    if (retired.test(path)) errors.push('Retired Sandboxie source: ' + path);
    if (!path.includes('/')) {
      if (!rootFiles.has(path)) errors.push('Unreviewed root owner: ' + path);
    } else if (!roots.has(path.split('/')[0])) errors.push('Unreviewed directory owner: ' + path);
    if (path.startsWith('Development/') && !development.has(path.split('/')[1])) errors.push('Unreviewed development owner: ' + path);
    if (path.startsWith('Archive/') && !['README.md', 'experiments'].includes(path.split('/')[1])) errors.push('Unreviewed archive owner: ' + path);
    if (path.startsWith('workspace/') && path !== 'workspace/README.md') errors.push('Tracked workspace output: ' + path);
    if (/^Development\/apps\/desktop\/src-tauri\/(?:icons|gen|binaries|package-resources)\//.test(path)) {
      errors.push('Tracked desktop staging output: ' + path);
    }
    if (/(?:^|\/)(?:node_modules|target|dist|coverage|test-results|\.svelte-check)\/|\.(?:log|tmp)$/.test(path)) {
      errors.push('Tracked generated output: ' + path);
    }
    if (/^\.github\/workflows\//.test(path)) errors.push('GitHub Actions must stay disabled: ' + path);
  }
  return errors;
}
export function checkPhysical(root) {
  const errors = [];
  for (const entry of readdirSync(root, {withFileTypes:true})) {
    if (entry.name === '.git') continue; // A linked worktree may use a .git file.
    if (entry.name === 'Lazy Tester.exe') {
      const info = lstatSync(resolve(root,entry.name));
      if (!info.isFile() || info.isSymbolicLink()) errors.push('Expected ordinary root launcher: ' + entry.name);
      continue; // Generated, ignored entry point; never source control a local executable.
    }
    if (['App', 'Data', 'Logs', ...roots].includes(entry.name)) {
      const info = lstatSync(resolve(root, entry.name));
      if (!info.isDirectory() || info.isSymbolicLink()) errors.push('Expected ordinary directory: ' + entry.name);
    } else if (!rootFiles.has(entry.name) || !entry.isFile()) errors.push('Unreviewed physical root entry: ' + entry.name);
  }
  return errors;
}
export function checkLinks(root, paths) {
  const errors = [];
  for (const path of paths.filter(p => p.endsWith('.md') &&
    !p.startsWith('planning/history/'))) {
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
  const errors = [...checkTracked(paths), ...checkLinks(root, paths), ...checkPhysical(root)];
  for (const path of ['Development/apps/desktop/package.json', 'Development/apps/desktop/package-lock.json',
    'Development/apps/desktop/src-tauri/Cargo.toml', 'Development/apps/desktop/src-tauri/Cargo.lock',
    'Development/engine/runtime-core/Cargo.toml', 'Development/engine/runtime-core/Cargo.lock',
    'Development/tooling/repository', 'Development/tooling/dependencies', 'docs/system', 'planning/development.md']) {
    if (!existsSync(resolve(root, path))) errors.push('Missing canonical owner: ' + path);
  }
  const packageJson = JSON.parse(readFileSync(resolve(root, 'Development/package.json'), 'utf8'));
  for (const command of Object.values(packageJson.scripts)) {
    for (const match of command.matchAll(/(?:node (?!-)|--prefix |--manifest-path[ =])([^\s&]+)/g)) {
      if (!existsSync(resolve(root, 'Development', match[1]))) errors.push('Missing command path: ' + match[1]);
    }
  }
  assert.deepEqual(errors, [], errors.join('\n'));
  console.log('Repository structure: PASS (' + paths.length + ' tracked files; retired vendor source denied)');
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  verifyStructure(resolve(dirname(fileURLToPath(import.meta.url)), '../../..'));
}
