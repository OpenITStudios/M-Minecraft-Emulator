# Current Development Handoff — Lazy Tester

Local only; main/publication untouched; CI OFF. Legacy VM/Sandboxie and RAM/renderer
experiments PAUSED. User chose to RETAIN compiler caches; delete nothing.

## Current scope and installed revision

User requested consistent frame, smooth movement, per-launcher separate windows,
persistent/larger previews, usable Close and understandable Task Manager entries.
Installed App/root starter:1cddf597505b705920d31665730a6267e64bf9a4, replacing04b5c52.
Actual Explorer startup Connected/five enabled/cacheInsideData=true PASS.
Root/desktop shortcuts point to project Lazy Tester.exe. User reopened manager13660
(root29568); controller40924 later observed with no game children. Recheck before
staging. Do not stop games/processes by name or force termination.

Latest user correction SUPERSEDES leave-games-open behavior: X must FULL EXIT,
closing all managed Minecraft sessions normally, then the manager/controller.
New full-exit source is pending commit/build/staging; installed1cddf59 still leaves
games open. Do not claim full exit fixed in installed App until updated/tested.

## Current full-exit change

app_bootstrap.rs intercepts all product close requests (even no active workspace),
marks surface closing, restores original game windows, then invokes CloseAll on the
original authenticated native_clients_session/session_controller owner. The actor
uses original child handles plus current PID/birth/session/HWND checks, sends only
WM_CLOSE to its own enabled top-level game windows, reaps/saves via existing owner,
and waits up to20s. No Taskkill/TerminateProcess/game injection/profile changes.

A missing compatible controller with no legacy gate returns stopped without
provisioning or spawning anything. Another-build controller is refused, not hidden.
Games still starting, disabled by a dialog, identity mismatch or timeout keep the
manager alive with parented explanation and retryable Close controls. Exit error
resets the actor's exiting flag; successful exit prevents a late queued launch.
Close progress is visible; Alt+F4 uses the same path. Controller naturally exits
when the manager peer exits and no owned game remains. Unrelated games untouched.

Full-exit tests:4 actor tests PASS, including own hidden Windows close fixture:
forged birth rejected, disabled window refused, exact own WM_CLOSE destroys only
the chosen fixture; second window survives. Idle close never prepares stores and
rejects a new launch.35 frontend tests/typecheck0errors0warnings/G0 PASS.
Real Minecraft closure/gameplay NOT_EXECUTED. Do not claim synthetic WM_CLOSE
proves actual game's save/exit behavior. Log:workspace/runs/full-exit-tests.log.

## Installed1cddf59 window improvements and proof

Native own-window probe: SetWindowRgn changes DWM nonclient rendering1->0;
forcing policy enabled still returns0. Product/base now decorations=false;
WindowTitlebar.svelte owns consistent dark draggable frame, accessible minimize,
maximize, close. Main-only Tauri window permissions. Independent top-level games,
no owner/parent/input injection. Winit may retain WS_CAPTION style bits while
handling nonclient layout: those bits alone are NOT a valid frame assertion.
Actual installed client top inset1px and all three custom controls present,
Connected/five enabled in user-reopened PID13660, read-only observation.
Receipts:workspace/runs/interaction-desktop-startup.json and
interaction-frame-readonly.json. First style-bit smoke assertion was invalid;
subsequent control-invocation smoke was refused because user reopened manager.
No actual maximize/restore/close invocation PASS is claimed from that smoke.

Pure Moved events use identity-checked ASYNCWINDOWPOS/NOSIZE and SURFACE try_lock,
without helper roundtrip/game-thread acknowledgement. Contended/invalid geometry
falls back to existing latest-event worker. Resize/activation/checkpoint recovery
retain original helper. Own stalled4s window test proves async follow<100ms.

Separate action names/releases only active exact process. Current-manager separate
slot set prevents automatic recapture; choosing that card explicitly returns it.
Other running sessions stay playable in Play. Preview adoption receives only tile
sources so a desktop session is not parked by another session's preview.
DWM visibility now follows manager visibility/minimization, not foreground focus.
Owned non-topmost/nonactivating destinations stay visible in a visible monitor
panel; transient WouldBlock preserves last preview. Hidden/minimized updates skip
invalid viewport geometry. Preview cap grows from100px high to responsive16vh,
up to180px16:9 fit, dock1600px; compact heights stay smaller. Main16:9 unchanged.

Executed before full-exit extension:35 frontend tests, typecheck, G0, Rust check,
nine surface tests PASS (includes actual offscreen DWM focus/minimize behavior).
Own synthetic Windows cases separately PASS: normal/maximized release, pipe loss,
helper crash, selected release preserving another window, surface close refusing
reacquisition without terminating fixture, retained swap p95=2ms/max3ms.
Combined fixture run once failed recovery timing and contaminated next shared-state
case; fresh isolated tests passed without loosening deadlines. Keep limitation.
Logs:interaction-stall-isolated.log, interaction-recovery-isolated.log,
interaction-surface-tests.log, interaction-rust-check.log under workspace/runs.
None establishes actual Minecraft drag smoothness or multi-account acceptance.

Candidate1cddf59:workspace/packages/lazy-tester/0.1.0-1cddf597505b-20261026T172111066Z.
NSIS SHA25632829a6592209e512d5eb7dd79e90ff106874f76c3b8e99bc13dcdbb1b49b1c3.
Root starter FileDescription verified Lazy Tester - App Launcher. Helper-specific
Task Manager labels still unresolved: exact same executable file identity is used
for pipe authentication. Do not weaken identity checks for cosmetic naming.
Task Manager Apps may omit a closed UI while controller/game remains under
background processes; full exit addresses leftover managed-game lifetime.

## Preserved data and boundaries

Five original profiles migrated opaquely into Data/Launchers/Minecraft-1..5;
original outside-project copies remain recovery material. No repeated setup/migration.
Data/Settings owns preferences/location/lease; Data/Cache owns Temp and actual
WebView through explicit builder data_directory. Five private hives, protected ACL,
zero Data links verified previously. Windows identity/DPAPI, licensed game and OS
prerequisites remain external. Native payloads unchanged:
workspace/builds/central-data-20261026/Release;
compat0f45c07b370ff6c1bf150ccc20879ff6406c208f2b8c3ccdc596b82f2552381e;
enroller51517528886483ba89f1b93b448c1ddd349f26f0e48e5322db434cbfe9c4de99.
Compiler caches36,354,177,582 bytes retained, zero reclaimed. Submoduleabd4c2400...
(actual full SHAabd4c240031cfb46666daa74ca47ed4e48b561e4) and other worktree47a45f9
unchanged. Keep High/Low,144/10 and16:9. Full account clear/export/restore absent.
Signing NOT_VERIFIED; installer execution/gameplay/license/multiplayer NOT_EXECUTED;
publication DISABLED.

Next: commit/build full-exit source, stage when old processes are closed normally,
verify actual X exits idle application tree; real Minecraft full-exit and helper
Task Manager labels remain explicit outstanding work. Overall request ACTIVE.
