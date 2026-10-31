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

## Phase 12: supported Windows multiseat viability decision (31 October 2026)

- Verified `native_clients_session/pipe.rs` enforces same Windows
  logon/session, user/integrity and executable peers. The existing
  controller cannot adopt Windows Agent Workspace / RDP session targets.
  Win32 alternate desktops have only one active input desktop.
- Official Minecraft Education Windows installation guidance explicitly
  excludes **Windows Server clients**. Microsoft disallows production
  Windows Enterprise multi-session outside Azure Virtual Desktop;
  this project has no approved cloud/VM fallback.
- Reviewed MultiSeat's own README: RDPWrap + RDP sessions and drivers
  are required; its optional keyboard/mouse isolation hook is a known
  no-op. No term-service patching, RDPWrap or external streaming/driver
  integration is authorized.
- Microsoft documents contained ODR MCP processes inside a separate
  agent Windows session, with packaged .exe/MSIX requirements and no
  direct access to human-session apps/windows. This is the strongest
  first-party research path, but not a drop-in extension of the
  current Node stdio MCP or native Rust session controller.
- A one-human/one-AI foreground-in-separate-session gameplay test,
  multiple independent workspaces, Education graphics/sign-in,
  per-account host/join and Codex connection are all unproven.
  Do not create another controller/daemon, Windows installer or
  "test mode" for an unverified mechanism.

**Decision:** retain existing native architecture and input BLOCKED.
**Proof ceiling:** REMOTE_GITHUB plus official vendor/maintainer source
review; no tests, builds, installs, gameplay, RDP or agents executed.

## Phase 13: on-device Windows Agent Workspace / Codex viability (31 October 2026)

- Official Windows ODR contained MCP server documentation confirms that
  packaged .exe MCP servers can run and launch other executables *inside
  an agent's Windows session*; it does NOT supply a demonstrated local
  per-client game keyboard/relative-mouse/screenshot provider, or proof
  of Minecraft Education graphics, sign-in, networking or game acceptance.
- Containment requires package identity and ODR registration; the current
  Codex Node stdio MCP is a different connection mechanism. Generic
  Codex MCP configuration does not create a Windows Agent Workspace,
  and the public docs inspected do not establish a working Codex-ODR
  lifecycle for Lazy Tester. Do NOT weaken connector containment.
- Windows 365 for Agents offers documented computer-use actions and
  session lifecycle, but acts on Cloud PCs. It is excluded by the
  approved local-only, no-cloud/no-VM scope and is NOT ODR Agent Workspace.
- **Decision:** close Windows Agent Workspace as an immediately
  implementable independent Minecraft input backend. Do not create
  new ODR/MSIX/agent-session/input source without a concrete supported
  API and acceptance evidence. Existing one-session native controller,
  read-only MCP and disabled input dispatch remain unmodified.

**Proof:** REMOTE_GITHUB plus primary Microsoft/OpenAI documentation.
No test, build, packaging, ODR, device, Windows or Minecraft execution.

## Phase 14: GPU observation source-to-client identity binding (31 October 2026)

- Source audit found a real attribution gap: `captureVerifiedFrame`
  inspected a native target before/after reading the persistent GPU
  stream, but the cached WGC worker was pinned at *start* to a potentially
  different PID/HWND/birth/session. A restarted client could receive
  the previous worker's frame as if it belonged to its replacement.
- Tightened the existing `gpu-frame-source.mjs` owner: `latest(slot,
  expectedNativeTarget)` now requires and compares the exact native
  process identity against the original worker binding before exposing
  any PNG. A mismatch stops that read-only worker and discards its cache.
  MCP passes the verified `before` target and still checks the `after`
  target. Future-dated capture frames are not presented as fresh.
- Existing source fixtures extended for four independent identity
  mismatches and future timestamps, plus MCP contract verification.
  No extra service, store, broad test suite, driver, game input route or
  Minecraft session owner introduced. MC01 remains protected in hybrid.
- **Proof ceiling:** REMOTE_GITHUB source-only. Tests AUTHOR_ONLY /
  NOT_EXECUTED; native Windows WGC pixels, Minecraft game frames and
  agent gameplay remain UNVERIFIED.

## Phase 15: bounded observation startup and evidence session isolation (31 October 2026)

- The optional GPU worker's `start` waited for an asynchronous file check
  before reserving a slot. Concurrent starts, stop, or shutdown could race
  and leave a late child worker. The existing source now reserves pending
  slots synchronously, cancels on stop/shutdown, and attempts termination
  of unsupervisable/partly initialized workers. No Minecraft game process
  is targeted by these cleanup paths.
