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

## Windows Computer Use Phase 1 — native target handoff (source only)

- Added `get_computer_use_target` and `revalidate_computer_use_target` to
  the existing local stdio MCP, with human MC01 explicitly denied in
  `human-plus-four-ai` mode.
- Same-image Rust `--lazy-tester-agent-cli target <slot>` reads the existing
  authenticated retained controller's Inspect and uses the canonical native
  window verifier (PID, birth FILETIME, session and root HWND). The result
  includes a numeric window selector for an optional external Cua Driver
  `get_window_state` request, with NO focus change and no process adoption.
- Strict schema and exact identity comparison refuse stale/replaced windows
  in the MCP. No clipboard, keyboard, mouse, gameplay action or screenshot
  is generated by Lazy Tester. Cua's own background limitations for custom
  GPU game surfaces remain applicable and cannot be bypassed by MCP.
- Added authored (not executed) source tests and an agent-side handoff guide.
  No new dependencies, runner, installer, dedicated server, game DLL, OS
  driver or source-owner duplication.
- **Operator restriction remains:** source-only work on GitHub. Local tests,
  Windows build, native execution, packaging and Minecraft gameplay are
  explicitly deferred until user approval.

**Proof ceiling:** REMOTE_GITHUB exact-source inspection only. Native target
query correctness, Cua window capture and multi-client background input
NOT_EXECUTED / NOT_VERIFIED.

## Windows Computer Use Phase 2 — read-only screenshot MCP (source only)

- Added optional `observe_client` to the existing local stdio MCP. It
  resolves the native owned Minecraft HWND for MC01–MC05, calls an explicitly
  configured external Cua Driver **read-only** `get_window_state` CLI to
  obtain a bounded per-window PNG, then rechecks exact process birth/window/
  session identity before returning an MCP image content block.
- The optional Cua daemon is never installed, started, or granted global
  input automatically. No extra game controller, login owner, agent daemon,
  video recorder, WinRT graphics dependency, or native game injection added.
- Hybrid mode refuses MC01; image reads are bounded to one window. Temporary
  capture is cleaned on normal exit. A valid PNG is *not* evidence of a fresh
  rendered Minecraft gameplay frame, world join or functioning background
  input; the system reports those separately as unverified.
- Authored one visual-capture source test and extended existing MCP source
  fixtures for human protection and screenshot rejection on changed HWND.
  **No test, build, install, local computer or Minecraft execution** was
  performed per current operator instruction.

**Proof ceiling:** REMOTE_GITHUB source review only; Cua capture integration,
Windows live game pixels and parallel game input NOT_EXECUTED/UNVERIFIED.

## Phase 3: local rolling visual observation for Codex — source only

- Added a bounded `visual-observer.mjs` owned by the current MCP stdio
  process: explicit start/read/stop, five per-slot in-memory latest-frame
  caches, timeout after at most 90 s, serialized Cua capture attempts, and
  adaptive backoff for repeated PNG bytes. MC01 stays excluded in hybrid mode.
- MCP now exposes `start_visual_observation`,
  `read_visual_observation`, `stop_visual_observation`. Model receives
  an image only when it explicitly reads a newer sequence; unchanged frames
  return metadata only. The controller is closed with MCP stdio.
- Reused existing exact-native-window verification before/after every local
  Cua capture, retaining PID/HWND/process birth/session identity across
  samples. Changed targets/failures terminate and drop cached pixels.
  Cua reads now use `display_only:true` to avoid invalidating action
  snapshots. A cached PNG older than 3500 ms cannot be delivered to Codex
  as a current frame (even while an observation session is still open).
- No UI, input injection, additional native controller, new long-running
  standalone service or AI decision daemon. Underlying provider still makes
  **bounded sequential CLI snapshots** rather than persistent high-FPS
  Windows Graphics Capture. This is not real-time semantic game perception.
- Authored source-level fixtures only; **no Node tests, Windows build,
  local/installed app operations or Minecraft gameplay executed**.
  Windows Cua behavior, game-frame freshness, latency, memory/CPU impact
  and independent game input are NOT_VERIFIED.

**Proof ceiling:** REMOTE_GITHUB source/consumer inspection only. Await
explicit operator permission before any runtime test or installed changes.

