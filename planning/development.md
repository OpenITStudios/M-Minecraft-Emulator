# Current Development Handoff — Lazy Tester

**Updated:** 3 November 2026. **Branch:** `Local` only;
`main` release-reserved; GitHub Actions CI disabled.
This file is the **single current status and next-action owner**.
Older detailed sections are preserved in
[the November 2 historical handoff](history/source-handoff-through-2026-11-02.md).
Always pin the actual latest `Local` SHA; an old report is not
new-revision acceptance.

## Product and owners

- **Native session preparation ordering (3 November, source-only):**
  the existing MC01–MC05 controller now validates its reviewed bootstrap
  DLLs before `registered_slots::<5>(true)` may recover an unfinished
  clear or persist the profile locator. Missing/mismatched DLLs therefore
  refuse before these profile-side effects. The controller and installed
  data contracts are unchanged. Source-order fixture AUTHORED_NOT_EXECUTED;
  no Windows DLL load, Rust build, session or Minecraft acceptance occurred.

- **First local Windows PC test handoff (3 November, source-only):**
  [one standard desktop runbook](../docs/validation/local-pc-first-run.md)
  now routes clean `Local` clone, pinned toolchain, source preflight,
  reviewed native DLL admission, developer UI, optional package and
  progressive MC01–MC05 tests through existing `DEV.cmd`/`RUN.cmd`.
  A checkout does NOT contain the two reviewed DLL binaries or a root
  executable. `DEV.cmd localcheck` now checks clean Local/source locks,
  toolchain presence, the Windows SDK and two pinned native DLL hashes
  without downloads or execution; `bootstrap` now propagates the real
  PowerShell exit status. Follow-up source-only hardening verifies tracked
  lockfiles, rustfmt, MSVC x64 compiler/linker files and native DLL directory
  ancestry; package now verifies native x64 OS architecture and runs pinned
  toolchain doctor before npm; dev verifies architecture before startup.
  PowerShell 5.1 localcheck labels platform architecture unverified rather
  than claiming native x64. Regression
  fixtures are AUTHORED_NOT_EXECUTED. No Windows, package, account or gameplay action
  was executed or approved by documenting it. This does not resume
  BLOCKED F3 or experimental AI-input work; the single next action
  below remains unchanged.

- **Windows SDK packaging prerequisite correction (3 November, source-only):**
  the native helper resource build requires x64 `muirct.exe` as well as
  root launcher `rc.exe`. The read-only localcheck now refuses a missing
  MUI compiler before Tauri/Cargo compilation, matching the existing
  `prepare-process-roles.mjs` consumer. Fixture AUTHORED_NOT_EXECUTED;
  native tool operation and package remain NOT_EXECUTED.

- **Native DLL first-run admission correction (3 November, source-only):**
  a source-pinned DLL pair discovered in existing `App/` is not sufficient
  to declare dev/package ready if `LAZY_TESTER_NATIVE_PAYLOAD_DIR` is unset.
  Read-only localcheck now reports BLOCKED with the exact action instead of
  returning a false-success 0 exit code. Test fixture AUTHORED_NOT_EXECUTED.

- **Post-package desktop startup verifier (3 November, source-only):**
  the existing Windows verification script now refuses automatic manager
  cleanup unless the observed process was created after the explicit launch
  request and its Explorer-parented root launcher matches the staged receipt.
  A concurrently observed older/unverified manager is left alone; no game
  is launched or stopped. Fixture AUTHORED_NOT_EXECUTED; actual PowerShell,
  desktop UI, packaged startup and five-client acceptance NOT_EXECUTED.

- **Desktop:** Tauri/Svelte manager for up to five separately licensed
  native Minecraft Education Windows profiles; the operator launches
  games explicitly. Svelte/typed bridge is presentation only.
- **Desktop UX (2 November):** the existing visual theme, fixed five-card
  bottom rail, central Play viewport, and native DWM peer previews remain.
  Rail order is sorted by fixed MC01-MC05 storage slot in the presentation
  layer only; selection does not recreate preview elements. The rail heading
  now matches its existing 1600px width cap. Relevant regression scenarios
  are AUTHORED_NOT_EXECUTED; live Windows/Minecraft behavior NOT_VERIFIED.
