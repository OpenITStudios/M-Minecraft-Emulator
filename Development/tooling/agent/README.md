# Lazy Tester — local AI/MCP bridge

**Status:** source-only optional MCP integration for up to five separately
licensed Minecraft Education Windows profiles. Default game input is
read-only/planning; independently controlled AI input is experimental,
**compile-OFF** and not verified in Minecraft. Neither an IPC ACK nor a
screenshot establishes gameplay acceptance.

**Read only what the task needs.** Current implementation ownership is in
[the repository implementation map](../../../docs/system/implementation-map.md)
and [architecture](../../../docs/system/architecture.md). The complete
[P1–P52 historical source reference](../../../planning/history/mcp-agent-source-reference-through-2026-11-05.md)
was archived without deleting the technical notes. Use the
[single current handoff](../../../planning/development.md) for status and
one next action; old archived "Next" entries are not authorization.

## Optional AI integration (Unreleased, source-only)

Lazy Tester is a standalone manual Windows application. No MCP server,
Codex subscription, local LLM, Code Mode feature, Node host or capture worker
is required for manual operation. Do not start a bridge, register tools,
install packages or change the user's host settings automatically.

### Choose whether to connect

| User workflow | Connection |
| --- | --- |
| Manual application only | No MCP connection; nothing to configure |
| AI-assisted playtest | Explicit `--surface playtest` (15 MCP tools) |
| Restricted analysis/planning | Explicit `--surface code-mode` (13 MCP tools) |
| Existing MCP client | `--surface legacy` (25 default tools; retained compatibility) |

The profile names select Lazy Tester's **tool contract**, not a host
feature switch. In particular, `--surface code-mode` is the restricted
read/plan profile, not a request to turn on Codex Code Mode. Playtest tools
can be called using ordinary MCP or composed using a host's supported Code
Mode feature. The host owns code execution, script budgets, model usage
and approvals. Users choose whether to enable that feature; no change
to host settings is made by Lazy Tester.

Development direction: Code Mode first for AI orchestration, ordinary MCP for
compatibility, and no AI requirement for manual use. The explicit Playtest
direct SDK and MCP share the same admitted operations and projection;
host-owned isolation and approvals remain mandatory.

Use one connection for the chosen workflow. The manual application
and optional MCP connection reuse the same controller. Do not run duplicate
bridges for the same task or silently widen a restricted profile.
Independent keyboard/mouse execution is still blocked; there is no
production autonomous gameplay tool or automatic close. The intended
future control contract remains Execute / Observe / Stop, not a preset
game engine. No new server, sandbox or dependency.

### Optional local connection

Only if the user chooses to connect an AI host, configure an explicit
stdio MCP entry. The following is an illustrative PC-local Codex entry,
not an installed configuration. Replace paths with reviewed absolute
local paths, and retain host-controlled approval settings:

```toml
[mcp_servers.lazy_tester]
command = 'C:\Tools\node.exe'
args = ['D:\Source\M-Minecraft-Emulator\Development\tooling\agent\mcp.mjs', '--surface', 'playtest', '--mode', 'human-plus-four-ai', '--exe', 'D:\LazyTester\App\lazy-tester.exe']
default_tools_approval_mode = 'prompt'
```

For visual observation, add `--gpu-capture-exe` and the reviewed absolute
path to the matching WebP-capable WGC worker; without it, Playtest image
requests refuse explicitly. Configure other capture, evidence, diagnostic,
Probe or map-contract paths only if needed. Native app capture approval
remains held. No automatic startup, credentials, sign-in, world join,
map import or command dispatch.

First call `agent_capabilities({inspectProfiles:true})` when an actual
native controller is configured. A configured source is not proof that
its provider works. Code Mode may be enabled separately in the host
if that host supports it; ordinary MCP still uses the same catalog.
To stop using the integration, stop the optional sampler if active and
disconnect/remove or disable the MCP entry in the host. Closing the
bridge ends its local observer and in-memory checkpoints, not the
manual game lifecycle. No configuration has been installed or tested
on a user PC.

### One product catalog, existing operations only

| Task | Public playtest tool |
| --- | --- |
| Discover sources and inspect profiles in one optional preflight | agent_capabilities (inspectProfiles=true) |
| Inspect / explicitly launch an AI-owned profile | inspect_profiles, launch_profile |
| Observe on demand / semantic UI | observe_client, inspect_client_ui |
| Optional bounded monitoring | start_visual_observation, get_multi_client_visual_state, read_visual_observation, stop_visual_observation |
| Prepare/review original Probe, explicit restart or read historical result | probe_qa |
| Read bounded analytics / game results for one or several slots | summarize_test_evidence |
| Query imported issue evidence when needed | get_diagnostics_overview, list_issue_candidates, get_issue_details, query_diagnostic_evidence |

