# Current Development Handoff — Lazy Tester

Local is the only active product branch. Sole current handoff.
Full goal: modular five-profile native control center, manual launches, one active
viewport/four previews, storage/cache management, archival and safe packaging.
Renderer/RAM experiments remain PAUSED; GitHub Actions remains entirely OFF.

## Current bounded outcome — align normal development with the product

Source parent: 8caefcdf31d262ea0ecc56dd9a2d8e967e38fc2d.
The root npm dev and DEV.cmd dev already delegate to desktop dev:app. That command
now selects dev:lazy-tester rather than plain tauri dev. Development and packaging
use the same icon preparation, package contract, reviewed DLL staging, x64 target,
Lazy Tester configuration and lazy-tester-package feature with Cargo --locked.
The existing package verifier rejects drift between the two routes. An explicit
dev:preview retains the developer-only adjacent-binding route under its separate
preview configuration/feature. No new lifecycle owner or runtime API was added.

LAZY_TESTER_NATIVE_PAYLOAD_DIR must point to the reviewed two-DLL directory.
The shared stager checks both hashes before writing, refuses stale output and
copies no adjacent capabilities. Product first-run binding remains private and
manual; opening the app never launches a game. README documents both routes.
No game/DLL bytes, installed identity, profile paths, presets or FPS policy changed.

## Executed evidence for this change

- Native package verifier PASS; 11 targeted routing/package tests PASS (two files).
- Root source verifier --g0 and repository structure/link check PASS.
- Actual payload staging PASS; only the two reviewed DLLs were copied.
- Windows x64 cargo check --locked --features lazy-tester-package with the actual
  product TAURI_CONFIG overlay PASS; 104 existing warnings remain. Build-script
  resource copying placed both DLLs in the debug independent-bootstrap directory;
  both SHA-256 values match the accepted runtime policy and no capability is there.
- Manual source tools used workspace Node 24.22.0 and npm 12.2.0. Earlier notes
  that no pinned Node was available were stale. Fresh engine-strict installation
  and complete dependency reproducibility remain NOT_EXECUTED.
- tauri dev itself, candidate GUI, Minecraft, installer execution and Computer Use
  were NOT_EXECUTED. Cargo check is compilation/resource evidence, not GUI or
  Minecraft acceptance. No account/token/world contents were read or copied.

## Product source and preserved prior evidence

The active frontend has seven reachable files. The 35 old UI/helper files,
20 exclusive test files and three old UI-only cases remain recoverable at
5601c354fb91db78a2697ceb6ec710590d7d17be. Earlier archive branch
codex/archive-before-lazy-tester-20261026 pins
 d166b5c8b24e8435f2c84675cb00a61c753aeb6a. Legacy Rust/shared helpers remain gated.
No account-bearing disks, profiles, registry stores or installed apps were removed.
Both checkouts were synchronized cleanly at the source parent; prior colliding
research originals remain hash-verified in ignored workspace/private archives.

At 3a925a3b031125dacc36a5a6deeaf51e62f49fbd the Svelte/frontend build, 25 frontend
and seven repository fixture tests, G0/structure and x64 NSIS construction passed.
This change modifies scripts/tests/docs only; no new installer is claimed.
The latest preserved unsigned candidate is candidate-installer-clean-frontend:
manager da4fc32ad3d6f63de7f52fc70f063e2c91e6975b0568af5005d83246fe54857c;
installer 84e7e7bd489da702b5e88a4d30161e2c66440a5a991f2f4f228002d8e5f92555.
Installer execution remains NOT_EXECUTED; binaries and receipts stay ignored/local.

Installed first-run selection imports only 288 reviewed profile identity metadata
bytes into the private NativePreview binding, never account/world contents.
Binding fixtures/replay passed at 5601c354; real installed dialog/import/cancel and
launch/login/world persistence are NOT_EXECUTED. Configured is not game-ready.
Corrupt/different bindings are refused and preserved. Generic new-machine profile
provisioning and cross-binary retained-controller handoff remain unsupported.

## Remaining full-goal proof

The prior independent pilot was operator-confirmed with five licensed accounts in
one Education Host World and persistent login/world/FPS after restart. That does
not validate the new UI/controller/installer integration. Real five-client preview
pixels, direct input/focus, 144 active/10 background, offscreen Education-host
behavior and UI recovery remain NOT_EXECUTED. Taskbar/browser-shell and Task Manager
grouping remain unproven; Minecraft remains separate OS processes.
Integrated install/import/workspace/storage/restart/repair/uninstall acceptance
remains NOT_EXECUTED. Active compatibility DLL source still lives under acceptance
bootstrap-ordering tooling; relocation is not a prerequisite for this acceptance.

## Next — one action

Run one integrated Lazy Tester acceptance session when the operator is available:
use the preserved candidate, select the original launcher folder, manually launch
five existing profiles and verify workspace selection/previews/input, retained
sign-in/worlds and foreground144/background10 in one Education Host World.
Do not open the GUI or control the desktop while the operator is working. Preserve
all data and record failures in this handoff instead of expanding general cleanup.
