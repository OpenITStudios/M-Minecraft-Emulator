# Current Development Handoff — Lazy Tester

## AI QA source-continuity and exact-target handoff — 2 November 2026

**State: SOURCE_ONLY / NOT_EXECUTED.** The existing single `probe_qa`
tool now returns an exact-window, read-only `targetHandoff` when the
original controller verifies PID/HWND/process birth. No new target
resolver, command dispatcher or user procedure was added. On every
subsequent read, `probe_qa` rechecks the same approved ZIP, runner SHA
and complete auto-ID catalog using the existing cheap cache; any
source change or missing ZIP invalidates the in-memory plan and
blocks acceptance. The source catalog contract is explicitly
validated before a new plan is stored.

The result of an unbound target cannot become a `PROBE_REPORTED_PASS`;
it is marked INCOMPLETE while preserving the original limited
Probe counts for troubleshooting. Replacement of a previously pinned
game process also downgrades the outcome. These are source-level
trust gates, not proof that the capture came from the chosen world or
that the admin Player executed any command. Tests were AUTHORED but
NOT_EXECUTED. No Windows/Node test, Minecraft runtime, AI input,
debugger connection, CI, installer, or world/account mutation occurred.

**Next requiring separately authorized Windows QA:** validate one
original Player/admin Skills Probe command and prove a profile/world
capture binding; only then consider connecting a command sender to
the existing authenticated controller.

## Consolidated single-entry AI Gameplay QA — 2 November 2026

**State: SOURCE_ONLY / NOT_EXECUTED.** AI now has one recommended
`probe_qa` MCP tool, defaulting to MC02. First call composes the existing
reviewed ZIP/capture preflight with optional read-only native PID/HWND
target admission, while returning BLOCKED rather than claiming an
authorized game command. Repeated calls read the same session's bounded
original Probe markers without passing a session ID; `restart=true`
explicitly starts a new source/capture checkpoint. The prior
`prepare_probe_qa` and `read_probe_qa_result` remain available unchanged
for callers already using them.

The approved ZIP catalog is cached in memory only when filename, stable
file identity, byte size, mtime/ctime and exact approved baseline bytes
still match. A changed source is revalidated by the existing ZIP reader;
nothing is automatically synced, installed, executed or approved.
Compact MCP output keeps FAIL, NEEDS_REVIEW, WAITING, INCOMPLETE and
Probe-reported PASS distinct from actual Minecraft gameplay acceptance.
Existing strict native target validation is reused; target changes
downgrade results to incomplete, and native identity is not proof of
the QA world, admin rights, input delivery or log source attribution.

New Node regression fixtures were **AUTHORED_NOT_EXECUTED**. No
source tests, CI, native build, game process or Windows QA were run.
No new engine, daemon, computer-use input, world mutation, provider,
source mirror or installer change was introduced.

**Single next proof gate (requires separate Windows authorization):**
Run the original Skills Probe command in one isolated Education QA world
via a validated Player/admin input channel and confirm exact capture
world/profile identity. Only then evaluate command dispatch enablement.


## Stable Gameplay Probe capture checkpoint — 2 November 2026

**State: SOURCE_ONLY / NOT_EXECUTED.** The first source-only
`prepare_probe_qa` revision compared timestamps in the capture,
which could confuse cached, rotated or replayed logs with a new run.
The existing stable `MC0x.log` reader now returns an internal
file-identity/byte-offset/anchor checksum checkpoint at preparation,
then parses Probe events only from complete bytes appended after that
checkpoint. Identity change, truncation, rewritten capture tail,
lost 256-KiB window, repeated/interleaved markers, game-tick
regression, excessive markers and explicit logger throttling fail closed. Official ZIP Debug
Console examples have timestamp-free [probe:probe] lines; these
are now accepted without fabricating timestamps. Older map/source
evidence readers keep their bounded projections. No new listener,
file writer, background collector, command sender, second controller,
CI, game/world/process execution, credential access or native input
was introduced. Source regression fixtures were **authored but not run**.

This strengthens LOG provenance only: a file name and stable bytes
still do not attest that the game, world, player or AI input came from
the chosen profile. The separate runtime acceptance gate remains
one authorized Skills Probe test in an isolated Minecraft Education
Windows world with exact target/process attribution.

