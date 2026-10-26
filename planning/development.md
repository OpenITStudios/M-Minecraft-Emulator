# Current Development Handoff — Lazy Tester

Local product branch only; main release-reserved; GitHub Actions entirely OFF.
RAM/renderer, Sandboxie and VMware routes remain PAUSED. Physical layout retained.

## Current bounded outcome — Desktop mode and stable window names

Parent77323a76217f152186824b4662277e10b07199ce. Operator interrupted the two-game
acceptance run to fix desktop-window naming and the transparent empty Workspace.
User requested familiar English naming and useful mode information. Gameplay
validation is deferred while this bounded source fix is completed.

The existing Svelte owner calls the modes Workspace mode and Desktop mode.
Use desktop windows replaces Restore windows. Desktop mode replaces the empty
viewport/filmstrip with numbered active-session cards, actual status, Open in
Workspace and Back to Workspace. It never tells the player to launch again merely
because games are outside the app. It retains the existing native release/select
bridge, clears pending presentation on release, and adds no session-state owner.

The session controller originally labeled a window only on first discovery. Its
retained-window branch now reuses the same identity-checked, bounded label helper
on existing inspection. A game-reset caption is corrected to Minecraft - n; an
already-correct title incurs no WM_SETTEXT. Identity never comes from title text.
No injected hook, title-watcher thread or new timer. The original native aperture
owner now invalidates/repaints manager children/frame on clear, including an
already-clear region, to address stale WebView transparency after native release.
No API/wire, installer, storage or FPS-policy changes.

## Executed evidence

32 frontend tests, Svelte zero errors/warnings and production frontend build PASS.
Expanded optional headless mocked-Tauri regression PASS, including Desktop mode
with two numbered sessions, no old empty-stage/filmstrip,920x620 bounds and return
to Workspace. Synthetic preview screenshot (no user data):
workspace/runs/lazy-tester-ui-regression/desktop-mode.png.

Two targeted Windows Rust tests PASS: caption reset/reapply plus wrong-PID/retired
window refusal, and child repaint when aperture is already clear while manager
region remains removed. These own synthetic windows only. Repository structure,
G0 and diff whitespace checks PASS. Real WebView repaint/caption persistence with
Minecraft after this change still requires a new-controller restart and retest.
Existing unrelated DACL disposable fixture, oversized-pipe hang and rustfmt drift
remain unresolved; no globally-green suite claim.

## Actual runtime before the interruption — older candidate only

Exact running App source b0ecca914a11c7e5ad23c73d946f37c925d83450; all four payload
hashes verified before launch. Starting with zero game processes, profiles1 and2
were explicitly launched through the manager. Both reached Education v26.32 main
menus concurrently, with different visible player labels. Workspace displayed the
selected game and the other game's live preview. Restore windows returned both
to independent top-level desktop windows. Observed menu counter around133..137FPS
foreground and10.0FPS background is a spot observation, not full FPS acceptance.
No authentication UI was automated; labels alone prove neither license nor auth
isolation. Gameplay/world/join/Host World not accepted.

Profile1 opened Play/Create New and the draft name LT Acceptance MC01 2026-10-26
was entered. Creation/loading was not confirmed before user interruption. Existing
worlds were not opened/deleted or read from disk. Do not assume this draft created
an accepted world. Two real Minecraft processes and their old controller remain
running; preserve them. The manager/controller may need normal closure to stage
or activate a new candidate. Do not kill them or overwrite their executable.

## Packaging and preservation

Build a clean-revision Windows/NSIS candidate with --export-only while old games
remain active. This creates reviewable output without replacing the running App.
Actual App remains b0ecca9 until a safe normal restart/staging step is authorized.
Native DLL pins remain:
compatibility9d1cf60673c1ebbc0d94c30c1e28e226aa4429653901e5abd6e95400457ef8cb
enroller6e7c000bee2dcf043d372317c77a04078ca12451c7273a18bd54e8f89e62af2a
Previous package receipt/history: workspace/runs/setup-recovery-20261026.
Previous source scopes: Settings19860393, usability300d591, setup UI88aa73f,
selected-folder message b0ecca9. New source acceptance must not inherit their proof.

Existing retained pilot stores remain in place; root Data is empty and has no
verified product catalog. No migration, token/account copy, reset, cache clear,
installer run, service/driver change, UAC, main/release or CI operation. Metadata
proof only in workspace; user screenshots/player identifiers never committed.
Submodule, second worktree and unknown local material retained.

## Remaining proof / Next — one action

After a safe normal exit of the existing games/controller, stage the reviewed
candidate and retest Desktop mode with two games: persistent numbered titles,
opaque manager status area and return to Workspace. Until observed, actual fix
acceptance is NOT_EXECUTED. Resume gameplay/license/FPS checks afterward; no forced
shutdown or real data migration is implied. STOP for the bounded implementation.
