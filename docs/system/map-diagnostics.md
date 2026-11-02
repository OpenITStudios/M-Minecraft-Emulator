---
id: document.system.map-diagnostics
class: DOCUMENT
domain: system
role: GUIDE
authority: WORKING
lifecycle: SOURCE_ONLY
---

# Minecraft Education local-first diagnostics

## Scope and proof

Minecraft **Education** is the runtime target. The existing
**local evidence reader** provides explicit import of normalized diagnostic
JSONL into versioned run archives with SHA-256 integrity checks, a
bounded issue index, and four read-only MCP queries. This reader is
NOT another Minecraft analytics module, and it never directly opens
worlds, launches Minecraft, or calls Codex.

The separate optional in-game addon is **help-only**; it does not
produce telemetry. No background observer, automatic Minecraft
Content Log ingestion, debugger socket, network upload, native input
or game-world mutation is enabled. Education 26.33 Windows runtime
compatibility is still NOT_VERIFIED.

Owners:
- Development/tooling/agent/diagnostics-local.mjs: local evidence import,
  validation, archive, summaries and issue evidence queries.
- Development/tooling/agent/mcp.mjs: existing one MCP stdio server, delegates
  strictly bounded read-only queries when a local directory is configured.
- Original five-profile native controller, visual observer, and existing
  [ANALYTICS]/[GAME_RESULTS] reader remain unchanged.

## Normalized JSONL v1 (NOT a raw Education content log)

One complete finite JSON object per line, newline-terminated. Each record
requires schema=1, a nonsecret run slug, strictly increasing seq, a
nondecreasing game tick and a kind:

| Kind | Fields | Meaning |
| --- | --- | --- |
| assertion | check slug, expected, actual | equality observation |
| script_error | symbolic code | error candidate, not raw stack trace |
| telemetry_gap | positive count | lost-record candidate |
| state_change | check, expected, actual | previous and new state |
| metric | check, numeric actual | local metric sample |

Optional arena is a nonsecret validated slug. Expected and actual are
restricted to bounded numeric/boolean/scalar enum tokens, not arbitrary
strings or objects. Unsupported fields, secrets, filenames, game account
data or unstructured logs are rejected at import.

Example fixture (not observed Minecraft data):

    {"schema":1,"run":"td-test","seq":1,"tick":100,"kind":"state_change","arena":"arena-02","check":"wave.phase","expected":"WAITING","actual":"RUNNING"}
    {"schema":1,"run":"td-test","seq":2,"tick":101,"kind":"assertion","arena":"arena-02","check":"flag.player_damage_blocked","expected":350,"actual":330}

Assertion mismatches, symbolic script errors, telemetry gaps and missing
sequence numbers produce grouped candidates. Matching assertions count
as matched observations, **never as accepted gameplay PASS**. The source
is explicitly operator-imported; world/actor identity remains UNVERIFIED.

## Explicit local import (not triggered by Codex)

After a trusted application Data directory exists and operator-authorized
local tooling is available, execute from the Development directory:

    node tooling/agent/diagnostics-local.mjs import --source "C:\absolute\capture.jsonl" --store "C:\Lazy-Tester\Data\Diagnostics" --run td-test

The importer validates a stable ordinary source file **before writing**
and then creates one exclusive run directory. The actual storage layout for
new **v2 run archives** is:

    Data/
      Diagnostics/
        runs/
          td-test/
            manifest.json            -- published LAST, commit marker
            evidence/
              events.jsonl            -- original validated normalized bytes
            analysis/
              issues.json             -- grouped candidate index (not raw logs)

The manifest records the run ID, storage schema (2), import timestamp,
source classification, immutable/manual-retention status, byte counts and
SHA-256 digests for both archived files. Per-run overview includes import
time and logical event+index bytes, without opening the entire event file;
these are not physical disk-usage measurements. Use descriptive nonsecret
run IDs such as dgn-defense-20261102-r01 (maximum 48 characters). The timestamp records the local
import time, **not** a verified game time. The SHA-256 fields detect damaged
or unexpectedly changed local files, **not** authenticity of a Minecraft
Education game session.

Each file is written with an exclusive-create flag. Data files are flushed
before a temporary manifest is renamed to manifest.json as the final commit
marker. An interrupted import leaves a reserved but uncommitted run
directory; no MCP query treats it as completed. The overview reports the
number of incomplete run entries without reading or transmitting their
contents. Recovery and deletion remain manual.

**Compatibility:** v1 archives from the previous revision remain readable
in their original `runs/<id>/events.jsonl` and `runs/<id>/index.json` layout,
without silent migration. Such records are marked
`LEGACY_NOT_CHECKSUMMED` because they do not contain a committed
manifest and source digest. No old archive is rewritten.

Git already ignores Data/. The importer refuses symlinked Data parents;
no files in world, account, profile or existing run directories are
modified or automatically removed. The source input is not deleted.

## Limits: local storage and model context are separate budgets

| Boundary | Source-side limit |
| --- | --- |
| Captured import | 4 MiB, 10,000 records, 4,096 bytes per line |
| Computed index | 256 KiB; reject oversized imports |
| Commit manifest | 4 KiB; fail closed if invalid or missing on v2 |
| Run archives | 64 directories including interrupted imports; no automatic prune/delete |
| Issue sample cache | At most 8 selected records |
| MCP overview | Up to 10 run IDs, sorted lexically, not by recency |
| MCP issue page | At most 10 issue candidates |
| MCP evidence page | At most 8 projected records |
| AI | Zero calls on import; only model-requested MCP tool results |

An incomplete v2 run is counted but not exposed as a committed run;
its identifier stays reserved until explicit, separately reviewed recovery.
No silent migration or automatic deletion is performed. Future gameplay
volume beyond these prototype limits needs separately measured segmentation.