## Simple Gameplay Probe AI QA handoff — 2 November 2026

**State: SOURCE_ONLY / NOT_EXECUTED.** The existing local MCP has two
read-only AI QA actions: `prepare_probe_qa` and `read_probe_qa_result`.
The first needs one configured original pinned Skills Probe ZIP and
existing per-slot capture directory; it derives the auto probe ID set
from approved source and returns the official command and volatile
session ID. The second correlates only fresh RUN_START/RESULT/RUN_DONE
markers via the same stable MC0x.log reader. It will not treat old
probe:report STORED output, missing results, INFO/LOOK outcomes, or a
native input ACK as completed gameplay acceptance.

The supplied ZIP source currently declares 22 literal auto probes;
the historical AGENT-TEST-RUN.md says 12. The whole approved ZIP SHA-256
and runner blob digest gate the catalog. No new in-game scorer, second
MCP/game controller, daemon, installer, world writes, CI, source
upload, command dispatch, Content Log discovery or AI input was added.
World/profile/player attribution and Minecraft Education 26.33
compatibility remain NOT_VERIFIED. Authored source fixtures were
NOT_EXECUTED.

**Next (separate runtime approval):** In an isolated Education world,
verify the original Skills Probe command from an authorized Player,
capture fresh run markers and confirm exact world/profile identity.
Only then can a separately validated Computer Use command adapter
execute the prepared action; the native multi-AI input feature remains OFF.

## Minecraft Education team source parity + command help — 2 November 2026

**Operator-approved modular team ZIP intake (SOURCE_ONLY /
NOT_EXECUTED):** Added `team-source-baseline.json` with hash/version
pins from the two operator-provided archives (team v0.35.0, installed
Skills Probe modules), `team-update-zip.mjs` as a bounded ZIP32
read-only source inspector, and `team-update.mjs` to compare runtime
module fingerprints, upstream schema constants, Skills Probe
`modules-lock.json` / `skills-lock.json` and candidate updates.
One explicit local command `npm run team:update:inspect` writes
a metadata-only, content-addressed review report under gitignored
`workspace/diagnostics/team-updates/`. New modules/command/schema
changes are REVIEW_REQUIRED or BLOCKED until separately approved;
the staged candidate pin is explicitly NOT_APPROVED and cannot
be loaded as the current approved baseline. The tool does not
sync/execute team modules, write Minecraft data, connect Codex,
call a game controller or change the live MCP wire parser.
Synthetic regression fixtures are AUTHORED_NOT_EXECUTED.
The only next evidence gate for real module updates is the
developer's separately approved team `skills:check` /
`modules:check` and Minecraft Education Windows validation.
Documentation: `docs/system/education-team-source-updates.md`.



**Simplified in-game help v2.1.0 (SOURCE_ONLY / NOT_EXECUTED):**
The user requested a single memorable help command, not
`/scriptevent lazytester:help commands`. Minecraft already reserves
`/help` for built-in help. We therefore replaced the optional
Script API help listener with a **data-only** behavior pack containing
`functions/help.mcfunction` (one-line wrapper) and the only canonical
list `functions/lazytester/help.mcfunction`. Operators type
`/function help` (fallback `/function lazytester/help` if a map has
a conflicting help function). The list uses `tellraw @s`, displays
the exact original team commands but executes none, and never
broadcasts or calls AI. The pack retains its UUID but changes to
version 2.1.0, removes its script module and
`@minecraft/server` dependency, and updates the offline ZIP builder
to the exact three-file pack layout. Its structure follows the
Skills Probe ZIP's existing `functions/probe/command_sources.mcfunction`
convention. Authored Node fixtures were NOT_EXECUTED. No Minecraft
Education 26.33 Windows import, real function execution, or pack
build was performed. No CI, original world/account changes,
team analytics fork or new MCP feature was introduced.


**State: SOURCE_ONLY; Education Windows runtime BLOCKED / NOT_EXECUTED.**

Operator corrected the earlier approach: do NOT build a second
in-game analytics or diagnostics engine, and do NOT replace team
command names with ambiguous short aliases. The canonical sources are:
(1) operator-uploaded `minecraft-bedrock-agent-skills-main` modules,
and (2) its `skills-probe-main` test project. Of 40 matching
module-relative files in the two uploaded archives, 39 are byte-identical
and one differs (`modules/README.md`). Full archives are not identical.

