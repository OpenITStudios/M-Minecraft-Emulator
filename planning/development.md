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

- Full vendored Sandboxie candidate/owned trees and historical upstream
  gitlink removed from `Local` (earlier 1fd4bdad).
- VMware/Guest Agent/App Sandbox runtime modules and old CLI removed from
  `engine/runtime-core`; platform-independent `independent_engine`
  contracts and bounded subprocess helper retained (earlier b1daa1ec).
- Retired guest scripts, fork compiler/provision scripts, VM installer
  resources and standalone service prototype removed. Common Tauri package
  base is resource-free; the old VM packager remains a hard refusal
  (earlier faef0a84).
- Unmounted Tauri VM/setup/window modules, inaccessible legacy IPC handlers,
  fork candidate and box inventory inspectors removed (earlier a83d6bcf,
  d054b446, 4d790401).
- Final desktop source cleanup removes unregistered Sandboxie Client02
  inspection/launch/focus, boxed process adapters, obsolete four-slot catalog,
  owned-provider helper roles and their scripts. `OwnedProcess` moves to
  `engine/process_identity.rs` preserving the exact JSON schema and field
  names used by the current native controller/workspace.
- The registered Education path resolver is moved byte-for-byte to
  `tooling/acceptance/windows/education-target.ps1`; existing two native
  embeds and the MC01 observer use the same source. Historical docs are
  linked to pinned Git revisions rather than retaining obsolete provider
  folders. Source admission now rejects reintroduced legacy paths.
- Source verifier and frontend routing tests are updated for the current
  package resources and native-only IPC. These are **authored/static reviewed
  only**; no source/runtime test has been executed in this ChatGPT session.

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

At retirement baseline `01b4807e3321ed4077eb73d468e25b7a90070549`, the 28 desktop Rust source/build
owners outside the six research fixture modules were inspected for imports
of `independent_job`, `independent_private_hive`, `independent_private_files`,
`independent_root_access`, `independent_pipe_peer` and
`independent_pipe_server`. There were no production consumers. These six
modules (2,866 source lines combined) are now declared only under
`#[cfg(all(windows, test))]`; their existing Windows unit-test coverage stays
reachable. This is a production compile-graph reduction, **not** verified
runtime CPU/RAM savings. Actual compiler/tests/bundles NOT_EXECUTED.

Review identified three separate scheduling concerns, not yet changed:
- `session_controller.rs` serializes lifecycle operations with
  `InspectStorage`; repeated directory/cache time budgets have now been
  consolidated, but native inspection and settings/clear metadata may still
  block. Never allow parallel game authorities.
- Idle controller receive polling is 250 ms; normal-close observation polls
  every 50 ms for up to 20 seconds. Both are bounded, and neither is proof of
  a meaningful CPU bottleneck. Preserve exact dialog/progress/exit semantics.
- Mounted Svelte status refresh runs every two seconds while visible;
  native DWM previews and focus events already use one owned worker each.
  Avoid adding a second observer or an unverified periodic optimizer.

This phase is source-only. Do not claim five-session gameplay, MC02 normal
exit, power/RAM improvement, Windows build, native DLL/installer acceptance,
or macOS support from these changes. No tests, CI, games, or user PC operations
were run.

## Shared inspection budget — 27 October 2026

Source-only: `runtime-core/inspection_budget.rs` now holds the reusable,
OS-neutral monotonic deadline. One cooperative two-second budget spans all
five read-only cache classifiers and profile metadata walks per
`InspectStorage`; expired work becomes partial, never deletion permission.
Each storage report reuses one read-only admission observation per slot.
Mutating cache clears retain their independent fresh two-second safety plan.
Existing controller, IPC wire, profiles, DLL, normal-close and Win32
authority are unchanged; no new worker, queue or macOS backend is added.
Added source regression cases for expired budget and incomplete cache.
Compiler/tests/Windows/gameplay/performance remain NOT_EXECUTED. OS calls and
clear-confirmation inspection are not preempted, so a hard latency cap is not
claimed.

## Next bounded action

Review controller request scheduling and read-only inspect coalescing versus
CloseOne/CloseAll while preserving exact process authority, then evaluate the
existing Tauri/Rust Windows-only compile boundary before macOS adapter work.
No Local PC testing without explicit approval.
