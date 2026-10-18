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

Read `../../toolchain.json`. Developer/build versions are Node `24.21.0`, npm `11.19.0`, Rust `1.98.1` and PowerShell 7 for the optional Windows command wrapper. Desktop uses its only npm lockfile, `apps/desktop/package-lock.json`; Rust backend uses `engine/runtime-core/Cargo.lock`. The repository root intentionally has **no separate Node dependency graph**.

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

`DEV.cmd` delegates to `tooling/windows-toolchain/dev.ps1` for optional Windows developer tasks. The selected native pilot uses one fail-fast preflight before manual gameplay; the old `g0` packaging route remains separate:

```text
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

GitHub workflow files in `.github/workflows/` are `workflow_dispatch` only and may be invoked solely when appropriate. They are evidence tools, not a parallel source of readiness state. No push-based CI, unattended release or automatic promotion to `main`.

**Approved manual registration (18 October 2026):** default branch `main` contains only the registration version of `.github/workflows/verify.yml`, with one permanently skipped job and no checkout. The real Quick / Full Source / Windows Build jobs are only in `Local`, each guarded by `github.ref == 'refs/heads/Local'`. The default branch cannot be used to run tests. To test: Actions → Emulator Source Verification (manual) → Run workflow → select **Local** → choose `quick` (default), `full-source` or `windows-build` → Run workflow. No job is started by commits.

The Local source-verification workflow has three explicitly selected manual scopes: `quick` (Windows, lightweight Node/source tests without dependency installation; default), `full-source` (complete Node/Svelte/Vitest/Rust checks on Windows), and `windows-build` (Windows frontend and Tauri Rust compilation only, without installer or kernel-driver activation). Only one scope is scheduled for a manual run. The separate historical `package-smoke.yml` remains **legacy VMware packaging** and is not an approved native pilot command; do not use it as a substitute for `windows-build`. These source definitions are not proof of executed tests.

**Workflow activation:** GitHub requires a manual workflow definition on the default branch. The user authorized exactly one registry file on `main`; its always-skipped job does not check out or build the app. Source and real jobs are loaded from the selected `Local` ref. Only explicit `workflow_dispatch` selections run CI, never commit hooks. This exception does not authorize a release, a merge to `main`, a driver install, or the legacy NSIS `package-smoke.yml` workflow.

### Operator proof and branch safety

**Full Source diagnostics (18 October 2026):** all twelve commands from the canonical root `verify:source` script now run as independent stages in the **same manually selected Windows Full Source job**. Individual stages use `continue-on-error` only to collect later diagnostics; an always-run final gate explicitly fails the overall job if *any* stage is failed or skipped and records each stage outcome in the summary. This prevents early isolation-source failures from hiding Svelte/Vitest/Rust errors, without granting false source PASS. Each job remains manual-only on `Local`; installer/driver/account operations stay absent. This diagnostic mode can consume more Windows-runner time per manually requested Full Source run, but avoids repeated entire runs to uncover the next failure. Use the final aggregate gate, not individual continued-stage checkmarks, as the actual acceptance outcome.


**Quick run #3 false-positive (18 October 2026):** the GitHub Actions job was marked SUCCESS but `npm run verify:repository` actually failed with `Active native/isolated pilot may not call legacy VM lifecycle commands`. The verifier expected the removed `<h1>Instances</h1>` label, while the mounted UI correctly has `<h1>Virtual Manager</h1>` plus `<h2>Minecraft sessions</h2>`. The five subsequent test commands passed a total of 62 tests, but a multi-command Windows PowerShell step continued after the first native command failed. Quick now uses **one GitHub Actions step for each command** to make failures fail the job. Windows Build icon and frontend commands are also separate steps. This is a CI/verifier correction, not proof of full-source readiness. Rerun Quick manually at the new `Local` SHA; do not rerun the old revision.


The first manually triggered Quick run (#2, `c8bcbc4`) failed before test execution because `actions/setup-node@v7` automatically enabled npm caching and searched for a nonexistent root `package-lock.json`. The lockfile is intentionally at `apps/desktop/package-lock.json`. The Windows Quick job disables that unnecessary automatic cache via `package-manager-cache: false`; the two larger jobs retain their explicit scoped npm cache paths. Quick's summary uses PowerShell syntax. No new source-test outcome is claimed until the user reruns it manually. GitHub's Windows runner is not an equivalent of interactive Windows 11 with Minecraft installed.

Choose branch `Local` **every time**. GitHub's dropdown may initially display `main`, whose registration-only job is skipped; such an outcome is not PASS. Verify the workflow run head SHA, selected scope and log exit status. Manual CI results prove source/compilation only, not installed Windows behavior, Microsoft authentication, Minecraft gameplay or four-client stability.

## Single guest renderer compatibility pilot

Before registering/finalizing a production Base, read [Client Feasibility](../validation/client-feasibility.md). The Windows guest collector is in `../../tooling/acceptance/windows/collect-client-feasibility.ps1`; it inspects a real interactive guest's video adapter/OpenGL context and captures separately attributed manual Minecraft menu, scene and input observations. The pure reviewer (`npm run feasibility:review -- <path-to-report.json>`) and its `npm run test:feasibility` regression tests do **not** launch Minecraft or assert official VM-client support.

Source-only changes may be completed on GitHub `Local`, but guest graphics and gameplay remain `NOT_EXECUTED` without real machine evidence. Avoid asking for local PC access to close GitHub-only work. No changes to `main`.

## Real-machine evidence

Read [Acceptance](../validation/acceptance.md) and [Failure Matrix](../validation/four-client-failure-matrix.md). The privacy-bounded Windows evidence collector lives in `../../tooling/acceptance/windows/collect-acceptance.ps1`. Running the collector does not prove Microsoft sign-in, 3D graphics or multiplayer unless separately observed.

No source edit should be left unfinished merely because real-machine verification is unavailable; mark the exact corresponding claim `NOT_EXECUTED`.
