# Current Development Handoff — Lazy Tester

## Desktop runtime simplification — 26 November 2026

Work: `desktop-runtime-efficiency`. MCP and AI integration are explicitly out of scope.
Routine refresh now compares validated profile observations first and constructs layout identities only when a profile field changes or layout comparison is needed; unavailable connection still forces layout invalidation. Issue-only changes still update UI without invalidating geometry. Existing polling cadence, native admission, IPC, preview epochs and close recovery remain unchanged. Source review only; tests and native performance NOT_EXECUTED. Next source audit: desktop-specific redundant inspections and resource work; do not modify MCP or shared transport without preserving its consumers.

Explicit launch, navigation, settings, close reconciliation and storage receipts now share the existing full-field profile comparison before publishing UI state. Identical observations avoid reactive assignment; action-specific receipts, storage timestamps and recovery effects remain independent. No native request or MCP change; existing comparison fixtures NOT_EXECUTED.

Healthy idle Play now shares the existing 10s read-only inspection cadence; all five profiles must be STOPPED with no process/issue, no selected/pending launch, no recovery or active agent report. Live/uncertain Play and connection recovery retain 2s observation. Explicit actions and native validation remain unchanged; external changes during healthy idle can take up to the next idle inspection to appear. Cadence fixtures extended, NOT_EXECUTED; actual savings and UI acceptance remain unmeasured. MCP implementation/contract unchanged.

DWM preview admission now constructs the fallback tile report only on the no-source cleanup path. Live layout no longer builds and discards an unused status vector or clones its issue strings before dispatch; worker-generated statuses, cleanup epochs, native identity, deadlines and rollback remain unchanged. Existing preview status fixtures reviewed, NOT_EXECUTED. Foreground watcher and retained DWM worker lifetime remain unchanged pending native acceptance.

Desktop CloseOne now moves the owned successful SessionSnapshot directly into SelectedCloseReport instead of cloning it. Error mapping and mutual exclusion of snapshot/issue remain identical. No request count, wire schema, native lifecycle, recovery or MCP change. Existing selected-close receipt fixtures remain NOT_EXECUTED. Shared IPC admission and fresh native inspections are retained because their safety and consumer contracts are not redundant.

Profile size traversal and its existing regression fixture now live in `independent_bootstrap/profile_storage.rs`, alongside disk/cache observation. The session controller retains admission, shared two-second budget and report assembly; the storage module owns recursive metadata traversal and depth. No changed traversal limits, data operations, IPC or MCP contracts. Test relocated unchanged except owner/function naming, NOT_EXECUTED.

Window caption policy and its native fixture now live in `independent_bootstrap/window_caption.rs`; both session observation and legacy startup observation explicitly call that owner. Existing slot bounds, HWND/PID validation, timeout, caption repair and fail-closed behavior are unchanged. No native lifecycle or MCP contract change. Relocated tests NOT_EXECUTED.

Registered Education installation validation and its four existing preflight fixtures now live in `independent_bootstrap/registered_image.rs`. Bootstrap and controller call the same owner explicitly; registered target resolution, pinning, signature, digest, reviewed recipe and refusal behavior are unchanged. Developer source routing and implementation map updated. Tests NOT_EXECUTED; MCP unchanged.

Startup, explicit Refresh and automatic Refresh now use one start timestamp for the same Inspect/InspectSettings path. The healthy idle cadence no longer schedules an extra read immediately after a recent startup/manual read; live and disconnected recovery cadence stays unchanged. Updated stale repository cadence assertion and added source/cadence regression fixture, NOT_EXECUTED. A separate pre-existing discrepancy remains between the registered dormant coordinate-bookmark command and the repository allowlist assertion; no IPC permissions or bookmark behavior changed. MCP unchanged.

Reconciled the exact Tauri handler assertion with the prior retirement decision to retain the native bookmark command and storage. Registration, handler admission, data format and MCP remain unchanged; no new access granted. Repository assertion now matches the observed ordered allowlist, NOT_EXECUTED. Next: desktop startup and inspection cost review.

Settings reports now resolve and validate the settings directory once per report rather than three times, and perform one directory-creation call rather than two. Individual file validation, default values, read order, write admission and wire schema are unchanged. No cross-request cache or MCP implementation change. Regression fixture authored NOT_EXECUTED; runtime savings unmeasured. Next: workspace request flow and remaining startup costs.

## Workspace stability — source checkpoint (26 November 2026)

Work: `workspace-ui-lifecycle`. Surface: `CHATGPT_GITHUB`. The current track
prioritizes Play geometry, native modal recovery, foreground FPS and DWM.
Review-only resize clears old preview destinations before retained placement;
critical warnings share one bounded scroll region. Existing lifecycle, identity,
profile data and normal close ownership are unchanged. Preview blanking during
review is intentional; normal configuration resumes after Cancel or close.
Final integration correction: review-only geometry rechecks the captured bounds
after preview cleanup; Refresh remains present but disabled during selected
review; compact feedback overrides keep their intended CSS precedence.
Proof: source review only, all tests `NOT_EXECUTED`. Windows import/cancel,
one-click switching, FPS, DPI and DWM stacking remain `NOT_VERIFIED`.
Do not claim runtime acceptance or resume held tests without authorized PC-local
execution. Other historical F3/native-input holds below remain unchanged.


## Source simplification — 26 November 2026

