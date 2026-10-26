# Current Development Handoff — Lazy Tester

Local only; main release-reserved; GitHub Actions entirely OFF. VMware,
Sandboxie and RAM/renderer experiments remain PAUSED. No account/world migration.

## Current bounded implementation — Play and familiar Settings

Parent c097363b215ab1d66ef409352500900980dd2d87. The user explicitly approved
moving the reviewed design into the actual application, requested a prominent
Advanced disclosure, consistent controls, and High/Low graphics only.

The mounted Svelte owner now combines launchers and session switching in Play.
All five numbered launchers remain visible, including separate-window mode.
Settings has General and Data Storage with a shared selected launcher. Graphics
lives in General. Data Storage exposes actual bounded metadata, selected profile
folder, guarded web-cache cleanup and emphasized Advanced options. No illustrative
world sizes, fabricated installation paths or simulated successful actions ship.

The existing preset writer supports High and Low, both at 12 chunks and unchanged
simulation distance. High enables the reviewed effect keys; Low retains the
previous reduced preset. manager_settings.rs owns graphics.json (five choices).
The sole session actor serializes ApplyGraphicsPreset with launches and rejects
manual writes while games are active. Existing options backups/identity checks
are preserved. Launch reapplies the saved selection. App staging preserves the
new preferences alongside performance.json and data-location.json.

Native executable icon now uses the same LT identity as the interface. Windows
frame appearance and actual game composition still need interactive acceptance.

## Evidence executed for this source change

Svelte check: zero errors/warnings. 33 frontend tests PASS. Production frontend
build PASS. Windows Rust cargo check --all-targets with lazy-tester-package PASS
(existing warnings retained). Targeted High/Low rendering and graphics preferences
tests PASS. Compiled Svelte mocked-Tauri browser regression PASS: five launchers,
High/Low/FPS save, navigation during inspection, partial storage metadata, disabled
destructive actions, separate-window return, offline/recovery, existing-data
reconnection and no unintended launch/delete. 920x620 and1366x900 bounds checked.
Synthetic screenshots: workspace/runs/lazy-tester-ui-regression/.
Repository structure and G0 contracts PASS. Full unrelated Rust suite not rerun;
previous DACL/pipe/rustfmt issues remain outside this change.

## Explicitly unavailable functionality

Full account clear, world export, game-settings reset/restore, background audio
control, automatic game shutdown and data migration have no accepted backend.
Their UI is disabled/explanatory. No real account/world contents were read,
deleted or copied. Clear All Data must not be enabled until exclusive profile
ownership, sign-in cleanup and isolation from Windows/other launchers are proven.
The intended full-clear scope includes worlds and launcher-local backups; external
exports and the Minecraft installation are preserved. This differs intentionally
from cache/settings-only cleanup. No real destructive operation was executed.

Existing close behavior is preserved: the manager releases game windows and
keeps Minecraft running, with its existing native confirmation while embedded.
High graphics visual acceptance and all gameplay/license/multiplayer acceptance
for this revision are NOT_EXECUTED. Source/UI checks are not runtime proof.

## Packaging / continuation

Built Windows/NSIS from clean Local source
1c8a3cd129cebf41472facb25021ed6011bc70a9. Package command completed successfully
and staged the actual root App using its guarded backup/swap path. App build-info
matches this source and all four payload hashes were independently verified.
Installer SHA256485144c526013e95928832d05e4fd1319a56aa8a74752a407f6b353382eeacc7.
Candidate: workspace/packages/lazy-tester/0.1.0-1c8a3cd129ce-20261026T122616256Z.
Build log: workspace/runs/play-settings-implementation/package.log.
The package manifest retains signing NOT_VERIFIED, installedAcceptance and
minecraftAcceptance NOT_EXECUTED, publication DISABLED. No installer or GUI was
executed and no game was started. RUN.cmd now resolves to the refreshed App.

No Lazy Tester or Minecraft processes were observed before/after packaging.
Existing data and preference pointers were preserved; no controller was killed.
Submodule remains abd4c240031cfb46666daa74ca47ed4e48b561e4;
second worktree remains detached at47a45f9c7d4a8ce6799d56997fd38e4b4df49ce7.
Repository structure and G0 contracts passed again after staging. The following
handoff commit changes documentation only; the packaged product SHA above is
the precise source of the current App, not an inferred gameplay acceptance.

Next: implement and test bounded game-settings backup/reset/restore in the
existing options owner before enabling those controls. Account-clear isolation
remains an independent unresolved P0 prerequisite; current UI is not an account
readiness authority. STOP after the current source/package handoff.
