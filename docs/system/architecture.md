---
id: document.system.architecture
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester — current Windows architecture

This document defines the **active architecture only**. The four-client,
Sandboxie, AppContainer, VMware and Guest Agent designs are historical,
recoverable from prior `Local` commits and the
[legacy retirement register](legacy-retirement.md). They must not supply
a fallback provider or change present-day launch decisions.

## Product and flow

Lazy Tester is a Tauri 2 / Svelte 5 desktop application for **five**
individually licensed Minecraft Education Windows profiles, MC01–MC05.
It uses one official Minecraft installation and persistent private
per-profile storage. The operator explicitly launches selected profiles;
starting the manager does not launch Minecraft.

```text
Svelte UI (presentation and user requests only)
  -> typed TypeScript bridge (validated session contracts)
  -> allowlisted Tauri commands + one manager resource gate
  -> retained native Rust controller (process/session authority)
     -> verified official game image + reviewed native DLL payload
     -> persistent private profile handles and normal lifecycle
  -> Windows workspace/DWM presentation (no lifecycle authority)
  -> real Minecraft Education processes (game owns login/world/render/network)
```

The selected running game is presented in the main workspace. Up to four
other independent native game windows contribute passive preview tiles.
Minecraft remains separate OS processes; Task Manager grouping is **not**
a product acceptance claim.

## One owner per responsibility

- Native lifecycle, launch authorization, close and uncertain outcomes:
  `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`
  and `independent_bootstrap/session_controller.rs`.
- Independent profile files, registry, settings and storage identity:
  the existing `independent_bootstrap/` modules; no credential store.
- Retained controller IPC and exact Windows peer admission:
  `engine/native_clients_session.rs` with its `pipe.rs` module.
- User actions and backend admission:
  `src/LazyTesterApp.svelte`, typed `src/app/bridge/` and
  `src-tauri/src/commands/native_clients.rs`.
- Window layout, DWM previews and restoration:
  `engine/workspace_surface.rs` and its existing child modules.
- Pure portable contracts, not a provider or second state machine:
  `Development/engine/runtime-core/`.
- Official game resolver and reviewed compatibility DLL source:
  `Development/tooling/acceptance/windows/`.
- Distribution and account/world preservation:
  `Development/engine/distribution/lazy-tester-package.json`
  and native Tauri product config.

The [implementation map](implementation-map.md) gives precise routes
to source owners and affected consumers. This architecture is a
boundary contract, **not** a duplicate module inventory or task queue.

## Safety and lifecycle invariants

1. A Minecraft process PID, window or profile folder alone is not a
   signed-in, playable, independent licensed session.
2. No silent adoption of unknown games, token copying, automated Microsoft
   sign-in, implicit game launch or forced process termination.
3. Each profile retains its verified storage identity; migration or deletion
   requires its existing explicit authorization and preservation checks.
4. Native window presentation never becomes a second game lifecycle owner.
   Restore presentation before coordinated normal CloseOne/CloseAll.
5. Unknown or interrupted mutations require inspection/reconciliation,
   never automatic blind replay.
6. Foreground/background FPS values (first-run 144/10) are policy
   **ceilings**, not measured achieved FPS or memory savings.
7. Unsupported operating systems cannot invoke native Minecraft controls.
   macOS five-client support is not implemented or accepted.
8. Runtime and developer build caches are distinct from accounts, worlds
   and installer-preservation data. No unscoped cleanup is authorized.

## Acceptance and engineering discipline

`Local` is the only development branch; `main` is release-reserved.
GitHub Actions CI is entirely disabled. Source reading can verify current
contracts and program structure but cannot establish Rust/Tauri compilation,
Windows installation, native DWM pixels, independent five-client sign-in,
gameplay, networking or performance. Record those as `NOT_EXECUTED`
unless exact-revision evidence exists.

Use the existing `DEV.cmd` / `Development/tooling/` checks where an
appropriate execution environment is actually available; they are not
required to finish independent GitHub source review. Do not create a new
orchestrator, daemon, driver, provider, alternate state ledger or test runner
without a concrete current-product need.

Current work and the **single next step** belong only to
[planning/development.md](../../planning/development.md).
