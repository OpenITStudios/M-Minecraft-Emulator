---
id: document.system.education-team-source-updates
class: DOCUMENT
domain: system
role: GUIDE
authority: CANONICAL
lifecycle: SOURCE_ONLY
---

# Minecraft Education: team ZIP update intake (local-first)

## One runtime source of truth

The operator's `minecraft-bedrock-agent-skills` modules own the real
`[ANALYTICS]` / `[GAME_RESULTS]` output and logging. The team's
`skills-probe` project owns its tests and installed module copies.
Lazy Tester owns only **read-only ZIP comparison**, bounded local
evidence storage and an optional Codex/MCP consumer. Never copy
the team's runtime module code into Lazy Tester.

The two archives can advance independently. An archived ZIP is a
point-in-time snapshot, not a live GitHub branch, version
attestation, complete Minecraft Education acceptance, or a
trusted executable. Do NOT automatically install/execute/sync
either archive, alter a world, or replace the team's module lock.

## Three small components

| Existing/new owner | Purpose |
| --- | --- |
| `Development/tooling/agent/team-update-zip.mjs` | Bounded ZIP32 reader: never extracts files; only STORE/DEFLATE and regular entries |
| `Development/tooling/agent/team-source-baseline.json` | The last **reviewed** pair of archive SHA-256 pins, team+probe module-tree digests, exact contract-source hashes, wire constants and upstream team tag/commit |
| `Development/tooling/agent/team-update.mjs` | Computes module/runtime deltas, checks installed modules against modules-lock.json and skills-lock.json, reports contract drift and stages one **local-only review report** |

`team-update.test.mjs` contains synthetic ZIP fixtures. They were
**authored only**; Node test execution, Windows runtime, and Minecraft
Education behavior remain NOT_EXECUTED.

Baseline initial snapshot: team skills v0.35.0, upstream commit
`1856f1875ef0fb7039710c3a215da2123f87e7ba` and the two ZIP
hashes documented in `education-team-telemetry-compat.md`. This
baseline is a **source-review pin**, never an assertion that the
current remote team repository still has the same commit.

## Normal update procedure

1. Receive the updated `minecraft-bedrock-agent-skills` ZIP and
   the updated `skills-probe` ZIP from the developer. Keep both on
   the operator's local machine. Do not send all source to Codex.
2. From the Lazy Tester `Development/` directory, when source
   execution is separately approved, run:

       npm run team:update:inspect -- --team "C:\absolute\minecraft-bedrock-agent-skills.zip" --probe "C:\absolute\skills-probe.zip"

3. Review the compact status printed in the terminal. The full
   **metadata-only** report is written under the ignored
   `workspace/diagnostics/team-updates/review-<24-hex>.json`.
   The report contains fixed status codes, module names/hashes,
   a proposed metadata pin if reviewable, and no raw TS/JS,
   source path, account/credential/world data or console logs.
4. If source changed, a human checks the developer's changelog,
   `modules-lock.json`, `skills-lock.json`, Skills Probe's
   `packs/BP/manifest.json` (minimum Education engine and
   `@minecraft/server` version), affected team contract and map wiring.
   A staged `proposedBaseline.reviewState` is
   `CANDIDATE_NOT_APPROVED` and **cannot** be used as an approved
   pin. Only after review may a developer deliberately set
   `reviewState` to `APPROVED_SOURCE_REVIEW` in the committed
   baseline. The command never promotes or commits its own report.
5. The developer runs `bun run skills:check` and
   `bun run modules:check` **in the Skills Probe/project repo**.
   If a deliberate update is needed, use that project's existing
   `bun run skills:sync` or `bun run modules:sync` command, then
   verify and commit the team project's own lockfile/source.
   The standalone team modules repo remains the canonical source.
6. Only after source and gameplay-contract review: update the
   tracked `team-source-baseline.json` pin in Lazy Tester and,
   if necessary, adjust the existing MCP parser or the
   `/function help` list to the new upstream command IDs.
   Never make a second in-game collector.
7. A Minecraft Education **Windows** test on an isolated world
   is still required before claiming runtime compatibility.

## Status semantics

| Report status | Meaning / response |
| --- | --- |
| `PINNED_SNAPSHOTS_MATCH` | Both ZIPs match the approved metadata and installed locks; **not gameplay PASS** |
| `REVIEW_REQUIRED` | Compatible pinned wire constants; source/lock/archive changed. Read the local report and review affected modules |
| `BLOCKED_WIRE_CONTRACT_CHANGE` | Analytics/scoring command, marker, size, pacing or scored-record types changed. Update the reader only after deliberate versioned review |
| `BLOCKED_EDUCATION_API_CHANGE` | Skills Probe's Minecraft Education minimum engine version or `@minecraft/server` dependency changed. Check target runtime support before adoption |
| `BLOCKED_CONTRACT_SOURCE_REVIEW` | A canonical contract file changed even if the extracted constants stayed the same. The record shape may have changed |
| `BLOCKED_TEAM_PROBE_DESYNC` | Team module runtime hashes differ from the copies installed in Skills Probe. Re-sync only through the team's existing workflow |
| `BLOCKED_PROBE_MODULE_LOCK` | Skills Probe copied module bytes do not match its own recorded lock, or team tag/commit locks disagree. Investigate before use |
| Input refused | Missing/unsupported, malformed, oversized, encrypted, duplicate-path or unsafe ZIP. No files are changed |

A contract mismatch is **never** auto-adapted using guesses about the
new TypeScript or JSON payload. A source-compatible change still
needs explicit review before updating the approved baseline. New
ZIPs may contain unrelated doc, test, asset or dependency changes:
these should be reported without dumping their contents or
treating them as gameplay failures.

**Jika hanya satu ZIP yang diperbarui:** gunakan ZIP kedua dari
snapshot terakhir untuk pemeriksaan awal. Jika kode modul yang
diinstal sudah berbeda, laporannya akan berstatus
`BLOCKED_TEAM_PROBE_DESYNC`. Mintalah developer melakukan
sinkronisasi resmi di Skills Probe, lalu kirim ulang pasangan ZIP
yang konsisten. Jangan menyamakan versi dengan mengganti nama
file atau mengedit lock secara manual.

## Resource/privacy gates

- ZIP archives must be explicitly supplied via absolute local paths.
  No GitHub fetch, file watcher, background job or updater daemon.
- Each ZIP is ≤64 MiB; ZIP32 entry count ≤2500; central
  directory ≤2 MiB; individual relevant decompressed source files
  ≤128 KiB. Encrypted, symlinked and unsafe names are refused.
- Compare only the selected team module .ts/.js/.mjs/module.json files,
  relevant contract/runner and real Skills Probe locks. The module
  README difference in the historical snapshots is ignored for
  runtime parity (but a changed ZIP still receives review status).
- Skills Probe's `database` may have a separate canonical upstream
  repository; its own module lock still has to match the installed
  bytes. Do not incorrectly compare this external module to team
  source.
- The approved baseline also pins the Skills Probe manifest's
  Education minimum engine and stable `@minecraft/server` version.
  Changing either requires a distinct Education runtime review.
- The output JSON is capped at 24 KiB, locally stored, never
  overwritten with different bytes, and has a content-derived name.
  Console output returns only status and bounded module changes.
- No source code, full logs, screenshots, account IDs or game data
  automatically enter Codex. Existing MCP tools remain read-only.

## Boundary and next proof

This source-only implementation prepares a **safe review gate** for
future snapshots. It is not an automatic upgrade engine, not an
Education 26.33 debugger verification, and not a substitute for the
team's module installation / lockfile sync. An approved source pin
and a real Education gameplay test are separate decisions.
