# Current Development Handoff — Lazy Tester

## Single active goal — user revised27October2026

Dependable Lazy Tester: stable Play/Settings/game windows, faster switch/Launch,
clear process roles, functional Settings/Data Storage. Defaults120/5, minimum5FPS,
background<=active. Goal ACTIVE, not complete.

Local only; main/publication untouched; CI OFF. VM/Sandboxie and RAM/renderer work
PAUSED. Compiler caches explicitly RETAINED. No incidental cleanup or actual profile
wipe. Clear all data is a product feature tested with synthetic fixtures; actual use
requires selected-launcher review/confirmation.

## Acceptance — preserve all outcomes

1. Stable presentation: correct the native composition/coordination causing drag
   snag and desktop/Explorer exposure. Same Play/Settings frame through drag end/
   cancel, resize, maximize/restore, focus and DPI/monitor changes. Main game16:9.
   No game parenting, binary patching, destructive owner relationship or input relay.
2. Fast switch/Launch: measure focus/bounds and controller/validation/enrollment/
   window-ready phases. Reuse original leases/image pins without weakening identity,
   inventing readiness or busy polling.
3. Process clarity: main UI, root starter, session controller, recovery per game,
   official Minecraft, WebView renderer/GPU/services. Investigate supported Task
   Manager naming/grouping and prove actual results. Do not hide processes, rename
   official binaries, disable browser sandbox or weaken same-image pipe checks.
   Role/PID pages do not prove grouping; explain proven upstream limitations.
4. General: High/Low only; truthful valid whole5..1000 FPS saves with120/5 defaults.
   Preserve explicit older preferences; new defaults do not overwrite user choices.
   Keep/Mute background audio must actually scope original game audio sessions,
   never other apps. No ignored selections or placeholder controls as features.
5. Data Storage per launcher: actual sizes/path; clear cache for eligible selected
   launcher with specific blocked reason, allowing proven unrelated managed games.
   Complete settings reset plus verified backup/restore via existing writer. Clear
   all data removes only selected owned local accounts/world/cache/settings/backups,
   leaves empty launcher ready for a new account, retains installation/others, states
   Windows identity limits. Verify confirmation, recovery/catalog identities,
   interruption, active profiles, malformed paths, links, cancel and peer isolation.
6. Delivery: clean exact Local Windows candidate, frontend/Rust/fixtures/package
   checks; stage App/root EXE/shortcuts preserving Data. Verify actual Explorer
   startup, common frame, drag/switch/previews/full exit/settings/storage/process
   roles to available proof. Compilation never substitutes for usability acceptance.

## Current exact source and delivery

Installed App/root source7eb8ca1d490d32a70a54bd6d1ed2302086a269bd, clean Local build.
App SHA256:2658f0e8a031902f2dbcd9e91a48c73ee10845c7af23246fff0feb0ee06e219a.
Root starter/DLL hashes: App/build-info.json, verified during Explorer startup.
Candidate workspace/packages/lazy-tester/0.1.0-7eb8ca1d490d-20261026T182535133Z;
NSIS2663915bytes SHA256:
88b1d893ea9001ba559370bcba210c46f6f8ea6eff457189c6d90ba6f93883bc.
Signing NOT_VERIFIED; installer execution/publication NOT_EXECUTED/DISABLED.
Actual Explorer startup: Connected/five enabled/no issue/project-local WebView cache.
Real UI PID37400: Settings->Data Storage->Advanced->Reset review->Cancel PASS;
review closed, options metadata unchanged, no real reset. Owned idle UI closed
normally; zero remaining app/game processes. Log workspace/runs/settings-recovery-
ui-smoke.log and .ps1. Data retained; no repeated setup. Recheck before staging.
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
No real cache/account/world deleted. Existing Rust warnings remain.
## Remaining work and owners

Reset/restore source added: game_preset.rs backs up options.txt, flushes/readbacks,
publishes SHA256 receipt before atomic replacement; restore backs up current state.
Stopped-profile admission and review wired through existing actor/bridge/UI. Eight
preset/recovery fixtures PASS. No real settings mutation; review/Cancel proof above.
Full clear implemented in product_profiles/clear.rs through the same actor/bridge/UI:
fresh selected hive/directory, generation-bound confirmation, same-volume cutover,
catalog identity update, pinned-handle purge. Precommit rollback; postcommit cleanup
requires explicit resume. Read-only inspection never mutates; startup never performs
a large committed purge. Name+ack confirmation; one review panel at a time.
13 profile tests (7 transactional clear fixtures),5 controller tests, invalid request
admission,37 frontend tests/typecheck PASS. Locked/hardlinked/junction/malformed paths,
cutover failures, partial purge, stale token, readonly files and peer preservation
covered with synthetic data. No real full clear executed. Candidate not yet staged.
Audio remains incomplete. Use the existing
session_controller.rs actor, game_preset.rs options writer, profile_storage.rs/
product_profiles.rs pinned stores, manager_settings.rs and current UI/bridge.
Launch performance and actual Task Manager roles/grouping remain unproven.
WebView2 grouping matches OPEN upstream reports; AppUserModelID/metadata alone
insufficient. Investigate actual parent/logon/integrity and supported APIs:
https://github.com/MicrosoftEdge/WebView2Feedback/issues/5628
https://github.com/tauri-apps/tauri/issues/15567
No one-group Task Manager success claim. Full-exit source from6eba183 closes only
managed games normally and keeps manager open on pending dialog/timeout; actual
Minecraft save/full-exit acceptance remains NOT_EXECUTED.

Preserve migrated Data/Launchers/Minecraft-1..5, Data/Settings and Data/Cache; external
originals remain recovery only. Keep file-backed hives/ACLs/DLL pins; no account/world
content reads. App/build-info.json records reviewed payloads; no repeat migration.
Submoduleabd4c240031cfb46666daa74ca47ed4e48b561e4; second worktree detached47a45f9c7d4a8ce6799d56997fd38e4b4df49ce7 unchanged.
Four licensed playable clients, gameplay/network/migration/installer acceptance
are separate obligations, NOT_EXECUTED here. Compiler caches remain ignored.

Next: build/stage full-clear candidate and verify real UI review/Cancel only;
then audio and remaining presentation/performance/process acceptance.
Overall goal stays ACTIVE.
