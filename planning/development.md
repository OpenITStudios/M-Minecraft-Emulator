# Current Development Handoff — Lazy Tester

## Single active goal — user revised27October2026

Priority: repair stuck launcher/full exit, active/background FPS response, Task Manager
process grouping and naming; retain blackscreen/input acceptance. Make one
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

Installed App/root source 6dff3d15f237436e0f259a0d41521a21e52d51a6, clean Local build.
App SHA256:228f46f4f19ce212777ddc5d3235e59871f7ee10bd165798709fc237e7b87204.
Root starter/DLL hashes: App/build-info.json, verified during Explorer startup.
Signing NOT_VERIFIED; installer execution/publication NOT_EXECUTED/DISABLED.
Earlier two-session testing found blackscreen and exit timeout; six synthetic exit-dialog fixtures passed. Real acceptance is revision-specific.
Earlier audio toggle and Clear/Reset review/Cancel PASS; no real deletion.
## Implemented corrections and executed evidence

Presentation: opaque host and primary DWM image; original pins/recovery retained.
At ba2731d two games launched but black screen remained. The focused game was parked
outside the manager and its preview hidden (black-screen-live-stability.log).
Installed change wakes the ordered geometry worker for foreground transitions as well;
focus event fixture failed before and passes after (black-screen-focus-*.log). Clean
Windows build and App staging PASS; live stability and user-visible proof pending.
Source9d5ec16 keeps the preview until the focused game is positioned, refreshes it
after repositioning and rechecks focus after parking. Five surface regressions PASS.
Frontend schedules preview reconstruction after failed selection, launch or page
transition. Headless synthetic failed-switch test reproduced the missing refresh on
prior source and passed after the fix; previous session and error remain visible.
Switch rollback now reports failed parking/restoration and suspends automatic
updates on incomplete recovery. Logic test PASS; actual recovery remains unverified.
Clean candidate c5ef097: workspace/packages/lazy-tester/0.1.0-c5ef0970a16b-20261027T034907265Z; built, not staged. Interactive testing deferred.

FPS120/5, saves5..1000/background<=active, legacy preferences preserved. V3 applies
next launch. FPS save now queues during polling: headless synthetic UI passed success,
invalid limits and refresh-failure cases; previous source disabled Save. Logs
workspace/runs/fps-queue-headless-*.log. Real WebView/game pacing still unverified.
Graphics applies the selected slot/preset after polling, rechecking idle admission.
Headless synthetic refresh and changed-admission cases PASS; actual UI not verified.

Cache: selected profile stopped, other managed games allowed; unknown games block.
Review confirmations pause automatic polling, avoiding ignored clicks/flicker; manual
refresh disables actions while pending, backend rechecks admission. Headless cache,
reset, full-clear and folder controls PASS; no real data deleted. Lock/link fixtures PASS.
## Remaining work and owners

Reset/restore backs up and verifies settings before replacement;8 fixtures PASS.
Full clear uses generation-bound confirmation, rollback and explicit purge recovery;
13 profile and5 controller fixtures PASS. No real reset/clear; review/Cancel above.
Audio: session_audio.rs; exact PID/birth, preserves mute/volume. Silent Windows audio
and6 settings fixtures PASS; logs workspace/runs/audio-*. Checkbox queues saves.
Abrupt controller-crash mixer recovery, long-launch audio response and actual
Minecraft audio remain unverified. Continue in these owners, no second lifecycle.
Launch preflight retains identity pins; five controller tests PASS.
Process roles: pinned hard links for Sessions/Window Recovery, copies rejected;
MUI descriptions verified. Installer execution NOT_EXECUTED.
WebView2 grouping matches OPEN upstream reports. Browser/controller parented toUI;
Task Manager GUI NOT_OBSERVED. Log workspace/runs/process-role-observation.log.
No one-group Task Manager success claim. Full-exit closes only managed games normally
and keeps manager open on pending dialog/timeout; first attempt failed as above.

Preserve migrated Data/Launchers/Minecraft-1..5, Data/Settings and Data/Cache; external
originals remain recovery only. Keep file-backed hives/ACLs/DLL pins; no account/world
content reads. App/build-info.json records reviewed payloads; no repeat migration.
Submoduleabd4c240031cfb46666daa74ca47ed4e48b561e4; second worktree detached47a45f9c7d4a8ce6799d56997fd38e4b4df49ce7 unchanged.
Four-client, gameplay, network, migration and installer acceptance NOT_EXECUTED.

Latest 6dff3d1 runtime: launch two sessions PASS; first switch test failed because
Minecraft 2 was covered, retry six switches/drag PASS. Pixels/input NOT_PROVEN.
Full exit timed out: MC1 exited, MC2 and manager remain. No force termination.
Logs workspace/runs/current-two-session-{launch,presentation,presentation-retry,exit}.log.
Source: full exit can confirm the exact owned exit dialog even if its main window
is missing; identity rechecked before posting. Cargo check and refusal test PASS;
actual dialog without window NOT_EXECUTED. Exit wait also checks the retained child
after window loss. Starting games are polled for a verified window until the existing
exit deadline; cargo check and idle test PASS, startup transition NOT_EXECUTED.
Other stuck states still require diagnosis.
FPS foreground sampling retries transient instability; unit test PASS, actual FPS
response and sustained instability NOT_EXECUTED.
Next: inspect FPS focus policy and process grouping;
source tests first without PC control. Then verify two real sessions,
including visible output, interaction, drag, preview, switching and one-click exit.
User explicitly AUTHORIZED2sessions started by Lazy Tester for Launch/switch/drag/
preview/full-exit. No world/login/account-content access or closing unrelated games.
Audio crash/long-launch response and presentation/process acceptance remain open.
Overall work is not complete.
