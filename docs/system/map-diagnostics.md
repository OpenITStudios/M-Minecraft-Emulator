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

Three explicitly selected offline modes are supported:

- `envelope-survey`: inspect only the **shape of lines containing**
  `[LT_EDU_PROBE]`, counting exact-marker, timestamp-like and other
  prefixes, plus repeated/oversized markers. **Never parses marker bodies**,
  emits text snippets, or certifies a log transport.
- `marker-lines`: validate one complete `[LT_EDU_PROBE]{...}` line
  per record, only when no prefix exists.
- `timestamped-print`: validate the candidate
  `ISO_TIMESTAMP log|warn|... MESSAGE` wrapper, including escaped
  debugger PrintEvent text.

These are **candidate formats, not proven Education Content Log layouts**.
If a marker is present but the selected strict parser cannot understand its
wrapper, it reports `UNSUPPORTED_MARKER_WRAPPER_OR_CONTEXT` instead of
incorrectly reporting that no marker exists. Even a perfectly matched
two-marker pair remains `NOT_VERIFIED` as Minecraft Education evidence.

## Generate a complete disposable test-only behavior pack (source preparation)

The reviewed two-marker emitter stays in
`Development/tooling/agent/education-script-probe.js`. A new explicit
source-only stager, `prepare-education-probe.mjs`, can create an isolated
behavior-pack folder from this **one canonical source**, without duplicating
the emitter or opening any Minecraft directories.

After independently reviewing which **stable** `@minecraft/server`
module version is actually supported in the targeted Education build,
an operator may later run from `Development/`:

    node tooling/agent/prepare-education-probe.mjs prepare --server-version 1.17.0

The `1.17.0` value is a documented **Bedrock stable API candidate**, NOT
a verified Minecraft Education 26.33 module version. It may need adjustment
when actual Education compatibility is proven. The stager checks only stable
semver syntax, not whether that module loads in Education. Manifest
`format_version:2` declares one data module, one JavaScript `script` module
with `scripts/main.js` entry, and one explicit `@minecraft/server` dependency;
it does not select preview/beta capabilities. The test-only entry explicitly
calls the two-marker emitter when the pack loads. Repeated reloading can
produce duplicate markers: this intentionally fails strict two-marker
verification instead of hiding a session ambiguity.

The only output destination is **under the existing gitignored workspace**:
`workspace/diagnostics/education-probe/`. It is not an active Minecraft
profile or the private evidence archive:

    workspace/
      diagnostics/
        education-probe/
          edu-<12-hex-run-id>/
            receipt.json         -- prepared last, local hashes/status
            pack/
              manifest.json      -- unique header/module UUIDs
              scripts/
                main.js          -- reviewed emitter + ONE explicit invocation

The CLI returns the generated safe `run` ID and relative folder path for
future manual installation. Every invocation creates new random run/pack
identity; there is no overwrite, prune, zip/upload, service, account
lookup, Minecraft launch or behavior-pack activation. A partially staged
folder stays reserved for manual inspection; no automatic cleanup occurs.

**Runtime verification remains a separate approval:** the operator must
use an isolated, disposable **Minecraft Education test-copy world**, activate
only the generated `pack/` folder and inspect the actual log output/path
using an independently authorized Windows session. Never install the
generated probe into original world data, production resource packs or
the five retained account/profile directories automatically.

The generated receipt and matching run marker help correlate a deliberately
staged test source with a captured pair, but they **do not authenticate**
Minecraft player, host/server, world, device or profile. Those must be
verified independently before a live collector can trust the source.



The JavaScript probe is only emitted if a compatible behavior pack
actually loads and invokes the code. Staging a folder is not a test run.
The packaging source can be reviewed independently of the runtime test.

Once a bounded local capture is explicitly saved to a trusted absolute
path, first survey its envelope from `Development/`:

    node tooling/agent/education-log-probe.mjs inspect --source "C:\absolute\education-capture.log" --run edu-probe --format envelope-survey

Survey output contains only fixed counters and a source SHA-256. If a
previously documented explicit format is applicable, independently inspect
the same private file using `--format marker-lines` or
`--format timestamped-print`. An unknown prefix is **not** automatically
accepted or guessed. Review a deliberately redacted example before
adding any Education-specific parser.

The inspector accepts only absolute `.log` / `.txt` ordinary files, refuses
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

## Universal lightweight diagnostics pack — same .mcpack for multiple maps

**Team-contract ownership:** See
[Education team telemetry compatibility](education-team-telemetry-compat.md).
Team `[ANALYTICS]` and `[GAME_RESULTS]` remain canonical; the universal
`[LT_EDU_DIAG]` marker is an optional generic fallback. Their counts
are not automatically merged or treated as game acceptance.
The existing local MCP parser is now prepared to read all three
markers from an explicitly configured, identity-unverified capture file,
without sending raw records or exceeding the team's 24 KiB wire contract.
**It does not automatically collect Education Content Logs.**



**This is the recommended general QA pack**; the separately named
`education-script-probe.js` and `prepare-education-probe.mjs` remain narrow,
disposable **transport-verification tools**, not the generic runtime collector.

Source owners:
- `Development/tooling/agent/education-universal-pack/manifest.json`
- `Development/tooling/agent/education-universal-pack/scripts/main.js`
- `Development/tooling/agent/build-education-universal-pack.mjs`

