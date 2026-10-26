---
id: document.system.canonical-naming
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Canonical Naming

Name source for the **responsibility**, not the parent monorepo, developer nickname or temporary implementation phase.

## Repository and folder vocabulary

| Name | Meaning |
| --- | --- |
| `M-Minecraft-Emulator` | Dedicated repository/project identity |
| `Development/apps/desktop/` | One Tauri/Svelte user-facing desktop application |
| `Development/engine/runtime-core/` | Canonical Rust runtime: lifecycle, provider, guest transport and resource policy |
| `Development/engine/guest/windows/` | Guest Windows setup and Agent lifecycle scripts |
| `Development/engine/distribution/` | Machine-readable install/release/update policy and packaging helpers |
| `Development/tooling/acceptance/windows/` | Test/evidence collection utilities, not live VM authority |
| `docs/{product,system,validation}/` | Human documentation split by concern |
| `planning/` | Current future work, blockers and handoff, never stable system facts |
| `workspace/` | Working material; sensitive VM account evidence is untracked |
| `Archive/experiments/` | Temporary non-authoritative research |
| `.agents/` | Bounded AI/software work procedures, not a second VM manager |

Avoid top-level `virtual-clients/` now that Virtual Clients is the sole application domain; no nested monorepo-style product root is needed.

## Active developer-pilot names

| Canonical name | Meaning |
| --- | --- |
| **Minecraft Instance Manager** | Current Windows developer UI/window title |
| **Client 01 — Native** | Official host-Windows Minecraft Education Desktop |
| **Client 02 — Isolated** | Official Minecraft Education Desktop requested inside Sandboxie |
| `MinecraftClient02` | Exactly one persistent Sandboxie Plus box used in the experiment |
| `%LOCALAPPDATA%\\MinecraftInstanceManager\\NativePilot` | Experimental manager lock/state directory, not the legacy VM data root |
| `%TEMP%\\MinecraftInstanceManager\\SandboxieFeasibility` | Optional explicit diagnostic report location; UI `-SummaryOnly` does not write a report |

The old installed product name and identifier below are **compatibility boundaries**, not names to propagate into the new UI. Never rename existing Sandboxie boxes, VM snapshots, accounts or installed package identities as a cosmetic cleanup.

## Stable compatibility names

`M-Bedrock Virtual Clients` is the **existing installed product identity**, not a source folder identity. Its Tauri application identifier, NSIS asset names, `M-Bedrock/VirtualClients` data path, Guest Agent/CLI binary names, stored snapshot names, wire protocol versions and signed-release tag prefix must remain intact until a separately authorized migration with compatibility proof.

A source rename **must not** break existing guest disks, account sessions, user installations or trusted-update origin checks. Changing GitHub release repository to this repo is separate from renaming installed artifacts.

## Version suffix policy

- Internal implementation: one canonical source module/folder; Git history holds revisions. Do not create `-new`, `-final`, `-v2` duplicate classes or managers.
- Public schema, wire protocol, Minecraft version, persisted payload and historical experiment: keep explicit distinct version identifiers when they actually describe a compatibility boundary.
- Product version: root `Development/VERSION` matches `Development/package.json`, desktop app, Tauri config, both Rust Cargo manifests. A mismatch is a repository error.

## Avoid ambiguous owners

Do not create parallel `runtime/`, `state-manager/`, `engine2/`, `services/` or `vm-controller/` roots. Use the established Rust owner or an appropriately scoped module. Desktop adapter and backend policy are distinct even when both mention the same operation.

Use [Implementation Map](implementation-map.md) to locate semantic responsibility before renaming a file. Verify imports, installer resource keys, scripts, tests, tooling, documentation and actual user-facing compatibility before moving a consumed path.