- MCP's unfinished multi-slot GPU startup now honours a matching Stop
  request and reports `STARTUP_CANCELLATION_REQUESTED` until the original
  start call settles; remaining requested workers cannot start after
  cancellation. Known native/GPU identity replacement causes immediate
  visual sampler stop instead of repeated stale-capture retries.
- Skills Probe evidence dedupe was scoped only to MC slot + record ID,
  allowing reused IDs across different game sessions to overwrite one
  another. The existing summary now includes `session_id`, and refuses
  dedupe when session identity is absent. File, player and world identity
  remain UNVERIFIED, and evidence is not gameplay acceptance.
- Existing targeted source fixtures were extended for pending startup
  stop/shutdown, missing worker stdout, MCP cancellation, immediate
  identity-based sampler shutdown, and cross-session record collisions.
- **Proof ceiling:** REMOTE_GITHUB exact source and authored fixtures only.
  Tests/builds/Windows Minecraft execution NOT_EXECUTED. Input backend
  remains BLOCKED and MC01 protected.

## Phase 16: bounded Skills Probe file replacement and partial-write safety (31 October 2026)

- Identified a real read race in `mcp.mjs`: source tail offsets came
  from pre-open `lstat(path)` and could be applied to a different
  rotated/replaced log after `open(path)`. Active append/truncate
  also lacked a consistency check at the end of the bounded read.
- `readGameEvidence` now compares the ordinary named source with
  its opened file handle via a nonzero exact file ID, device number,
  size and nanosecond ctime/mtime. It reads the pinned 256 KiB tail
  with bounded positional-read completion, then compares handle and
  pathname state again before decoding a single completed line.
  Replacement, disappearance or concurrent mutation produces
  `CAPTURE_FILE_UNSTABLE` and **no events**. Ordinary initial
  absence remains `NO_CAPTURE_FILE`; symlink paths still fail closed.
- Summary now keeps unstable sources explicitly unaccepted and
  surfaces whether an incomplete final line was ignored.
  Player/account/world identity remains OPERATOR_NAMED_UNVERIFIED.
- Existing test files were extended with controlled rotation, append,
  short-read/partial-line and unstable-source aggregation fixtures.
  There is no new daemon, test runner, game lifecycle owner, driver,
  debugger listener, input route or CI workflow.
- **Proof ceiling:** REMOTE_GITHUB exact source/static review only;
  regression fixtures AUTHORED NOT_EXECUTED. Windows file-identity
  API behavior, concurrent actual Skills Probe writes, Minecraft
  gameplay and independent AI input remain NOT_EXECUTED/UNVERIFIED.

## Phase 17: Skills Probe parser and result-classification trust boundary (31 October 2026)

- Found that `parseCapturedLine` searched marker substrings anywhere
  inside debug text and accepted any JSON object following
  `[GAME_RESULTS]`. The summary likewise labeled any internal
  `game_results` event a `RESULT` without checking its record type.
- The existing parser now requires a parseable timestamp, one leading
  source marker, a bounded JSON object, and a non-empty event type.
  Game results specifically require the currently recognized
  `schema:1/record_type:'gameplay_result'` contract. Analytics with
  a nonempty `type` and no version remain legacy unverified telemetry.
- The existing summary applies the same typed game-result check even
  when called through a fixture/evidence reader rather than the parser.
  `resultCount` counts recognized untrusted record markers, not
  successful gameplay; `worldIdentity` and `playerToProfileIdentity`
  remain NOT_VERIFIED. No account or world claims are promoted.
- Reused existing Node test files for incidental marker text, untyped
  `passed:true`, wrong schema, canonical events, and direct-caller
  result counting. No extra tests/test runners, input route or
  Minecraft lifecycle authority were introduced.
- **Proof ceiling:** REMOTE_GITHUB exact source and authored fixtures
  only; source tests, Windows debugger output and Minecraft gameplay
  NOT_EXECUTED. Compatibility with the real Skills Probe log producer
  remains UNVERIFIED until approved runtime evidence is available.

## Phase 18: bounded MCP response serialization and evidence privacy (31 October 2026)

- Found two concrete outward trust-boundary leaks in `mcp.mjs`:
  `read_game_evidence` returned arbitrary nested Skills Probe JSON,
  while JSON-RPC tool failures forwarded `error.message` from native/
  filesystem/providers (potentially revealing local paths or private
  diagnostic strings). There was no explicit serialized text response
  ceiling on model-visible MCP tool content.
