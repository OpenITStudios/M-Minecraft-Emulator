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

## P39 — Raw Input expiry recheck after Windows foreground calls (SOURCE_ONLY)

The existing optional C++ `GetRawInputBuffer` reader maintains
`rawHeldMask` and advances its per-button cursor only after records
were copied to the caller. A game that stopped polling cannot be made
to consume key-up/mouse-up merely by changing native shared memory.

The source-level gap: `currentPacket()` checked the 500-ms lifetime
before calling the real Windows foreground API, but subsequently
checked only the ABI revision. A delayed foreground check could return
an active packet that had already expired without any revision change.
Other Win32 keyboard/focus/cursor hooks could also cross expiry after
their own foreground queries.

The existing C++ owner now checks a bounded unsigned modular
GetTickCount deadline at the final read point. The Raw Input hook
treats an active packet that expired during its foreground sampling
as **zero** before composing the buffer plan. Previously delivered
down events remain pending in `rawHeldMask` and can be returned as
up events on that resumed poll; partial-buffer reads still advance
cursors only for copied records. The existing game-window thread,
exact v4 revision, physical-input suppression and verified HWND
guards remain unchanged. Compile-time `static_assert` negative
fixtures cover expiry at 0/1/500 ms, over-horizon, zero packet and
32-bit tick-counter rollover. AUTHORED_NOT_EXECUTED.

**Not proven:** a paused Minecraft client may never poll; Raw Input
events are not injected by this change. Minecraft Education's actual
GetRawInputBuffer usage and acceptance of any key-up/mouse-up remain
NOT_VERIFIED. No v4 ABI, Rust controller, release protocol, MCP tool,
installer, game profile, feature enablement or CI change occurred.

## P38 — Legacy packet TTL versus native neutral acknowledgement (SOURCE_ONLY)

The retained Rust shared-memory input channel restricts an accepted
active MC02 packet to at most **500 ms**; the optional C++ Bedrock
Win32 reader refuses an expired word and releases virtual keys when
polled. That TTL controls the **input packet lifetime**, not whether
Minecraft has actually consumed key-up, nor whether the original
native controller has confirmed a neutral-state write.

An original compatibility gap remained after P37: the modern
`AiLeaseBook` retained unacknowledged non-neutral 4/5-client claims,
but legacy MC02 writes only set `legacy_until_ms = now+500`.
At expiry, a modern 4/5-AI writer could claim MC02 even if a legacy
active packet had been published without any neutral ACK.

The original **single native `AiLeaseBook`** now retains one bounded
`legacy_neutral_pending` flag per original slot. A legacy input
begins with **possibly active** before channel I/O; a successful
active write keeps it pending. Only a confirmed native exact-process
neutral write or confirmed original-process reap clears it. Expiry
still stops polling and the original monotonic 500-ms watermark
still rejects delayed packets, but expiry **never clears the
neutral-confirmation obligation**. The modern atomic group-acquire
refuses an unresolved legacy MC02 claim even after the 500-ms
packet deadline; unrelated original client slots can be acquired
and released independently. Legacy MC02 can still explicitly write
a neutral packet on its existing authenticated channel.

This is the **native transport ACK**, not proof of Minecraft Raw
Input consumption, Win32 game input, or a completed gameplay action.
No change to the 96-byte shared-memory ABI, client C++ reader,
lease epochs, game lifecycle, default OFF feature flags, MCP
contract, installer, profile state or GitHub Actions. Existing
Rust fixtures that previously inferred safety at +500ms now
require explicit neutral ACK; additional negative 4/5-client
fixtures cover lost active/neutral outcomes, healthy peers and
confirmed process reap. Regression fixtures are
AUTHORED_NOT_EXECUTED; Windows/Minecraft and five-client
acceptance remain NOT_VERIFIED.

## P37 — Expired writer lease never proves neutral input (SOURCE_ONLY)

A source audit of the retained Rust `AiLeaseBook` found that its
original normal acquisition logic treated an expired 10-second
writer claim as freely replaceable and initialized the new
`neutral` bit from elapsed time. An input that was possibly
published but never received a native neutral ACK could therefore
be replaced by a *normal active* writer without the operator's
explicit neutral-recovery action. The separate legacy MC02
admission also ignored the active/non-neutral bit after expiry.

