#!/usr/bin/env node
// Focused, install-free dependency metadata checks. Does not replace npm ci/build/test.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import assert from "node:assert/strict";

const desktop = resolve(dirname(fileURLToPath(import.meta.url)), "../../apps/desktop");
export function checkDependencyLock(manifest, lock) {
const root = lock.packages?.[""];
assert.equal(manifest.version, root?.version, "npm lock root version mismatch");
assert.equal(lock.lockfileVersion, 3, "npm lockfile v3 required");
assert.equal(manifest.name, root?.name, "npm lock root name mismatch");
assert.deepEqual(root?.engines, manifest.engines, "npm lock engine requirements mismatch");
for (const area of ["dependencies", "devDependencies"]) {
  assert.deepEqual(root?.[area] ?? {}, manifest[area] ?? {}, area + ' lock ownership mismatch');
  for (const [name, declared] of Object.entries(manifest[area] ?? {})) {
    assert.equal(root?.[area]?.[name], declared, `root lock mismatch for ${name}`);
    const installed = lock.packages?.[`node_modules/${name}`];
    assert.ok(installed?.version, `missing installed package: ${name}`);
    assert.ok(installed.resolved && installed.integrity, `missing provenance for ${name}`);
    // Exact pins can be asserted without an external semver dependency.
    const exact = declared.startsWith("npm:") ? declared.slice(declared.lastIndexOf("@") + 1) : declared;
    if (/^\d+\.\d+\.\d+$/.test(exact)) {
      assert.equal(installed.version, exact, `installed version mismatch for ${name}`);
    }
  }
}
const installed = (name) => lock.packages?.[`node_modules/${name}`];
const major = (name) => Number(installed(name)?.version?.split(".")[0]);
// API and CLI are independently released; their supported Tauri major must agree.
assert.equal(major('@tauri-apps/api'), major('@tauri-apps/cli'), 'Tauri JS major mismatch');
// Peer/engine compatibility is checked by npm ci and npm ls, not a version matrix.
for (const [name, expected] of Object.entries(installed("@tauri-apps/cli").optionalDependencies ?? {})) {
  assert.equal(installed(name)?.version, expected, `Tauri platform package mismatch: ${name}`);
  assert.ok(installed(name)?.integrity, `missing integrity: ${name}`);
}
// NPM type packages can introduce new transitive constraints without changing
// the directly pinned versions. In particular Node 24 types require newer undici.
const undiciConstraint = installed("@types/node")?.dependencies?.["undici-types"];
if (undiciConstraint) {
  const actual = installed("undici-types");
  assert.ok(actual?.version, "missing undici-types required by @types/node");
  const match = /^>=([0-9]+)\.([0-9]+)\.([0-9]+) <([0-9]+)\.([0-9]+)\.([0-9]+)$/.exec(undiciConstraint);
  // Reject an unfamiliar range instead of silently skipping a safety check.
  assert.ok(match, `review new @types/node undici range: ${undiciConstraint}`);
  const tuple = (x) => x.split(".").map(Number);
  const compare = (a, b) => {
    for (let i = 0; i < 3; i++) {
      if (a[i] !== b[i]) return Math.sign(a[i] - b[i]);
    }
    return 0;
  };
  const lower = match.slice(1, 4).map(Number);
  const upper = match.slice(4, 7).map(Number);
  const version = tuple(actual.version);
  assert.ok(compare(version, lower) >= 0 && compare(version, upper) < 0,
    `undici-types ${actual.version} does not satisfy ${undiciConstraint}`);
}

const svelteCheckPeer = installed("svelte-check").peerDependencies?.typescript ?? "";
if (!svelteCheckPeer.includes("^7.0.0")) {
  assert.ok(major("typescript") === 5 || major("typescript") === 6,
    "Svelte Check does not advertise TypeScript 7 support");
}
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  checkDependencyLock(JSON.parse(readFileSync(resolve(desktop, 'package.json'), 'utf8')),
    JSON.parse(readFileSync(resolve(desktop, 'package-lock.json'), 'utf8')));
  console.log('Dependency lock metadata: PASS (npm ci / npm ls validate peers and engines)');
}