The Lazy Tester Universal Diagnostics v1.x implementation and
`[LT_EDU_DIAG]` parsing have been RETIRED. Their old source remains
in Git history; old downloadable .mcpack files must be disabled
manually in test-copy worlds. Source-only standalone probe pack
stagers were also removed; use the team's existing Skills Probe
for in-game runtime verification rather than making a second probe.

Only the existing local evidence store + read-only MCP adapter
remain as consumers of the team's unmodified `[ANALYTICS]` and
`[GAME_RESULTS]` streams. The original 24 KiB team wire budget,
`level_score` / `gameplay_result` / `export_summary` distinction,
and privacy-limited Codex responses are preserved. No data is
automatically exported, no native controller changed.

The **single optional help-only addon** v2.1.0 retains the former
Lazy Tester pack UUID for one upgrade path and contains NO analytics,
scoring, telemetry, Script API, timers or AI calls. Its normal command:

    /function help

The help alias delegates to `/function lazytester/help` (a fallback
for map name conflicts) without duplicating command-list content.
An earlier v2.0.0 script-event help addon was retired as of this
revision. The help list clearly separates team exports from
Skills Probe-only admin commands. Canonical team module updates use
`bun run skills:check` / `bun run skills:sync` through the TEAM
repo's versioned lockfile mechanism, not copies in Lazy Tester.

Docs: `docs/system/education-team-telemetry-compat.md` and
`docs/system/map-diagnostics.md`. Source tests are
AUTHORED_NOT_EXECUTED, no package build, Minecraft install, Windows
world/account access, CI or debugger socket was exercised.

**One next action after separate runtime authorization:** on a
disposable Education Windows world, verify the team modules/Skills
Probe capture and the help-only addon command, then establish
source/profile/world identity before live local ingestion. Do not
promote any observed marker to verified gameplay PASS without evidence.

## Decision — 1 November 2026

**State: SOURCE-ONLY NATIVE BUILD/ABI PREPARATION; EXPERIMENTAL DLL TRUST BLOCKED / NOT_EXECUTED.**

Lazy Tester has one native controller for five Minecraft Education Windows
profiles. The separate optional Computer Use candidate includes source for
4/5-client AI control, but remains **compile-OFF, default-OFF and absent from
the normal product installer**. Latest native source work now includes compile-time assertions for all 15 field offsets of the Rust/C++ 96-byte input ABI v4. The earlier functional input change was `22057376b8e60760abaeab968cb69f6d3a9bef45` (Raw Input malformed-argument isolation).
Re-pin the latest `Local` HEAD before any future test; older commits are not
acceptance for the current revision.

**Local test eligibility is not local test success.** No current-exact-revision
Node/Rust/MSVC build, actual Windows game input, release of held keys, visual
capture or multi-AI gameplay acceptance has been executed or verified. This
request authorizes source cleanup, not background tests, game hooks, packages
or use of local account/world data.

## One source graph

```text
External Codex/Claude model (vision/reasoning)
  -> Development/tooling/agent/mcp.mjs (local stdio)
  -> same-image agent_cli.rs
  -> original authenticated native_clients_session.rs
  -> independent_bootstrap/session_controller.rs (one game owner)
  -> independent per-client 96-byte input mappings (opt-in only)
  -> bootstrap-compat C++ Win32 polling/Raw Input candidate (opt-in only)
  -> Minecraft Education's actual input consumer (NOT_VERIFIED)
```

Read-only visual observation is optional via Cua Driver or an unbuilt WGC
worker; frames, IPC acknowledgements and native state writes are not
Minecraft movement or game-level key-up proof.

| Area | What exists | Evidence ceiling |
| --- | --- | --- |
| Five native profiles | Existing manager, prior operator pilot | Not current AI acceptance |
| Hybrid MC01-human + MC02–MC05 AI; five-AI mode | MCP planning and opt-in source | REMOTE_GITHUB |
| Per-client writer lease, transaction epoch, neutral TTL | Native feature-gated source and authored fixtures | NOT_EXECUTED |
| Keyboard, relative mouse, attack/use hooks | Test-only Win32 API candidates | Consumer and gameplay UNKNOWN |
| Per-window capture | Optional Cua/WGC source adapters | Windows capture NOT_VERIFIED |
| Four/five concurrent AI gameplay | Source schedules, not observed control | NOT_EXECUTED |
| Production AI input package/release | Explicitly excluded | BLOCKED |

