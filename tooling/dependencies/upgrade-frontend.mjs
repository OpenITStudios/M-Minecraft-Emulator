#!/usr/bin/env node
// Reproducible, opt-in upgrade on Node 24/npm 11 with an accessible npm registry.
// No CI or Rust manifests are modified by this command.
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";
import { readFileSync } from "node:fs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const desktop = resolve(root, "apps/desktop");
const requested = ["vite@8.3.4", "@sveltejs/vite-plugin-svelte@7.3.1", "vitest@5.0.5"];
const args = process.argv.slice(2);
if (!args.includes("--apply")) {
  console.log("Dry-run: upgrade " + requested.join(", "));
  console.log("Requires Node 24, npm 11, registry access, and a clean Git worktree.");
  console.log("Run with --apply to migrate, regenerate the lockfile, and run focused checks.");
  process.exit(0);
}
if (args.some((arg) => arg !== "--apply")) {
  console.error("Only --apply is supported.");
  process.exit(2);
}
function run(command, argv, cwd = root) {
  const result = spawnSync(command, argv, { cwd, stdio: "inherit", shell: process.platform === "win32" });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
if (Number(process.versions.node.split(".")[0]) !== 24) {
  console.error("Upgrade requires Node 24.x to match package.json engines.");
  process.exit(2);
}
const npmVersion = spawnSync("npm", ["--version"], { encoding: "utf8", shell: process.platform === "win32" });
if (npmVersion.status !== 0 || Number(npmVersion.stdout.trim().split(".")[0]) !== 11) {
  console.error("Upgrade requires npm 11.x.");
  process.exit(2);
}
const status = spawnSync("git", ["status", "--porcelain", "--untracked-files=normal"], {
  cwd: root, encoding: "utf8", shell: process.platform === "win32"
});
if (status.status !== 0 || status.stdout.trim()) {
  console.error("Stop: Git worktree must be clean before modifying the lockfile.");
  process.exit(2);
}
const manifest = JSON.parse(readFileSync(resolve(desktop, "package.json"), "utf8"));
for (const name of ["vite", "@sveltejs/vite-plugin-svelte", "vitest"]) {
  if (!manifest.devDependencies?.[name]) throw Error("Missing expected direct dependency " + name);
}
run("npm", ["install", "--save-dev", "--save-exact", "--no-audit", "--no-fund", ...requested], desktop);
run("npm", ["ci", "--no-audit", "--no-fund"], desktop);
run("node", ["tooling/repository/check-dependency-lock.mjs"]);
run("npm", ["run", "typecheck"], desktop);
run("npm", ["run", "test"], desktop);
run("npm", ["run", "build:frontend"], desktop);
console.log("Frontend major upgrade and focused checks complete; inspect git diff and commit.");