All 15 names are existing operations with existing native admission.
`read_game_evidence` is no longer advertised or admitted in playtest: use
`summarize_test_evidence({slots:[slot]})` for the same bounded evidence source.
Legacy and restricted Code Mode retain the old single-slot method.
The same descriptor catalog and projection serve both standard MCP and
Codex Code Mode on this connection. The independently trusted Node SDK
below offers explicit Playtest coverage; its default remains read/plan-only.
The 13-tool restricted profile is retained (including compact input previews). Legacy also accepts
optional inspectProfiles=true; publication and combined inspection cannot
both be requested in the same call. No public
planning, duplicate context, old Probe aliases, low-level HWND targeting,
screen-text review, native reservation, status publication or input
experiment in playtest. Experimental core attachment is refused.
MC01 stays protected in hybrid mode. `five-ai` is a slot policy, not proof
of autonomous play.

### Minimal playtest / report workflow

1. For a new playtest, call `agent_capabilities({inspectProfiles:true})` once.
   This combines capabilities/configuration and the existing fresh Inspect
   receipt without a second MCP request, extra native query, log read or
   launch. If native Inspect refuses, the combined request fails; use the
   no-argument capability read for offline review. Use `inspect_profiles`
   alone for later status refreshes. Launch only explicitly selected AI slots
   with the existing launch operation; reuse its returned roster. Code Mode
   may compose multiple explicit single-slot launches serially, stopping on
   refusal or uncertain outcome; no server-side launch batch or retries.
   A launch receipt is not a game/world/login receipt.
2. For a quick look, call `observe_client({slot:2})`: one WGC WebP
   preview, 640x360 by default, no sampler startup or separate read.
   Override both width and height only as needed (each 160–1536); e.g.
   320x180 for a glance, 960x540 for detail. Same worker, direct WebP
   lossless encoding of fitted pixels; no PNG intermediary or new codec.
   Save the returned frameRef in Code Mode and pass it as afterFrame on a
   later one-shot call to suppress identical encoded images. This still
   captures and verifies a fresh frame. A different target, size or hash
   returns a new image; omitting afterFrame explicitly requests the image.
   If repeated monitoring is useful, start its bounded WebP sampler with
   the chosen size; use metadata first and read with afterSequence to avoid
   sending unchanged images. afterSequence is scoped to that running sampler;
   omit it after restart. Stop before changing sampler resolution or
   starting a separate one-shot on that slot. No automatic HD escalation.
   An image hash is not gameplay proof. WebP is required for Playtest image
   routes; if no reviewed optional WGC worker is configured, return
   WEBP_CAPTURE_REQUIRES_WGC rather than silently use CUA PNG. CUA remains
   semantic/no-image inspection and Legacy PNG compatibility.
3. Actual gameplay input is BLOCKED_NO_VERIFIED_INPUT_BACKEND. A human can
   play using the normal app; do not turn a planned action into executed
   evidence or enable experiments to fill the gap. Close remains manual
   until the full desktop recording/window release admission can be reused.
   Native product capture/recording approval is still held; the optional
   WGC source requires a separately reviewed matching worker and host;
   CUA PNG compatibility and WGC are not runtime verified.
4. Before gameplay, call `summarize_test_evidence({slots:[2],window:'begin'})`
   to record a bounded local capture baseline for the selected slots. After
   play, call the same tool with `window:'read'`. Reuses the existing pinned
   file cursor: baseline reads at most 128 bytes, then reads bounded appended
   bytes plus the original anchor. No native query, polling or new tool.
   Explicit begin replaces only the selected in-memory baselines; failed
   begin clears them. Read never silently falls back to historical log tails.
   Replacement, truncation, changed anchor or missing baseline reports
   unavailable evidence. Definitive continuity failure invalidates that slot's
   baseline; only a new explicit begin can establish another one. At most
   20 records/slot, bounded summary timeline;
   partial lines and record limits are explicit. Baseline is not verified
   game-session identity. Use original Probe only when relevant; it prepares
   a command for an authorized human/admin and reads original reported
   results, not map gameplay success.
