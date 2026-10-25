# Current Development Handoff — Lazy Tester

`Local` is the only active product branch. This is the sole current handoff.
Goal: a modular five-profile native Minecraft control center named Lazy Tester,
manual profile launches, one large viewport/four lower previews, storage/cache
controls and safe legacy archiving. Renderer/RAM experiments remain PAUSED; CI OFF.

## Current bounded outcome — desktop/controller integration

Source parent: d166b5c8b24e8435f2c84675cb00a61c753aeb6a.
Archive branch `codex/archive-before-lazy-tester-20261026` preserves that exact
repository snapshot. Only Local receives product changes.

Mounted entry now uses LazyTesterApp.svelte and validated bridge lazyTester.ts:
five fixed profiles, manual launch, selected-game viewport, next/previous usable
game, status cards, on-demand metadata-only size inspection and Open Folder.
Window/display title is Lazy Tester. Installed identifiers, packaging product/
binary identity, existing profile paths and account/world data remain unchanged.

native_clients_session.rs transports bounded fixed-action requests to one hidden
same-executable helper inside the existing bootstrap owner:
independent_bootstrap/session_controller.rs. It reuses exact game/DLL/profile
checks, suspended-child enrollment, original handles, FPS channels and journals.
It shares the original experiment gate; existing unknown games are refused.
No arbitrary path/PID selection, title-order attribution, auto-launch, credentials
store, token copying, Sandboxie or VMware runtime was added.

Opening/refreshing the UI starts no helper/game. Explicit launch/storage actions
start the controller. Before launch, only the selected inactive owned profile
receives the existing approved preset (with backup/atomic replacement and other
settings preserved). No current profiles were actually modified in this session.
The prior preset runtime deferral does not authorize operating running games.

GUI EOF preserves games; the helper retains FPS/storage handles until normal
game exit. Uncertain/resumed children are never killed or automatically retried.
Hidden workspace windows retain their original process birth/session/HWND.
Responsive windows are not evidence of sign-in/world readiness.

Storage scanning is on demand, metadata-only, directory-pinned, reparse-skipping,
and bounded by depth/count/time. Partial totals are identified. Open Folder uses
the canonical retained profile directory. No cache deletion or account/world
content reading was implemented.

## Executed evidence

- Svelte/TypeScript: zero errors/warnings; frontend production build PASS.
- Desktop Vitest: 24 files / 212 tests PASS. Entry/title checks target Lazy Tester;
  old function-extraction checks were corrected without changing their assertions
  about restoration/nonblocking stop. Shared focus policy must observe manager focus.
- Windows Rust check PASS; existing 104 desktop warnings remain, including old
  conflicting extern declarations. No warning-free product claim.
- New bounded protocol test PASS; new disposable metadata-scan fixture PASS.
- Existing native-preview packaging source contract PASS, without inherited VMware
  resources, Guest Agent binaries or legacy installer hooks.
- Windows x64 release manager build with explicit preview overlay PASS.
  SHA-256: c176bd1c13f844468708826c38aedfa548fddb8dcbe0ccfffc25f9b01d8071b4.

New GUI/controller Minecraft acceptance: NOT_EXECUTED.
No game launching/closing/input/sign-in, installer installation, PC UI control,
profile migration, account/world deletion or GitHub Actions execution occurred.
Prior five-client pilot acceptance does not prove the new integration.

## Unfinished product work

- No controller reconnection after UI restart yet. Games survive UI exit, but a
  competing controller refuses while the retained helper remains. Not completed
  browser-like session recovery.
- Lower rail is status/selection cards, not live thumbnails. Native DWM preview
  composition/hidden-window behavior remain unimplemented/unproven.
- Task Manager grouping is unproven. Multiple game processes remain visible.
  No process hiding or game-binary renaming.
- New windows are adopted into native workspace/taskbar presentation on selection;
  automatic all-session shell adoption is not implemented.
- Cache classification/deletion, free-space reporting, installed Lazy Tester name/
  identity migration and self-contained native payload packaging remain.
- Payload/capabilities/singleton recipe remain bound to the reviewed local game
  build/profiles. Developer candidate, not a portable/general installer.
- Archive snapshot and inactive UI routing exist. Physical legacy source relocation
  remains pending consumer retirement. Current compatibility DLL source still lives
  in bootstrap-ordering acceptance tooling and must not be archived as old work.
- Primary D: checkout remains older than the managed implementation worktree.
  No reset or deletion of its untracked files occurred.

## Next — one action

Implement authenticated reconnection to the retained bootstrap controller so
reopening Lazy Tester recovers its five exact sessions without duplicate launch
or closing games. Sequence live previews/cache cleanup/physical legacy relocation
after that ownership seam is correct.

## Previous accepted state

The private five-client pilot was operator-confirmed: distinct licensed accounts
in one Education Host World; after normal save/close/restart, login/worlds and
144 active / 10 background FPS counters persisted. Compatibility DLL unchanged:
f65f263b6cbf76054a86d67e50ba4d02d3c5b15fc98a70e627ea19a73eab2edf.
Enroller unchanged:
73f9038364b2b853a50e6d058613049234568fccd054ce9a758f572531707368.

Prior handoff/evidence and explicit preset deferral:
https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/d166b5c8b24e8435f2c84675cb00a61c753aeb6a/planning/development.md