Phase 1: retired superseded world-scoped F3 prototypes and dedicated review
routing; also retired six unmounted desktop coordinate research/fixture files.
Historical research remains in Git; native bookmark command/store, shared
process identity parser, native F3, controller, profile data and packaging
are preserved. Proof: source review only; tests and CI remain held.
Grouped six retained Windows test-only isolation primitives under
`engine/native_isolation_fixtures/`, with domain names and one compile gate.
No fixture behavior changed. Consolidated the repeated Windows process
creation-time read across bootstrap, identity, focus, IPC and workspace
into `engine/windows_process_time.rs`; existing admission checks remain.
Consolidated Windows executable attribute and file ID queries across bootstrap,
process roles and session IPC; caller admission and file pins remain unchanged.
Extended the same process-time reader to native observation, normal close
and descendant handoff; independent test oracles remain unchanged.
Replaced redundant process/snapshot handle wrappers in native observation and
workspace with Rust's standard OwnedHandle; DWM retains the original process
pin and explicit failure checks remain. The identity observer now uses the
same standard OwnedHandle for process and token handles; specialized Windows
resource cleanup is retained. Consolidated session response routing into one
ReplySnapshot policy (Full / Close / Receipt), preserving journal and label
behavior and the existing actor. Profile cache, full clear and game settings
now share fresh mutation admission; each settings request maps directly to
its existing game_preset operation. Settings observation and fresh admission
share one idle predicate; explicit preset methods replace the boolean mode,
and audio restoration has one path. Bootstrap and controller now share one
startup existence-lease implementation and one stable controller gate name;
profile preparation retains its separate gate. Both managed launch and
registered observation call the single suspended-process creation owner;
fixture conveniences are test-only and name their purpose. Historical startup
CLI dispatch and observation loop now have a separate named source owner;
product entrypoint naming reflects its real purpose. CLI flags and shared
journal/profile contracts remain unchanged. Next: audit remaining bootstrap
responsibilities and active IPC ownership. Shared request classification now
keeps agent reports and every experimental input operation out of desktop
admission and absent-controller spawn; this closes the previous incomplete
webview AI request exclusion. Transport validation is separated from the
original worker lifecycle; completed-worker cleanup and previous-reply drain
share one owner without replay or timeout cancellation. Next: audit IPC
contract module boundaries and frontend scheduling. The frontend queue now
uses a flat first-eligible priority list after the existing common gate;
all payloads and the independent parity fixture remain unchanged.
Selected-close failure paths now share one read-only reconciliation and one
presentation-uncertainty transition; no additional IPC or close is issued.
Storage snapshot publication now shares one profile/storage/timestamp update;
refresh and navigation share the current-observation predicate while preserving
caller-specific connection and recovery effects. Removed three unreferenced
mounted-component handlers and its unreachable launchers route alias; shared
bridge and scheduling contracts retained pending consumer audit. Next: audit
remaining frontend bridge and CSS consumers. Retired nextProfile and the
launchers scheduling alias after reviewing all 18 TS/Svelte sources and 25
frontend test files; active stopped-state and cadence coverage retained.
CSS audit found every lt-prefixed class textually referenced in mounted
components. Consolidated the unconditionally overridden filmstrip width and
gap into its original rule, preserving media queries. A retired popup-shadow
assertion was confirmed stale and replaced by the two active overlay-shadow
contracts; the unused token is removed. Visual runtime acceptance remains held.
Audio session ownership checks now narrow candidates by retained PID/birth
before fresh native identity/liveness verification; no cache or cadence change.
Routine session snapshots reuse the HWND returned by fresh native validation
instead of parsing its text three times. Malformed prefixes fail closed;
close callers retain the same native checks. Periodic frontend refresh now
avoids replacing unchanged profile observations, comparing every current wire
field including issue text; independent settings/activity updates and native
reads remain unchanged. Identical settings and Agent Activity reports now
avoid redundant reactive assignment; timestamp, stale status, receipts and
optional fields remain significant. DWM invalidation and layout IPC are
unchanged pending native acceptance. Next: review remaining resource paths.

## Continue development outside Codex

In any editor, open this checkout or clone:
`git clone --branch Local --single-branch https://github.com/OpenITStudios/M-Minecraft-Emulator.git`.
The PC and GitHub matched at `2ed588589bd5`.
Start with the [implementation map](../docs/system/implementation-map.md)
and [Windows setup guide](../docs/validation/local-pc-first-run.md).
Use `DEV.cmd`. Source tests remain held and CI disabled.

For approved native development, point `LAZY_TESTER_NATIVE_PAYLOAD_DIR` at
the absolute path of the reviewed DLL pair. The PC's `App/independent-bootstrap/`
pair matched the source-pinned hashes. On another PC, obtain the pair from
an authorized artifact source and install official Minecraft Education.
Keep `Data/` and `workspace/private/` backups on the PC.
See the [data layout](../docs/system/data-layout.md).

## Launch repair incident — 17 November 2026

Local build `a15dd9f5811dcc6a937890b3e90cc0673e99214d` matched its App/root
receipt. Root startup showed verified data and five enabled controls.
The manager's normal close stopped itself, both responsive managed Education
1.26.33 sessions and the controller without force. A repeat start showed five
enabled controls and was left open. Setup reported `configured=true, issue=null`.

App-only promotion preserved 29 protected metadata identities, including five
profile directories and hives. Prior App/root backups remain under
`workspace/private/local-app-backups`. No profile reset, migration, credential
copying, sign-in, world interaction or Computer Use occurred.

The PC retains an unsigned installer under
`workspace/packages/lazy-tester/0.1.0-a15dd9f5811d-20261117T075038267Z/`.
Its SHA-256 matched the local `SHA256SUMS.txt` at handoff
(`65b6cef20c83c8adad96cae379a5237c779d6a70f36e79e4e793b0837404c080`).
Signing, installer lifecycle, licensing, sign-in, worlds, networking and
five-client gameplay are unverified. Builds, tests, CI and native execution
for this handoff were `NOT_EXECUTED`.

Earlier `cea37a26` hung on a native lock. Only that manager was terminated,
after confirming no game was alive. Broader source-contract failures are
recorded below. The bounded launcher delivery is complete. STOP.

Earlier build/diagnosis evidence:
The cea37a26 build was packaged/staged and successfully
launched two retained managed Education 1.26.33 sessions concurrently. Both
reported RUNNING, a responsive HWND, and no attention issue. The original
controller's close-all then returned all five slots STOPPED, with no live
Minecraft process and no forced termination. This is startup/lifecycle
evidence, not sign-in, licensing, world or gameplay acceptance.

App-only staging backed up the previous App/root launcher. Before/after
metadata checks verified 29 protected objects, including all five profile
directories and all five existing hive file identities; Data metadata was
unchanged during replacement. No profile contents were copied/read by the
stager and no reset/migration/login/world action was issued.

Ordinary root startup exposed an additional UI scheduling defect: initial
native layout/preview work was eligible before initial session inspection,
so refresh could return early while resizing, producing zero enabled cards.
Queue regression tests first failed (two cases), then all 49 frontend tests
passed after gating layout/selection until initialization completes. Startup
diagnostic consumers also now use the current Profile data verified label,
not the obsolete Connected label. This startup correction is being rebuilt
and staged; its actual installed UI acceptance remains to verify.
The operator explicitly requests completion to a usable local build while
preserving old profiles/authentication, with no Computer Use or data reset.

A transparent native observer without a debugger reproduced one initially
owned GUID-shaped CreateMutexA call at RVA 0x2cf0f0: first process returned
0, the second returned ERROR_ALREADY_EXISTS (183) and exited normally with
code 0. The first process then accepted normal close and exited 0. Only
callsite/result metadata was recorded; no object names, login or world data.
The exact signed image digest is the 1.26.33 hash recorded below. The shared
bootstrap now selects and returns the caller together with its pinned image;
both launch owners use that selected caller, retaining the old image recipe.
All four preflight tests PASS, including actual current registered-image
validation that previously failed, and the scoped CLI argument test PASS.
The existing CLI now forwards close/close-all to the original normal-close
actor; no force kill or parallel game owner is introduced.

The build/stage/startup/normal-close step above is now complete for the
delivered a15dd9f5 product source; retained earlier notes below are historical.
No new release/main/CI publication is authorized.

Earlier controller stabilization (retained evidence):
At parent 353e0e7027fca4a959ccf4bec46b66bcd1672e3b, an unsupported image
prevented the entire retained controller from starting. The controller now
signature/identity-checks the installation, reports unsupported existing
profiles as ATTENTION, releases its idle image pin, and leaves inspection
and Settings accessible. Each Launch resolves and fully validates the current
signed image before game-settings writes or process creation. Original child
image pins survive for their full managed lifetimes. Fresh profile creation,
migration and experimental startup routes retain their existing review gates.
No unreviewed image or callsite is admitted; automatic compatibility with all
future game versions is NOT claimed.

Necessary build repairs also restore explicit nested Rust module paths,
missing window verification/error conversion/imports, an existing stale
process-image reader call, a recovery error return and test fixture field;
remove one extra CLI delimiter; and restore the existing frontend recovery
import and numeric type guard. These were compilation blockers in the pinned
source, not changes to game input, public schemas or provider selection.

