# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Full goal: modular five-profile native control center, manual launches, one active
viewport/four previews, storage/cache management, archival and safe packaging.
Renderer/RAM experiments remain PAUSED; GitHub Actions remains entirely OFF.

## Current bounded outcome — native four-preview workspace

Source parent: 14ae0a93f21c613fb4df5607c97c2514741fc31b.
The retained-controller transport and its evidence remain at that exact commit.

The existing workspace owner now supplies four DWM thumbnail destinations in
nonactivating manager-owned top-level windows. Sources come only from retained
controller-attributed game identities. UI requests contain slots and bounded
rectangles, never arbitrary source HWNDs/PIDs. Native clicks enter the existing
selection path; input goes directly to the selected game. No capture/encoding loop,
game parenting, second session controller or authentication store was introduced.

Live-preview mode parks background game windows beyond the virtual desktop,
retaining their main viewport dimensions. Stable geometry avoids repeated helper
RPCs. Dialog-blocked/unresponsive windows are not newly parked. Offscreen occlusion
and Minecraft simulation behavior remain unproven. The worker starts only with an
eligible tile, blocks when idle and coalesces updates through a bounded queue.
Previews hide when the manager is minimized or an unrelated application has focus.

A native recovery gate is retained jointly by parent and recovery helper until
original checkpoint restoration completes. A reopened GUI refuses fresh capture
during that handoff. This is concurrency protection, not attribution or a security
boundary. Release still attempts game restoration if preview cleanup fails.

The mounted interface has one large selected game, four lower cards, explicit
manual launch, pending startup and queued selection. Startup launches no game.
Detached windows remain detached across polling. Explicit selection supersedes
older automatic selection intent. The old app.css is excluded from the active
frontend bundle; legacy source remains preserved.

## Executed evidence

- Svelte check: zero errors/warnings.
- All 214 frontend tests in 24 files PASS, including preview report slot/status
  validation and small-preview viewport bounds.
- Frontend production build PASS: CSS 5.25 kB (gzip 1.71); JS 57.13 kB (gzip 21.62).
  Bundle size is not game RAM/performance proof.
- Windows Rust check PASS; existing 104 desktop warnings remain.
- Three targeted preview tests PASS: geometry/slots/overlap, aspect letterboxing,
  hidden own-window DWM register/update/query/release. Those windows were never
  shown or focused; no desktop/game was captured.
- Native recovery-gate lifetime test PASS with synthetic key, no external HWND.
- Native-preview packaging source contract PASS.
- Windows x64 release manager build with explicit preview overlay PASS.
  SHA-256: f65dbe94c67f74b15d1f9579e6662ff26d1d3aafc55c9b82ffd3b45e4315eabd.
- No dependency versions/lock file changed; existing windows-sys DWM features only.
- No Minecraft, candidate GUI, installer, Computer Use, UAC or trace executed.

API registration is not live Minecraft proof. Pixel freshness, preview click/input,
144/10 FPS under offscreen parking, background Education host simulation,
DPI/minimize/dialog/crash/restart behavior: NOT_EXECUTED.
The interactive five-window fixture was previously compiled, remains NOT_EXECUTED.

## Remaining full-goal work

- Integrated preview/adoption/gameplay acceptance. Metadata-driven adoption may
  briefly expose a standalone game window before discovery.
- Taskbar/browser-shell acceptance and Task Manager grouping remain unproven;
  Minecraft still owns separate OS processes.
- Safe per-profile cache classification/deletion, free-space reporting/storage UX.
- Installed Lazy Tester branding/identity migration and native payload packaging.
  Current recipe/capabilities remain bound to reviewed local game/profiles.
- Physical legacy relocation after shared consumers retire. Native DLL source in
  bootstrap-ordering is active and must not be archived.
- Primary D: checkout is older and has not been reset or overwritten.
- Same-binary controller reconnect is implemented; cross-binary/version handoff
  remains unsupported. Do not replace an installed executable during live sessions.
- Launch failure recovery needs review: taking inactive-slot storage before
  create/enroll can leave it unavailable after pre-resume failure. Data is intact;
  do not assume retry/controller restart is safe for live peers.

## Next — one action

Implement safe per-profile storage/cache controls in the existing session owner:
classify proven disposable caches first, expose free space and preserve accounts,
worlds, presets and active-process files. Never blanket-delete TEMP/Local/Roaming
or authentication data.

## Preserved baseline

Archive branch codex/archive-before-lazy-tester-20261026 pins
d166b5c8b24e8435f2c84675cb00a61c753aeb6a. Only Local receives product work.
The prior private five-client pilot was operator-confirmed: five licensed accounts,
one Education Host World, persisted login/world/FPS after normal restart.
New UI/controller acceptance is separate. Compatibility C++ DLL bytes, original
profile capabilities, accounts/worlds and 144/10 FPS implementation were unchanged
and not operated by this increment. Approved automatic presets remain integrated;
no current options.txt was written.