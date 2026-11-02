# Lazy Tester — optional local AI/MCP Computer Use bridge

**Status:** source-only experimental per-client AI input; Windows/Minecraft
runtime **NOT_VERIFIED**. This README is the active short navigation,
not an installed/tested AI controller. Follow the
[controlled Windows local-test guide](../../../docs/validation/ai-computer-use-localtest.md)
for the future approved verification. Earlier phases and detailed source
notes are preserved in [history](../../../planning/history/mcp-agent-implementation-notes-through-2026-11-01.md).

## One-entry flow selection

| Task | Existing route | Runtime authority |
| --- | --- | --- |
| Human-controlled MC01–MC05 desktop | Lazy Tester UI -> original Rust session controller | Minecraft owns login/world |
| AI inspection / optional target and capture | `mcp.mjs` stdio -> same-image CLI/native owner | Identity/capture only, not gameplay acceptance |
| Foundation QA | `probe_qa` -> approved original Skills Probe ZIP -> stable log cursor -> bounded result | Actual admin Player command and world origin are NOT_VERIFIED |
| Map-specific requirements | Optional map JSON -> `map-test-plan.mjs` -> `probe_qa.mapPlan` | Source-only plan; all map requirements NOT_TESTED |
| Team source update | `team:update:inspect` offline -> official lockfile review | No automatic module sync |
| Evidence investigation | Explicit local import -> `Data/Diagnostics` -> read-only MCP | No auto capture, no game control |

