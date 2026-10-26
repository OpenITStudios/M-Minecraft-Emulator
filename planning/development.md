# Current Development Handoff — Lazy Tester

Local only; main/release publication untouched; GitHub Actions entirely OFF.
VMware, Sandboxie and RAM/renderer experiments remain PAUSED.

## Current bounded outcome — comfortable game proportions

Parent efce0be722ed6251a1c0803c1d74b15c9ad13845. The user reports the staged
application now runs normally and approved a comfortable largest-fit 16:9 layout.
This is user-reported operation, not independent gameplay/license acceptance.

LazyTesterApp.svelte now centers the existing native viewport inside a measured
stage. The viewport fits both available dimensions at 16:9; the same DOM rectangle
still supplies native placement, aperture, input and preview IPC. Observing the
outer stage also catches center-position changes when the game is width-limited.
No second geometry authority, bitmap stretching or game-renderer change is added.
The duplicate Play/stage headings are combined, leaving more height for the game.
The bottom five-session dock stays visible, caps its width at 1320 logical pixels,
and uses 16:9 preview destinations with clearer Playing/background status.
Body and card text are larger; existing Windows DPI conversion is retained.

The initial normal window targets 1280x900 logical pixels, without auto-maximizing.
app_bootstrap.rs bounds its physical client size and minimum against the current
monitor work area, decorations, taskbar and DPI, and centers the outer window.
Normal Windows maximize remains available. Window history and a separate UI-scale
preference are not implemented in this bounded change; Minecraft's own UI scale
and all graphics/FPS/runtime data settings are preserved.

Typecheck, 33 frontend tests and production frontend build PASS. Structure/G0 and
Lazy Tester installer source contract PASS. The targeted Rust startup-size test
PASS for large/small work areas and 150% DPI; no live game is touched by that test.
Compiled-frontend regression checks largest-fit 16:9, centering, no cropping,
bounded dock and proportional previews across seven window sizes, including
3440x1440 and a tall window. Native IPC uses the visible rectangle; browser DPI
emulation verifies 125/150% conversion inputs. These are mocked native checks,
not real Windows mouse/game acceptance. Existing 30-second quiet-polling and
settings/setup/session-switch regressions remain required.

Clean product source: 835f40d25a6017a6c5012d37defa15779a42ef43 on Local.
Windows release/NSIS construction and guarded App staging completed with exit0.
No Lazy Tester process was running when inspected; no user process was terminated.
All four staged payload hashes and the exported installer hash were verified.
Desktop/Start Menu shortcuts were created and verified by the existing helper.
App now contains this source revision; prior App and preferences are preserved
by the guarded stager. The existing screenshot-era sessions were not controlled.

Candidate: workspace/packages/lazy-tester/0.1.0-835f40d25a60-20261026T134529139Z.
Installer SHA256: e822d5b32a6587527b11d2a6ce027853cda952f60e49413c7b8d40ba36c7e532.
Build log: workspace/runs/play-settings-implementation/comfortable-layout-package.log.
Signing NOT_VERIFIED; installer execution and new-layout live game interaction
NOT_EXECUTED; publication DISABLED. The existing submodule and second worktree
were checked and remain unchanged. This handoff-only update follows the product
commit; no source changes were made after that clean package build.

## Previous outcome — quiet polling, honest setup and normal shortcuts

Parent607294376a96a603e8dfe5181729c335d6c63b03. User reported the staged UI was
blocked at setup, directed them to RUN.cmd and flickered during automatic checks.
They approved diagnosis followed by repair; no account/world replacement implied.

The compiled frontend reproduced the exact issue before edits: unconfigured setup
displayed Ready when you are; over6.5seconds the probe saw recurring Refresh/help
messages and launcher disabled transitions. The original loop used the same busy
feedback for two-second background inspection and manual Refresh, marked geometry
dirty after unchanged snapshots, and exposed resizing status as a changing footer.

The existing Svelte owner now separates manual feedback from internal inspection.
Automatic polling starts only after binding is configured. Unchanged process/state
snapshots do not update native geometry. Cards are not disabled by background
inspection; launch/navigation/manual-refresh requests queue behind it. Setup state
never claims Ready and exposes explicit setup/reconnect actions. The stable help
text no longer flashes Updating display. Existing actor/identity/launch guards stay.

