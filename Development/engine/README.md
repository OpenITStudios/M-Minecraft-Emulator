# M-Minecraft-Emulator engine source

The active product is **Lazy Tester**: five managed Minecraft Education profiles,
owned by the Windows-native session controller. Minecraft remains responsible for
its own authentication, rendering, worlds and networking. This is not a VMware
or Sandboxie-backed product.

## Current source boundaries

- `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs` and
  `independent_bootstrap/session_controller.rs`: reviewed native process/profile
  startup, exact process identities and normal-exit lifecycle.
- `Development/apps/desktop/src-tauri/src/engine/workspace_surface.rs`:
  Windows presentation, restoration and DWM preview ownership.
- `Development/tooling/acceptance/windows/bootstrap-ordering/`: **active**
  compatibility/enrollment DLL source and FPS/presentation support. Its historical
  location under acceptance tooling does not make it disposable.
- `Development/engine/distribution/lazy-tester-package.json`: native product
  package contract and preservation policy.
- `Development/engine/runtime-core/`: still compiled by desktop, but now
  contains only portable independent-engine contracts, a bounded subprocess runner,
  and a synthetic diagnostic helper. VMware/Guest Agent/provider runtime modules
  and VM CLI source have been retired from this crate.

## Retired source

Both full Sandboxie v1.18.6 checkout copies and the historical Sandboxie
submodule/gitlink are removed from the active `Local` tree. Their exact source
is recoverable from Git history / the pre-Lazy-Tester archive branch, not copied
into another live directory. No Sandboxie fork or driver is a product launch
provider. Installed systems, guest disks, worlds, accounts and registry profiles
were not touched by this source-only change.

Windows VMware Guest Agent and historical package/acceptance scripts outside
this crate have **not yet been fully removed**. Trace their remaining desktop/runtime/tooling
consumers before deleting them. This work cannot by itself establish gameplay,
packaging or cross-platform acceptance.

See [retirement register](../../../docs/system/legacy-retirement.md),
[implementation map](../../../docs/system/implementation-map.md) and
[current handoff](../../../planning/development.md).
