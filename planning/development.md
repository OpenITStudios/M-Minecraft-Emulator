# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Current goal: a modular five-profile native app with visible application/data
folders, persistent locations, English UI and safe distribution. CI remains OFF.
Renderer/RAM experiments remain PAUSED. Main is release-reserved.

## Current bounded outcome — physical checkout entry point and app staging

Source parent: 84d5af5b0813b2ece3d49c085d650ecb3f2d9187.
The operator requested that the actual local root match the documented layout,
and explicitly authorized removal of unused material. Root RUN.cmd now starts
only App/lazy-tester.exe. The existing package owner stages the four required
runtime files plus a source/hash receipt into ignored root App after building
and exporting a clean Local candidate. It preserves data-location.json, archives
the previous app under workspace/private/local-app-backups, refuses a running
local app, and never copies adjacent private capabilities. Existing Data is
untouched; absent Data is created empty. The product owner now admits an empty
unconfigured default folder, while nonempty partial/saved unavailable data still
fails closed. Old accounts/worlds remain in place, not migrated into Data.

Executed: 30 frontend tests PASS, Svelte check zero errors/warnings, two targeted
product-profile Rust fixtures PASS, two structure fixtures PASS, structure/G0
checks PASS, and Windows release/NSIS build PASS (existing compiler warnings).
No UI, game, installer or Computer Use was launched. Runtime acceptance remains
NOT_EXECUTED for this change.

Physical staging completed from clean Local source
5b578653bd7d576ac92a90e56eb06137e97efb05. The actual checkout root now contains
RUN.cmd, App and empty Data. All four App payload SHA-256 values match the
receipt App/build-info.json, and each copied payload has one filesystem link.
The candidate export is
workspace/packages/lazy-tester/0.1.0-5b578653bd7d-20261026T075700753Z.
The initial staging attempt correctly refused Cargo's two-link source executable;
the corrected reader copies its bytes to an independent file, now covered by
the staging fixture. Existing accounts/worlds were neither read nor moved.
Authenticode remains NotSigned; this is a development candidate, not a release.

Cleanup inventory: apps/desktop/src-tauri/target contains 22,912,765,268 bytes
(21.34 GiB), is Git-ignored/untracked, has a Cargo CACHEDIR.TAG, no reparse points
and no running process below it. Both guarded and literal-path removal attempts
were rejected by automatic execution policy as "blocked by policy". Nothing
was deleted; do not report reclaimed space. Toolchains, old profile/capability
folders, worlds and historical evidence remain preserved. The separate current
build cache under workspace/builds/lazy-tester-layout-20261026 is retained.

The physical staging scope is complete. Cleanup remains blocked by execution
policy; no repeated deletion attempts or alternate deletion mechanism is authorized
by that rejection. Next: resolve the blocked old compiler-cache removal before
claiming disk-space cleanup. Do not resume Minecraft/RAM experiments incidentally.

## Previous bounded outcome — App/Data layout and persistent product profiles

Source parent: d216830bd0e5e4923399db34235e0ac84bc22885.
The operator approved a visible Lazy Tester root with App and Data, an automatic
relative default, user-selected locations and verified reattachment after moves.
product_profiles.rs is the single owner of location/catalog logic; the existing
session controller still owns live games and storage handles. It reuses the
bootstrap constructor through create_private_slot_at; DLL ABI/bytes, identity
checks and singleton/FPS policy remain unchanged.

New installer roots use <chosen-root>/App, with a visible per-user default at
%USERPROFILE%/Lazy Tester/App. Existing flat installs are preserved using either
the executable or a retained data-location.json pointer, including upgrades whose
old executable was removed before the preinstall hook. SetOutPath is refreshed
before file extraction. Installer identity, idle admission and data preservation
remain intact; no hook moves/deletes account or world data.

Runtime creates Data/Profiles/Minecraft-1..5 only after supported game/DLL/host
validation and idle admission. A create-once Data/profiles.json binds five stores
to current-user and directory identities. App/data-location.json stores relative
Data by default, or an explicitly selected absolute location. A saved unavailable
location never falls back to fresh profiles. Complete catalogs repair an absent
pointer before launch; partial/corrupt/replaced data is preserved and refused.
Root App/Data/Logs are ignored and refused by source structure checks.

