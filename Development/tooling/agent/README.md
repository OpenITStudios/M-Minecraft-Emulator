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

## Minecraft Education local-first diagnostic import (source only)

The existing MCP now offers four **read-only**, bounded diagnostics tools:
get_diagnostics_overview, list_issue_candidates, get_issue_details and
query_diagnostic_evidence. They are inactive without an explicit
--diagnostics-dir / LAZY_TESTER_DIAGNOSTICS_DIR rooted at Data/Diagnostics.
The underlying diagnostics-local.mjs importer runs only as a separately
invoked local CLI; **Codex cannot import raw files, perform arbitrary path
reads, auto-collect, upload, or delete anything** over MCP.

Every new import creates a versioned local run folder with separated
evidence/, analysis/ and a manifest.json commit marker carrying SHA-256
checksums. Partial imports remain quarantined; v1 two-file archives stay
read-only and are flagged as legacy without checksums. No automatic
cleanup or raw-event upload exists. The importer retains the normalized
JSONL archive and a compact issue index. Data from unverified Education logs is never claimed
as gameplay PASS. This is not a live Minecraft Education collector, not
a debugger connection and not part of the release. For contract, bounded
usage, quotas and future Education verification see
[local-first map diagnostics](../../../docs/system/map-diagnostics.md).

## Education Script API capture probe — offline source preparation

`education-script-probe.js` is an **explicit** two-marker Script API smoke
probe for an isolated Education test-copy behavior pack, not an installed
pack or automatic game logger. `education-log-probe.mjs` reads only an
operator-supplied absolute .log/.txt file, recognizes a tiny fixed marker
contract, and outputs sanitized counts plus SHA-256. The parser offers an **envelope-survey** that counts potential marker
wrappers without parsing or exposing their contents, followed by two
strict candidate validators. Unsupported marker wrappers are distinguished
from genuinely absent markers. None is a proven Education Content Log
format. It does not create a diagnostics run, contact Codex, enumerate
Minecraft account/world paths or advertise an MCP tool. Authored fixtures
are NOT_EXECUTED. See `docs/system/map-diagnostics.md`.

## Disposable Minecraft Education Script API probe pack stager

`prepare-education-probe.mjs` is an **explicit, offline, source-only**
CLI that generates a self-contained `manifest.json` + `scripts/main.js`
behavior pack from the single canonical `education-script-probe.js`
emitter. It requires an explicitly selected stable `@minecraft/server`
dependency version, assigns unique run/pack UUID identities, and puts
source output **only** below ignored `workspace/diagnostics/education-probe/`.
It does not install/activate a pack, locate a Minecraft account/world,
launch a game, connect a debugger, run tests, or contact Codex. The
receipt is a local source artifact, not proof of Education compatibility.
See `docs/system/map-diagnostics.md` for the invocation and limits.

## One reusable Education .mcpack for generic QA (source-only)

`education-universal-pack/` holds the **single fixed-identity** lightweight
behavior pack, separate from the disposable two-marker transport probe.
Its `manifest.json` pins the exact pack UUIDs, version 1.1.0, min engine
1.26.30 and stable `@minecraft/server` 2.8.0 (the user-supplied Skills
Probe has a matching Education 26.32 macOS test manifest; Windows 26.33
is NOT_VERIFIED). Its `scripts/main.js` is an original tiny read-only
counter collector; it does not import the developer's full skill system.
Collection is OFF by default, controlled only by namespaced
`/scriptevent lazytester:diagnostics start|flush|status|stop`, with
optional short-lived `combat on|off` and `entities on|off` commands.
Following the team analytics tier policy, only lifecycle subscriptions
are enabled on start; high-frequency sources require explicit focus and
auto-expire after 600 game ticks.
No event-by-event dumping, player names, coordinates, chat, secrets,
world/inventory writes or per-tick entity scans.

`build-education-universal-pack.mjs prepare` produces one reusable
`.mcpack` in ignored `workspace/diagnostics/education-universal/v1.1/`.
No per-map UUIDs, no direct game install, no automatic Codex/MCP log
stream and no account/world access. Re-running refuses a mismatched
existing archive. Import once, then activate on each **test-copy**
Education world through Behavior Packs. See `docs/system/map-diagnostics.md`.
No source tests or Education runtime executions were performed for this
commit; any manually distributed artifact is structurally verified only.

## Developer team telemetry compatibility

The canonical contracts are the **operator-supplied** Bedrock Agent Skills
modules: `[ANALYTICS]` (general mechanics) and `[GAME_RESULTS]`
(scoring). Skills Probe provides the real Education 26.32 macOS debugger
capture evidence; it is not target Windows 26.33 acceptance. Lazy Tester
consumes both existing markers and the distinct optional `[LT_EDU_DIAG]`
marker, with **24 KiB team wire-record limits**, no raw player-record ID
in projected MCP timelines, and recognized `export_summary` failure
counts. None of this adds background AI calls.
See `docs/system/education-team-telemetry-compat.md`.

## Test/release boundary

Read [current handoff](../../../planning/development.md) and
[local-test guide](../../../docs/validation/ai-computer-use-localtest.md).
No normal `DEV.cmd preflight`, `dev` or installer validates/enables
this input experiment. First separately approve an exact-HEAD native
build/ABI preflight, then consider a distinct scoped MC01-human/MC02-AI
Minecraft run. Preserve all profiles/worlds; fail closed on uncertain
release/identity. Tests authored in source are NOT_EXECUTED at this SHA.
