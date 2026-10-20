#!/usr/bin/env node
// Read-only registry/advisory audit; continues across independent package graphs.
import { spawnSync } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const npmCli = process.env.npm_execpath || resolve(dirname(process.execPath), 'node_modules/npm/bin/npm-cli.js');
const commands = [
  ['npm', ['ls', '--prefix', 'apps/desktop', '--all']],
  ['npm', ['outdated', '--prefix', 'apps/desktop', '--all'], true],
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
    command === 'npm' ? [npmCli, ...args] : args, {cwd: root, stdio: 'inherit', windowsHide: true});
  if (result.error) console.error(result.error.message);
  // npm outdated uses 1 for a successful query containing update candidates.
  if (result.error || (result.status !== 0 && !(informational && result.status === 1))) failed = true;
}
return failed ? 1 : 0;
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  process.exitCode = auditDependencies();
}
