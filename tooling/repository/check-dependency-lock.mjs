#!/usr/bin/env node
// Focused, install-free dependency metadata checks. Does not replace npm ci/build/test.
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import assert from "node:assert/strict";

const desktop = resolve(dirname(fileURLToPath(import.meta.url)), "../../apps/desktop");
const manifest = JSON.parse(readFileSync(resolve(desktop, "package.json"), "utf8"));
const lock = JSON.parse(readFileSync(resolve(desktop, "package-lock.json"), "utf8"));
const root = lock.packages?.[""];
assert.equal(lock.lockfileVersion, 3, "npm lockfile v3 required");
assert.equal(manifest.name, root?.name, "npm lock root name mismatch");
for (const area of ["dependencies", "devDependencies"]) {
  for (const [name, declared] of Object.entries(manifest[area] ?? {})) {
    assert.equal(root?.[area]?.[name], declared, `root lock mismatch for ${name}`);
    const installed = lock.packages?.[`node_modules/${name}`];
    assert.ok(installed?.version, `missing installed package: ${name}`);
    assert.ok(installed.resolved && installed.integrity, `missing provenance for ${name}`);
    // Exact pins can be asserted without an external semver dependency.
    if (/^\\d+\\.\\d+\\.\\d+$/.test(declared)) {
      assert.equal(installed.version, declared, `installed version mismatch for ${name}`);
    }
  }
}
const installed = (name) => lock.packages?.[`node_modules/${name}`];
const major = (name) => Number(installed(name)?.version?.split(".")[0]);
const vite = major("vite"), plugin = major("@sveltejs/vite-plugin-svelte"), vitest = major("vitest");
assert.ok(
  (vite === 7 && plugin === 6 && vitest === 3) ||
  (vite === 8 && plugin === 7 && vitest >= 5),
  `unsupported Vite / Svelte Plugin / Vitest combination: ${vite}/${plugin}/${vitest}`
);
assert.equal(installed("@tauri-apps/api").version, installed("@tauri-apps/cli").version,
  "Tauri JS API/CLI versions must remain aligned");
for (const [name, expected] of Object.entries(installed("@tauri-apps/cli").optionalDependencies ?? {})) {
  assert.equal(installed(name)?.version, expected, `Tauri platform package mismatch: ${name}`);
  assert.ok(installed(name)?.integrity, `missing integrity: ${name}`);
}
const svelteCheckPeer = installed("svelte-check").peerDependencies?.typescript ?? "";
if (!svelteCheckPeer.includes("^7.0.0")) {
  assert.ok(major("typescript") === 5 || major("typescript") === 6,
    "Svelte Check does not advertise TypeScript 7 support");
}
console.log("Dependency lock metadata and selected toolchain compatibility: PASS");