- **Desktop clarity UX (2 November):** unify global/per-slot read-only
  Refresh wording; distinguish verified profile data from actual game
  readiness, suppress stale running-count claims during connection loss,
  and require positive window response before offering guided Select game
  after a DWM preview failure. Preserve rail geometry, theme and native
  admission. Updated UI fixtures AUTHORED_NOT_EXECUTED.
- **Desktop focused UX (2 November):** selected-game Close review focuses
  Cancel, supports Escape and restores keyboard focus to the invoking
  launcher after cancellation. Graphics legacy details and read-only
  resource-pack diagnostics use native disclosure; basic Storage actions
  remain visible. No visual theme, five-slot rail, native IPC or game
  session change. Regression fixtures AUTHORED_NOT_EXECUTED.
- **F3 Coordinate Copy P0 (3 November, source-only):** the desktop pure
  `src/app/coordinates.ts` now parses an explicitly UNBOUND, minimal
  Script API-shaped position/rotation/dimension/gameTick contract. It
  provides full-decimal XYZ, floor-based block/chunk and same-dimension
  /tp text (yaw then pitch) without reading data or calling clipboard.
  It rejects fabricated slot/world/source/verified metadata. Focused
  Vitest regression fixtures AUTHORED_NOT_EXECUTED; Windows/Minecraft
  identity, live coordinates, bridge, UI, clipboard, hotkeys and game
  behavior are NOT_VERIFIED / NOT_IMPLEMENTED. Canonical team telemetry
  and the existing FPS HUD remain unchanged. See
  [coordinate inspector design](../docs/system/coordinate-inspector.md).
- **F3 Coordinate Evidence P1 (3 November, source-only):** read identical
  original uploaded team skills ZIP snapshots (SHA-256
  26c0a322cabed172ca74970413d5bb7a20d6f67181446df5f688e36057c54e21).
  Team modules/coords already handles spawn/block math; tier-3
  analytics position_samples are opt-in HISTORICAL floored XYZ
  without rotation or live player/world attribution. Added a read-only
  per-slot/process candidate screen to existing src/app/coordinates.ts
  with exact PID/birth/session/HWND, selected slot and receipt-age
  screening; every status has copyAllowed=false and still refuses
  actual player/world origin as UNVERIFIED. Tests AUTHORED_NOT_EXECUTED;
  no new source emitter, collector, IPC, HUD, clipboard, hotkey,
  native lifecycle, gameplay, CI or release action.

- **F3 Coordinate Continuity P2 (3 November, source-only):** separated
  coordinates.ts (pure XYZ/block/chunk/teleport formatting) from
  coordinateEvidence.ts (untrusted candidate screening). Added stateless
  two-frame checks for slot, PID/birth/HWND, dimension changes, tick
  replay/reset and receiver monotonic gaps. The original team's analytics
  position_samples remain historical and cannot identify the local
  Minecraft player in a Windows multi-client session. Every source-only
  assessment still returns copyAllowed=false; independent game/world
  attribution remains missing. Focused Vitest fixtures AUTHORED_NOT_EXECUTED.
  No runtime capture, Script API command, source logger, native IPC,
  HUD, clipboard, UI, CI or installer change.

- **F3 manual Copy Coordinate P3 (3 November, source-only):** Microsoft
  documents Bedrock Creator Copy Coordinate UI and Ctrl+Alt+C/X for
  player/target XYZ, but Minecraft Education 26.33 compatibility
  remains UNVERIFIED. Added bounded user-pasted XYZ parser and
  XYZ/BLOCK/position-only /tp string formatting in the existing pure
  desktop coordinates.ts; source is USER_PASTED_UNVERIFIED and no
  player/client/world trust is invented. Test fixtures AUTHORED_NOT_EXECUTED.
  No new source emitter, game input, clipboard read/write, native IPC,
  Svelte panel, installer, CI or source test execution.

- **Manual XYZ desktop P4 (3 November, source-only):** added one
  collapsed Paste XYZ disclosure to the existing Play-header action
  group. Only explicit user-pasted text is formatted to exact XYZ,
  floored block XYZ or rotation-free teleport text; readonly output
  can be selected but there is NO clipboard reader/writer. Ephemeral
  state clears on close or Play unmount, and is explicitly labeled
  User-pasted · Unverified. Five-card DWM rail, native controller,
  Testing/Probe sources and IPC remain unchanged. Authoring-only
  Vitest fixture added, NOT_EXECUTED; no Windows/Minecraft/CI/build.

