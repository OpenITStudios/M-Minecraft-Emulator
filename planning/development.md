# Current Development Handoff — Lazy Tester

Local only; main release-reserved; GitHub Actions entirely OFF. RAM/renderer,
Sandboxie and VMware routes PAUSED. Development/Archive organization retained.

## Current bounded outcome — real setup/picker validation

Operator continued the proposed interactive setup/picker validation after
9b6429ea14bcd30557ca7604c46fd08d560e8079. This authorized opening the locally
built Lazy Tester GUI/native folder dialog for that scope. Computer Use used
only those windows; no game/auth/security UI, installer or account/world reading.

Baseline App 88aa73f1fcb11d7193e0ccd574018e8d176bd0a0 opened and showed
Connected, zero games and the retained five-profile pilot binding. Storage showed
its original pilot root, not the new product Data layout. Actual Locate cancel
restored usable controls and retained the location. Selecting empty checkout Data
was refused with raw os-error-2. No data pointer/catalog was created.

The existing product_profiles.rs owner now contextualizes missing files during
selected-folder verification. The message requests an existing Data folder with
profiles.json and all five profiles, explains empty/old-pilot folders are not
reattachment targets, and states the current location did not change. Error kind,
identity/user/idle/open/publish gates remain intact. Default empty-directory setup
resolution is unchanged; no registry/data migration or reset added.

Implementation and exact new candidate source:
b0ecca914a11c7e5ad23c73d946f37c925d83450.
Three targeted Windows product_profiles fixtures PASS: create/reopen/preserve,
valid selected-folder verification, same-volume relocation, replaced-profile
refusal, empty/partial selection preservation, relative defaults, custom missing
location refusal and corrupt-locator preservation. Only original disposable
filesystem/HKCU fixture stores were used. Feature lazy-tester-package plus
TAURI_CONFIG from its real overlay; filter:
engine::native_clients::independent_bootstrap::product_profiles::tests.
Repository structure and G0 PASS. No frontend/native DLL logic changed in this fix.

Fresh candidate was opened and actual empty-folder selection repeated. The exact
new English message appeared, original pilot location remained displayed, controls
recovered, root Data stayed empty and App/data-location.json was absent. Both test
GUI instances closed normally; final process counts: zero Lazy Tester/Minecraft.
Cancellation PASS belongs to baseline88aa73f; improved-message/refusal PASS belongs
to b0ecca9. Do not merge those into a full exact-revision acceptance claim.

## Exact package

Clean Local b0ecca914a11c7e5ad23c73d946f37c925d83450 built Windows/NSIS via
npm --prefix Development run package:windows. App refreshed with previous App
preserved; four payload hashes match build-info.json. Export:
workspace/packages/lazy-tester/0.1.0-b0ecca914a11-20261026T102645960Z.
Installer SHA256: 8cb8d5ec4c65807726228b669e8ecfeb13e4659d5a9411b4230604b236965f4b.
Installer NotSigned; no installation executed. Metadata-only receipt:
workspace/runs/setup-recovery-20261026/interactive-picker-completion.json.
Build log: picker-message-build.log in the same directory. No raw user/registry
identity, profile catalog, account/world content or GUI screenshot copied to Git.
A follow-up handoff commit changes documentation only; App uses b0ecca9 source.

## Preserved implementations

Setup UI recovery: 88aa73f1fcb11d7193e0ccd574018e8d176bd0a0. Failed saved-location
inspection retains Locate; backend verifies selected identity before publishing.
Unknown location never exposes Choose as a reset. Manage data opens Storage;
setup/location errors survive session refresh. Failed verification invalidates
configured state and stale sizes/cleanup confirmations; reattachment rechecks
binding. Expanded headless mocked-Tauri cases and 32 frontend tests passed at
that implementation, not rerun as new frontend evidence for the message-only fix.

Usability: 300d5911c80a9caed5966d3e74052c12bf1734d3. Latest navigation survives
in-flight operations; game-ready events do not pull users away from Settings.
Storage refresh checks location/usage explicitly; partial sizes are lower bounds;
four-page layout checked at920x620/1366x900. Existing native recovery remains owner.

Settings/native: 19860393bb54a8d573ff8b16e4d597f10e8753a3. Shared FPS defaults144/10,
integers1..1000, background<=active. manager_settings.rs atomically persists only
FPS settings beside executable. Missing file uses defaults without writing;
invalid file preserved/refused. Save applies at next explicit launch; running
games retain limits. No watcher/new limiter/per-frame config I/O. Original graphics
preset writer retains Simple/12chunks/reduced effects and unowned options. Apply
all requires idle verified stores; multi-profile failure is not atomic.
Private presentation V3 uses the reserved DWORD of the120-byte capability; V2
retains144/10. Identity/event/handle/storage/account ABI checks unchanged. Native pins:
compatibility9d1cf60673c1ebbc0d94c30c1e28e226aa4429653901e5abd6e95400457ef8cb
enroller6e7c000bee2dcf043d372317c77a04078ca12451c7273a18bd54e8f89e62af2a
Prior native build/Pacer/Rust logs: workspace/runs/settings-20261026.

## Remaining proof and preservation

Successful real product-Data reattachment NOT_EXECUTED: observed active data uses
retained pilot stores and no verified product catalog is available. Do not convert,
copy or fabricate user data merely to manufacture acceptance. Partial setup repair,
pilot-to-product/cross-volume migration and backup/restore remain unimplemented.
Actual installed fresh setup, FPS144/10/custom, licensed login/world persistence
and Host World/gameplay remain NOT_EXECUTED. Zero processes and GUI success do
not establish game readiness or licensing. Prior pilot acceptance is historical.

Root Data remains empty; performance.json and data-location.json absent. No real
cache clear, account/world read, UAC, service/driver, installer or Minecraft launch.
Known unrelated DACL-fixture refusal, oversized-pipe hang and rustfmt drift remain;
no globally-green suite claim. Submodule, second worktree and unknown workspace
material preserved. No main/release changes or CI. Higher proof ceiling here is
TARGET_WINDOWS for the specifically observed GUI behaviors only.

## Next — one action

Define a non-destructive migration/reattachment design for the retained pilot data
before attempting product-Data acceptance. No live account/world move/reset is
implied. STOP for completed bounded picker validation and message correction.
