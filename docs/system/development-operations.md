---
id: document.system.development-operations
class: DOCUMENT
domain: system
role: GUIDE
authority: CANONICAL
lifecycle: ACTIVE
---

# Development Operations

## GitHub-first work

The regular ChatGPT developer path is **REMOTE_GITHUB**:

```text
pin Local HEAD
→ AGENTS.md / GITHUB_RULES.md
→ docs/system/implementation-map.md
→ canonical owner / consumers
→ smallest coherent repository change
→ exact-head static verification / evidence classification
→ commit to Local / STOP
```

A user-PC checkout, npm installation, `DEV.cmd`, VMware or Minecraft execution is not required for source-verifiable tasks. Actual compilations, package builds, Windows guest behavior and gameplay must not be claimed without executed evidence.

## Toolchain authority

Read `../../toolchain.json`. Developer/build versions are Node `24.22.0`, npm `12.2.0`, Rust `1.99.1` and portable PowerShell `7.6.7` for the Windows command wrapper. Desktop uses its only npm lockfile, `apps/desktop/package-lock.json`; Rust backend uses `engine/runtime-core/Cargo.lock`. The repository root intentionally has **no separate Node dependency graph**.

## Optional source execution

```bash
npm run setup
npm run verify:repository
npm run test
npm run check:desktop
npm run check:runtime
npm run test:runtime
npm run format:runtime
npm run verify:source
npm run dev
```

`verify:repository` is a structural policy/consumer check. Desktop `verify:source` runs Svelte typecheck, Vitest source tests, frontend build, Tauri Rust check and Rust desktop tests; `verify:source` at **root** also checks canonical runtime Rust and formatting. Neither automatically starts VMware or Minecraft.

`DEV.cmd bootstrap` is the single opt-in Windows x64 setup entrypoint: it downloads verified portable Node/PowerShell and checksum-verified official Rustup only to `workspace/`. Other `DEV.cmd` commands run the local PowerShell runtime, source `tooling/windows-toolchain/local-env.ps1`, refuse global Node/npm/Rust fallbacks, and use process-scoped caches, Cargo/Rustup homes and temp within `workspace/`. Git, Visual Studio C++/MSVC and WDK still require a system installation; game/driver/service locations are not rewritten. `DEV.cmd` delegates to `tooling/windows-toolchain/dev.ps1` for optional Windows developer tasks. The selected native pilot uses one fail-fast preflight before manual gameplay; the old `g0` packaging route remains separate:

```text
DEV.cmd bootstrap
DEV.cmd setup
DEV.cmd doctor
DEV.cmd preflight
DEV.cmd check
DEV.cmd test
DEV.cmd dev
DEV.cmd package
```

`doctor` checks developer toolchain only. `preflight` checks a clean committed `Local` SHA, installed tool versions and committed lockfiles, performs `npm run setup` and `npm run verify:source`, then confirms the checkout remains unchanged. It does **not** test Microsoft licensing, Sandboxie, Windows gameplay, VM state or installer preservation.

## Package and release boundary

`npm run package:windows` is **intentionally blocked** in the active native feasibility pilot: the old NSIS configuration still carries VMware resources and installed identity. Its retained source describes historical packaging only, not an installable native prototype. Do not bypass this release hold to obtain a local-test installer. A future approved native package needs separate identity, contents and preservation review. Packaging resources come from `engine/guest/windows/`, `tooling/acceptance/windows/` and `engine/distribution/`. Resource destination paths inside the installed application remain stable.

GitHub Actions on `Local` now operate in **full CI** mode by explicit user authorization. The `verify.yml` source accepts `push` events scoped to branch `Local` and still supports three manual `workflow_dispatch` scopes. Every `Local` commit automatically runs all **12 canonical source verification stages on a Windows runner**, including Svelte/Vitest/frontend/Rust tests. The final aggregate gate fails the run if any source stage fails or is skipped. When all twelve stages succeed, the same Windows job also performs `cargo build --locked --target x86_64-pc-windows-msvc` (compile only, no installer). If any source stage fails, automatic compilation does not run. Superseded push runs may be cancelled; manual operator runs are not cancelled by this policy.

