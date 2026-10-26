# Current Development Handoff — Lazy Tester

Local only; main/publication untouched; GitHub Actions entirely OFF.
VMware, Sandboxie and RAM/renderer experiments remain PAUSED.

## Current bounded repair — registry reopening and real UI startup proof

Parent 8d407a65da824182a4d3a4da0a692b44b9eebf64. The user correctly rejected the
previous completion claim: normal App still showed Minecraft - 1: saved registry
store cannot be opened (Windows error 2). CLI initialization alone was insufficient.
Do not equate earlier packaging/CLI checks with a functioning normal UI.

Confirmed code defect: user_registry_path used RegOpenCurrentUser but existing
store reopening used process-wide predefined HKEY_CURRENT_USER. A disposable
child-process regression maps the predefined key to an empty fixture. The real
old store-opening function then fails with the exact Windows error 2 despite
an existing store. The test was executed RED before the fix and GREEN after it.
The precise component/trigger affecting the earlier user's process is not traced;
the test proves the inconsistent-root failure mechanism, not that attribution.

Native independent_bootstrap.rs now opens explicit current-user registry handles
for store creation/reopening. Legacy replay and product_profiles.rs use one
open_existing_registry_store function; no missing-store create/reset fallback is
added. Directory identities, owner/catalog checks, payload pins, permissions and
native launch admission stay unchanged. No account/world contents are read,
copied or reset. No registry migration or public wire change is introduced.

## Stronger verification

Product-only --lazy-tester-check-webview runs actual compiled Svelte inside a
hidden real Tauri/WebView, using the real IPC startup sequence and normal window
initializers. It registers no game presentation handlers and rejects import,
location changes and session requests other than Inspect. The UI reports rendered
connection status and card/enable counts through startup_diagnostic_result.
This observational stdout is not a readiness store or launch authority. Normal
startup also emits that bounded observation without exiting, enabling verification
of the actual visible App process. No synthetic binding result is substituted.

Local package delivery now verifies both native initialization and the real UI:
Connected, five cards, five enabled controls, no connection issue. Export-only
builds do not run runtime initialization. Installer identity/hooks remain unchanged.
Prior hidden real-UI probes (before the root fix) also sometimes passed; they do
not retroactively establish the failed old GUI healthy. Normal App must be checked.

Executed: 35 frontend tests; Svelte check and production build; nine targeted Rust
tests (new registry-root child fixture, five binding tests, three product-store
preservation/reopen tests) PASS. Compiled browser regression PASS including startup
retries, seven-size 16:9 layout, DPI IPC, 30-second stable polling and recovery.
Logs remain ignored under workspace/runs/play-settings-implementation/.
Clean candidate and actual App startup observations will be recorded after build.

## Existing App and preservation

App before this repair contains d2399c0acf7053a71b238a7136a95e3376925ce1.
Its previous initialization succeeded but user-provided normal UI still failed;
that revision is not accepted as fixed. Existing normal Desktop/Start Menu links
point directly to App/lazy-tester.exe with App working directory and embedded icon.
Canonical guarded staging preserves the old App and all data/preference pointers.

Keep existing 16:9 largest-fit viewport, same native/input rectangle, bottom dock
capped at1320 logical pixels and normal1280x900 work-area/DPI-bounded initial window.
General owns High/Low and configurable144/10 defaults. Manual graphics changes and
cache cleanup require all games stopped; new FPS limits apply next launch.
Window history and a separate interface-scale preference are not implemented.

Clear All Data, world export, settings reset/restore, background audio, automatic
game shutdown and data migration remain unavailable. Do not revive or simulate them.
Preserve initialized submodule, second worktree, accounts/worlds, toolchains and
native payload pins. No game/installer GUI or account sign-in is automated.
Signing remains NOT_VERIFIED; installer execution/gameplay/license/multiplayer
acceptance remain NOT_EXECUTED. Publication remains DISABLED.

Next: build the clean Windows candidate, replace App after normal manager closure,
then verify real hidden and normal visible App startup before reporting completion.