- **Live F3 source feasibility P5 (3 November, research-only):**
  Microsoft Education Code Builder exposes current-player position and
  Education /connect is a possible client-originated WebSocket mechanism.
  Win32 GetExtendedTcpTable can report OS TCP connection PID, but
  the game or a Code Builder helper may own the socket; PID alone
  does not prove player/world origin. These mechanisms have NOT been
  shown to interoperate on Education 26.33. Historic Code Connection
  v1.5 supports 1.18.xx or earlier; official changelog also lists 1.X
  for 1.19.xx and higher, which is NOT specific 26.33 acceptance.
  A read-only, no-listener source design and EDU/WIN/MULTI/WORLD
  acceptance matrix are recorded in coordinate-inspector.md.
  No native code, game action, WebSocket endpoint, clipboard, CI,
  installed team pack or auth/Testing route was changed.

- **F3 TCP owner witness P6 (3 November, source-only):** in the EXISTING
  native instance_identity.rs owner, authored a private #[cfg(test)]
  Rust fixture for IPv4/IPv6 loopback 4-tuple, ESTABLISHED state,
  wrong/helper PID, duplicate/missing rows, network-port mismatch,
  process creation FILETIME changes and port reuse. Even unique PID
  equality yields PidCandidateOnlyUnverified, never local-player/world
  proof or Copy permission. No OS GetExtendedTcpTable call, listener,
  Tauri/native IPC, team telemetry, game command or HUD change.
  Rust test fixtures AUTHORED_NOT_EXECUTED; source builds, Windows,
  Minecraft and CI NOT_EXECUTED. Manual Paste XYZ unchanged.

- **F3 P7 transport decision (3 November, research-only):** official
  Education 26.33 changelog does not attest an external position
  stream. Code Builder player.position() has current-player context
  but NO proven read-only desktop export or yaw/pitch. Microsoft
  Bedrock /connect command requires cheats/admin per its Bedrock
  docs; Education 26.33 runtime behavior is UNKNOWN. Community
  WebSocket bridges require extra behavior packs/servers and report
  packaged-app loopback exemptions; do NOT add these to Lazy Tester.
  Microsoft's old Bedrock Code Connection lost support in 2023;
  Education's 2023 1.X compatibility entry is not 26.33 proof.
  WIN TCP 4-tuple owner PID alone cannot authenticate coordinates.
  Decision: no adapter/listener/game changes or more synthetic
  authorization fixtures without a real approved endpoint. Retain
  P4 manual unverified Paste XYZ and P6 test-only witness.
  Next source owner is EXISTING DXGI FPS HUD for source-only
  F3 performance feasibility review; no tests/Windows/CI run.

- **P8 existing native FPS HUD audit and diagnostic correction (3 November):**
  one process-owned DXGI Policy/Hud already measures accepted S_OK
  outer Present/Present1 rates, not displayed/scanned-out or configured
  FPS. BootstrapHudSnapshot remains a private child-process DLL export,
  unavailable as a Rust/desktop SessionSnapshot without a reviewed
  cross-process source route. In fps-hud.cpp a worker-retired reading
  can no longer retain `valid=true` just because its QPC age is short;
  accepted counts remain available for diagnostics. Added a bounded
  destroyed-HWND/retired-worker source fixture in fps-hud-control.cpp.
  No new hooks, monitor, renderer, native IPC, game/input command,
  Svelte fields or clipboard handling. C++ fixture AUTHORED_NOT_EXECUTED;
  source compilation, Windows HUD tests, Minecraft and CI NOT_EXECUTED.

- **F3 FPS P9 in-process path and unbind gate (3 November, source-only):**
  BootstrapHudSnapshot is a fixed-size C++ export in the game child;
  only probe.cpp and frame-pacing-control.cpp currently consume it
  inside the same process. Original Rust SessionSnapshot and Tauri/Svelte
  have no read route; adding desktop FPS now would require a separately
  reviewed interprocess transport, not a call to the manager DLL.
  Corrected Hud::snapshot() to hold the existing State gate across
  display/meter/worker reads, invalidating rate while unbound (even
  during normal one-second worker grace), while preserving the accepted
  counter. Extended hidden-window control fixture for synthetic
  Present calls after unbind. No new renderer, native IPC, worker,
  DXGI interception or clipboard. C++ tests AUTHORED_NOT_EXECUTED;
  Windows/gameplay/source builds/CI NOT_EXECUTED.

