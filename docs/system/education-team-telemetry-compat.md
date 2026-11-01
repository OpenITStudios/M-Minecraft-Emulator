---
id: document.system.education-team-telemetry-compat
class: DOCUMENT
domain: system
role: GUIDE
authority: WORKING
lifecycle: SOURCE_ONLY
---

# Minecraft Education — team analytics + Lazy Tester compatibility

## Canonical sources (operator-supplied ZIPs, not repo dependencies)

| Responsibility | Team-owned source | Team wire contract | Lazy Tester consumer |
| --- | --- | --- | --- |
| Player/map analytics | minecraft-bedrock-agent-skills-main/modules/analytics/{contract.ts,analytics-events.ts,analytics-export.ts} | [ANALYTICS] schema 1, /scriptevent mivubi:get_analytics | tooling/agent/mcp.mjs + evidence-summary.mjs |
| Official game scoring | minecraft-bedrock-agent-skills-main/modules/game-results/{game-results-contract.ts,game-results-export.ts} | [GAME_RESULTS] schema 1, /scriptevent daigon:get_all_scoring | Same bounded read-only MCP projection |
| Channels, issue codes, throttling | minecraft-bedrock-agent-skills-main/modules/logging/{log-core.ts,log.ts} | [project:channel] WARN/INFO etc; per-issue throttle | Debugger/Content Log transport; NOT a gameplay-result record |
| Runtime/export proof | skills-probe-main/docs/test-runs/2026-10-28-debug-log-capture-run-1.md | TCP debugger protocol 7 (Education 26.32, macOS) | Future optional Education-debugger adapter, NOT Windows proof |
| Generic fallback (not a team module) | Development/tooling/agent/education-universal-pack/ | [LT_EDU_DIAG], schema 1, manual start/stop | Same read-only MCP capture projection; separate source label |

Both user-supplied ZIPs are *snapshot references*, not automatically
installed plugins or checked-in raw archives. The files are not copied,
vendored, or changed by Lazy Tester. Of 40 matching module-relative file
paths across the two snapshots, 39 are byte-identical; the one difference
is the modules/README.md. This is evidence of snapshot consistency, not
proof of compatibility on other Education builds.

SHA-256 of the uploaded archives (local provenance pins; do not automatically
upload raw archives to the repository):

- minecraft-bedrock-agent-skills-main (3)(1).zip:
  26c0a322cabed172ca74970413d5bb7a20d6f67181446df5f688e36057c54e21
- skills-probe-main (2)(1).zip:
  462fb6fe48b60f2bcf255f59d7ce0cb810c3a7ec598102c9e3888c68b341653b

Contract file pins (source bytes extracted from the user-supplied ZIP):

- modules/analytics/contract.ts: ef43cd5a6af36c266664e7a0f935bb510df9e853712105d82eed9de63ee45d16
- modules/game-results/game-results-contract.ts: 0fc57cac2d30d7a3bf1c2eeb14146700662cde79eabf94babd7f451f7bdbb7a4
- modules/logging/log-core.ts: c1dd1b88b02e515d815b4c974f966e2d194273f78efbc55f40883bc6c6efc674
- skills-probe-main/packs/BP/manifest.json:
  be7a063a4b73e216c819d6f943fa482094c70115e2fecdb4a591d38560a12aed

## Protocol and resource policy

### [ANALYTICS] — canonical general map analytics

The team analytics contract defines three collection tiers:
1. Lifecycle/low-volume facts.
2. Map-selected aggregated counters, **off by default**.
3. Bounded detail samples, explicit opt-in only.

The wire line limit is **24 KiB total marker + JSON**. Pacing is
**8 exported lines per tick**, checkpoint interval 100 ticks, and the
default caps include records=5000, counters=16, keys=10, samples=4096,
pageSize=128 and failures=3. These are upstream facts, not newly
calibrated Lazy Tester performance guarantees. A dropped/failed
counter must remain visible as ATTENTION, not silently become PASS.

The map/developer still owns optional stage/task boundaries and action
semantics. Lazy Tester must not fabricate them using generic event counts.

### [GAME_RESULTS] — canonical scoring

