# Current Development Handoff — Lazy Tester

**Updated:** 2 November 2026. **Branch:** `Local` only;
`main` release-reserved; GitHub Actions CI disabled.
This file is the **single current status and next-action owner**.
Older detailed sections are preserved in
[the November 2 historical handoff](history/source-handoff-through-2026-11-02.md).
Always pin the actual latest `Local` SHA; an old report is not
new-revision acceptance.

## Product and owners

- **Desktop:** Tauri/Svelte manager for up to five separately licensed
  native Minecraft Education Windows profiles; the operator launches
  games explicitly. Svelte/typed bridge is presentation only.
- **Desktop UX (2 November):** the existing visual theme, fixed five-card
  bottom rail, central Play viewport, and native DWM peer previews remain.
  Rail order is sorted by fixed MC01-MC05 storage slot in the presentation
  layer only; selection does not recreate preview elements. The rail heading
  now matches its existing 1600px width cap. Relevant regression scenarios
  are AUTHORED_NOT_EXECUTED; live Windows/Minecraft behavior NOT_VERIFIED.
- **Desktop clarity UX (2 November):** unify global/per-slot read-only
  Refresh wording; distinguish verified profile data from actual game
  readiness, suppress stale running-count claims during connection loss,
  and require positive window response before offering guided Select game
  after a DWM preview failure. Preserve rail geometry, theme and native
  admission. Updated UI fixtures AUTHORED_NOT_EXECUTED.
- **Desktop focused UX (2 November):** selected-game Close review focuses
  Cancel, supports Escape and restores keyboard focus to the invoking
  launcher after cancellation. Graphics legacy details and read-only
  resource-pack diagnostics use native disclosure; basic Storage actions
  remain visible. No visual theme, five-slot rail, native IPC or game
  session change. Regression fixtures AUTHORED_NOT_EXECUTED.
- **AI/QA activity contract (2 November):** existing MCP
  `agent_capabilities.activity` projects only allowlisted in-memory
  visual-sampling, Probe checkpoint/report, and experimental AI-input
  recovery/request state; **never** external Windows control or Minecraft
  gameplay success. No new tool/collector, native IPC, hidden UI badge
  or extra action is introduced. Desktop has no authenticated MCP status
  reader at that earlier revision; the new explicit native-controller
  report mirror below supersedes this transport limitation without
  proving Windows runtime. Fixtures AUTHORED_NOT_EXECUTED.
- **Optional AI/QA desktop status mirror (2 November):** the existing MCP
  `agent_capabilities({publishToDesktop:true})` can explicitly report only
  enum-allowlisted local activity through a matching one-shot native CLI to
  the already-retained original controller. It adds an optional
  `SessionSnapshot.agentActivity`, bounded monotonic 12-second freshness
  and a small Play-header disclosure for relevant or stale-ATTENTION status.
  Ordinary capabilities are read-only; no hidden watch, auto-publish,
  Tauri publishing authority, native input or layout change. The source
  reports are not verified Windows control, gameplay or map QA. All newly
  authored Rust/Node/desktop fixtures are NOT_EXECUTED and the complete
  cross-process UI handoff on Windows remains NOT_VERIFIED.
- **AI/Testing UI wording (2 November):** all activity labels and descriptions
  are in English, with user-facing "Testing" instead of "QA".
  The existing status source, five fixed DWM cards, controller paths and
  `probe_qa` API identifiers remain unchanged. Updated regression assertions
  are AUTHORED_NOT_EXECUTED.
- **One-click Testing Summary (2 November):** the existing Play-header
  activity disclosure presents original Probe target and completed
  reported PASS/FAIL/REVIEW outcome, plus allowlisted counts already
  present in the in-memory `probe_qa` terminal snapshot (if complete
  and consistent). Reports always show "Map Testing: Not tested" and
  "Verification: Not verified"; blocked/incomplete checkpoints do not
  masquerade as Results available or acquire fabricated counts. No new
  page/engine/IPC or result archive, no layout/previews changes.
  Native/TypeScript/source validation, MCP fixtures and browser
  regressions AUTHORED_NOT_EXECUTED. The explicit status mirror still
  expires after 12 seconds and requires agent opt-in publication.
- **Native owner:** original Rust session controller handles game
  process identity, private profiles, normal exit and recovery.
  Workspace/DWM presentation never becomes a second lifecycle owner.
- **External model:** optional local stdio `mcp.mjs` delegates through
  the existing same-image CLI/authenticated session controller.
  Default AI input is read-only/action-planning; the independent
  keyboard/mouse candidate is opt-in, compile-OFF and excluded from
  ordinary releases.
- **Team source:** original `minecraft-bedrock-agent-skills` owns
  game analytics/results; original `skills-probe` owns foundation
  tests. Lazy Tester reads reviewed ZIP/Content Log contracts; it
  does not duplicate their modules, analytics or test runner.
- **Local diagnostics:** explicitly imported normalized evidence is
  stored under ignored `Data/Diagnostics` and exposed through
  bounded read-only MCP queries, never automatically collected.