- **P10 manual Paste XYZ keyboard/a11y (3 November, source-only):**
  Scope Escape to a focused/open manual details panel only, close
  disclosure and restore focus to its summary. The XYZ textarea
  now exposes aria-invalid and links guidance/validation error;
  the readonly output links the Ctrl+C instruction. No clipboard
  I/O, selected-game Escape conflict, modal focus trap, persistent
  input, five-launcher layout or controller mutation. Fixtures
  AUTHORED_NOT_EXECUTED, Windows/gameplay NOT_EXECUTED.

- **P11 Paste XYZ latest-value selection (3 November, source-only):**
  Reactive XYZ/Block/Teleport formatting already existed. Hardened
  Select text: independently reformat current raw input/mode, wait
  for Svelte tick, and select only when the panel is still open,
  input/mode and valid output remain unchanged, and readonly DOM
  field equals fresh text. Typing or switching mode clears any
  prior selected output. Additional regression fixtures cover
  mode transitions, negative floor, tiny decimals, partial and
  malformed values. NO clipboard I/O, game control, native IPC,
  extra timer, five-client layout or telemetry change.
  Tests AUTHORED_NOT_EXECUTED; Svelte/Windows/gameplay NOT_EXECUTED.

- **P12 lossless manual-coordinate correction (3 November, source-only):**
  found that Number('0.99999999999999999') can round to 1,
  causing incorrect Block XYZ and lost pasted digits. The existing
  coordinates.ts manual formatter now validates/normalizes exact
  digit strings before producing XYZ, position-only /tp text and
  mathematically floored BLOCK. Numeric parseUserPastedCoordinates
  API remains backward-compatible but approximate and is not
  used for text/block output. +/-1e9 bounds checked from digits;
  significant precision and tiny negative fractions retained.
  Fixtures AUTHORED_NOT_EXECUTED; no UI, FPS, controller,
  clipboard, game command or new telemetry work.

- **P13 F3 Inspector selected-client UI (3 November, source-only):**
  officially Education lacks Java F3 debug screen; Show Coordinates
  is native world-settings fallback. Added an opt-in read-only
  F3 Inspector in Play header, independent of P4 Paste XYZ.
  It projects only current selected native PID/session/window and
  latest configured FPS limits from the existing SessionSnapshot.
  No active client, mismatched native PID/FILETIME/HWND, recovery
  ambiguity, lost binding or connection suppresses native details.
  Measured FPS, XYZ, rotation and dimension remain explicitly
  UNAVAILABLE rather than guessed. No new native IPC, Script API
  emitter, game hook, extra HUD, polling, clipboard, or controller.
  Pure-view and UI source tests AUTHORED_NOT_EXECUTED; no Windows
  gameplay or source compilation claimed.

- **P14 F3 Inspector pending-operation admission (3 November, source-only):**
  original UI busy/close-review/switch/launch/navigation markers
  now withhold previous PID/window/settings (ACTION_PENDING) until
  the original operation settles. This is a derived pure projection,
  not new process or game ownership. F3 shows last reported GLOBAL
  FPS target limits as settings, never a per-client measured rate;
  all live player fields remain unavailable. Added regression fixtures
  AUTHORED_NOT_EXECUTED; no extra native IPC, game hook,
  clipboard permission, collector or launcher layout change.

- **P15 F3 per-window visual-source triage (3 November, source-only):**
  Audited pinned original analytics ZIP: historical opt-in floored
  position_samples are not live/current per-client coordinates.
  Existing MCP observe_client and read_visual_observation are
  native exact-window screenshots for explicit review, not player
  or world proof. Added f3VisualReview on their existing metadata
  via pure projectF3VisualReview() in visual-observer.mjs.
  It reports a reviewable fresh window frame vs. unavailable;
  never parses XYZ, establishes a local actor/world, grants
  Copy, starts another collector or changes the desktop HUD.
  CUA/WGC positive/negative fixtures AUTHORED_NOT_EXECUTED.
  No new MCP tool, native IPC, Script API, world/game input,
  Windows tests, CI or external authorization changes.

