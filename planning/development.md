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
NSIS SHA256:d7cc20867ab82f31b44ec10e0c9162a08558fa2623327c50ef61d3f7a4a4d193.
Signing NOT_VERIFIED; installer execution/publication NOT_EXECUTED/DISABLED.
Actual Explorer startup PID18724: Connected/five enabled/no issue/local WebView cache,
log workspace/runs/launch-desktop-startup.log. Data Storage UI opened controller
without games; own idle UI/controller closed normally, no remaining app/game process.
Earlier7b7b0e1 PID37644 audio checkbox stable across2refreshes, Mute/Keep saved viaUI;
workspace/runs/audio-ui-smoke.log/.ps1. No real game audio tested.
Earlier03a7d49 Clear review/name+ack gating/Cancel preserved catalog/options and
created no deletion journal;7eb8ca1 Reset review/Cancel PASS. Data retained; no setup.
## Implemented corrections and executed evidence

Presentation: opaque workspace_surface.rs host and previews.rs primary DWM image,
removed before native activation. Original pins/recovery and coalesced foreground/
move worker retained. Nine surface/opacity/translation/focus/stall/recovery fixtures
PASS.40 synthetic swaps p95/max2ms. Logs workspace/runs/opaque-*. Actual Minecraft
drag/focus/frame/switch/DPI NOT_EXECUTED.

FPS120/5, saves5..1000/background<=active, explicit legacy1..4/144/10 preserved.
V3 applies next launch, V2 unchanged. Rust policy fixtures PASS; game pacing untested.

Cache: selected profile stopped; other exact managed games may continue, unknown
external games block. Actor serializes pinned/exclusive cleanup. Four deletion and
PID/birth admission fixtures PASS (locks/links/incomplete plans/peer preservation).
No real cache/account/world deleted. Existing Rust warnings remain.
## Remaining work and owners

Reset/restore: game_preset.rs verified/flushed backup+SHA256 receipt before atomic
options.txt replacement; restore backs up current settings.8 fixtures PASS.
Full clear: product_profiles/clear.rs via same actor/bridge/UI; fresh hive/directory,
generation-bound name+ack, same-volume cutover/catalog/pinned purge. Precommit rollback,
explicit committed purge recovery; read-only inspection never mutates.13 profile
tests (7 clear),5 controller/invalid admission PASS: locks/links/corrupt paths/faults/
partial purge/stale token/readonly/peers. No real reset/clear; review/Cancel proof above.
Audio: session_audio.rs, existing controller/bridge/General; Data/Settings/audio.json
defaults Keep. Exact live PID/birth, no cross-process/system sessions, preserves
mute/volume.250ms wakeups, discovery1second/ownership change. Silent Windows sessions
PASS (mute/restore/user mute/foreign process/lease removal);6 settings and38 frontend
tests/typecheck PASS. Logs workspace/runs/audio-*. UI checkbox stays enabled through
polling and queues saves; real toggle/persistence proof above.
Abrupt controller-crash mixer recovery, long-launch audio response and actual
Minecraft audio remain unverified. Continue in these owners, no second lifecycle.
Launch preflight: original EXE pin, one fresh full Windows resolver instead of two,
named-file ID check and reviewed hash per launch. Initial admission stays two-pass.
Release3sample median2280->1176ms for validation only; read-only benchmark never
opens profiles/starts Minecraft. Identity/path/hash/write-sharing test and5 controller
regressions PASS. Logs workspace/runs/launch-preflight-release-*.log. Candidate staged;
actual launch/window-ready timing and Task Manager roles/grouping remain unproven.
WebView2 grouping matches OPEN upstream reports; AppUserModelID/metadata alone
insufficient. Actual process graph at installed source: WebView2 manager24624 and
controller28932 both parented toUI39612; browser owns renderer/GPU/network/storage/
crash children. UI/controller both FileDescription Lazy Tester. No LazyDesigner
attribution. Log workspace/runs/process-role-observation.log/.ps1; Task Manager GUI
grouping NOT_OBSERVED. Investigate role labels without weakening same-image gates:
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

Next: investigate supported per-role labels preserving binary/peer identity.
User question pending: permission for2managed-game Launch/switch/drag/preview/full-exit
tests, no world/login/account-content access; earlier no-game restriction still holds.
Audio crash/long-launch response and presentation/process acceptance remain open.
Overall goal stays ACTIVE.
