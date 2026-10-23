---
id: document.docs.router
class: DOCUMENT
domain: docs
role: ROUTER
authority: CANONICAL
lifecycle: ACTIVE
---

# Emulator Documentation

This is the **single documentation entrypoint**. Load only the domain needed for the task; GitHub source and machine-readable contracts remain implementation authority.

| Task | Domain route | Canonical source where applicable |
| --- | --- | --- |
| Product goal: four logged-in Minecraft Education clients on one PC | [Four-account definition](./product/four-concurrent-accounts.md) | Implemented two-client pilot; [historical gameplay/crash report](./validation/two-client-first-success-2026-10-14.md) and [current handoff](../planning/development.md) have separate proof boundaries |
| Paused VMware, App Sandbox and Guest Agent routes | [Legacy Retirement](./system/legacy-retirement.md) | Mutations blocked and account-bearing data preserved |
| Paused VMware Base and guest setup (historical reference only) | [Base image](./product/base-image.md) | `engine/runtime-core/`, `engine/guest/windows/` (inactive) |
| Retired Sandboxie v1.18.6 source/build history | [Historical isolation tooling](../tooling/isolation-source/README.md) | `engine/isolation-owned/` and upstream files are historical, NOT an independent game-launch provider |
| Architecture, ownership, naming, developer commands | [System](./system/README.md) | `apps/desktop/`, `engine/runtime-core/`, `tooling/` |
| Source proof, Windows/VMware/ Minecraft acceptance | [Validation](./validation/README.md) | `tooling/acceptance/windows/` and live evidence |
| Installer packaging, update and trust policy | [Distribution](../engine/distribution/README.md) | `engine/distribution/`, Tauri packaging |
| Dependency maintenance and modernization evidence | [Dependency tooling](../tooling/dependencies/README.md) / [Report](./validation/dependency-modernization-2026-10-20.md) | npm/Cargo manifests and package-manager-generated locks |
| Current tasks, blockers and continuation | [Planning](../planning/README.md) | Current branch HEAD and `planning/development.md` |
| Working artifacts and local-test storage | [Storage Policy](./system/artifact-storage.md) → [Workspace](../workspace/README.md) | Private evidence, caches and generated packages are not committed |

## Canonical navigation

```text
question
→ docs/README.md
→ one domain README
→ one topic owner / section
→ source and evidence only when needed
→ STOP
```

All documents under `docs/` have stable `document.<domain>.<topic>` frontmatter IDs. Paths locate documents; IDs express durable roles. Do not create parallel docs under `engine/`, `apps/` or a legacy `virtual-clients/docs/` hierarchy. README files next to source only explain local ownership and commands; they are not second normative documentation paths.

Stable implementation contract: `CONTEXT.md` + owning source. Current progress belongs only in `planning/`. Historical experiments cannot prove current VM behavior.
