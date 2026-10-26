# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Full goal: modular five-profile native control center, manual launches, one active
viewport/four previews, storage/cache management, archival and safe packaging.
Renderer/RAM experiments remain PAUSED; GitHub Actions remains entirely OFF.
Previous goal turn: progress (retired frontend archived at the source parent).

## Current bounded outcome — synchronize local source without losing local files

Source parent: 3a925a3b031125dacc36a5a6deeaf51e62f49fbd.
The primary D:/Work/AI Stuff/Minecraft-Emulator checkout was on Local at
07fd0fdc0e3c8a7bd8a478e65463bd6504e4bddd; the managed implementation worktree was
detached at 08e17b9012ede874f6fc304a630ce3579624db33 with delivered changes in its
working files. Both are now synchronized to the published Local source.
Primary remains the Local branch; managed remains detached at the same source.

Canonical source was fetched explicitly from OpenITStudios. Primary ancestry was
verified before a fast-forward. It had no staged/tracked edits, but four untracked
C++ research files collided with incoming tracked paths. Their original bytes
were moved to ignored workspace/private/lazy-tester-root-sync-20261026/primary,
with file sizes/SHA-256 verified after the move and receipt.json retained.
Incoming tracked copies were then checked out. No research was resumed.

All 20 previously untracked delivered files in the managed worktree matched the
target Git blobs before changing its HEAD/index. Other differences were checked;
only an existing README line-ending issue remained. A mixed reset aligned Git
metadata without overwriting working files. No hard reset, clean, ignored-output
removal or account/world operation occurred.

The old origin URL was MIVUBI-STD. Initial parsing mistook a redirect response for
repository metadata; no mutation was made on that assumption. The actual API
response is Moved Permanently to repository ID 1412902903, matching OpenITStudios.
The old URL is retained as legacy-mivubi; origin now uses the explicit canonical
URL, and Local tracks origin/Local. No remote push or main update occurred.

bootstrap-ordering/README.md had a committed CRLF blob despite the repository's
Markdown LF policy, producing a dirty checkout even with identical text. Raw
copies from both checkouts are saved with matching SHA-256 in the same ignored
archive. This increment normalizes only CRLF to LF; ignore-space-at-eol comparison
confirms unchanged content. The following documentation commit aligns both local
HEAD/index states without overwriting any other files.

## Executed evidence

- Canonical Local SHA and ancestry checks PASS; primary fast-forward PASS.
- Four preexisting source backups verified against their original SHA-256.
- Twenty managed worktree delivered files matched incoming Git blob IDs.
- README content comparison ignoring EOL PASS; raw originals preserved privately.
- Primary root verifier --g0 PASS: seven reachable frontend files.
- Primary repository structure/document-link check PASS.
- Runtime/GUI tests, fresh dependency install and installer execution were not
  needed for this source/Git metadata synchronization and were NOT_EXECUTED.
- Existing local node_modules, ignored native payloads, build outputs, capability
  files and game data were not removed or replaced.
- No candidate GUI, Minecraft, Computer Use, UAC, trace, driver, service or
  installed-program action. No credentials/token/world contents were read.

## Current product source and prior evidence

The active frontend has seven reachable files; 35 old UI/helper files, 20 exclusive
test files and three old UI-only cases are archived in immutable pre-removal
5601c354fb91db78a2697ceb6ec710590d7d17be. The earlier archive branch still pins
d166b5c8b24e8435f2c84675cb00a61c753aeb6a. Legacy Rust/shared helpers remain gated.
No obsolete source mirror was added.

At 3a925a3: 25 active frontend tests, seven repository fixtures, Svelte/frontend
build, source G0/structure and actual x64 NSIS build passed. This increment changes
no compiled source or dependencies, so no new binary/runtime acceptance is claimed.
The latest preserved installer candidate is candidate-installer-clean-frontend:
manager da4fc32ad3d6f63de7f52fc70f063e2c91e6975b0568af5005d83246fe54857c;
installer 84e7e7bd489da702b5e88a4d30161e2c66440a5a991f2f4f228002d8e5f92555.
Unsigned, NOT_EXECUTED. All binaries/receipts remain ignored/local.

Installed first-run folder selection imports only 288 reviewed identity metadata
bytes into the private NativePreview binding, never account/world contents.
Binding fixture/replay evidence belongs to 5601c354. Real folder-dialog/import/
cancel, installed launch/login/world persistence remain NOT_EXECUTED.
Configured identity is not sign-in/game-ready proof.

Native workspace still needs real Minecraft pixel/input/FPS and offscreen
Education-host proof. Hidden DWM fixtures and old five-client acceptance do not
substitute for current integrated acceptance. Taskbar/browser-shell and Task Manager
grouping remain unproven; game processes remain separate. Cross-binary retained
controller handoff and generic new-machine provisioning remain unsupported.
Corrupt/different private binding is refused, not automatically reset.

## Remaining full-goal work

- Default dev:app still uses plain tauri dev/base configuration, while build:app
  correctly selects the Lazy Tester native package. Align the normal developer
  entry with the product feature/config/payload so it exposes first-run binding
  instead of the private adjacent-capability fallback.
- Available manual runner Node is 24.19.0, system Node 24.14.1; declared floor
  remains ^24.22.0. Fresh engine-strict declared-toolchain reproduction is not proven.
- Active native DLL source still resides under bootstrap-ordering acceptance
  tooling. Trace build/probe consumers before promoting it to a product owner.
- Integrated installer/import/workspace/storage/gameplay/restart/repair/uninstall
  acceptance remains NOT_EXECUTED. No installed data was changed during this work.
- Further legacy retirement must preserve shared helpers and account-bearing
  disks, profiles, worlds, registry stores, services, installed apps and snapshots.

## Next — one action

Align the default local development entry with the reviewed Lazy Tester native
configuration/feature/payload route, preserving the explicit private-preview path
and manual game launch. Verify resource placement without opening the GUI.

## Baseline

Prior independent pilot was operator-confirmed with five licensed accounts in one
Education Host World and persistent login/world/FPS after restart. New UI/controller/
installer acceptance remains separate. Accepted DLL bytes, original profile paths,
approved presets and foreground144/background10 policy are unchanged.