The original **single native lease book** now separates the
authority to **write** (which expires normally) from evidence of
**native neutral acknowledgement** (which never follows from time).
An expired claim whose last write was not ACKed neutral blocks
normal active acquisition for that slot, including for the
same writer. The recovery-only acquisition is still permitted
after the old claim's expiry, or for the same current writer
under the existing ownership rules. Its new claim always
starts `neutral=false`; the original controller must complete
a **fresh exact-PID/birth/epoch neutral write** before releasing
the claim. A normal claim with existing confirmed neutral state
can still be replaced after expiry, and independent healthy slots
are unaffected. Confirmed original process reap is still the
only path that clears a dead child's claim without writing into
an adopted replacement. The legacy MC02 input gate also refuses
expired **non-neutral** AI claims until that recovery completes.

This is a bounded native IPC **ACK**, not a claim that Minecraft
consumed key-up or that a game/window is playable. The expiry
limit still fences delayed input through `require()`.
No CLI schema, MCP owner, UI, driver, autonomous loop, alternate
controller, profile files, installer or feature enablement changed.
New pure Rust negative fixtures cover four- and five-AI atomic
refusal, healthy peers, cross-writer and same-writer expiry,
recovery-only admission, stale epoch denial, missing neutral
ACK, successful neutral/release and confirmed process reap.
Previous fixtures relying on implicit neutral state now model
explicit neutral ACK for each recovered slot.
Fixtures AUTHORED_NOT_EXECUTED; Windows, source suites and Minecraft
acceptance NOT_VERIFIED.

## P36 — Definitive pre-mutation epoch refusal versus uncertain IPC

The retained Rust `ai_lease_acquire` checks the temporal lease epoch
against the selected original process births **before** calling
`AiLeaseBook.acquire`. If the epoch is stale or predates a relaunched
child, that exact `PermissionDenied` cannot have acquired a writer
or dispatched input. Previously the CLI projected this proven
no-write refusal as generic `NATIVE_REFUSED`, which MCP deliberately
treated as uncertain and quarantined otherwise healthy selected
clients even though no native lease mutation occurred.

The same native CLI now maps **only** the exact original pre-admission
error message to `NATIVE_WRITER_EPOCH_STALE`, accepting either the direct
Rust `PermissionDenied` or the `ErrorKind::Other` form produced when
the authenticated `SessionReply` converts its error string through
`native_clients_session::request`. The
existing MCP acquire-denial classifier accepts that fixed code along
with known native writer conflicts and launch-mode mismatches.
An ambiguous/missing IPC result (`OUTCOME_UNCERTAIN`), timeout,
broken pipe, generic refusal or malformed receipt **does not**
qualify for definitive denial and still quarantines the entire
selected action cohort. Other clients outside the attempted cohort
remain independent, while a rejected startup neutral-recovery
acquire never clears any pre-existing reconnect hold.

The private CLI response uses the same existing allowlisted code
field; no new controller, wire schema, game channel, release
shortcut, automatic retry or status database was introduced.
Source-authored Rust/Node fixtures cover the exact-code versus
transport-uncertain distinction in both 4-AI and 5-AI modes,
continued explicit healthy-peer input, and refusal to clear
startup holds without neutral/release ACKs. All are
AUTHORED_NOT_EXECUTED; no Windows/Minecraft acceptance is implied.
## P32 — MCP disconnect fences late input (source-only)

The existing optional MCP stdio handler previously aborted a running
multi-client batch during `shutdown()` but did not refuse a late
continuation after an awaited native lease acquire or observed-action
identity preflight. A pending acquire ACK could therefore still start
a new AI action with an already-aborted transaction; a completed native
target read could likewise proceed after stdin closed.