5. Code Mode filters/compares bounded observations and writes the report
   in the host conversation. No new report database, background runner
   or second telemetry engine. Record map/revision (operator-confirmed),
   slots, test objective, actions actually performed and who performed
   them, expected/observed result, issue and reproduction, relevant
   evidence/time/identity, coverage and blockers. Use PASS/FAIL only
   when independently supported; otherwise BLOCKED or NOT_TESTED.
   Existing analytics/log attribution remains operator-named and unverified;
   even appended records may belong to another world/player. Do not treat
   missing errors, bounded/truncated coverage or unbound records as PASS.
   Without a window flag the prior tail summary is unchanged. Legacy may
   use the additive read-only window option; restricted code-mode does not
   advertise/admit it. Baselines are lost at core shutdown.

Configured source flags are not proof of provider availability. The
operator remains in control of manual app actions. Code Mode uses the
same MCP request/result bounds; no additional native ownership or
execution privilege is granted. Tool count is not measured latency,
CPU, memory or token savings.

### Visual context budget (source-only)

Use WebP for new Playtest image calls, with the smallest readable fitted
canvas. One-shot frameRef is an opaque equality hint, not an input token,
world/session identity or guarantee of gameplay sameness. It binds slot,
verified process/window/birth/session, format, dimensions and the original
encoded-image hash; it requires no server-side image/history store. Suppress
only exact byte matches requested by the caller; never suppress based on
assumed visual similarity. Playtest observe/read image responses project
compact receipts (size, status, timestamp, sequence/reference, freshness
where applicable); native identity and validation remain in the core. Full
legacy responses and the existing multi-client metadata/baseline contract
are preserved. One-shot deduplication still performs capture, resize and
WebP encoding; identical frames now skip Base64 conversion as well as
image delivery in the MCP response. A stopped persistent
worker reserves its slot until close is observed; retry only after it exits.
GPU_WORKER_STOP_PENDING is a refusal, not a successful restart.
WebP reduces potential encoded payload size, not a guaranteed model
token discount; dimensions and number of delivered images affect vision
context according to the actual model/host. Use the host's supported low-detail
option only when appropriate. OpenAI API vision accepts WebP and documents
model-specific image sizing: https://developers.openai.com/api/docs/guides/images-vision.
Actual Codex MCP image handling, native latency, compression ratios, clarity
and token savings remain unverified. No automatic PNG retry or format lie.
Legacy defaults remain PNG; explicit imageFormat='webp' with optional size is
available only on non-experimental WGC connections. Resized WebP does not
admit the legacy F3/XYZ review path or authorize coordinate-based input.
Manual product screenshot remains original-resolution lossless WebP with its
existing binary approval and no-overwrite save gates. No gate is opened here.

### Compact player action authoring (source-only)

The intended playtest input boundary is one bounded action timeline plus
optional fresh observation and independent cancellation, not gameplay presets.
Codex chooses controls, overlap, timing and camera paths. Local deterministic
compilation/scheduling should expand them; no model invocation per key, tick,
heartbeat or mouse sample. OpenAI VPT's 20 Hz keyboard/mouse interface is a
reference for human-like control granularity, not Education backend proof:
https://openai.com/index/vpt/. Code execution with MCP supports filtering
intermediate results before model context:
https://www.anthropic.com/research/code-execution-with-mcp.

The existing **read-only** planner in the restricted/legacy profiles accepts
either the original `actions` or compact `groups`, never both. The trusted
SDK's existing `tester.input.plan` accepts the same groups. For example:

```js
await tester.input.plan({groups:[
  {slot:2,atMs:0,durationMs:600,hold:['move_forward','strafe_right']},
  {slot:2,atMs:100,durationMs:300,look:{deltaX:80,deltaY:-12,steps:3}},
  {slot:2,atMs:200,durationMs:100,hold:['jump','attack']}
]});
```

One group can describe several independent held controls with a shared
interval and an optional relative camera path. Separate groups overlap
freely subject to the original channel rules. Gaps are expressed by offsets.
Camera defaults to deterministic smoothstep interpolation, with optional
linear curve or explicit sample count; the default count is ceil(duration /
100 ms). Integer cumulative quantization preserves the exact total delta,
omits zero-motion samples, and divides time without accumulating rounding
drift. Each nonzero sample retains the existing 50–250 ms duration bound.
Expanded actions retain the original 32-action/mode/channel bounds;
experimental native limits are unchanged. This is sampled authoring, not
verified smooth game-camera movement, degrees, or keybind calibration.