- The raw per-slot tool now reuses the existing summary projector,
  retaining its name and slot/status/events envelope but returning
  bounded allowlisted event metadata, not arbitrary `data` objects.
  This intentionally narrows the event shape; summary remains source
  of truth for multi-slot evidence and still says gameplay not proven.
- JSON-RPC errors expose only fixed allowlisted policy strings and
  narrowly constrained native refusal codes; unexpected errors are
  replaced with `Tool unavailable; details withheld`. Source
  `callTool` exceptions are unchanged. MCP text responses are bounded
  to 128 KiB and image base64 responses to 8 MiB; oversized outputs
  are refused without returning partial content.
- One existing MCP regression file extended for raw-field redaction,
  a private-path error, legitimate MC01 refusal, and oversized text/
  image responses. **AUTHORED_NOT_EXECUTED** under operator policy.
- **Proof ceiling:** REMOTE_GITHUB source review only. Tests, Windows
  file/system execution and Minecraft gameplay are NOT_EXECUTED;
  producer/player/world identity remains UNVERIFIED, independent AI
  gameplay input remains BLOCKED.

## Phase 19: five-profile native status projection and MCP privacy (31 October 2026)

- Read canonical `engine/agent_cli.rs`: current Rust inspect/launch
  output already exposes only `slot/state/hasWindow/responding/
  needsAttention` per profile, plus fixed schema/operation/evidence.
  No actual Rust response path or account/world leak was observed.
- Found a Node boundary weakness: `validateNativeReport` checked the
  five required fields but returned the entire original object, and
  the direct MCP `inspect_profiles` and `launch_profile` handlers
  did not invoke it for injected/future providers.
- `validateNativeReport` now projects an exact five-profile status
  allowlist and fixes the non-gameplay evidence marker. Inspect and
  launch handlers use it; existing `get_test_context` and
  `plan_peer_session` calls inherit the same narrowed output.
  Malformed native required fields continue to fail closed.
- Retained Phase 18's 128 KiB JSON text-response ceiling; its prior
  injected oversize native-object test was corrected to expect
  sanitized output and instead checks the serializer ceiling directly.
  Added targeted tests for private native fields and invalid status
  in the existing test file, AUTHORED_NOT_EXECUTED.
- **Proof:** REMOTE_GITHUB static source and consumer review only.
  Node/Rust test suites, real Windows controller, Minecraft account
  binding/gameplay and five-client acceptance NOT_EXECUTED. AI input
  remains BLOCKED and MC01 protection unchanged.

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

## P0 source correction: explicit MC02 experimental launch (31 October 2026)

The earlier source-only native input candidate had a blocking admission
bug: `mc02_input::Channel::create` read an ambient opt-in flag in the
already-running native controller, not the separately launched Codex MCP
or one-shot CLI. An explicit MCP opt-in therefore could request input
to an ordinary MC02 whose shared input section did not exist.

The same authenticated session controller now admits a feature-gated
`LaunchMc02Experiment {slot:2}` request from the existing one-shot
`launch-mc02-input 2` operation. That CLI requires the exact experimental
environment opt-in, and the MCP selects it only for MC02 in
`human-plus-four-ai` mode when `--mc02-input-experimental on`
has already been explicitly configured. The existing `launch_profile`
tool name and normal launch path are unchanged.

The original bootstrap creator has a separate explicit
`experimental_input` argument at its internal entry point; every
ordinary bootstrap call passes false. This avoids relying on ambient
process environment or changing the profile/world storage, controller
ownership, named pipes, or already-enrolled game DLL mechanism.
Experimental MC02 launch requires the controller to own MC01 already;
all other slots remain normal, no other client is hooked by this mode.

**Input command critical-path correction:** the previous opted-in MCP
action started TWO additional `target` one-shot CLI processes to inspect
MC01 and MC02 before the THIRD `mc02-input` CLI could write one input
packet. All of those launches consumed the same 500 ms command deadline.
The actual authoritative check already lives in the retained Rust
controller, not in the MCP preflight. MC02 input now starts only the
one `mc02-input` CLI; the controller itself checks both independently
owned/responding windows, same session, exclusive MC01 foreground,
MC02 visibility/non-minimization and original PID/birth/root HWND
identities while holding BOTH verified process handles through the
shared-memory write. Its existing 500 ms monotonic stale-request check
remains the final gate. No global input, new executable, new provider
or product mode was added. The existing focused fixture was corrected
to reject duplicated CLI target calls; AUTHORED_NOT_EXECUTED.
This is a source-side input delivery latency improvement, NOT proof
of actual Minecraft Education movement/camera/attack.

