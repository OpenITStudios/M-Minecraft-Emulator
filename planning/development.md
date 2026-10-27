# Current Development Handoff — v0.1

## Current source initiative — 27 October 2026

**Goal:** retire unused Sandboxie and VMware implementations from the
`Local` source tree, without damaging the active five-profile Minecraft
Education Windows manager. Prepare later efficiency and Windows/macOS
architecture work only after the native implementation graph is clean.

**State:** ACTIVE (source retirement complete for the known desktop/runtime/
vendor/Guest Agent/legacy-package paths; actual build/gameplay NOT_EXECUTED).

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

## Next bounded action

Perform exact-HEAD static consumer and package-contract verification of the
retirement commit and record remaining unknown compiler/Windows/gameplay
proof; afterwards proceed to source-only lifecycle scheduling and performance
architecture work. No Local PC testing until explicitly authorized.
