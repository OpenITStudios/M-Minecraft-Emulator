# Current Development Handoff — Lazy Tester

## Single active goal — user revised27October2026

Priority: fix and prove the reported black screen and lost game interaction.
Then verify stable Play/Settings frame, preview/drag, per-slot focus and full exit
with two owned sessions; optimize measured Launch/switch, finish functional
Settings/Data Storage, clear process roles and honest Windows acceptance. Defaults
120/5, minimum5FPS, background<=active. Work remains incomplete.

Local only; main/publication untouched; CI OFF. VM/Sandboxie and RAM/renderer work
PAUSED. Compiler caches explicitly RETAINED. No incidental cleanup or actual profile
wipe. Clear all data is a product feature tested with synthetic fixtures; actual use
requires selected-launcher review/confirmation.

## Acceptance — preserve all outcomes

1. Same Play/Settings frame without drag snag/desktop exposure through drag end/
   cancel, resize, maximize/restore, focus, DPI/monitor changes. Main game16:9;
   persistent larger monitoring previews. No parenting, binary patch or input relay.
2. Fast switch/Launch: measure focus/bounds and controller/validation/enrollment/
   window-ready phases. Reuse original leases/image pins without weakening identity,
   inventing readiness or busy polling.
3. Actual Task Manager roles/grouping: UI/starter/controller/recovery/Minecraft/
   WebView. No hiding, official binary renames, sandbox disable or weaker peer gates.
   Role/PID pages do not prove grouping; explain proven upstream limitations.
4. General: High/Low only; truthful valid whole5..1000 FPS saves with120/5 defaults.
   Preserve explicit older preferences; new defaults do not overwrite user choices.
   Keep/Mute background audio must actually scope original game audio sessions,
   never other apps. No ignored selections or placeholder controls as features.
5. Per-launcher Data Storage: real sizes/path, cache cleanup, settings reset/backup/
   restore, full selected account/world/cache/settings/backup clear to empty launcher.
   Preserve installation/peers; allow unrelated verified managed games. Explain
   Windows identity limits. Verify confirmation, recovery/catalog identities,
   interruption, active profiles, malformed paths, links, cancel and peer isolation.
6. Clean exact Local Windows candidate/checks/receipts, App/root EXE/shortcuts,
   Data preserved. Verify Explorer startup and all requested UX/runtime flows;
   compilation never substitutes for usability acceptance.

## Current exact source and delivery

Installed App/root source8c0d171c6326988bb2a7bcf52f4b83059b3f46ad, clean Local build.
App SHA256:cf573d6469c9d1d33419d8bb0b7c09c6650f8c0ae0637060acda09d32c568006.
Root starter/DLL hashes: App/build-info.json, verified during Explorer startup.
Candidate workspace/packages/lazy-tester/0.1.0-8c0d171c6326-20261027T030410242Z;
installer hash is recorded in its SHA256SUMS.txt.
Signing NOT_VERIFIED; installer execution/publication NOT_EXECUTED/DISABLED.
Actual Explorer startup PID31852: Connected/five enabled/no issue/local WebView cache,
log workspace/runs/process-roles-desktop-startup.log. Two owned game windows launched
in 7977/6587ms; controller and recovery roles verified by actual process metadata.
Six switch samples took 481–581ms from click to foreground. One 60x30px drag kept
the background preview aligned, and Play/Settings native frame styles matched.
These checks did not prove visible pixels or usable gameplay; the user then reported
a black primary viewport and inaccessible game. First full-exit attempt timed out
with both games retained; a subsequent request exited the owned games and manager.
One-click full exit remains unproven. Logs workspace/runs/two-session-*.log.
Earlier audio toggle/persistence, Clear/Reset review/Cancel PASS; logs audio-ui-smoke,
full-clear-ui-smoke, settings-recovery-ui-smoke in workspace/runs. No real deletion.
## Implemented corrections and executed evidence

Presentation: opaque host and primary DWM image; original pins/recovery retained.
Potential black-screen race reproduced synthetically: an asynchronous drag move
displaced the game after presentation. Working change defers moves to the ordered
helper/worker; regression failed before and passed after (black-screen-drag-*.log).
This proves the race, not yet that it alone caused the user's black screen. Build,
stage and validate actual pixels/input after the fix. Multi-DPI remains untested.

FPS120/5, saves5..1000/background<=active, explicit legacy1..4/144/10 preserved.
V3 applies next launch, V2 unchanged. Rust policy fixtures PASS; game pacing untested.

Cache: selected profile stopped, other exact managed games allowed, unknown external
games block. Four lock/link/admission/peer fixtures PASS. No real data deleted.
## Remaining work and owners

Reset/restore backs up and verifies settings before replacement;8 fixtures PASS.
Full clear uses generation-bound confirmation, rollback and explicit purge recovery;
13 profile and5 controller fixtures PASS. No real reset/clear; review/Cancel above.
Audio: session_audio.rs via existing actor/bridge; Data/Settings/audio.json defaults
Keep. Exact PID/birth, no system/cross-process sessions, preserves mute/volume.
250ms wakeups/1second discovery. Silent Windows audio and6 settings fixtures PASS;
logs workspace/runs/audio-*. Checkbox stays enabled through polling, queues saves.
Abrupt controller-crash mixer recovery, long-launch audio response and actual
Minecraft audio remain unverified. Continue in these owners, no second lifecycle.
Launch preflight retains the original EXE pin and rechecks registration/identity/hash.
Release3sample validation median2280->1176ms; identity and5 controller tests PASS.
Logs workspace/runs/launch-preflight-release-*.log; actual timings are above.
Process roles (installed at 8c0d171): same-file pinned hard links for Sessions/Window
Recovery, copies rejected. MUI descriptions verified via Windows and live processes.
39 frontend tests/typecheck and clean NSIS build PASS; installer execution NOT_EXECUTED.
WebView2 grouping matches OPEN upstream reports; metadata alone insufficient.
Actual previous process graph: browser/controller parented toUI; browser owns
renderer/GPU/network/storage/crash children. No LazyDesigner attribution.
Log workspace/runs/process-role-observation.log/.ps1; Task Manager GUI NOT_OBSERVED.
https://github.com/MicrosoftEdge/WebView2Feedback/issues/5628
https://github.com/tauri-apps/tauri/issues/15567
No one-group Task Manager success claim. Full-exit closes only managed games normally
and keeps manager open on pending dialog/timeout; first attempt failed as above.

Preserve migrated Data/Launchers/Minecraft-1..5, Data/Settings and Data/Cache; external
originals remain recovery only. Keep file-backed hives/ACLs/DLL pins; no account/world
content reads. App/build-info.json records reviewed payloads; no repeat migration.
Submoduleabd4c240031cfb46666daa74ca47ed4e48b561e4; second worktree detached47a45f9c7d4a8ce6799d56997fd38e4b4df49ce7 unchanged.
Four licensed playable clients, gameplay/network/migration/installer acceptance
are separate obligations, NOT_EXECUTED here. Compiler caches remain ignored.

Next: build and stage the black-screen fix, then repeat authorized real-session tests
including visible output, interaction, drag, preview, switching and full exit.
User explicitly AUTHORIZED2sessions started by Lazy Tester for Launch/switch/drag/
preview/full-exit. No world/login/account-content access or closing unrelated games.
Audio crash/long-launch response and presentation/process acceptance remain open.
Overall work is not complete.