One existing per-handler `closed` flag now fences newly received tools
and these two asynchronous admission boundaries. A late lease acquire
ACK is **not** authorization for active input: its already-confirmed,
still-neutral lease enters existing `finally` exact-owner release
without pulse dispatch. In-flight active transport is aborted and
its original selected cohort remains uncertain, while existing
per-slot neutral/release cleanup can still run. A stale preflight
returns `MCP_CONNECTION_CLOSED` before acquiring a writer lease;
the stdio owner suppresses late replies after closure.

There is **no** extra MCP/server instance, agent daemon, retry,
unverified input, process adoption, status store, or changed native
wire schema. Reconnecting MCP still starts with all opted-in AI slots
held pending explicit original-controller reconciliation. Negative
Node fixtures cover disconnect during an awaited acquire, visual
preflight, already-dispatched batch, and post-close tool admission
in four/five-client modes. Tests are AUTHORED_NOT_EXECUTED;
real Windows/Minecraft operation, source suites and five-client
acceptance remain NOT_VERIFIED.
## P31 — Epoch-bound native process generation (SOURCE_ONLY)

An existing writer lease already binds subsequent `AiLeasedInput`,
`AiInputGroup` and `AiLeaseRelease` to the exact original
slot/PID/process birth/epoch. However, the earlier
`AiLeaseAcquire` used a fully random 32-hex epoch and fetched each
*current* game process from the retained Rust controller at admission
time. A delayed pre-relaunch **acquire** could therefore ask for a lease
on the replacement child after the previous process was reaped, even
though that transaction was created for the earlier process.

Without changing the private IPC schema, the existing MCP now mints
a versioned **32-hex** transaction epoch: first 16 hex represent its
Windows FILETIME issue timestamp (100-ns ticks since 1601), followed
by 14 hex of fresh entropy and a 2-hex `e1` marker. It is still only
an ephemeral writer correlator, **not** OS authentication. The original
Rust actor checks the epoch against the birth FILETIME of **every**
selected retained child before any group lease mutation: issue time
must be after child creation, not more than 10 seconds old and not
more than 1 second ahead of the native wall clock. Time drift or
unversioned/stale epochs fail closed; a fresh separate transaction is
required for a genuinely new game child.

This is **transaction-time fencing**, not a signed process identity.
No independent server, session owner, network endpoint, game hook,
new resource pack, or CLI/IPC JSON field was introduced. Native
PID/birth/epoch checks on later input/release remain unchanged.
Synthetic Rust and Node fixtures cover a replaced child in 4/5-AI
cohorts, malformed/stale/future epochs, fresh admission, and stale
input/release refusal. Fixtures are AUTHORED_NOT_EXECUTED; Windows
clock alignment, Minecraft input acceptance, real process
replacement, Node/Rust compilation and five-client behavior
remain NOT_VERIFIED. Native feature flags stay compile-OFF by default.

## P30 — Per-slot release after a native neutral ACK (SOURCE_ONLY)

The original Rust controller validates `AiLeaseRelease` atomically for
the supplied group. Previously, MCP issued individual neutral writes,
then released all ACKed slots in one native group. A child could exit
or change PID/birth between its neutral ACK and group release. That
would reject the whole release even when peer channels were neutral.

MCP now **retains atomic group acquisition**, independent exact-slot
neutral writes, and then attempts one **serialized, non-retried**
exact-owner/epoch lease release for every acknowledged slot. Only
slots with their own confirmed release ACK clear their recovery hold.
One child refusing a release does not prevent release attempts for
the others. A failed/ambiguous release remains uncertain and is
never redirected to a different process. Uncertain action delivery
still quarantines its original full action cohort; healthy peers are
eligible only for a separately explicit next action.

Authored Node fixtures cover both hybrid (4-AI) and full (5-AI)
modes, departure after neutral ACK, partial explicit operator stop,
quarantine, peer continuation and fresh fencing epochs.
**AUTHORED_NOT_EXECUTED:** no source tests, Windows, Minecraft,
additional controller, plugin runtime, CI or installer were run.
The native wire schema, compiled feature defaults, and original
resource-pack and profile data remain unchanged.

## Shared MC02 legacy → multi-AI lease fence (source-only P29)

