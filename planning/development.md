# Current Development Handoff — Lazy Tester

## Minecraft Education team source parity + command help — 2 November 2026

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

The **single optional help-only addon** v2.0.0 shares the former
Lazy Tester pack UUID to avoid two active Lazy Tester packs, but
contains NO analytics, scoring, telemetry, timers, gameplay events
or AI calls. It handles only the full readable command:

    /scriptevent lazytester:help commands

In-game help lists the team commands and makes the Skills Probe-only
admin scope explicit. Source list must be updated when the team
releases a changed command contract. Canonical module updates use
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