## Native ABI source gate — 1 November 2026

The original Rust game-input writer and optional C++ reader now each declare
the 15 exact field offsets (0–88 bytes), alongside size/alignment and atomic
width assertions for their 96-byte ABI v4 mapping. A future compile with the
experimental input feature must fail if any local field layout drifts, even
when the aggregate type remains 96 bytes. This is **compile-time source
hardening, not an executed build**, binary identity check or verified
Minecraft input consumer. No source/toolchain/runtime/package permission was
changed.

## Experimental AI DLL trust gate — 1 November 2026

A concrete native integration blocker was identified: previously the Rust
experimental input feature still selected the ordinary reviewed production
`bootstrap-compat.dll`, which intentionally excludes optional virtual-input
hooks. Thus native input-state ACKs could be emitted for an original game whose
DLL has no game-side AI consumer. A rebuilt experimental C++ DLL would also
fail the old production DLL hash.

The existing `BootstrapLibrary::reviewed()` now selects a distinct
`REVIEWED_EDUCATION_AI_BOOTSTRAP` compile-time digest when the Rust input
feature is enabled, for the **entire** retained native controller. The
experimental digest is deliberately `None` until separately reviewed:
the feature-enabled manager refuses its unapproved DLL rather than silently
falling back to the normal one. Normal builds keep the exact production hash
and error semantics unchanged. Human MC01 and AI MC02–MC05 must use one
source-matched experimental DLL pair, with no AI input section for MC01.
No new DLL loader, path override, provider, installer enablement or game hook
was introduced. A negative test was authored but NOT_EXECUTED.

This is a *source-side refusal*, not build success or a working AI input
mechanism. The approved experimental C++ DLL hash and binary identity do
not exist yet; game-level input consumption remains UNKNOWN.

## Experimental release build separation — source-only

The opt-in native feature cannot be shipped with the standard Lazy Tester
release pipeline. `build.rs` now refuses a Rust release-profile build when
either `mc02-native-input` or its `multi-client-native-input` alias is
selected. Developer-profile source compilation is not implicitly authorized
and the experimental DLL review pin remains `None`. The existing repository
source verifier and bounded negative fixtures now assert this release hold,
feature alias, CMake default-OFF switches and independent DLL trust boundary.
These fixtures are AUTHORED_NOT_EXECUTED. No installer, feature activation,
profile/world data or runtime process was modified.

## Current proof boundary and safety

The single procedure for the next stage is
[Computer Use Windows local test](../docs/validation/ai-computer-use-localtest.md).
Once separately authorized, `DEV.cmd inspect --json`, `catalog`, and
`plan` inspect a clean pinned checkout. `DEV.cmd preflight` may install
locked dependencies and verifies **standard** source; it does not enable
experimental native input. The C++/Rust experiment must be compiled and
reviewed together at the same source revision before any Minecraft test.
A normal `DEV.cmd dev` or installer is not an AI input candidate.

No global `SendInput`, forced desktop focus, undocumented alternative
provider, CI activation, unapproved hooks, credential automation, data
migration, force-close, or source-to-production promotion. Preserve
persistent Minecraft accounts/worlds, original process identity and
normal game exit/recovery.

## Separate native AI input next gate (still blocked)

**Only after separate operator approval on a compatible Windows x64 host:**
compile the exact-`Local`-SHA C++ input-enabled DLL and Rust input-feature
candidate (without launching Minecraft), verify binary identity and ABI,
and review the resulting C++ hash for a future explicit experimental
digest pin. Stop at any compile/identity mismatch. Game-level MC01-human /
MC02-AI acceptance needs a separate later approval.

**Proof recorded now:** REMOTE_GITHUB source/tree/doc and consumer inspection
only; executable test suites, Windows/MSVC build, installer and Minecraft
remain NOT_EXECUTED.

All previous detailed development notes were preserved in
[the historical handoff](history/ai-input-source-handoff-through-2026-11-01.md);
the former large MCP implementation README is also preserved in
[historical MCP notes](history/mcp-agent-implementation-notes-through-2026-11-01.md).
These records have no active authorization or alternative `Next`.