The game-results module owns Daigon-compatible scores.
The team identifies level_score and gameplay_result as scored record
types. Lazy Tester counts **only gameplay_result** as final
resultCount; level_score has a separate teamLevelScoreCount. The
record_type=export_summary is the final export completeness record,
including counts and optional failed_counts. Lazy Tester now recognizes
**both** but includes only gameplay_result in resultCount. A summary
with failed_counts is ATTENTION. Neither means verified gameplay PASS.
The maximum line size is 24 KiB; this must not be silently reduced
by a downstream 16 KiB filter.

### [LT_EDU_DIAG] — optional generic, low-overhead fallback

The universal test pack is deliberately *not* the team's analytics
implementation or scoring implementation. It uses a stable UUID and
version 1.1.0, keeping the same file reusable across Education test-copy
worlds. By default nothing is collected. Commands:

    /scriptevent lazytester:diagnostics start
    /scriptevent lazytester:diagnostics status
    /scriptevent lazytester:diagnostics flush
    /scriptevent lazytester:diagnostics combat on
    /scriptevent lazytester:diagnostics combat off
    /scriptevent lazytester:diagnostics entities on
    /scriptevent lazytester:diagnostics entities off
    /scriptevent lazytester:diagnostics stop

After start, **only lifecycle facts** are subscribed: first spawn,
respawn, leave, dimension change and player-only deaths. Combat
and broad entity event subscriptions activate only on explicit
commands, **auto-expire after 600 game ticks** and never emit an
event-per-hit/player payload. Summary is aggregated at most once per
200 game ticks when nonempty, with an explicit flush/stop option.
A 600-tick heartbeat can distinguish quiet from inactive captures.
This follows the upstream *tiered capture and throttle* principle
without duplicating the full team analytics/storage framework.

The generic marker has its own schema/source label and cannot be
relabelled as [ANALYTICS] or [GAME_RESULTS]. No developer-defined
wave, shop, damage-protection, arena, scoring or player outcome
can be inferred from this generic counter stream. Running both
packs may collect overlapping general facts; **do not add their
counters together**. Prefer the map's canonical team analytics
for stage/task QA. Generic fallback is useful when a map has no
team-instrumented analytics.

## Evidence, storage and AI budgets

- All raw local capture evidence remains under the existing
  approved Data/Diagnostics or explicitly configured local capture
  folder. A .mcpack itself does not collect logs into local files.
- Existing MCP reads a bounded 256 KiB tail / at most 20 parsed records
  per requested profile; a truncated tail is **partial**, not complete.
- The MCP parser recognizes [ANALYTICS] and [GAME_RESULTS] records up
  to their 24 KiB wire contract and separately recognizes the tiny
  [LT_EDU_DIAG] record (2 KiB ceiling).
- Source markers are untrusted. File name MC02.log is not proof of
  player/world/account authority; direct Education Windows Content Log
  and debugger transport still require their own verification.
- The reader produces a bounded timeline and grouped counters without
  forwarding raw JSON, player names, untrusted record IDs, or screenshots.
  Codex requests deeper evidence only through the existing on-demand
  read-only MCP tools, never by a continuous stream.
- This is source/contract compatibility **NOT_EXECUTED** on target
  Minecraft Education Windows; the macOS 26.32 protocol-7 evidence
  in the ZIP does not certify Windows 26.33 or multi-client testing.

## How a developer updates the team modules

1. Continue developing analytics, game-results and logging in the
   team's canonical Minecraft Bedrock Agent Skills repository.
2. Consume those modules inside a map using the team's documented
   module-sync workflow. Avoid copying Lazy Tester marker logic
   into the team's module implementation.
3. When a contract intentionally changes, provide the new contract
   and a small captured sample with raw credentials/player names
   removed, or an updated source ZIP. Confirm schema, line budget,
   marker/event ID, summary/failed-count semantics and version.
4. Update only the Lazy Tester source-compatible parser/projection
   and regression fixtures as needed. Do not duplicate the team's
   storage, scoring rules, or JS runtime subscriber manager.
5. Verify on an authorized Minecraft Education world before
   claiming real run success.

## Open validation gates

- Real Education 26.33 Windows Script API and supported module version.
- Actual local log transport, receipt of generic/team markers,
  protocol negotiation and verified profile/world attribution.
- Overhead under multiplayer stress (up to 30 players remains a
  stated team target, not an observed Windows performance result).
- Executed Node fixtures, Windows build, live pack import,
  and end-to-end MCP token/byte measurements.

No game launch, testing, installation, CI or external network
collection is enabled by this document.
