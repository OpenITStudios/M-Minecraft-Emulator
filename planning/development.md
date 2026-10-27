# Current Development Handoff — v0.1

## Current source initiative — 27 October 2026

**Goal:** optimize the existing five-profile Minecraft Education Windows
runtime after the completed Sandboxie/VMware source retirement. Protect the
one authoritative controller, native presentation, account/world data and
normal-exit contracts. Extract only genuinely portable policies in preparation
for Windows/macOS; a macOS game backend remains future feasibility work.

**State:** ACTIVE (retirement source audit complete; runtime-efficiency review
underway; current build/gameplay and performance measurements NOT_EXECUTED).

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

The last recorded real two-session Windows run was on source a788a06:
MC01 exited, MC02 did **not** complete normal exit within the observer's
35-second bound, and its process/controller remained. Many exact-source
shutdown fixes followed, but five actual normal exits, input/black-screen
stability, five-client performance, installed package operations, and macOS
multi-account/profile behavior remain **NOT_EXECUTED** for the current source.

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

Six unused Windows research fixture modules (2,866 source lines; no active
consumers) are compiled only for Rust unit tests, not the production graph.
The single session controller, exact native authority, five-profile storage,
DWM and FPS owners remain unchanged. Review identified polling (250 ms idle,
50 ms close, two-second UI) and potential storage head-of-line blocking;
these observations do not prove CPU or RAM savings. No code execution,
installer, Windows or game acceptance has been performed.

## Shared inspection budget — 27 October 2026

The portable `InspectionBudget` applies one cooperative two-second deadline
to five read-only cache/metadata walks. Expired scans are partial and never
authorize cache clear; mutating cleanup still uses fresh admission and its
independent plan. Native I/O/other metadata remain unpreemptible. Regression
sources were authored; builds, tests and performance were NOT_EXECUTED.

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

`workspace_surface` now compiles its existing window recovery/controller
only on Windows. The non-Windows compilation path has a fail-closed,
side-effect-free contract with no process/helper ownership. The public
presentation API and Windows implementation stay unchanged. Authored
non-Windows Rust regression and source guard have not executed; Mac build,
account/profile isolation, game playback and actual optimizations remain
NOT_EXECUTED.

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

## Native observation compile boundary — 28 October 2026

The mounted Win32 process/window/pipe observers are already target-gated.
`native_clients` now also keeps the unused non-Windows Explorer-launch
mutex, cooldown and admission function Windows-only, without altering
Windows behavior or public IPC. This is source compilation hygiene, not
a demonstrated runtime CPU/RAM improvement. Mac diagnostic shell readiness
requires its own compile, main WebView, capability IPC, exit policy and
separate package contract. All remain NOT_EXECUTED.

## Next bounded action

Audit one remaining Mac diagnostic-shell lifecycle boundary: unsupported
platform shutdown and absence of native game/helper work. Do not add a Mac
game controller, run build/tests, or publish an unsupported package.