- **P16 F3 window ↔ player pairing feasibility (3 November, source-only):**
  re-read pinned team ZIP SHA-256 26c0a322...5e21.
  display-service.ts is the only per-player actionbar writer;
  hud-line.ts composes per-player HUD slots, refreshes every 10 ticks
  and offers NO existing native/MCP challenge entrypoint or
  player/world ↔ Windows PID binding. Analytics samples are historical.
  Path A: original client Show Coordinates + pinned visual capture
  can at most offer manually reviewed SCREEN_OBSERVED_UNVERIFIED
  XYZ, not a real-time/actor-authenticated feed. Path B: future
  team-owned per-player short-lived HUD challenge needs a separately
  authenticated game/server ↔ original native controller channel;
  nonce echo, username, @s or matching PID alone is insufficient.
  Eight fail-closed gates EDU-HUD/ORIGIN, WIN-BIND, CHALLENGE, FRAME,
  MULTI, WORLD, F3 are in coordinate-inspector.md, all NOT_EXECUTED.
  NO team source or product runtime was changed; no Script API,
  command, network, new tool, UI/IPC, account/world mutation or CI.

- **P17 simplified F3 screen-XYZ candidate (3 November, source-only):**
  Reused original MCP read_visual_observation screenshot cache.
  Optional screenReview accepts one AI-transcribed top-left
  `Position: X, Y, Z` line with exact frame sequence, source time
  and process PID/birth/session/HWND. New pure validator refuses
  stale/replaced/invalid frames, ambiguous text and forged fields.
  Candidate always AI_TRANSCRIBED_SCREEN_TEXT_UNVERIFIED;
  `automaticCopyAllowed=false`. No actual OCR engine, new MCP
  tool, new capture, Windows Media OCR/MSIX packaging change,
  native IPC, in-app Copy, world command, CI or test execution.
  Static source scenarios AUTHORED_NOT_EXECUTED; real screenshot
  readability and desktop linkage remain unresolved.

- **P18 F3 image delivery gate and desktop transport blocker (3 November):**
  DWM thumbnails supply no Svelte pixel buffer. Exact-window PNG
  exists only in opt-in MCP; the separate WGC helper is unbuilt.
  Microsoft Windows.Media.Ocr needs MSIX identity, while this
  product uses NSIS. Corrected P17's metadata-only XYZ review:
  original volatile visual observer now remembers the last
  explicitly delivered PNG sequence/source time/owner, denying
  reviews without that image witness. All candidates unverified,
  Copy always false. No new capture, IPC, installer or UI.
  Node fixtures AUTHORED_NOT_EXECUTED.

- **AI/QA activity contract (2 November):** existing MCP
  `agent_capabilities.activity` projects only allowlisted in-memory
  visual-sampling, Probe checkpoint/report, and experimental AI-input
  recovery/request state; **never** external Windows control or Minecraft
  gameplay success. No new tool/collector, native IPC, hidden UI badge
  or extra action is introduced. Desktop has no authenticated MCP status
  reader at that earlier revision; the new explicit native-controller
  report mirror below supersedes this transport limitation without
  proving Windows runtime. Fixtures AUTHORED_NOT_EXECUTED.
- **Optional AI/QA desktop status mirror (2 November):** the existing MCP
  `agent_capabilities({publishToDesktop:true})` can explicitly report only
  enum-allowlisted local activity through a matching one-shot native CLI to
  the already-retained original controller. It adds an optional
  `SessionSnapshot.agentActivity`, bounded monotonic 12-second freshness
  and a small Play-header disclosure for relevant or stale-ATTENTION status.
  Ordinary capabilities are read-only; no hidden watch, auto-publish,
  Tauri publishing authority, native input or layout change. The source
  reports are not verified Windows control, gameplay or map QA. All newly
  authored Rust/Node/desktop fixtures are NOT_EXECUTED and the complete
  cross-process UI handoff on Windows remains NOT_VERIFIED.
- **AI/Testing UI wording (2 November):** all activity labels and descriptions
  are in English, with user-facing "Testing" instead of "QA".
  The existing status source, five fixed DWM cards, controller paths and
  `probe_qa` API identifiers remain unchanged. Updated regression assertions
  are AUTHORED_NOT_EXECUTED.
