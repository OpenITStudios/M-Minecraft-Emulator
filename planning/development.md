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
  blocked; P65 synthetic counter; P66-P68 native multi-AI cohort
  cleanup/recovery; P69+ exact OGLES root, rejected-epoch, physical-input
  isolation, Raw Input owner-thread and keyboard final-read guards.
  Integrated A/B/C now verifies exact 4/5-AI pulse bytes AND source
  neutral ACK coverage. The original CLI explicitly reports each
  controller-neutralized slot; the canonical verified MCP sender may
  reuse it on a successful receipt, avoiding duplicate per-slot neutral
  CLI/IPC calls. Failure, cancellation, timeout, uncertain or injected
  results retain independent neutral/quarantine and explicit recovery.
  The original controller/C++ reader and v4 ABI remain unchanged;
  Node/Rust fixtures AUTHORED_NOT_EXECUTED.

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
| Integrated exact-content ACK | Node hashes exact ordered native pulse argv; Rust CLI echoes the same SHA-256; mismatch is uncertain and keeps existing 4/5-AI neutral/quarantine | Node/Rust negative fixtures AUTHORED_NOT_EXECUTED |
| Raw Input size-query admission | Size-only query now reports one RAWINPUT record only for a coherent same-word, even-revision, same original OGLES game-thread plan; stale/torn plans report zero without advancing any press/release cursor | C++ compile-time negative cases AUTHORED_NOT_EXECUTED; compile-OFF |
| Raw Input signed-look bound | Share overflow-safe bounded i64 delta with cursor polling before planning Raw Input; retains pending key-up/mouse-up ordering at extreme cumulative look totals | C++ constexpr boundary assertions AUTHORED_NOT_EXECUTED; feature compile-OFF |
| Virtual cursor recenter publication fence | SetCursorPos permits live recenter only against original active publication, or stable native zero-packet menu state; a concurrent ReleaseAll/replacement rolls back both virtual axes without touching physical cursor | C++ constexpr negative fixtures AUTHORED_NOT_EXECUTED; compile-OFF |
| Virtual cursor position interleaving | One short local mutex keeps initial physical sampling, recenter, virtual X/Y reads and camera delta commits coherent; lost ReleaseAll publication rolls back both temporary axes and leaves look backlog unconsumed | Source-only static contract inspection; tests NOT_EXECUTED |
| Virtual cursor scope correction | Close `observedKeyboard` before the virtual cursor implementation so optional Raw-buffer and default polling branches remain namespace-level functions | Both C++ preprocessor branches statically scope-balanced; MSVC tests/build NOT_EXECUTED |
| Virtual cursor bounded backlog | Per-poll cursor move limited to ±32767 without dropping remaining accumulated look; same native revision can deliver its unconsumed prefix after exact publication revalidation | C++ compile-time negative boundary assertions AUTHORED_NOT_EXECUTED; compile-OFF |
| Win32 last native-write fence | Additional lightweight v4 publication check after keyboard diagnostics; rollback tentative async press cursor on lost native publication, zero held KeyState and scrub KeyboardState | C++ constexpr negatives authored NOT_EXECUTED; compile-OFF |
| Win32 virtual polling final-read gate | Preserve publication revision with initial virtual input; recheck full target/focus/TTL before returning keyboard masks, committing async press counts, or consuming camera delta; scrub GetKeyboardState on late neutral | C++ negative constexpr fixtures AUTHORED_NOT_EXECUTED; compile-OFF |
| Raw Input pinned-thread revalidation | Same original OGLES HWND and game-thread owner are checked before and after provisional prefix copy; on observed replacement the copied records are scrubbed and read cursors stay unchanged; release-only still permits hidden original root | C++ constexpr negatives AUTHORED_NOT_EXECUTED; compile-OFF |
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

## Integrated A/B/C native input implementation

The operator selected ONE integrated full-stack source deliverable, not
additional per-function micro-milestones:

- **A — Orchestration and exact-content receipt (SOURCE_IMPLEMENTED):**
  MCP retains its deterministic four/five-AI compiler and the one
  original CLI timestamp-group scheduler. Node SHA-256 hashes the
  EXACT sorted pulse JSON bytes passed to the CLI argv. The same-image
  Rust CLI echoes SHA-256 of those exact bytes after complete batch
  validation. Node requires matching digest, slots, pulse/group counts
  and explicit NOT_GAMEPLAY transport status. Changed payload with
  identical slot/count metadata is not accepted as an ACK.
- **B — Original native consumer (COMPILE_OFF):** one Rust controller,
  independent per-process v4 96-byte mapping and existing C++ keyboard,
  cursor and buffered Raw Input readers. Previous source fixes remain:
  exact OGLES owner/thread, TTL/epoch, bounded look, zero-word key-up,
  mouse-up and partial-prefix consumption. No driver, second writer or
  default product enablement.
- **C — Neutral / recovery provenance (SOURCE_IMPLEMENTED):**
  The original native CLI returns `neutralSlots` and
  `neutralState:NATIVE_PER_SLOT_NEUTRAL_ACK` only after every attempted
  original channel has an acknowledged native neutral write. Node
  validates full exact slot coverage alongside the pulse digest. MCP
  reuses this confirmed neutral ONLY from its canonical verified sender
  and successful transport; it can then proceed to existing per-slot
  lease release without spawning 4/5 redundant neutral CLI calls.
  Any injected provider, missing/altered receipt, abort, timeout, or
  partially failed group still attempts the original independent
  neutral and quarantines uncertainty. Node/Rust negative fixtures
  cover hybrid 4-AI and full 5-AI, all AUTHORED_NOT_EXECUTED.

**Source flow:** agent intents -> deterministic pulse stream -> one
native CLI scheduler / grouped IPC -> original per-slot Rust state
and neutral -> exact-content transport ACK -> existing uncertain and
recovery path. SHA-256 is not a native signature, installed binary
attestation, game effect, simultaneous delivery or human-input proof.

**Efficiency/safety:** SHA-256 is computed once per bounded batch.
Successful, exact-content native ACKs can eliminate redundant 4/5
one-shot neutral CLI/IPC exchanges; error paths remain conservative.
This is a source-level reduction in calls, not a measured latency
claim. No new per-frame IPC/Win32 reads, driver, input provider,
additional state owner or default product activation. Existing
150-ms lateness, 350-ms TTL, human MC01 protection and compile-OFF
flags remain.

**Proof:** REMOTE_GITHUB source review; Node/Rust/C++ fixtures
NOT_EXECUTED, Windows/Minecraft performance and five-client input
NOT_VERIFIED, Cua strict Invoke-only `BACKGROUND_UNAVAILABLE`.
GitHub Actions CI remains disabled.

## Single next step

Conclude the SAME integrated A/B/C source package by reconciling the
end-to-end source acceptance matrix: exact 4/5-client pulse content,
canonical native neutral-slot proof, direct writer lease/ReleaseAll,
partial-group failure, cancel/timeout, explicit MCP recovery, and
C++ Raw Input zero-word pending UP. Compare fixture coverage at the
exact Local HEAD. Fix only an actual remaining cross-layer mismatch.
No new controller/driver/dispatch backend, no default activation, no
CI. Cua strict Invoke-only stays `BACKGROUND_UNAVAILABLE`;
Node/Rust/C++ tests, builds, Windows, Minecraft and five-client
gameplay remain NOT_EXECUTED/NOT_VERIFIED.
