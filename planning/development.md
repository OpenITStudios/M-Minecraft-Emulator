# Current Development Handoff — M-Minecraft-Emulator

## Dependency modernization and structure cleanup — 20 October 2026

**State:** implementation delivered; full validation remains incomplete.
Source revision: `e6eb1358bf94ab524d70666ccbfb8674dfa8f35e`.
Canonical [report](../docs/validation/dependency-modernization-2026-10-20.md)
records all dependency versions, structural changes, commits and exceptions.
Source batches through `5ecbf268c7cac9d9d8d71911090a15018543bc27` were pushed and remotely verified; a later tooling-only registry-error correction has its own test evidence in the report.
Vite/plugin/Vitest, native TS7 + Svelte TS6, Tauri/Rust networking/system crates,
and portable toolchains were modernized with npm/Cargo-generated locks.
Windows source checks, 174 core tests, 62 desktop Rust tests and 161 Vitest tests
pass. The aggregate has an unchanged PresentMon symlink EPERM failure; existing
symlink skips, desktop formatting debt, GTK RustSec warnings and native package
release hold remain explicit. No CI, main, driver, accounts or gameplay changed.
**Next for this topic:** resolve the recorded acceptance exceptions before claiming
fully verified modernization; do not bypass checks or resume legacy operations.


**Repository:** `OpenITStudios/M-Minecraft-Emulator`; all product changes on
branch `Local`. Current operator decision: **GitHub Actions CI OFF**.

## Continued development: non-blocking observer Stop

The last observer source owner still used a potentially indefinite
`JoinHandle::join()` immediately after requesting `WM_QUIT`, holding
the global Rust manager operation lock. A message queue blocked in a
Windows callback could freeze other manager operations indefinitely.
In this bounded change, Stop posts at most one `WM_QUIT` per worker,
returns a truthful `WouldBlock` while cleanup is pending, preserves the
only worker handle, and joins **only when `is_finished()` is true**.
A later explicit Stop confirms cleanup, including propagation of a
`GetMessageW` or `UnhookWinEvent` failure. Startup and account/focus
attribution are unchanged.

This is a source-only correction on the CI-disabled `Local` branch.
It is not a Windows runtime test, game FPS measurement or proof of
four Minecraft instances. No Actions workflow is restored.

---

## Current work: retire CI and preserve safe foreground observation

The operator explicitly revoked approval for GitHub Actions on
20 October 2026. Remove **both** workflow definitions from `Local`:
`.github/workflows/verify.yml` (automatic Full Source / manual Quick /
Full Source / Windows Build) and `.github/workflows/package-smoke.yml`
(manual installer/package testing). The separate registration-only
`main` workflow was removed in
[main commit cb106f9](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/cb106f9013206a73f6172c12fb62ee590a1c6b15).
The canonical `toolchain.json`, root rules, repository verifier and
validation router now reflect the CI-disabled policy. Never restore an
Actions workflow, third-party hosted runner, or scheduled CI without
fresh explicit user approval. Existing `npm run verify:source`, Rust, Svelte
and Windows scripts remain *available* for optional manual execution
when supported and requested. They do not run automatically.

This repository write is not the same as changing the repository-wide
GitHub **Settings → Actions** administrative switch: the connector exposes
source editing but not that settings mutation. To disable GitHub Actions
even against future workflow re-additions or other branches, an
administrator must turn Actions off in the repository settings. No new
Actions workflow definitions remain on current Local/main after this
batch if both deletion commits succeed. Previously started workflow
runs may finish; historical logs remain historical evidence only.

## Continued engineering: fail-closed observer recovery

The existing opt-in Windows foreground event hook is still ONE Rust worker
in `apps/desktop/src-tauri/src/engine/instance_focus_events.rs`, with
unbuffered startup handshake, same-thread unhook and explicit Stop. The
operator-facing Start/Read/Stop state previously had a blind spot: a
failed IPC could cause the UI to offer Start even when hook cleanup was
unverified, and a worker holding the lock could be misreported as stopped.

Changes in this batch:
- A Rust status read returns an error for uncertain worker ownership
  (installing, exiting, unreaped or busy), not a fabricated `enabled=false`.
- The Svelte UI blocks Start after any failed transition or status read,
  offers **Retry Stop/cleanup** and a separate explicit **Read event count**.
- A successful Stop or valid status read restores the appropriate controls.
- Transport checks and negative frontend/source contracts updated.
- No extra service, timers, process mutation, Sandboxie driver action,
  or Minecraft graphics/simulation/account setting change.

## Evidence / remaining blockers

The preceding `Local` commit `b5db7f6c4a026ab17a29edc012427b0d04cb05d9`
[failed desktop Vitest on GitHub](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39292712346):
one assertion sought a cleanup log string without its `MIVUBI` prefix.
The actual backend log was correct. Repository, native/isolation stages
and Rust checks passed on that prior SHA. The assertion has been fixed.

**No CI will run for this new commit**. Static review supports source
coherence only; the new Rust/UI behavior is **NOT_EXECUTED** in Windows.
Source compilation, hook lifecycle on a real desktop, active Client 03/04,
four independent licensed Minecraft Education sessions and 15–20 FPS
external limiting are still unverified or not implemented. Editable owned
v1.18.6 source exists but is not an installed, supported runtime.

**Next source task:** establish positive, provider-owned Client 01 and
03/04 process attribution; do not infer it from process names or generic
Sandboxie query failures. Keep the entire performance system external
to Minecraft and leave FPS mutation disabled until a supported vendor/API
route has been measured on Windows.

Historical work: [planning/history](./history/development-through-2026-10-19.md).
