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

Installed App/root sourcea0b88440a526fea4499a1715ff1dd05fde862b53, clean Local build.
App SHA256:24717334c6c93f8b04f0832bb5c0697edd5e4c37c524e762b0d4456b071e371a.
Root starter/DLL hashes: App/build-info.json, verified during Explorer startup.
Candidate workspace/packages/lazy-tester/0.1.0-a0b88440a526-20261026T194457432Z;
installer hash is recorded in its SHA256SUMS.txt.
Signing NOT_VERIFIED; installer execution/publication NOT_EXECUTED/DISABLED.
Actual Explorer startup PID18724: Connected/five enabled/no issue/local WebView cache,
log workspace/runs/launch-desktop-startup.log. Data Storage UI opened controller
without games; own idle UI/controller closed normally, no remaining app/game process.
Earlier audio toggle/persistence, Clear/Reset review/Cancel PASS; logs audio-ui-smoke,
full-clear-ui-smoke, settings-recovery-ui-smoke in workspace/runs. No real deletion.
## Implemented corrections and executed evidence

Presentation: opaque workspace_surface.rs host and previews.rs primary DWM image,
removed before native activation. Original pins/recovery and coalesced foreground/
move worker retained. Nine surface/opacity/translation/focus/stall/recovery fixtures
PASS.40 synthetic swaps p95/max2ms. Logs workspace/runs/opaque-*. Actual Minecraft
drag/focus/frame/switch/DPI NOT_EXECUTED.

FPS120/5, saves5..1000/background<=active, explicit legacy1..4/144/10 preserved.
V3 applies next launch, V2 unchanged. Rust policy fixtures PASS; game pacing untested.

Cache: selected profile stopped, other exact managed games allowed, unknown external
games block. Four lock/link/admission/peer fixtures PASS. No real data deleted.
## Remaining work and owners

Reset/restore: game_preset.rs verified/flushed backup+SHA256 receipt before atomic
options.txt replacement; restore backs up current settings.8 fixtures PASS.
Full clear: product_profiles/clear.rs via same actor/bridge/UI; fresh hive/directory,
generation-bound name+ack, same-volume cutover/catalog/pinned purge. Precommit rollback,
explicit committed purge recovery; read-only inspection never mutates.13 profile
tests (7 clear),5 controller/invalid admission PASS: locks/links/corrupt paths/faults/
partial purge/stale token/readonly/peers. No real reset/clear; review/Cancel proof above.
Audio: session_audio.rs via existing actor/bridge; Data/Settings/audio.json defaults
Keep. Exact PID/birth, no system/cross-process sessions, preserves mute/volume.
250ms wakeups/1second discovery. Silent Windows audio and6 settings fixtures PASS;
logs workspace/runs/audio-*. Checkbox stays enabled through polling, queues saves.
Abrupt controller-crash mixer recovery, long-launch audio response and actual
Minecraft audio remain unverified. Continue in these owners, no second lifecycle.
Launch preflight: original EXE pin, one fresh full Windows resolver instead of two,
named-file ID check and reviewed hash per launch. Initial admission stays two-pass.
Release3sample median2280->1176ms for validation only; read-only benchmark never
opens profiles/starts Minecraft. Identity/path/hash/write-sharing test and5 controller
regressions PASS. Logs workspace/runs/launch-preflight-release-*.log. Candidate staged;
actual launch/window-ready timing and Task Manager roles/grouping remain unproven.
Process roles (working source, not yet installed): process_roles.rs accepts only
same-file hard links for Sessions/Window Recovery, retaining pins through spawn.
Copies/stale binaries rejected; native identity regression PASS. Build-only MUI
resources give LT Sessions/LT Recovery descriptions; actual Windows version API probe
PASS with identical NTFS FileID.39 frontend tests/typecheck and dirty iteration
NSIS build PASS; exact clean candidate/installed helper launch still pending.
Stager/NSIS create fixed links; installer removes only identity-verified links by
handle, leaving main/Data intact. Installer execution NOT_EXECUTED.
WebView2 grouping matches OPEN upstream reports; metadata alone insufficient.
Actual previous process graph: browser/controller parented toUI; browser owns
renderer/GPU/network/storage/crash children. No LazyDesigner attribution.
Log workspace/runs/process-role-observation.log/.ps1; Task Manager GUI NOT_OBSERVED.
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

Next: finish exact clean role-label candidate, stage and run authorized2game tests.
User explicitly AUTHORIZED2sessions started by Lazy Tester for Launch/switch/drag/
preview/full-exit. No world/login/account-content access or closing unrelated games.
Audio crash/long-launch response and presentation/process acceptance remain open.
Overall goal stays ACTIVE.