- **One-click Testing Summary (2 November):** the existing Play-header
  activity disclosure presents original Probe target and completed
  reported PASS/FAIL/REVIEW outcome, plus allowlisted counts already
  present in the in-memory `probe_qa` terminal snapshot (if complete
  and consistent). Reports always show "Map Testing: Not tested" and
  "Verification: Not verified"; blocked/incomplete checkpoints do not
  masquerade as Results available or acquire fabricated counts. No new
  page/engine/IPC or result archive, no layout/previews changes.
  Native/TypeScript/source validation, MCP fixtures and browser
  regressions AUTHORED_NOT_EXECUTED. The explicit status mirror still
  expires after 12 seconds and requires agent opt-in publication.
- **Single AI-driven Testing entry (2 November):** The existing
  `probe_qa({slot,publishToDesktop:true})` can explicitly mirror its
  current source-only Testing activity to desktop in the SAME MCP call,
  rather than requiring a second `agent_capabilities` tool call.
  Returns a conservative `operatorNextAction` hint while preserving
  the ONE original per-slot checkpoint/immutable `terminalReport` and
  optional nested map `qaReport`. The default `probe_qa` behavior is
  unchanged, no automatic Minecraft command/UI automation, no second
  test-runner, account/world mutation, CI, app/window rearrangement,
  persistent report archive or background observer. Regression fixtures
  AUTHORED_NOT_EXECUTED; actual PC control and Skills Probe world/result
  provenance NOT_VERIFIED.
- **Cooperative cross-MCP Testing run correlation (2 November):**
  The original native controller now holds a bounded one-entry-per-slot
  reservation book for new `probe_qa({coordinateNative:true})` calls
  from cooperating MCP agents. Other owner IDs see the existing ID,
  cannot prepare a second canonical Probe checkpoint, and cannot
  overwrite a finalized source-only status + report digest. An explicit
  same-owner `restart:true` can replace only a FINAL reference.
  Default legacy uncoordinated `probe_qa` remains compatible, so this
  does NOT guarantee universal deduplication. Native book and MCP
  terminal reports remain volatile, not a durable report archive.
  No second Testing runner or game controller, no automatic
  in-game command, no extra desktop UI, no CI or local execution.
  Native/Node regression tests AUTHORED_NOT_EXECUTED; real Windows
  source/game/world identity and five-client acceptance NOT_VERIFIED.
- **Optional immutable Testing SUMMARY archive and recovery fence (2 November):**
  The existing `probe_qa` can use an explicitly configured absolute
  `Data/Testing` root for bounded, write-once per-run Skills Probe
  terminal summaries after the original controller acknowledges FINAL.
  A manifest is committed last and checksum verification precedes
  read-only `probe_qa({archivedRunId})` retrieval after MCP exit.
  A newly started MCP with an earlier committed result on the same
  slot receives a RECONCILIATION_REQUIRED hold, not an automatic
  second Testing run. A competing owner cannot take over an abandoned
  RESERVED run based on age or missing screenshots. The original full
  terminalReport and native reservation book are still volatile,
  the private summary archive is opt-in, and the desktop Play header
  remains a nonpersistent 12-second status. No new Testing engine,
  world/input command, Tauri UI page or CI. Regression source fixtures
  AUTHORED_NOT_EXECUTED; Windows world/log provenance unverified.
- **Read-only orphan Testing inspection (2 November):** the SAME
  `probe_qa({slot,inspectRun:true})` route reads the original native
  controller's volatile reservation phase, run ID and fixed recovery
  action without preparing a new Probe, publishing activity or writing
  game input. It preserves an explicit `transferAllowed=false`,
  `operatorApproval=NOT_AVAILABLE_IN_PROTOCOL`. At that earlier revision,
  `ownerProcessProof=NOT_BOUND_TO_PROCESS_HANDLE`; new coordinated runs now
  bind an OS-observed Node parent lifetime as described below. A token
  match alone is not proof of MCP-script identity. No unapproved native transfer,
  extra UI/Testing engine, CI or background service. Native/Node
  read-only regressions AUTHORED_NOT_EXECUTED; Windows runtime unknown.
