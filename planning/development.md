# Current Development Handoff — Lazy Tester

## Minecraft Education local-first diagnostics — 2 November 2026

**Operator command UX — source-only:** v1.2.0 of the fixed-UUID
Education universal diagnostic behavior pack now recognizes
`/scriptevent lt:diag help|on|off|status|more` with the same internal
collector/state as the legacy `lazytester:diagnostics` script event.
`help` responds in chat when invoked, without activating telemetry or
sending analytics to MCP/Codex; `on` and `off` form the normal two-command
QA workflow. `more` temporarily activates both already-bounded focus
sources for up to 600 game ticks. When no player is identifiable from a
script event, explicit replies can be broadcast to the test world; no
periodic chat or player identities are used. Old command messages and
the team's [ANALYTICS]/[GAME_RESULTS] contracts are unchanged.
Manifest/modules and the offline builder are versioned 1.2.0; prior pack
versions must not run alongside the new version. Regression source
fixtures were AUTHORED_NOT_EXECUTED. Actual Education Windows 26.33
support for source, chat, commands, logs, and multiplayer remains
NOT_VERIFIED. No native controller/input/MCP mutation, AI automation,
GitHub Actions, test execution or actual game installation occurred.


**Team-skills contract alignment — source-only continuation:** The user-supplied
Bedrock Agent Skills and Skills Probe ZIPs were compared at source-file
level: 39/40 matching module paths have byte-identical contents.
Upstream [ANALYTICS] and [GAME_RESULTS] remain authoritative,
24 KiB per line and 8 paced analytics export lines/tick; final
game-results export_summary with failed_counts is recognized separately
from scored gameplay_result. Existing bounded MCP log reader is extended
to parse team records and tiny [LT_EDU_DIAG] records, with schema,
counter/size guardrails and raw record IDs withheld from AI timelines.
Universal pack v1.1.0 keeps its UUID and uses lifecycle-only normal
subscribers; noisy combat/entity sources require explicit, timed 600-tick
operator opt-in. Team source is **not** vendored/modified; compatibility
pins and mapping are in docs/system/education-team-telemetry-compat.md.
Source test fixtures were AUTHORED_NOT_EXECUTED. Education Windows runtime,
MCP live ingestion, actual FPS/CPU cost and token savings remain NOT_VERIFIED.
No CI, native controller change or profile/world access.


**Reusable universal .mcpack (source-only + separately prepared artifact):**
The operator supplied two team archives (`minecraft-bedrock-agent-skills-main`
and `skills-probe-main`), which include a documented protocol-7 direct
debugger capture on Minecraft Education 26.32/macOS, an Education BP
manifest with `@minecraft/server` 2.8.0 and bounded analytics exports.
We used these as **source/evidence references only**, not direct live
compatibility proof for Windows 26.33. Added fixed-identity
`education-universal-pack/` with one original opt-in generic diagnostics
module; it uses simple stable event subscriptions, 200-tick aggregated
counters and 600-tick heartbeat, privacy-safe `[LT_EDU_DIAG]` lines,
`/scriptevent lazytester:diagnostics start|status|flush|stop` and no
world mutation or per-map identity. The packager
`build-education-universal-pack.mjs` prepares one local `.mcpack` under
ignored workspace without overwriting or installing it. Its ZIP/manifest
fixtures are AUTHORED_NOT_EXECUTED. This pack is **not** a full
map-specific assertions adapter, and its emitted log marker is not yet
connected to the existing read-only MCP. Actual Education Windows
pack import, Script API dependency load, command reception, log capture,
resource overhead and world/profile attribution are NOT_VERIFIED.



