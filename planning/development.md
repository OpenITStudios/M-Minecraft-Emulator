# Current Development Handoff — Lazy Tester

## Single active goal — user revised27October2026

Priority: fix and prove the black screen and lost game interaction. Next make one
normal app exit close its two owned sessions, then verify preview/drag/focus and
Play/Settings frames. Measure Launch/switch and finish functional Settings/Data
Storage and clear process roles. Defaults120/5, minimum5FPS, background<=active.
Work remains incomplete. Keep the user's desktop undisturbed; interactive testing
is deferred until the end.

Local only; main/publication untouched; CI OFF. VM/Sandboxie and RAM/renderer work
PAUSED. Compiler caches explicitly RETAINED. No incidental cleanup or actual profile
wipe. Clear all data is a product feature tested with synthetic fixtures; actual use
requires selected-launcher review/confirmation.

## Acceptance — preserve all outcomes

1. Same Play/Settings frame; stable drag/resize/focus/DPI, 16:9 main game and
   persistent monitoring previews. No parenting, binary patch or input relay.
2. Measure Launch/switch; reuse original leases and pins without weakening identity.
3. Task Manager UI/starter/controller/recovery/Minecraft/WebView roles and grouping;
   do not hide processes, alter official binaries or weaken peer gates.
4. General: High/Low only; truthful valid whole5..1000 FPS saves with120/5 defaults.
   Preserve explicit older preferences; new defaults do not overwrite user choices.
   Keep/Mute background audio must actually scope original game audio sessions,
   never other apps. No ignored selections or placeholder controls as features.
5. Per-launcher Data Storage: real sizes/path, cache cleanup, settings reset/backup/
   restore, full selected account/world/cache/settings/backup clear to empty launcher.
   Preserve installation/peers; verify confirmation, recovery, interruption,
   active profiles, malformed paths, links, cancel and peer isolation.
6. Clean exact Local Windows candidate/checks/receipts, App/root EXE/shortcuts,
   Data preserved. Verify Explorer startup and all requested UX/runtime flows;
   compilation never substitutes for usability acceptance.

## Current exact source and delivery

Installed App/root sourcee4be7ed4c7b7da6a8f7ad3bf786e0a0208e1df4b, clean Local build.
App SHA256:186f0ca6fb2f760ac6271c2e07839836c2d8b62c113cd45bb9cf9fdc6c430220.
Root starter/DLL hashes: App/build-info.json, verified during Explorer startup.
Candidate workspace/packages/lazy-tester/0.1.0-e4be7ed4c7b7-20261027T033008416Z;
installer hash is recorded in its SHA256SUMS.txt.
Signing NOT_VERIFIED; installer execution/publication NOT_EXECUTED/DISABLED.
Previous startup: Connected/five enabled/local cache; process role metadata verified.
Two games opened in 7977/6587ms, six switches took 481–581ms. One drag kept preview
aligned; Play/Settings frame styles matched. Logs workspace/runs/two-session-*.log.
These checks did not prove visible pixels or usable gameplay; the user then reported
a black primary viewport and inaccessible game. First full-exit attempt timed out
with both games retained; a subsequent request exited the owned games and manager.
Both games showed the exact native "Exit Minecraft Education?" OK/Cancel prompt.
Installed controller change confirms only this exact prompt for the pinned original
game; unknown prompts are preserved. Six controller fixtures PASS, including exact
dialog and unrelated/unknown refusal. Real one-click exit remains unproven.
Log workspace/runs/full-exit-dialog-fixture.log.
Earlier audio toggle and Clear/Reset review/Cancel PASS; no real deletion.
## Implemented corrections and executed evidence

Presentation: opaque host and primary DWM image; original pins/recovery retained.
At ba2731d two games launched (8509/5759ms), but user confirms black screen remains.
Actual probe captured focused Minecraft - 1 parked at x2592 outside the manager;
its primary preview was hidden. Live stability check reproduced displacement after
selection (black-screen-live-stability.log). Previous fix removed an unordered drag
move; synthetic race and stalled-drag fixtures PASS, but actual issue persisted.
Installed change wakes the ordered geometry worker for foreground transitions as well;
focus event fixture failed before and passes after (black-screen-focus-*.log). Clean
Windows build and App staging PASS; live stability and user-visible proof pending.
Source9d5ec16 additionally keeps the primary preview until the focused game is
visible at its viewport, refreshes after repositioning, and rechecks focus after
parking. Five surface logic regressions PASS; real pixels/input remain unverified.
Frontend schedules preview reconstruction after a failed selection, launch or page
transition;39 frontend tests and typecheck PASS. Real failure recovery is unverified.
Clean candidate workspace/packages/lazy-tester/0.1.0-9d5ec163726b-20261027T033405938Z
built, not staged. Interactive testing deferred; do not control the user's desktop.

FPS120/5, saves5..1000/background<=active, explicit legacy1..4/144/10 preserved.
V3 applies next launch, V2 unchanged. Rust policy fixtures PASS; game pacing untested.

Cache: selected profile stopped, other exact managed games allowed, unknown external
games block. Four lock/link/admission/peer fixtures PASS. No real data deleted.
## Remaining work and owners

Reset/restore backs up and verifies settings before replacement;8 fixtures PASS.
Full clear uses generation-bound confirmation, rollback and explicit purge recovery;
13 profile and5 controller fixtures PASS. No real reset/clear; review/Cancel above.
Audio: session_audio.rs; exact PID/birth, preserves mute/volume. Silent Windows audio
and6 settings fixtures PASS; logs workspace/runs/audio-*. Checkbox queues saves.
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

Next: when the user is available, verify two real sessions on installed e4be7ed,
including visible output, interaction, drag, preview, switching and one-click exit.
User explicitly AUTHORIZED2sessions started by Lazy Tester for Launch/switch/drag/
preview/full-exit. No world/login/account-content access or closing unrelated games.
Audio crash/long-launch response and presentation/process acceptance remain open.
Overall work is not complete.
