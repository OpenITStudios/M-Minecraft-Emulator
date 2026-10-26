#!/usr/bin/env node
// Read-only registry/advisory audit; continues across independent package graphs.
import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const npmCli = process.env.npm_execpath || resolve(dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
const commands = [
  ['npm', ['ls', '--prefix', 'apps/desktop', '--all']],
  ['npm', ['outdated', '--prefix', 'apps/desktop', '--all', '--json'], true],
  ['npm', ['audit', '--prefix', 'apps/desktop']],
  ...['engine/runtime-core', 'apps/desktop/src-tauri'].flatMap(path => [
    ['cargo', ['tree', '--locked', '--duplicates', '--manifest-path', `${path}/Cargo.toml`]],
    ['cargo', ['audit', '--file', `${path}/Cargo.lock`]],
  ]),
];
export function auditDependencies(run = spawnSync) {
let failed = false;
for (const [command, args, informational] of commands) {
  console.log('\n' + command + ' ' + args.join(' '));
  const result = run(command === 'npm' ? process.execPath : command,
    command === 'npm' ? [npmCli, ...args] : args, {cwd: root, encoding: 'utf8',
      stdio: informational ? 'pipe' : 'inherit', windowsHide: true});
  if (result.error) console.error(result.error.message);
  let updateFindings = false;
  if (informational) {
    if (result.stdout) process.stdout.write(result.stdout);
    if (result.stderr) process.stderr.write(result.stderr);
    try {
      const data = JSON.parse(result.stdout);
      updateFindings = data && !Array.isArray(data) && !data.error && Object.keys(data).length > 0;
    } catch { /* Malformed/missing metadata is a failed query, not update findings. */ }
  }
  // Only a valid outdated report may use exit 1 without failing the audit.
  if (result.error || (result.status !== 0 && !(informational && result.status === 1 && updateFindings))) failed = true;
}
return failed ? 1 : 0;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exitCode = auditDependencies();
}
