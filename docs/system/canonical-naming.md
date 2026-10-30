---
id: document.system.canonical-naming
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester — canonical naming and compatibility

Name responsibilities by their current owner. Do not introduce
duplicate `-new`, `-final`, `-v2`, `engine2`, `state-manager`
or other alternate implementation trees merely to preserve history.
Git retains the older source.

## Current product and repository vocabulary

| Name/path | Meaning |
| --- | --- |
| `M-Minecraft-Emulator` | GitHub repository identity; **not** a separate emulation runtime |
| **Lazy Tester** | Active Windows desktop application |
| **Minecraft 1–5 / MC01–MC05** | Five stable separate profile **storage identities**, not window enumeration order or login proof |
| `Development/apps/desktop/` | One Tauri 2 / Svelte 5 desktop, typed commands and native session owner |
| `Development/engine/runtime-core/` | Platform-neutral contracts and process helpers; **not** a second game launcher, guest controller or mutable lifecycle store |
| `Development/engine/distribution/` | Native Lazy Tester package contract and preservation rules |
| `Development/tooling/` | Source checks, native compatibility source, scoped developer control and optional evidence collection |
| `DEV.cmd` | Single optional Windows-local developer entrypoint, not a second build engine |
| `docs/` | Canonical domain router, active architecture, contracts and historical references |
| `planning/development.md` | One current development-status and next-action handoff |
| `workspace/` | Ignored generated tools, developer caches and packages, never game account source authority |

## Persistent product data

- Current profile folders: `Data/Launchers/Minecraft-1..5`.
- Current catalog: `Data/profiles.json`.
- Current canonical location pointer:
  `Data/Settings/data-location.json`.
- Historical schema 1 and legacy App pointers are kept for
  preservation/explicit reviewed migration only.

These names are actual compatibility contracts. Do **not** rename
installed application identifiers, original Microsoft account data,
worlds, registry hives, reviewed DLLs or paths as a cosmetic cleanup.
Inspect [data layout](data-layout.md) and
[native distribution contract](../../Development/engine/distribution/README.md)
before any proposed migration.

## Historical identity boundary

Former `Minecraft Instance Manager`, `M-Bedrock Virtual Clients`,
`MinecraftClient02` Sandboxie, VMware VM/Guest Agent, native-box
pilot and legacy NSIS/wire names may exist in preserved data or Git
history. Their mention is not authorization to reactivate a provider
or rewrite installed state. Source retirement does not remove old
applications, accounts, snapshots or user files.

## Version and responsibility

`Development/VERSION` must match the application and relevant
Node/Rust/Tauri manifests. Preserve actual public schema and wire
versioning when needed for compatibility; do not invent version-suffixed
internal source owners. Verify the first wrong owner, consumers,
import paths, Tauri/Cargo contract and distribution identifiers
before a rename. See the [implementation map](implementation-map.md).
