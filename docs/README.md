---
id: document.docs.router
class: DOCUMENT
domain: docs
role: ROUTER
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester documentation router

**Current product:** five independent Minecraft Education Windows
profiles, one native game installation and one Tauri/Svelte desktop
manager. Only `Local` is active for development; `main` is reserved
for an approved release. GitHub Actions CI is disabled.

Read one relevant owner, not the entire documentation tree:

| Question | Start here |
| --- | --- |
| What does the user operate? | [Product](product/README.md) / [current workspace contract](product/unified-workspace.md) |
| Where is the active source owner? | [Implementation map](system/implementation-map.md) |
| What are the runtime safety boundaries? | [Architecture](system/architecture.md) and [data layout](system/data-layout.md) |
| How do source changes and optional checks work? | [Development operations](system/development-operations.md) |
| What is actually proven or still untested? | [Validation router](validation/README.md) |
| What is the one current work status/next action? | [Development handoff](../planning/development.md) |
| How are installer data and developer cache protected? | [Storage](system/artifact-storage.md) / [distribution](../Development/engine/distribution/README.md) |
| Where are the retired VMware/Sandboxie studies? | [Legacy retirement](system/legacy-retirement.md) and previous Git commits |

```text
user task -> domain README -> exact current source owner
          -> matching evidence only if needed -> STOP
```

Other historical, experimental and source-snapshot documents
remain references, not alternative active implementation decisions.
Exact current code and its real consumers outrank stale prose.
Remote GitHub code review cannot certify Windows/native Minecraft
runtime or an installed package.
