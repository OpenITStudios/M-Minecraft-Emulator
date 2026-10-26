# Current Development Handoff — Lazy Tester

Local only; main/publication untouched; CI OFF. Legacy VM, Sandboxie and
RAM/renderer experiments remain PAUSED.

## User outcome and exact state

User requests reliable Desktop startup, project-local Data, root native EXE and
removal of unused development output. Installed App/root starter source is
04b5c52498393492fed6b734a49f00f36004d0a7. Startup/data repair verified on that EXE.
Cleanup NOT complete: automatic approval rejected recursive compiler-cache deletion
again despite renewed explicit user authorization. ZERO bytes deleted/reclaimed.
Do not bypass denial, call archival reclaimed space, or mark the whole job DONE.

## Final actual Desktop evidence

Explorer -> root Lazy Tester.exe -> App/lazy-tester.exe exercised three times on
04b5c52: Connected, five cards/five enabled Launch controls, no connection error,
cacheInsideData=true. Final PID17848/root starter5596 deliberately left open.
No Minecraft game or installer launched. Actual enabled controls are not proof of
independently licensed/playable Minecraft sessions.

Ignored receipts: workspace/runs/project-desktop-cache-final-1.json,
project-desktop-cache-final-2.json and project-local-final-integrity.json.
All five App/root hashes verified; five HKCU.dat files; zero Data links; protected
Data ACL;183 actual browser-cache files. Desktop/Start Menu shortcuts target root
Lazy Tester.exe with checkout CWD. Starter owns no settings/session state and only
forwards to App. Embedded icon/fixed target/CWD/exit propagation tested previously.

Default package build exported/staged04b5c52, then failed updating shortcuts because
inherited PowerShell module discovery hid Get-FileHash. Shortcut/startup scripts
now explicitly import PSHOME Utility (and CimCmdlets for startup). Actual shortcut
creation and normal Explorer verification passed afterwards; PlanOnly also passed
under the same local build environment. Full default build not rerun for these
script-only imports; installed backend source remains04b5c52.

Candidate: workspace/packages/lazy-tester/0.1.0-04b5c5249839-20261026T164826275Z.
NSIS SHA256 1fe175807b0079aa6eb543347c6564eb8005277985e31eaf5d5b4fbd4f1b8d0a.
App SHA256 1787bf71391224a3bb60aa3c4e48eecf2686ac4c3ce6c5634efe17ab719dbb32.
Starter SHA256 992366e8dacc32d2f4e8692b44378d8222ca7f4cda5bbe70ca77595d21a02791.
Signing NOT_VERIFIED; installation/gameplay/license/multiplayer NOT_EXECUTED;
publication DISABLED.

## Root cause and canonical storage owner

Actual Explorer failed opening legacy HKCU stores with Windows error2. Same user
and logical registry root hashes, neither elevated, but development saw1264 pilot
keys and Explorer zero; old pointer was also development-package AppData redirected.
Builder CLI/hidden WebView success did not establish Desktop readiness.
User authorized project-local Data instead of the proposed native-HKCU transfer.

product_profiles.rs owns schema2 Data/Launchers/Minecraft-1..5 with Roaming,
Local/Temp and Registry/HKCU.dat. Catalog binds Windows user, directory and hive
identities. Missing hives cannot become empty replacements. Data ACL protects
user/System/Administrators. Enroller/DLL verify inherited application hives using
nonsecret directory/slot markers, retaining ordinary-HKCU/access checks/private ABI.
Data/Settings holds preferences, location pointer and original instance lease.

Operator-only --lazy-tester-migrate-data copied all five reviewed originals opaquely
using CopyFileW/SHCopyKeyW, preserving sources and publishing catalog last.
workspace/runs/central-project-migration-final.json: exit0, migratedProfiles5,
configured=true, sourcePreserved=true, gameLaunched=false. No account/world contents
inspected/logged; no installer/package migration hook or bidirectional sync.
Exact Local/Microsoft/Windows/INetCache/Content.IE5 mount-point junctions became
empty destination folders; external targets untouched; every other link refused.

Tauri2.12 ignores absolute config dataDirectory. Real browser inspection ran RED on
f66a662 even with Connected/five enabled. app_bootstrap.rs now disables automatic
main creation and uses WebviewWindowBuilder.data_directory(Data/Cache/WebView).
TEMP/TMP are process-local Data/Cache/Temp. Actual browser command line proves final
cache location; no parallel readiness owner. Windows identity/DPAPI, licensed game
installation and OS prerequisites remain external. Original outside-project data
is recovery material, not active app data; do not delete it as compiler junk.

## Executed supporting checks

34 frontend tests, eight prior repository fixtures, Rust check, typecheck and source
structure PASS. Targeted Rust: six migration/store, five binding, five settings,
six instance-lock tests PASS. Native own C++ fixture: file-hive enrollment before
dependency/TLS/main, two slots/two relaunches and foreign-hive rejection PASS;
ordinary-HKCU control PASS. Full Rust suite not rerun (prior unrelated DACL/pipe
fixture issues). After script imports: three structure fixtures and structure PASS.

Native outputs retained workspace/builds/central-data-20261026/Release.
Compat SHA256 0f45c07b370ff6c1bf150ccc20879ff6406c208f2b8c3ccdc596b82f2552381e.
Enroller SHA256 51517528886483ba89f1b93b448c1ddd349f26f0e48e5322db434cbfe9c4de99.

## Compiler cleanup blocked — all three folders still exist

Reviewed: Cargo CACHEDIR.TAG signatures, no tracked files, no reparse points and
no consuming project/compiler process. App runs from its own payload copies.
- workspace/archive/compiler-cache-20261026/desktop-target:22,912,765,268 bytes.
- workspace/archive/compiler-cache-20261026/runtime-target:3,522,646,095 bytes.
- workspace/builds/lazy-tester-layout-20261026/target:9,918,766,219 bytes.
Total36,354,177,582 bytes (33.86GiB), NOT reclaimed. Two source targets previously
moved physically into archive, preserving contents and freeing ZERO space.
Receipts: workspace/runs/compiler-cache-cleanup-blocked.json,
root-cleanup-receipt.json and root-inventory-before.json. Unknown/private/tmp
history, original profiles, active toolchains and source dependencies retained.
No blanket tmp deletion. Root source target paths remain absent.

Submodule retained abd4c240031cfb46666daa74ca47ed4e48b561e4; second worktree
47a45f9c7d4a8ce6799d56997fd38e4b4df49ce7 unchanged. Updater/unrelated processes
untouched. Keep16:9, High/Low and144/10. Full account clear/export/restore absent.

Next: resolve execution-policy blocker for these exact reviewed compiler caches;
verify actual deletion/free space before claiming cleanup complete. Do not repeat
profile setup/migration, rebuild unnecessarily or expand into gameplay experiments.
