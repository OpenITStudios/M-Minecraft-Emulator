---
id: document.system.map-aware-probe-qa
class: DOCUMENT
domain: system
role: GUIDE
authority: WORKING
lifecycle: SOURCE_ONLY
---

# Map-Aware Probe QA — source-only P1–P3

The original team Skills Probe remains the exclusive foundation-probe
runner. The new map planner is source-only data, not a second in-game test
engine, telemetry publisher, scoreboard, command sender or map adapter.

## One optional contract, one existing QA tool

Maintain a reviewed JSON contract in each map's own repository. Point the
existing MCP to it once using `LAZY_TESTER_MAP_CONTRACT` or
`--map-contract` and an absolute path to the local .json file.
Without it, existing `probe_qa` behavior and old tools are unchanged.

`probe_qa` supports optional `scope`: `full` (default), `smoke` or
`affected`. Its first response includes a compact `mapPlan` only when
configured. A changed scope needs `restart=true` to create a new
checkpoint. Normal polls do not repeat the plan, while terminal
snapshots retain its historical plan. No additional MCP tool or session
identifier is required.

**Important:** `mapPlan.execution=NOT_EXECUTED` and
`mapPlan.mapAcceptance=NOT_TESTED` are always true in P1, regardless of
official Skills Probe PASS/FAIL. Skills Probe auto probes test their
existing team targets, not uninstrumented map gameplay requirements.

## Example contract (illustrative, not installed)

```json
{
  "schema": 1,
  "map": "dgn-defense",
  "revision": "spec-v1",
  "requirements": [
    {
      "id": "flag.initial_hp",
      "given": "arena.running",
      "action": "none",
      "observe": "flag.health",
      "expect": {"operator": "equals", "value": 350},
      "evidence": ["map_state"],
      "affects": ["scripts/flag/"],
      "smoke": true
    },
    {
      "id": "flag.player_protection",
      "given": "arena.running",
      "action": "player.attack.flag",
      "observe": "flag.health",
      "expect": {"operator": "unchanged"},
      "evidence": ["confirmed_action", "map_state"],
      "affects": ["scripts/combat/", "scripts/flag/"],
      "dependsOn": ["flag.initial_hp"]
    }
  ]
}
```

The illustrated source selectors are not verified against the DGN
Defense repository. Only the map owner may authorize actual expected
values and map-specific source paths.

Evidence families are `team_analytics`, `team_game_results`,
`map_state`, `confirmed_action` and `visual_frame`. They describe
**required proof**, not running backend capabilities. Actions other
than `none` must require `confirmed_action`. Expectations support
`equals` (bounded boolean, integer or symbol), `unchanged`,
`increases`, `decreases`, `exists`. No arbitrary JS expressions,
in-game commands or filesystem paths are interpreted from a contract.

A contract contains 1–64 unique requirements with acyclic dependencies,
bounded relative source selectors, validated identifiers and exact fields.
Unknown data fails closed. The read-only loader pins a source identity
and SHA-256 digest once and reuses it while unchanged. An active
checkpoint stops if contract file metadata changes. The completed
snapshot remains historical; only explicit restart begins a fresh plan.

## Selection

- `full`: every requirement.
- `smoke`: tagged requirements and prerequisites; no tags falls back
  to full.
- `affected`: the pure planner can select affected/dependent/prerequisite
  tests from a caller-asserted source diff. Unknown source paths, malformed
  diff or absent diff falls back to full. The planner does **not**
  authenticate a Git diff merely because it includes SHA strings.
- Current MCP does not fetch/verify a map diff, so `scope=affected`
  conservatively selects FULL, with an explicit fallback reason.
  No AI guessing can silently skip requirements.
- Skipped tests remain NOT_TESTED, never PASS.

Plan responses contain total/required counts, no more than 12 selected
checks with bounded precondition, action, state observation, expected outcome,
required evidence family and NOT_TESTED status. They also distinguish
required selected versus required skipped checks; more than 12 selected
checks are counted without dumping the full contract. They cannot establish
world identity, installed adapters, authorized command dispatch, actual
game input, or map acceptance. Foundation Probe result is separate.

## P2 — original Probe command and Content Log proof boundary

The P2 source increment adds **no** command transport. The existing
`probe_qa` now includes a compact `proofGate` on its first handoff and
historical terminal snapshot. Its source/capture/window fields mean
*at preparation*, not continuous attestation. `readiness=BLOCKED`,
`automaticDispatch=DISABLED`, and the admin Player, QA world and log
origin remain `NOT_VERIFIED`. No MCP input field accepts a caller's
assertion of those capabilities. A probe-reported PASS is separate
from gameplay acceptance.

The same original runner source marks each RESULT with a
`build=dev|release` value. Mixed known build origins in one supposed
run are now `INCOMPLETE_MIXED_PROBE_BUILDS`; only an all-consistent
result can include its bounded `reportedBuild` field. An existing
checkpoint also refuses a second simultaneous MC slot whose named
log has the same observed underlying file identity. Neither check is
proof of process/world ownership or defense against malicious edits
to local log bytes.

### Required runtime proof before any automatic dispatch

1. In a separately approved **isolated copy** of a Minecraft Education
   Windows QA world, verify the original Skills Probe behavior pack
   is actually installed, its runner executes and an authorized
   Player/admin can invoke the *original* `/scriptevent probe:run auto`.
   A source ZIP and `/function help` alone cannot prove this.
