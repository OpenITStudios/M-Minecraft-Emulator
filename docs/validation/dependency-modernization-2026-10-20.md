# Emulator dependency modernization (2026-10-20)

Target branch: `Local`. Source of comparison: `OpenITStudios/M-LazyDesigner` at `e9e40749e08f4fea71b7a64438e3b3d1741b0d9e`.

## Batch 1: lockfile-synchronized toolchain refresh

- `@tauri-apps/api`: 2.11.1 -> 2.12.3.
- `@tauri-apps/cli`: 2.11.4 -> 2.12.3, including all 11 platform-specific optional packages.
- `svelte-check`: 4.7.6 -> 4.7.7.
- `typescript`: 5.9.3 -> 6.0.3.
- Node.js 24.x and npm 11.19.0 remain the declared environment.
- Kept existing Vite 7, vite-plugin-svelte 6 and Vitest 3 together to avoid incompatible peer dependencies.
- Did **not** modify Rust dependency ranges, `Cargo.lock`, Sandboxie snapshots, the Windows isolation engine, or CI policy.

Metadata (including integrity hashes) for the replacement lock entries is taken from the already-pinned LazyDesigner desktop npm lockfile; it is not guessed or fabricated.

## Required focused acceptance before declaring Batch 1 complete

From a machine with Node 24 and npm 11.19.0:

```sh
npm --prefix apps/desktop ci --no-audit --no-fund
npm --prefix apps/desktop run typecheck
npm --prefix apps/desktop run test
npm --prefix apps/desktop run build:frontend
```

Windows/Tauri-focused acceptance (requires the project Rust and Windows toolchains):

```sh
npm --prefix apps/desktop run check:rust
npm --prefix apps/desktop run test:rust
```

**Execution status:** manifest/lock synchronization checked structurally; npm installation, typecheck, frontend build, Vitest and Rust acceptance were NOT run in the connector environment. Do not claim passing CI/build from this commit.

## Next upgrade batch: major-version migration

1. Upgrade `vite` 7 -> 8 and `@sveltejs/vite-plugin-svelte` 6 -> 7 **together** with Vitest 3 -> compatible current Vitest 5; run focused tests and regenerate the npm lock using npm, not manual version edits. Validate Vite 8/Rolldown changes and any test-runner API differences.
2. Evaluate `@typescript/native` 7 for `svelte-check --tsgo` after baseline `typescript` 6 is verified. Retain TS6 fallback if the Go-native checker does not support a project feature.
3. Upgrade Tauri Rust and `tauri-build` together and regenerate both `apps/desktop/src-tauri/Cargo.lock` and `engine/runtime-core/Cargo.lock` with Cargo, respecting Rust MSRV.
4. Audit pinned `ureq=2.9.7`, `url=2.5.0`, `native-tls=0.2.12`, `getrandom=0.2.17`, `sha2=0.10.8` separately. `ureq` 2 -> 3 is an API rewrite requiring source migration; do not bump manifest alone.
5. Re-evaluate copied Sandboxie 1.18.6 only as a separately versioned external codebase, not as a general dependency update.
6. Keep repository GitHub Actions disabled per existing commit/policy; use affected/local checks, not a blanket full-suite run.

Update this document as each batch is verified; keep untested work explicitly labeled.

## Batch 2: safe npm patch alignment (2026-10-20)

- Svelte 5.57.0 -> 5.57.2 (version and verified-integrity lock entry from LazyDesigner).
- @types/node 24.13.4 -> 24.19.2 (version and verified-integrity lock entry from LazyDesigner).
- Added `node tooling/repository/check-dependency-lock.mjs` and `npm run check:dependencies` for install-free manifest consistency, synchronized Tauri platform packages and the selected Vite/Vitest/plugin major-version contract. This check deliberately rejects an isolated Vite 8 bump until Vitest is updated.
- Note: this metadata-only gate is not a substitute for `npm ci`, TypeScript checking, Vitest, Windows Rust compilation, or packaging. Those executions are still outstanding.

### Release gate for the major modernization

**Do not edit npm lockfiles by hand for Vite 8/Vitest 5.** On Node 24 with npm 11.19.0 use a clean checkout and perform a coordinated install:

```sh
cd apps/desktop
npm install --save-dev --save-exact vite@8.3.4 @sveltejs/vite-plugin-svelte@7.3.1 vitest@5.0.5
npm ci --no-audit --no-fund
node ../../tooling/repository/check-dependency-lock.mjs
npm run typecheck
npm run test
npm run build:frontend
```

Inspect migration warnings and update affected Vitest tests or Vite options before committing the npm-generated lockfile. Maintain the existing CI shutdown. Upgrade Cargo and its lockfiles only from a machine with installed Rust/Cargo; verify Windows-specific builds separately.