The universal behavior pack has stable header and module UUIDs, pack version
1.2.0, manifest format 2, minimum engine 1.26.30 and stable
`@minecraft/server` 2.8.0. The package UUID remains constant while
its version advances from 1.0.0 through 1.1.0 to 1.2.0; do not confuse these as
separate map-specific packs. Those version choices are grounded by the
operator-supplied `skills-probe-main` archive, whose BP manifest targeted
Education 26.32 / engine 1.26.30 with the same Script API. This is
**not** proof of Education 26.33 Windows compatibility. The same source
pack can be activated in more than one world without generating a UUID,
run ID or new build per map. Updating pack source in the future must
increment its package version; do not silently overwrite an installed
pack with different bytes under the same version.

Prepare exactly once, from `Development/` when local tool execution is
separately authorized:

    node tooling/agent/build-education-universal-pack.mjs prepare

The builder reads only the reviewed local source pack and writes under
`workspace/diagnostics/education-universal/v1.2/`, ignored by Git, using
exclusive creation. It generates a small ZIP STORE `.mcpack` containing
only root-level `manifest.json` and `scripts/main.js`. Repeating the
command accepts only byte-identical existing output; it never overwrites
or deletes an existing file. The prebuilt `.mcpack` shared in the
conversation uses the same exact source entries. The repository does not
commit binaries, run installs, or silently publish packages.

**Using the same file across test maps:**

1. Import `LazyTester-Universal-Diagnostics-Education-1.2.0.mcpack`
   into Minecraft Education, then activate `Lazy Tester - Universal Diagnostics`
   under Behavior Packs on each **copied QA world**. The old v1.1.0 and
   new v1.2.0 share a pack UUID; only one version should be active.
2. With cheats/operator permission, type `/scriptevent lt:diag help`.
   Instructions are shown in-game. No VS Code, new pack per map,
   or secondary manager is needed.
3. Start the test with `/scriptevent lt:diag on`; finish with
   `/scriptevent lt:diag off`. These are the only commands necessary
   for a normal QA session.
4. Optional: `/scriptevent lt:diag status` reports the current state.
   `/scriptevent lt:diag more` enables combat+entity detail for a maximum
   of 600 game ticks (approximately 30 seconds at 20 TPS), then expires.

The old developer command ID `/scriptevent lazytester:diagnostics`
still supports all previous payloads, including `start`, `stop`,
`status`, `flush`, `combat on/off` and `entities on/off`.
Both IDs use the same capture state and produce the same
`[LT_EDU_DIAG]` schema; no separate analytics engine exists.

**In-game feedback:** help/status/on/off/more show a brief reply only
when requested. Script event sources do not always identify a player,
so a feedback line may be broadcast to the QA world. The pack does
not emit periodic chat, read names, or copy chat into its telemetry.
`world.sendMessage` is a stable Bedrock API; Education 26.33 Windows
runtime compatibility still needs to be verified.

The pack emits only explicitly namespaced `[LT_EDU_DIAG]` console markers.
On import/initial load it prints one `ready` marker; **collection stays
OFF until `start`**. Normal collection subscribes only to lower-volume
lifecycle events: player spawn/respawn, player leave, dimension change
and **player-filtered** deaths. Combat damage and broad entity events
are **off by default**; an operator can enable each temporarily for
600 game ticks. Repeated event data is aggregated into fixed counters,
not printed event-by-event. The pack prints at most one nonempty
periodic summary per 200 game ticks (nominally 10 s at 20 TPS),
plus a heartbeat every 600 ticks and explicit control messages. No world/player inventories, player
names/IDs, coordinates, message text, raw chat, other packs' private
JavaScript variables, scoreboard, or per-tick entity scanning is involved.
An unsupported event is disclosed as a safe event name, not silently
declared valid. The `session` field is a nonsecret **correlation token**,
never an authenticated world/profile ID.

**Coverage boundary:** This generic pack can describe basic activity but
cannot validate tower defense wave rules, shop/coin transactions, arena
isolation, entity pathfinding decisions, or arbitrary script exceptions
inside unrelated packs. Map-specific logic still requires voluntarily
instrumented, versioned assertions or the developer's existing
`[ANALYTICS]` and `[GAME_RESULTS]` exports. An entity count or a damage
count is *not* evidence of a bug by itself.

**Local-first boundary:** A `.mcpack` is not a Windows log collector.
Output becomes available locally only through a Content Log or optional
debugger capture transport separately proven for the Education build.
Neither this pack nor its builder opens a TCP socket or sends data to
Codex. The existing MCP keeps its read-only/issue-on-demand behavior;
this new marker is not automatically imported or presented as a verified
test result. Any raw Content Log may contain unrelated sensitive data:
never dump an unredacted raw log directly into an AI prompt.

### Relevance of the team's uploaded Bedrock Skills and Skills Probe

The operator-provided archives `minecraft-bedrock-agent-skills-main` and
`skills-probe-main` contain implementation patterns relevant to QA:
a single channel-based throttled logger, event-driven counters, staged
and paginated analytics exports, bounded `[ANALYTICS]` and
`[GAME_RESULTS]` contracts, diagnostics/profiler capture, and small
scenario-specific test assertions. Those are architecture references;
they were **not copied wholesale** into the portable universal pack.
Importing a broad system would increase per-world resource and
compatibility risks.

The included `docs/test-runs/2026-10-28-debug-log-capture-run-1.md`
reports a direct TCP debugger capture from **Minecraft Education 26.32
on macOS**, with negotiated protocol 7 and repeated complete 176-message
exports. The archives contain the corresponding raw capture file.
This is stronger evidence for a future **optional local debugger
collector without VS Code** than relying on general Bedrock docs.
It remains a team report with scope limited to that client/OS/world,
not Windows 26.33 or any five-profile run. The benchmark documents
also show a large 285 KB analytics export printing in one tick caused
roughly 472–486 ms of measured wall-clock work, reinforcing our
event aggregation and paced export design. Neither measurement is a
benchmark of the new universal pack.

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