- **Map QA:** the optional approved map-owned JSON contract contains
  requirements, prereqs, evidence families and source selectors.
  It produces a source-only test plan, not an executing map scorer.

Canonical routing: [active architecture](../docs/system/architecture.md),
[source ownership](../docs/system/implementation-map.md),
[Probe QA](../docs/system/map-aware-probe-qa.md).

## Current source workflow (P1/P2/P3)

```text
Optional approved map contract -> bounded full/smoke/affected plan
Approved original Skills Probe ZIP -> cached official auto-probe catalog
Configured MC0x capture file -> stable appended-only checkpoint
Optional retained native game target -> read-only PID/HWND/birth binding
External authorized Player/admin -> original /scriptevent probe:run auto
    [NOT SENT OR VERIFIED BY LAZY TESTER]
Appended original Probe RUN_START / RESULT / RUN_DONE markers
    -> bounded foundation result + source-only map coverage/evidence
    -> frozen optional qaReport and per-slot historical snapshot
```

- `probe_qa` is the preferred **one** MCP QA tool; legacy
  `prepare_probe_qa` and `read_probe_qa_result` remain compatible.
  One volatile plan/terminal snapshot per client; only explicit
  `restart=true` creates another checkpoint. No daemon or timer.
- Approved ZIP/catalog and map contract are cached while unchanged;
  pending reads inspect only the needed added log bytes and cheap
  source metadata. A source change, lost cursor, irrecoverable marker
  gap or replaced target is held, not silently replayed.
- Original foundation-Probe reports distinguish PASS/FAIL/REVIEW,
  waiting and incomplete, **never** map/gameplay acceptance.
  Unknown/unbound native targets cannot produce a verified game PASS.
- P2 `proofGate` explicitly separates source ZIP, local capture and
  process/window observation from **unproven** Player/admin command
  delivery, exact QA world, and process/world Content Log provenance.
  Mixed `dev`/`release` Probe results and detected duplicate local
  capture file identities across slots are refused.
- Optional `mapPlan` is always `NOT_TESTED`. `full` selects all;
  `smoke` needs declared checks; `affected` currently falls back
  to full because MCP does not authenticate an exact map Git diff.
  A plan cannot skip required tests based on AI guesses.
- P3 `mapPlan.selectedChecks` includes up to 12 bounded map
  requirements and their needed evidence, with a local contract
  SHA-256 fingerprint (not author/game proof); the same `probe_qa`
  returns optional `qaReport` during preflight/initial preparation
  and on terminal snapshot. A map plan is still available when
  Probe ZIP/capture is missing. `qaReport.foundation` preserves
  original Probe status; `qaReport.map` always records 0 executed,
  0 passed, 0 failed, and all map requirements NOT_TESTED.
  `qaReport.acceptance=NOT_ACCEPTED` regardless of Probe-reported
  PASS. Pending polls remain compact; final snapshots are reused
  without additional I/O.
- P3 report-integrity correction: `map-test-plan.mjs` now imports
  the original `probe-qa.mjs` auto-count cap (previous undefined
  reference could fail terminal composition). Only bounded status
  reasons are included; required selected/skipped counts are
  validated and exposed. Regression fixtures AUTHORED_NOT_EXECUTED;
  there is still no game command or map PASS.
- Team ZIP changes are inspected offline through existing lock
  fingerprints; new baseline versions are never auto-approved or
  synced into a running world.

## Proof ceiling and unresolved gates

**Confirmed from REMOTE_GITHUB source review:** module ownership,
declared contracts, static data/handler paths, authored bounded
regression fixtures, default-disabled experimental flags and
repository layout. This is not compiler or runtime proof.

**NOT_EXECUTED at the current exact revision:** Node/TS/Rust/MSVC
source tests/builds, package/installer, live Windows/Minecraft
Education, Player/admin ScriptEvent delivery, actual capture origin,
per-client native keyboard/mouse consumption, or five-AI gameplay.
The earlier operator five-client native pilot has an older scope/SHA,
not acceptance of this source.

**Runtime blockers (deferred while local testing is unavailable):**
actual installed Skills Probe in an isolated Education Windows QA
world; one original Player/admin command; Content Log attribution to
the exact game process/world; input/capture behavior. Neither file
names, PINNED ZIP SHA, native IPC ACK nor DWM screenshot proves them.
Do not authorize automatic commands, install game hooks, launch a
world, copy tokens or mutate account/world data through this handoff.

**Current operator priority:** source-only Map Test Contract →
original Skills Probe → evidence gap projection → `qaReport` integration,
with safe repository maintenance. Do not request local testing, run
source test suites, install packages, use CI or delete private artifacts. Preserve
`App/`, `Data/`, old pilot data, and all historical proof.

## Single next step

Retain the source-only P3 report with map tests NOT_TESTED and real-world
evidence gates BLOCKED. When a separately authorized Windows QA
environment eventually becomes available, validate **one** original
Skills Probe Player/admin run and its process/world-bound Content Log
before considering command dispatch. This gate is deferred; no local
testing is requested now.