- **Pinned MCP Testing parent lifetime (2 November, source-only):**
  the existing same-image CLI identifies its Windows-observed direct Node
  parent; the original native TestingRunBook retains its handle, PID and birth
  when a coordinated reservation succeeds. Same-token reserve/finalize
  now rejects changed, exited or unknown parents, including PID reuse.
  Inspection reports only bounded running/exited/uncertain OS-parent state,
  not MCP-script authenticity, Minecraft game/world provenance or operator
  consent. `transferAllowed=false` and no operator approval remain enforced.
  Windows/runtime tests and source suites NOT_EXECUTED; source regressions
  authored only. No second controller, automatic input or new desktop UI.
- **Read-only native/archive Testing reconciliation (3 November, source-only):**
  the existing `probe_qa({slot,inspectRun:true})` now combines the
  original native inspection with an explicit optional `Data/Testing`
  immutable-summary scan. Final run ID/outcome/digest correlations,
  archived historical summaries, missing archives and contradictory or
  unreadable records get distinct bounded statuses. Missing archives,
  confirmed OS-parent exit, hash matches and model input **never**
  authorize a run transfer: `archiveReview.authorizesTransfer=false`
  and original `inspection.transferAllowed=false` remain enforced.
  Pure Node regression scenarios authored NOT_EXECUTED; no new native
  controller, Test runner, desktop panel, CI, Windows or Minecraft action.
- **Original-controller Testing CAS correctness core (3 November, source-only):**
  `native_clients_session.rs` now contains a sealed, private
  `transfer_with_sealed_admission` operation that can replace ONE
  original RESERVED run only after exact old run/owner comparison, distinct
  successor identity, pinned old-parent OS exit and live pinned successor.
  Pure Rust preflight regressions reject stale IDs, FINAL, wrong owners
  and invalid proposals. Both operator-verification and native-archive
  markers have no production constructors; no command, Tauri or MCP
  path can invoke transfer. Authored tests NOT_EXECUTED; no gameplay,
  Windows native runtime, source suite, CI or installer execution.
  P0 is still BLOCKED pending native archive trust and independently
  authenticated operator consent; an archive screenshot/token is not proof.
- **Native owner:** original Rust session controller handles game
  process identity, private profiles, normal exit and recovery.
  Workspace/DWM presentation never becomes a second lifecycle owner.
- **External model:** optional local stdio `mcp.mjs` delegates through
  the existing same-image CLI/authenticated session controller.
  Default AI input is read-only/action-planning; the independent
  keyboard/mouse candidate is opt-in, compile-OFF and excluded from
  ordinary releases.
- **Team source:** original `minecraft-bedrock-agent-skills` owns
  game analytics/results; original `skills-probe` owns foundation
  tests. Lazy Tester reads reviewed ZIP/Content Log contracts; it
  does not duplicate their modules, analytics or test runner.
- **Local diagnostics:** explicitly imported normalized evidence is
  stored under ignored `Data/Diagnostics` and exposed through
  bounded read-only MCP queries, never automatically collected.
- **Map QA:** the optional approved map-owned JSON contract contains
  requirements, prereqs, evidence families and source selectors.
  It produces a source-only test plan, not an executing map scorer.

Canonical routing: [active architecture](../docs/system/architecture.md),
[source ownership](../docs/system/implementation-map.md),
[Probe QA](../docs/system/map-aware-probe-qa.md).

## Current source workflow (P1/P2/P3)

```text
Optional approved map contract -> bounded full/smoke/affected plan
Approved original Skills Probe ZIP -> cached official auto-probe catalog
Configured MC0x capture file -> stable appended-only checkpoint
Optional retained native game target -> read-only PID/HWND/birth binding
External authorized Player/admin -> original /scriptevent probe:run auto
    [NOT SENT OR VERIFIED BY LAZY TESTER]
Appended original Probe RUN_START / RESULT / RUN_DONE markers
    -> bounded foundation result + source-only map coverage/evidence
    -> frozen optional qaReport and per-slot historical snapshot
```

- `probe_qa` is the preferred **one** MCP QA tool; legacy
  `prepare_probe_qa` and `read_probe_qa_result` remain compatible.
  One volatile plan/terminal snapshot per client; only explicit
  `restart=true` creates another checkpoint. No daemon or timer.
- Approved ZIP/catalog and map contract are cached while unchanged;
  pending reads inspect only the needed added log bytes and cheap
  source metadata. A source change, lost cursor, irrecoverable marker
  gap or replaced target is held, not silently replayed.