Executed on the operator-authorized PC, without Computer Use or new Minecraft
launches: three Rust image-preflight tests PASS (one timing test ignored),
four pure actor/error-regression tests PASS, Windows product Cargo check PASS,
Svelte/TypeScript check PASS with nine existing warnings, frontend build PASS,
and the 49-test launcher/frontend suite PASS. The broader Vitest run is NOT
green: 168 PASS, 11 FAIL in nine files, including existing source-contract
expectations and import failures; do not claim complete regression acceptance.
Rust emitted existing warnings, including clashing GetSecurityInfo declarations.
Production package, App staging, native UI and repaired Minecraft launch/close
acceptance are NOT_EXECUTED. Installed App/root receipt remains unchanged.

Follow-up actual checks requested by the operator, at source
4e1f9cc08906f86fc62a69b2ab901e226decc7aa: installed a788a06 App binary
matched its build receipt, but its --lazy-tester-check-webview diagnostic
exited 2 with connection Unavailable, zero cards and zero enabled controls.
This hidden/build-context observation is NOT ordinary Explorer/Desktop
acceptance and does not identify the separate startup failure's root cause.
The ignored read-only registered_launch_validation_cost test was explicitly
run against the actual registered installation and FAILED at registered_image(true)
with Unsupported (current Education version lacks a reviewed recipe).
No timing result or successful Launch is claimed. No game was launched and
no Computer Use, login, world access or installed payload replacement occurred.

Current operator request: restore the installed Lazy Tester launch behavior.
The operator explicitly authorized local compatibility review, builds and
launch/normal-close tests without sign-in or world access, then prohibited
Computer Use because the PC is in use. No further UI/focus/input automation
or foreground game experiments are in scope now. The F3/native-input work
below is separate and is not resumed by this repair request.

Initial diagnosis (before the source changes above): source was fast-forwarded from
aad08f54dce59836b8543718c0c9bbbeea7ce08b to GitHub Local
f2d9b8ca62c22c6284b228f2c09ec5df28c9a724 without replacing App or Data.
Installed App/build-info.json still identifies a788a06. No installed payload,
profile, world, or game installation was changed by that initial observation.

Observed installed Education: file version 1.26.33.0, valid Microsoft
Authenticode signature, SHA-256
59fac48cfb6b2cf8d0821e9a0ae6d586481ab1e73ceef46cea53ecee3ac48b43.
The screenshot's exact refusal originates in independent_bootstrap.rs:
validate_registered_image_with requires
04164c1c64458a5d984dbc1687d0e8023d1808c44ee7be03519cdf8355233eaa.
Both prepare_registered_mode and session_controller::Sessions::launch_inner
use the previously reviewed mutex caller RVA 0x2cdc30. New-image caller
compatibility is UNKNOWN; changing the accepted digest alone is unjustified.

Read-only PE inspection found a changed section/import layout. One temporary
local startup observer using DEBUG_ONLY_THIS_PROCESS reached a Themida
debugger-rejection dialog before producing any mutex-call evidence. The
observer detached successfully with kill-on-debugger-exit disabled; at the
last observation its game process/dialog remained alive. This is not a
normal-startup failure or proof that the ordinary bootstrap DLL is rejected.
No protection bypass, forced process termination, login or world operation
was performed. Do not reuse historical PIDs or repeat this debugger test.
The operator's no-Computer-Use instruction remains active.

The initial new-image review prerequisite above has now been met by the
non-debugger observation. Actual managed-launch and installed acceptance
still require the current build/stage/verification step. All unrelated
historical holds remain unchanged.

**Updated:** 8 November 2026. **Branch:** `Local` only; `main` is
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

## Current operator priority: client-native F3 (8 November 2026)

**Deliverable:** a single F3 key toggles an actual in-game
Minecraft Education Windows debug HUD, with live game-local XYZ,
chunk/facing/dimension and an explicit in-game Copy Coordinates
action. No world pack, global Resource Pack, Script API player
event, modal form, Windows layered overlay or LazyTester
application UI. The superseded world/Script API and JSON UI prototypes
are retired from active source; see the [retirement register](../docs/system/legacy-retirement.md).

**Owner/evidence:** the exact installed Education 26.33 game-native
input, Player/HUD and render ABI must be verified before any
client hook or production integration. Existing source now also contains an OFF-by-default D3D11
in-backbuffer F3 compositor and input/Player-frame source contracts,
but it still does NOT expose a verified Minecraft Education Player
or keyboard ABI. None of these source-only components is active.
Do not turn a layered FPS window into a counterfeit Minecraft F3
panel. No client F3 implementation has been built or tested.
Read [native client F3 design](../docs/system/minecraft-education-native-f3-client.md).
The prior Codex background CUA track remains unchanged but is not
the requested F3 solution.

## Cross-chat continuation — one canonical handoff

- **Repository/branch:** `OpenITStudios/M-Minecraft-Emulator` /
  `Local` only. Pin **actual current** HEAD, not a remembered SHA.
- **Current priority:** Minecraft Education F3 **client-native** overlay;
  F3-only input, player-sourced runtime data and actual in-game renderer.
  The required exact Education 26.33 input/Player/HUD ABI is unavailable;
  no credible mod integration can be committed yet. See the F3 design.
- **Separate previous track:** Codex Windows Computer Use, strict
  zero-disturbance background via existing Cua Driver; no input
  implementation or proof is promoted by the F3 request.
- **Prior native source stages:** P58-P64 Cua UIA read-only / upstream Invoke
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
  Node/Rust fixtures AUTHORED_NOT_EXECUTED. Source acceptance review
  found one remaining C++ read-path mismatch: virtual foreground/active/
  focus hooks could return stale spoofed focus after ReleaseAll or
  a changed native publication during their owned-window lookup. Their
  final return now reuses the existing same-live-publication gate,
  without new ownership, ABI or enablement. C++ tests NOT_EXECUTED.
  This continuation closes one MCP source mismatch: the exact model-
  reviewed PNG basis is now checked AFTER the asynchronous original
  4/5-AI writer lease ACK and immediately before the native batch.
  Changed/expired images abort before dispatch and release neutral
  original leases. The observed receipt now measures transport start
  at native batch handoff rather than pre-lease admission.
  Node regression fixture AUTHORED_NOT_EXECUTED. A follow-up CLI/MCP
  completion audit found a post-ACK cancellation race: the original
  batch could be acknowledged, then Stop/shutdown could arrive during
  asynchronous per-slot lease release, while MCP still returned a
  successful action receipt. The original MCP now rechecks cancellation
  after every lease release attempt, keeps the selected cohort in
  explicit-recovery quarantine, and never replays it. Four/five-AI
  late Stop/shutdown fixtures AUTHORED_NOT_EXECUTED. The next native
  partial-group review found a demonstrable final-write deadline gap:
  the actor checked 150-ms lateness BEFORE potentially slow HWND/root
  verification and memory mapping; Channel::write only checked 500-ms
  request age and could publish after the group's deadline. The same
  actor deadline is now passed internally into the existing Rust writer
  and rechecked immediately before v4 publication. Direct single-slot
  input and ReleaseAll remain unchanged; the CLI keeps explicit
  AI_SEQUENCE_DEADLINE only for a genuine actor reply, not lost IPC.
  Rust negative fixtures AUTHORED_NOT_EXECUTED.

