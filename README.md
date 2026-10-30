# Lazy Tester

A Windows desktop control center for five independent Minecraft Education profiles.
Users open the profiles they need, select one game to control, and keep the others
running. The runtime uses our native compatibility implementation, one official
Minecraft installation, and persistent private profiles. It does not require
Sandboxie or VMware.

## Repository and distribution layout

This checkout is the development project, not the folder distributed to players.
There is one product source tree; do not maintain separate developing/release
copies of the code.

```text
Lazy Tester.exe       Open the prepared local app directly, with its application icon
App/                  Ignored runnable app, reviewed DLLs and build receipt
Data/                 Settings and private per-launcher files/registry hives
Development/apps/desktop/         Desktop source, native integration and desktop build scripts
Development/engine/               Runtime source and distribution contracts
Development/tooling/              Developer tools, source checks and acceptance collectors
docs/                 Architecture and user/developer documentation
planning/             Current handoff and historical work
Archive/experiments/          Historical research
workspace/            Ignored local tools, builds, candidates and private material
```

## AI-ready developer control

`DEV.cmd` is the single local developer entrypoint. AI can inspect the
checkout, discover a fixed task catalog, recommend the smallest appropriate
check, and run an explicitly selected reviewed task without memorizing Cargo,
Tauri or PowerShell commands. **No UI, daemon or additional Rust build** is
introduced.

```text
DEV.cmd inspect --json
DEV.cmd catalog --json
DEV.cmd plan --json
DEV.cmd plan --base-sha <40-char-ancestor-SHA> --json
DEV.cmd run frontend.validate --expect-sha <exact-40-char-Local-SHA> --json
DEV.cmd run auto --expect-sha <exact-40-char-Local-SHA> --json
DEV.cmd run auto --expect-sha <exact-40-char-Local-SHA> --base-sha <40-char-ancestor-SHA> --json
```

Execution uses pinned portable Node and existing scripts. The `run` route
requires an exact `Local` revision; setup, lockfile generation, full preflight,
and candidate packaging additionally require explicit operator approval.
Unknown/mixed source changes escalate to the full check. The optional
`--base-sha` covers multi-commit development ranges without adding a runner;
default `auto` on a clean checkout checks only the latest commit. Final
candidate acceptance still requires the existing full source/Windows/game
checks. The old developer commands remain available, and CI stays disabled. Read
[AI Developer Controller](Development/tooling/developer/README.md) for the
task, output and safety contract. GitHub-only development cannot execute this
Windows-local controller.

`DEV.cmd dev` opens the development app. `DEV.cmd package` (or
`npm --prefix Development run package:windows`) builds the
native installer from a clean, committed `Local` revision, refreshes the local
`App/` payload, and exports a new folder:

```text
workspace/packages/lazy-tester/<version>-<commit>-<UTC>/
  Lazy-Tester-<version>-windows-x86_64-setup.exe
  SHA256SUMS.txt
  package-manifest.json
```

Use `npm --prefix Development run package:windows -- --export-only` to export a
candidate while preserving the current App. After a successfully completed
build, the packager retains the latest **two verified installer exports**
and can automatically retire up to eight older verified three-file exports
per build. Unknown or modified receipts block pruning. This does **not**
remove Rust incremental data, historical compiler archives, previous App
backups, accounts or worlds. Developer configuration is inside
[Development](Development/README.md), and retained research is in
[Archive](Archive/README.md). The old root apps/engine/tooling directories are gone.
Portable toolchains stay in workspace/toolchains; new Cargo output defaults to
workspace/builds/desktop/target. Old source-tree Cargo caches were moved to
workspace/archive/compiler-cache-20261026 without deleting their contents.

The root executable starts only App/lazy-tester.exe and writes no separate
configuration. Desktop/Start Menu shortcuts use that same entry point. RUN.cmd
is retained only as a compatibility wrapper around the root executable.