Existing reviewed pilot profiles are discovered from bounded directory metadata
and adopted in place before fresh setup, without finding developer launcher folders.
Their original reviewed digest and existing HKCU stores remain mandatory. Invalid
old bindings block fresh replacement. The earlier automatic-adoption/English drafts
are now incorporated under the operator's current implementation authorization.
No old accounts, worlds, profile directories or installed programs were moved.

Storage exposes actual root/five paths through a strict typed DataLocation command,
Open folder, Locate data and initial Choose data folder. Locate validates an
existing product catalog before atomically changing the pointer. Choose refuses
replacement of an existing product/legacy profile set. Dialogs and heavy operations
share the manager lock; the controller/game idle gate is rechecked around selection.
All mounted UI copy and the touched storage/binding errors are English. This is
functional location UI, not acceptance of the earlier visual design.

## Actual evidence

- Six targeted Windows Rust fixtures PASS: two product-store/location tests and
  four original-binding tests. Five temporary filesystem/HKCU stores reopen with
  the same identities; same-volume directory relocation preserves identity;
  existing/partial/replaced data and corrupt/missing custom locations are refused.
  Only own disposable fixture registry keys/files were removed.
- Svelte check: zero errors/warnings. 29 frontend tests in seven files PASS.
- Two repository structure/link fixtures, root G0 and structure PASS.
- Actual Windows x64 Tauri release and NSIS construction PASS with the native
  package feature and --locked. Existing 104 native warnings remain.
- Build output: workspace/builds/lazy-tester-layout-20261026/target.
  final manager SHA256 55f86b74a6fcb0f7dd4020ad3597cae0a812754f6f544e2814503fd83a76ab16;
  final installer SHA256 7f26ba1932c62d29229b6cf1ec0d74e277edd895cf7491b4d1f44e051ac155c9.
  Authenticode status NotSigned, inspected rather than inferred.
- Actual clean-source npm run package:windows PASS at
  6760526c730a4c2161ae09e386fe79036b395873, including the existing desktop builder
  and immutable export. Receipt is under ignored workspace/packages/lazy-tester/
  0.1.0-6760526c730a-20261026T071623990Z. Exactly three files were verified; the
  exported installer checksum matches its manifest and the source SHA is exact.
  This follow-up changes only the handoff; no newer binary acceptance is implied.
- No candidate GUI, installer, Minecraft, Computer Use, UAC, driver or service
  execution. No real account/token/world contents were read or copied. The old
  Lazy Tester PID 20904 was observed by metadata only and left running unchanged.

## Limits and remaining work

New-store construction is not fresh-PC Minecraft login/gameplay proof. Runtime
acceptance of installed default/custom/silent paths, same-volume Locate dialog,
restart, upgrade/uninstall and five-client gameplay remains NOT_EXECUTED.
The prior five-account pilot acceptance does not validate this catalog or UI.

Same-volume reattachment is implemented, not a Move data copy engine. Cross-volume
moves, individual-slot moves, migration of old pilot data into Data, backup/restore
and destructive reset are unfinished. Copies with changed identities are refused;
no token export or account cloning exists. Partial setup is preserved; an interactive
repair flow is unfinished. Manually moving App does not repair Windows shortcuts.
Current DLL limits remain: bounded ASCII paths, ordinary local storage, supported
unsplit AppData and the exact reviewed Minecraft executable. Existing preferences
still use their previous private path; they were not silently migrated.

The active native DLL source remains under acceptance/bootstrap-ordering. Settings
editing, final Play UI design, source-owner promotion and signing/public release
are separate remaining work, not solved by folder changes. See the canonical
application/data policy in docs/system/data-layout.md.

## Next — one action

Complete safe data relocation and partial-setup recovery in the same product
profile owner, preserving existing identities/HKCU/account data, before offering
Move data in the UI. Retain explicit unsupported cases rather than silently
creating replacement profiles. Do not launch/control the desktop while the
operator is working; integrated acceptance needs an available session.
