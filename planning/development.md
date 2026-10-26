# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Full goal: modular five-profile native control center, manual launches, one active
viewport/four previews, storage/cache management, archival and safe packaging.
Renderer/RAM experiments remain PAUSED; GitHub Actions remains entirely OFF.
Previous goal turn: progress (startup/storage recovery delivered at parent below).

## Current bounded outcome — native Lazy Tester installer candidate

Source parent: b370acdf2094c01bd530c0202956cc3707d9b44a.
Startup recovery evidence remains there; storage/cache at 87b6576 and native
previews at d36a536. These do not prove integrated product/gameplay acceptance.

The native distribution owner is engine/distribution/lazy-tester-package.json,
consumed by tauri.lazy-tester.conf.json and the opt-in lazy-tester-package feature,
which inherits the existing native-workspace-preview runtime gates.
Root package:windows -> desktop build:app -> build:lazy-tester now builds this
route. The old VM installer builder remains explicitly blocked. Private preview
configuration remains available for source candidates, not another product owner.

Identity decision: separate per-user Lazy Tester installation, identifier
com.mivubi.lazytester, binary lazy-tester.exe. The old preview and VM products
are neither silently upgraded/uninstalled nor relocated. This is explicit because
the actual NSIS template keys default installation/uninstall metadata by product
name, not identifier alone. Existing MinecraftInstanceManager/NativePreview
preferences, NativePilot single-manager lease, game profile identities/worlds/
HKCU data and 144/10 FPS implementation remain unchanged.

The new native hooks check the shared controller mutex before install/uninstall,
refusing when present or not queryable, without killing anything. Uninstall
forces application-data preservation before the template's deletion conditional.
No hook runs a tool, migrates data or recursively deletes runtime material.
Hook compilation/generated-script ordering is proven; actual mutex admission,
installation, repair, upgrade and uninstall behavior is NOT_EXECUTED.

prepare-native-payload.mjs derives reviewed SHA-256 hashes from the existing
bootstrap source policy, validates both complete DLLs, then stages exactly those
bytes into the ignored package-resources/lazy-tester folder. It rejects links,
unreviewed bytes and different preexisting output. Adjacent profile bindings are
never copied. Build-time Rust checks and the Node package checker enforce the
dedicated identity, current-user mode, exact native resources, removed legacy
resources, no sidecars and preserved desktop security permissions.

## Executed evidence

- 218 frontend/source tests in 25 files PASS. New package tests cover identity
  isolation/excluded private bindings, runtime hash extraction/malformed policy,
  exact two-DLL staging/idempotence, changed bytes, stale output and preservation
  of an adjacent private fixture.
- Both existing private-preview and new native package source checks PASS.
- Actual Tauri x64 release + NSIS installer construction PASS, including native
  hook compilation. Existing 104 desktop warnings remain.
- Actual generated installer.nsi inspection confirms Lazy Tester identity/name,
  new hooks, hook-before-app-check ordering, exact two DLL + contract file entries,
  no profile capability or VM/Guest Agent/acceptance entries, and the guarded
  DeleteAppDataCheckboxState condition. This is generated-script inspection,
  not archive extraction or installed-payload proof.
- Manager Windows version metadata: ProductName/FileDescription Lazy Tester,
  ProductVersion 0.1.0.
- Manager SHA-256 (after Tauri bundle patch):
  a7d35917073f176dbe614a56875ae8c9a9e0f6d101d74c491dc83f1fe998070c.
- Installer SHA-256:
  3a0194896463ebfabbf44b8ccf10583985882e57b6f0a8669205b236246fde6a.
- Authenticode status: NotSigned. No signing/publication/release performed.
- Frontend build PASS, unchanged UI assets from the prior storage milestone.
- Dependency versions/lockfiles unchanged; Cargo feature/configuration only.
- No installer, candidate app, Minecraft, Computer Use, UAC or trace executed.
  Generated binaries/capabilities remain ignored/local; none enter source history.

Installer output: ignored workspace/tmp/hud-direct-surface-20261025/manager-build/
x86_64-pc-windows-msvc/release/bundle/nsis/Lazy Tester_0.1.0_x64-setup.exe.
It is a DEVELOPMENT CANDIDATE, not a ready-to-distribute product.

## Critical remaining setup gap

Profile capabilities are machine-local metadata and deliberately excluded from
the installer. The runtime currently looks beside the executable for those files.
Therefore a fresh installation cannot launch the five old profiles yet.
Do not fix this by embedding capabilities/accounts/worlds in the installer or
guessing directories. Add a reviewed explicit existing-profile binding workflow
in the sole bootstrap owner/UI, preserving original file identities. Current
reviewed recipe remains local/game-build-bound; generic portable provisioning
is not implemented and must not be claimed.

## Remaining full-goal work

- First-run binding above; installed payload/repair/upgrade/uninstall acceptance
  and signed publication remain unexecuted. No current installer was run.
- Integrated adoption/preview/storage/gameplay acceptance. Hidden DWM fixture
  registration is not live Minecraft pixels/input/FPS proof. Offscreen Education
  host simulation, DPI/dialog/minimize/crash/restart and real cleanup/login
  persistence remain NOT_EXECUTED.
- Taskbar/browser-shell acceptance and Task Manager grouping remain unproven;
  games still own separate OS processes.
- Physical legacy relocation after shared consumers retire. Bootstrap-ordering
  contains active native DLL source and must not be archived as obsolete tooling.
- Primary D: checkout is older and was not reset or overwritten.
- Cross-binary/version retained-controller handoff is unsupported; a different
  binary preserves/refuses an existing controller. Do not replace live binaries.
- Cache cleanup remains explicit HTTP-only disk maintenance. Unknown helper
  quiescence or unsupported caches never justify broad TEMP/profile deletion.

## Next — one action

Implement explicit existing-profile binding for the installed Lazy Tester through
the existing bootstrap/UI owner, with validated original capabilities stored
privately outside the installer and no copying/reset of login/world contents.

## Preserved baseline

Archive branch codex/archive-before-lazy-tester-20261026 pins
d166b5c8b24e8435f2c84675cb00a61c753aeb6a. Only Local receives product work.
The prior five-client pilot was operator-confirmed with five licensed accounts in
one Education Host World, persisted login/world data and 144/10 FPS after restart.
The new UI/controller/installer has separate acceptance. Approved presets remain
integrated; no real options.txt, installed state, account or world was modified.