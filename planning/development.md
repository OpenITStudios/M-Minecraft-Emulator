# Current Development Handoff — Lazy Tester

**Updated:** 6 November 2026. **Branch:** `Local` only; `main` is
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

## Cross-chat continuation — one canonical handoff

- **Repository/branch:** `OpenITStudios/M-Minecraft-Emulator` /
  `Local` only. Pin **actual current** HEAD, not a remembered SHA.
- **Current track:** Codex Windows Computer Use, strict zero-disturbance
  background; first read-only semantic UIA via existing Cua Driver,
  then action admission and independently verified background delivery.
  Keep Minecraft native game input separate and **NOT_VERIFIED**.
- **Last source stages:** P58-P64 Cua UIA read-only / upstream Invoke
  blocked; P65 synthetic counter; P66-P68 native 4/5-client
  attempted-slot, neutral-failure and MCP recovery integrity.
  P69 gates the native reader on its owned OGLES root HWND. Subsequent
  corrections latch physical-input isolation on first coherent active
  publication, and now reject the **entire cursor epoch** after observed
  deadline/focus/HWND failure. Merely incrementing a packet revision
  within the rejected epoch cannot replay stale press counters or
  accumulated look. The existing Rust writer owns the ReleaseAll/expiry
  epoch boundary; no ABI, controller or default product build changed.
  C++ assertions AUTHORED_NOT_EXECUTED; no runtime execution.
- **Evidence:** official and open-source macOS/Windows sources in
  [Windows parity research](../docs/system/windows-background-computer-use.md);
  upstream Cua Driver source pinned there. No REA binary session, no
  Codex/Minecraft/Windows runtime proof.
- **Continue:** read this file's **Single next step** and the named
  source owners before changing code. Git commits preserve precise
  history; never create another active roadmap/agent/controller.
- **Permission boundary:** GitHub source-only; no localtest, build,
  CI, installation, launch, game action, personal data or `main` write.

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
| Raw Input epoch fence | Reject whole cumulative press/look epoch after active source deadline/focus/HWND refusal; same-epoch heartbeats cannot replay rejected input. New Rust-owned epoch after neutral/expiry admits fresh input | C++ constexpr negatives AUTHORED_NOT_EXECUTED; compile-OFF |
| Input isolation fail-closed | The existing C++ consumer now latches physical-input isolation after observing a coherent first active packet, even when its focus/HWND/TTL admission refuses it; neutral-only recovery does not latch | Compile-time negative assertions AUTHORED_NOT_EXECUTED; feature still compile-OFF |
| P69 follow-up | Game-local C++ read path revokes an observed invalid active revision until a fresh controller write; rechecks human focus after HWND verification; zero-word key-up/mouse-up path unchanged | C++ static regression assertions AUTHORED_NOT_EXECUTED; compile-OFF |
| P69 | C++ virtual key/cursor packet admission now verifies current exact owned, visible OGLES root HWND at read-time; rejects replacement/hidden roots without input fallback | Compile-time negative assertions AUTHORED_NOT_EXECUTED; experiment remains OFF |
| P68 | Reviewed P67 actor -> CLI -> MCP failure path; added 4/5-AI regression for allowlisted AI_RELEASE_UNCONFIRMED, independent neutrals, quarantine, no replay and explicit recovery | Authored Node fixture NOT_EXECUTED; existing production MCP path unchanged |
| P67 | Controller partial-group failure no longer hides failed neutral writes; every attempted slot still gets one best-effort release, and CLI preserves exact AI_RELEASE_UNCONFIRMED status | Rust negative fixtures authored, NOT_EXECUTED; compile-OFF |
| P66 | Native multi-client batch records possibly touched slots only after final per-group deadline gate, just before IPC; retains uncertain per-group cleanup | Rust negative fixture authored, NOT_EXECUTED; experiment remains compile-OFF |
| P65 | Added opt-in UIA counter mode to existing synthetic Windows fixture, exposing an independent read-only 0->1 state oracle without action dispatch | AUTHORED_NOT_EXECUTED; no package/controller/runtime change |
| P64 | Upstream exact-source admission review: no no-fallback Invoke-only tool, native birth/session dispatch fence or Minecraft postcondition | BLOCKED_NO_SUPPORTED_UPSTREAM_ROUTE; no files outside docs/handoff |
| P63 | Exact-source upstream Rust contract and negative fixture matrix; refuse malformed assessment intent before native/UIA reads | Source-authored fixtures NOT_EXECUTED; no input/backend enabled |
| P62 | Selected minimal upstream Cua registry/snapshot extension boundary; existing verify_state is conditional readback only; no safe provider exists yet | REMOTE_GITHUB design/source review; no code/tests executed |
| P61 | Audit pinned/newer Cua paths; explicitly refuse even advertised Invoke candidates without no-fallback transport and app-owned readback | Authored negative fixtures, NOT_EXECUTED |
| P60 | Check Cua snapshot/token/index at read; refuse unbound candidates; expose blocked non-Invoke-only delivery | Authored negative fixtures, NOT_EXECUTED |
| P59 | Fix valid Cua UIA element index 0; add read-only strict-background Invoke assessment using fresh native-fenced UIA, without input/fallback | Authored negative fixtures, NOT_EXECUTED |
| P58 | MCP semantic UIA read via existing configured Cua Driver; 64-node tree-only result fenced by native pre/post identity | Authored fixtures, NOT_EXECUTED |
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