Only these three files are exported. Do not distribute this repository, a raw
compiler output directory, `workspace`, or developer profile folders. The manifest
identifies an unaccepted development candidate, not an approved public release.
Signing, installed/gameplay acceptance and publication remain separate. See the
[storage policy](docs/system/artifact-storage.md) for the source/build/runtime split.

For this checkout, start the prepared candidate with `RUN.cmd`. Packaging does
not start the app or any game. A refresh preserves the complete `Data/` tree
including `Data/Settings/data-location.json`, and copies any historical
`App/data-location.json` pointer for backward compatibility. The previous app
is archived under `workspace/private/local-app-backups/`. A fresh installation
does not automatically migrate old pilot accounts or worlds. The preserved
original stores require the separate reviewed migration; an empty folder is
not a migration.

## Active architecture

- `Development/apps/desktop/src/LazyTesterApp.svelte`: the mounted Svelte interface.
- `Development/apps/desktop/src/app/bridge/lazyTester.ts`: validated five-profile desktop contract.
- `Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs`: bounded reconnecting controller transport; `native_clients_session/pipe.rs` checks both Windows peers.
- `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs`: manual per-profile session ownership, retained handles, and metadata-only storage inspection.
- `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`: reviewed game/payload/profile checks and startup enrollment, shared with the accepted five-client pilot.
- `Development/apps/desktop/src-tauri/src/engine/workspace_surface.rs`: recoverable five-window presentation/parking; its `workspace_surface/previews.rs` renders the four native DWM tiles and `recovery_gate.rs` serializes checkpoint handoff.
- `Development/tooling/acceptance/windows/bootstrap-ordering/`: current compatibility DLL source; relocating this runtime out of acceptance tooling remains unfinished.

Minecraft still owns authentication, worlds, rendering, and Education Host World
multiplayer. Lazy Tester does not store passwords or copy tokens. The native
compatibility layer is not a general Windows security sandbox.

## Resource-efficiency previews (source-only; closed)

The temporary RAM/resource-efficiency development stage was closed on
29 October 2026. [Current decision and future evidence gates](planning/development.md)
are owned by one handoff; no optimization or measurement task is currently active.

| Retained independent preview | Source owner | Boundaries |
| --- | --- | --- |
| **Block Texture Lite v0.1** | [`lazytester-textures/`](Development/tooling/lazytester-textures/README.md) | Static pinned vanilla block albedo 16×16 → 8×8; entities, UI, animations, geometry and PBR unchanged. |
| **Particle Tuning v0.3** | [`lazytester-particles/`](Development/tooling/lazytester-particles/README.md) | Twenty selected ambient definitions; optional 75/50/25/0 relative distance budget. Manual emitter caps do not guarantee spawn-rate reductions. |

Both are **offline builders**, not launched game features, installers or a
background texture/FPS switch. No `.mcpack` is bundled or applied to any
Minecraft account/world. Minecraft Education 26.32 pack compatibility,
independent per-client activation, and real RAM/VRAM/FPS improvement remain
**NOT_VERIFIED**. See [tooling navigation and source-only checks](Development/tooling/README.md)
for a future explicitly requested review; historical native heap candidates
stay OFF.

## Product and evidence

Settings includes editable Active/Background FPS limits, Save and Restore
defaults (144/10 on first run). Saved values override these defaults on each
profile's next launch: for example, 120/10 means a **120 FPS foreground ceiling**
and **10 FPS background ceiling**, and foreground focus restores 120, not
a permanent 10 FPS limit. A ceiling is not a guaranteed achieved frame rate.
New profiles with no saved graphics preference use
**Preserve Visuals**, leaving Minecraft's own texture quality, lighting,
MSAA and render distance unchanged; existing explicitly saved Low/High presets
keep their historical 12-chunk behavior. A change to these preferences requires
the original all-sessions-stopped gate. Data Storage exposes an explicit,
backed-up Texture Streaming 0/1 switch only when the key exists on a stopped
profile, plus bounded read-only counts of on-disk pack folders and saved
world_resource_packs.json references. The latter are declarations, **not**
proof of current world activation, actual loaded textures or RAM savings. These controls do not launch
or stop Minecraft. See [data layout](docs/system/data-layout.md) for persistence
and validation. Native pacer fixtures are not fresh Minecraft gameplay acceptance.

