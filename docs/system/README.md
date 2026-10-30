---
id: document.system.router
class: DOCUMENT
domain: system
role: ROUTER
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester system documentation

Use the smallest current owner relevant to the task:

| Concern | Document |
| --- | --- |
| Active layered Windows native architecture | [Architecture](architecture.md) |
| Exact Rust/Tauri/UI/tooling source owner | [Implementation map](implementation-map.md) |
| Current profile and product persistence | [Data layout](data-layout.md) |
| Source naming and existing compatibility constraints | [Canonical naming](canonical-naming.md) |
| Optional scoped source checks, `DEV.cmd` and disabled CI | [Development operations](development-operations.md) |
| Existing local developer artifacts and safe retention | [Artifact storage](artifact-storage.md) |
| Minimum complete change and evidence rules | [Development discipline](development-discipline.md) |
| Retired VMware/Sandboxie source and historical provenance | [Legacy retirement](legacy-retirement.md) |
| Historic renderer/RAM experiments, only when asked | [Research](native-memory-efficiency-research.md) |
| Current status and one next action | [Development handoff](../../planning/development.md) |

Do not load old owned-isolation, VMware, Sandboxie, Guest Agent or
resource-tuning designs by default. The active product has five native
Minecraft Education profiles, not a four-slot VM or Sandboxie manager.
The current source owner takes precedence over a historical document.
