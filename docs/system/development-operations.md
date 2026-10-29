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

Read [toolchain.json](../../Development/toolchain.json) for canonical Node/npm/Rust/PowerShell pins. Desktop owns `Development/apps/desktop/package-lock.json`; desktop Tauri and runtime-core own independent Cargo locks. The root intentionally has **no separate Node dependency graph**.

## Optional source execution

```bash
npm --prefix Development run setup
npm --prefix Development run verify:repository
npm --prefix Development run test
npm --prefix Development run check:desktop
npm --prefix Development run check:frontend
npm --prefix Development run check:rust
npm --prefix Development run check:runtime
npm --prefix Development run test:runtime
npm --prefix Development run format:runtime
npm --prefix Development run verify:source
npm --prefix Development run dev
```

`verify:repository` is a structural policy/consumer check. Desktop `verify:source` runs Svelte typecheck, Vitest source tests, frontend build, Tauri Rust check and Rust desktop tests; `verify:source` at **root** also checks canonical runtime Rust and formatting. Neither automatically starts VMware or Minecraft.

`DEV.cmd bootstrap` is the single opt-in Windows x64 setup entrypoint: it downloads verified portable Node/PowerShell and checksum-verified official Rustup only to `workspace/`. Other `DEV.cmd` commands run the local PowerShell runtime, source `Development/tooling/windows-toolchain/local-env.ps1`, refuse global Node/npm/Rust fallbacks, and use process-scoped caches, Cargo/Rustup homes and temp within `workspace/`. Git, Visual Studio C++/MSVC and WDK still require a system installation; game/driver/service locations are not rewritten. `DEV.cmd` delegates to `Development/tooling/windows-toolchain/dev.ps1` for optional Windows developer tasks. The selected native pilot uses one fail-fast preflight before manual gameplay; the old `g0` packaging route remains separate:

```text
DEV.cmd bootstrap
DEV.cmd setup
DEV.cmd doctor
DEV.cmd preflight
DEV.cmd check
DEV.cmd check:frontend
DEV.cmd check:rust
DEV.cmd bench:frontend
DEV.cmd bench:rust
DEV.cmd test
DEV.cmd dev
DEV.cmd package
```

`doctor` checks developer toolchain only. `preflight` checks a clean committed `Local` SHA, installed tool versions and committed lockfiles, performs `npm --prefix Development run setup` and `npm --prefix Development run verify:source`, then confirms the checkout remains unchanged. It does **not** test Microsoft licensing, Sandboxie, Windows gameplay, VM state or installer preservation.

## Scoped developer checks and timing

Daily UI/TypeScript changes can use `DEV.cmd check:frontend`, which delegates
to the existing Svelte typecheck and Vitest suites. It **does not** run Rust
checks, Tauri, Vite release build, installer packaging or Minecraft. The
original full `DEV.cmd check` and `DEV.cmd preflight` are unchanged and
still required when claiming a complete source checkpoint.

Rust changes can use `DEV.cmd check:rust`; it delegates to the
desktop `check:rust:target` and standalone `runtime-core` Cargo check
using the explicit Windows MSVC target `x86_64-pc-windows-msvc`, pinned
lockfiles and the existing `CARGO_TARGET_DIR`. This aligns the
**target output layout** with the existing `tauri dev --target` command.
It does not guarantee artifact reuse across different feature sets,
Cargo profiles or build-script configurations and can require a one-time
new compilation. The quick route deliberately skips `--all-targets` and
Rust tests. The original full `verify:source` commands remain unchanged.
This check is **not** a Windows game test, DLL build or packaging approval.
No native Tauri feature/config policy is bypassed.

Manual opt-in `DEV.cmd bench:frontend` or `DEV.cmd bench:rust` runs the
corresponding scoped check twice on the **same cache**, after verifying the
pinned portable toolchain and clean `Local` SHA. Stopwatch durations for
pass 1 and pass 2 are printed to the console only; a failed pass aborts,
and source is checked again after each pass. Neither pass is a proven
**cold** or **warm** build: there is no clean, new target directory,
cache deletion, log collection, runner, automation, installer or Minecraft
launch. The second pass may have greater reuse but a faster time is not
guaranteed. Actual cold-build and disk comparisons remain separate
opt-in experiments requiring sufficient disk and new evidence.

Existing `DEV.cmd dev` retains Tauri's frontend hot reload / Rust rebuild
behavior, the reviewed compatibility DLL staging and native identity. The
new scoped routes do not modify game/session/process authority.

## Package and release boundary

The current product is **Lazy Tester**, with Windows-native independently
managed Minecraft Education profiles. VMware/Sandboxie runtime providers and
the legacy VMware package path are retired and remain blocked. The
`npm --prefix Development run package:windows` command is the dedicated
native Lazy Tester candidate packager; it requires clean committed `Local`
source, reviewed DLL payload, pinned toolchain and its original safety
gates. Its export is **unsigned, unaccepted and not published**. The
`--export-only` option leaves the local `App/` candidate unchanged;
the normal local candidate refresh retains protected `Data/` and
archives the previous `App/` under the protected private backup owner.

GitHub Actions is **disabled entirely** on `Local` and `main`.
No push, manual, scheduled, PR, package or deployment workflows are allowed
without new explicit operator approval. The existing checks are only run
when manually invoked in an actually available environment. Do not infer
Windows/Minecraft runtime acceptance or release readiness from source
inspection, authored tests, compiler success, a package manifest or an
unsigned candidate. Historic VMware/CI reports remain provenance only.

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
admission. This is a source contract, not completed Windows installed-PC or Minecraft
acceptance. No GitHub Actions runners are active and no package may be
published without separate review.
