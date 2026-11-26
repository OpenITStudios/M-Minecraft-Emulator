# Lazy Tester Desktop — Current Product

This is the current Tauri 2 / Svelte 5 / TypeScript / Rust application.
The mounted UI is `src/LazyTesterApp.svelte`, not the historical
`UnifiedManagerApp.svelte`. Play keeps the selected game in the workspace; manual window separation is not offered.
Native recovery remains available through the original controller. Healthy running
launchers use their preview and selection highlight; connection and exceptional
status remain visible when attention is needed. The AI & Testing disclosure
appears only for attention or terminal Probe results, not routine activity. General Settings prioritizes frame
limits and audio; existing per-profile graphics preferences remain available in a
collapsed legacy disclosure. Native settings and stored values are unchanged. Profile data-location inspection
runs on Data Storage navigation, not when opening General Settings.
The fixed-height launcher action strip hosts selected-game close review and
pending-close status without inserting a new workspace row. Native close
admission and recovery remain unchanged. Every launcher reserves a bounded
health row so a warning does not resize the rail or shift native previews.
Unconfirmed launch receipts are scoped to the affected slot and block its
retry until a fresh session inspection succeeds.
Native Windows Minecraft Education has five
fixed profile identities and an explicit manual launch path; the old
Sandboxie/VMware provider route is retired.

## Canonical source owners

| Area | Owner |
| --- | --- |
| Play, Settings/Storage state and UI action dispatch | `src/LazyTesterApp.svelte` |
| General Settings forms and read-only product details | `src/GeneralSettings.svelte` |
| Data Storage, inventory and confirmation presentation only | `src/DataStorage.svelte` |
| Pure pending-action eligibility and inspection scheduling | `src/app/inspectionScheduling.ts` |
| App titlebar drag and normal exit request (no custom minimize/maximize buttons) | `src/WindowTitlebar.svelte` |
| Fixed five-launcher rail, visual actions only | `src/LauncherRail.svelte` |
| Health and recovery guidance | `src/SessionHealthNotice.svelte`, `src/app/presentation/` |
| Authored diagnostic categories, stable support codes and user error messages | `src/app/presentation/sessionFeedback.ts` |
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

Minecraft version compatibility is checked for every managed launch. An
unreviewed version leaves existing configured launcher inspection and Settings available, with
the stopped profiles marked as needing attention. Launch remains blocked
until its exact image/callsite is reviewed. The idle controller retains no
game executable handle; each launch resolves the current signed installation
again. Running games retain their original image pins until they exit.
This does not promise automatic compatibility with future Minecraft builds.

The reviewed Education 1.26.33.0 image selects its own observed mutex caller;
the earlier reviewed image retains its original recipe. The existing one-shot
`--lazy-tester-agent-cli` adapter also accepts `close <1..5>` and `close-all`,
using the retained controller's normal close path. These commands never force
terminate Minecraft, sign in, reset profiles or replace their data.

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
npm run check:frontend    # typecheck + Vitest only; no Cargo
npm run check:rust:target # Windows-target Cargo check without all-targets
npm run build:frontend
npm run check:lazy-package
npm run verify:source
```

`npm run dev:app` uses the current Lazy Tester native configuration.
Full Cargo checks and tests for desktop and runtime-core use the
`x86_64-pc-windows-msvc` target, consistent with the scoped Windows developer
check and Tauri development layout. This reduces duplicate host-default
cache trees, but different feature sets and profiles can still have separate
compiled artifacts; no active `cargo clean` is implied.

Node/Vitest/Svelte verification does not prove Windows app integration or
Minecraft Education playability. Installed signing, Windows desktop
acceptance, and five-client gameplay remain separate evidence gates.
No CI runs are enabled; the only development branch is `Local`.

See the [implementation map](../../../docs/system/implementation-map.md),
[current handoff](../../../planning/development.md), root `AGENTS.md`
and `GITHUB_RULES.md`. Retired designs remain in Git history, not active UI.

Workspace placement and preview IPC share one captured geometry per layout pass.
Position, size, target identity and pixel ratio are rechecked after native awaits;
ancestor scroll events reuse the existing dirty scheduler.

Play-wide preview feedback uses the existing fixed launcher action strip;
redundant below-rail guidance is removed. The existing Refresh action remains
available for explicit preview recovery.

Selection/preview action errors with a retained, available workspace reuse the
fixed launcher action strip. Connection and restoration uncertainty stay global;
selected close confirmation takes precedence over noncritical display feedback.
