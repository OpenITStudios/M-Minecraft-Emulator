# Lazy Tester — optional local AI/MCP Computer Use bridge

**Status:** source-only experimental per-client AI input; Windows/Minecraft
runtime **NOT_VERIFIED**. This README is the active short navigation,
not an installed/tested AI controller. Follow the
[controlled Windows local-test guide](../../../docs/validation/ai-computer-use-localtest.md)
for the future approved verification. Earlier phases and detailed source
notes are preserved in [history](../../../planning/history/mcp-agent-implementation-notes-through-2026-11-01.md).

## Source ownership

`mcp.mjs` is the single optional **local stdio MCP** bridge. It uses the
same `--lazy-tester-agent-cli` adapter and retained authenticated original
native session controller; it does not own Minecraft, start a background
agent daemon, host/join worlds, access credentials or launch a second game
controller.

| Existing owner | Purpose |
| --- | --- |
| `mcp.mjs` | MCP schema, opt-in, mode/slot and lease admission |
| `control-plan.mjs`, `input-timeline.mjs`, `mc02-action-sequence.mjs` | Bounded semantic action plans and independent pulse frames |
| `input-route.mjs`, `input-delivery.mjs` | Read-only native target assessment and nonproduction input contract |
| `visual-observer.mjs`, `visual-capture.mjs`, `gpu-frame-source.mjs` | Per-window optional read-only frame capture/cache |
| `native-gpu-capture/` | WGC source candidate; not built or packaged |
| Desktop `agent_cli.rs`, `native_clients_session.rs`, `independent_bootstrap/session_controller.rs` | Same-image CLI, authenticated IPC, original game process/channel owner |
| Native `acceptance/windows/bootstrap-ordering/mc02-virtual-input.cpp` | Compile-OFF C++ Win32 input candidate; no game-level proof |

## Modes and default capabilities

- `human-plus-four-ai` (default): human MC01 is protected; only MC02–MC05
  may be opted-in AI input targets. Active AI inputs require the **real**
  foreground MC01 to remain human-owned.
- `five-ai`: all MC01–MC05 may be explicitly opted-in AI targets, each
  with the native matching launch mode. Not a tested multi-AI result.

The **default MCP** exposes profile inspection, explicit launches, action
planning, evidence reading, target identity and optional visual capture.
Independent active keyboard/mouse tools are **not advertised** by default.
The separate experimental mode adds
`run_experimental_multi_client_actions`,
`stop_experimental_multi_client_actions` and
`run_experimental_observed_actions`, but their receipts only acknowledge
native transport and never establish Minecraft movement.

## Optional input/capture integration (not authorized to execute yet)

- Desktop Rust `multi-client-native-input` alias `mc02-native-input`
  and CMake `BOOTSTRAP_MULTI_CLIENT_VIRTUAL_INPUT` are **OFF** by
  default. The CMake experiment also requires
  `BOOTSTRAP_ORDERING_TEST_ONLY=ON` and clean reviewed Detours.
  `BOOTSTRAP_MC02_RAW_BUFFER_POLL` is an independent OFF option whose
  actual Minecraft consumer has not been proven.
- To expose experimental multi-client MCP tools requires the separate
  `--multi-client-input-experimental on` flag and exact consent environment
  `LAZY_TESTER_MULTI_CLIENT_VIRTUAL_INPUT=I_ACCEPT_EXPERIMENTAL_MULTI_CLIENT_INPUT`.
  Their presence in source is **not** authorization to turn them on.
  The historical MC02-only mode is mutually exclusive.
- Each opted-in original game owns one 96-byte read-only input channel
  bound to its original PID/birth/session/verified HWND, control mode,
  native writer lease/transaction epoch, input TTL and neutral cleanup.
  ReleaseAll/IPC ACK is not proof Minecraft saw key-up; no global
  `SendInput` or Windows foreground takeover is provided.
- Optional visual capture requires separately available Cua Driver or
  unbuilt/unpackaged WGC worker. Screenshots can be stale or blank.
  The **external** AI model must inspect individual images and decide
  next actions; there is no autonomous model loop in this repo.

## Minecraft Education team source sync, help and read-only MCP

**One authoritative game runtime:** the team's original
`minecraft-bedrock-agent-skills` modules; only the team `skills-probe`
project is used to validate them. Lazy Tester does NOT reimplement
their analytics, gameplay scoring, subscriber engine or telemetry.
The earlier `[LT_EDU_DIAG]` parallel universal diagnostics pack has
been retired from active source and must not be enabled on QA maps.

The optional minimal `education-team-help-pack/` is now a
**data-only function pack** with no Script API dependency, listener,
background work, telemetry or gameplay writes.

Minecraft command (operator/cheats in a copied QA world):

    /function help

This runs one canonical team-command list via `tellraw @s`, mirroring
the static `.mcfunction` convention used by Skills Probe.
When a map owns a conflicting `help` function, the fallback is
`/function lazytester/help`. Minecraft's native `/help` command
remains untouched.

The offline `build-education-team-help-pack.mjs` stages one v2.1.0
.mcpack only under ignored
`workspace/diagnostics/education-team-help/v2.1/`. It does not install
a team module, perform runtime testing, touch a Minecraft world,
or use Codex. Developer exports and Skills Probe-only commands
are displayed but never executed by the helper.

**Existing local MCP** remains a bounded, read-only consumer of
`[ANALYTICS]` and `[GAME_RESULTS]` captured records. Four
`get_diagnostics_overview/list_issue_candidates/get_issue_details/query_diagnostic_evidence`
tools query explicitly imported `Data/Diagnostics` evidence only.
The game capture path and source identity are still NOT_VERIFIED in
Minecraft Education 26.33 Windows. No raw capture automatically
streams to Codex.

For team-owned modules, use the project's existing
`bun run skills:check` and `bun run skills:sync` workflow rather
than copying module code into Lazy Tester. For module-specific
prerequisites and upstream ZIP hashes, see
[team source mapping](../../../docs/system/education-team-telemetry-compat.md).
For storage and help-addon details see
[map diagnostics](../../../docs/system/map-diagnostics.md).

## Team ZIP update intake (local-only, no module duplication)

When new versions of the **two developer ZIPs** arrive, use the single
offline inspector `team-update.mjs` with approved pins in
`team-source-baseline.json`. It checks installed source bytes versus
Skills Probe `modules-lock.json` and `skills-lock.json`, compares
runtime-module fingerprints and identifies contract changes for
`[ANALYTICS]` and `[GAME_RESULTS]`. It never imports, installs,
runs, overwrites or re-exports developer modules.

From `Development/`, after separate source-execution approval:

    npm run team:update:inspect -- --team "C:\absolute\team-skills.zip" --probe "C:\absolute\skills-probe.zip"

The command prints only a compact status and writes metadata under
ignored `workspace/diagnostics/team-updates/`. A changed snapshot
requires manual source review before updating its approved pin.
Existing MCP stays a read-only telemetry consumer. No new Codex
stream, auto-update daemon, test runner or Minecraft game process.
See [team update workflow](../../../docs/system/education-team-source-updates.md).

## Test/release boundary

Read [current handoff](../../../planning/development.md) and
[local-test guide](../../../docs/validation/ai-computer-use-localtest.md).
No normal `DEV.cmd preflight`, `dev` or installer validates/enables
this input experiment. First separately approve an exact-HEAD native
build/ABI preflight, then consider a distinct scoped MC01-human/MC02-AI
Minecraft run. Preserve all profiles/worlds; fail closed on uncertain
release/identity. Tests authored in source are NOT_EXECUTED at this SHA.
