---
id: document.system.implementation-map
class: DOCUMENT
domain: system
role: GUIDE
authority: CANONICAL
lifecycle: ACTIVE
---

# Active implementation ownership map — Lazy Tester

**Purpose:** locate the first current source owner for one concrete
problem and read only its direct callers, consumers and relevant tests.
This is a source navigation guide, **not a task backlog**. The current
product has five independently managed native Minecraft Education
Windows profiles. Earlier VMware, Sandboxie, Guest Agent, four-slot
catalogs and owned-driver prototypes are **retired**, not active routes.
Their old file paths and experiments remain in Git history, not here.

## Locate the first owner

All desktop paths below start at `Development/apps/desktop/`.

| Problem | Existing canonical source | Follow only when needed |
| --- | --- | --- |
| Main Play, Settings, Storage and action sequencing | `src/LazyTesterApp.svelte` | `src/app/inspectionScheduling.ts`, `src/app/presentation/` |
| Optional MCP-reported AI/QA disclosure | `src/app/presentation/agentActivity.ts`, `src/AgentActivityNotice.svelte` | Existing `agent_capabilities` explicit status mirror in `Development/tooling/agent/mcp.mjs`; original controller `SessionSnapshot.agentActivity` only, no new watcher/QA owner |
| Launcher rail and status guidance | `src/LauncherRail.svelte`, `src/SessionHealthNotice.svelte` | `src/app/presentation/sessionFeedback.ts` |
| Settings and storage presentation | `src/GeneralSettings.svelte`, `src/DataStorage.svelte` | Typed `src/app/bridge/lazyTester.ts` |
| F3 coordinate math and read-only evidence continuity (source-only) | src/app/coordinates.ts (math), src/app/coordinateEvidence.ts (untrusted candidate gate) | XYZ/block/teleport text, strict process hint screening, and two-frame slot, identity, dimension, tick and monotonic-time checks. Every result remains copyAllowed=false. See docs/system/coordinate-inspector.md. |
| Local agent MCP tools / bounded evidence reading | `Development/tooling/agent/mcp.mjs` | The opt-in MCP and `agent_cli.rs` pass a fresh per-transaction fencing epoch to the original mode-bound writer leases in `native_clients_session.rs` and `independent_bootstrap/session_controller.rs`; stale old-child input/neutral/release cannot adopt a newly leased replacement; per-slot recovery and `visual-observer.mjs` remain source-only, gameplay NOT_VERIFIED |
| Local-first Minecraft Education diagnostic archive and MCP issue queries | `Development/tooling/agent/diagnostics-local.mjs`, existing `mcp.mjs` | Explicit source-only JSONL import into ignored `Data/Diagnostics`, allowlisted read-only MCP projections; no live Education log transport or actual gameplay identity verified. See `docs/system/map-diagnostics.md` |
| Canonical Education team analytics and results (read-only capture) | `Development/tooling/agent/mcp.mjs`, `evidence-summary.mjs` | Consume only team `[ANALYTICS]` and `[GAME_RESULTS]` under the upstream 24 KiB contract; no local game telemetry engine. The team's modules and Skills Probe remain the source of truth. See `docs/system/education-team-telemetry-compat.md`. |
| One reusable in-game team command help (not telemetry) | `Development/tooling/agent/education-team-help-pack/functions/`, `build-education-team-help-pack.mjs` | Data-only v2.1.0 pack: `/function help` delegates to `lazytester/help` which privately displays original team commands with `tellraw @s`; no Script API, telemetry, analytics, game control or Codex calls. This replaces the retired universal diagnostics pack. |
| One-entry Skills Probe AI QA (no dispatch) | `Development/tooling/agent/probe-qa.mjs`, existing `mcp.mjs` | Preferred source-only `probe_qa` owns one frozen original result per MCP/slot. `coordinateNative:true` reserves in the SAME native controller for cooperating MCP clients and pins its verified CLI's OS-observed direct Node parent handle/birth; `inspectRun:true` reports bounded parent lifetime, token equality and a blocked takeover gate, plus an optional read-only reconciliation against checksum-checked Data/Testing summaries; it never transfers ownership. Neither parent identity nor token is proof of MCP-script authenticity or operator approval. Optional `Data/Testing` immutable original-Probe SUMMARY is accessible by `archivedRunId` after MCP restart. Live caller identity is observed at the pinned Windows parent-handle level; the original controller has a private, unwired sealed CAS transfer core, but independent native archive attestation, operator authentication, live takeover and gameplay acceptance are NOT available; no gameplay acceptance or new runner. See `Development/tooling/agent/README.md`. |
| Optional map-aware QA planning (source-only) | `Development/tooling/agent/map-test-plan.mjs`, existing `mcp.mjs` | Operator-reviewed map JSON contract, bounded stable-file cache, source-only smoke/full/affected selection, bounded per-check mapPlan and one existing `probe_qa.qaReport` projection of original Probe result/map NOT_TESTED/evidence gaps. No map test executor, world command, verified diff, or gameplay PASS. See `docs/system/map-aware-probe-qa.md`. |
| New team ZIP versions / source-compatibility review | `Development/tooling/agent/team-update.mjs`, `team-update-zip.mjs`, `team-source-baseline.json` | Explicit local-only comparison of two developer ZIPs and Skills Probe lock files. Emits bounded review report; no module copy, network, Minecraft process, Codex upload or automatic approval. See `docs/system/education-team-source-updates.md`. |
| AI input timeline / native-bound preview (no default input dispatch) | `Development/tooling/agent/input-timeline.mjs`, `input-route.mjs` | `control-plan.mjs`, nonexecuting input transaction port `input-delivery.mjs`, and MCP `assess_input_route`; real background game input remains blocked |
| One raw Tauri invoke boundary | `src/app/bridge/invokeRuntime.ts` | `src/app/bridge/lazyTester.ts`, `workspaceSurface.ts` |
| Allowlisted native commands / heavy-operation admission | `src-tauri/src/commands/native_clients.rs` | Session and workspace owners |
| App startup / normal manager exit | `src-tauri/src/app_bootstrap.rs` | `engine/app_instance.rs`, native close command |
| Manual game launch / reaping / normal CloseOne and CloseAll | `src-tauri/src/engine/independent_bootstrap/session_controller.rs` | `independent_bootstrap.rs` |
| Game image, native enrollment and reviewed DLL checks | `src-tauri/src/engine/independent_bootstrap.rs` | `independent_bootstrap/owned_handoff.rs` |
| Profile data, binding, settings and storage inspection | `src-tauri/src/engine/independent_bootstrap/` | `product_profiles.rs`, `profile_binding.rs`, `profile_storage.rs`, `game_preset.rs` |
| Verified process/window identity | `src-tauri/src/engine/instance_identity.rs`, `process_identity.rs` | Existing native process and workspace callers |
| Retained authenticated controller transport | `src-tauri/src/engine/native_clients_session.rs` | `native_clients_session/pipe.rs` |
| Window placement / restoration | `src-tauri/src/engine/workspace_surface.rs` | `workspace_surface/controller.rs`, `native.rs` |
| DWM preview scheduling and native Windows worker | `src-tauri/src/engine/workspace_surface/previews.rs` | `workspace_surface/previews/windows.rs` |
| Persisted manager performance settings | `src-tauri/src/engine/manager_settings.rs` | Existing per-profile native settings |
| Pure cross-platform domain contracts | `Development/engine/runtime-core/src/independent_engine/mod.rs` | Applicable Rust module only; not a second launcher |
| Official Education target detection | `Development/tooling/acceptance/windows/education-target.ps1` | Verified native game-image owner |
| Active native compatibility DLL source | `Development/tooling/acceptance/windows/bootstrap-ordering/` | Reviewed payload/package admission |
| Native Windows installer and preservation | `Development/engine/distribution/lazy-tester-package.json` | `Development/apps/desktop/src-tauri/tauri.lazy-tester.conf.json` |
| One developer command/validation entry | `DEV.cmd` | `Development/tooling/developer/controller.mjs`, `tasks.json`, `Development/tooling/windows-toolchain/dev.ps1` |
| Developer artifact inventory and retention | `Development/tooling/storage/` | Current storage and package scripts; no generic deletion |
| Source contracts and optional Windows evidence | `Development/tooling/repository/`, `Development/tooling/acceptance/windows/` | `docs/validation/README.md` |
| Current work, blockers, proof ceiling and single next action | `planning/development.md` | Exact `Local` commits |

