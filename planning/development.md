# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Full goal: a modular five-profile native desktop app suitable for distribution,
with automatic setup, clear data ownership, English UI, editable settings and
per-profile storage management. Renderer/RAM experiments remain PAUSED; CI is OFF.

## Current bounded outcome — separate source, developer output and distribution

Source parent: e09fd620e6fc1a0a4f9b13856819d0ba5370aed7.
The operator requested a professional project root with development and release
materials kept separate. There is one source tree, not developing/release copies.
Existing apps/engine/tooling/docs/planning/experiments owners stay in place.
Generated local material stays ignored under workspace or tool-native output paths.
Installed game/account/world locations are not moved by this repository change.

Root package:windows now invokes apps/desktop/scripts/package-lazy-tester.mjs.
The wrapper admits clean committed Local source, or a detached worktree at the
exact Local revision; rejects tracked/untracked drafts; checks declared Node/npm,
product configuration and disabled publication; then runs the existing desktop
build:app. It rechecks source identity/cleanliness before exporting. Compiler
output overrides must be absolute and inside this checkout, with ordinary parents.
Desktop build:app remains the single native installer construction route.

Only a fresh workspace/packages/lazy-tester/<version>-<shortSHA>-<UTC>/ is exported:
Lazy-Tester-<version>-windows-x86_64-setup.exe, SHA256SUMS.txt and
package-manifest.json. There is no directory-recursive copy. Existing candidates
are not merged or overwritten; receipt is written last. It records the full source
SHA and installer checksum and always labels a DEVELOPMENT_CANDIDATE, not a
release: signing NOT_VERIFIED, installed/Minecraft acceptance NOT_EXECUTED and
publication DISABLED. This receipt is not another runtime readiness flag.

Structure checks now also reject tracked Tauri icons/gen/binaries/package-resources
outputs. Existing top-level allowlists refuse develop/release source copies.
README, workspace README and the existing artifact-storage/distribution documents
explain the actual source/build/candidate/installed-data split. Stale workspace
instructions about CI debug downloads/Sandboxie packaging were corrected.

## Actual evidence

- 13 targeted package/export/routing tests across three frontend test files PASS.
- Two repository structure/link fixtures PASS; G0 and structure checks PASS.
- Export fixtures verify exactly three output files, installer checksum/bytes,
  no adjacent private/source copies, no reuse, invalid identity/executable refusal.
- Disposable Git fixture verifies clean Local/detached admission and tracked,
  untracked and other-branch refusals. No product branch was changed by fixtures.
- Actual wrapper entry on the dirty primary checkout refused before build invocation.
- End-to-end invocation through a fresh native build remains NOT_EXECUTED for
  this wrapper. Existing NSIS construction evidence is from 3a925a3, not this change.
- No installer/GUI/game execution, Computer Use, driver, service or installed-data
  action. No account/token/world contents were inspected or copied.

## Preserved drafts and product limitations

The earlier automatic old-profile connection and English-text changes in
independent_bootstrap/profile_binding.rs, profile_storage.rs and LazyTesterApp.svelte
remain uncommitted drafts on the primary checkout. The operator stopped that work
for design discussion; they are not part of this packaging/layout commit.
Their bytes were preserved. This commit does not claim automatic setup is shipped.

The current engine still uses reviewed machine-specific pilot identities and an
exact reviewed Minecraft executable hash. Fresh-PC five-profile provisioning,
custom data location/migration, Unicode paths and redirected/split AppData need
implementation/review. Do not present the app as compatible with every PC yet.
Active compatibility DLL source still resides in acceptance/bootstrap-ordering;
source-owner promotion must trace all build/probe consumers before relocation.

The prior pilot had operator-confirmed five licensed accounts in one Education
Host World, persisted login/worlds and 144/10 FPS. New UI/controller integration,
preview/input/offscreen host behavior and install/import/update/uninstall remain
NOT_EXECUTED. User requested further product design before interactive acceptance.
No earlier runtime result is promoted to new-app or cross-PC acceptance.

## Next — one action

Agree the fresh-PC setup and storage contract with the operator: supported Windows/
Minecraft builds, permanent default data location, five stable profile identities,
old-profile adoption and safe user-selected relocation. Implement that owner only
after the design discussion; keep the paused local drafts out of unrelated work.
