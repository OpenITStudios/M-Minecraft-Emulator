# Current Development Handoff — Lazy Tester

**Updated:** 5 November 2026. **Branch:** `Local` only; `main` is
release-reserved and GitHub Actions CI remains disabled. This file owns
the ONE active development handoff, proof ceiling, and next action.
Always pin the actual `Local` HEAD. Historical detail through P46 is
[archived without deletion](history/source-handoff-through-2026-11-05.md)
at `d1fbd9e12e48b92ada5f5ef13b6b7d3c10c604a6`, with older material in
[2 November history](history/source-handoff-through-2026-11-02.md).
Archived "Next" statements are not current authorization.

## Primary product goal — operator reset 5 November 2026

**Build Codex Computer Use for Windows with a strict background/no-user-
interruption contract, modeled after the Mac experience.** Codex should
inspect and control specifically authorized windows without stealing the
operator's mouse, keyboard, focus or z-order. Minecraft Education five-client
management remains a key target, but its native game-input backend is
experimental and still not accepted. Cua Driver's existing Windows
background UI Automation is the first integration candidate; screenshots
are an on-demand fallback, **not** the foundation of gameplay input.
Read the [evidence-based parity design](../docs/system/windows-background-computer-use.md)
before continuing. REA was not executed; public macOS/Windows source
and independent RE reports were reviewed. All runtime behavior remains
NOT_VERIFIED.

## Product and owners

- **Product:** Lazy Tester, one Windows desktop manager for up to five
  independent licensed Minecraft Education profiles (MC01–MC05).
  Minecraft owns game content, gameplay, accounts and networking.
- **Only game/process authority:** original Rust session controller and
  authenticated same-image CLI/IPC. Persistent private profiles/worlds in
  ignored `Data/` and installed `App/` must never be modified by source
  cleanup. Window/DWM presentation is not a second lifecycle owner.
- **Optional AI bridge:** one local stdio `Development/tooling/agent/mcp.mjs`
  delegates to the original controller. Experimental per-client AI input
  remains **compile-OFF**, explicit opt-in, absent from normal releases,
  and without verified Minecraft input consumption. Human MC01 stays
  protected in hybrid mode; native writer leases are exact
  PID/birth/session/slot/epoch and require neutral ACK to release.
- **Source-only QA:** original upstream team modules exclusively own in-game
  analytics/results; original Skills Probe owns foundation probes.
  Lazy Tester only reads bounded evidence/approved ZIPs. It does not
  dispatch game commands or certify map/world gameplay.
- **HUD/coordinates:** native Minecraft coordinates are the live owner.
  The additive, world-scoped P22 v0.3 JSON UI prototype is **uninstalled**
  and unverified in Education 26.33. Desktop Saved Locations stores only
  manually pasted XYZ. Automatic F3/OCR, live world Save/Load, and sealed
  Testing CAS transfer remain blocked.
- **Developer harness:** one existing `DEV.cmd`/developer controller.
  Harness² Propose–Probe–Compose, DeepSeek scoped roles and REA are
  **developer-side reference patterns**, not installed agent runtimes or
  replacement Minecraft controllers.

## Recent source outcomes

| Revision | Source-only change | State |
| --- | --- | --- |
| P57 | Stopped WGC slot retains status counters only; releases source-owned child/decoder/PNG references without affecting next worker | Authored late-callback/restart fixture, NOT_EXECUTED |
| P56 | CUA screenshot reader bounds the exact bytes read and refuses changed/incomplete files after the initial stat | Authored short-read/negative fixtures, NOT_EXECUTED |
| P55 | WGC provider reuses existing base64 when a new sequence contains identical PNG bytes and dimensions; retains fresh sequence/time | Authored fixture, NOT_EXECUTED |
| P54 | WGC decoder refuses stale/future frame timestamps from header before allocating PNG; retains post-assembly expiry check | Authored negative fixtures, NOT_EXECUTED |
| P53 | MCP README compacted; all technical stage notes preserved in historical source snapshot | Read-only doc consolidation, NOT_EXECUTED |
| P52 | MCP image response reuses observer's 4 MiB base64 boundary rather than stale 8 MiB allowance | Authored boundary fixture, NOT_EXECUTED |
| P51 | WGC native encoder reuses one PNG output allocation per worker and writes its 28-byte header on the stack | Authored Windows-only Rust fixture, NOT_EXECUTED |
| P50 | QA-only Skills Probe reads skip unnecessary full-tail UTF-8 conversion; newline determined directly from existing bytes | Authored regression, NOT_EXECUTED |
| P49 | Existing MCP evidence reader scans backward until 20 valid team records, without parsing discarded log history | Authored regression, NOT_EXECUTED |
| P48 | WGC producer FPS follows validated observer interval (1–5 FPS, with 2× headroom), no needless fixed-rate capture | Authored regression, NOT_EXECUTED |
| P47 | WGC assembles PNG fragments with bounded linear copies, and encodes each requested frame once | Authored regression, NOT_EXECUTED |
| P46 | WGC packet decoder reuses `inspectPng` SHA-256 instead of hashing the same PNG twice | Authored regression, NOT_EXECUTED |
| P45 | CUA and WGC use one 3 MiB lossless PNG limit, matching observer 4 MiB base64 bound | Authored regression, NOT_EXECUTED; CUA >3 MiB now refused |
| P44 | Existing developer review flags critical Rust/C++ shared-memory ABI v4 and MCP callers (96 bytes, 15 offsets, 8 rising-edge counters) | Read-only review request, NOT_EXECUTED |
| P29–P43 | Native input lease/neutral reconciliation, epoch/IPC uncertainty, Raw Input expiry/key-up/partial reads and counter overflow guards | Source only; Windows input and gameplay NOT_VERIFIED |
| P22–P28 | World-scoped HUD prototype and bounded developer source-review routing | Source only; HUD merge NOT_VERIFIED |