- **Evidence:** official and open-source macOS/Windows sources in
  [Windows parity research](../docs/system/windows-background-computer-use.md);
  upstream Cua Driver source pinned there. No REA binary session, no
  Codex/Minecraft/Windows runtime proof.
- **Continue:** read this file's **Single next step** and the named
  source owners before changing code. Git commits preserve precise
  history; never create another active roadmap/agent/controller.
- **Permission boundary:** GitHub source-only; no localtest, build,
  CI, installation, launch, game action, personal data or `main` write.

## Non-primary architecture navigation adaptation — 8 November 2026

The existing repository documentation router remains canonical. A bounded
source-only navigation layer adapts M-Lazy-Developer's stable Resource
Catalog, provenance-bearing authored Graph, scoped Retrieval, derived
sections and Obsidian/Mermaid/Pele views to the Emulator's legacy document
metadata. It adds no separate agent/runtime manager, code-dependency proof,
execution, CI, status owner or product change. The primary Windows
Computer Use continuation and its single next action remain unchanged.
See [knowledge navigation](../docs/system/repository-knowledge-graph.md).
Optional Node fixtures are AUTHORED_NOT_EXECUTED. The same bounded
architecture work now adds an opt-in, file-level
[source dependency analysis](../docs/system/source-dependency-graph.md):
TypeScript/Svelte declared parsers, lexical Rust module/use references,
source-site witnesses and conservative import impact/trace. Unresolved
parser availability and cfg/feature conditions are never promoted to
proof. The latest source-only follow-up corrected Rust source-site provenance,
added 28 existing frontend test files to indexed coverage, conservative
literal `new URL(..., import.meta.url)` test-source references, C/C++
local headers, CMake literal target source lists, and a separate exact-name
Tauri bridge/registration/handler witness projection. All impact/trace
queries retain `NOT_PROVEN_COMPLETE`, including global gaps and unobserved
reverse importers; the developer controller and Windows source tests have
**not** executed this graph. Fixture tests are AUTHORED_NOT_EXECUTED.
PowerShell calls, full CMake/CFG resolution, semantic Rust/TS call paths and
runtime behavior remain outside this milestone. No Windows Computer Use,
native controller, product CI, account/world data, or current primary next
action changed.

The bounded architecture improvement now includes
[Architecture Reasoning and Decision Provenance](../docs/system/architecture-reasoning.md):
one read-only explanation composed of the existing implementation-map
owner references, canonical invariant text, conservative Source Graph
impact and optional direct Git commit history by exact Work trailer or
source path. Git author decisions remain historical claims; dirty or
unavailable Git, missing owners, incomplete imports and unexecuted
checks stay UNKNOWN. No ADR database, memory store, agent, CI, native
runtime/installer, credentials, account/world migration or automatic
approval was added. Fixtures are AUTHORED_NOT_EXECUTED. The existing
primary Windows background Computer Use goal and single current next
action remain unchanged.

## Non-primary Matt Pocock agent-method adaptation — 8 November 2026

