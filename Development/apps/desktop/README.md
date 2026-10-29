# Lazy Tester Desktop — Current Product

This is the current Tauri 2 / Svelte 5 / TypeScript / Rust application.
The mounted UI is `src/LazyTesterApp.svelte`, not the historical
`UnifiedManagerApp.svelte`. Native Windows Minecraft Education has five
fixed profile identities and an explicit manual launch path; the old
Sandboxie/VMware provider route is retired.

## Canonical source owners

| Area | Owner |
| --- | --- |
| Play/Settings/Data Storage and UI action dispatch | `src/LazyTesterApp.svelte` |
| Pure pending-action eligibility and inspection scheduling | `src/app/inspectionScheduling.ts` |
| Fixed five-launcher rail, visual actions only | `src/LauncherRail.svelte` |
| Health, recovery guidance and authored user messages | `src/SessionHealthNotice.svelte`, `src/app/presentation/` |
| Validated desktop protocol | `src/app/bridge/lazyTester.ts`, `workspaceSurface.ts` |
| Only raw Tauri invoke boundary | `src/app/bridge/invokeRuntime.ts` |
| Game lifecycle / retained process identity | `src-tauri/src/engine/independent_bootstrap/session_controller.rs` |
| Authenticated same-user controller transport | `src-tauri/src/engine/native_clients_session.rs` |
| Native window recovery and DWM previews | `src-tauri/src/engine/workspace_surface.rs` and child modules |
| Profile data location, catalog and migration | `src-tauri/src/engine/independent_bootstrap/product_profiles.rs` |
| Installer and preservation policy | `../../engine/distribution/lazy-tester-package.json` |

An observed running process or registered thumbnail does **not** certify
gameplay, sign-in, visible pixels or multiplayer. Minecraft owns login,
worlds and rendering; Lazy Tester never stores credentials or force-closes
unverified games.

## Profile layout and legacy compatibility

- Current schema 2 profiles: `Data/Launchers/Minecraft-1..5`.
- Catalog: `Data/profiles.json`.
- Canonical pointer: `Data/Settings/data-location.json`.
- Legacy schema 1 profiles: `Data/Profiles/Minecraft-1..5`.
- Legacy flat App pointer: `App/data-location.json`, consulted only when
  the canonical pointer is absent and retained by packaging preservation.
- Preferences: `Data/Settings/`, with first-run 144/10 FPS limits.

These are storage identities, not instructions to copy, reset, relocate,
or regenerate any account or world data. Read
[Data layout](../../../docs/system/data-layout.md) before any migration.

## Developer commands

Use only the pinned versions in `../../toolchain.json`.
With genuine dependencies and a compatible developer environment:

```bash
npm ci
npm run typecheck
npm test
npm run build:frontend
npm run check:lazy-package
npm run verify:source
```

`npm run dev:app` uses the current Lazy Tester native configuration.
Node/Vitest/Svelte verification does not prove Windows app integration or
Minecraft Education playability. Installed signing, Windows desktop
acceptance, and five-client gameplay remain separate evidence gates.
No CI runs are enabled; the only development branch is `Local`.

See the [implementation map](../../../docs/system/implementation-map.md),
[current handoff](../../../planning/development.md), root `AGENTS.md`
and `GITHUB_RULES.md`. Retired designs remain in Git history, not active UI.