The original Rust `AiLeaseBook` arbitrates both the historical MC02-only
input pilot and later feature-gated 4/5-client AI writers. The legacy
`Mc02Input` call previously set its short compatibility hold only
*after* a successful native write. Since a native I/O error can arrive
after a shared packet was already published, a later multi-client
writer could otherwise acquire MC02 before the original channel's
maximum 500-ms input TTL expired.

The same retained native actor now marks **MC02 possibly active before**
calling the existing `mc02_input` operation. Failure retains the
bounded compatibility fence. A subsequent successful exact-owner
neutral write clears it; confirmed process reap uses the existing
`forget_slot` path. The other slots remain independent. This is
source-only protection, not an assertion that Minecraft accepted input,
or that legacy and modern writers should run together. Both opt-in
native features stay compile-OFF and unavailable in normal builds.
A focused Rust fixture is AUTHORED_NOT_EXECUTED; no Node, Rust, Windows,
gameplay, CI, or external agent was run.

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
background work, telemetry or gameplay writes. The same static
help listing now includes Minecraft Education's native Show Coordinates
setting and the conditional Bedrock-only Ctrl+Alt+C guidance. It does not
create an in-game Save button, read clipboard, override the native coordinate
HUD, or claim Education 26.33 Copy Coordinate UI parity.

Minecraft command (operator/cheats in a copied QA world):

    /function help

This runs one canonical team-command list via `tellraw @s`, mirroring
the static `.mcfunction` convention used by Skills Probe.
When a map owns a conflicting `help` function, the fallback is
`/function lazytester/help`. Minecraft's native `/help` command
remains untouched.

The offline `build-education-team-help-pack.mjs` stages one v2.2.0
.mcpack only under ignored
`workspace/diagnostics/education-team-help/v2.2/`. It does not install
a team module, perform runtime testing, touch a Minecraft world,
or use Codex. Developer exports and Skills Probe-only commands
are displayed but never executed by the helper.

**Source-only native HUD prototype:** The optional
`education-native-hud-prototype/` is an isolated resource-only pack
containing a minimal `root_panel/chat_stack`
`insert_after(player_position)` JSON UI patch. The revised v0.2
does not modify Minecraft's content-sized `player_position` image;
it can show only a static visibility-bound hint below XYZ, if accepted
by Education 26.33. It has no Script API, dynamic dimension/facing,
in-game Save/Load, or clipboard capability. See
[HUD extension contract](../../../docs/system/education-native-coordinate-hud.md).
The original team remains the sole gameplay UI logic owner.

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
  The optional coordinated+archived flow below improves cooperative
  deduplication and retrieval but does not authenticate historical world
  provenance or automatically recover abandoned native owners.

**Still unverified:** The native input experiment is disabled in regular
builds; actual Player/admin command delivery, game/world-bound log
attribution, Minecraft movement and full map acceptance have not been
tested. Removing those trust gates is not a UI-efficiency improvement.
The desktop status mirror expires after 12 seconds and does not become
a persistent result archive. No Windows execution, app build or CI is
authorized by this source change.

## Native Testing Run correlation — optional cooperative mode

The existing `probe_qa` tool accepts
`{slot:2,coordinateNative:true,publishToDesktop:true}` to ask the
**original retained native controller** for one MC02 run reservation BEFORE
preparing the original source-only Skills Probe checkpoint. The one-shot
native CLI runs in the same executable and authenticates to its original
pipe. It never starts a game, test runner or second controller.

- The controller retains **one correlation record per MC01–MC05 slot**.
  A competing cooperating MCP process receives `TESTING_RUN_ALREADY_RESERVED`
  and the existing `testingRunId`, without performing a second Probe
  preparation or publishing an independent result.
- The MCP process keeps a private 32-hex correlation owner ID. The owner ID
  goes to the same-image CLI through its child environment, not argv.
  The public run ID is **not** a password and its existence does not prove
  AI, player, world, or game-log identity. Same-user tampering remains
  outside this handshake's claimed security guarantee.
- A resulting original `probe_qa` terminal report (or blocked preflight)
  may publish a **write-once** status + SHA-256 digest to the native book.
  Repeating the same final reference is idempotent. Different owners or
  a different digest cannot overwrite it. The original MCP terminalReport
  remains the *only source-only Testing result*, not a second native grader.
