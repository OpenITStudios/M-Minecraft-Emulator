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
3. Process clarity: main UI, root starter, session controller, recovery per game,
   official Minecraft, WebView renderer/GPU/services. Investigate supported Task
   Manager naming/grouping and prove actual results. Do not hide processes, rename
   official binaries, disable browser sandbox or weaken same-image pipe checks.
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
6. Delivery: clean exact Local Windows candidate, frontend/Rust/fixtures/package
   checks; stage App/root EXE/shortcuts preserving Data. Verify actual Explorer
   startup, common frame, drag/switch/previews/full exit/settings/storage/process
   roles to available proof. Compilation never substitutes for usability acceptance.

## Current exact source and delivery

Installed App/root source03a7d49036c6053480d1e77cc415dfc1a3cff0ea, clean Local build.
App SHA256:5ea1751f08555fa1bd0133844c8bf444dbc0eb23d5dfa418f4162d19dc08a25c.
Root starter/DLL hashes: App/build-info.json, verified during Explorer startup.
Candidate workspace/packages/lazy-tester/0.1.0-03a7d49036c6-20261026T191043879Z;
NSIS SHA256:8b241fbab0843f25a1173f4bb1d555fe707301281abf66b2cacb245078a84c28.
Signing NOT_VERIFIED; installer execution/publication NOT_EXECUTED/DISABLED.
Actual Explorer startup: Connected/five enabled/no issue/project-local WebView cache.
Real UI PID35760: Clear all data review starts disabled; wrong typed launcher+ack
stays disabled; exact name+ack enables. Cancel closes review, catalog hash/options
metadata unchanged, no clear journal, no real deletion. Log workspace/runs/
full-clear-ui-smoke.log and .ps1. Owned idle UI closes normally; zero remaining app/
game processes. Earlier7eb8ca1 Reset review/Cancel PASS. Data retained; no setup.
## Implemented corrections and executed evidence

Presentation: workspace_surface.rs opaque host and previews.rs primary DWM image;
native activation removes image first. Original pins/checkpoints/recovery retained.
Existing coalesced foreground/move worker reconciles; no input relay. RED->GREEN
stalled opacity, nine surface tests, translation/minimize/owner, focus, isolated
stall/recovery, release/pipe-loss/helper-crash PASS.40 synthetic swaps p95/max2ms,
not Minecraft latency. Logs workspace/runs/opaque-*. Actual game drag/focus/frame/
switch/DPI NOT_EXECUTED.

FPS:120/5 defaults, saves5..1000, background<=active across UI/bridge/backend.
Historical explicit1..4 and144/10 readable without rewriting. V3 pacer gets saved
values next launch; historical V2 defaults unchanged. Rust default/roundtrip/legacy/
boundaries PASS; actual120/5 game pacing NOT_EXECUTED.

Cache: selected profile stopped; other exact managed games may continue, unknown
external games block. Actor serializes pinned/exclusive cleanup. Four deletion and
PID/birth admission fixtures PASS (locks/links/incomplete plans/peer preservation).
No real cache/account/world deleted. Existing Rust warnings remain.
## Remaining work and owners

Reset/restore source added: game_preset.rs backs up options.txt, flushes/readbacks,
publishes SHA256 receipt before atomic replacement; restore backs up current state.
Stopped-profile admission and review wired through existing actor/bridge/UI. Eight
preset/recovery fixtures PASS. No real settings mutation; review/Cancel proof above.
Full clear: product_profiles/clear.rs, same actor/bridge/UI, fresh hive/directory,
generation-bound name+ack confirmation, same-volume cutover, catalog update and
pinned-handle purge. Precommit rollback; committed purge needs explicit resume.
Read-only inspection never mutates.13 profile tests (7 clear fixtures),5 controller
and invalid admission PASS: locks/hardlinks/junctions/corrupt metadata/cutover faults/
partial purge/stale token/readonly/peers. No real clear; candidate staged above.
Audio source: session_audio.rs through existing controller, typed bridge and General.
Data/Settings/audio.json defaults Keep; Mute scopes exact live PID/birth and excludes
cross-process/system sessions, preserves user mute/volume, restores own changes.
Existing250ms wakeups; endpoint/session discovery1second or ownership change.
Real silent Windows sessions PASS: inactive mute, active restore, preexisting/later
user mute, lease removal and another subprocess unchanged.6 settings tests and38
frontend tests/typecheck PASS. Logs workspace/runs/audio-*. Candidate not yet staged.
Real UI exposed polling closing the audio dropdown: native probe observed Expanded
become Collapsed/disabled during2second refresh. Audio selection now remains enabled
during background polling and queues its save until that request finishes. UI proof
of the correction and exact candidate delivery are pending.
Abrupt controller-crash mixer recovery, long-launch audio response and actual
Minecraft audio remain unverified. Continue in these owners, no second lifecycle.
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

Next: deliver scoped audio, then address crash recovery/launch response and
remaining presentation/performance/process acceptance.
Overall goal stays ACTIVE.