## Phase 4: optional native persistent GPU capture through existing MCP — source only

- Added an isolated Windows WGC 3.1.0 Rust window producer with pinned
  process/birth/session/root-HWND checks and a bounded binary PNG stdout
  stream. Added a Node reader to the existing MCP process: per-slot latest
  frame only, protocol integrity/ordering, age validation, deadlines and
  no retry. This does NOT create a new Minecraft lifecycle owner.
- Existing MCP visual observation tools can now select the source-only
  WGC provider via `--gpu-capture-exe`. The unchanged default Cua path
  remains available; all GPU workers require an explicitly configured
  binary and a native controller verification for each selected slot.
  On partial start failure, workers from that request are stopped.
  When a visual slot terminates (expiry, replaced HWND, repeated errors,
  user stop), its native GPU worker is explicitly killed and its PNG cache
  discarded; merely expiring the MCP sampler cannot leave the producer live.
- Source-level tests authored for frame protocol, worker lifecycle and
  MCP target/MC01 protection. No new UI, internet server, game input,
  world/login automation, CI, dependency tree in the native Tauri app or
  installed package was changed.
- **No local tests, Cargo build, Windows executable, GPU capture or
  Minecraft operation were performed.** The standalone worker Cargo
  dependency lock is intentionally absent pending authorized toolchain
  resolution; this feature is NOT ready for distribution.
- WGC may require OS permission and show a capture border; captured
  pixels might be stale/black or absent with a background game. Five GPU
  capture producers' performance and actual frame freshness are UNKNOWN.

**Proof ceiling:** REMOTE_GITHUB source review only. Do not claim that
Minecraft can be observed at live gameplay speed or controlled by AI.

## Phase 5: simultaneous AI input channels — source-only

- Updated the canonical `control-plan.mjs` so one AI Minecraft can
  simultaneously move, strafe, look, jump, attack and interact using
  distinct input channels. Conflicts on the same channel and exclusive
  waits remain rejected, as does MC01 in human-plus-four-AI mode.
- Added bounded relative mouse-intent deltas and a pure
  `input-timeline.mjs` compiler of timestamp-grouped PRESS/RELEASE/
  POINTER_DELTA semantics. The existing MCP `plan_player_actions`
  optionally includes this timeline via `includeTimeline:true`.
  The default is unchanged; there is NO executable input command.
- All plans remain `NOT_DISPATCHED` and gated by
  `BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`. The future provider
  must implement real isolated controls and guaranteed release-all,
  independent of the authored emergency-release intent list.
- Regression tests authored for per-client simultaneous channels,
  conflicting directions, MC01 protection, delta bounds, ordering,
  and emergency-release intent. **No Node/Rust/local tests, builds,
  installation or Minecraft execution performed**.
- No game DLL hooks, input driver, raw desktop SendInput, MakeCode or
  UI changes were introduced.

**Proof ceiling:** REMOTE_GITHUB source-only. Independent background
WASD/mouse for Minecraft Education and four/five AI gameplay still
UNIMPLEMENTED or UNVERIFIED.

## Phase 6: native-identity-bound independent input route (source only)

- Added `input-route.mjs` as a read-only binding from existing
  `input-timeline.mjs` schedules to native PID/HWND/birth/session
  observations; no second Minecraft session controller or input worker.
- MCP `assess_input_route({actions})` inspects targets through the
  retained controller. In hybrid mode it observes human MC01 solely as
  protected reference and returns AI bindings for MC02–MC05. It rejects
  duplicate process/window identities, missing/unresponsive targets,
  unexpected session mixing and snapshots older than 5 seconds.
- The route always reports `NOT_DISPATCHED` /
  `BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`. It does not claim
  a particular HWND can receive game input or that the human desktop
  has been preserved. No background keyboard/mouse dispatch, DLL
  injection, global SendInput, MakeCode, or OS driver is installed.
- Source-only fixtures were added for protected MC01, five-AI mode,
  identity conflicts, stale observation and MCP native inspection.
  **Tests, builds, local execution and Minecraft runtime remain
  explicitly forbidden until new operator approval.**