- `restart:true` is required to replace a FINAL run, and only its own
  cooperating MCP process may request it. A live RESERVED run cannot be
  replaced automatically, even if its owner disappears; safe recovery
  requires an explicit future protocol. Other slots are unaffected.
- This is an **opt-in migration path**: legacy `probe_qa` without
  `coordinateNative:true` remains functional for backward compatibility,
  so global uniqueness is **not yet guaranteed against uncoordinated agents**.
  To avoid duplicates in new workflows, designate one MCP Testing
  coordinator and always request native coordination where available.
- **Not persistent:** the native book lasts only while the retained
  controller is alive. It stores an outcome/digest reference, not a complete
  retrievable report. It is NOT a durable archive across app restarts and
  cannot verify that any Minecraft command was sent. A future persistent
  archive must have a separately reviewed private location and immutable
  original-report integrity policy; it must not write to game profiles,
  Skills Probe ZIPs or unverified capture/log files.

Controller-unavailable or uncertain native receipts fail closed. The
source changes and Rust/Node regression fixtures are **NOT_EXECUTED**;
live Windows, per-slot input, player/admin command delivery, original
Probe world/source attribution and gameplay remain NOT_VERIFIED.

## Read-only native Testing run inspection and safe recovery gate

The existing **single** MCP entry supports
`probe_qa({slot:2,inspectRun:true})`. This is an explicit, read-only
inspection through the same executable's authenticated pipe to the
original retained native controller; it does not run `probe_qa` source
preparation, read game/log evidence, publish activity, or change a
reservation.

The reply contains the original controller's current **volatile**
per-slot correlation observation:

- `NO_RECORD`: the controller has no current reservation record. This
  also happens after a controller restart; it is **not** proof that no
  previous Testing run exists. Check the private `Data/Testing`
  archive (when configured) before proposing a new run.
- `RESERVED`: a run ID is present but terminal evidence is absent.
  `ownerTokenMatch` only compares the requesting MCP's random
  correlation token; it does **not** prove the original AI process is
  running or has exited. The proper recovery action is
  `REQUIRE_VERIFIED_OWNER_DEATH_AND_OPERATOR_APPROVAL`.
- `FINAL`: the controller has only a fixed terminal source-only
  status and digest. The proper action is
  `READ_FINAL_REPORT_NO_OWNER_TRANSFER`; use the canonical
  `probe_qa({archivedRunId})` route if its immutable archive exists.

Every inspection explicitly reports
`ownerProcessProof=NOT_BOUND_TO_PROCESS_HANDLE`,
`operatorApproval=NOT_AVAILABLE_IN_PROTOCOL`, and
`transferAllowed=false`. A new coordinated reservation also binds the
Windows-reported direct **Node parent** of the verified one-shot CLI to
a retained process handle, PID and creation time in the same native controller.
Further same-token reserve/finalize requests require the *same live parent*;
PID reuse and a new Node lifetime cannot finalize the old run.
The read-only inspection exposes only `BOUND_PARENT_RUNNING_AT_INSPECTION`,
`BOUND_PARENT_EXIT_CONFIRMED`, `BOUND_PARENT_STATUS_UNKNOWN`, or
`NOT_BOUND_TO_PROCESS_HANDLE` for an unbound historical/pure fixture.
Parent attribution is an OS observation, NOT cryptographic attestation of
which MCP script is running; same-user parent spoofing is out of scope.
An uncertain parent refuses new writes. Parent exit is not approval.
No private owner ID, PID, HWND, account, raw
log or world information is returned. The ordinary app/webview cannot
forge a native inspection request, and a missing retained controller is
reported as unavailable rather than created.

**Operator-approved takeover is NOT yet implemented.** A retained Windows
parent handle/birth is process-lifetime evidence, not authentication of
the MCP script, operator approval, a Minecraft world, or consent to takeover.
Token equality, arbitrary caller-reported death, timeouts, screenshots and
archive SHA alone cannot establish a safe handoff. A future transfer
must be separately reviewed against verified original owner exit,
operator authentication, the expected old run ID and immutable archive
integrity; it must use an atomic compare-and-swap inside the original
controller, and must not modify Minecraft process or world state. Until
then an abandoned `RESERVED` run remains blocked. This is a deliberate
safety constraint, not a missing automatic retry.