**Source evidence only:** exact GitHub review; existing focused regression
fixture AUTHORED_NOT_EXECUTED. Rust/Node/MSVC build, Windows input
and Minecraft gameplay NOT_EXECUTED. The separately reviewed installed
native DLL/hash admission is not bypassed, and Raw Input/GameInput
acceptance remains UNVERIFIED. Default production input remains BLOCKED.

## P0 source implementation: controller-to-MC02 virtual input state (31 October 2026)

- Added an **opt-in native input path, not another observer**. Existing
  Rust controller allocates a private unnamed shared section before
  launching its verified MC02 child, inherits a read-only handle into
  that exact game and retains the sole writable handle. Existing
  authenticated session request / same-image agent CLI can publish a
  bounded key mask (WASD, jump, sneak, attack/use) and signed mouse
  deltas for MC02 only; no second IPC server or controller is created.
- The existing bootstrap DLL gains an optional, x64-only Win32 polling
  adapter that consumes this state for GetAsyncKeyState, GetKeyState,
  GetKeyboardState, GetCursorPos, SetCursorPos and ClipCursor. The
  physical OS input, global cursor and MC01 game process are untouched
  by these new hooks. Input expires automatically after at most 500ms
  and virtual state then reports released controls, even if the command
  writer stops. The game is *not* assumed to use these specific APIs.
- This source is behind two **nondefault compile options**:
  Cargo `mc02-native-input` and CMake `BOOTSTRAP_MC02_VIRTUAL_INPUT`.
  Both the already-owned child and the experimental CLI require
  explicit `LAZY_TESTER_MC02_VIRTUAL_INPUT=I_ACCEPT_EXPERIMENTAL_MC02_INPUT`.
  The default MCP has no input tool, and no DLL/input code is executed in
  this conversation. An additional development-only MCP tool is now
  available **only** when --mc02-input-experimental on is paired with
  LAZY_TESTER_MC02_VIRTUAL_INPUT=I_ACCEPT_EXPERIMENTAL_MC02_INPUT in
  human-plus-four-ai mode. It accepts 1-500ms MC02 keyboard/button and
  signed cursor delta commands, checks MC01 foreground and distinct
  MC02 identity, and delegates to the same authenticated native CLI.
  Its receipt explicitly says state written, gameplay UNVERIFIED; no
  normal-product input authorization is implied.
- Private input section ABI: 40-byte fixed header, PID, session and
  process birth, then one 64-bit atomic command. The controller's
  inherited read-only handle cannot write commands; it is bound to the
  original child before resume. No named mapping, profile data,
  account token or world file is created or modified.
- **Limits:** this candidate implements Win32 polling APIs only. It does
  not intercept GameInput, DirectInput, the Raw Input WM_INPUT queue or
  Minecraft-specific input. Actual Minecraft Education movement,
  camera and hit detection are **NOT_EXECUTED/UNVERIFIED**. Virtual
  cursor behavior and Minecraft keybind mapping are uncalibrated.
  It must not be described as a production-ready game-input backend.
  The existing MCP still advertises `actualInputEnabled:false`.
  Follow-up static review corrected a feature-gated Rust match branch:
  both Cargo feature-OFF and feature-ON configurations now use valid
  conditional code. A further source correction makes the optional
  DLL attach a no-op for MC01 and MC03-MC05; previously it returned
  failure for every non-MC02 client. Mutually exclusive optional CMake
  observation/virtual-input detours prevent duplicate API interception.
  Repeated identical pointer commands now have a reserved sequence
  byte so a second command within the same Windows tick is not dropped. The optional DLL still needs the reviewed binary
  compatibility/hash admission and cannot silently replace the installed
  bootstrap payload. This is source, not a successful Windows build.
- The exact code was reviewed on GitHub only; Rust/CMake/MSVC builds,
  Windows system calls, full client lifecycles and gameplay remain
  NOT_EXECUTED. No localtest was requested or performed.

## P0 source correction: MCP-to-native MC02 command path (31 October 2026)

- The exact existing native one-shot `mc02-input` CLI is now reachable
  from a dedicated experimental MCP command ONLY under explicit
  `--mc02-input-experimental on` plus
  `LAZY_TESTER_MC02_VIRTUAL_INPUT=I_ACCEPT_EXPERIMENTAL_MC02_INPUT`
  in hybrid mode. The normal tools/list is unchanged; native
  `actualInputEnabled` remains false because Minecraft gameplay is
  not proven. Revalidation pins MC01 foreground and MC02 separate
  PID/HWND/session before an input-state write.