Grouped preview defaults to only slots, expanded action count, last nonzero
control's authored end (horizon),
concurrency and the blocked execution gate. Ask includeTimeline or
includeClientFrames only when reviewing details. The local source export
`expandActionGroups` in control-plan.mjs expands to the same canonical
actions for trusted host composition. No additional preview/execute tool
is exposed in Playtest (15 tools). Actual input is still blocked; raw keys,
sprint, menu pointer, scroll, inventory, hotbar and backend calibration are
not newly supported. No preset, random humanization, native ABI change,
new scheduler, or experimental activation.

The existing experimental multi-client pulse builder now reuses one
validated cross-client compilation rather than recompiling per slot; exact
per-slot validation, heartbeat, expiry, retrigger refusal and neutral
behavior are preserved. The observed action path and ordinary experimental
action path share the same private schedule runner; the observed path reuses
its original preflight compilation for native transport and fingerprint.
Read-only route assessment retains its initial validated timeline across
native observation instead of compiling twice. No public prepared-plan
argument or cached execution token. These reduce deterministic compiler work,
not a measured runtime or token saving. Source fixtures are authored NOT_EXECUTED.

Native experimental input/lease requests use the same authenticated
controller envelope with an empty-roster receipt (schema 1, no status
payloads). The actor performs the original slot, process, lease, focus,
expiry and release checks before acknowledging, but skips unrelated roster
observation for each pulse/group/neutral/lease. The common request owner
validates the compact response even for cleanup-only consumers and marks a
wrong success shape uncertain; no retry or false neutral ACK. Normal and
experimental Launch and Inspect retain their five-profile observation.
Desktop admission and same-image peer pinning are unchanged; native input
remains compile-OFF and no gameplay effect is inferred from transport ACK.
This avoids up to five profile observation iterations per private receipt,
not a measured timing, resource or token improvement.

### Direct SDK for independently trusted hosts

For a separately authorized trusted PC-local host, create one
`createOperationCore(config)` from `operation-core.mjs`. Attach
`createCodeModeClient(core,'playtest')` from `code-mode.mjs` per script for
all 15 existing Playtest operations through 17 named methods (three Probe
methods share one operation). The default `createCodeModeClient(core)`
retains the restricted read/plan contract. The trusted host passes only the
fixed `tester` facade to its permission-controlled execution environment
and closes the facade in finally. If MCP is needed, attach
`createMcpHandler(core,'playtest')` (or the matching restricted profile);
do not start a second core for the same workflow. The existing stdio entry `mcp.mjs` and
all 25 default legacy tool names remain compatible. No arbitrary-code execution
tool, Deno/Node guest, dependency, new controller or PC permission is added.
Direct SDK call budgets apply per client facade; Codex owns its own cell budget.
MCP retains its existing request/response limits, not the direct SDK's per-script
64-call budget. The stdio adapter retains at most 64 KiB plus an optional CR
per LF-delimited request; oversized frames are discarded until the next LF.
At most 16 requests may be pending per connection. Asynchronous stdout pressure
pauses stdin until drain; overload, output error or a 30-second drain stall
closes the connection and invokes core shutdown. There is no waiting request
queue or automatic retry. These are source-level bounds, not measured Windows
pipe latency or guaranteed interruption of synchronous OS writes.
Use unique JSON-RPC IDs and one connection per workflow.
Duplicate in-flight IDs are ignored (the original retains its only reply);
there is no replay cache, automatic retry, second dispatcher or serial queue
that delays an independent stop. Closing the core suppresses late tool replies.
The verified native target checks before/after capture and before delivery are
intentionally separate freshness/ownership checks. Each target read now uses
a private one-slot receipt from the existing actor (no unrelated roster or
window relabeling); full Inspect remains unchanged. Selected target reads
cannot spawn or reconcile an uncertain mutation and are not exposed to desktop
commands. The same original process/window proof remains required.
The same-image native CLI uses authenticated IPC per request; a persistent
replacement channel is not introduced without measured benefit and equivalent
admission. Runtime connection latency and host compatibility remain unverified.

