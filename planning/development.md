# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Full goal: modular five-profile native control center, manual launches, one active
viewport/four previews, storage/cache management, archival and safe packaging.
Renderer/RAM experiments remain PAUSED; GitHub Actions remains entirely OFF.
Previous goal turn: progress (native previews delivered at the exact source below).

## Current bounded outcome — storage and classified HTTP-cache controls

Source parent: d36a5362a863d9acd30b638bd4f020373f8197ea.
Native workspace previews/recovery and their evidence remain at that exact commit;
same-binary retained-controller transport is preserved from 14ae0a9.

Storage details stay in the sole bootstrap session owner, with filesystem work in
independent_bootstrap/profile_storage.rs. The private UI/transport now reports
profile logical bytes/file count, available-to-caller/total-to-caller disk space,
and recognized cache bytes. Disk space is shared capacity, not five reservations.
Unavailable disk data stays unknown; cache bytes are included in profile size.
Nothing scans periodically: the storage page requires an explicit measurement.

The only disposable subtree is the observed per-profile
Local/Temp/minecraftpe/codeBuilderV1/Default/Cache HTTP cache.
Default itself, Cookies, Local Storage, GPU/Code Cache, other TEMP content,
authentication helpers, worlds and presets are excluded. Product contract records
Chromium/CEF primary-source rationale and Windows quota-aware disk semantics.

Cleanup is per-profile and explicitly confirmed in UI. The backend reaps completed
games, refuses while any managed or externally discovered Minecraft game remains,
and serializes cleanup with launches. Every ancestor is pinned; every candidate
file must open exclusively and pass ordinary-file/single-link metadata checks
before any disposition. Reparses, hard links, locks or incomplete bounds refuse
the whole preflight. File deletion uses retained handles, never recursive/path
deletion or directory removal. Disposition failure reports possible partial
removal of this disposable cache. External same-user process races/unowned helper
quiescence are not globally guaranteed; currently open files fail exclusive access.

The UI separates profile usage, shared free space and recognized cache, exposes
open-folder/clear-cache actions, and refreshes measurements after successful
cleanup. Failure invalidates the displayed measurement rather than showing stale
success. This is disk maintenance, not a claimed Minecraft RAM optimization.

## Executed evidence

- Svelte check: zero errors/warnings.
- All 215 frontend tests in 24 files PASS, including storage fields, invalid disk
  capacity, partial/locked/empty cache, forbidden path and active-session refusal.
- Frontend production build PASS: CSS 5.84 kB / gzip 1.83; JS 60.57 kB / gzip 22.56.
- Four targeted Windows cache fixtures PASS: HTTP-only removal while synthetic
  login/world/options/other-TEMP sentinels persist; inactive admission; native
  disk query; locked/hard-linked files refuse before deletion; missing cache;
  reparse ancestor rejection; incomplete/deep plan refuses before deletion.
- One existing bounded metadata scan regression PASS.
- Two native session protocol/controller-gate tests PASS.
- A first parallel fixture run failed with a truncated diagnostic; isolated and
  full reruns passed. Fixture names now also use an atomic uniqueness counter;
  exact initial cause was not retained and is not asserted. Final four-test
  suite after that change PASS.
- Native-preview packaging source contract PASS.
- Windows x64 release build PASS; existing 104 desktop warnings remain.
  Manager SHA-256: 263b0b3b8d656390388f4a355d6a7fa461e9a51e0680d8a453fb7a58e08d1d74.
- No dependency versions/lockfile changed.
- Actual profile directory names were inspected as bounded metadata only to
  identify the exact cache subtree. No account/token/world content was read.
- No actual cache cleanup, candidate GUI, Minecraft, installer, Computer Use,
  UAC or trace executed.

Actual UI/cache cleanup plus Education login/world persistence: NOT_EXECUTED.
Prior preview limitations persist: Minecraft pixels/click/input, 144/10 FPS under
offscreen parking, background Education host simulation and full DPI/dialog/
crash/restart recovery are NOT_EXECUTED. Hidden DWM fixture/API success is not
Minecraft acceptance. Interactive five-window fixture remains NOT_EXECUTED.

## Remaining full-goal work

- Integrated adoption/preview/storage/gameplay acceptance; metadata-driven
  adoption may briefly expose a standalone game window before discovery.
- Taskbar/browser-shell acceptance and Task Manager grouping remain unproven;
  Minecraft still owns separate OS processes.
- Installed Lazy Tester branding/identity migration and native payload packaging.
  Current recipe/capabilities still bind to reviewed local game/profiles.
- Physical legacy relocation after shared consumers retire. Native DLL source
  in bootstrap-ordering is active and must not be archived.
- Primary D: checkout is older and has not been reset or overwritten.
- Same-binary controller reconnect is implemented; cross-binary/version handoff
  remains unsupported. Do not replace an installed executable during live sessions.
- Pre-resume launch errors can leave an inactive slot's storage handle unavailable.
  Persistent files survive; resetting a controller must not disrupt live peers.

## Next — one action

Repair inactive-slot recovery after a pre-resume launch failure in the existing
session owner, retaining original validated profile identity and never retrying
or reclaiming a resumed/uncertain process. Then resume packaging/archive work.

## Preserved baseline

Archive branch codex/archive-before-lazy-tester-20261026 pins
d166b5c8b24e8435f2c84675cb00a61c753aeb6a. Only Local receives product work.
The prior private five-client pilot was operator-confirmed with five licensed
accounts in one Education Host World and persisted login/world/FPS after restart.
New UI/controller acceptance is separate. Compatibility C++ DLL bytes, original
profile capabilities, account/world paths and 144/10 FPS implementation are
unchanged. Approved automatic presets remain integrated; no current options.txt
was written and no installed data was migrated during this increment.