- Original foundation-Probe reports distinguish PASS/FAIL/REVIEW,
  waiting and incomplete, **never** map/gameplay acceptance.
  Unknown/unbound native targets cannot produce a verified game PASS.
- P2 `proofGate` explicitly separates source ZIP, local capture and
  process/window observation from **unproven** Player/admin command
  delivery, exact QA world, and process/world Content Log provenance.
  Mixed `dev`/`release` Probe results and detected duplicate local
  capture file identities across slots are refused.
- Optional `mapPlan` is always `NOT_TESTED`. `full` selects all;
  `smoke` needs declared checks; `affected` currently falls back
  to full because MCP does not authenticate an exact map Git diff.
  A plan cannot skip required tests based on AI guesses.
- P3 `mapPlan.selectedChecks` includes up to 12 bounded map
  requirements and their needed evidence, with a local contract
  SHA-256 fingerprint (not author/game proof); the same `probe_qa`
  returns optional `qaReport` during preflight/initial preparation
  and on terminal snapshot. A map plan is still available when
  Probe ZIP/capture is missing. `qaReport.foundation` preserves
  original Probe status; `qaReport.map` always records 0 executed,
  0 passed, 0 failed, and all map requirements NOT_TESTED.
  `qaReport.acceptance=NOT_ACCEPTED` regardless of Probe-reported
  PASS. Pending polls remain compact; final snapshots are reused
  without additional I/O.
- P3 report-integrity correction: `map-test-plan.mjs` now imports
  the original `probe-qa.mjs` auto-count cap (previous undefined
  reference could fail terminal composition). Only bounded status
  reasons are included; required selected/skipped counts are
  validated and exposed. Regression fixtures AUTHORED_NOT_EXECUTED;
  there is still no game command or map PASS.
- Team ZIP changes are inspected offline through existing lock
  fingerprints; new baseline versions are never auto-approved or
  synced into a running world.

## Proof ceiling and unresolved gates

**Confirmed from REMOTE_GITHUB source review:** module ownership,
declared contracts, static data/handler paths, authored bounded
regression fixtures, default-disabled experimental flags and
repository layout. This is not compiler or runtime proof.

**NOT_EXECUTED at the current exact revision:** Node/TS/Rust/MSVC
source tests/builds, package/installer, live Windows/Minecraft
Education, Player/admin ScriptEvent delivery, actual capture origin,
per-client native keyboard/mouse consumption, or five-AI gameplay.
The earlier operator five-client native pilot has an older scope/SHA,
not acceptance of this source.

**Runtime blockers (deferred while local testing is unavailable):**
actual installed Skills Probe in an isolated Education Windows QA
world; one original Player/admin command; Content Log attribution to
the exact game process/world; input/capture behavior. Neither file
names, PINNED ZIP SHA, native IPC ACK nor DWM screenshot proves them.
Do not authorize automatic commands, install game hooks, launch a
world, copy tokens or mutate account/world data through this handoff.

**Current operator priority (3 November):** P18 identifies the
actual MVP blocker: selected-client pixels and an accepted local
recognizer must reach desktop F3. Neither DWM thumbnails nor
MCP alone offers an approved in-app XYZ/Copy channel. Stop adding
fake controls or speculative frame collectors. No assumed Code Builder/WebSocket
parity, automatic clipboard input, native connection listener, source
test execution, CI, game/world mutation or installer changes.
Map Test Contract and original Skills Probe remain unchanged. Do not request local testing, run
source test suites, install packages, use CI or delete private artifacts. Preserve
`App/`, `Data/`, old pilot data, and all historical proof.

## Single next step

STOP speculative F3 source changes until one authorized Windows
Education 26.33 real screenshot confirms Show Coordinates
readability and a deployable OCR/recognition route is chosen.
Then consider ONE explicit user-initiated exact-window PNG
handoff to the existing F3 panel, with independently reviewed
process binding and bounded memory, never a second collector.

Do not silently add Tauri IPC, alter NSIS packaging, launch
the experimental unbuilt WGC executable, enable game
commands, connect Code Builder/WebSocket or present model
text as authenticated gameplay. Existing manual P12 Paste
XYZ remains usable; live F3 XYZ and one-click Copy stay
NOT_CONNECTED. Node/Vitest/TS/Svelte/Rust/Windows/Minecraft
and CI remain NOT_EXECUTED. Testing sealed CAS BLOCKED.
