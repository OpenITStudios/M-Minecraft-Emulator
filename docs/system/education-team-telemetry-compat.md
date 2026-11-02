---
id: document.system.education-team-telemetry-compat
class: DOCUMENT
domain: system
role: GUIDE
authority: CANONICAL
lifecycle: SOURCE_ONLY
---

# Minecraft Education: one canonical team source

## No duplicate analytics or debugging engines

The operator's canonical Minecraft runtime implementation is the
`minecraft-bedrock-agent-skills` team repository. `skills-probe` is the
separate validation project. Lazy Tester is an optional local
evidence reader/MCP bridge, **not** a second source of analytics,
scoring, game-state truth, or in-game telemetry.

Earlier Lazy Tester Universal Diagnostics versions 1.0.0–1.2.0 and
their `[LT_EDU_DIAG]` data format are **RETIRED**. Old source remains
in Git history only; old previously distributed .mcpack files must
be disabled in copied QA worlds. No original world is modified or
migrated automatically.

The sole optional Minecraft-side helper is **Lazy Tester - Team
Commands Help v2.1.0**, now a **data-only behavior pack with Minecraft
function files** and NO Script API, JavaScript module, dependency or
event subscriber. The only normal operator command is:

    /function help

This executes the tiny alias `functions/help.mcfunction`, which
delegates to the one canonical command list in
`functions/lazytester/help.mcfunction`. If a map already defines its
own function named `help`, operators may instead use the unique
`/function lazytester/help` path. These are two names for ONE static
list, not two alternative analytics implementations.

The command list uses `tellraw @s` to show the original team commands
and their prerequisites to the invoking player, without broadcasting
to all players. It never exports logs, runs tests, scans a world,
or calls Codex. `/help` remains Minecraft's native command and is NOT
replaced or modified. `/function` requires cheats/operator permission
in the copied QA world.

## Commands verified from the user-supplied source ZIPs

| Command | Canonical source | When it works |
| --- | --- | --- |
| /scriptevent mivubi:get_analytics | modules/analytics/contract.ts and analytics-export.ts | Map must initialize the team analytics module |
| /scriptevent daigon:get_all_scoring | modules/game-results/game-results-contract.ts and game-results-export.ts | Map must initialize the team results adapter |
| /scriptevent probe:list | Skills Probe: packs/BP/scripts/probe/runner/runner.ts | Skills Probe isolated test world only; admin |
| /scriptevent probe:run auto | Same Skills Probe runner | Skills Probe only; admin; may alter test setup |
| /scriptevent probe:report | Same Skills Probe runner | Skills Probe only; admin |

The first two commands export output to the server Content Log;
the help-only pack does not pull, parse or forward that output.
`probe:list` prints the detailed list of probes in the Content Log,
not in a universal Minecraft production world. Its in-game status
may be restricted to developer builds.

A help-only pack **cannot** make team commands work in arbitrary maps
where the needed modules were never installed or initialized.
It does not recreate map-defined stages, coins, flags or scoring.

## Canonical installation and update path

Follow the team's actual `modules/README.md` and `README.md`:

- Install a module in a map with `bun run modules:add analytics` or
  `bun run modules:add game-results`, when its prerequisites exist.
- Check pinned skills/modules with `bun run skills:check`.
- Explicitly update project copies with `bun run skills:sync` or
  the team `bun run modules:sync --from <team-clone>` workflow.
- Do not edit the copied project modules directly. Fix the team
  source, release it, then sync projects using `tools/sync.mjs`.

Analytics requires the team's database/time/logging integration and
the map's own session/stage ownership. Game-results requires the
map's adapter. Copying either module by itself is not proof that
a map exports complete or correct gameplay evidence.

## Exact snapshot comparison / provenance

The two original user-supplied ZIPs are snapshots, not evidence that
the private team repositories have not changed since upload.

- `minecraft-bedrock-agent-skills-main (3)(1).zip` SHA-256:
  26c0a322cabed172ca74970413d5bb7a20d6f67181446df5f688e36057c54e21
- `skills-probe-main (2)(1).zip` SHA-256:
  462fb6fe48b60f2bcf255f59d7ce0cb810c3a7ec598102c9e3888c68b341653b

Of 40 matching module-relative paths in both ZIPs, 39 are
byte-identical; the differing module-relative path is
`modules/README.md`. The full archives are NOT identical. The
earlier universal diagnostics implementation was NOT an exact copy.

Canonical command-source checksums:
- `modules/analytics/contract.ts`:
  ef43cd5a6af36c266664e7a0f935bb510df9e853712105d82eed9de63ee45d16
- `modules/game-results/game-results-contract.ts`:
  0fc57cac2d30d7a3bf1c2eeb14146700662cde79eabf94babd7f451f7bdbb7a4
- Skills Probe `packs/BP/scripts/probe/runner/runner.ts`:
  d2fd141a833d91e9cc923dc8e41552574b81c58917a5293cdd0d7935281fd796

If an upstream command changes, review the newer team source, update
the minimal help text and its authored fixtures, and resync the
project through team tooling. Do not fork the team analytics engine.

## Lazy Tester local-only consumer

The existing MCP parser recognizes only team `[ANALYTICS]` and
`[GAME_RESULTS]` markers. Their maximum source wire size is 24 KiB;
analytics export is paced at 8 lines/tick, and game-results
`level_score`, `gameplay_result` and `export_summary` remain distinct.
Failure counts are preserved as signals, never claimed gameplay PASS.

The local `Data/Diagnostics` store and optional MCP bridge are
**evidence readers**, not in-game analytics engines. Raw account,
player and world identity must remain private, and MC01–MC05 log
filenames alone never establish authoritative game source identity.
Codex is not called automatically and receives only explicitly
requested bounded summaries/evidence.

The team Skills Probe's direct debugger protocol-7 capture was
reported on Education 26.32/macOS; it has not verified Education
26.33 Windows or five simultaneous clients.

## Verification boundary

No Node test suites, local package builds, Minecraft Education
execution, Windows profile access, CI or live telemetry ingestion
were performed at this revision. In-game help and pack compatibility
still need separate acceptance on a disposable QA world.