- A real source correctness issue in the optional in-process DLL
  previously returned failure for every game except MC02, potentially
  breaking unrelated profile launches. Non-MC02 attach is now a
  strict no-op. CMake rejects enabling duplicate GetAsyncKeyState/
  cursor detours. One-command look deltas cannot be lost when
  repeated with identical payloads in one GetTickCount tick.
- Fixed two concrete MC02 input defects without adding a provider:
  (1) an authenticated native IPC could finish after Codex/CLI gave up,
  then activate an old movement request for a fresh TTL; the MCP now
  stamps the request, the one-shot CLI checks wall-clock age, and the
  controller's sole shared-memory writer checks its own monotonic
  GetTickCount64 age immediately before publication. Any request older
  than 500ms or timestamped in the future refuses instead of moving
  late; each accepted packet still expires after at most 500ms.
  (2) before the first AI pulse, an unfocused MC02 polling Win32
  keyboard/cursor APIs inherited physical human input. MC02 now returns
  virtual neutral controls while background, allows manual foreground
  login/menu interaction before the first AI command, and latches
  exclusive virtual polling from the first valid command until exit.
  This isolation applies only to the implemented Win32 polling hooks;
  unimplemented Raw Input/GameInput handling remains UNVERIFIED.
- Windows Raw Input receives kernel HRAWINPUT via WM_INPUT, not
  arbitrary fabricated HWND messages; GameInput is a different
  reference-counted input stream. Neither is implemented by the
  Win32 polling candidate. Minecraft Education support documentation
  currently specifies keyboard and mouse on Windows, so adding an
  unproven virtual XInput gamepad is not a supported shortcut.
- No source test, Rust/CMake build, Windows action, Minecraft gameplay,
  driver, new loader, new controller, CI or private file write was
  executed. A single focused regression fixture was **AUTHORED,
  NOT_EXECUTED** for opt-in tool visibility, input-pulse encoding and
  MC01 focus refusal.

- Fixed a further first-owner input loss in the MC02 camera path:
  the original 40-byte single-slot packet held only the LAST pointer
  delta. If Codex sent two valid look commands before GetCursorPos was
  polled, the earlier camera movement disappeared. The private Rust
  controller and MC02-only DLL now share an ABI-v2 **72-byte** unnamed
  mapping: the packet still owns the latest held keys and expiry,
  while two atomic cumulative cursor axes preserve all movement within
  one live command epoch. A paired odd/even revision lets the DLL read
  a consistent snapshot; a bounded nonblocking in-process guard stops
  concurrent Win32 cursor reads from applying the same delta twice.
  Once the previous packet expires, the next writer begins a NEW
  epoch, discarding unseen expired camera movement. The source also
  checks ABI version/size on the already-authenticated bootstrap.
  This is not additional Win32 input coverage: Raw Input/GameInput
  remains unsupported and Minecraft game response UNVERIFIED.

## P0 source correction: experimental MC02 admission retains a live human host (31 October 2026)

- `LaunchMc02Experiment` previously checked `running[0].is_some()`
  BEFORE the original `launch_inner` reaped exited processes. A
  stale/exited MC01 could therefore authorize the creation of MC02's
  experimental input channel. The controller now reaps first and
  verifies MC01's original process birth/liveness and unique,
  responding visible game window BEFORE game settings or MC02 startup.
  Ordinary `Launch` remains unchanged. An existing controller test
  has one additional feature-gated assertion (AUTHORED_NOT_EXECUTED);
  no new suite was added.
- The native session transport now refuses an experimental MC02
  launch if the existing authenticated controller endpoint is absent.
  Previously only the input write was blocked, so an experimental
  launch could spawn a controller that has no retained MC01 host.
  Neither experimental operation may create a new controller.
- The normal `lazy_tester_session` Tauri command now rejects the
  experimental launch AND input-write variants at its boundary.
  This prevents the desktop's generic session command from becoming
  an alternative to the explicitly opted-in native agent CLI while
  preserving all ordinary profile/session operations.
- The changes affect only source under Cargo's `mc02-native-input`
  feature. No Windows tests, builds, Minecraft login, DLL hooks,
  CI or localtest was executed. Existing optional DLL/binary admission
  remains unresolved and gameplay input compatibility UNVERIFIED.