2. Independently identify the exact MC slot's original game PID,
   process creation identity, session, HWND and world. Check the process
   before and after the command. Game window identity alone does
   **not** establish the loaded world or administrator authority.
3. Establish a **source-bound** Content Log transport attributable to
   that exact process **and** QA world. If the original runner/log
   transport cannot expose enough world identity or run correlation,
   mark this gate BLOCKED and request a minimal upstream team-owned
   probe/transport contract change; never synthesize an identity from
   `MC0x.log`, filenames, timestamps or a model assertion.
4. With a fresh stable checkpoint, observe exactly one original
   RUN_START, its complete unique RESULT set, RUN_DONE, consistent
   build origin, source revision and native target. Reject stale,
   shared, replaced, interleaved and incomplete captures.
5. Record actual run outcomes against an exact repo SHA, original
   runner digest, game version and private environment. A successful
   **foundation** Probe run still does not certify a map's own
   mechanics. Source-only proof gates must not be promoted to PASS.

Do not auto-send ScriptEvent, install packs, start a debugger socket,
reopen worlds or execute Windows games based on these instructions.
Runtime proof requires explicit separate authorization and real
observation. This milestone prepares the trust boundary; it does
not satisfy the external-world identity gate.

## P3 — Map Test Contract → original Probe → evidence → report

When a reviewed map contract is configured, the **same** `probe_qa`
returns two separate source-only projections:

- `mapPlan`: the selected checks (up to 12 detailed checks), counts,
  dependency-aware scope, and evidence families **required**, not available.
- `qaReport`: an allowlisted synthesis of the original Probe's compact
  status plus this map plan, with the hard boundary that **zero map
  requirements have been executed**. It never runs any code from a map
  contract and never uses arbitrary model-supplied gameplay observations.

The map plan and derived report carry an optional `contractSha256`
fingerprint of the exact local JSON bytes read at preparation.
This binds the historical summary to a source snapshot; the hash
does **not** authenticate the map author, a running world or gameplay.

The first prepared response includes the plan and report in a
`BLOCKED` foundation state. When the official Skills Probe ZIP or capture
is unavailable, an independently valid map contract still returns its
plan/report in the **PREFLIGHT** refusal; it does not create a fake game
run. Intermediate `WAITING` calls remain small and need no repeated plan,
ZIP scan or native target poll. A terminal outcome freezes the same plan,
`proofGate` and one derived `qaReport` in the existing per-slot memory.
Further calls return the exact same snapshot without extra source/capture
or game reads; only explicit `restart=true` starts a new checkpoint.

The `qaReport.foundation.status` conveys the **original Probe's**
reported PASS/FAIL/NEEDS_REVIEW or its incomplete/blocked state. If
native target identity is missing/replaced, the terminal status is
`INCOMPLETE` while `probeReportedStatus` can preserve the original
limited report for diagnosis. Such a report is still unauthenticated to
the Minecraft world. `qaReport.map` always retains
`status=NOT_TESTED`, `tested=0`, `passed=0`, `failed=0` and
`notTested=total`, including checks skipped by a narrow plan.
`qaReport.evidence` explicitly lists needed families, reports zero
verified map-evidence families and retains mandatory Player/admin,
QA-world and Content Log proof gaps. Overall
`qaReport.acceptance=NOT_ACCEPTED`, even when foundation Probe markers
report all PASS.

The foundation summary reads its original maximum auto-probe count
from `probe-qa.mjs`, the existing authoritative source, rather than
redeclaring a conflicting limit. An earlier source-only composition
referenced an undefined `MAX_AUTO_PROBES`, which could abort terminal
`qaReport` creation when actual expected counts were present; that
reference is corrected without enabling any execution. Counts larger
than the original catalog limit are omitted instead of being promoted
to valid proof. `qaReport.foundation.reason` retains only bounded
uppercase status codes for BLOCKED/WAITING/INCOMPLETE, not raw log
messages, filesystem paths, or an arbitrary exception. A preflight
refusal and the initial no-command state also carry those codes.

The map summary now includes `requiredSelected` and
`requiredSkipped` so a Smoke selection cannot conceal mandatory
untested requirements. Those counts are checked for internal
consistency; the underlying map checks remain NOT_TESTED.

Only fixed, validated summary fields reach the MCP caller; no raw logs,
player identifiers, private paths, world data, source code or arbitrary
`result.detail` are forwarded. The existing optional diagnostic evidence
tools remain separately read-only and are **not** silently treated as
map-specific pass evidence. This is one report projection within the
existing owners, not a second result database or test/score engine.

Without a configured map contract, old `probe_qa`, its legacy
`prepare_probe_qa`/`read_probe_qa_result` tools and normal output
shape remain compatible; no `mapPlan` or `qaReport` is added.

## Boundaries and next runtime proof

Canonical owner: `Development/tooling/agent/map-test-plan.mjs`; the
existing `mcp.mjs` only hands off a compact plan through `probe_qa`.
P2 adds a bounded read-only proof gate to the existing Lazy Tester `probe-qa.mjs`; the TEAM'S original Skills Probe runner and native game controller remain unchanged. No new
daemon, observer, data store, CI, Minecraft world/account mutation, ZIP
rehash, map source vendoring, native input or auto-execution.

Regression fixtures are authored but NOT_EXECUTED. Next runtime
acceptance, when separately authorized: verify one original Skills
Probe admin-Player command and bind its Content Log to the exact
Minecraft Education Windows QA world/profile. Map-specific execution
and Computer Use acceptance remain separate later gates.
