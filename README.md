# Lazy Tester

A Windows desktop control center for five independent Minecraft Education profiles.
Users open the profiles they need, select one game to control, and keep the others
running. The runtime uses our native compatibility implementation, one official
Minecraft installation, and persistent private profiles. It does not require
Sandboxie or VMware.

## Active architecture

- `apps/desktop/src/LazyTesterApp.svelte`: the mounted Svelte interface.
- `apps/desktop/src/app/bridge/lazyTester.ts`: validated five-profile desktop contract.
- `apps/desktop/src-tauri/src/engine/native_clients_session.rs`: bounded reconnecting controller transport; `native_clients_session/pipe.rs` checks both Windows peers.
- `apps/desktop/src-tauri/src/engine/independent_bootstrap/session_controller.rs`: manual per-profile session ownership, retained handles, and metadata-only storage inspection.
- `apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`: reviewed game/payload/profile checks and startup enrollment, shared with the accepted five-client pilot.
- `apps/desktop/src-tauri/src/engine/workspace_surface.rs`: recoverable five-window presentation/parking; its `workspace_surface/previews.rs` renders the four native DWM tiles and `recovery_gate.rs` serializes checkpoint handoff.
- `tooling/acceptance/windows/bootstrap-ordering/`: current compatibility DLL source; relocating this runtime out of acceptance tooling remains unfinished.

Minecraft still owns authentication, worlds, rendering, and Education Host World
multiplayer. Lazy Tester does not store passwords or copy tokens. The native
compatibility layer is not a general Windows security sandbox.

## Product and evidence

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
binding of the reviewed five existing profiles is implemented; installed startup,
integrated acceptance and generic portable provisioning remain unfinished.
Task Manager grouping is not claimed.

See [workspace contract](docs/product/unified-workspace.md) for product behavior,
[implementation map](docs/system/implementation-map.md) for ownership, and
[current handoff](planning/development.md) for actual evidence and the next action.

## Development and archive

`Local` is the only active product-development branch. `main` remains release
reserved. GitHub Actions is disabled entirely. Follow [AGENTS.md](AGENTS.md) and
[GITHUB_RULES.md](GITHUB_RULES.md).

The pre-Lazy-Tester source is preserved at
[`codex/archive-before-lazy-tester-20261026`](https://github.com/OpenITStudios/M-Minecraft-Emulator/tree/codex/archive-before-lazy-tester-20261026),
pinned to `d166b5c8b24e8435f2c84675cb00a61c753aeb6a`. This is a historical
snapshot, not another active implementation branch. The
[retirement register](docs/system/legacy-retirement.md) owns remaining source
removal/movement. Legacy disks, worlds, accounts, and installed data are retained.

The GUI continues to use Tauri 2, Rust, Svelte, and TypeScript with locked existing
dependencies. The [native packaging decision](engine/distribution/README.md)
creates a separate Lazy Tester product while retaining old installations/settings
and private game profile paths. Legacy installer generation remains blocked.
