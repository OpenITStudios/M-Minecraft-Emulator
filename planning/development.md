# Current Development Handoff — Lazy Tester

Local only; main/publication untouched; GitHub Actions entirely OFF.
VMware, Sandboxie and RAM/renderer experiments remain PAUSED.

## Active goal — root-local application and reliable Desktop startup

The user explicitly requested all app-owned data beneath the project, an ordinary
root EXE and cleanup of scattered developer material, alongside the unresolved
Desktop connection failure. The earlier proposal to copy five stores into native
HKCU was NOT approved and remains unused. The chosen, subsequently authorized
migration destination is project Data, with original data retained.

Source parent b7c5bf36150362e47d24bf7003a8619516bcf14c. Existing App still contains
09eaca9eb7c2a7593e316848be833d8179f3cb84 until the new clean build/staging step.
No original account/world data has been migrated or deleted at this checkpoint.

## Confirmed cause and repaired source

Actual Explorer startup reproduced the exact error: PID38812, Explorer4764,
five launcher cards, zero enabled, saved registry store Windows error2.
Same user SID/logical HKCU-root hashes, neither elevated, but the development
context sees1264 pilot subkeys and Explorer sees zero in32/64 views. Development
CLI configured=true; Explorer CLI has no binding. The old binding is also under
the development package's redirected LocalCache/Local. Build-tool/hidden WebView
success therefore did not establish Desktop startup. The former completion claim
was wrong. Root starter -> App verification must use Explorer and real UI.

product_profiles.rs now owns schema2 Data/Launchers/Minecraft-1..5, each with
Roaming, Local/Temp and Registry/HKCU.dat. Windows user, profile-directory and
hive-file identities are validated. Missing hives fail before RegLoadAppKey and
are never recreated as recovery. Data gets a protected user/System/Admin ACL.
Legacy schema1 ordinary-HKCU reopening remains. The native enroller and DLL accept
application hives only with correct directory/slot identity metadata, preserving
user/access checks and the existing private/public wire versions.

Fixed operator-only --lazy-tester-migrate-data validates the original reviewed
five stores, game/image/payload and idle state, takes the original app lease,
preflights links, then copies each same slot opaquely using CopyFileW/SHCopyKeyW.
The latter preserves the uniform destination hive descriptor. Original registry,
accounts and worlds remain intact; no raw values or world contents enter logs.
Preferences are retained and the catalog is published last. Existing/partial
Data launchers are never overwritten, cleared or treated as fresh setup.

manager_settings.rs owns shared project-root resolution and Data/Settings FPS/
graphics/location state; app_instance.rs uses its product marker there. WebView
and process-local temporary storage now use Data/Cache. Existing outside-project
location pointers fail for review rather than being ignored/reseeded. New
selections stay in Data. No session/readiness owner or account automation added.

The root icon-bearing native Lazy Tester.exe forwards to fixed App/lazy-tester.exe,
uses App working directory and waits/propagates exit. It writes no parallel state.
Packaging builds/stages both with source/hash receipts and rollback preservation;
Desktop/Start Menu shortcuts target the root starter. RUN.cmd is only a wrapper.
Actual UI acceptance verifies Explorer -> root starter -> App, Connected and five
enabled launchers at the expected revision. Native CLI success cannot replace it.

## Source/native evidence actually executed

34 frontend tests PASS; eight repository fixtures PASS; structure and G0 PASS.
Targeted Rust: five product-store/migration tests, five binding tests, five settings
tests (one separate test remains ignored), six app-instance tests PASS.
Own C++ fixture with file hives exercised enrollment before dependency/TLS/main,
two slots/two relaunch rounds and foreign-hive rejection PASS. No game launched.
Root starter target/CWD/exit propagation passed with a disposable own executable;
embedded icon count1 confirmed. Full Rust suite was not rerun (prior unrelated
DACL/pipe fixture issues); no broad performance claim. Final source/native checks
must be repeated only where later source/payload changes require them.

New reviewed native directory: workspace/builds/central-data-20261026/Release.
Compatibility SHA256 0f45c07b370ff6c1bf150ccc20879ff6406c208f2b8c3ccdc596b82f2552381e.
Enroller SHA256 51517528886483ba89f1b93b448c1ddd349f26f0e48e5322db434cbfe9c4de99.
Old generated payload staging was preserved under workspace/archive before refresh.
Logs/receipts remain ignored in workspace/runs/; never publish raw machine data.

## Physical organization actually performed

Metadata inventory: Development/apps21.49GiB; workspace/builds14.33GiB;
workspace/tmp11.78GiB. Large size alone is not evidence material is disposable.
Both obsolete source Cargo targets had valid CACHEDIR.TAG, no tracked files and
no affected active process. Recursive deletion was rejected by automatic approval
review. Reversible same-volume archival retained all contents:
- Development/apps/desktop/src-tauri/target -> workspace/archive/compiler-cache-20261026/desktop-target
- Development/engine/runtime-core/target -> workspace/archive/compiler-cache-20261026/runtime-target
No space was reclaimed. Source directories no longer contain those compiler trees.
Active toolchains/builds, tmp/private/profile/capability material, legacy working
copies and unrelated TranslateIT processes were retained. No guessed junk deletion.
Inventory/operation receipts: workspace/runs/root-inventory-before.json and
root-cleanup-receipt.json. Initialized submodule and second worktree untouched.

## Remaining work and evidence ceiling

Clean c89d428 Windows/NSIS construction and guarded local App/root-starter staging
completed. No source game/installer execution. The explicit original-data migration
then correctly stopped before creating Launchers: MC03/04/05 each contain the
Windows WinINet Content.IE5 external junction. Originals remained intact. Source
now handles only that exact cache mount-point path by creating an empty ordinary
destination cache folder without following the external target. Unknown links
remain refused. A real disposable junction regression and all six store tests PASS.
New clean build/staging/migration/actual Desktop checks are still pending.

Next: commit clean source, construct Windows/NSIS candidate, guard/stage App and
root starter, run the fixed approved project-Data migration, then repeatedly verify
actual Explorer/Desktop startup and identical source/native-Desktop data opening.
Do not announce completion before those real process observations succeed.

Original source backups outside the project are preserved for recovery until
explicit retirement; no unrelated Windows profile/account cleanup is authorized.
Full clear, world export, settings reset/restore, audio control and automatic game
shutdown remain unavailable. Keep existing16:9 layout, High/Low and144/10 defaults.
No Minecraft/game/installer GUI or sign-in is automated. Signing NOT_VERIFIED;
installer execution/gameplay/license/multiplayer acceptance NOT_EXECUTED.
Publication DISABLED. The overall repair remains ACTIVE at this checkpoint.
