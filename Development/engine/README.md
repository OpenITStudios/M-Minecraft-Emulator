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

The shared subprocess runner owns only the original direct helper (e.g.
registered Education query or approved one-shot screenshot worker). Its
24 I/O-worker limit is unchanged; at most 12 direct helper leases are
admitted, including retained children. On failure it requests direct-child
termination and checks exit for up to 2 s using nonblocking `try_wait`.
Unconfirmed children retain their original handle and lease; later helper
admission checks these handles once and refuses new launches while any
remain unconfirmed. Already admitted callers are unaffected. No idle
reaper, descendant termination, automatic replay, or PID reopen. The
budget is not a hard wall-clock bound on every OS call; inherited pipes
can still outlive their child and consume the existing I/O-worker slots.
This is source policy, not executed containment or performance proof.

## Retired source

Both full Sandboxie v1.18.6 checkout copies and the historical Sandboxie
submodule/gitlink are removed from the active `Local` tree. Their exact source
is recoverable from Git history / the pre-Lazy-Tester archive branch, not copied
into another live directory. No Sandboxie fork or driver is a product launch
provider. Installed systems, guest disks, worlds, accounts and registry profiles
were not touched by this source-only change.

The VMware Guest Agent scripts, fork-only build tree and legacy distribution
resources have now been retired from the active source checkout. Unreachable legacy VM/setup/window modules were also deleted from desktop.
The retired Sandboxie Client02 adapter, box observer and owned-fork helper scripts
are now absent from the native desktop source tree. Registered Education image
resolution is retained with the active Windows-native tooling. This work cannot by itself establish gameplay,
packaging or cross-platform acceptance.

See [retirement register](../../../docs/system/legacy-retirement.md),
[implementation map](../../../docs/system/implementation-map.md) and
[current handoff](../../../planning/development.md).
