# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Current goal: a modular five-profile native app with visible application/data
folders, persistent locations, English UI and safe distribution. CI remains OFF.
Renderer/RAM experiments remain PAUSED. Main is release-reserved.

## Current bounded outcome — App/Data layout and persistent product profiles

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
  manager SHA256 839ee86bed96363d50e35ca897863a0014c6b79d37bb9d5f0582e5cf746c4e0a;
  installer SHA256 5c9e891fab503227e74a816d3f4d1a68c7b105a56a8524f25d93c567add36bba.
  Unsigned. Built source precedes the documentation-only handoff update.
- The clean-source packaging wrapper's post-commit export is not claimed yet;
  any later export receipt must name its actual source commit and artifact hashes.
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