## Minimum-change routing

```text
pin current Local HEAD + read AGENTS.md / GITHUB_RULES.md
  -> select one row and read current source
  -> check directly affected callers and tests
  -> reuse existing owner or native capability first
  -> make the smallest complete correction if authorized
  -> verify at the strongest genuinely available proof level
  -> commit coherent outcome on Local, update one handoff, STOP
```

For AI context economy, do **not** automatically load
`docs/system/independent-engine-architecture.md`,
`independent-isolation.md`, old VM/Sandboxie validation reports,
archived experiments, or large RAM research notes. Load them only
when a specific comparison requires historical evidence.

## Boundaries never to cross

- The Svelte frontend is not Minecraft's lifecycle, identity, signing
  or data-preservation authority.
- Portable Rust domain contracts are not a competing launcher,
  mutable session registry or Windows native adapter.
- DWM windows/previews are not independent Microsoft accounts.
- Test-authored, source-reviewed, installer-built and Minecraft-played
  are different evidence classes. No synthetic proof becomes gameplay.
- Never reintroduce retired launch routes, extra Manager, permanent
  monitor, hidden retry/recovery daemon, general cleanup tool or CI.
- Installed product identities and account/world paths are preserved;
  code-folder refactors must not incidentally migrate or delete them.

For product behavior see [unified workspace](../product/unified-workspace.md)
and [data layout](data-layout.md); for stable layering see
[architecture](architecture.md). For previous pilot history see
[legacy retirement](legacy-retirement.md). The historical 67 KB
implementation map remains recoverable from the previous `Local`
commit, and should not be duplicated into the active tree.
