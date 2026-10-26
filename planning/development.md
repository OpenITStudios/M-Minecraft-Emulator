# Current Development Handoff — Lazy Tester

Local only; main/publication untouched; CI OFF. Legacy VM, Sandboxie and
RAM/renderer experiments PAUSED. User explicitly chose to RETAIN compiler caches;
the previous deletion blocker is no longer an active task. Delete nothing.

## Current request — 27 October 2026

Repair classic-looking border in Play, game drag latency, per-launcher separate
window behavior, Close while games run, disappearing previews, preview size and
ambiguous Task Manager names. Base source36bbf2e7e621e0edc728c54113d7939d699e798a.
Installed App still04b5c52498393492fed6b734a49f00f36004d0a7. Live old manager17848,
controller36188, games19336/42248 and recovery helpers were observed; never terminate
user games. Their exact current presence must be rechecked before staging.

## Current source changes

Canonical window presentation owner remains workspace_surface.rs. A native own
window probe proves SetWindowRgn changes DWM nonclient rendering from1 to0;
forcing DWM policy enabled still returns0. Preserve independent top-level games,
no parenting/ownership/input injection. Product/base config now decorations=false;
WindowTitlebar.svelte provides a consistent draggable dark frame and accessible
minimize/maximize/close controls in Play and Settings. Minimal window permissions
are restricted to main. Native region composition remains unchanged.

Pure WindowEvent::Moved calls identity-checked SetWindowPos ASYNCWINDOWPOS/NOSIZE
through a nonblocking SURFACE try_lock. No helper-pipe roundtrip or game-thread
acknowledgement in the normal drag path. If contended/temporarily invalid, existing
latest-event worker receives fallback so final movement is not discarded. Resize,
activation, checkpoint restoration and original process identity checks remain.

Close no longer waits for an unparented Yes/No dialog. It releases original windows
then destroys the manager, leaving Minecraft running. The same surface owner marks
closing before recovery so frontend reflows cannot reacquire a session during exit.
Recovery failure keeps manager alive and shows a parented error dialog; no game kill.

Separate-window action now names and releases only the selected exact process.
Frontend tracks separate slots for the current manager session and does not auto
recapture them. Other running sessions remain selectable here; explicitly choosing
a separate session returns it to Play. Preview adoption uses only requested tiles,
so excluded desktop sessions cannot be parked incidentally. No persistent setup flag.

DWM preview visibility now follows manager visibility/minimization, not input focus.
Destinations remain owned, non-topmost, nonactivating windows. Focus elsewhere must
not blank a visible monitoring panel. Hide no longer requires valid minimized
viewport geometry. Transient WouldBlock while moving preserves previous preview.
Preview width cap grows from100px-high to a responsive16vh/up-to180px-high16:9 fit,
within five cards and1600px dock. Compact heights retain a smaller dock. Main viewport
remains largest-fit16:9. No capture/encoding loop or extra polling.

Root starter now has FileDescription Lazy Tester - App Launcher. IMPORTANT: helper
Task Manager display names are NOT yet individually renamed. They intentionally
execute the same image; the controller pipe authenticates exact executable file
identity. Generic per-image descriptions must not be presented as fixed per-role
labels. Preserve that security contract when addressing the remaining naming work.

## Executed evidence / limitations

Typecheck0errors/0warnings;35 frontend tests PASS including separate-slot selection.
Rust all-target check PASS before the final test-only helper assertion (which then
compiled and passed in the isolated test). Nine workspace surface tests PASS,
including actual offscreen DWM registration, visibility without foreground and
minimize/hide. Actual own synthetic Windows fixtures separately PASS:
- stalled_window_reports...: async follow under100ms while synthetic game stalls4s;
  timeout recovery preserves game and original styles/geometry.
- actual_surface_helper_restores...: release/pipe loss/helper crash, normal/maximized,
  targeted release preserves another window, final close preserves game and denies
  reacquisition; retained switching p95=2ms/max3ms in this run.
Combined fixture run earlier failed timing/recovery and contaminated following
shared-state test; do not erase that limitation. Fresh isolated runs passed without
weakening deadlines. Rebuild current helper before rerunning synthetic cases.
Logs: workspace/runs/interaction-surface-tests.log, interaction-stall-isolated.log,
interaction-recovery-isolated.log, interaction-rust-check.log and frame-probe.cs.
No game was launched, signed into or controlled by these tests. They are not proof
of actual Minecraft drag smoothness or final release UI acceptance.

Package/build/staging for this source remains pending. Never claim user App updated
until exact-source receipt and real Explorer verification. The running old game
controller must stop normally before replacing its executable. Preserve all Data.

## Retained project-local startup/data repair

Installed04b5c52 was verified across three real Explorer -> root starter -> App
starts: Connected, five enabled Launch controls, cacheInsideData=true. Five App/root
hashes match receipt, five private HKCU.dat files, zero Data links, protected ACL.
Receipts: workspace/runs/project-desktop-cache-final-1.json,
project-desktop-cache-final-2.json and project-local-final-integrity.json.
Five original profiles migrated opaquely to Data/Launchers/Minecraft-1..5; original
outside-project copies preserved as recovery, not active dependencies. No repeated
setup/migration. Data/Settings owns preferences/location/lease; Data/Cache owns
Temp and actual WebView path through explicit WebviewWindowBuilder.data_directory.
Windows identity/DPAPI, licensed game and OS prerequisites remain external.
Native payloads unchanged under workspace/builds/central-data-20261026/Release:
compat0f45c07b370ff6c1bf150ccc20879ff6406c208f2b8c3ccdc596b82f2552381e;
enroller51517528886483ba89f1b93b448c1ddd349f26f0e48e5322db434cbfe9c4de99.

Compiler caches36,354,177,582 bytes retained by latest user decision under
workspace/archive/compiler-cache-20261026/{desktop-target,runtime-target} and
workspace/builds/lazy-tester-layout-20261026/target. No bytes reclaimed. Submodule
abd4c240031cfb46666daa74ca47ed4e48b561e4 and second worktree47a45f9 unchanged.
Keep High/Low and144/10. Full account clear/export/restore absent. Signing NOT_VERIFIED;
installer execution/gameplay/license/multiplayer NOT_EXECUTED. Publication DISABLED.

Next: finish source checks/build an exact clean candidate, then arrange normal
closure for staging and actual release UI verification; helper-role naming remains
explicitly unresolved. Overall user request ACTIVE, not DONE.