The prior five-client pilot was operator-confirmed with five licensed accounts in
the same Education Host World, persistent sign-in/world data, and the 144 active /
10 background FPS policy. That acceptance does not automatically validate the new
Lazy Tester UI/controller integration.

Manual profile launch, workspace selection, next/previous selection, and on-demand
profile-size inspection are integrated in source. Same-binary controller
reconnection after UI restart is implemented and covered by native Windows pipe
fixtures, including two successive client processes. Four native DWM preview tiles
are now wired into the workspace, with background adoption and retained recovery
checkpoints. Native DWM registration/update/release passed on hidden test windows;
live Minecraft pixels, offscreen host-world behavior and the complete UI remain
unverified. Per-profile disk usage/free-space reporting and explicit classified
HTTP-cache cleanup are implemented with inactive-game admission and handle-based
filesystem checks, verified only on disposable fixtures. A separate Lazy Tester
installer candidate now includes the native DLLs and branding. Explicit private
binding of reviewed existing profiles is implemented. New product data uses a
five-profile catalog beneath the application home's Data folder; Storage exposes
its current location and verified reattachment. Fresh-profile gameplay, installed
startup, cross-volume migration and integrated acceptance remain unfinished.
Task Manager grouping is not claimed.

See [workspace contract](docs/product/unified-workspace.md) for product behavior,
[implementation map](docs/system/implementation-map.md) for ownership, and
[current handoff](planning/development.md) for actual evidence and the next action.
The [application/data layout](docs/system/data-layout.md) defines visible App/Data
folders, automatic existing-profile adoption, persistent locations and remaining
relocation limits. Old accounts/worlds are preserved in place, never silently moved.

## Development and archive

`Local` is the only active product-development branch. `main` remains release
reserved. GitHub Actions is disabled entirely. Follow [AGENTS.md](AGENTS.md) and
[GITHUB_RULES.md](GITHUB_RULES.md).

The pre-Lazy-Tester source is preserved at
[`codex/archive-before-lazy-tester-20261026`](https://github.com/OpenITStudios/M-Minecraft-Emulator/tree/codex/archive-before-lazy-tester-20261026),
pinned to `d166b5c8b24e8435f2c84675cb00a61c753aeb6a`. This is a historical
snapshot, not another active implementation branch. The
[retirement register](docs/system/legacy-retirement.md) owns remaining source
removal/movement. The 35 retired frontend files and their exclusive tests have
left the active tree and remain recoverable at pre-removal commit 5601c354.
The sole mounted frontend is Lazy Tester, with seven reachable source files.
Legacy disks, worlds, accounts, and installed data are retained.

The GUI continues to use Tauri 2, Rust, Svelte, and TypeScript with locked existing
dependencies. The [native packaging decision](Development/engine/distribution/README.md)
creates a separate Lazy Tester product while retaining old installations/settings
and private game profile paths. Legacy installer generation remains blocked.

`npm --prefix Development run dev` (also `DEV.cmd dev`) now uses the same Lazy Tester identity,
native feature and reviewed DLL staging as `npm --prefix Development run package:windows`. Set
`LAZY_TESTER_NATIVE_PAYLOAD_DIR` to the directory containing the two reviewed
bootstrap DLLs before either command; missing or different DLLs are refused.
Staging copies only those DLLs, never adjacent profile bindings. First-run profile
selection uses the product's private binding; opening the app does not launch games.
The old developer-only adjacent-binding route remains explicit through
`npm --prefix Development/apps/desktop run dev:preview`; it is not the default product route.
