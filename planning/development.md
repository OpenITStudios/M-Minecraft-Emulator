# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Full goal: modular five-profile native control center, manual launches, one active
viewport/four previews, storage/cache management, archival and safe packaging.
Renderer/RAM experiments remain PAUSED; GitHub Actions remains entirely OFF.
Previous goal turn: progress (native installer source/package built at parent).

## Current bounded outcome — explicit existing-profile binding

Source parent: ce3fd44ed0d831ff29ec6ed71429f6bb990e0ed5.
Native installer identity/preservation decision and its evidence remain there.
Startup recovery is b370acd, storage/cache 87b6576, previews d36a536; those separate
milestones are not integrated product/gameplay acceptance.

The installed product now exposes a typed binding report and a simple first-run
panel. The user selects the OLD LAUNCHER FOLDER; implementation filenames are not
required knowledge. Backend reads only its fixed
independent-bootstrap/replay-profile-five.bin, bounded to 288 bytes, validates the
reviewed hash, original folder IDs and existing scoped HKCU keys through the
original replay validator, then publishes only that metadata privately.
The location is LocalAppData/MinecraftInstanceManager/NativePreview/
profile-binding-five.bin, outside the installer and alongside the existing
preferences directory. No account/token/world/settings contents are copied.
The contract records this location; the installer still excludes capability files.

The existing bootstrap owns semantics in independent_bootstrap/profile_binding.rs.
Thin Tauri command -> native_clients wrapper -> same replay validator. There is
no second identity scheme, lifecycle authority, account store or setup-complete
flag. The original profile identities remain authoritative. Installed five-profile
launches read the private binding, without falling back to executable-adjacent
capabilities. Private developer preview candidates retain their original route.

Import refuses while a retained session-controller gate exists or cannot be
queried, checks again after folder selection, and never starts a controller/game.
Ancestor directory handles remain pinned through private read/publication.
Create-new temporary + flush + same-directory MoveFileEx without replacement
publishes complete metadata. Existing identical bytes are idempotent; different,
malformed or inaccessible metadata is preserved/refused, not silently replaced.
An interrupted temporary does not become canonical. Folder-dialog cancellation
only returns current status. Unknown/corrupt canonical metadata requires deliberate
repair; no automatic reset is provided.

The UI waits for a strict binding report before enabling new launches/storage
measurement. Status is inspected at startup or explicit retry, not polled/scanned.
Existing game selection/recovery remains independent. A configured binding means
only that identities can be opened, not sign-in/game readiness.
Existing-profile reopen now uses existing_private_temporary and NEVER creates a
missing Local/Temp during inspection. Explicit new-profile preparation remains
separate. Shared Known Folder/root validation was extracted once as replay_roots.

## Executed evidence

- Svelte check: zero errors/warnings.
- All 219 frontend/source tests in 25 files PASS. New report test rejects conflicting
  status, unsupported schema/fields and an invented authenticated claim.
- Three targeted Windows binding fixtures PASS: complete create-once/idempotent
  publication, refusal to replace different/invalid metadata, malformed-size
  refusal, and read-only missing-TEMP behavior. No host profile policy is enabled
  in tests. An initial fixture caught missing OpenOptions write access; fixed.
- Existing five-slot replay regression PASS: preserves original pair/synthetic
  sentinels and refuses changed extra-slot identities/invalid capabilities.
- Actual Tauri x64 release + NSIS installer build PASS after final folder-picker UI.
  Existing 104 desktop warnings remain. No dependency versions/lockfile changes.
- Native package source check PASS; generated installer script still includes
  only reviewed DLLs/contract and excludes profile capability files.
- Manager SHA-256 (post Tauri patch):
  6e19cccdecb156a2bc09640f5e0b95cc03c52c93bd47edc2d26d56a3cea068a2.
- Installer SHA-256:
  f9b098c1766ca766359a8a4f45579bd8d9ff634e289a98dade1e8b24881cf4ce.
- Installer Authenticode: NotSigned. No publication/release/signing.
- No actual profile import, private binding write, installer execution, candidate
  GUI, Minecraft, Computer Use, UAC or trace. All new filesystem mutations were
  disposable fixtures or ignored build outputs. Native replay fixtures use only
  synthetic data/keys, not installed accounts/worlds.

Actual folder-dialog/import/cancel, installed launch with original profiles and
login/world/gameplay persistence: NOT_EXECUTED. Fixture publication plus replay
validation is not proof of the integrated production import workflow.

## Remaining full-goal work

- Integrated installer/import/startup/repair/uninstall acceptance and real
  Minecraft workspace/storage/restart proof. Hidden DWM registration is not live
  pixels/input/FPS proof. Offscreen Education host simulation, DPI/dialog/minimize/
  crash recovery, actual cache cleanup/login persistence remain NOT_EXECUTED.
- Taskbar/browser-shell and Task Manager grouping remain unproven; games continue
  to own separate OS processes.
- Physical legacy source archival/retirement after tracing shared consumers.
  Bootstrap-ordering still contains ACTIVE DLL source, not obsolete test code.
- Primary D: checkout is older and was not reset or overwritten.
- Generic new-machine profile provisioning remains unsupported. This import
  accepts only the reviewed existing five-profile capability/game-build recipe.
- Cross-binary retained-controller handoff remains unsupported; a different
  binary preserves/refuses an active controller. No live binary replacement.
- Different/corrupt private binding is refused; HTTP-only cache cleanup remains
  explicit disk maintenance, never broad TEMP/profile deletion.

## Next — one action

Trace and archive the retired unmounted desktop screens and their exclusive
consumers/tests, keeping shared native owners and account-bearing local data
intact. Use the existing archive snapshot/register, not duplicate active apps.

## Preserved baseline

Archive branch codex/archive-before-lazy-tester-20261026 pins
d166b5c8b24e8435f2c84675cb00a61c753aeb6a. Only Local receives product work.
The prior five-client pilot was operator-confirmed with five licensed accounts in
one Education Host World, persisted login/world data and 144/10 FPS after restart.
New UI/controller/installer acceptance is separate. Accepted DLL bytes and profile
paths/presets/FPS policy are unchanged. No installed data was migrated or reset.