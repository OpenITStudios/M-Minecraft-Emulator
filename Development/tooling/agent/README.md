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

## Playtest-first operation (Unreleased, source-only)

Lazy Tester remains a manual Windows application. A separately authorized
local Code Mode host can use the SAME existing session controller and
observation/evidence owners through one stdio MCP connection. The product
goal is Launch -> Play -> Observe -> Report, not a separate AI game engine.

### Recommended local connection

Select `--surface playtest` explicitly for new playtesting workflows. The
existing `legacy` default and restricted `code-mode` surface remain
compatibility profiles; do not run them alongside playtest or silently
upgrade the old read/plan permissions. Codex's existing Code Mode host
composes the selected MCP tools. No new server, sandbox or dependency.

Example for a separately authorized PC-local Codex configuration (replace
all illustrative paths with reviewed absolute local paths):

```toml
[mcp_servers.lazy_tester]
command = 'C:\Tools\node.exe'
args = ['D:\Source\M-Minecraft-Emulator\Development\tooling\agent\mcp.mjs', '--surface', 'playtest', '--mode', 'human-plus-four-ai', '--exe', 'D:\LazyTester\App\lazy-tester.exe']
default_tools_approval_mode = 'prompt'
```

Configure only the explicitly selected capture, evidence, diagnostic,
Probe or map-contract paths needed for the session. No automatic startup,
credentials, account signin, map import, world join or command dispatch.
Use the installed Codex host's supported Code Mode setting if available;
do not change global features or permissions automatically. No configuration
has been installed or tested on a user PC.

### One product catalog, existing operations only

| Task | Public playtest tool |
| --- | --- |
| Discover configured sources and available operations | agent_capabilities |
| Inspect / explicitly launch an AI-owned profile | inspect_profiles, launch_profile |
| Observe on demand / semantic UI | observe_client, inspect_client_ui |
| Optional bounded monitoring | start_visual_observation, get_multi_client_visual_state, read_visual_observation, stop_visual_observation |
| Prepare/review original Probe, explicit restart or read historical result | probe_qa |
| Read bounded analytics / game results | summarize_test_evidence, read_game_evidence |
| Query imported issue evidence when needed | get_diagnostics_overview, list_issue_candidates, get_issue_details, query_diagnostic_evidence |

All 16 names are existing operations with existing native admission.
The same descriptor catalog and projection serve both standard MCP and
Codex Code Mode on this connection. The independent trusted Node SDK
below stays read/plan-only; it does not transport images or grant writes.
The old 13-tool restricted profile is retained unchanged. No public
planning, duplicate context, old Probe aliases, low-level HWND targeting,
screen-text review, native reservation, status publication or input
experiment in playtest. Experimental core attachment is refused.
MC01 stays protected in hybrid mode. `five-ai` is a slot policy, not proof
of autonomous play.

### Minimal playtest / report workflow

1. Read capabilities once and inspect profiles when needed. Launch only
   explicitly selected AI slots with the existing launch operation, reuse
   the returned roster, and wait for observed readiness without tight
   polling. A launch receipt is not a game/world/login receipt.
2. Prefer one on-demand CUA screenshot for a concrete observation.
   If WGC is explicitly selected, start its bounded sampler before reading
   frames; stop it when done. Fetch metadata first for repeated monitoring
   and retrieve pixels only for relevant fresh observations. Never assume
   a changed image hash proves a game action.
3. Actual gameplay input is BLOCKED_NO_VERIFIED_INPUT_BACKEND. A human can
   play using the normal app; do not turn a planned action into executed
   evidence or enable experiments to fill the gap. Close remains manual
   until the full desktop recording/window release admission can be reused.
   Native product capture/recording approval is still held; existing CUA
   or WGC providers are optional and not runtime verified.
4. Read evidence only for selected slots. Use original Probe only when
   relevant; it prepares a command for an authorized human/admin and
   reads original reported results, not map gameplay success.