**Proof:** REMOTE_GITHUB exact-source and consumer review only.
Real independent game-input routing remains BLOCKED pending supported
Windows input mechanism and MC01/MC02 gameplay proof.

## Phase 7: bounded input transaction port — source only

- Added `input-delivery.mjs` to the existing agent tooling: time-bounded
  semantic event scheduling across independent MC slots, event ordering
  within a single slot, fresh exact-native target checks per timestamp
  group, per-client admission and hybrid human-desktop protection callbacks,
  cancellation, and mandatory per-client `releaseAll` attempts. Reports
  transport-only acknowledgements, never gameplay acceptance.
- Production has NO independent Windows game-input backend. The runner's
  callback port is NOT exposed as an agent MCP execution tool. MCP
  `agent_capabilities` and `assess_input_route` explicitly report
  `provider:NONE` / `execution:DISABLED` / the same BLOCKED gate.
- Future real native provider must independently guarantee release-all
  under process crash/hang, exact target checking during input, no global
  keyboard/mouse dispatch and MC01 human preservation; Node callbacks
  alone cannot deliver those assurances.
- Source fixtures (NOT_EXECUTED) cover no-backend refusal, concurrent
  controls, target replacement, protected MC01, cancellation, and
  missing release ACK. No new Minecraft/game owner, installer, driver,
  DLL injection, CI, or foreground input route is introduced.

**Proof ceiling:** REMOTE_GITHUB exact source and public API review only.
Actual Windows independent input, real Minecraft player movement,
MC01 uninterrupted input and five-client gameplay are NOT_EXECUTED.
Local tests/builds/gameplay remain prohibited until explicit permission.

## Phase 8: non-production virtual XInput signal source candidate

- Source audit of ProtoInput `protoloader.h` confirms its documented API
  primarily selects physical input devices for hooked processes; it is
  NOT an existing AI synthetic-key-to-specific-Minecraft backend.
- Added optional independent `native-gamepad-lab/` Rust candidate using
  `vigem-rust 0.2.0`: when separately built and explicitly approved,
  it could produce ONE global virtual Xbox 360 input device, with bounded
  timed reports, 350 ms neutral watchdog and ten-second hard duration.
  No driver installation, injection or native app integration occurred.
- Added pure `global-gamepad-candidate.mjs` mapping genuine Minecraft
  gameplay intent into Xbox analog sticks/buttons/triggers with calibrated-
  later camera-rate semantics, 200 ms report heartbeat, final neutral.
  It refuses the human-plus-four-AI mode and multiple simultaneous slots
  because a global pad cannot be proven exclusive to MC02.
- This is an input *signal* path, NOT independent game-input routing.
  Minecraft Education's background gamepad behavior and unique per-client
  device assignment remain unverified. Codex MCP still reports
  `provider:NONE`, `execution:DISABLED` for actual input.
- Source tests authored; NOT_EXECUTED. No local commands, builds, packages,
  Windows driver actions, Minecraft gameplay, CI or user-PC access.

**Proof:** REMOTE_GITHUB pinned source/API review only. Virtual
gamepad provider NOT_BUILT, device signal NOT_EXECUTED, and independent
multiplayer control remains BLOCKED.

## Phase 9: global gamepad safety and device isolation evidence (31 October 2026)

- The optional Rust experiment previously accepted future queued reports
  without a live input producer. Source now rejects reports over 100 ms
  early or 250 ms late, non-contiguous sequence/timestamps, unbounded
  stdin queueing, truncated final lines and missing explicit final neutral.
  This is NOT a verified runtime watchdog; all execution is deferred.
- The mapper stays an offline action-to-pad compiler. No paced sender,
  driver installation, game process hook, MCP input tool or desktop
  production device route was added.
- The existing read-only native input route now reports an XInput capacity
  diagnostic separately from per-client device assignment. Standard XInput
  covers at most four controller indices; meeting that count still does
  NOT isolate MC02–MC05. Five AI devices exceed that API's capacity.
- ViGEmBus is retired; HidHide's image-path application filtering and
  global device hiding cannot independently route controllers to copies
  of the same Minecraft executable. ProtoInput per-instance XInput index
  redirection uses game-process hooks, outside the approved approach.