### Combined native + private archive review (source-only)

When `Data/Testing` was explicitly configured, the SAME
`probe_qa({slot:2,inspectRun:true})` additionally scans the original,
bounded immutable summary archive read-only, using the existing
`testing-result-store.mjs` verifier. It returns `archiveReview`:

- `NOT_CONFIGURED`: no archive read was requested;
  `NO_COMMITTED_SUMMARY_FOUND`: no committed summary was found in
  this configured root, **not** proof that no historical run ever existed.
- `HISTORICAL_SUMMARY_PRESENT`: a validated source-only result
  exists, but does not itself match the current native FINAL reference.
  Open it explicitly through `probe_qa({archivedRunId})`.
- `FINAL_REFERENCE_MATCHES_ARCHIVE`: current native FINAL ID,
  source-only status and SHA-256 digest match a committed summary;
  this is **not** world, player, or operator authentication.
- `CONFLICT_RESERVED_HAS_FINAL_ARCHIVE` or
  `CONFLICT_FINAL_REFERENCE_MISMATCH`: do not transfer or restart;
  the native and archive observations disagree.
- `INTEGRITY_UNAVAILABLE_OR_CONFLICT`: a missing, partial, corrupt,
  linked or otherwise unreadable archive must be reviewed manually.

Every result fixes `archiveReview.authorizesTransfer=false` and
the native `inspection.transferAllowed=false`. Owner alive or unknown
means wait/review. Even `BOUND_PARENT_EXIT_CONFIRMED` together with
`NO_COMMITTED_SUMMARY_FOUND` only returns
`AWAIT_SEPARATE_OPERATOR_APPROVAL_NO_TRANSFER_AVAILABLE`. No user
can approve takeover through MCP, Tauri, an environment variable,
a time threshold, an AI assertion, or an archive checksum.

**Future operator-approved CAS admission:** preserve the original
per-slot run ID and original pinned handle, require verified death,
revalidate archive integrity and the same controller epoch, obtain
independent real-operator authentication (not a model-supplied flag),
then compare-and-swap the exact RESERVED run to one fresh owner/run
inside the original retained native actor. Reject changed run IDs,
owner replacement, FINAL references, missing provenance, failed
operator authentication and every interrupted/ambiguous operation.
Do not introduce a second Testing runner or mutate Minecraft.
This design is **not** implemented as an accessible transfer action.

### Staged original-controller compare-and-swap (not callable)

The original Rust `NativeTestingRunBook` now contains a **private,
unreachable-from-production** `transfer_with_sealed_admission` branch.
It accepts only private native operator-presence and native archive-review
markers, an exact old run ID/owner, distinct new run ID/owner, a
confirmed-exited pinned old process and a live pinned successor.
The actor checks all conditions again before ONE atomic slot replacement;
old tokens cannot finalize the replaced reservation. It refuses
`FINAL`, changed run IDs, mismatched owners, unknown processes and
unsafe same-owner reuse.

These markers deliberately have **no production constructors**;
no MCP schema, Tauri command, native CLI operation, or automatic job
can issue them. The new pure Rust tests use explicitly scoped fixtures,
**AUTHORED_NOT_EXECUTED**. This is a correctness foundation, not an
available transfer workflow, approval mechanism, Windows verification,
gameplay result, or permission to take over a previous agent.

A Windows Hello operator-verification implementation could become the
separate local presence boundary after review. Microsoft documents
`IUserConsentVerifierInterop::RequestVerificationForWindowAsync` for
desktop applications; its current Win32 interop requirements include
Windows build 22000 or later. An ordinary confirmation dialog, MCP
environment flag, AI answer, or UAC elevation alone is **not** an
independently authenticated operator. Do not add an insecure fallback.
See https://learn.microsoft.com/en-us/windows/win32/api/userconsentverifierinterop/nf-userconsentverifierinterop-iuserconsentverifierinterop-requestverificationforwindowasync

