# Current Development Handoff — v0.1

## Current intent — 28 October 2026

**Goal:** Restore maintainable, resource-efficient Windows Lazy Tester
source, then identify legitimate, material RAM savings for 0–5 Minecraft
Education clients. The operator explicitly requested CODE-FIRST development
without Windows localtest and asked not to repeat the previous P0 close loop.

**State:** ACTIVE, source-only. Original MC01–MC05 CloseOne/CloseAll code
and native recovery guards remain intact. The last historical MC02 >35 s
normal-exit incident is **RUNTIME ACCEPTANCE DEFERRED**, not a new P0 source
gate. Reopen only for a confirmed new source defect or explicit operator
authorization to perform Windows/game acceptance.

## Active owners

- Mounted Tauri 2/Svelte UI: `Development/apps/desktop/src/LazyTesterApp.svelte`
  and typed bridges; the WindowsNative capability gates unsupported OS.
- Original game/session/profile storage and normal close:
  `src-tauri/src/engine/independent_bootstrap/session_controller.rs`.
  Exactly one authoritative controller and no new account manager.
- Presentation facade: `src-tauri/src/engine/workspace_surface.rs`.
  Win32 adapter: `workspace_surface/native.rs`. Retained recovery controller:
  `workspace_surface/controller.rs`; tests are extracted into submodules.
- DWM facade: `workspace_surface/previews.rs`. Sole capacity-one Win32
  worker: `workspace_surface/previews/windows.rs`; fixtures are separate.
- Existing native compatibility/FPS/HUD DLLs:
  `Development/tooling/acceptance/windows/bootstrap-ordering/`.
- RAM observation only: `Development/tooling/acceptance/windows/collect-native-performance.ps1`
  with its opt-in `native-profile-baseline.ps1` helper, reviewed by existing
  `Development/tooling/acceptance/evaluate-native-performance.mjs`.

## Completed bounded source work

- Separated the original Win32 preview/recovery API, implementations and
  tests mechanically. Retained DWM, Win32 handle checks, original lifecycle,
  one worker, IPC and unsupported-platform refusal without runtime claims.
- Existing read-only memory sampler supports zero to five current native
  clients, explicit sparse MC slot labels, manager and whole-system metrics,
  with optional bounded direct-child attribution. Schema 7 retains exact
  process birth pins and UNKNOWN on identity loss. Schemas 1–6 remain
  historical diagnostic compatibility only.
- Existing evaluator accepts schema 7, rejects false identity/attribution
  and compares matched A/B private commitment, private working set and system
  memory descriptively. Three schema-7 pure logic fixtures were run in a
  sandboxed JS review environment; repository Node/Rust/PowerShell tests
  were NOT_EXECUTED. Updated source verifier owns the new file boundaries.
- Exact Win32 process-image reads now share one bounded Windows-only owner
  (`src-tauri/src/engine/windows_process_image.rs`) in five existing
  callsites: native observation, authenticated IPC, signer verification,
  workspace parent validation and launch bootstrap. Typical image paths
  now start with a 512 UTF-16-character buffer (1 KiB) instead of 32,768
  characters (64 KiB), expanding only when Win32 reports insufficient
  space. This removes repeated temporary heap allocations and one large
  bootstrap stack buffer; it is **not** a measured Minecraft RAM reduction.
  Windows-only fixture authored, NOT_EXECUTED.
- Settings/Storage with no running Minecraft now skip four of five automatic
  native refresh opportunities (10 s instead of 2 s); live games, workspace,
  launcher views and recovery keep the existing 2 s cadence. Uses the single
  existing timer. Only source-level scheduling logic/fixtures were authored.
- The existing native FPS HUD worker now retains one DIB/font across redraws
  at identical DPI/size and releases it at worker exit, eliminating repeated
  GDI create/delete churn without introducing an extra cache, worker or
  process owner. C++/Windows test NOT_EXECUTED.
- The native schema-7 collector now includes opt-in cumulative process
  PageFaultCount from its existing process-memory API call; historical
  schema-5/6 shape and older schema-7 reports remain supported. Page fault
  count is not a hard-fault/disk-read indicator. Five isolated evaluator
  fixture cases passed; PowerShell/Windows NOT_EXECUTED.
- Read-only `InspectSettings` now uses the original exact image path
  pinned during controller preparation for process discovery, avoiding a
  repeated registered-app PowerShell resolution while showing idle status.
  `ApplyPreset`, `ApplyGraphicsPreset`, data-clear/reset and other writes
  still use their existing fresh registration/identity checks. The passive
  `canApplyPreset` flag is only a UI hint, never write authority.
  This reduces temporary Windows process/registry work; memory savings and
  build/runtime behavior are NOT_EXECUTED.
- CI stays OFF. No install, release, Windows game process or user PC touched.

## Evidence and preservation

Only REMOTE_GITHUB source changes and isolated evaluator logic checks are
supported. Rust/Windows compilation, Minecraft preview and normal exit,
CPU/RAM benchmarks, concurrent five-client gameplay, account/multiplayer
continuity and macOS support are NOT_EXECUTED or unsupported.
Historical ~2.49/2.68 GiB client private commit is not a current
revision's measurement. No verified RAM/CPU improvement percentage exists.

Preserve all original MC01–MC05 Roaming/Local/HKCU data, accounts, worlds,
process identity, normal exit/recovery, DLL admission, 144/10 FPS limits,
12-chunk minimum render distance, host simulation/network/save continuity.
No force-close, blind working-set trims, foreign heap releases, speculative
game hooks, new monitoring service, test bypass, CI, installer or main branch
publication. The Mac build remains a diagnostic shell without game backend.

## Next bounded source action

Check any remaining source-owned memory retention that has material allocation
volume AND a complete release/rebuild owner. Prioritize true Minecraft RAM
efficiency only where the Education-native lifecycle is established;
otherwise stop speculative changes, and defer measured 0–5 client RAM,
page-fault, HUD and gaming validation to the explicitly authorized Windows
localtest stage. Do not revive P0 as a code gate.

