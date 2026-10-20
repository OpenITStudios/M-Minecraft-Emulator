# Minecraft Instance Manager — Experimental Desktop

The sole mounted application is `UnifiedManagerApp.svelte`: an English dashboard
with four cards, 01 native and 02–04 owned. Settings contains the shared Minecraft
executable and owned developer engine location; there is no official-pilot mode
switch or automatic Sandboxie fallback. `CompactManagerApp.svelte` and the older
surfaces remain unmounted historical source; their account-bearing data is preserved.

The canonical target catalog is
[`minecraft-targets.json`](../../engine/isolation-native/owned-runtime/minecraft-targets.json).
Rust admission/observation, the profile planner and UI derive slot identities from it.
Local selections live at `%LOCALAPPDATA%\MinecraftInstanceManager\NativePilot\manager-settings.json`
inside the existing manager data directory. They are preferences, never readiness proof.
Every owned operation rechecks the live engine; the shared Education resolver matches
the selected signed executable to its registered installation and native Windows launcher.

Choose an engine once in **Settings → Choose engine**. The game is detected from
Windows registration; **Choose executable** permits an explicit matching location.
Return to **Clients**, confirm separate licensed accounts, then use **Set up**,
**Open**, or **Focus**. Reduced-containment compatibility consent is required when
creating a profile. Window actions include placement and normal close only.
**Check all clients** refreshes observations; raw errors are under **Diagnostics**.
There is no automatic launch retry, credential automation, force-close, or background poll.

Developer test signing and borrowed service registrations remain distinct from a
standalone production installation. The installed app identity and data paths stay
unchanged; the old VMware bundle remains blocked. See the
[canonical acceptance report](../../docs/validation/native-pilot-localtest.md).

## Stack

```text
Tauri 2
Svelte 5
Vite
TypeScript
Rust
```

This follows the same desktop architecture as `M-LazyBuilder-Plugin/apps/launcher`.

## Layer boundary

```text
ACTIVE: UnifiedManagerApp.svelte + OwnedManagerPanel.svelte (presentation)
        app/presentation/unifiedManager.ts (pure status projection)
            ↓
        app/bridge/nativeFeasibilityApi.ts (typed, fail-closed IPC)
            ↓
        src-tauri/src/commands/native_clients.rs (thin Tauri adapters)
            ↓
        src-tauri/src/engine/native_clients.rs (Windows/Sandboxie owner)
            ↓
        read-only collector / official vendor launchers

INACTIVE: NativeFeasibilityApp.svelte, App.svelte, virtualClientsApi.ts, VMware Rust routes
          are retained for compatibility, but not registered.
```

Ownership:

```text
src/                         presentation + ephemeral UI state
src/app/bridge/              typed Tauri boundary
src-tauri/src/commands/      thin adapters
src-tauri/src/engine/        desktop-native behavior only
engine/runtime-core/
                             VM lifecycle/domain authority
```

There is no localhost HTTP bridge, injected browser-global API, or second VM state authority.

**Legacy window arrangement (unmounted):** Window arrangement was desktop-native presentation behavior. VM lifecycle eligibility stays in the backend. Its Windows discovery enumerates **visible, ownerless top-level HWNDs**, checks owning PIDs and constrained Minecraft Education/VMware window titles, and refuses duplicate or ambiguous client/window mappings. Placement rechecks HWND ownership/title and rejects a failed Win32 `MoveWindow` result instead of reporting success. VMware tabs are not assumed to be independent windows; grid/focus/columns geometry and the public layout response remain unchanged.

## Packaging

**Native-first pilot guard:** npm run build:app is intentionally blocked
with PACKAGE_BLOCKED_NATIVE_FEASIBILITY. The former NSIS contract still
embeds VMware scripts and Guest Agent under the old application identity.
The guard prevents staging old guest binaries as a new native product.
No new, smaller installer has been built or verified. Desktop source
checks and experimental dev mode are separate from packaging.
A future package migration must preserve existing account-bearing data.

The legacy Tauri NSIS package includes the support CLI as the only host sidecar. Guest Agent is a guest payload and is packaged beside the guest preparation scripts under `guest/windows/`. `scripts/prepare-package-binaries.mjs` builds both binaries, stages the CLI as a target-specific sidecar, and stages Guest Agent as a package resource. Normal dev/check/test does not depend on package binary staging.

## Commands

```bash
npm ci
npm run dev:app
npm run typecheck
npm run verify:source
npm run build:app  # intentionally blocked during native feasibility
```


## Legacy first-run orchestration (unmounted)

The previous VMware manager used `DoctorReport.nextSetupAction` as the sole setup decision owner. Its old desktop mapped that machine action to one of four presentation owners:

```text
APP      → safe operation can run directly
USER     → Windows/VMware infrastructure step requires a person
CLIENTS  → continue in the client manager
BLOCKED  → stop and route to support
```

The desktop never infers the next setup state from health issues. User-owned steps always finish by refreshing backend truth; there is no UI-completed setup flag.

Base creation and Windows Sysprep remain guided user steps because the backend has no safe primitive that owns those actions yet. The UI must not pretend they are automated.

## Complete source checkpoint

From repository root, use `npm run verify:source` as the optional complete developer checkpoint. The root route includes the app's Svelte/Vitest/build/Tauri checks and the canonical backend crate's check, tests and rustfmt. App-local `verify:source` covers the desktop but does not replace the backend's own checks.

For an operator's Windows checkout, the supported one-root route is
`DEV.cmd bootstrap` → `DEV.cmd doctor` → `DEV.cmd preflight` →
`DEV.cmd dev` from the repository root. The launcher provisions pinned
Node/npm, Rust/rustfmt and PowerShell under `workspace/toolchains/`
and directs downloads, caches and temporary files to `workspace/`;
Git and Microsoft C++/Windows prerequisites remain external. Direct
`npm --prefix apps/desktop ci` (using the locked
`apps/desktop/package-lock.json`) remains a low-level source/CI command,
not the promised portable Windows entrypoint. The repository root
intentionally has no second Node dependency graph. See
[Development Operations](../../docs/system/development-operations.md).

This command does not build/install the distributable or start VMware/Minecraft.
Package and real-machine acceptance remain separate gates.
