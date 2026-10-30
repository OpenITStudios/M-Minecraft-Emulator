# Current Development Handoff — Lazy Tester

## Current state — 30 October 2026

**State:** CANONICAL SOURCE ROUTING ALIGNED / NATIVE VALIDATION DEFERRED. The active product remains
Lazy Tester for five independent, individually licensed Minecraft Education
Windows profiles, using the original official game installation, reviewed
native compatibility payload, one retained session controller and private
profile storage. Only branch Local receives product development; CI is off.

**Proof ceiling:** Current source and GitHub revisions can be inspected
remotely. The pinned Windows Node 24 / PowerShell / Cargo / Tauri suites,
installed NSIS candidate, account sign-in, live DWM previews and five-client
Education gameplay have **NOT_EXECUTED** acceptance at the exact new revision.
A successful static source review is not a Windows or Minecraft PASS.

## Source-only authority consolidation (30 October 2026)

The canonical documentation router, product and validation indexes,
architecture, implementation ownership map and naming standard now select
five-profile native Lazy Tester as the single active source of truth.
Superseded four-profile Sandboxie/VMware experiments and Compact Manager
designs are historical references only; their previous contents remain
recoverable from immutable Git revisions. This eliminates approximately
100 KB of stale current-route prose without touching executable code,
installer identity or profile/world data.

**Evidence:** exact GitHub source/blob/tree inspection and relative-link/
source-path review only. No Windows runtime or Minecraft tests were run.

## Consolidated source outcome

- DEV.cmd is the only developer entrypoint. Its existing Node controller
  routes inspected changes to one allowlisted check; task execution requires
  exact Local HEAD, and high-impact operations retain separate approval.
- The default automatic check uses working-tree changes or the latest
  committed delta. Explicit --base-sha selects the changed-path union
  from a verified ancestor through HEAD plus local edits, so final
  multi-commit development is not misrepresented as one-commit coverage.
  It introduces no new runner, CI, state file, process, or build engine.
- Desktop npm dependencies are reused only after pinned manifest/lock,
  installed dependency-graph and nonlinked-root checks. Missing or invalid
  ordinary dependencies fall back to frozen npm ci; uncertain links refuse.
- Desktop and runtime-core Rust checks use the existing shared Windows-MSVC
  Cargo target. Active incremental cache stays reusable. Neither a new target
  tree nor unscoped cargo clean is introduced.
- Successful package export may retire at most eight older verified installer
  candidates, retaining the newest two. Historic compiler archive retirement
  remains an **independently approved, fail-closed** action, never an automatic
  consequence of run auto, preflight, or packaging.

These are source implementation decisions, not measured timing/disk savings.
No Minecraft lifecycle, Microsoft account/world data, runtime DLL, installed
identity, game package or native preview behavior is changed in this stage.

## End-to-end source hygiene (30 October 2026)

- Align `AGENTS.md` runtime-acceptance navigation, current developer discipline
  and Windows-native toolchain metadata with the single five-profile product.
- Direct `DIAGNOSE` at the existing native session/identity/workspace owners,
  and define `TARGET_WINDOWS` as observed native host/game evidence; historical
  VMware/Guest Agent artifacts remain protected but inactive.
- Reclassify explicitly retired VMware/Sandboxie/earlier two-client documents
  as historical; keep their original evidence recoverable without treating
  their obsolete instructions as canonical product procedures.
- Retire the unmounted old workspace UI harness and route browser-only checks
  to the existing Lazy Tester fixture. No new runtime, test runner, installer,
  workflow, version or mutable profile owner is introduced.
- The existing validation index now defines one bounded real Windows/product
  smoke sequence. No step is recorded as executed merely by documenting it.

**Proof:** REMOTE_GITHUB exact-blob/tree inspection and scoped source/consumer
checks only. Current Windows toolchain, source suites, installer and five-client
Minecraft runtime acceptance remain NOT_EXECUTED.

## Documentation-only DEV.cmd selection (30 October 2026)

- Correct the existing source-path classifier: Markdown skill instructions and
  README/AGENTS documents inside otherwise native folders should select the
  existing repository-structure check, not unnecessary Rust/native compilation.
- Unknown executable extensions under `docs/`, `planning/`, `Archive/` and
  `.agents/` now conservatively require full source verification, rather than
  being misidentified as documentation. Protected `App/`, `Data/` and
  `workspace/` paths remain blocked.
- Extend the existing controller regression suite only; no new runner, task,
  database, automation or version introduced. Full source/package acceptance
  is unchanged, and Windows Node 24 / Cargo / Minecraft acceptance remains
  `NOT_EXECUTED` at this revision.

## Fresh-install App rollback repair — source-only