The subsequent implementation must independently validate all private
`Data/Testing` manifests **inside the native controller**, bind the
original per-controller reservation to that evidence and to a fresh
operator challenge, and mint the two private markers only on successful
verification. No absence-of-archive shortcut authorizes an overwrite.
Do not expose this method or claim P0 acceptance before those gates and
exact-revision Windows verification succeed.

## Immutable original Probe summary archive (explicit opt-in)

Configure the optional absolute private `Data/Testing` root ONCE with
`--testing-archive-dir <absolute/Data/Testing>` or
`LAZY_TESTER_TESTING_ARCHIVE_DIR`. The root is independent of
`Data/Diagnostics` (which remains an explicit diagnostic import) and is
never a Minecraft profile/world directory. `Data` must already exist as
an ordinary, non-symlink directory. No archive is written by default.

When the existing `probe_qa({slot:2,coordinateNative:true})` reaches an
**original frozen terminal** Skills Probe result, and the native controller
has acknowledged its immutable FINAL status/digest, the SAME MCP process
archives a **bounded, allowlisted source-only summary** in an exclusive
per-run directory `Data/Testing/runs/<testingRunId>/`:

- `report.json` is written exclusively with original terminal outcome,
  validated Probe counts when complete, fixed NOT_TESTED map status,
  NOT_VERIFIED gameplay, original SHA-256 report digest, slot/run ID and
  an allowlisted reason where available. **No raw logs, account paths,
  player IDs, screenshots, PID/HWND, world files, credentials or arbitrary
  model text are stored.**
- `manifest.json` is committed last after a flushed report and
  `manifest.pending`. Immutable per-run directories cannot be
  overwritten or automatically deleted. Interrupted/partial writes
  remain uncommitted and fail closed; matching repeated run ID/digest is
  idempotent. The archive has a bounded 128-run quota.
- The archive is read back with pinned ordinary-file metadata and
  checksums. A checksum matches stored bytes but **does not authenticate**
  which game/world/player produced them, nor guarantee durability after
  physical disk failure or malicious same-user tampering.

AI reads historical source-only results through the SAME tool:
`probe_qa({archivedRunId:"<32-char run ID>"})`. This is strictly
read-only even if the native controller or original MCP has exited. It
does not prepare a new run or certify native ownership. The result says
`ARCHIVED_SOURCE_ONLY`, `Map Testing: NOT_TESTED`,
`gameplayAcceptance: NOT_VERIFIED`, and
`nativeRunState: NOT_VERIFIED_AFTER_ARCHIVE_READ`.

**Abandoned-run recovery policy:** no heartbeat timeout, screenshot, process
absence guess or historical digest can authorize an owner transfer.
If the original native reservation remains RESERVED but the MCP owner
is unavailable, a competing agent receives
`TESTING_RUN_ALREADY_RESERVED` and
`VERIFY_PREVIOUS_OWNER_NO_AUTOMATIC_TAKEOVER`. If the archive has a
previous committed result and a *fresh* MCP has no matching native
run ID, its first coordinated request returns
`TESTING_PRIOR_ARCHIVED_RUN_REQUIRES_RECONCILIATION` instead of
creating a new run. Missing or partially committed archive evidence
returns `TESTING_ARCHIVE_RECONCILIATION_REQUIRED` and blocks a new run.
Users can still read historical results by ID. An active/abandoned
reservation cannot be forcibly cleared via these MCP APIs; verified
native owner death, controlled operator approval and a fenced recovery
protocol remain prerequisites for any future transfer or restart.

**What is durable:** a short original Probe result summary per committed
run ID on the configured private disk.
**What remains volatile:** the controller's current reservation book and
the original MCP's complete frozen terminal report. The archive is not
a second Skills Probe runner, not an accepted map result, not automatic
Windows input and not yet a persistent UI results list. The Play
header's 12-second activity status is unchanged. Tests are authored
but NOT_EXECUTED; Windows/Minecraft runtime proof remains unavailable.

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