**Manual scopes remain available:** `quick` (small Windows Node tests), `full-source` (the same twelve source stages, without the automatic final compile), and `windows-build` (Windows frontend + Tauri compilation only). The previously approved default-branch `main` file is still a *registration-only* `workflow_dispatch` shell with a permanently skipped job and no checkout. Real verification remains restricted to `Local`; `main` is not merged, deployed or tested.

**Optional CI debug artifact:** after automatic `Local` Full Source and the Windows `cargo build --locked` step are successful, the same Windows job stages exactly one unsigned debug EXE, an exact-SHA/SHA-256 manifest and a copy of [CI Debug Candidate](../validation/ci-debug-candidate.md), then uploads them as a 30-day GitHub Actions artifact. Earlier artifacts keep their original expiry. The staging step also checks the copied PE32+ AMD64 Windows GUI header before upload; this is binary-structure evidence only, not proof of execution or signature trust. This is evidence and an optional UI smoke candidate, **not** an installer, signed package, deployed release or proof of game runtime. No Minecraft, account, Sandboxie, driver or legacy VMware payload is copied into the artifact. Manual CI scopes do not upload the artifact.

**Safety boundary:** the historical `package-smoke.yml` remains manual-only and is **not** part of automatic CI. Neither the source workflow nor automatic Tauri build may install/sign/start kernel drivers, deploy installers, launch Minecraft Education, authenticate accounts, mutate Sandboxie data or claim real Windows gameplay. Standard GitHub-hosted Windows builds do not substitute for target machine acceptance.

### Interpreting CI results

Read the exact SHA, event, source-stage summary, and final job conclusion. A green source stage with `continue-on-error` is diagnostic only; the final aggregate gate must succeed before Full Source counts as PASS. A successful compile job still does not prove a functioning installed app, gameplay or four-client stability. Do not combine PASS results across different commits. The manual workflow UI is optional after this change; `Local` pushes trigger verification automatically.

## Single guest renderer compatibility pilot

Before registering/finalizing a production Base, read [Client Feasibility](../validation/client-feasibility.md). The Windows guest collector is in `../../tooling/acceptance/windows/collect-client-feasibility.ps1`; it inspects a real interactive guest's video adapter/OpenGL context and captures separately attributed manual Minecraft menu, scene and input observations. The pure reviewer (`npm run feasibility:review -- <path-to-report.json>`) and its `npm run test:feasibility` regression tests do **not** launch Minecraft or assert official VM-client support.

Source-only changes may be completed on GitHub `Local`, but guest graphics and gameplay remain `NOT_EXECUTED` without real machine evidence. Avoid asking for local PC access to close GitHub-only work. No changes to `main`.

## Real-machine evidence

Read [Acceptance](../validation/acceptance.md) and [Failure Matrix](../validation/four-client-failure-matrix.md). The privacy-bounded Windows evidence collector lives in `../../tooling/acceptance/windows/collect-acceptance.ps1`. Running the collector does not prove Microsoft sign-in, 3D graphics or multiplayer unless separately observed.

No source edit should be left unfinished merely because real-machine verification is unavailable; mark the exact corresponding claim `NOT_EXECUTED`.

## Before localtest: one-root command protection

Use `DEV.cmd bootstrap` once, then `DEV.cmd doctor`, then
`DEV.cmd preflight`, then `DEV.cmd dev`. The `setup`, `check`,
`test`, and `dev` routes each revalidate pinned local tool versions and
lockfile presence before proceeding; they do not assume that the operator
remembered to run `doctor`. `bootstrap` uses a process-only local
TEMP/TMP before network downloads, and the Rustup sidecar has bounded plain-file
admission. This is a source/CI contract, not a completed Windows installed-PC
smoke test. Debug artifact output is a separate compile-only candidate and does
not install the system dependencies.
