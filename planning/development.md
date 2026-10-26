# Current Development Handoff — Lazy Tester

Local only; main is release-reserved and GitHub Actions remains OFF. RAM/renderer,
Sandboxie and VMware work remain PAUSED. Development/Archive layout is retained.

## Current bounded outcome — actual setup/picker validation

Operator explicitly continued the proposed interactive setup/picker test after
9b6429ea14bcd30557ca7604c46fd08d560e8079. Computer Use was used only on the
locally built Lazy Tester GUI and its native folder dialog; no game/auth/security
UI. Exact App source 88aa73f1fcb11d7193e0ccd574018e8d176bd0a0 opened, showed
Connected/zero games and verified the retained five-profile pilot binding. Storage
showed the original pilot location, not product Data; root Data stayed empty.
Actual Locate dialog cancel restored usable controls and retained the location.
Selecting the empty checkout Data was refused; no pointer or catalog was created.
The resulting raw os-error-2 message was unhelpful. The existing product_profiles
owner now contextualizes missing files only during selected-folder verification:
select a complete existing Data folder with profiles.json and all five profiles;
empty/old-pilot folders are not product reattachment targets. Error kind and all
identity/idle/open/publish gates remain intact. Default empty setup resolution is
unchanged. Three targeted Windows fixtures PASS, including empty/partial folder
preservation and valid selection. A fresh clean package and real rejection retest
follow this commit. No registry/data migration or automatic replacement added.

Successful real product-data reattachment is NOT_EXECUTED: this observed machine
uses retained pilot stores and has no verified product Data catalog. No user data
will be converted or fabricated merely to manufacture that acceptance result.

## Preserved setup/data recovery implementation

Operator resumed setup and location validation from Local
943af543fb388b5b9f707357fa6ea1a237188e05. Implementation:
88aa73f1fcb11d7193e0ccd574018e8d176bd0a0. Owner remains the mounted
Development/apps/desktop/src/LazyTesterApp.svelte; no native storage/schema changes.

Inspection found failed saved-location checks disabled Locate data, blocking the
existing backend recovery path. Storage now retains Locate for unknown locations;
its backend still rechecks idle/controller/catalog/Windows-user/directory identity
before changing the pointer. Unknown location does not expose Choose as a reset.
Setup's Manage data links to Storage. Setup/location errors survive unrelated
session refresh. Failed binding checks clear configured state; failed location
inspection also invalidates binding, sizes and cleanup confirmation. Verified
reattachment rechecks binding before enabling Launch. Picker cancel or invalid
selection does not authorize replacing data. No account/world migration or partial
setup reseeding was added. See docs/system/data-layout.md.

## Executed evidence for this change

32 frontend tests PASS; Svelte zero errors/warnings; production frontend build,
repository structure and G0 PASS. Expanded optional headless Edge regression:
Development/tooling/acceptance/lazy-tester-ui-regression.mjs. PASS with mocked Tauri:
startup, queued navigation, Storage refresh/partial counts, FPS draft/save,
four-page bounds at 920x620 and 1366x900, game-ready navigation, disconnect/reconnect;
also missing/invalid data, retained setup error, cancel, invalid folder refusal,
verified reconnect and stale setup disabling Launch. Failed-setup layout fits
920x620. Run after frontend build with MIVUBI_PLAYWRIGHT_MODULE pointing to an
available Playwright module. No application dependency added.

Two Windows product_profiles fixtures PASS with --features lazy-tester-package and
TAURI_CONFIG from src-tauri/tauri.lazy-tester.conf.json. Filter:
engine::native_clients::independent_bootstrap::product_profiles::tests.
They cover create/reopen/preserve, same-volume relocation, replaced-profile refusal,
relative default, missing custom location refusal and corrupt locator preservation.
Only disposable synthetic filesystem/HKCU stores were used and cleaned by their
original fixture owner. A default-feature invocation selected zero tests; that was
not counted. No product backend change or account-bearing profile inspection.

Known unrelated DACL-fixture refusal, oversized-pipe hang and rustfmt drift from
prior work remain unresolved; no globally-green suite claim.

## Exact Windows candidate

Clean Local 88aa73f1fcb11d7193e0ccd574018e8d176bd0a0 built Windows/NSIS via
npm --prefix Development run package:windows. App refreshed by the existing stager,
with previous App retained. All four payload hashes match build-info.json; export
installer hash matches its receipt. Data remains empty; no actual performance.json
was created. Export:
workspace/packages/lazy-tester/0.1.0-88aa73f1fcb1-20261026T101013679Z.
Installer SHA256: c5feeae822f429788440f81f84bee7625f8e968f17d0838a0819848ee492647f.
App and installer are NotSigned. Metadata-only receipt:
workspace/runs/setup-recovery-20261026/completion.json. A subsequent handoff commit
changes documentation only; binary acceptance belongs to the implementation SHA.
Submodule and detached second worktree unchanged; output/private paths stay ignored.

## Preserved behavior and proof limits

Usability implementation: 300d5911c80a9caed5966d3e74052c12bf1734d3.
Settings/native implementation: 19860393bb54a8d573ff8b16e4d597f10e8753a3.
Shared FPS defaults144/10, integer1..1000, background<=active. Existing
manager_settings.rs owns atomic executable-adjacent performance.json. Missing means
defaults without writing; invalid files are preserved/refused. Saved limits apply
at next explicit launch; running games retain old limits. No watcher/new limiter
or per-frame configuration I/O. Original preset writer retains Simple/12 chunks/
reduced effects and unowned options. Apply all requires idle verified stores; retry
is safe, but multi-profile failures are not a cross-profile atomic transaction.
Private presentation V3 reuses the reserved DWORD of the 120-byte capability;
V2 retains144/10. Identity/event/handle checks and account/storage ABI unchanged.
Native pins remain:
compatibility 9d1cf60673c1ebbc0d94c30c1e28e226aa4429653901e5abd6e95400457ef8cb
enroller 6e7c000bee2dcf043d372317c77a04078ca12451c7273a18bd54e8f89e62af2a
Prior native build/Pacer/targeted-Rust evidence is in workspace/runs/settings-20261026;
it was not rerun as native logic was unchanged.

No Computer Use, product GUI, actual picker, installer execution, Minecraft, UAC,
service/driver, real cache deletion or account/world-content reading performed.
Actual installed setup, FPS, licensed login/world persistence and Host World remain
NOT_EXECUTED. Source/package success does not establish gameplay proof. Partial
setup repair, cross-volume migration, backup/restore and unknown workspace cleanup
remain unfinished/deferred. Previous pilot gameplay is historical proof only.

## Next — one action

Build and retest the selected-folder message in the real native picker, then
record exact candidate/evidence and stop. Complete-product-data reattachment,
fresh installed setup and Minecraft/FPS acceptance remain separate and unfinished.