For Playtest, use `tester.capabilities({inspectProfiles:true})`,
`tester.profiles.launch({slot:2})`, `tester.observation.capture({slot:2})`,
`tester.observation.start/read/stop` and the existing evidence/diagnostics
methods as needed. Observation results return compact metadata and optional
WebP Base64 separately; unchanged frames retain the same omission contract.
`tester.testing.probe({slot:2})` may prepare a command but never dispatches
it. Explicit restart, archive and evidence-window operations reuse their
original admission. The restricted facade retains `tester.testing.context`
and action previews. Neither facade grants native reservation, publication,
writer leases, arbitrary commands, profile close or input dispatch.
Each facade permits 64 attempted calls, one in flight, 64 KiB input,
128 KiB text per call and 1 MiB aggregate text. Playtest has a separate
4 MiB Base64 aggregate image budget, also the existing per-image ceiling;
image delivery does not stringify and parse Base64 again. The trusted
host must independently enforce script time, memory, output and OS
permissions. Closing a facade does not cancel native operations or an
active sampler: stop it explicitly when finished; core shutdown remains
host-owned.

The stdio adapter negotiates the retained initialize-era revisions
2025-06-18 and 2025-11-25; 2026-07-28 is not claimed until its stateless
contract is implemented (see https://blog.modelcontextprotocol.io/posts/2026-07-28/).
Capability inventory now derives from the same advertised descriptors;
self-exclusion is retained, ordering follows tools/list. No runtime or
protocol conformance tests executed.

### Source ownership

- `operation-core.mjs` is the single coordinator for session-bound
  state, admission, QA checkpoints and recovery.
- `tool-catalog.mjs` owns the one descriptor inventory. `code-mode.mjs`
  declares the shared SDK routes and projects the playtest/compatibility subsets
  from the original descriptor catalog.
- The canonical native transport already validates its receipts. The core
  selects it once or validates injected provider results at the same seam;
  fresh target reads, independent consumer validation and native gates remain.
- `native-transport.mjs` owns the existing same-image CLI and exact
  receipt validators; `evidence-reader.mjs` owns the bounded capture
  file read; `agent-contract.mjs` owns common argument and text bounds.
- Existing domain modules remain their semantic owners. The host
  facade and stdio adapter do not create additional domain state.

Trusted host integration (source example, not a runnable guest sandbox):

```js
import {createOperationCore,parseConfiguration} from './operation-core.mjs';
import {createCodeModeClient} from './code-mode.mjs';

const core = createOperationCore(parseConfiguration(operatorSelectedArgs));
const client = createCodeModeClient(core,'playtest');
try {
  // A trusted local host can compose this bounded API without MCP framing.
  const context = await client.tester.capabilities({inspectProfiles:true});
  // Keep only the needed bounded evidence; do not log private source paths.
  returnToAuthorizedCaller(context);
} finally {
  client.close();
  core.shutdown(); // only the owner of the entire core may do this
}
```

This example describes the trusted host, not model-authored code.
The host supplies configuration and any guest isolation; never grant
arbitrary model code direct Node access. No automatic local execution.

## Choose the existing owner

| Goal | Canonical route | Hard boundary |
| --- | --- | --- |
| Inspect or launch an owned profile | `mcp.mjs` / trusted Playtest Code Mode → `operation-core.mjs` → same-image `agent_cli.rs` → original Rust session controller | No second launcher or Minecraft session owner |
| Observe a game window | `visual-capture.mjs` (CUA) or opt-in `gpu-frame-source.mjs`/native WGC worker | Exact PID/birth/session/HWND revalidation; no global desktop or game input |
| Plan or check an action | `control-plan.mjs`, `input-timeline.mjs`, `input-route.mjs`, `input-delivery.mjs` | Plans and transport ACKs are not gameplay proof |
| Read imported diagnostics | `diagnostics-local.mjs` through existing bounded MCP tools | Explicit local data only; no live log collector |
| Inspect original Skills Probe QA | One preferred `probe_qa` entrypoint → `probe-qa.mjs` | No command dispatch or certified map PASS |
| Review map requirements | Optional `map-test-plan.mjs` and `probe_qa.mapPlan/qaReport` | Every map requirement remains NOT_TESTED |
| Review supplied team ZIP updates | `team-update.mjs` and approved source fingerprints | No automatic module installation or sync |
| Report limited agent activity to desktop | Existing `agent_capabilities` + explicit same-image status mirror | Presentation-only; desktop does not control MCP/game |

## Select the same Project Data

For an installed or prepared `App/lazy-tester.exe`, select optional archives
with the existing flags, using `project` instead of repeating an absolute
path: `--diagnostics-dir project --testing-archive-dir project`. Each flag
enables only its own archive; leaving it unset keeps that archive disabled.
Both resolve beneath the sibling `Data/` shown by Desktop. Explicit archive
paths must agree with that root for the standard App layout and with each
other when both are configured. For developer/legacy executable layouts or
archive-only use, supply the exact absolute paths under the selected Project
Data instead; no guessed root. The existing `--evidence-dir` is a separately
selected read-only capture input, not an output or a copy destination.
See [canonical data layout](../../../docs/system/data-layout.md).

## Runtime and permission boundaries

The **only** Windows game lifecycle and native process owner is
`Development/apps/desktop/src-tauri/src/engine/`, especially
`native_clients_session.rs` and
`independent_bootstrap/session_controller.rs`.
The optional `mcp.mjs` uses local stdio and an authenticated original
same-image CLI. It never operates as a separate game authority, HTTP
server, credential store, background gameplay agent or independent
test runner. Profile/account/world data in ignored `App/` and `Data/`
must not be reset during development.

- `human-plus-four-ai` protects MC01 for the human; only MC02–MC05 are
  potential AI targets. `five-ai` allows MC01–MC05 only in separately
  enabled experimental mode. No automatic takeover.
- Experimental native AI control requires both source feature enabling
  and explicit operator opt-in. The 96-byte shared-memory ABI v4,
  per-slot identity, bounded timing, writer/lease epochs, exact native
  neutral ACK and uncertain-outcome quarantine must remain intact.
  The opt-in Win32/Raw Input consumer is excluded from normal builds.
- Retired VMware/Sandboxie/Guest Agent paths are not fallback providers.
  Do not add a game hook, driver, auto sign-in or additional IPC/QA owner.
- Do not publish status as verified native gameplay, world identification
  or five-client acceptance.

## P58 — Read-only semantic Windows UIA inspection (SOURCE_ONLY)

The existing MCP now advertises `inspect_client_ui({slot})` for one
AI-owned native window, only when the configured Cua Driver is available.
It brackets `get_window_state` with the original native controller's
PID/HWND/birth/session verification, requests `include_screenshot:false`,
and limits UIA to 64 nodes / depth 10. It returns small, allowlisted
control roles, selected button/menu labels, enabled flags, and advertised
pattern names. All edit/password values, window titles, snapshot action
tokens, raw provider diagnostics and PNGs are excluded. Empty/degraded/
truncated walks remain unverified. **This is observation, not input,
not proof of Minecraft gameplay, and not verified on Windows.**
The Cua daemon must already be running; the tool never starts it.
No new controller, startup worker, CI or native game-control ability.

## P59 — Strict background action assessment (SOURCE_ONLY)

The new read-only `assess_client_background_action({slot,elementIndex,action:"invoke"})`
reuses P58's **fresh** PID/HWND/birth/session-fenced Cua UIA read. It
originally returned either `BACKGROUND_UNAVAILABLE` or
`SEMANTIC_CANDIDATE_NOT_EXECUTED` for a uniquely identified, enabled
`Button` advertising `invoke` in a complete tree. P61 now returns
`BACKGROUND_UNAVAILABLE` for **all** cases (details below). Cua UIA may still
activate its target window when invoked; therefore candidacy is **not**
proof of nondisturbance or permission to act. No action token or
input/foreground fallback is exported. A zero-based element index is
valid in Cua's documented token contract. Tests are authored but not
executed; no new Windows game input capability.

## P60 — Snapshot provenance and strict route refusal (SOURCE_ONLY)

The existing Cua tree projector now matches the provider's `snapshot_id`
against **every** returned `element_token` and zero-based index;
repeated indices cannot become action candidates. The only exported
indicator is `snapshotProvenance: TOKEN_INDEX_MATCH_AT_READ | UNVERIFIED`;
neither tokens nor snapshot IDs leave MCP. Missing/mismatched provenance
refuses P59 candidacy while preserving read-only UIA inspection.

The pinned upstream Windows `click` is NOT Invoke-only: it can route
element actions to `PostMessage` or injection, or other UIA patterns.
Its foreground shield documents a WPF handler limitation. Every
assessment therefore reports `backgroundRoute: BLOCKED_NOT_INVOKE_ONLY`,
`targetPostcondition: NOT_DECLARED` and
`focusIsolation: NOT_VERIFIED`, even for a read-time token match.
No dispatch, live token reservation, new daemon, foreground fallback or
implicit retry was added. Future execution requires a no-fallback
Invoke-only transport, target-owned postcondition, native identity fences
and separately authorized Windows no-focus proof. Fixture cases were
authored, NOT_EXECUTED.

## P61 — No-fallback Invoke-only admission stays blocked (SOURCE_ONLY)

The pinned Cua Driver revision and the examined newer upstream
`290552c6374da181314e26d69156198f7d48c0df` do **not** expose a
no-fallback `InvokePattern.Invoke` tool through the inspected Windows
`click`, `invoke_menu` or `set_value` paths. In particular,
`invoke_menu` explicitly activates the foreground window. Neither an
Invoke advertisement nor a driver `delivery_mode:"background"`
guarantees no user-focus interruption. The Cua snapshot token is not
a reserved action capability; a later read can invalidate it.

As a fail-closed contract correction, the **existing**
`assess_client_background_action` now returns
`status: BACKGROUND_UNAVAILABLE` and
`reason: CUA_INVOKE_ONLY_ROUTE_NOT_AVAILABLE` even for a complete,
uniquely indexed, enabled Invoke Button. `semanticCandidate:true`
is informational evidence only; it cannot be passed as a click
authorization. `transportFallbackPermitted:false`,
`actionPermitted:false` and `actionDispatched:false` remain explicit.
No extra MCP tools, background loop, native controller changes or
Windows side effects.

**Future acceptance must be two independent checks, never inferred
from transport success:** (1) exact-window, no-fallback Invoke-only
transport with provider snapshot/token owned within one bounded action,
native identity fenced before/after, no cursor/focus/z-order changes
across the entire action; and (2) an application-owned, typed
postcondition in the same exact target, read independently after
dispatch (not a PNG hash, generic UIA tree change, window title or
transport ACK). If either is missing, return BACKGROUND_UNAVAILABLE
before dispatch. Any uncertain timeout after attempted action must
be quarantined without replay. Source fixtures AUTHORED_NOT_EXECUTED;
Windows nondisturbance NOT_VERIFIED.

## P62 — Driver integration decision (SOURCE_ONLY)

No existing Cua Windows `click`/`invoke_menu` call meets the
strict no-fallback, no-focus contract; `get_window_state` tokens
are not an atomic action lease. If implemented later, a
**maintainer-supported Cua upstream `invoke_element_strict`**
must reuse its original Windows tool registry, snapshot cache and
window ownership checks, adding exact process-birth/session fences,
one UIA Invoke transport with no alternative and independent
application-owned readback. Reuse Cua `verify_state` only for
reviewed typed target predicates, **never** as generic Minecraft
gameplay confirmation. Focus/no-interruption requires separate
Windows evidence, including transient activation checks; upstream
`fg_bypass` is not such proof. In this revision no compatible
provider or Minecraft postcondition is available. The existing MCP
assessor remains `BACKGROUND_UNAVAILABLE`; no action, dependency,
fork, daemon, test or release changes. Detailed owner/boundary
decision is recorded in the existing Windows parity design.

## P63 — Upstream Invoke proposal + early MCP refusal (SOURCE_ONLY)

The existing `assess_client_background_action` now shares its one
`validateCuaStrictBackgroundIntent` validator with the pure assessor
**before** native/UIA observation. Invalid or unrecognized action
types, indices, caller-supplied tokens, foreground overrides, fake
postconditions and retries are rejected without touching Cua or
the native controller. This is read-only gate hardening, not action
support. Negative Node fixtures were authored, NOT_EXECUTED.

The full proposed upstream Rust operation and refusal matrix live
in the existing [Windows parity design](../../../docs/system/windows-background-computer-use.md).
Current Cua `click`/`invoke_menu` still cannot satisfy zero-interruption;
no local Rust fork, driver executable, UIA mutation, experimental MCP
tool or gameplay command was added. `BACKGROUND_UNAVAILABLE` remains
the only actionable outcome; a semantic candidate is information only.
## Efficient visual observation

Use `start_visual_observation`, `get_multi_client_visual_state`,
`read_visual_observation`, `stop_visual_observation` and
`observe_client` through **one existing MCP process**.
The metadata route never sends PNG pixels by itself; a model/operator
must explicitly request an eligible frame to review it.
The selected original game window must remain independently
identical (PID, birth, Windows session and root HWND) across reads.

- Capture providers retain bounded frames per slot; CUA PNG and native
  WGC PNG/WebP share a **3 MiB encoded image** limit. CUA reads
  exactly the admitted file length and refuses changed or incomplete
  output, instead of unbounded `FileHandle.readFile()` after a size check.
  The observer and MCP image response share a **4 MiB base64** ceiling.
  Oversize, expired, replaced or inconsistent frames fail closed.
- WGC frames use the existing bounded `LTG1` protocol. Invalid
  timestamps are refused from the fixed header before allocating PNG
  bytes, and rechecked after assembly. Native producer FPS follows
  the requested observer interval within 1–5 FPS; no worker is
  automatically installed or packaged.
- Identical WGC image hashes, format and dimensions reuse one private
  payload between producer and verified observer. Polling and metadata-only
  reads do not convert WGC frames to Base64; explicit image delivery encodes
  once and releases raw bytes. A newer, not-yet-verified different frame
  may coexist with the observer's prior verified payload. Worker receipts
  and native identity/freshness remain independently checked.
- When WGC stops or fails, the provider's slot map retains bounded
  status counters. Replacement is refused until the exact persistent
  or one-shot child closes, including cancelled, timed-out or failed
  previews. No new process owner or retry. Late callbacks from the old
  worker cannot change a restarted slot.
- The observer uses one short-lived per-slot cache, bounded freshness
  and sampling intervals, adaptive backoff for identical bytes.
  WebP also backs off repeated producer packets without renewing freshness;
  adaptive delays are capped at 2.5 s or the explicitly requested longer
  interval. The existing native producer FPS and legacy PNG cadence remain.
  Metadata-only group summaries remain. Replayed worker sequences are
  not new observations; a hash change is not semantic movement.
- Lossless capture and native target revalidation are mandatory.
  Avoid extra compression, constant screenshot transmission, second
  watchers or retries after uncertain native operations.

Implementation: `visual-observer.mjs`, `visual-capture.mjs`,
`gpu-frame-source.mjs`,
[native WGC candidate](native-gpu-capture/README.md).
MCP efficiency stages P45–P50 and P52 are preserved in the
[historical source reference](../../../planning/history/mcp-agent-source-reference-through-2026-11-05.md);
P51 native worker details remain in its [own README](native-gpu-capture/README.md).

## One-entry original team and QA workflow

The uploaded team's original `minecraft-bedrock-agent-skills` owns
in-game analytics/results and the original Skills Probe owns foundation
probe execution. Lazy Tester reads only reviewed
`[ANALYTICS]`/`[GAME_RESULTS]` capture records; it does not implement
a duplicate collector, analytics engine or Skills Probe runner.

`probe_qa` is the preferred MCP QA tool. First use validates the
operator-provided ZIP against the pinned baseline and caches the catalog
while stable. The QA plan pins a local capture cursor and may correlate
a read-only original native window. Only an independently authorized
Player/admin can dispatch the original in-game command: Lazy Tester
**never sends it**. Repeated pending reads inspect appended bytes;
terminal snapshots are reused without re-reading the complete ZIP.
`restart=true` starts a separate checkpoint, not an automatic retry.

A read-only Testing run may optionally be correlated with the original
controller and a private immutable `Data/Testing` summary, but the
sealed transfer/CAS path has no external authorization route.
Neither source ZIP hashes, OS process handles nor log markers prove
the exact Player, world, command delivery or gameplay success.

A reviewed `map-test-contract.json` may select `full`, `smoke` or
`affected`; without a verified map source diff, `affected` falls
back to `full`. Every map check remains `NOT_TESTED`,
`qaReport.acceptance=NOT_ACCEPTED`, even when the original Probe
reported PASS. No map command or in-game scorer is added.

Canonical details:
[Map-aware Probe QA](../../../docs/system/map-aware-probe-qa.md),
[team telemetry](../../../docs/system/education-team-telemetry-compat.md),
[local diagnostics](../../../docs/system/map-diagnostics.md),
[team update intake](../../../docs/system/education-team-source-updates.md).

## Agent activity, UI, and proof ceiling

`agent_capabilities` exposes a bounded local-memory activity snapshot.
A separate explicit `publishToDesktop` request can mirror that status
through the existing same-image controller to the Play header. The
desktop only describes it as **reported by local agent**; missing,
expired or stale activity is not verified PC control or successful
gameplay. No new desktop watcher, Stop action or controller is implied.

**REMOTE_GITHUB** source review may verify code owners, constraints,
authored fixtures and default-off flags. It does **not** execute Node,
Rust, TypeScript, Windows processes, CUA/WGC, native input or Minecraft.
All source tests on the current revision remain
`AUTHORED_NOT_EXECUTED`; Windows/Education and five-client acceptance
remain `NOT_VERIFIED`. Release/package, gameplay/Player/admin commands,
private data changes and local tests require separate operator approval.

For actual operator-controlled procedures (not authorized by this
document), see the
[validation guide](../../../docs/validation/ai-computer-use-localtest.md).
