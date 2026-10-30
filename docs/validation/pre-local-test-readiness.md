---
id: document.validation.pre-local-test-readiness
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: HISTORICAL
---

# Legacy VMware G0 — Source & Package Integrity

> **Superseded:** This is a historical VM gate, not the native Lazy Tester
> packaging route. Current five-profile verification starts at
> [validation routes](README.md); commands below are historical only.

**Current selected-isolation clarification (2026-10-12):** This is the
historical VMware NSIS release gate, **not** the current Minecraft Instance
Manager test entry. The active native + Sandboxie Plus pilot uses pinned
source checks and `npm --prefix Development run dev`, with the operator procedure at
[Sandboxie Client 02 Feasibility](sandboxie-two-client-feasibility.md).
The legacy `npm --prefix Development run package:windows` / `DEV.cmd g0` path is intentionally
blocked with `PACKAGE_BLOCKED_NATIVE_FEASIBILITY`; do not bypass it to
obtain a test installer. A verified legacy installer at an older SHA does
not validate the new native desktop.

This document describes the **four mandatory checks within G0**, not four extra product gates. The five and only five product-testing gates are defined in [Acceptance G0–G4](acceptance.md). Existing `Local` workflows and build scripts remain unchanged.

## Exact-SHA G0 checks

| Check within G0 | Required executed evidence | Current status |
| --- | --- | --- |
| **Source integrity** | `npm --prefix Development run verify:source` on one pinned `Local` SHA: repository invariants, Node/Svelte/Vitest, Tauri Rust checks/tests, runtime Rust checks/tests/fmt, and guest feasibility evaluator | NOT_EXECUTED |
| **Windows package construction** | Successful `npm --prefix Development run package:windows`; actual NSIS output, build result, target identity and installer hash tied to that SHA | NOT_EXECUTED |
| **Installed payload completeness** | Inspect all mandatory items in `Development/engine/distribution/package-contract.json`, including host CLI, guest Agent, Base setup/finalization scripts and single-guest evidence collector | NOT_EXECUTED |
| **Disposable installation safety** | Observe install/repair/uninstall/reinstall preserving `M-Bedrock/VirtualClients` guest/account data; no unexpected downgrade or untrusted updater behavior | NOT_EXECUTED |

**G0 PASS requires all four checks to have real positive evidence for the same revision.** A GitHub commit, authored unit test, static repository check, or absence of reported CI failure does not satisfy any unexecuted check. One failure is diagnosed in the first causal owner and G0 is re-evaluated on the corrected SHA. Do not count checks as G1–G4; those names are reserved for gameplay and lifecycle outcomes.

## Development boundary

ChatGPT + GitHub `Local` is the available source-work mode. Workflow definitions are `workflow_dispatch` only and are not dispatchable with `main` kept README-only. Do **not** edit `main`, change the default branch, activate automatic CI, assume another execution service, or ask the user to perform incremental machine tests.

When executable evidence is unavailable, **G0 stays `NOT_EXECUTED`**. This is neither a build failure nor permission to mark G1–G4 as passed. Source-level engineering can continue only for a grounded defect and must keep its proof ceiling explicit.

## Subsequent gates, when real evidence exists

G1's first real gameplay question is a *disposable account-clean Windows VMware guest*, not three production clones. Observe Windows desktop boot and VMware Tools, install one legitimate Minecraft Education Windows variant, sign in with a properly licensed organization account, and interact with a rendered world for the documented initial observation period. The [Single Guest Feasibility](client-feasibility.md) procedure and its existing collector separate operator statements from measured graphics/process facts. A G1 failure stops escalation to G2; no premature account-bearing cloning.

G2 tests Base-to-three-guest identity and provenance; G3 tests four simultaneous independently usable accounts; G4 tests recovery and required multiplayer paths. Their detailed assertions remain in [Acceptance](acceptance.md), with existing scenario coverage in [Failure Matrix](four-client-failure-matrix.md).

## Evidence handoff

Record the pinned GitHub SHA, command/environment **only if actually executed**, exit result, installer hash and inspected payload when applicable, and unresolved proof in `planning/development.md`. Do not create a new gate database, runner, auto-generated PASS, or duplicate readiness flag.

### One-batch local G0 preparation (Windows, optional)

`DEV.cmd` is an optional developer convenience; it never starts or modifies a VMware guest. Use only a checkout of branch `Local` on a Windows x64 machine with PowerShell 7+, Node/npm and Cargo matching `Development/toolchain.json`:

```powershell
.\DEV.cmd lock    # only while the desktop Cargo.lock is missing; performs real Cargo registry resolution
```

The `lock` command checks the exact clean `Local` checkout and pinned Rust/Cargo toolchain; it deliberately does **not** require Node/npm, which are needed later for G0, not for Rust dependency resolution. Cargo still requires genuine crate index access or a usable cached index. It will not synthesize a lockfile when resolution is unavailable.

Review the generated `Development/apps/desktop/src-tauri/Cargo.lock`, then commit and push it to `Local`. Do **not** manually construct or copy another application's lockfile. Once that committed lockfile is present in a clean `Local` checkout, the following single batch runs the pinned toolchain check, npm `ci`, exact-revision source checks, and an unsigned Windows NSIS build:

```powershell
.\DEV.cmd g0
```

The pinned-language check is **not** a full Windows packaging prerequisite probe. Before `g0`, Windows must have the Tauri-required MSVC C++ build toolchain, Windows SDK and WebView2 availability as appropriate to the installation mode. Missing system prerequisites should be diagnosed as G0 environment blockers, not patched around by weakening source contracts.

`g0` explicitly rejects untracked Cargo.lock, dirty source, wrong branch, toolchain mismatch and a changed HEAD. After an actual successful package build, the launcher also verifies that the just-produced versioned NSIS installer exists and is nonempty, that the staged CLI and Guest Agent binaries exist, and prints the exact source SHA, installer path and SHA-256. It does not run NSIS or claim installed payload correctness.

Successful commands cover only the source/build/package-construction portion of G0. A real installer/payload inspection and **disposable** install, repair, uninstall and reinstall preserving runtime data remain independent requirements. A successful console message never establishes G0 PASS or Minecraft/GPU/client readiness. Never conduct first acceptance using account-bearing VM disks.

Neither command changes `main`, installs guest agents, starts VMs, resets snapshots, signs into Minecraft, or commits on the user's behalf.

### Current desktop Rust lockfile and release hold

The repository now tracks `Development/apps/desktop/src-tauri/Cargo.lock`, as well as
`Development/engine/runtime-core/Cargo.lock`. The historic missing-lockfile notes
are resolved in source, but the new exact-SHA Node/Svelte/Rust execution
status must still be determined through real checks. `npm run
verify:source` checks the pinned lockfiles and frontend/backend contracts.
Its success does **not** prove Minecraft authentication, graphical output
or multiplayer.

This G0 procedure is retained for the legacy VMware distribution. The
current native UI cannot be shipped under that NSIS identity with VMware
Guest Agent resources. A new native package contract, identity, data
isolation and install/uninstall preservation require separate review.

**Current native release:** BLOCKED / NOT_EXECUTED. Historical G0 success
at `509ce10f0fd5129e4da2990614651fb57241b256` applies only to the
older VMware package, not to this revision.