**P0 direct game-input correction — MC02 in-process focus-query virtualization
(31 October 2026).** Existing Win32 keyboard/cursor polling hooks could
remain ineffective if Minecraft Education first checks whether its own
window is foreground and stops polling in the background. The same
optional `bootstrap-compat` DLL now hooks the Win32 read-only queries
`GetForegroundWindow`, `GetActiveWindow`, and `GetFocus` on MC02,
using the already attached in-process Microsoft Detours transaction.
The retained Rust controller writes both already-verified MC01 and
MC02 HWNDs to the MC02-only shared section (private ABI v3, 88 bytes)
when and only when it publishes a bounded input packet. The child
checks that the REAL Windows foreground is STILL the pinned MC01
HWND and that the MC02 HWND is still its own visible, unowned OGLES
root. `GetActiveWindow` and `GetFocus` return MC02 only from its
own game-window thread. After the packet expires or the host loses
foreground, all fake focus reads return the real Windows results;
background key polling remains neutral after the first AI packet.
No OS focus mutation, `SendInput`, invented `HRAWINPUT`, virtual HID
driver, new loader, second controller, or synthetic message injection
is added. The regular build remains OFF by default.
**Evidence:** REMOTE_GITHUB exact source/ABI review only;
Rust/CMake/MSVC/Windows/Minecraft NOT_EXECUTED. This gives a
source-grounded way to handle background foreground-gating in the
existing Win32 polling candidate, NOT evidence that Minecraft
Education uses these APIs. Actual MC02 walking, relative camera,
attack and Raw Input/GameInput compatibility are still UNVERIFIED.

**P0 direct input-consumption correction — Win32 key/mouse press edges
(31 October 2026).** The existing MC02-only `GetAsyncKeyState` hook
returned the high bit (key currently held) but ALWAYS cleared the low
bit (key newly pressed). Microsoft documents that the low bit is
retained for press-since-last-call compatibility, although it is
unreliable as a general Windows input signal. A game polling that
bit for a short mouse attack/use or key action would see no press.

The optional Rust-controller shared-memory input ABI is now
**version 4, 96 bytes**. The existing sole writer adds a packed
8 x 8-bit counter for rising-edge transitions of WASD, jump, sneak,
attack and use. These counters change independently from the current
held-key packet and are reset upon command-epoch expiration. The
existing in-process MC02 DLL reads the counters under the same
atomic revision as the key packet and cursor totals. A bounded
nonblocking per-process read guard returns the low GetAsyncKeyState
bit once per new press, even if the writer published press and
release between reads, without returning stale clicks from an
expired epoch. The high held-key bit, AI cursor accumulation,
MC01 human-focus gate and ordinary client launches are unchanged.

This corrects the implemented **Win32 polling** API contract only.
It does NOT implement Windows Raw Input (`WM_INPUT`,
`GetRawInputData` / `GetRawInputBuffer`) or Microsoft GameInput
(`IGameInput`). No Minecraft Education API was invented from the
similarly named but distinct Bedrock/GDK editions. Gameplay acceptance
still requires a compatible real input-consumption path; neither
actual player movement nor attack has been observed. Evidence:
REMOTE_GITHUB exact source and Microsoft GetAsyncKeyState semantics;
all Node/Rust/CMake/MSVC/Windows/Minecraft execution NOT_EXECUTED.
Default app/DLL input features remain OFF. No new test suite added.

## MC02 buffered Raw Input consumer candidate (31 October 2026)

**Direct input source work:** the existing MC02-only optional bootstrap
DLL now contains a separately OFF-by-default
`BOOTSTRAP_MC02_RAW_BUFFER_POLL` mode. It requires
`BOOTSTRAP_MC02_VIRTUAL_INPUT=ON` and consumes the SAME
controller-owned, ABI-v4 (96-byte) MC02 input section. It does not
create a second game controller, DLL, process, driver, or OS-wide
input route. All other profiles' DLL behavior stays unchanged.

This mode hooks the real `GetRawInputBuffer` export in the existing
MC02 Detours transaction. **Only if** the game itself calls that
buffered-read API on its verified OGLES window thread, it produces
bounded arrays of `RAWINPUT` keyboard and relative mouse structures
from the shared key-held mask, rising-edge counters, and accumulated
mouse deltas. Mouse attack/use and keyboard press/release transitions
can be represented even if Codex emitted a short press-and-release
between buffered reads. The caller's size-query, insufficient-buffer
and counted-return conventions are implemented. A nonblocking guard
prevents duplicate application across concurrent reads. When MC02
is under virtual input, the original Raw Input buffer is drained and
suppressed to avoid forwarding physical MC01 events into MC02.
Expired commands do not produce late movement. When buffered mode
is selected, `GetCursorPos` stops consuming the same look deltas
so movement cannot be duplicated across two APIs.

