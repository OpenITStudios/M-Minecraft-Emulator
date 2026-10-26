# Current Development Handoff — Lazy Tester

## Single active goal — revised by user27October2026

Deliver a dependable Lazy Tester: same Play/Settings frame; no stuck/misaligned game
or exposed desktop during drag; measured faster switching/Launch; clear process
ownership; real scoped Settings/Data Storage actions. New defaults120FPS active /
5FPS background, minimum configurable5FPS, background<=active. Supersedes144/10 for
new defaults; do not silently replace arbitrary saved user preferences. Goals tool
remains ACTIVE; this file is the sole detailed scope/evidence/next-action handoff.

Local only; main/publication untouched; CI OFF. VM/Sandboxie and RAM/renderer work
PAUSED. Compiler caches explicitly RETAINED; delete nothing as cleanup. Clear all
data is a product feature to implement/test on synthetic data, not permission to
wipe actual user profiles during development. Actual destructive use needs review
and confirmation for the selected launcher in the app.

## Ordered outcomes and acceptance

1. Stable presentation. Reproduce drag snag: game rectangle lags manager, aperture
   exposes desktop/Explorer. Correct composition/coordination, not just CSS or event
   frequency. Same frame in Play/Settings through drag end/cancel, resize, maximize,
   restore, focus and DPI/monitor changes. Main game stays16:9. No destructive game
   reparenting, game-binary modification or input injection.
2. Fast switch/Launch. Measure UI request -> actual focus/bounds; separate launch
   phases (controller, validation, enrollment, window ready). Reuse original verified
   leases/immutable pinned image safely; remove duplicate work/inspection queues.
   Preserve game/DLL verification and original process identity. No fake readiness,
   unsupported timing claims, extra busy polling or weakening isolation for speed.
3. Process clarity. Identify main UI, starter, session controller, recovery per game,
   official Minecraft and WebView renderer/GPU/services. Investigate supported
   Task Manager grouping and prove local result before claiming one group. Do not
   hide/rename official binaries, disable browser sandbox, merge unrelated apps or
   weaken same-image pipe authentication. A role/PID explanation is not proof of
   actual Task Manager naming/grouping. Explain any proven upstream limitation.
4. Functional General. High/Low only. Save/apply valid whole5..1000 FPS (default120/5)
   with truthful application timing. Background audio must really support Keep audio
   and Mute background, scoped to original games/audio sessions, never other apps.
   No ignored dropdown values or placeholder actions presented as features.
5. Functional Data Storage per launcher. Actual sizes/path and cache cleanup for
   eligible selected launcher with specific blocked reason. Verify whether stopped
   launcher4 can safely clean while other games run; avoid unnecessary blanket gate.
   Implement settings reset plus verified settings backup/restore through existing
   writer. Clear all data leaves a fresh empty launcher, removes only its owned local
   account/world/cache/settings/backup data, preserves installed game/other launchers,
   explains external Windows identity limits. Define confirmation, optional recovery,
   catalog/identity updates and interruption behavior. Test running profile, malformed
   paths, junctions, cancellation, partial failure and cross-profile isolation before
   enabling real destructive use. A permanently disabled button is not completion.
6. Delivery. Build clean exact Local candidate, check frontend/Rust/fixtures/package,
   stage App/root EXE/shortcuts preserving Data. Verify real Explorer startup, common
   frame, drag, switching, previews, full exit, functional settings/storage and process
   ownership to available proof. Static/synthetic/Windows/Minecraft evidence remain
   separate; successful compilation is not usability acceptance.

## Exact source/installed state and user evidence

Source HEAD at revised-goal start91e82526a0bd161b9869cb3bd85ee501356d6984, Local clean.
Installed App/root source6eba183ff437c4da410acd5e308a0b28864d2d90. Last observed
manager43284/root20804, controller18816, games3564/42036, helpers34184/8128. Recheck IDs;
never kill by name. User confirms unclear Task Manager ownership WHILE UI OPEN.
Screenshots show Lazy Tester four identical names, two separate Minecraft Education
rows, WebView2 Manager six entries (Lazy Tester renderer/GPU/services). No evidence
this is LazyDesigner; attribute exact PIDs/parents/paths. User still sees different
Play/Settings border and slow switching/launch. Drag screenshot proves a displaced
game exposing Explorer/desktop. Prior synthetic PASS is insufficient. Disabled
cache/clear/reset/restore/audio controls are now explicitly in scope to complete.

## Canonical owners and investigation

Presentation: workspace_surface.rs and workspace_surface/previews.rs; main events/
full exit: app_bootstrap.rs; UI: LazyTesterApp.svelte/WindowTitlebar.svelte; transport:
native_clients_session.rs/pipe.rs; lifecycle: independent_bootstrap/session_controller.rs;
settings/storage: manager_settings.rs, game_preset.rs, profile_storage.rs. No second
lifecycle/readiness/credentials owner.

Presentation source correction27October (based on17acc411): remove region holes;
retain opaque host and DWM frame. Existing preview worker gains unfocused primary
image; explicit selection removes it before native focus. Independent game parks
while manager owns input; same retained identity/recovery helper, no input relay.
Focus/move-size boundaries reconcile through existing worker. Executed RED then
GREEN stalled-drag opacity test; nine surface tests, native DWM primary translation/
minimize test and focus-event test PASS. Isolated stall/recovery/pipe-loss/helper-
crash fixtures PASS;40 synthetic retained swaps p95/max2ms. Logs: workspace/runs/
opaque-*.log. Actual game drag/focus/frame acceptance NOT_EXECUTED; not staged.

WebView2 grouping matches OPEN upstream reports: AppUserModelID/EXE metadata alone
was insufficient. Investigate actual parent/logon/integrity; do not call locally fixed:
https://github.com/MicrosoftEdge/WebView2Feedback/issues/5628
https://github.com/tauri-apps/tauri/issues/15567
https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmregisterthumbnail

## Previous evidence — revision6eba183 only

Explorer Connected/five enabled/project cache; Settings/maximize/restore; idle X
exits with zero remaining App processes85ms; payload hashes PASS.35 frontend tests,
typecheck/G0, nine surface tests, four scoped synthetic close tests PASS. Isolated
recovery/stall tests passed; one combined run failed recovery timing. Logs remain
in workspace/runs. These do not prove actual-game drag/exit/save/audio/storage.
Source still implements144/10, minimum1 until changed/tested.

## Preservation and delivery boundary

Five profiles already migrated opaquely into Data/Launchers/Minecraft-1..5; outside
originals remain recovery only. No repeated setup/migration. Data/Settings owns
preferences/location/lease; Data/Cache owns Temp/actual WebView. Five private hives,
protected ACL, zero Data links verified previously. Licensed game/Windows identity
remain external. Keep private/account/world material out of Git and diagnostics.
Reviewed DLLs unchanged in workspace/builds/central-data-20261026/Release;
pins remain in independent_bootstrap.rs and App/build-info.json.
Submoduleabd4c240031cfb46666daa74ca47ed4e48b561e4 and second worktree47a45f9 preserved.
Signing NOT_VERIFIED; installer execution, real Minecraft full-exit/save, audio and
multi-client acceptance NOT_EXECUTED. Publication DISABLED.

Next: update120/5 defaults and validation, then complete settings/storage/audio;
build a clean candidate and verify presentation before replacing installed App.
Continue through ordered outcomes; overall goal ACTIVE, not complete.
