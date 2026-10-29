---
id: document.system.router
class: DOCUMENT
domain: system
role: ROUTER
authority: CANONICAL
lifecycle: ACTIVE
---

# System Documentation

Choose one document based on the requested concern:

| Concern | Canonical document |
| --- | --- |
| Retired Sandboxie v1.18.6 source/build history | [Historical Isolation Source Tooling](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/4d790401823e066adb9c00790ba274b560cdcde2/Development/tooling/isolation-source/README.md) — not active runtime or current build path |
| Layering, source authority and architecture | [Architecture](architecture.md) |
| Future app-owned isolation backend, compatibility and cutover gates | [Owned Isolation Design](owned-isolation-design.md) — design only, not active runtime |
| Paused VM pathways and safe future deletion | [Legacy Retirement](legacy-retirement.md) |
| Locate Rust/Tauri/Guest Agent owner from an issue | [Implementation Map](implementation-map.md) |
| VM states, provenance, parity, readiness and resource policies | [Runtime Core](runtime-core.md) |
| Naming, source path and installed compatibility contracts | [Canonical Naming](canonical-naming.md) |
| Current RAM/resource-efficiency decision, closed source scope, deferred evidence and reopen gates | [Current Development Handoff](../../planning/development.md) — sole current decision owner |
| Historical RAM measurements, graphics-heap experiments and attribution limits | [Native Memory Efficiency Research](native-memory-efficiency-research.md) — evidence only, not an active task list |
| Optional historical native-efficiency mechanisms and Windows API feasibility | [Native Efficiency Technical Reference](native-efficiency-implementation-playbook.md) — dormant, not an active task list |
| Smallest correct change and evidence discipline | [Development Discipline](development-discipline.md) |
| Source verification, optional local commands, CI and release boundaries | [Development Operations](development-operations.md) |
| Installer downloads, build outputs, local-test evidence and Git hygiene | [Artifact and Local-Test Storage](artifact-storage.md) |

Source files remain the current implementation authority; documents organize understanding. Root `AGENTS.md` selects task mode and `GITHUB_RULES.md` controls repository writes. Work in progress is recorded in `../../planning/development.md`, not here.
