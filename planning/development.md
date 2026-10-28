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

Source-only: `native_clients` now separates installed Windows, existing
Windows developer fallback, and non-Windows profile admission. On unsupported
OS, binding/data-location explicitly return `Unsupported`, never a false
`configured: true`; the existing UI cannot enable five launchers from this
path. Windows product behavior and IPC schema are unchanged. Native refusal
regression and static repository guard authored; none executed.

## Tauri startup and instance boundaries — 28 October 2026

Read-only `WindowsNative` / `Unsupported` capability gates Svelte
profile requests and honest UI messaging before Windows binding retries.
The default non-Windows Tauri main window is observed, not recreated.
Windows product WebView overrides, `LOCALAPPDATA` instance lease,
release guards and IPC schema are preserved behind Windows-only gates.
Mac product packaging is prohibited; no shell compilation or gameplay
has been executed.

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

Windows transport fails fast on contention (WouldBlock) and checks queued
requests ahead of idle reaping/audio (250ms). CloseOne persists only its
confirmed selected-slot journal and skips a second global reply reap; CloseAll
and ordinary Inspect still reap all slots. Selected CloseOne no longer issues
a preliminary Inspect with possible unrelated journal I/O. After exact
workspace restoration, the same retained session controller now refuses idle
slots, while a missing controller refuses CloseOne even when no game is seen.
CloseAll's idle-host fallback remains intact. The IPC wire, credentials,
worlds, native identity and recovery owners are unchanged. Synthetic tests
authored, NOT_EXECUTED. Disk/COM work already started may still block.
MC02 normal exit and Windows/macOS runtime remain unverified.

## Next bounded action

Source-review selected-window release failure receipts and UI close recovery
without introducing a second session authority or running local/CI tests.
