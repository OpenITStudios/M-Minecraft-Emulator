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

## Choose the existing owner

| Goal | Canonical route | Hard boundary |
| --- | --- | --- |
| Inspect or launch an owned profile | `mcp.mjs` → same-image `agent_cli.rs` → original Rust session controller | No second launcher or Minecraft session owner |
| Observe a game window | `visual-capture.mjs` (CUA) or opt-in `gpu-frame-source.mjs`/native WGC worker | Exact PID/birth/session/HWND revalidation; no global desktop or game input |
| Plan or check an action | `control-plan.mjs`, `input-timeline.mjs`, `input-route.mjs`, `input-delivery.mjs` | Plans and transport ACKs are not gameplay proof |
| Read imported diagnostics | `diagnostics-local.mjs` through existing bounded MCP tools | Explicit local data only; no live log collector |
| Inspect original Skills Probe QA | One preferred `probe_qa` entrypoint → `probe-qa.mjs` | No command dispatch or certified map PASS |
| Review map requirements | Optional `map-test-plan.mjs` and `probe_qa.mapPlan/qaReport` | Every map requirement remains NOT_TESTED |
| Review supplied team ZIP updates | `team-update.mjs` and approved source fingerprints | No automatic module installation or sync |
| Report limited agent activity to desktop | Existing `agent_capabilities` + explicit same-image status mirror | Presentation-only; desktop does not control MCP/game |

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
returns either `BACKGROUND_UNAVAILABLE` with the reason or
`SEMANTIC_CANDIDATE_NOT_EXECUTED` for a uniquely identified, enabled
`Button` advertising `invoke` in a complete tree. Cua UIA may still
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