Profile binding status now distinguishes missing connection metadata from a saved
store that cannot reopen. Reopen errors identify the stage/slot without raw
identities or account contents. The fixed --lazy-tester-inspect-setup mode only
checks existing product/legacy metadata. Its product lookup does not create
profiles or persist a locator. It exits before UI/controller startup.

Actual metadata checks found the reviewed existing connection, all five launcher
directories and registry stores intact. The new read-only backend inspection
returned configured=true, issue=null. Default Data remained empty and no App
locator was created by diagnostics. This is binding evidence, not sign-in/license
or independent gameplay acceptance. No profiles were deleted, reset or migrated.
The original screenshot's complete machine-specific transient failure was not
recreated; the stale/blocked readiness presentation and its recovery were reproduced.

Local packaging now invokes create-app-shortcuts.ps1 after guarded App staging.
It verifies the executable against build-info, preflights both destinations,
preserves unrelated shortcuts, and creates Desktop and Start Menu Lazy Tester.lnk
with the direct executable, App working directory and embedded LT icon. It verifies
all fields after Save; it never launches a GUI/game or installs a service.

## Executed checks

Svelte: zero errors/warnings;33 frontend tests PASS; production frontend build
PASS. New compiled-Svelte regression observes30seconds of stable background
polling with zero control/label/native-geometry churn, plus queued launch and
navigation, truthful setup, deliberate Refresh recovery, existing-data reconnect,
offline recovery, High/Low/FPS and separate-window return (mocked Tauri only).
Five targeted profile-binding Rust tests PASS, including the new missing-metadata
versus missing-store classifier test. Actual Windows debug binary built and its
read-only setup inspection succeeded. Shortcut PlanOnly preflight succeeded.
Logs and synthetic screenshots remain ignored under workspace/runs/.

## Previous App/package state and remaining work (superseded above)

App now contains clean product sourcef7f42fb067ec13a5d92f92b190c0988c1ea618a5.
Windows/NSIS construction, guarded staging and Desktop/Start Menu shortcut creation
completed successfully. All four App payload hashes and installer hash were checked.
Candidate: workspace/packages/lazy-tester/0.1.0-f7f42fb067ec-20261026T132654576Z.
Installer SHA256d45d33f200c9af28409f09c3bb347579a3a594bd42a483d4169aa92f3bf372f0.
Log: workspace/runs/play-settings-implementation/startup-package-final.log.

The first shortcut attempt failed after successful staging because Windows
PowerShell lacked Get-FileHash. The helper now uses .NET SHA256 and was tested
with the exact powershell.exe host. The final complete package command exited0;
both shortcuts' target, working directory, arguments and icon were verified.
Desktop: C:/Users/Administrator/Desktop/Lazy Tester.lnk.
Start Menu: Programs/Lazy Tester/Lazy Tester.lnk under the user's Start Menu.

The staged executable's fixed read-only setup mode returned configured=true,
issue=null and exit0. Its receipt is ignored at
workspace/runs/play-settings-implementation/staged-setup-status.json.
Then the Desktop shortcut was actually opened. The expected App/lazy-tester.exe
process appeared and Responding was true. This proves shell launch, not visual
acceptance or game readiness. The user-facing app was left open; no Minecraft
process was started, account UI automated, or account/world contents read.
No user process was terminated for staging. Earlier app process had exited before
the guarded build/stage; all data and preference pointers remained preserved.
Submodule and second worktree remain unchanged. This final handoff update changes
documentation only; the product source SHA above remains the exact App source.

Clear All Data, world export, settings reset/restore, background audio, automatic
game shutdown and data migration remain unavailable. Preserve all prior data and
preferences, native payload pins, initialized submodule and second worktree.
NSIS construction is not installer acceptance. No public release or CI operation.

Next: verify the new 16:9 layout through the normal shortcut on the user's display,
including maximize/restore and mouse alignment during normal session switching.
Real game/DPI interaction, gameplay/license and multiplayer acceptance remain
NOT_EXECUTED for the new layout revision.