- `stageLocalApp()` now tracks whether its staged `App/` was actually
  promoted. If root `Lazy Tester.exe` promotion fails on a fresh install,
  it moves only that newly promoted App to the existing private attempt's
  `failed-App/`, leaving `App/` and root launcher absent for a clean retry.
- The prior-App and launcher recovery paths remain unchanged in intent; the
  old-launcher move is now inside the same guarded rename sequence.
- The existing `package-candidate.test.ts` gains one bounded regression
  scenario covering injected launcher failure, preserved `Data/`, private
  failed-payload retention and a successful explicit retry.
- This changes no Minecraft game/session logic, installed NSIS contract,
  profile content, runtime DLL, version, branch policy or CI setting.

**Proof ceiling:** GitHub exact-source review and isolated fault-injected source
execution where performed. Pinned Windows Node/Vitest, native NSIS package
and live five-profile Minecraft Education remain `NOT_EXECUTED`.

## AI computer-control UI accessibility — source-only (30 October 2026)

- Existing MC01–MC05 launcher buttons remain the only game-selection interface. Running profiles now expose an explicit **Select Minecraft** accessible action; stopped profiles retain **Launch Minecraft**. A named profile group and running-only selected state make the target distinguishable without exposing account credentials.
- Existing recovery selection says **Select game**, and the native manager's Close accessible name explicitly warns that all managed Minecraft sessions are affected. No send-command feature, keyboard injector, new agent API, IPC route, timer, controller, profile authority, or UI page is added.
- The retained browser-only UI regression fixture now asserts stable profile action names and selection state using accessibility roles. This is authored coverage, not an executed result.

**Proof ceiling:** REMOTE_GITHUB source/consumer inspection. Windows WebView accessibility exposure, native game keyboard focus after selection, real Minecraft input and five-profile AI computer-control usability remain **NOT_EXECUTED**.

## Windows peer-to-peer AI bridge — source-only (30 October 2026)

- Added optional local stdio MCP in `Development/tooling/agent/` with capability disclosure, five-profile inspection, explicit AI-slot Launch, and bounded read-only `[ANALYTICS]`/`[GAME_RESULTS]` evidence reading from the existing external debugger capture format.
- `--lazy-tester-agent-cli inspect|launch <slot>` is a narrowly restricted same-executable one-shot adapter to the **retained authenticated native session controller**. It is not a second game owner and never opens a Tauri UI or transmits keyboard/mouse input.
- Default `human-plus-four-ai` reserves MC01 from the MCP launch action; `five-ai` explicitly permits launch of all five. Neither mode makes foreground computer use safe for human-plus-AI parallel gameplay.
- The new Node tests were executed outside Windows with synthetic fixtures: MCP handshake, mode limits, serialized operations, error handling, and evidence parsing. The existing `test:localtest` includes this suite for future source checks.
- **NOT_IMPLEMENTED / NOT_VERIFIED:** independent background WASD/mouse, auto host/join, game command injection, five simultaneous debugger transports, per-game attribution by real player identity, actual Windows native bridge integration. These cannot be asserted from static source or synthetic tests.

**Evidence:** REMOTE_GITHUB source contracts + actual Node source fixture execution only. Windows native source compilation, packaged console mode, five-game gameplay and AI independent input remain `NOT_EXECUTED`.

## MC01 human + MC02 AI P0 — bounded Windows input feasibility (30 October 2026)

- The same-executable native agent CLI gained a manually opted-in
  `probe-background 2` diagnostic ONLY. The authenticated session controller
  must confirm responsive running MC01 and MC02. The canonical HWND verifier
  retains process identity (PID/birth/session/window) throughout the attempt.
- The diagnostic posts only one bounded W-key down/up pair to MC02's window
  with MC01 as the verified foreground. It never sends global SendInput,
  activates AI windows, owns a second game controller, changes a world,
  introduces a new daemon, or exposes an agent input tool.
- The result `WINDOW_MESSAGES_POSTED_ONLY` with `gameplayObserved:false`
  is NOT proof Minecraft Education accepted the input, and is NOT an
  independent WASD/mouse implementation. If background Windows messages
  are ignored by Minecraft, reject this approach; do not label it PASS.
- Tests authored: argument parsing rejects every profile other than MC02,
  source policy checks for down/up flags, MCP refuses experimental input.
  These do not establish game movement or safe concurrent human controls.

**Proof ceiling:** REMOTE_GITHUB source-only. Windows Rust build, installed
CLI, real game background input, human input preservation, auto host/join,
and four AI/5 AI gameplay remain NOT_EXECUTED or NOT_IMPLEMENTED.

## Source-only multi-player AI coordination contract — 30 October 2026

- Added one stateless `control-plan.mjs` owner for both modes: MC01 human
  host + MC02–MC05 AI, or MC01–MC05 AI with MC01 as intended host. The
  `plan_peer_session` MCP tool reuses exact native Inspect evidence, refusing
  to infer world creation, sign-in or peer joins from RUNNING profiles.
