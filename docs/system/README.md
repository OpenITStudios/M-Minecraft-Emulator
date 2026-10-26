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
| Native engine current v1.18.6 build/verification | [Isolation Source Tooling](../../Development/tooling/isolation-source/README.md) — no v1.18.5 fallback |
| Layering, source authority and architecture | [Architecture](architecture.md) |
| Future app-owned isolation backend, compatibility and cutover gates | [Owned Isolation Design](owned-isolation-design.md) — design only, not active runtime |
| Paused VM pathways and safe future deletion | [Legacy Retirement](legacy-retirement.md) |
| Locate Rust/Tauri/Guest Agent owner from an issue | [Implementation Map](implementation-map.md) |
| VM states, provenance, parity, readiness and resource policies | [Runtime Core](runtime-core.md) |
| Naming, source path and installed compatibility contracts | [Canonical Naming](canonical-naming.md) |
| Concrete native efficiency algorithms, Windows APIs, Sodium/FerriteCore feasibility and Codex review | [Native Efficiency Implementation Playbook](native-efficiency-implementation-playbook.md) — design/reference, not a second active task list |
| Smallest correct change and evidence discipline | [Development Discipline](development-discipline.md) |
| Source verification, optional local commands, CI and release boundaries | [Development Operations](development-operations.md) |
| Installer downloads, build outputs, local-test evidence and Git hygiene | [Artifact and Local-Test Storage](artifact-storage.md) |

Source files remain the current implementation authority; documents organize understanding. Root `AGENTS.md` selects task mode and `GITHUB_RULES.md` controls repository writes. Work in progress is recorded in `../../planning/development.md`, not here.