The single existing `emulator-development` Skill now conditionally routes
bug reproduction discipline, deep-module interface reasoning, two-axis
Standards/Spec code review, domain language, primary-source research and
low-context agent instructions. The source of truth is
[Development Discipline](../docs/system/development-discipline.md#matt-pocock-engineering-method-adapted)
with authored [routing scenarios](../.agents/evals/matt-pocock-method-scenarios.md).
This is a source-policy change based on pinned mattpocock/skills and a
comparison with M-Lazy-Developer/Local; no third-party plugin, additional
Skill, issue tracker, CI, game controller, local-source test, Windows build
or Minecraft execution was authorized or performed. Static routing-fixture
tests are AUTHORED_NOT_EXECUTED; actual agent-effectiveness improvement is
NOT_MEASURED. The primary Windows background Computer Use track and its
single next action remain unchanged.

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
  and unverified in Education 26.33. The dormant native Saved Locations
  store retains existing XYZ data; its desktop UI has been removed.
  Automatic F3/OCR, live world Save/Load, and sealed Testing CAS transfer
  remain blocked.
- **Developer harness:** one existing `DEV.cmd`/developer controller.
  Harness² Propose–Probe–Compose, DeepSeek scoped roles and REA are
  **developer-side reference patterns**, not installed agent runtimes or
  replacement Minecraft controllers.

## Recent source outcomes

| Revision | Source-only change | State |
| --- | --- | --- |
| F3 same-world frame supersession correction (8 November) | Source audit found the previous two-lock D3D11 Present path validated only world generation at its final check. An updated Player snapshot in the same world could make an older copied XYZ draw after D2D preparation, and an old graphics failure could hide that newer panel. Final draw admission and conditional failure cleanup now additionally match the exact engine capture timestamp and recheck freshness, leaving newer Player state untouched. Microsoft documentation confirms the current `ID3D11Device1` emulated-interface argument for `CreateDeviceContextState` is valid; it is unchanged. | REMOTE_GITHUB source and API documentation inspection only; no tests, builds, Windows, DLL or Minecraft execution. F3 engine keyboard/Player interface not wired and source option remains OFF; actual GPU timing/parity NOT_VERIFIED |
| Native F3 separate input and GPU locks (8 November) | Removed a confirmed source-level responsiveness issue: D2D device creation, Draw/EndDraw and FPS sampling held the exclusive Player/keyboard SRW lock, delaying F3 and `publish_player`. The same original F3 session now uses one independent GPU-resource SRW lock with graphics-before-state ordering; Present snapshots/rechecks Player identity and releases the state lock before COM/GPU work. Paired before/after mode-change gates across all four existing DXGI switch methods prevent drawing or F3 input during native resize/fullscreen. No new game controller, engine hooks, world/App changes, input ABI, or default activation. | REMOTE_GITHUB static source/consumer review only; user-prohibited tests/build/CI/DLL loading/Minecraft NOT_EXECUTED. Native Education keyboard/Player/render compatibility NOT_VERIFIED; source F3 remains OFF |
| F3 hidden-backbuffer cleanup and DXGI mode transitions (8 November) | Corrected a source-level retention gap in the opt-in C++ in-game F3 candidate: a hidden panel kept Direct2D/backbuffer COM references until resize/retirement. The next native Present now releases successful cached F3 graphics once, preserving a zero-graphics hot path on later hidden frames and preserving sticky denial of failed renderers. Existing DXGI wrapper now also releases F3 resources before SetFullscreenState and ResizeTarget (as it already did for ResizeBuffers/ResizeBuffers1). Panel Block/Chunk formatting and clipboard both use integer floor, handling negative fractions and -0 consistently. No world/App changes or new hooks. | REMOTE_GITHUB source/consumer inspection ONLY, no tests/builds/CI/Windows/Minecraft as instructed; F3 source flag OFF and Education keyboard/Player/render acceptance NOT_VERIFIED |
| Native F3 clipboard lock separation and swapchain refusal (8 November) | Fixed source-level F3+C clipboard reentrancy risk: previously Open/Empty/SetClipboardData ran under the exclusive shared F3 state lock. Now capture exact Player frame under the lock, release it, recheck matching world/time/foreground/HWND, then allocate/perform the OS clipboard operation without holding the render lock. Memory ownership is transferred only on successful SetClipboardData. Backbuffer index/GetBuffer/CreateBitmap/EndDraw failures now release GPU targets and refuse repeat draw for the current chain until Resize/retire. | REMOTE_GITHUB source and immediate consumer review ONLY; tests/build/CI/Windows/Minecraft explicitly NOT_EXECUTED. F3 physical key and Player engine callbacks remain unwired and source feature OFF; clipboard/runtime rendering NOT_VERIFIED |
| Native F3 lazy render, ordinary-key admission and D3D11 state restore (8 November) | Corrected three source-identified issues in the OFF-by-default C++ F3 candidate: hidden panels had driven per-Present COM/Direct2D/FPS work, system-key Alt+F3 was treated as F3, and Direct2D drawing did not preserve the original D3D11 immediate pipeline. Hidden Present now exits before F3 graphics/FPS; Win32 normal F3 down/up is required with repeat suppression; Microsoft D3D11.1 context-state save/restore brackets Draw/EndDraw and incompatible devices fail closed. Removed the now-unused rendererReady state. No input/Game Player adapter, product activation, world changes, or separate UI. | REMOTE_GITHUB pinned source and immediate header/DXGI/CMake consumer inspection; no testing, build, CI, DLL execution or Minecraft (operator explicitly withheld permission). Exact Education engine ABI, D3D11.1 state restore and actual F3 input remain NOT_VERIFIED |
| Native F3 true capture freshness and D3D11 admission (8 November) | Closed two source-proven defects in the default-OFF C++ F3 candidate: pending Player frame delivery previously reset its freshness to callback receipt time, and old world generations could overwrite new world data. Now requires original engine-captured `capturedTickMs`, monotonic captures/generations and rejects expired/rollback frames. Renderer now requires BGRA-compatible existing D3D11 device and caches unsupported-device refusal until the original Resize/retirement. No new hook, game Player source, world, desktop or product activation. | REMOTE_GITHUB code/consumer review; tests/build/CI and Windows/Minecraft deliberately NOT_EXECUTED per operator. Engine input/Player ABI and actual F3/D3D11 use remain NOT_VERIFIED |
| Native client F3 input/render source slice (8 November) | Introduced `education-client-f3.h/.cpp` under the original test-only bootstrap folder. It has OGLES/window-thread gated F3 key-edge+F3+C, same-client live Player-frame validation/world-epoch freshness, a real Direct2D/D3D11 in-swapchain draw path and swapchain resize/retire cleanup. Linked only under new `BOOTSTRAP_EDUCATION_CLIENT_F3_SOURCE=OFF` CMake flag and existing `dxgi-pacing.cpp` wrapper. No new game keyboard hook/Player ABI provider, no layered overlay, no world/UI/installer changes; gameplay source and D3D12/HDR support remain unavailable. | REMOTE_GITHUB source and immediate native consumer review; zero tests/builds executed at operator instruction. Client ABI, compile, graphics state preservation, F3 toggle/clipboard and Minecraft acceptance NOT_VERIFIED |
| Native Education F3 read-only PE evidence (8 November) | Added a developer-only, explicit-call PE metadata collector that reuses the registered Microsoft-signed Desktop resolver, validates bounded x64 PE structure and optional CodeView reference, computes SHA-256 and excludes install/PDB paths, account and world data. Always reports F3 input/Player/HUD ABI NOT_ESTABLISHED; no client hook, F3 renderer, installer, profile or runtime activation. | REMOTE_GITHUB source review; PowerShell NOT_EXECUTED, client build ABI, live F3 and Minecraft NOT_VERIFIED |
| Native-only Education client F3 routing correction (8 November) | Operator explicitly rejected world-based and app-based F3. Audited native `bootstrap-compat`, DXGI Present and measured FPS HUD: current HUD uses a separate layered Win32 window and does not expose a native Player/XYZ ABI. Pinned Git tree contains no game EXE/DLL to inspect. Added one authoritative client-native F3 design/validation gate; retired existing Script API/JSON UI experiments as historical F3 candidates, preserving sources. No speculative hook, binary patch, dummy output or client mod installed. | REMOTE_GITHUB code/source and official Education/Bedrock reference review; no native game ABI, real F3 input, HUD renderer or copy action verified; Windows/Minecraft NOT_EXECUTED |
| In-game F3 live debug-data refresh projection (8 November) | Closed a source gap: the earlier pure toggle supplied only one position snapshot and would become stale as the player moved. Added a pure visible-panel update/unchanged/hide projection for player location, facing, rotation and dimension; an invalid Player hides rather than leaving stale text. The existing team `coords/display` remains the sole scheduler/HUD owner. Authored stationary/moving/dimension/invalid/five-player fixtures; no new key hook, macro, render, app UI, clipboard or world mutation. | REMOTE_GITHUB static source/consumer inspection; Node tests AUTHORED_NOT_EXECUTED; actual F3-only input, Education HUD updates, team integration and game acceptance NOT_VERIFIED |
| In-game player-origin F3 toggle adapter (8 November) | Added a pure SourceEntity/ScriptEvent admission-and-intent adapter for the already-authoritative team coords/display module. Requires exact team-chosen event ID, player Entity source and `toggle` message; returns same-player show/hide intent and verified-in-code position data, with negative source fixtures. No new Script API listener, macro registration, player state store, HUD writer, game injection or LazyTester app UI. Bedrock Alt+key command macros and /scriptevent are evidence-backed candidates, NOT exact F3. | REMOTE_GITHUB source/consumer review; Node tests AUTHORED_NOT_EXECUTED, Education F3 keyboard/macro/player attribution, team integration, HUD and clipboard NOT_VERIFIED |
| In-game F3 debug groundwork (8 November) | Confirmed Education lacks Java F3 and the documented Script API InputButton exposes Jump/Sneak, not raw F3. Added pure player-sourced XYZ/block/chunk/facing/dimension formatter and authored negative unit fixtures, explicitly without keyboard listener, HUD writer, clipboard, app UI, installed add-on or new team module. Team GitHub source not available; no integration or fake F3 toggle. | REMOTE_GITHUB static source/research; fixtures AUTHORED_NOT_EXECUTED; actual F3 input, Education render, Copy Coordinates and gameplay NOT_VERIFIED |
| FPS follows verified Play selection (8 November) | The existing native foreground event worker now considers the already-approved in-app game window ONLY when the original manager itself is Windows foreground and a live native-aperture readback proves the selected managed PID/HWND/geometry. Existing Windows-foreground game still wins; another application, minimized/covered/suspended window, changed identity or uncertain helper demotes. Original workspace entrypoints request a post-operation refresh on the same worker (including when a private policy reuses an already-running public observer), which signals existing kernel changed events; no additional process, pacing producer, timer, C++ ABI, IPC command or saved-settings mutation. Authored negative Rust fixtures for five slots and fail-closed selection. | REMOTE_GITHUB source and direct-owner review, Rust fixtures AUTHORED_NOT_EXECUTED; actual priority timing, DXGI Present, Windows/WebView2 and five Minecraft clients NOT_VERIFIED |
| Navigation/Refresh close-review epoch consistency (8 November) | Reused existing observationStillCurrent/observationEpoch in asynchronous Play↔Settings↔Storage navigation and manual Refresh. A native close review invalidates late suspend, Settings, DataLocation and Storage continuations before UI state is changed; late errors are not shown as current. An undispatched Refresh intent is canceled on real page navigation. Authored synthetic delayed-suspend/settings/storage and source regression cases; no controller/IPC/data operation changed. | REMOTE_GITHUB pinned source and consumer inspection; fixtures AUTHORED_NOT_EXECUTED; Windows/WebView2/DWM/Minecraft NOT_VERIFIED |
| Refresh/reconnect UI confidence and source routing alignment (8 November) | Fixed manual Data Storage Refresh when binding verification still fails: DataLocation may be inspected for recovery, but cached storage and success receipt are cleared, the existing binding error is surfaced, and no InspectStorage/game operation is issued without a verified binding. Play header now distinguishes verified binding from unavailable session inspection, and never shows ready-to-select wording while initializing. Retired General folder/navigation test expectations and current launcher/refresh guards were aligned. Browser and static negative/positive fixtures authored. | REMOTE_GITHUB exact-source/consumer review; source/browser tests AUTHORED_NOT_EXECUTED; Windows/WebView2/host/Minecraft NOT_VERIFIED; native controller/Data untouched |
| Data Storage per-launcher feedback and review preservation (8 November) | Parent chooser now refuses profile switches during a queued graphics change, native CloseOne review, Storage confirmation or in-flight mutation, instead of silently clearing the review. A safe selection clears previous slot's Settings/Storage success notice without invalidating native state or the measured storage report. Current UX acceptance no longer requests removed desktop F3/Coordinates screenshots. Authored static and synthetic-browser regression guards. | REMOTE_GITHUB source and immediate consumer review; fixtures AUTHORED_NOT_EXECUTED; Windows/WebView2, native controller, data/world and Minecraft NOT_VERIFIED |
| Dev root guard when App already exists (7 November) | Confirmed a bypass in the shared manager/profile application_root resolver: staged App/ previously short-circuited the check for old build-local Data. The official dev target is now checked first and refuses any old Data beside the compiled executable even after App staging. Retains installed App/../Data and nonproject fallback, no migration, deletion or account/world content read. Added both-state Rust and source assertion fixtures | REMOTE_GITHUB source and consumer review; Rust/Vitest/Windows NOT_EXECUTED; no gameplay or installer acceptance |
| Pre-localtest interaction admission (7 November) | Removed a source-proven CloseOne bypass: pending confirmation no longer disappears when users select/launch another card; guard source prevents overlapping CloseOne and queued actions. Storage review locks competing buttons, Refresh and Advanced collapse until explicit Cancel; audio checkbox reconciles after native receipt/refusal. Added source and synthetic UI regression assertions, updated stale source assertions without changing native session authority | REMOTE_GITHUB SOURCE_PREPARED; Vitest/Playwright/Windows/WebView2/Minecraft NOT_EXECUTED |
| One-checkout native root and storage continuity (7 November) | Corrects manager_settings::application_root for approved workspace/Tauri dev targets BEFORE App is staged; profile owner and manager cache share this resolver. Older build-local Data is detected and preserved with a fail-closed error, no automatic migration; installed App/../Data and legacy fallback unchanged. Updated canonical layout, workspace/root docs and read-only localtest procedure with root/UX verification | REMOTE_GITHUB SOURCE_PREPARED; Rust/TypeScript regression fixtures AUTHORED_NOT_EXECUTED, Windows filesystem/installer/gameplay NOT_VERIFIED |
| Retired desktop F3/Coordinates validation drift (7 November) | Aligned README, Windows first-run guide, standard-case inventory and `DEV.cmd plan` LT-13 with the removed F3 Inspector / Coordinates & Saves UI. Retained stable case IDs; old STD-SET-12..16 cases explicitly NOT_APPLICABLE, LT-13 now checks actual General/Data Storage navigation. Authored a bounded plan regression fixture; no game, data, installer or native controller change | REMOTE_GITHUB source and consumer review only; Node/Vitest/Windows/Minecraft NOT_EXECUTED |
| Approved LazyTester logo and one-word UI brand (7 November) | Replaced canonical app-icon.svg with a real vector trace of the approved blue-white L/window + check + five tiles design. Existing WebView header/titlebar/empty workspace reuse the SVG. Tauri prepare:icons now fingerprints SVG content to prevent stale Windows ICO/PNG outputs. Aligned visible LazyTester wordmark and window title only; installed productName, executable, bundle identity, NSIS paths, native session and profile data untouched. Generated downloadable SVG/PNG/ICO pack; source and packaging fixtures authored | REMOTE_GITHUB SOURCE_PREPARED, SVG/PNG/ICO renders generated in container and inspected; Tauri CLI/Windows icon embedding, WebView2 and installed update runtime NOT_EXECUTED |
| Concise and clear UI copy (7 November) | Simplified global error explanations, per-launcher health guidance, General help, Storage overview/Advanced recovery copy and successful operation receipts; set 80-character authored-feedback budget. Kept unconfirmed-status, next-start, backup replacement, world deletion, Windows sign-in and native confirmation semantics explicit. Reconciled source and synthetic browser fixtures; no UI controls, session owners or file data changed | REMOTE_GITHUB SOURCE_PREPARED; source fixtures AUTHORED_NOT_EXECUTED, Windows/WebView2/DWM and Minecraft NOT_EXECUTED |
| Final error feedback and familiar Settings labels (7 November) | Structured authored error headings, safe same-handler Refresh status action, collapsed Error codes; bounded sticky error banner in Settings/Storage only. Marked Preserve Visuals as recommended, explained legacy High/Low equivalence and FPS caps versus measured values, renamed Locate/Advanced recovery. Corrected stale source fixture expecting “Profiles verified.” No new native operation, observer, dashboard or destructive action | REMOTE_GITHUB SOURCE_PREPARED, fixture assertions AUTHORED_NOT_EXECUTED; Windows/WebView2/DWM, assistive-tech, Minecraft and actual native error recovery NOT_VERIFIED |
| Settings and Data Storage consistency (7 November) | One coherent General/Data Storage contract: per-launcher Graphics, global FPS, immediate audio, unified Save/Discard copy, no unsaved FPS quick preset, Data Storage as sole folder owner and sidebar destination, no duplicate General shortcut, MC01–MC05 display identities, human-readable binary storage sizes with partial/unmeasured/zero clarity, truthful UI report receipt timestamp, Locate before final full-clear danger zone. Original file/native deletion guards untouched; pure formatter and browser/source fixtures authored | REMOTE_GITHUB SOURCE_PREPARED; Vitest/Playwright/Svelte/Windows/WebView2/DWM/Minecraft and destructive profile acceptance NOT_EXECUTED; no data mutation |
| Remove unrequested desktop F3/Coordinates (7 November) | Operator explicitly rejected both as app features. Remove F3 Inspector, Coordinates & Saves and obsolete Tools UI; preserve Refresh and contextual Separate window; retire UI-specific popup components/helpers/tests, update active product docs. Keep pure coordinate research and legacy Saved Locations native store/user data untouched; do not create in-game features implicitly | REMOTE_GITHUB SOURCE_PREPARED; authored source/browser regression contracts NOT_EXECUTED; Windows/DWM/Minecraft NOT_VERIFIED |
| Play toolbar disclosure (7 November) | Reduced always-visible Play actions to Refresh + Tools; placed original Separate window, F3 Inspector and Coordinates in one nonmodal focus-aware menu. Native session/window action guards, five keyed DWM previews, viewport geometry unchanged. Browser/source regression fixtures authored | REMOTE_GITHUB SOURCE_PREPARED; Svelte/Vitest/Playwright, native WebView2 focus, Windows/DWM and Minecraft NOT_EXECUTED |
| Play label/error consistency (7 November) | Standardized small five-slot launcher labels to MC01–MC05 while keeping full accessibility/action/dialog names, bounded Play-only top-level warnings with keyboard scroll, and removed a duplicate earlier 12px preview-health CSS declaration. Added source/browser geometry/long-message fixtures; native DWM owners and session admission unchanged | REMOTE_GITHUB SOURCE_PREPARED; regression fixtures AUTHORED_NOT_EXECUTED; Windows/WebView2/DWM/high DPI and Minecraft NOT_VERIFIED |
| Single adaptive workspace (7 November) | Removed redundant Gameplay Focus/Monitoring choice and saved mode state; one height-responsive 16:9 preview budget balances the large native game viewport and five fixed cards. Preserves original CloseOne, Launch/Select, Graphics/FPS, coordinate tools and DWM ownership. UI/source/browser contracts and UX docs reconciled | REMOTE_GITHUB SOURCE_PREPARED; source/browser fixtures AUTHORED_NOT_EXECUTED; native Windows/WebView2/DWM, DPI and Minecraft usability NOT_VERIFIED |
| Workspace Layout selector (7 November) | Historical two-mode UI experiment is **SUPERSEDED** by the single adaptive workspace below; the selector and its localStorage reader/writer are retired without deleting existing user preferences | SUPERSEDED at REMOTE_GITHUB source ceiling; no runtime acceptance |
| Compact five-slot Play foundation (7 November) | Original main-game-first spacing, Close title action, native 16:9 viewport and five fixed DWM preview nodes retained as the basis of one adaptive layout | REMOTE_GITHUB SOURCE_PREPARED; authored source/browser fixtures NOT_EXECUTED, Windows/DWM/Minecraft NOT_VERIFIED |
| Minimal Professional empty-viewport status (7 November) | Fixed potential clipping of no-game/detached Play messages by making only the conditional empty-status content keyboard-focusable and safely scrollable; kept 16:9 native viewport and five keyed DWM cards unchanged. Updated visual source and optional synthetic browser fixtures, with acceptance criteria in `docs/product/visual-identity-and-ui-ux.md` | REMOTE_GITHUB source preparation; browser/Svelte, Windows DPI/Narrator, native DWM and Minecraft NOT_EXECUTED; unrelated CUA continuation remains unchanged |
| Minimal Professional UI readability (7 November) | Increased dense Settings/Storage and scrollable Play disclosure text to 13px, and launcher recovery target to 28px while offsetting the previous 4px margin. Preserved five fixed native preview slots, original header control geometry, controller admission and 16:9 viewport. Source/browser fixtures and UX contract updated; see `docs/product/visual-identity-and-ui-ux.md` | REMOTE_GITHUB source preparation; all source test execution, Windows/WebView2/DWM, DPI/Narrator and Minecraft NOT_EXECUTED; existing Codex CUA single-next-step remains unchanged |
| Final native actor/CLI/MCP no-write integrity review | The last source audit confirmed exact busy/mode/stale writer lease statuses from original actor-compatible errors and conservative uncertain-IPC recovery. It found the legacy fence rollback missed exact InvalidData window/foreground/mode refusals returned BEFORE Channel::write. These are now allowlisted alongside proven original Channel pre-v4 rejects; generic Win32/Unmap-after-write/transport errors remain quarantined. Prior pending state is preserved | Expanded native Rust error-kind fixtures AUTHORED_NOT_EXECUTED; no new owner, status code, IPC/game ABI or feature enablement |
| Original legacy pre-publication fence rollback | The retained actor restores the exact previous legacy 500-ms/pending state only when its original Channel returns a known InvalidData refusal BEFORE v4 packet/cursor publication. An old unresolved pulse remains fenced, and unknown/Unmap-after-publish/transport errors never roll back. A definitive stale/wrong-publisher native refusal therefore does not unnecessarily quarantine a clean MC02 for all future 4/5-AI acquisitions | Pure four/five-cohort AiLeaseBook and actor error-kind negative fixtures AUTHORED_NOT_EXECUTED; no IPC/ABI/second controller change |
| Expired MC02 batch native neutral recovery | The original Channel now compares the batch's original issued_at_ms and exact authenticated last-publisher PID/birth before deciding neutral admission. If 350ms pulse TTL and 500ms request age expire but the batch is STILL the last publisher, it may publish an exact original-child ReleaseAll using the current native commit time. If newer active/Stop or another sender exists, release is denied before publication and canonical MCP never sends a dangerous fresh fallback. Explicit Stop retains its original 500ms age admission | Pure Rust 4/5-cohort TTL/ack-loss negative fixtures AUTHORED_NOT_EXECUTED; no IPC/v4 ABI/second worker change |
| Same-tick MC02 batch cleanup source pin | The original authenticated pipe already pins the CLI process PID/birth; original Channel now remembers its last accepted publisher. A legacy batch's implicit ReleaseAll requires exactly that publisher, even within the same GetTickCount64 millisecond. Explicit Stop and 4/5-AI neutral bypass the batch-only source match. Known refusal reuses MC02_RELEASE_OBSOLETE and canonical MCP no-fresh-retry safety | Rust 4/5-cohort and CLI error fixtures AUTHORED_NOT_EXECUTED; zero IPC/game ABI changes |
| Same-millisecond active input admission fence | The existing original per-child Channel requires active issue time strictly newer than its last accepted active and neutral times. An equal-tick delayed CLI can no longer replace the current key/camera publication, while ReleaseAll preserves the explicit Stop safety tie-breaker | Pure source Rust 4-AI/5-AI negative fixture AUTHORED_NOT_EXECUTED; no IPC/v4 ABI/lease changes |
| Original MC02 legacy batch process pin | One existing authenticated native Inspect pins original MC02 PID/birth/session before first batch pulse. The already-existing private Mc02Input variant gains one optional batch_origin field, omitted for older one-shot bytes; actor compares it to the retained Channel binding BEFORE marking legacy input or publishing v4. Every batch pulse and its neutral carry the same origin; replacement/initial unverified target fail closed. Canonical MCP does not neutralize another child after proven pre-dispatch refusal | Rust actor/wire/CLI and Node negative fixtures AUTHORED_NOT_EXECUTED; no new public command, game-shared ABI, writer or per-pulse target lookup |
| Original MC02 child-bind timestamp fence | Original per-process native Channel now records its single monotonic bind time after exact child identity binding and refuses both active and ReleaseAll requests issued before or at that time, before v4 memory is mapped. Native batch cleanup classifies this known pre-publication refusal as MC02_RELEASE_OBSOLETE so the verified MCP sender cannot neutralize the newly launched game with another fresh CLI. This is a bounded old-child refusal; same-tick cross-CLI command origin after bind remains unknown | Pure Rust four/five-cohort and CLI error-kind fixtures AUTHORED_NOT_EXECUTED, no private IPC/ABI/enablement change |
| Legacy MC02 batch cleanup original-issue fence | The same-image native batch uses its last potentially dispatched pulse's original issue timestamp for ReleaseAll, not a fresh cleanup time. If a newer one-shot or neutral was accepted, original Channel refuses the stale release before shared-memory mutation. Definitive stale/expired actor refusal is exposed as MC02_RELEASE_OBSOLETE; only canonical MCP sender can avoid a dangerous second freshly timed neutral. Other uncertainty retains the original independent fallback | Rust/Node negative fixtures AUTHORED_NOT_EXECUTED; no IPC/v4 ABI or feature activation change |
| Legacy MC02 canonical neutral ACK reuse | Original MC02 CLI already releases the channel before successful batch response; MCP no longer launches a redundant, potentially backdated neutral CLI for its exact canonical sender. Failed/aborted/malformed/injected receipts keep independent neutral. Late Stop during cleanup cannot produce false success | Pure Node provenance and late-Stop fixtures AUTHORED_NOT_EXECUTED, no controller/IPC/v4 ABI change |
| Post-release AI epoch replay fence | Original AiLeaseBook stores one per-slot maximum admitted e1 FILETIME even after explicit native release. Older, equal-issue or replayed lease acquisitions refuse before any 4/5-cohort mutation. Only original child reap resets the maximum; source-old epoch cannot pass the existing process-birth admission for a replacement. The same stale error code feeds MCP no-write detection | Existing and new pure four/five AI Rust fixtures AUTHORED_NOT_EXECUTED; no IPC/v4 ABI or default-on changes |
| Live same-writer native action-lease isolation | The original AiLeaseBook now refuses a second normal acquire while any selected exact-owner action lease is still live, even if its state is neutral and writer ID matches. This prevents a delayed older CLI from replacing the active transaction epoch; existing same-owner recovery-only takeover and explicit neutral ACK remain available | Pure Rust hybrid four-AI and full five-AI fixtures AUTHORED_NOT_EXECUTED; no wire/ABI/feature change |
| Native lease-refusal provenance fence | The existing Rust CLI recognizes busy/mode actor refusal only when the error KIND and exact message agree. BrokenPipe, timeout, interruption and controller contention may no longer impersonate proven no-write; MCP keeps uncertainty and recovery on such responses | Rust negative actor-versus-transport fixtures AUTHORED_NOT_EXECUTED, no native controller/IPC change |
| Stale ReleaseAll after newer neutral fence | The original per-slot Rust writer refuses an older neutral issue timestamp after a newer accepted neutral, not only after newer active input. This avoids invalidating v4 cursor revisions/epochs and pending raw key-up reads from replayed old neutral CLI calls; same-tick release remains admitted | Original four/five-client pure Rust boundary fixtures AUTHORED_NOT_EXECUTED; experimental compile-OFF |
| Final native AI timestamp-group deadline | Original Rust Channel::write now receives the existing group deadline through the retained controller, checking GetTickCount64 before v4 publication after slow Windows HWND/mapping reads; late packets refuse and take existing partial-group neutral/recovery. Direct writes and ReleaseAll remain unchanged, no ABI/IPC migration | Rust hybrid four-AI/full five-AI boundary and CLI error fixtures AUTHORED_NOT_EXECUTED; compile-OFF |
| Late native ACK cancellation fence | Existing MCP batch completion now refuses stale success if Stop or shutdown arrives during per-slot lease release after native ACK. All original cleanup attempts finish, the affected cohort remains quarantined, and neutral/lease failures keep priority | Four/five-AI late Stop and shutdown negative fixtures AUTHORED_NOT_EXECUTED |
| Observed 4/5-AI post-lease PNG fence | The existing MCP round retains the exact delivered frame basis across the native lease wait, rejects stale/advanced images before batch dispatch, and reports the actual native request start time. No second observer or controller | Hybrid four-AI / full five-AI negative fixtures AUTHORED_NOT_EXECUTED; no Minecraft game input proof |
| Integrated virtual focus return fence | Existing game-local GetForegroundWindow/GetActiveWindow/GetFocus detours now pin original revision and recheck live native packet, OGLES ownership, human foreground and expiry before returning spoofed focus; native ReleaseAll/replacement cannot leave an acknowledged stale focus result after the check | Reuses existing sameActivePublication C++ negative fixtures; compile-OFF; Windows tests NOT_EXECUTED |
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
  mouse-up and partial-prefix consumption. Existing virtual foreground,
  active and focus returns now recheck the exact same live native
  publication after their HWND/thread lookup. No driver, second writer,
  extra hook, changed ABI or default product enablement.
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

**F3 client-native engine admission:** Review an **authorized exact
Minecraft Education 26.33 Windows client image**, with verified product
identity, to establish the original game's F3 input handling, live
Player XYZ/dimension/camera accessor and in-render HUD presentation
interface. Existing GitHub source has no Education executable or native Player
/keyboard ABI. The D3D11 renderer and key state now exist only behind
a disabled source flag, with no source-bound player/input owner.
Do not ship speculative binary offsets, a
Windows-layered overlay, a world Add-On or a fake F3 toggle.
The whole feature needs one integrated version-gated input + data +
in-game render implementation and actual Windows/Minecraft acceptance
before claiming completion. When these inputs are missing, maintain
`BLOCKED_ENGINE_ABI`; do not add more source-only feature stubs or
run a test/game without separate authorization. See
[client-native F3 design](../docs/system/minecraft-education-native-f3-client.md).

## Separate original native-input hold — unchanged

**Final source-only native/CLI/MCP integrity review completed.**
The only MCP writer lease acquisition refusals classified as
definitely denied are original `NATIVE_WRITER_BUSY`,
`NATIVE_WRITER_MODE_MISMATCH` and
`NATIVE_WRITER_EPOCH_STALE`. These correspond to exact native
actor-compatible error kinds/messages after original mode, child
birth, epoch and lease admission checks, before any acquisition
mutation. Unknown refusals, broken pipes, timeouts and lost ACKs
remain uncertain; per-slot recovery and quarantine are preserved.

A source-proven mismatch was corrected here: the original
Sessions::mc02_input can refuse a nonresponsive/absent MC02
window, missing human MC01 or changed MC01 foreground BEFORE
calling Channel::write. Such exact InvalidData early refusals now
restore the prior legacy pending fence, like original Channel
pre-v4 refusals. The actor keeps earlier unresolved input pending,
and generic Win32/HWND/IPC errors or native Unmap failures AFTER
possible publication NEVER rollback. Pure Rust negative/positive
provenance fixtures were AUTHORED_NOT_EXECUTED.
No extra writer/lease owner, status type, IPC schema, v4 ABI,
input hook or default-enable change was introduced.

**Separate native-input decision: STOP speculative source guard changes on
this track.** The native input source is integrated at the
REMOTE_GITHUB review ceiling only. To establish actual
correctness, source tests and Windows compilation would need
separate operator authorization, then Minecraft Education 4/5
client execution and gameplay/key-up acceptance. All of those
remain `NOT_EXECUTED`/`NOT_VERIFIED`. Preserve original
per-client neutral, hybrid human MC01, independent profile data,
strict Cua Invoke-only `BACKGROUND_UNAVAILABLE`, disabled
GitHub Actions, and release-reserved `main`.