5. Code Mode filters/compares bounded observations and writes the report
   in the host conversation. No new report database, background runner
   or second telemetry engine. Record map/revision (operator-confirmed),
   slots, test objective, actions actually performed and who performed
   them, expected/observed result, issue and reproduction, relevant
   evidence/time/identity, coverage and blockers. Use PASS/FAIL only
   when independently supported; otherwise BLOCKED or NOT_TESTED.
   Existing analytics/log tail remains operator-named and unverified;
   compare before/after where useful, do not attribute old/unbound records
   to this session or treat missing errors as PASS.

Configured source flags are not proof of provider availability. The
operator remains in control of manual app actions. Code Mode uses the
same MCP request/result bounds; no additional native ownership or
execution privilege is granted. Tool count is not measured latency,
CPU, memory or token savings.

### Direct SDK for independently trusted hosts

For a separately authorized trusted PC-local host, create one
`createOperationCore(config)` from `operation-core.mjs`. Attach
`createCodeModeClient(core)` from `code-mode.mjs` per script; the trusted
host passes only the fixed `tester` facade to its own permission-controlled
execution environment and closes the facade in finally. If MCP is also
needed inside that host, attach `createMcpHandler(core,'code-mode')` for the
same restricted read/plan contract; do not start a
second core for the same workflow. The existing stdio entry `mcp.mjs` and
all 25 default legacy tool names remain compatible. No arbitrary-code execution
tool, Deno/Node guest, dependency, new controller or PC permission is added.
Direct SDK call budgets apply per client facade; Codex owns its own cell budget.
MCP retains its existing request/response limits, not the direct SDK's per-script
64-call budget.

Prefer the existing combined `tester.testing.context({slots:[2,3,4,5]})`
instead of separate profile, peer and evidence reads. Compose diagnostics,
evidence, metadata and action previews locally and return only needed
results. `tester.testing.probe({slot:2})` uses the same canonical volatile
checkpoint as MCP; it may prepare a command but never dispatches it.
Read-only inspection and historical archive use explicit SDK methods.
The facade does not expose launch, image delivery, sampler start/stop,
native reservations, restart, publication, writer leases or input dispatch.
Those retain their existing explicitly admitted MCP paths; no fallback
or automatic switch. The preferred playtest connection above uses MCP
for launch and images without widening this independent SDK. Each facade permits up to 64 attempted calls,
one in flight, 64 KiB input / 128 KiB result per call and 1 MiB aggregate
result bytes; the trusted host must independently enforce script time,
memory, output and OS permissions. A closed facade cannot authorize later
calls; it is not native cancellation. Core shutdown remains host-owned.

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
const client = createCodeModeClient(core);
try {
  // A trusted local host can compose this bounded API without MCP framing.
  const context = await client.tester.testing.context({slots:[2,3,4,5]});
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
| Inspect or launch an owned profile | `mcp.mjs` / trusted Code Mode read → `operation-core.mjs` → same-image `agent_cli.rs` → original Rust session controller | No second launcher or Minecraft session owner |
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

- Capture providers retain at most one relevant bounded PNG per slot;
  CUA and native WGC share a **3 MiB lossless PNG** limit. CUA reads
  exactly the admitted file length and refuses changed or incomplete
  output, instead of unbounded `FileHandle.readFile()` after a size check.
  The observer and MCP image response share a **4 MiB base64** ceiling.
  Oversize, expired, replaced or inconsistent frames fail closed.
- WGC frames use the existing bounded `LTG1` protocol. Invalid
  timestamps are refused from the fixed header before allocating PNG
  bytes, and rechecked after assembly. Native producer FPS follows
  the requested observer interval within 1–5 FPS; no worker is
  automatically installed or packaged.
- Identical WGC PNG byte hashes and dimensions reuse the previous
  on-demand base64 representation, while worker sequence/timestamp and
  native identity/freshness are always checked independently.
- When WGC stops or fails, the provider's slot map retains only
  bounded status counters—not the stopped child, decoder or partial PNG.
  Late callbacks from the old worker cannot change a restarted slot.
- The observer uses one short-lived per-slot cache, bounded freshness
  and sampling intervals, adaptive backoff for identical PNG bytes,
  and metadata-only group summaries. Replayed worker sequences are
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
