# Current Development Handoff — Lazy Tester

## Single active goal — user revised27October2026

Deliver a dependable Lazy Tester: same Play/Settings frame; no stuck/misaligned game
or exposed desktop during drag; measured faster switching/Launch; clear process
ownership; functional Settings/Data Storage. Defaults120FPS active/5 background,
minimum newly configurable5FPS, background<=active. Goal ACTIVE, not complete.

Local only; main/publication untouched; CI OFF. VM/Sandboxie and RAM/renderer work
PAUSED. Compiler caches explicitly RETAINED. No incidental cleanup or actual profile
wipe. Clear all data is a product feature tested with synthetic fixtures; actual use
requires selected-launcher review/confirmation.

## Acceptance — preserve all outcomes

1. Stable presentation: correct the native composition/coordination causing drag
   snag and desktop/Explorer exposure. Same Play/Settings frame through drag end/
   cancel, resize, maximize/restore, focus and DPI/monitor changes. Main game16:9.
   No game parenting, binary patching, destructive owner relationship or input relay.
2. Fast switch/Launch: measure request -> actual focus/bounds and launch phases
   (controller, validation, enrollment, window ready). Reuse verified original leases
   and immutable image pins; remove duplicate work without weakening game/DLL/
   process identity or inventing readiness. No busy polling or unsupported claims.
3. Process clarity: main UI, root starter, session controller, recovery per game,
   official Minecraft, WebView renderer/GPU/services. Investigate supported Task
   Manager naming/grouping and prove actual results. Do not hide processes, rename
   official binaries, disable browser sandbox or weaken same-image pipe checks.
   An app role/PID page is not proof of actual Task Manager grouping. Explain any
   proven upstream limitation; do not attribute LazyDesigner without PID evidence.
4. General: High/Low only; truthful valid whole5..1000 FPS saves with120/5 defaults.
   Preserve explicit older preferences; new defaults do not overwrite user choices.
   Keep/Mute background audio must actually scope original game audio sessions,
   never other apps. No ignored selections or placeholder controls as features.
5. Data Storage per launcher: actual sizes/path; clear cache for eligible selected
   launcher with specific blocked reason, allowing proven unrelated managed games.
   Complete settings reset plus verified backup/restore via existing writer. Clear
   all data removes only selected owned local accounts/world/cache/settings/backups,
   leaves empty launcher ready for a new account, retains installation/others, states
   Windows identity limits. Define confirmation, recovery/catalog identity updates
   and interruption handling. Test active profile, malformed paths, links, cancel,
   partial failure and cross-profile isolation before real destructive use.
6. Delivery: clean exact Local Windows candidate, frontend/Rust/fixtures/package
   checks; stage App/root EXE/shortcuts preserving Data. Verify actual Explorer
   startup, common frame, drag/switch/previews/full exit/settings/storage/process
   roles to available proof. Compilation never substitutes for usability acceptance.

## Current exact source and delivery

Installed App/root source35ed4be6d9ae189cc3ec6dd0d6bb90b0533e4358 (Local clean at
build). Product source changes:2c9e284 opaque presentation;1262efb FPS defaults;
35ed4be selected-launcher cache admission. App/lazy-tester.exe SHA256:
ae4e8677d4f4065812863ebe0396f697df0cb5ce861fa7c1125465436043e8b6.
Root starter hash is recorded and verified in App/build-info.json.
Candidate workspace/packages/lazy-tester/0.1.0-35ed4be6d9ae-20261026T181321305Z;
NSIS2651492 bytes SHA256:
8b8e71ac97ef3c4f043878c8d584cac875ccf806f4c13e9dea7fd6b0ba9da37e.
Signing NOT_VERIFIED; installer execution/publication NOT_EXECUTED/DISABLED.

Actual Explorer startup after staging: source35ed4be, manager20252/root38940,
Explorer ancestry true, Connected, five enabled Launch cards, no issue, WebView
cache inside Data. Verifier closed only its own idle manager normally; subsequent
process inventory showed zero manager/starter/games. Recheck before next staging;
never terminate by name. Data and five profiles retained, no repeated setup.

## Implemented corrections and executed evidence

Presentation: workspace_surface.rs keeps host opaque; existing previews.rs worker
adds unfocused primary DWM image. Selection removes image before activating exact
native HWND. Background parking, original identity/checkpoints/recovery retained;
no input relay. Foreground/move-size events reconcile via existing coalesced worker.
RED->GREEN stalled-game opacity fixture, nine surface tests, DWM primary translation/
minimize/owner assertions, focus routing and isolated stall/recovery tests PASS.
Normal/maximized release/pipe-loss/helper-crash PASS. Forty synthetic swaps p95/max
2ms; not actual Minecraft latency. Logs workspace/runs/opaque-*. Actual game drag/
focus/frame/switch/DPI acceptance NOT_EXECUTED.

FPS:120/5 defaults, saves5..1000, background<=active across UI/bridge/backend.
Historical explicit1..4 and144/10 readable without rewriting. V3 pacer gets saved
values next launch; historical V2 defaults unchanged. Rust default/roundtrip/legacy/
boundaries PASS; actual120/5 game pacing NOT_EXECUTED.

Cache: selected pinned profile must be stopped; all retained game identities checked.
Other exact managed games may continue; unknown external game blocks. Actor still
serializes launch/deletion; pinned paths/exclusive HTTP-cache plans unchanged.
UI/parser/classifier agree. Four synthetic deletion tests and scoped PID/birth spoof
admission PASS, including locks/links/incomplete plan/cross-profile preservation.
No real cache/account/world deleted.35 frontend tests, typecheck0errors/warnings,
G0/structure PASS. NSIS built, export hash checked, App/root startup receipts verified.
Existing Rust warnings remain.
## Remaining work and owners

Full clear/reset/backup/restore/audio controls remain incomplete. Use the existing
session_controller.rs actor, game_preset.rs options writer, profile_storage.rs/
product_profiles.rs pinned stores, manager_settings.rs and current UI/bridge.
Launch performance and actual Task Manager roles/grouping remain unproven.
WebView2 grouping matches OPEN upstream reports; AppUserModelID/metadata alone
insufficient. Investigate actual parent/logon/integrity and supported APIs:
https://github.com/MicrosoftEdge/WebView2Feedback/issues/5628
https://github.com/tauri-apps/tauri/issues/15567
DWM destination must be caller-owned top-level, not child:
https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/nf-dwmapi-dwmregisterthumbnail
No one-group Task Manager success claim. Full-exit source from6eba183 closes only
managed games normally and keeps manager open on pending dialog/timeout; actual
Minecraft save/full-exit acceptance remains NOT_EXECUTED.

Five profiles already migrated opaquely into Data/Launchers/Minecraft-1..5; original
external stores remain recovery only, never new active dependencies. Data/Settings
owns preferences/location/lease, Data/Cache owns Temp/WebView. Private hives, ACLs,
reviewed DLL pins unchanged (App/build-info.json). Do not read account/world data.
Submoduleabd4c240031cfb46666daa74ca47ed4e48b561e4; second worktree detached47a45f9c7d4a8ce6799d56997fd38e4b4df49ce7 unchanged.
Four licensed playable clients, gameplay/network/migration/installer acceptance
are separate obligations, NOT_EXECUTED here. Compiler caches remain ignored.

Next: implement settings reset/backup/restore through the existing options writer,
with synthetic path/link/interruption tests, then full clear/audio and remaining
actual presentation/performance/process acceptance. Overall goal stays ACTIVE.