## Efficient implementation sequence — current native-input track

This is a **dependency order, not authorization for runtime work or a
second active backlog**. Develop only one source-proven issue at a time:

1. **Completed P66–P69: native input integrity** — source corrections
   for attempted-slot/release safety, exact MCP error/quarantine and
   per-poll live root HWND checks; no new transport.
2. **Source hardening on demand** — audit the *original* 4/5-client
   controller, CLI scheduler and per-client consumer for real remaining
   identity, epoch, deadline, cross-slot or key-up defects. Change only
   the first wrong source owner and its direct fixtures. Do not add an
   extra controller, dispatch abstraction or speculative stage count.
3. **Verification preparation** — reuse the existing source-only Node,
   Rust, C++ and manual synthetic UIA counter fixtures. Compare the
   exact planned source SHA and negative cases before any separately
   authorized execution; only actual runs count as test results.
4. **Independent runtime acceptance (separate permission)** — after
   explicit operator approval, validate original Windows input safety,
   neutral/readback and human foreground, then Minecraft Education
   actual motion and 4/5-client concurrency. ABI/IPC ACKs alone are not
   game proof. The unrelated Cua strict Invoke-only dependency stays
   `BLOCKED_NO_SUPPORTED_UPSTREAM_ROUTE` until upstream capability
   and verified focus isolation exist.

**Efficiency rule:** no extra framework, duplicate handoff, automatic
CI, periodic speculative audit, feature activation or rewrite. If
no source-verifiable correctness defect remains, stop source changes
and preserve the actual execution gate as NOT_EXECUTED.

## Single next step

**Source-only audit of buffered Raw Input on the ORIGINAL 4/5-client
input consumer after the epoch fence.** Pin fresh `Local` HEAD; trace
one-record `GetRawInputBuffer` reads, key-up/mouse-up pending from a
previously delivered DOWN, and one camera-only packet after an
invalidated epoch. Preserve zero-word release for the same original
OGLES game thread; no physical-queue fallback, stale press/delta or
cross-client reattachment. Fix only an actual source defect in the
existing C++/Rust owner and add bounded authored negative fixtures.
Do not create a second writer, driver, daemon or speculative
milestone ledger. Cua strict Invoke remains `BACKGROUND_UNAVAILABLE`.
Node/Rust/C++ execution, builds, CI, packaging, installed Windows,
Minecraft and five-client acceptance remain NOT_EXECUTED/NOT_VERIFIED.
