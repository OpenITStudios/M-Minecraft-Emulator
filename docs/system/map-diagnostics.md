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

Minecraft **Education** is the runtime target. The first source-only slice
adds **explicit local import of normalized diagnostic JSONL**, an immutable
private archive, bounded issue-candidate detection and four read-only MCP
queries. It does **not** claim Education Content Log, native debugger,
Script API output, world/session attribution, multiplayer or gameplay
compatibility. No background observer, network server, Codex upload,
in-game add-on, native input, world mutation or Windows execution is enabled.

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

The importer validates a stable ordinary source file before any write and
then creates, never overwrites, the following private archive:

    Data/Diagnostics/runs/td-test/events.jsonl  -- full validated import
    Data/Diagnostics/runs/td-test/index.json   -- grouped issues

Git already ignores Data/. The importer refuses symlinked Data parents;
no files in world, account, profile or existing run directories are
modified. Interrupted partial imports are unavailable as accepted evidence
and require separate operator review before any cleanup.

## Limits: local storage and model context are separate budgets

| Boundary | Source-side limit |
| --- | --- |
| Captured import | 4 MiB, 10,000 records, 4,096 bytes per line |
| Computed index | 256 KiB; reject oversized imports |
| Run archives | 64 directories; no automatic prune/delete |
| Issue sample cache | At most 8 selected records |
| MCP overview | Up to 10 run IDs, sorted lexically, not by recency |
| MCP issue page | At most 10 issue candidates |
| MCP evidence page | At most 8 projected records |
| AI | Zero calls on import; only model-requested MCP tool results |

Quotas fail closed; rejected sources remain local, not silently truncated.
MCP never returns full JSONL captures, raw error text or arbitrary file
contents. It does not enumerate or open an unapproved path.

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
actual Minecraft player, arena, world or gameplay outcomes.

## Technical gates before live Education capture

1. Separately verify Minecraft Education stable Script API behavior pack
   execution and the actual log transport/path on the target build.
2. Verify immutable native session-to-profile and world/actor attribution;
   a file called MC02.log alone is not authoritative.
3. Build a map-side normalized emitter on a **test copy** using only
   supported Education APIs, then measure overhead.
4. Benchmark event sampling, missing/dropped records, local disk usage,
   time-to-issue and token consumption against representative maps.
5. Consider native debugger, live diagnostics and profiler only with an
   explicitly verified Education protocol/capability handshake.

GitHub source and authored test fixtures only. Tests, Windows builds,
Education runtime and gameplay **NOT_EXECUTED / NOT_VERIFIED**.