**Explicit compatibility boundary:** unlike the standard
`GetRawInputData` path, this adapter NEVER synthesizes
`WM_INPUT` or a kernel `HRAWINPUT`. It does not register fake HID
devices, change the OS message queue, or implement GameInput.
Synthetic `RAWINPUTHEADER.hDevice` is null, not a real device ID.
Games that check device identity, wait for `QS_RAWINPUT` /
`WM_INPUT` before polling, or exclusively use
`GetRawInputData` / GameInput CANNOT be assumed compatible.
Minecraft Education's actual input consumption API remains
UNVERIFIED. This is a narrow possible buffered-reader route,
NOT a real-player movement or attack success claim.

**Proof ceiling:** REMOTE_GITHUB exact-source and Microsoft
`GetRawInputBuffer` / `RAWINPUT` contract review.
No tests were authored or executed; no MSVC/CMake/Rust/Windows
builds, Minecraft execution, localtest, package approval, CI or
gameplay proof occurred. Normal product input remains OFF.
Do not enable this candidate or admit a new DLL binary into the
signed/reviewed package without the separate existing operator gate.

## Next — P0 real independent AI gameplay input (operator reset, 31 October 2026)

**31 October P0 input-consumer thread classification (source-only):**
The existing, OFF-by-default MC02 input-API observer now distinguishes
successful non-null `GetRawInputBuffer` polls on the process-local
unique visible root OGLES window thread from polls on a different
thread. Both empty and nonempty successful polling calls qualify;
separate existing markers still distinguish nonempty real data.
No numeric PID, HWND, thread ID, account, key, or mouse data is logged.
Unknown/ambiguous OGLES ownership emits no thread classification.
This determines whether the current MC02 buffered candidate's
game-window-thread restriction is even applicable; it does not prove
Minecraft Education consumed synthetic data, moved a player, or
notified its input loop. No new hooks, SDK dependencies, controller,
runtime input permission, or product-mode changes. Evidence:
REMOTE_GITHUB exact-source review and Microsoft Raw Input thread-queue
contract; NO Node/Cargo/MSVC/Windows/Minecraft tests or execution.
The original gameplay-input blocker remains.

**31 October MC02 physical Raw Input suppression correction:** The existing
compile-OFF buffered Raw Input experiment no longer passes the game's
output buffer into the genuine OS `GetRawInputBuffer` call. Instead it
discards real keyboard/mouse events with a bounded, aligned, private
scratch array before emitting any MC02 virtual records. This avoids
placing MC01 physical input bytes in the MC02 caller's memory even
temporarily, and avoids incorrectly failing virtual delivery solely
because the caller sized its output for synthetic records. A queue
that cannot be drained within sixteen passes or contains an oversized
raw device record fails closed; no unfiltered physical data is returned.
The final virtual command and real MC01 foreground are revalidated
after suppression. This is a narrow correction to the existing
unbuilt, OFF-by-default candidate, not a new hook/provider, and
does not establish Minecraft input consumption or gameplay acceptance.
Proof: REMOTE_GITHUB exact source and Microsoft GetRawInputBuffer API
review only. MSVC, Rust, Windows, Minecraft, installer and source
test execution remain NOT_EXECUTED.

**31 October MC02 buffered-reader contract correction:** The existing
compile-OFF `GetRawInputBuffer` adapter now returns a bounded prefix of
whole `RAWINPUT` records when the caller's capacity fits fewer records
than the current batch. Its per-client held-key, rising-edge and relative
mouse cursors advance only for records actually copied. A subsequent poll
recomputes the undelivered suffix; an expired command cannot replay old
presses, and released buttons are derived from the last delivered state.
Only a buffer too small for ONE record returns
`ERROR_INSUFFICIENT_BUFFER`. This is the documented Win32 batched-read
behavior, NOT proof Minecraft Education calls the API or consumes the
virtual events. No extra detour, OS message forgery, production feature,
MCP tool, or gameplay execution was added. Exact-HEAD source review only;
Windows build and actual gameplay remain NOT_EXECUTED.

**31 October source correction — Codex/MC02 input feasibility:** The
already opt-in MC02 virtual polling hooks now emit one-shot debugger
markers for nonzero virtual keyboard, attack/use button and camera
output actually returned to a game caller (including buffered Raw
Input). These are **hook-return evidence only**, not proof of in-game
movement, PvP, world join or unaffected human input. The observer
separately distinguishes real nonempty Raw Input reads from mere API
calls. The source correction added no new hooks, input backend, controller,
agent daemon or production package admission. All execution remains
NOT_EXECUTED; the actual Minecraft input consumer is unresolved.

