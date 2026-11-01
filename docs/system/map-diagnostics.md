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

Minecraft **Education** is the runtime target. The source-only diagnostics
feature provides **explicit local import of normalized diagnostic JSONL**,
immutable per-run archives with a commit marker and SHA-256 integrity fields,
bounded issue-candidate detection and four read-only MCP queries. It does **not** claim Education Content Log, native debugger,
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

## Minecraft Education Script API log-path verification (source only)

**Next narrow technical gate.** An optional test-only probe pair now exists:

- `Development/tooling/agent/education-script-probe.js`: a tiny
  `@minecraft/server` source module whose explicit call emits exactly
  two `[LT_EDU_PROBE]` JSON markers using `console.warn()` and
  `system.currentTick`. The second marker is scheduled 20 ticks later.
  It does **nothing on import** and must not be used in production maps.
- `Development/tooling/agent/education-log-probe.mjs`: an **operator-only,
  read-only** offline capture validator. It reuses the same stable ordinary
  file reader as the local importer. It never searches Education's folders,
  connects to a debugger, writes a run, or calls Codex.

The two explicit capture format modes are:
`marker-lines` (one complete `[LT_EDU_PROBE]{...}` line per record),
and `timestamped-print` (a candidate `ISO_TIMESTAMP log|warn|... MESSAGE`
wrapper, including escaped debugger PrintEvent text). **These are probe
input formats only.** Neither has been demonstrated as Education's real
Content Log format. Unrecognized/oversized/malformed data is never
converted to a gameplay success or copied into an AI response.

The JavaScript probe has no manifest and is not an installed behavior
pack. Only after a separately authorized Education test, an operator may
place it in an isolated **test-copy** pack with a compatible stable
Script API module, explicitly call
`emitEducationScriptLogProbe('edu-probe')` once, and collect whatever real
Education logging mechanism is observed. The original map/world and private
game profile must remain unchanged.

Once a bounded local capture is explicitly saved to a trusted absolute
path, the optional offline inspector can be invoked from `Development/`:

    node tooling/agent/education-log-probe.mjs inspect --source "C:\absolute\education-capture.log" --run edu-probe --format timestamped-print

Or use `--format marker-lines` for a separately extracted marker-only
capture. It accepts only absolute `.log` / `.txt` ordinary files, refuses
linked/hardlinked source files, requires a complete newline-terminated
capture and reads at most **4 MiB / 12,000 lines / 4,096 bytes per line**.
It returns only fixed status/counters, a source SHA-256 and a validated
nonsecret run ID. It never prints raw lines, player names, usernames,
account identifiers, original source paths or exception messages.

`CANDIDATE_MARKER_PAIR_FOUND` means only that two well-formed source
strings with increasing tick numbers were recognized in the **operator
supplied file**. It is not proof of a real Education log transport or
world attribution. Mismatches, missing markers and unsupported wrappers
remain `NOT_VERIFIED`. Live watching, automatic ingestion and the MCP
interface **remain unchanged** until real source paths and provenance
are independently verified.

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
