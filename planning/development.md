# Current Development Handoff — Lazy Tester

Local only; main/publication untouched; GitHub Actions entirely OFF.
VMware, Sandboxie and RAM/renderer experiments remain PAUSED.

## Active goal and exact current state

User requests reliable startup without repeated setup, all app-owned runtime data
inside project Data, a native EXE at project root and tidy developer material.
Current installed App/root-starter source is f66a6620ca0a2356582b0f604c7d6588de801019.
Five original profiles HAVE migrated into Data successfully; originals remain
intact. Actual Explorer startup has Connected/five enabled across three restarts
and30seconds/55 stable Refresh samples. No Minecraft/game or installer was launched.

One remaining observed defect: WebView still uses AppData. Tauri2.12 ignores
absolute config dataDirectory. The strengthened real Desktop check ran RED on
f66a662 with Connected/five enabled but cacheInsideData=false. Pending source
creates the main WebView explicitly with WebviewWindowBuilder.data_directory,
disables automatic product-window creation, and requires the real browser child
user-data directory inside Data/Cache/WebView. Rust check and34 frontend tests
PASS. Rebuild/staging/actual browser verification are still required. Do not call
all runtime data centralized before that strengthened acceptance passes.

## Root cause and storage owner

Earlier actual Explorer PID38812 reproduced saved registry store Windows error2,
five cards/zero enabled. Same user and logical-HKCU-root hashes, neither elevated,
but development sees1264 pilot subkeys and Explorer zero in32/64 views. The old
binding is also under the development package's redirected LocalCache/Local.
Builder CLI/hidden WebView success was insufficient and prior completion claims
were wrong. The proposed transfer into native HKCU was NOT approved and remains
unused; the user subsequently authorized project-local Data as the destination.

product_profiles.rs owns schema2 Data/Launchers/Minecraft-1..5 with Roaming,
Local/Temp and Registry/HKCU.dat. Catalog validates Windows user, directory and
hive-file identities. Missing hives fail before RegLoadAppKey and cannot become
empty stores. Data receives a protected user/System/Administrators ACL. Legacy
schema1 ordinary-HKCU reopening remains. Enroller/DLL validate inherited file
hives using nonsecret directory/slot markers, preserving user/access checks and
the private/public wire versions. No parallel lifecycle/readiness/credential owner.

Fixed operator-only --lazy-tester-migrate-data validates the reviewed original
five stores, image/payload, idle state and the original app lease. CopyFileW and
SHCopyKeyW transfer the same slot opaquely, retaining uniform destination hive
security. Old App preferences are retained; complete catalog publishes last.
No account values/world contents enter diagnostics. Existing/partial destinations
are never overwritten or cleared; no installer/package hook runs migration.

The first real migration safely refused MC03/04/05 Windows WinINet Content.IE5
external junctions before creating Launchers. The exact Local/Microsoft/Windows/
INetCache/Content.IE5 mount-point case now becomes an empty ordinary cache folder;
its external target is never followed/copied/changed. Every other link is refused.
A real disposable-junction regression covers both accepted and unknown cases.
Rebuilt f66a662 then migrated all five originals (explicit process exit0,
configured=true, sourcePreserved=true). Data has five hives, zero reparse points,
protected ACL, valid catalog/location pointer. App/root payload hashes verified.

manager_settings.rs owns shared project root and Data/Settings preferences;
app_instance.rs uses its product lock there. Data/Cache is for interface/temporary
files. Old outside-project pointers fail for migration review rather than being
ignored/reseeded; new data selection stays in Data. See docs/system/data-layout.md.

Root Lazy Tester.exe is a small icon-bearing native starter for fixed
App/lazy-tester.exe, App working directory, wait/exit propagation; no extra state.
Desktop/Start Menu shortcuts point to it; RUN.cmd is a compatibility wrapper.
Packaging builds/stages both with exact receipts and rollback preservation.
Desktop verifier checks Explorer -> root starter -> App, source/hash, actual
Connected/five enabled controls and now the real browser's project-cache path.

## Actually executed supporting evidence

34 frontend tests; eight repository fixtures; source structure/typecheck PASS.
Targeted Rust: six stores/migration tests, five binding tests, five settings tests
(one separate ignored test), six lock tests PASS. Own C++ file-hive enrollment
before dependency/TLS/main, two slots/two relaunch rounds, foreign-hive rejection
and original ordinary-HKCU enrollment control PASS. No game launched. Root starter
fixture proves fixed target/CWD/exit propagation; embedded icon count1 confirmed.
Full Rust suite not rerun (prior unrelated DACL/pipe fixture issues).

Native binaries: workspace/builds/central-data-20261026/Release.
Compatibility SHA256 0f45c07b370ff6c1bf150ccc20879ff6406c208f2b8c3ccdc596b82f2552381e.
Enroller SHA256 51517528886483ba89f1b93b448c1ddd349f26f0e48e5322db434cbfe9c4de99.
Clean Windows/NSIS candidate built for f66a662:
workspace/packages/lazy-tester/0.1.0-f66a6620ca0a-20261026T163101372Z.
Ignored receipts: workspace/runs/central-project-migration-final.json,
central-project-staged-final.json, central-desktop-startup-1..3.json,
central-desktop-stability.json and project-webview-cache-red.log.
The previous stability kept manager PID26272 open; it was closed normally for
cache diagnosis after confirming no games. Cache-RED verifier closed only its own
manager normally; updater/unrelated TranslateIT processes untouched.

## Physical cleanup and preservation

Inventory: Development/apps21.49GiB, builds14.33GiB, tmp11.78GiB. Size alone is
not disposable-data proof. Two obsolete source Cargo targets had CACHEDIR.TAG,
no tracked files and no affected process. Automatic approval review rejected
recursive deletion. Same-volume archival preserved all contents and freed NO space:
- Development/apps/desktop/src-tauri/target -> workspace/archive/compiler-cache-20261026/desktop-target
- Development/engine/runtime-core/target -> workspace/archive/compiler-cache-20261026/runtime-target
Active builds/toolchains, private/tmp/profile/capability material, legacy copies,
initialized submodule and second worktree retained. Old generated native payloads
also archived before staging replacements. Root source cache paths remain absent.
Receipts: workspace/runs/root-inventory-before.json and root-cleanup-receipt.json.

Next: commit cache source, rebuild and stage exact source preserving migrated Data;
repeat strengthened Explorer verification and leave that verified App open.
Original outside-project source data remains recovery material, not an active
app dependency; no deletion/retirement authorized. Windows identity/DPAPI, official
Minecraft and OS tool prerequisites remain external. Full clear/export/restore,
audio and automatic game shutdown remain unavailable. Keep16:9, High/Low,144/10.
Signing NOT_VERIFIED; installer execution/gameplay/license/multiplayer NOT_EXECUTED.
Publication DISABLED. Overall root/cache repair remains ACTIVE until final proof.
