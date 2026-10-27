# Emulator Stable Context

Current product: **Lazy Tester**, a Tauri 2 / Svelte desktop manager for up to
five individually licensed Minecraft Education Windows sessions. The game
itself owns authentication, rendering, worlds and networking. The app retains
one verified game process/session controller, persistent private profile
Roaming/Local/HKCU handles, a Windows DWM preview owner, native compatibility
DLLs and explicit normal-exit/recovery handling.

The last live Windows observation recorded in the source handoff found one
MC02 normal-exit failure. Subsequent source corrections are not Windows/game
acceptance. Every actual execution requirement remains `NOT_EXECUTED` unless
measured on that exact revision.

## Project goal

Improve stability and efficiency without patching the original game or
copying credentials/tokens, then develop a platform-neutral orchestration
core with explicit Windows and macOS adapters. The existence of an official
Minecraft Education Mac application does not prove five independent accounts,
session storage or interactive window composition there.

Retired: the vendored Sandboxie trees/fork, native box/Client02 bridge,
VMware VMX/Guest Agent runtime, legacy packaging and unmounted frontend
and Tauri command routes. Legacy sources remain in Git history, not in
the active runtime or installer. Neither installed software nor any
account/world/VM disk is removed by repository commits.

## Current owners

- `Development/apps/desktop/src/LazyTesterApp.svelte`: mounted UI only.
- `Development/apps/desktop/src-tauri/src/commands/native_clients.rs`:
  allowlisted Tauri commands; one manager resource admission gate.
- `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`
  and `independent_bootstrap/session_controller.rs`: native five-profile
  launch, verified process lifetime, normal exit and policy ownership.
- `Development/apps/desktop/src-tauri/src/engine/process_identity.rs`:
  untrusted process/window value type; verification stays in native owners.
- `Development/apps/desktop/src-tauri/src/engine/workspace_surface.rs`:
  Windows surface placement, DWM preview, restoration and helper checkpoint.
- `Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs`:
  authenticated retained controller connection.
- `Development/engine/runtime-core/src/independent_engine/`:
  platform-independent contracts, not an alternate game launcher.
- `Development/tooling/acceptance/windows/bootstrap-ordering/`:
  **ACTIVE** native DLL source, despite its historical directory name.
- `Development/engine/distribution/lazy-tester-package.json`:
  native Windows installation and data-preservation contract.

## Source and deployment boundaries

`Local` is the only active GitHub development branch; `main` is
release-reserved. GitHub Actions are disabled by operator decision.
Remote GitHub review can establish source layout/contract changes only,
not compiler acceptance, installed Windows behavior or Minecraft gameplay.
No local-PC tests until explicitly authorized.

Never silently change the old installed application identifier, migrate
accounts/worlds, remove snapshots or reinstall legacy services.
`planning/development.md` owns current status and next action;
`docs/system/legacy-retirement.md` records retired source provenance.