- Background Minecraft Education gamepad response, MC01 human safety and
  multi-AI gameplay remain NOT_EXECUTED/UNVERIFIED.

**Proof:** REMOTE_GITHUB source/upstream review only; no tests/builds/driver/game.

## Phase 10: Windows same-session input routing and feasibility (31 October 2026)

- Fixed input-route admission: mixed Windows session IDs now fail closed
  in both `five-ai` and `human-plus-four-ai` (previously only hybrid
  mode compared MC01 session ID against its AI targets).
- Read-only `assess_input_route` and `agent_capabilities` now report the
  existing single-session native controller limit. Microsoft's experimental
  Windows Agent Workspace uses separate Windows sessions, and is therefore
  not a drop-in backend for this controller or evidence of Minecraft input.
- Fixed one existing MCP authored fixture incorrectly rejecting the
  intentionally available read-only `assess_input_route` tool by substring
  `input`. It now distinguishes nonexecuting assessment from actuation.
  Updated the existing input-route and MCP test files only.
- Windows research: system-wide synthetic input has no exclusive PID
  routing; Raw Input background reception requires game registration;
  GameInput focus policy is selected by its app; ProtoInput uses game
  process hooks (out of approved scope). No supported non-injecting
  per-process keyboard/relative-mouse sender has been established.
- Minecraft Education keyboard/mouse control is supported; public
  controller documentation/support is inconsistent, so gamepad support
  stays UNVERIFIED, not a basis for production enablement.

**Proof:** REMOTE_GITHUB sources and upstream documentation only.
All source tests AUTHORED NOT_EXECUTED; native builds, Windows gameplay,
injection and CI remain unexecuted/disabled. No input tool enabled.

## Phase 11: eliminate dead-end gamepad lab; native input research (31 October 2026)

- Reviewed the exact active source graph before removing dormant code.
  The optional global virtual-gamepad mapper, test and Rust ViGEm lab
  were not connected to Lazy Tester desktop, MCP input execution,
  native session controller or package. Retired their five files from
  `Local`, removed one optional test entry from `Development/package.json`,
  and removed the obsolete `legacyVirtualGamepadLab` diagnostic object
  plus its nonessential fixture assertions. Their exact source remains
  in prior Git history (Phase 10 HEAD `fc4d87d9defe`).
- A Minecraft Education support response dated 28 September 2026 says
  computer play currently uses keyboard and mouse; older documentation
  retains controller settings, so Xbox controller suitability is not
  accepted without actual runtime proof. Prior Phase 8/9 records below
  describe historical work, not files still available on `Local`.
- Additional comparison found `openMouse` (single foreground keyboard
  focus), `MultiSeat` (independent Windows RDP sessions but requires
  RDPWrap, drivers and additional software), `Duo` (parallel desktops
  via patched drivers / components), and ProtoInput (process hooks).
  None has been validated with five Minecraft Education accounts.
  These are NOT approved as an installer, input provider, OS-patch or
  controller replacement. Windows session isolation would require a
  distinct architecture and licensing/security review; the retained
  native controller is intentionally single-session.
- Keep `inputExecutionStatus().provider='NONE'`, all input MCP actions
  unavailable, human MC01 protection, and route execution
  `NOT_DISPATCHED`. No experimental system software was installed.

**Proof ceiling:** REMOTE_GITHUB exact-source static inspection and
upstream documentation. Tests/builds/Windows/gameplay NOT_EXECUTED;
native game input still BLOCKED.

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

Keep the existing single-session native identity and gameplay planner
unchanged. The next practical step is an architecture decision:
a compatible per-process input mechanism must be evidenced, or a
separately approved true multi-session design must satisfy Windows
licensing, input isolation, Education P2P and human desktop protection.
Do not substitute global keyboard, device, simulated gameplay or
Windows system patching for actual per-client input proof.
**Do not execute local tests, toolchain builds, package staging, native input
probes or gameplay until explicit operator approval.** When approval arrives,
the first separate acceptance boundary is the pinned Windows agent CLI,
followed by MC01 human + MC02 AI independent input with actual game evidence.
No VM, dedicated server, or global SendInput fallback is authorized.

Historical work notes and prior Next proposals remain in immutable
Local Git history, not in a parallel current-status register.
