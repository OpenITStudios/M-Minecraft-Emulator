# Current Development Handoff — Lazy Tester

Local only; main is release-reserved and GitHub Actions remains OFF. RAM/renderer,
Sandboxie and VMware work remain PAUSED. Development/Archive layout is retained.

## Current bounded outcome

The operator approved continuing APP usability after configurable Settings.
Parent: e19db96fddd9def41b7609fd6022e93d23ce7f03. The first owner remains
Development/apps/desktop/src/LazyTesterApp.svelte and its existing CSS/bridges.

Workspace, Launchers, Storage and Settings now retain the latest navigation click
while an inspection/layout/operation completes. Navigation waits for initial
profile/location checks; no second session owner or overlapping native operation.
A pending game becoming RUNNING does not force another page back to Workspace.
Existing native suspend/presentation/recovery gates remain in place.

Startup distinguishes checking profiles from required/unavailable setup. Empty
Launchers and disconnected Workspace explain recovery; stale launcher status is
explicitly labeled. Connection failures show Unavailable rather than Connecting
forever. The generic error banner no longer asserts preservation for unknown
operation outcomes. Backend error evidence remains visible.

Storage page Refresh actually checks location and storage, without background
scans. Counts include a last-check time, singular/plural files and an explicit
lower bound for partial measurements. Location changes or failed measurements
invalidate stale sizes/cleanup confirmations. Unknown location differs from a
confirmed missing folder. Existing cache/account/world protection and native
picker admission are unchanged.

Buttons wrap at smaller widths; long profile issues wrap; setup banners have a
bounded scroll area. Navigation exposes aria-current; loading/status feedback is
available to assistive technology. The UI remains English. Existing FPS inputs,
draft preservation, Save, Restore defaults and graphics preset remain intact.

## Executed source evidence

32 frontend tests PASS; Svelte zero errors/warnings; production frontend build
PASS. Repository structure and G0 contracts PASS. No Rust/native logic changed;
known unrelated DACL-fixture refusal, oversized-pipe hang and rustfmt drift from
prior work remain unresolved, with no globally-green suite claim.

Development/tooling/acceptance/lazy-tester-ui-regression.mjs is an optional source
regression using headless Edge, built frontend and mocked Tauri only. PASS:
startup, navigation queued during a pending inspection, Storage refresh/partial
counts, FPS draft across pages and Save, four-page bounds at 920x620 and 1366x900,
game-ready navigation behavior, disconnect/reconnect. One simulated Launch is in
memory; no actual profile/game/native session opens. Run after frontend build
with MIVUBI_PLAYWRIGHT_MODULE pointing to an available Playwright module. No
application dependency added.

## Preserved Settings/native implementation

Settings implementation: 19860393bb54a8d573ff8b16e4d597f10e8753a3.
Shared active/background limits default 144/10; integer 1..1000 and background
not above active. Existing manager_settings.rs atomically persists performance.json
beside the executable. Missing file means defaults without writing; invalid files
are preserved/refused. Saving applies at next explicit launch; running games keep
their limits. No watcher, additional limiter or per-frame config I/O. The original
preset writer retains Simple/12 chunks/reduced effects and unowned options.
Apply all requires all games idle and stores verified; retry is safe, but a failed
multi-profile update is not an atomic cross-profile transaction.

Private presentation V3 reuses the reserved DWORD of the 120-byte capability;
V2 retains 144/10. Identity/event/handle checks and storage/account ABI unchanged.
Pinned native hashes remain:
compatibility 9d1cf60673c1ebbc0d94c30c1e28e226aa4429653901e5abd6e95400457ef8cb
enroller 6e7c000bee2dcf043d372317c77a04078ca12451c7273a18bd54e8f89e62af2a
Prior native evidence, not rerun for this frontend-only change: MSVC Release
builds and configurable Pacer self-test (60/5 deadlines/focus/stop/v2/v3 validation),
12 targeted Rust tests. Logs: workspace/runs/settings-20261026.

## Package and remaining proof

Previous App/export: clean Local 19860393; a fresh Windows/NSIS candidate for the
current frontend change is the immediate completion step. Preserve locator,
performance.json and previous App through the existing stager.

No Computer Use, product GUI, installer execution, Minecraft, UAC, service/driver,
account/world-content reading or real cache deletion performed. Headless browser
checks use synthetic paths/process metadata only. Actual default/custom FPS,
licensed login/world persistence, Host World behavior, data migration, partial
setup recovery and installed lifecycle acceptance remain NOT_EXECUTED/unfinished.
Source/package success does not establish gameplay proof. Unknown workspace
material/cache deletion remains deferred; second worktree and initialized
upstream submodule must remain intact.

## Next — one action

Finish clean-revision Windows package/hash verification for this bounded usability
scope, then record the exact build revision and stop. Interactive acceptance is
separate and not a prerequisite for further useful source work.