**Stop scope drift.** Phases 14–19 hardened secondary observation/evidence
and MCP response contracts without moving a Minecraft player. They are
NOT a substitute for gameplay input. Do not continue the implied
"Phase 20" privacy/window-response audit, create extra microtests,
or add more speculative source-only backend adapters.

**One next action:** establish an acceptable **real** per-client Windows
keyboard AND relative-mouse input mechanism for Minecraft Education,
beginning with MC01 (human foreground) and MC02 (AI unfocused). The
existing `background_input_probe.rs` posts one 80 ms W keydown/keyup
message to MC02 only behind a manual opt-in; it does NOT deliver relative
mouse, claim movement, or establish game acceptance. The MCP currently
has `provider:NONE` / `execution:DISABLED`.

**Direct P0 source work:** the reviewed bootstrap already injects
`bootstrap-compat.dll` into each managed Education process and uses
Microsoft Detours. Before adding any synthetic control hooks, the
separate test-only CMake build now has an OFF-by-default
`BOOTSTRAP_MC02_INPUT_API_OBSERVE` option. With an additional explicit
MC02-only environment opt-in it forwards the existing Win32 input APIs
unchanged and emits one debugger marker per API *called*. The same
OFF-by-default observer now also emits separate one-shot markers only
when `GetRawInputData(RID_INPUT)` returns nonempty bytes or
`GetRawInputBuffer` returns nonempty records. This prevents a size
query or incidental function entry being treated as a successful read.
Markers contain no keys, mouse coordinates or account data. A real raw
record can still originate from MC01's physical input and does not
establish MC02 gameplay control.
Ordinary builds contain no added input detours. This gives a narrowly
scoped means to distinguish API candidates in Minecraft without
inventing an input-provider contract. API calls alone do NOT prove
the game uses that API for movement, camera, or attacks; absence of
a marker does not exclude GameInput/DirectInput or later calls.
The optional mode is NOT BUILT, NOT EXECUTED, and not an agent input tool.
The next source correction adds an observer for successful
`RegisterRawInputDevices` registration (keyboard/mouse, background flags
and NOLEGACY) because raw-device registration is required by Windows
before a game can receive `WM_INPUT`. This differentiates an actual
Raw Input receiver from incidental calls to raw input APIs, without
changing the registered device list or sending artificial events.
Win32 `GetLastError` is preserved around the one-shot diagnostics.
These observations are hints, not proof of Minecraft movement or that
background input is independently routed. The diagnostic is still
test-only, compile-OFF by default, and requires the explicit MC02
environment opt-in. No real per-client AI input provider exists yet.

Upstream ProtoInput `protoloader.h` exposes selected physical device
routing and hooks but no general `sendSyntheticKey/mouseDelta` API.
Accordingly a ProtoInput wrapper alone cannot serve as AI control.
No process hooks may be enabled on a user's Windows machine without
a separate approved target-Windows/rights/compatibility decision.

The actual engineering decision is between: (a) independently evidenced
per-process game hooks/input interception (ProtoInput-style; unapproved
injection and game compatibility risks), and (b) genuinely independent
supported Windows interactive sessions (not implemented, currently
unsupported or outside approved local-only constraints). Global
`SendInput`, fake `WM_INPUT`, XInput global pads, and additional
semantic timelines do not solve per-process game input by themselves.
No permitted, verified complete keyboard/mouse mechanism has yet been
established. Do not write an actuator until a defensible mechanism is
selected. If the no-injection/single-session constraint cannot be met,
report the conflict explicitly rather than inventing a "working"
background controller.

**Success gate (NOT_EXECUTED):** two licensed Education clients remain
in one P2P world; MC01 receives uninterrupted real human control;
without foreground switching MC02 independently walks, rotates the
camera, attacks, stops and releases all controls; game-level evidence
attributes the effects to MC02 and not MC01. Only after this can
MC03–MC05 concurrency become in scope.

The operator is still working through ChatGPT + GitHub source.
Local tests, Windows drivers, builds, Minecraft gameplay, injections,
CI and external/cloud runners are NOT authorized or executed.
Existing native session ownership, MC01 protection and profile/world
preservation remain mandatory.

Historical work notes and prior Next proposals remain in immutable
Local Git history, not in a parallel current-status register.