Follow the [single architecture flow map](../../../docs/system/architecture.md#canonical-end-to-end-flow-routing)
and [current concise handoff](../../../planning/development.md).
No second controller, test runner, log collector, command dispatch or
CI is introduced. The source-only QA `proofGate` is never an
authorization to send gameplay input.

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

## Minimal activity status contract (read-only, MCP-local)

The existing `agent_capabilities` tool now includes a compact `activity`
projection from the **same** MCP process's in-memory Probe checkpoints, local
visual sampler and experimental input/recovery markers. It does not add an
MCP endpoint, watcher, agent daemon, log reader, Windows process inspector,
game command or controller. See `activity-status.mjs` for the owned
allowlisted enum projection and `activity-status.test.mjs` for authored
source-only scenarios (NOT_EXECUTED).

- `activity.displayHint` is `HIDDEN_NO_LOCAL_SIGNAL`, `ACTIVITY`, or
  `ATTENTION`. No default banner or alert is inferred from an idle MCP.
- `items` include **only relevant slots**: local visual sampling (not
  AI control), a Probe checkpoint/limited original report (not a verified
  game/map test), and opted-in input requests or recovery holds (not proof
  Minecraft consumed keys). The source and reported outcome are allowlisted.
- `desktopControl=NOT_OBSERVED_NO_AUTHENTICATED_DESKTOP_PROVIDER` means
  **unknown external Windows control**, not that nobody controls the PC.
- `snapshotAt` timestamps this MCP read only, NOT the age/identity of any
  native game frame or QA world. Disconnecting/restarting MCP invalidates the
  UI's ability to treat prior activity as live.
- A terminal original Probe PASS is labelled
  `PROBE_REPORT_AVAILABLE` and `PROBE_REPORTED_PASS`; map checks remain
  `NOT_TESTED`. An active sampler is not gameplay readiness.
- A **completed** original Probe result may optionally include compact
  `probeCounts={expected,pass,fail,info,look}` from the same already-frozen
  `probe_qa` terminal report. Numbers must be integral, bounded by the
  original `MAX_AUTO_PROBES`, and sum exactly to the expected count.
  Reported PASS/FAIL/NEEDS_REVIEW categories must be consistent with their
  counts. Incomplete/blocked results, mismatches and unknown records omit
  counts rather than fabricating progress. No new ZIP, log or window reads.
- No raw log, PID/HWND, username, account path, source command, private
  lease ID, image bytes or world identity crosses this compact projection.

**Desktop integration gate:** Lazy Tester's Tauri UI does NOT currently have
an authenticated read-only MCP-status bridge. Therefore this source change
does **not** show AI/QA badges, an activity panel, or a Stop button. A future
authorized bridge must bind the exact same MCP session and fail to
"status unavailable" on connection loss; it must not treat missing signals
as verified idle or synthesize Windows control from native game input.
Preserve the existing five fixed Minecraft previews and style.

## Explicit desktop activity handoff (source only)

The already-existing MCP `agent_capabilities` tool can now be called with
`{"publishToDesktop":true}` to send **only** its current bounded activity
projection to the already-retained Lazy Tester controller through the
same-image `--lazy-tester-agent-cli report-agent-activity` command.
Default `agent_capabilities({})` remains read-only and does not contact
the native controller, create a window, spawn a watcher or poll any log.
This is an explicit optional **status mirror**, never a gameplay action.

Admission requires `--exe` naming the matching Lazy Tester executable,
an already-live original controller and its authenticated same-image
native pipe. No new controller is created. The Rust owner rejects unknown
status categories, unbounded items, per-profile duplicates, private detail,
claims of PC control or map PASS. The Tauri webpage is explicitly forbidden
to publish activity into the controller. A same-image CLI process is **not**
cryptographic proof of the parent Node MCP agent, and the GUI only says
"Reported by local agent"; no verified third-party Windows control is inferred.

When a completed original Probe report is explicitly mirrored, clicking
the existing **Testing · Results available · MC0x** header disclosure shows
a read-only **Testing Summary** in that same panel: target, reported Probe
result, reported check counts when validated, `Map Testing: Not tested`
and `Verification: Not verified`. A reported FAIL or NEEDS_REVIEW has
its own attention label and opens the same summary. Incomplete/blocked
reports do NOT use "Results available" and never display completed-run
counts. No new page, permanent results archive, extra MCP tool, in-game
testing workflow or native game control is introduced. UI terminology stays
English; `probe_qa` and other internal identifiers are unchanged.

The controller uses a monotonic 12-second freshness limit. Expired normal
reports disappear; expired **ATTENTION** reports become an explicit
`stale=true` status-unavailable warning until a fresh bounded report
arrives. This mirror is not a persistent QA result store or an input lease
owner, and is not sampled automatically. The original read-only desktop
Inspect response may include the optional `agentActivity` field. The Play
header shows a single unobtrusive disclosure only while relevant activity or
a prior unconfirmed issue exists. Five Minecraft live previews and native
game/window operation paths remain unchanged.

**Proof boundary:** Authored Rust, MCP and synthetic browser fixtures are
NOT_EXECUTED. The local native transport, actual UI disclosure, Windows PC
control, Skills Probe source/world attribution, and gameplay are NOT_VERIFIED.
An opt-in status report is not evidence of native AI movement or successful
Probe command execution.

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

## AI-friendly one-entry Testing and result workflow

The preferred AI operation remains the **original** `probe_qa`. An agent
may now request `probe_qa({slot:2,publishToDesktop:true})` to prepare or
read the same per-slot source-only checkpoint **and** explicitly mirror
its existing status to the Play header in one MCP call. Further calls with
`restart` omitted reuse the same run. Setting `restart:true` **always**
requests a fresh checkpoint and must not be repeated casually. This is not a new tool, controller, QA runner, log
collector, archive, or automatic game command.

- The response retains the original `probe_qa` fields (the source of
  truth) and adds one `operatorNextAction` code and
  `desktopDelivery`. The latter is only receipt of a short-lived
  `agent_capabilities.activity` projection, **not** proof the UI was
  viewed or a particular Minecraft run completed.
- `publishToDesktop:false` gives the same operator hint without native
  mirroring; omitting the flag preserves the original response exactly.
  The AI no longer needs to call `agent_capabilities` separately solely
  to update Testing activity. It may still call that tool for general
  agent/capture information.
- Preparation remains `AWAIT_AUTHORIZED_PLAYER_COMMAND_AND_QA_WORLD`:
  no command is sent, and no world/player authority is invented. The
  agent must not repeatedly restart or treat `WAITING` as verified
  in-game execution; it may poll the same `probe_qa` checkpoint only
  when there is new reason to expect evidence.
- The original frozen `terminalReport` remains the **one** Testing
  outcome in MCP memory. Re-reading a terminal response does not run
  Skills Probe again. Any optional `qaReport` is a map-scope projection
  inside that same response, not a second PASS/FAIL owner. The UI's
  `Testing Summary` reads this same report-derived status/counts only.
- A missing/preflight-blocked checkpoint does not publish a misleading
  older Testing state. Status publication failures are reported as
  `desktopDelivery=STATUS_DELIVERY_UNAVAILABLE` while the canonical
  Probe result remains readable. All user-facing UI labels stay English.
- **Scope of deduplication:** one frozen terminal result per Minecraft slot
  is guaranteed only inside the SAME MCP stdio process. Multiple independent
  MCP processes currently cannot prove global uniqueness or coordinate
  one shared report. Use one designated MCP Testing coordinator per app
  session; do not run independent competing Probe owners for the same
  slot. Native game/input ownership remains singular, but it is NOT a
  substitute for an authenticated cross-process Testing-run identity.

**Still unverified:** The native input experiment is disabled in regular
builds; actual Player/admin command delivery, game/world-bound log
attribution, Minecraft movement and full map acceptance have not been
tested. Removing those trust gates is not a UI-efficiency improvement.
The desktop status mirror expires after 12 seconds and does not become
a persistent result archive. No Windows execution, app build or CI is
authorized by this source change.

## Gameplay Probe AI QA — no repeated ZIP work

**One-time configuration:** point the existing MCP at the approved
Skills Probe ZIP and per-profile capture directory:

    LAZY_TESTER_PROBE_ZIP=C:\absolute\skills-probe-main.zip
    LAZY_TESTER_EVIDENCE_DIR=C:\absolute\existing-captures

AI only needs `probe_qa` (MC02 by default). On first use, the original
ZIP is checked against the approved baseline and its probe catalog
is cached **in MCP memory**. While unchanged, future QA requests and
result reads never reopen, hash or reparse that ZIP or baseline.
Instead, a tiny file-metadata check detects when the ZIP/baseline
changes. Only then does the next preparation perform a fresh,
approved-source review. Simultaneous client preparations share the
same in-flight review; unapproved versions remain blocked.

P2 source-only proof fencing: the same first `probe_qa` response now
includes one compact `proofGate`. It distinguishes an approved source
ZIP, a stable **local** capture file checkpoint and an optional
read-only native target *at preparation* from missing runtime proof:
Player/admin command delivery, exact isolated QA world and a capture
source tied to that process/world. The gate always reports
`readiness: BLOCKED` and `automaticDispatch: DISABLED`.
Even a completed original Probe report never changes that gate into
Minecraft gameplay acceptance. The frozen terminal snapshot preserves
the historical gate without repeated I/O.

The original runner's RESULT markers include `build=dev|release`.
Interleaved dev/release markers are now `INCOMPLETE_MIXED_PROBE_BUILDS`,
not an apparent all-PASS. Distinct MC slots with the same available
local capture-file identity are refused with
`CAPTURE_SLOT_COLLISION` before creating a second QA plan.
Neither check proves the origin of a Content Log; filenames, hashes,
PID/HWND, timestamps, operator claims and appended marker bytes do
not independently certify a world or admin Player.

The first QA call pins an existing capture cursor and may return an
exact read-only game-window target. Its short response distinguishes
`sourceCapture: APPROVED_SOURCE_LOG_FILE_PRESENT` from
`gameExecution: NOT_VERIFIED`; status is still BLOCKED because
command dispatch is not enabled. File presence does **not** prove a
live Content Log stream, installed Probe, authenticated world or
Player/admin. The official `/scriptevent probe:run auto` still needs
a separately approved in-game Player/admin Computer Use channel and
disposable QA world.

While the run is in progress, repeated `probe_qa` calls inspect only
newly appended log bytes. After a complete Probe result, unrecoverable
runner-marker mismatch, lost/replaced capture boundary, target
replacement or source change, the **same per-slot MCP session**
stores one terminal snapshot. Transient concurrent file reads can
still be retried rather than falsely finalized. Further
calls return it without ZIP, log or native reads. A still-pending run
expires after 20 minutes and becomes a terminal BLOCKED snapshot,
**not** a new run. The terminal report remains a historical source
snapshot and never certifies Minecraft map gameplay.

To deliberately begin another run, use `probe_qa` with
`{ "restart": true }`. No session ID, new wizard or extra tool is
required. Snapshots are bounded **in memory of the current MCP process**:
restarting the MCP process does not restore them, and this is not a
persistent QA report archive.

The original `prepare_probe_qa`/`read_probe_qa_result` calls remain
for backward compatibility. Probe output and Minecraft gameplay
acceptance are explicitly different; log identity alone is NOT proof
of the world/player or of successful native input.

**Runtime boundary:** no new daemon, watcher, background loop, ZIP
updater, capture collector, native input, in-game module, CI or
world/account mutation. Tests authored in source remain NOT_EXECUTED.

## Map-Aware Probe QA — P1 planning / P2 proof / P3 report (source-only)

Optionally configure one map-owned JSON contract with
`LAZY_TESTER_MAP_CONTRACT` or `--map-contract`. The same `probe_qa`
entrypoint accepts `scope=full|smoke|affected`, with FULL by default.
Its initial response includes a compact `mapPlan` for the chosen
slot when configured. The plan does **not** run gameplay, dispatch
commands or turn team Skills Probe PASS into a map PASS.
`scope=affected` currently falls back to full because the MCP does
not verify the map repository diff; no guessed change list can omit
required tests. Only explicit `restart=true` changes the session's
contract/scope checkpoint. Completed result snapshots remain frozen
without additional contract checks. All tests are source-authored but
NOT_EXECUTED.

P3 keeps one QA entry. When a map contract is configured, `mapPlan`
now includes up to 12 full, bounded check summaries (given/action/
observe/expect/evidence), with every check `NOT_TESTED`. It can
retain an exact local `contractSha256` byte fingerprint (source
identity only; not world or author approval). Its
`qaReport` separates the original foundation Probe result from
map test coverage and missing evidence. Even if the original Probe
reports all PASS, `qaReport.map` remains 0 executed/0 passed and
`qaReport.acceptance=NOT_ACCEPTED` until real map-specific tests
and world proof exist. The report now shares the original Probe
catalog's count limit (rather than an undefined local constant),
includes bounded status-code reasons, and separates mandatory
selected/skipped map checks. These are source-only integrity fixes,
not proof that the tests passed. The map plan/report are also available during
an original ZIP or capture PREFLIGHT refusal, so planning does not
require Minecraft. Repeated terminal MCP calls reuse the frozen
report without more file or native reads. Ordinary calls with no
map contract retain their previous shape.

Contract schema, safe selection rules and proof limits:
[Map-Aware Probe QA](../../../docs/system/map-aware-probe-qa.md).
The map must own its requirements; no map module code or world content
is copied into Lazy Tester.

## Test/release boundary

Read [current handoff](../../../planning/development.md) and
[local-test guide](../../../docs/validation/ai-computer-use-localtest.md).
No normal `DEV.cmd preflight`, `dev` or installer validates/enables
this input experiment. First separately approve an exact-HEAD native
build/ABI preflight, then consider a distinct scoped MC01-human/MC02-AI
Minecraft run. Preserve all profiles/worlds; fail closed on uncertain
release/identity. Tests authored in source are NOT_EXECUTED at this SHA.
