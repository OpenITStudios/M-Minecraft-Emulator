# Current Development Handoff — Lazy Tester

Local only; main/release publication untouched; GitHub Actions entirely OFF.
VMware, Sandboxie and RAM/renderer experiments remain PAUSED.

## Current bounded outcome — quiet polling, honest setup and normal shortcuts

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

## App/package state and remaining work

Before this bounded update, App remained clean source
1c8a3cd129cebf41472facb25021ed6011bc70a9. Build/stage a clean new source revision,
recheck all relevant processes and record exact new payload/installer/shortcut
verification after completion. Do not terminate a game/controller to force staging.

Clear All Data, world export, settings reset/restore, background audio, automatic
game shutdown and data migration remain unavailable. Preserve all prior data and
preferences, native payload pins, initialized submodule and second worktree.
NSIS construction is not installer acceptance. No public release or CI operation.

Next: after guarded staging, verify startup through the normal shortcut and then
observe real two-session use. Microsoft sign-in remains manual; gameplay/license,
native flicker and multiplayer acceptance are NOT_EXECUTED for this revision.