Quotas fail closed; rejected sources remain local, not silently truncated.
MCP never returns full JSONL captures, raw error text or arbitrary file
contents. It does not enumerate or open an unapproved path.
For a selected v2 run, metadata-only issue queries validate the manifest
and index SHA-256; a deeper evidence query separately checks the archived
JSONL digest. Evidence records are projected through an allowlist and
paginated, not passed through unchanged. Normal evidence queries scan a
bounded <=4 MiB archive but hold only one requested page of projected
records in memory; no persistent background database is necessary yet.

## Codex connection is optional and read-only

Start the existing MCP with --diagnostics-dir pointing at an absolute
Data/Diagnostics folder, or set LAZY_TESTER_DIAGNOSTICS_DIR. If unset,
new diagnostic tools report NOT_CONFIGURED; no directory is discovered
or created. A remote model requires a separately configured, permitted
local MCP connection. Local files are not automatically uploaded.

The four read-only MCP tools are:

1. get_diagnostics_overview: run summary or run list.
2. list_issue_candidates: bounded, paginated issue summaries.
3. get_issue_details: one issue and at most eight sanitized samples.
4. query_diagnostic_evidence: at most eight relevant projected records
   per page, using issue ID rather than arbitrary filesystem paths.

Codex should start from the overview, select an issue, then request
specific evidence only if necessary. Source proofs do not establish
actual Minecraft player, arena, world or gameplay outcomes. A committed
archive means "the importer finished writing local files", **not** that
the map's mechanics passed any test. An overview's issue-index checksum
does not imply the full evidence file was read: this is reported separately.

## One Minecraft team source, one optional in-game help

The uploaded `minecraft-bedrock-agent-skills` and `skills-probe` ZIPs,
rather than the retired Lazy Tester Universal Diagnostics implementation,
are now the canonical game-analytics and validation sources. See
[team command and source mapping](education-team-telemetry-compat.md)
for exact file hashes, module ownership, commands and update procedure.

**Retired source-only tooling:** the older universal diagnostic
counter pack, its `[LT_EDU_DIAG]` wire schema and the separate
disposable two-marker probe generator have been removed from the
active source tree. Historical commits preserve their source; old
already-imported .mcpack files must be disabled manually in QA
worlds to avoid a parallel telemetry system. No accounts/worlds
are opened or modified by this source correction.

### One help-only behavior pack — /function help

The optional **Lazy Tester - Team Commands Help v2.1.0** retains the
former Lazy Tester pack's UUID and raises the version for a clean
upgrade. It is **data-only**: no Script API module or dependency,
JavaScript, listeners, diagnostic counters or telemetry. This is
compatible in structure with the `.mcfunction` approach used by
the team's Skills Probe archive.

Minecraft reserves `/help` for built-in help and it cannot be safely
overwritten by a behavior pack. After the new helper is imported and
activated on an isolated test-copy world, the operator can enter:

    /function help

The command executes `functions/help.mcfunction`, a tiny alias for the
single canonical help list at `functions/lazytester/help.mcfunction`.
If another behavior pack or map already defines `help`, use the
collision-resistant fallback:

    /function lazytester/help

The one list displays these **team-owned** commands through
`tellraw @s` (only the command executor, not all world players):

- `/scriptevent mivubi:get_analytics` — team analytics export
- `/scriptevent daigon:get_all_scoring` — team game-results export
- `/scriptevent probe:list` — Skills Probe only, admin
- `/scriptevent probe:run auto` — Skills Probe only, admin, test world
- `/scriptevent probe:report` — Skills Probe only, admin

Each group explains that its team module must actually be installed.
The helper never installs those modules, provides its own scorer,
or executes any of the listed commands. `/function` requires
cheats/operator permission; availability and `tellraw @s` context
still need Education 26.33 Windows runtime verification.

Source:
- `Development/tooling/agent/education-team-help-pack/manifest.json`
- `Development/tooling/agent/education-team-help-pack/functions/help.mcfunction`
- `Development/tooling/agent/education-team-help-pack/functions/lazytester/help.mcfunction`
- `Development/tooling/agent/build-education-team-help-pack.mjs`

When source execution is separately approved, stage a reusable
.mcpack from `Development/`:

    node tooling/agent/build-education-team-help-pack.mjs prepare

The source-only builder checks three fixed archive entries and stores
the output under ignored
`workspace/diagnostics/education-team-help/v2.1/`. It refuses
overwriting non-identical files and touches no installed Minecraft
world, profile, account or server. The previously produced v2.0
Script API help pack should not remain active alongside v2.1.
No per-map rebuild is necessary; activation remains per world.

### How developer analytics are actually enabled

They are *not* enabled through the help addon. The map developer
uses the team's existing `tools/sync.mjs`, module prerequisites,
and project entrypoint to initialize the original
`modules/analytics` / `modules/game-results` code. The team's own
`bun run skills:check` provides copy/lock verification; the
Skills Probe project validates behaviors in its isolated world.

A drop-in .mcpack cannot reliably infer private wave, shop,
scoreboard, coin, arena or other map-script state without an
adapter authored in that map. That is an explicit engineering
boundary, not something the help-only pack will conceal.

## Proof boundary and next runtime gate

- Active GitHub source and its owner/consumer references can
  be reviewed without running Windows or Minecraft Education.
- Source-authored fixtures are NOT_EXECUTED.
- Minecraft Education 26.33 Windows support, in-game help/chat,
  Script API module loading and actual Content Log transport
  remain NOT_VERIFIED.
- No new automatic log capturing, debugger connection, app
  invocation, player/world access, network listener or Codex
  token usage is introduced.
- When separately approved, verify the *team's* Skills Probe
  scripts and original team modules on an isolated Education
  test world before accepting real telemetry.
