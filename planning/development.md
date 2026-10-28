# Current Development Handoff — v0.1

## Current source initiative — 28 October 2026

**Goal:** optimize the existing five-profile Minecraft Education Windows
runtime after the completed Sandboxie/VMware source retirement. Protect the
one authoritative controller, native presentation, account/world data and
normal-exit contracts. Extract only genuinely portable policies in preparation
for Windows/macOS; a macOS game backend remains future feasibility work.

**State:** ACTIVE (Windows reliability source hardening underway; current
build/gameplay and performance measurements NOT_EXECUTED).

The current mounted product uses Svelte/Tauri 2 and a retained Rust session
controller for MC01–MC05. Minecraft owns sign-in, network, worlds, rendering
and licensing. Native `bootstrap-ordering/` C++ DLLs, `workspace_surface.rs`,
DWM thumbnails, native FPS/HUD, exact PID/birth/process handle checks, and
all original per-launcher Roaming/Local/HKCU data ownership stay unchanged.

## Retirement and dependency audit

Sandboxie vendor/fork, VMware Guest Agent, obsolete setup/IPC/runtime owners,
guest scripts and legacy packaging paths were retired from `Local` in
earlier commits. No alternative provider or old installer is a fallback.
The native Education source/controller, verified five-slot profiles, DWM,
FPS DLL, process roles, original data and release safeguards remain current.
Git history retains provenance. Source verifiers were updated without CI,
build or game execution.

## Current owners

`apps/desktop/src/LazyTesterApp.svelte` → typed Lazy Tester Tauri commands
→ `engine/native_clients_session.rs` authenticated transport →
`engine/independent_bootstrap/session_controller.rs` retained one-authority
five-slot game owner. `engine/process_identity.rs` is a data-only type;
`engine/workspace_surface.rs` owns presentation/recovery; the native DLL
under `tooling/acceptance/windows/bootstrap-ordering/` owns the retained
compatibility/pacing code. `engine/runtime-core` contains portable domain
contracts and the bounded subprocess runner, not VM providers.

## Evidence ceiling and known runtime issue

The last recorded two-session Windows run (a788a06) closed MC01, but MC02
did not exit within 35 seconds. Subsequent source fixes, five-client normal
close, performance, packaging and macOS gameplay remain NOT_EXECUTED.

Neither removing Sandboxie source nor completing a Rust module refactor is
evidence that the original MC02 exit issue is solved. No forced game close,
account sign-out, profile mutation, disk/snapshot cleanup, installer
publication or local-PC testing is authorized in this task.

## Product and preservation boundaries

- Do not run GitHub Actions, enable CI, sign/publish a release or modify
  `main`. `Local` is the only active development branch.
- Old installed app identifiers, existing account/world/registry state and
  VM disks/snapshots remain untouched. Git history preserves retired source.
- Do not replace the retained controller with a second lifecycle manager.
  Unknown process/HWND, failed recovery or uncertain request must fail closed.
- No new Windows/macOS adapter, new game hook, renderer modification, cloned
  login, token read or unofficial game installation as a cleanup side effect.

## Efficiency source review — 27 October 2026

Six unused Windows fixture modules (2,866 lines) are test-only, not in
production. Native controller and five-profile owners remain intact.
Polling and storage head-of-line concerns are source observations only,
not CPU/RAM improvements. No local execution or game acceptance.

## Shared inspection budget — 27 October 2026

The portable two-second `InspectionBudget` spans five read-only metadata
walks. Expired results are partial and cannot authorize cache clear; mutating
cleanup uses fresh admission. Native I/O remains unpreemptible; tests and
performance NOT_EXECUTED.

## Close-review scheduling — 27 October 2026

Native titlebar confirmation emits `lazy-tester-close-review` before the
dialog and clears it on cancel/error. The Svelte view pauses automatic
inspect/layout during native and selected-game close review, tracks an
observation epoch to discard stale in-flight reads, and delays CloseOne
confirmation until earlier refresh/preview work settles. No new worker,
second controller, priority queue, game mutation or IPC wire change. The
existing admission gate still refuses concurrent operations, including
an already-active native read; no performance or CloseOne acceptance claim.

## Cargo target boundary — 28 October 2026

Desktop Win32 bindings now use Cargo `cfg(windows)`, keeping versions/features
unchanged; static repository guard and negative fixtures are added. Windows
packaging, Tauri IPC, lifecycle and installed data were not changed. This is
source-only; Mac compilation and native backend remain NOT_EXECUTED.

## Native presentation boundary — 28 October 2026

`workspace_surface` retains its original Windows controller/DWM/recovery
owner. Unsupported OS contracts return `Unsupported` without launching
helpers or owning windows. Rust source guards authored only; native
tests, gameplay and macOS compilation remain NOT_EXECUTED.

## Unsupported-platform profile readiness — 28 October 2026

Windows binding/data-location paths remain unchanged. Non-Windows returns
`Unsupported`, not `configured: true`, and cannot enable five launchers.
Source guard and refusal regressions authored, NOT_EXECUTED.

## Tauri startup and instance boundaries — 28 October 2026

Read-only `WindowsNative` / `Unsupported` capability gates Svelte
profile requests and honest UI messaging before Windows binding retries.
Non-Windows uses the default Tauri window; Windows WebView, IPC and
instance lease remain Windows-only. Mac gameplay NOT_EXECUTED.

## Native exit and macOS feasibility — 28 October 2026

Win32 process/pipe/window and CloseAll/recovery stay `cfg(windows)`.
Non-Windows keeps default Tauri close, no native helper or instance lease.
Both native package features refuse non-Windows targets. Mac compile,
single-instance, WebView, signing and gameplay remain NOT_EXECUTED or absent.

## Unsupported-platform UI admission — 28 October 2026

All mounted Svelte startup, queued work, previews, polling and unmount
operations require Rust-confirmed `WindowsNative`; unsupported hosts offer
read-only recheck and no game controls. The sole raw Tauri invoke owner is
`invokeRuntime.ts`; typed IPC remains in `lazyTester.ts` and
`workspaceSurface.ts`. Static/browser regressions authored, NOT_EXECUTED.

## Windows reliability source hardening — 28 October 2026

Windows controller reaps games every 250 ms; stable COM mute work is gated
by process/focus changes or its one-second cadence. Native DWM worker skips
duplicate property writes, revalidating HWND and geometry every visible
refresh. The single SURFACE Lease now skips redundant PASSIVE Place helper
IPC only after a full prior Place+foreground receipt, live helper check and
exact PID/birth/session/HWND/thread/class/frame/shell/geometry readback.
Explicit Play, unknown/failed Place, moved window or age >=1 second still
uses ordered native helper. Park's existing no-op also requires full native
readback; Park/Hide invalidate Place cache. No second worker, timer, IPC
protocol, game lifecycle or data migration. Source regressions NOT_EXECUTED.
MC02 exit, Windows DWM/helper and CPU/RAM performance unverified.

## Resource benchmarks — NOT_EXECUTED

Before/after Windows builds: idle 0/5 games, audio Keep/Mute, stable
previews, rapid drag/resize/minimize/restore, CloseOne with four survivors.
Measure per-process CPU/private bytes/working set, helper Place/Park IPC
counts and timings, DWM writes, native identity and actual normal exit.
No performance gain claimed without measured evidence.

## Next bounded action

Audit repeated native helper Hide, manager aperture and foreground
confirmation under stable event bursts without weakening game recovery.