Details, earlier phases, experiments and negative fixtures:
[historical P1–P46 handoff](history/source-handoff-through-2026-11-05.md) and
[complete archived MCP source reference](history/mcp-agent-source-reference-through-2026-11-05.md).
Canonical live source owners:
[implementation map](../docs/system/implementation-map.md) ·
[architecture](../docs/system/architecture.md) ·
[MCP](../Development/tooling/agent/README.md) ·
[development discipline](../docs/system/development-discipline.md).

## Current source workflow (P1/P2/P3)

`probe_qa` remains the preferred one MCP QA tool. It pins approved Skills
Probe source, a stable append-only capture cursor, and optional native
PID/HWND/birth evidence; only a separately authorized Player/admin may
actually run the original command. No command is sent by Lazy Tester.

The optional map contract chooses `full`, `smoke`, or `affected`.
Without an independently verified source diff, `affected` falls back to
`full`. `mapPlan` remains **NOT_TESTED**, while `qaReport` preserves the
original Probe-reported outcome separately from map results; acceptance
stays **NOT_ACCEPTED**. Cached unchanged ZIP/contract and bounded appended
reads avoid repeated full scans. A changed/unstable source fails closed.
There is no second testing engine, daemon, or implicit execution.

## Efficiency decisions

- **E01/P45 COMPLETE (source):** preserve PNG losslessness; refuse oversized
  frames before base64, without a second compressor or recursive resize.
- **P46 COMPLETE (source):** remove duplicate per-frame WGC SHA-256 work.
- **E02 REVIEWED — NO CODE CHANGE:** critical `crossModuleReviews` selectors
  are read-only and deliberately conservative across original MCP/Rust/UI/
  C++ owners. Narrowing them based only on filename guesses risks silent
  compatibility omissions. No second harness, coverage scorer, runtime
  observer or dependency graph is justified.
- **E03 COMPLETE:** compact handoff here, all prior long-form decisions in
  historical archive. Do not recreate a second active status ledger.
- **E04 COMPLETE (source):** the existing efficient-local-testing runbook
  now defines an exact-SHA, no-new-collector comparison for PNG payloads,
  frame freshness, native IPC, CPU/RAM/GPU/FPS and source iteration. The
  measurements remain UNMEASURED/NOT_EXECUTED until separately authorized.

## Proof ceiling and unresolved gates

**REMOTE_GITHUB verified:** repository layout, current source data paths,
default-off experimental controls, tool contracts, authored fixtures and
explicit static invariants. P45/P46 improvements are *source-level work
eliminations*, **not measured CPU, memory, FPS, network or token reductions**.

**NOT_EXECUTED on current source:** Node/TypeScript/Rust/MSVC tests and builds,
Windows native processes/installer, CUA/WGC capture, actual Minecraft
Education 26.33 HUD merge, game sign-in, Player/admin Probe execution,
input capture/key-up, native multiplayer and five-client acceptance.
A previous pilot at a different SHA is not acceptance of this revision.

Unresolved actual-world questions are explicitly BLOCKED pending separate
operator authorization. Preserve private data and packages; do not launch
Minecraft, run local source tests, enable GitHub Actions, install experimental
DLLs, touch credentials/worlds, dispatch game commands, or promote to `main`.
No source-only review may mark these runtime gates PASS.

## Single next step

**Source-only Codex Windows Computer Use parity, not another PNG or
runtime optimization.** Pin `Local` HEAD, compare the installed *source*
Cua Driver call/response contract and the existing
`Development/tooling/agent/visual-capture.mjs` + `mcp.mjs` owners.
Design the **smallest scoped semantic UIA read-only bridge** with exact
native PID/HWND/birth/session and strict `BACKGROUND_UNAVAILABLE` refusal.
Do not dispatch new input or add another controller until evidence
supports an independently reviewed action. REA requires an authorized
binary, unavailable in this GitHub-only session. No local tests, CI,
Windows/Codex/Minecraft execution, installer or binary acquisition.