**Disposable Education Script API pack preparation (source-only):** added
`prepare-education-probe.mjs`, an explicitly invoked offline generator for
an isolated behavior-pack folder. It reuses the single
`education-script-probe.js` source; each staging action generates a new
nonsecret 12-hex run ID, fresh manifest UUIDs, one explicit
`@minecraft/server` stable dependency selection and a local receipt with
hashes. Only ignored `workspace/diagnostics/education-probe/` is a
possible destination. This does **not** install/activate a pack, guess
Education's supported module version or read a Minecraft profile/world.
Authored pure unit fixtures are NOT_EXECUTED. Actual Education Script API
load/output, Content Log path/wrapper, host and profile identity remain
NOT_VERIFIED. The next gate remains a separately approved isolated Windows
Education test before live ingestion.


**State: SOURCE-ONLY BOUNDED IMPLEMENTATION AUTHORED, RUNTIME NOT_EXECUTED.**

**Storage hardening continuation (source-only):** new v2 run folders
separate evidence/events.jsonl, analysis/issues.json, and manifest.json.
The importer writes with exclusive-create semantics, flushes files,
publishes the manifest last, and records file-size/SHA-256 integrity.
Incomplete folders are excluded from overview run listings (and counted);
checksum mismatches block access. v1 files are still readable but
marked LEGACY_NOT_CHECKSUMMED. Evidence queries only materialize a requested
page of projected records in memory. Authored regressions for incomplete
imports, checksum corruption, v1 compatibility and gap pagination are
NOT_EXECUTED; no Windows, Minecraft Education or Codex runtime tested.

**Education offline capture-format hardening (source-only):** the existing
bounded local inspector now provides an explicit `envelope-survey` to count
marker-wrapper shapes without reading marker payloads, exposing private log
text or calling AI. Strict parsers distinguish unsupported wrappers from
absent markers, preventing false-negative Education transport conclusions.
Additional marker-wrapper/privacy regression fixtures are AUTHORED_NOT_EXECUTED.
No Minecraft Education capture, profile attribution, build or Node tests were
run; the blocker remains real Education output/source verification.

**Education Script API log proof preparation (source-only):** authored one
test-only `@minecraft/server` two-marker emitter and one local-only read-only
capture inspector. The offline inspector accepts only explicitly selected
absolute .log/.txt files and the candidate `marker-lines` or
`timestamped-print` envelopes; it emits bounded sanitized counters and
a source SHA-256, not raw data. No Minecraft log path is guessed, no
world/profile is visited and no MCP or AI call is made. Source fixtures
for marker ordering, malformed/escaped lines and data suppression are
AUTHORED_NOT_EXECUTED. Bedrock documentation is not Education runtime proof.
The next gate remains separately approved Education Windows capture with
correct world/pack/slot identity before any live ingestion.

Operator approved keeping all captured evidence **locally** and exposing
only compact issue candidates/on-demand evidence to Codex, using the
existing MCP. On the Local branch, the first slice adds:
- diagnostics-local.mjs: an explicit offline normalized JSONL importer,
  immutable runs under ignored Data/Diagnostics, finite source/index quotas,
  deduplicated candidates and read-only indexed/paginated evidence queries.
- The existing mcp.mjs exposes four bounded read-only diagnostics tools.
  No path may be supplied by an MCP caller; diagnostics directory is explicit
  and disabled when absent. Import is CLI-only, not AI driven.
- Authored Node regression fixtures and the contract in
  docs/system/map-diagnostics.md. No DB server, remote upload,
  automatic tailer, second game actor, profile/world changes, CI,
  Windows build or Minecraft execution.

This first slice does **not** implement live Education Script API collection.
The JSONL format is a proposed normalized adapter contract, not a verified
Minecraft content-log message. World/player-to-profile identity, stable
Education transport, timing, telemetry performance and debugger compatibility
are UNKNOWN. A local import/ACK never establishes gameplay PASS. All
new diagnostic tests are AUTHORED_NOT_EXECUTED.

**One next diagnostics action, subject to distinct execution approval:**
verify Education build-specific stable Script API output and the real
per-profile Content Log transport/source identity on an approved Windows
target; only then author a bounded live ingestion adapter. Keep the
experimental AI input feature blocked and default-OFF independently.


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