- Added `plan_player_actions` to validate and sort up to 32 bounded intended
  actions with explicit slot and timing. Hybrid mode rejects MC01; overlapping
  actions on one slot are rejected. No actions are dispatched; every plan
  declares `BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`.
- Authored source contract tests in the existing Node test convention and
  extended the current MCP fixture; **no tests, builds, launcher or Minecraft
  operations executed**, following the operator's source-only instruction.
- UI, original five-profile lifecycle, official game binaries, account/world
  storage, and disabled GitHub Actions remain unchanged.

**Proof ceiling:** exact GitHub source/blob review only. Native Minecraft
parallel input remains UNVERIFIED, all execution NOT_EXECUTED.

## Bounded multi-client gameplay evidence summaries — source-only

- Added `evidence-summary.mjs` as a pure, bounded consumer of the existing
  per-profile Skills Probe capture reader. MCP `summarize_test_evidence`
  provides a compact cross-slot timeline, deduplicates stable record IDs,
  surfaces meaningful failure/result signals and reports missing captures.
- Agent-visible data is structured, limited and explicitly untrusted. It does
  not claim authenticated MC/profile-player mapping or infer world joins,
  actor control, or pass/fail game acceptance from captured log markers.
- Added source-only tests and connected them to the existing deferred
  `test:localtest` command. **No code or tests were executed** because the
  operator explicitly deferred all local tests until later approval.

**Proof ceiling:** GitHub source review only. In-game evidence acquisition,
Windows MCP execution and parallel input are NOT_EXECUTED.

## One-request AI observation handoff and deferred execution policy

- Added `get_test_context` to the same MCP owner: one authenticated native
  Inspect plus existing stateless peer-session plan and compact per-slot
  analytics/results evidence. One read-only request instead of several
  agent/tool round trips. Input backend remains NOT_VERIFIED.
- No new agent runner, background monitor, native window command, data store
  or UI route was added; optional capture status remains unverified.
- **Operator restriction:** continue source-only GitHub development. Do not
  run Node/Vitest tests, Windows Rust/Tauri builds, installer packaging, native
  probes or Minecraft local tests until explicitly authorized in a later turn.
  New source tests are written for future execution only.

**Proof ceiling:** REMOTE_GITHUB exact-source review, no execution at this
revision.

## Rejected: MakeCode/file relay as gameplay controller (30 October 2026)

The operator rejected MakeCode for AI player control because teleport and
scripted world actions cannot reproduce responsive, independently controlled
player movement, camera, PvP, or context-sensitive game interaction. The
experimental file relay and its deferred tests were removed from the active
`Local` tree; the historical commit remains in Git only as provenance.
Do not reintroduce MakeCode/file relay, teleport-as-WASD, or scripted
substitutes as AI player controllers.

The required outcome remains five **real** Minecraft Education Windows
clients sharing one normal peer-to-peer hosted world: MC01 human + four AI
or all five AI. Agent actions must target independently controlled game clients
without taking the human's keyboard/mouse focus. Codex/Claude should use
actual gameplay observations for decisions and testing. Independent background
input remains UNIMPLEMENTED/UNVERIFIED; do not imply `PostMessageW` has
passed Minecraft gameplay acceptance.

**Proof:** REMOTE_GITHUB cleanup only; no tests/build/Minecraft execution
authorized.

## Unverified / deferred

- Exact Windows dependency-reuse behavior, source regression tests,
  desktop/compiler integration, signed/installed launch and full five-session
  gameplay remain unexecuted at this revision.
- The historical Cargo archive's real eligibility, linked-file risks and
  physical storage reclamation are UNKNOWN. Preserve all archive contents
  unless the separate approved retirement gate and actual evidence allow it.
- Fresh-profile gameplay, cross-volume data migration, live DWM pixels and
  reconnect/normal-exit paths need their existing target-Windows acceptance.
- Extra RAM/FPS or graphics optimization, macOS support, new native providers,
  hosted runners and CI remain outside the active development scope.

## Next — source-only continuation, testing deferred by operator

Keep the five-client AI control/observation source bounded to the current
native controller and Skills Probe evidence contract. Prepare further safe
source corrections and future independent-input adapter contracts only where
they have a justified consumer; do not claim Minecraft moved in the background.
**Do not execute local tests, toolchain builds, package staging, native input
probes or gameplay until explicit operator approval.** When approval arrives,
the first separate acceptance boundary is the pinned Windows agent CLI,
followed by MC01 human + MC02 AI independent input with actual game evidence.
No VM, dedicated server, or global SendInput fallback is authorized.

Historical work notes and prior Next proposals remain in immutable
Local Git history, not in a parallel current-status register.
