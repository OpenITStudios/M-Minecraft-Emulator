---
id: document.product.router
class: DOCUMENT
domain: product
role: ROUTER
authority: CANONICAL
lifecycle: ACTIVE
---

# Product and User Setup

**Current product target:** four separately licensed, concurrently playable Minecraft Education **Windows Desktop** clients on one PC. The only selected engineering approach is **Client 01 native + Client 02 isolated through Sandboxie Plus** (`MinecraftClient02`) until two real game sessions are demonstrated.

- [Four-account product contract](four-concurrent-accounts.md) — goal, limits and preserved historical implementation.
- **[Compact Manager Implementation Specification — ACTIVE DESIGN](compact-manager-implementation.md)** — single English-only, no-sidebar, instance-first UX/interaction blueprint; **design frozen, not implemented**.
- Historical only: [original UX draft](compact-manager-ux-spec.md) and [screen design draft](compact-manager-screen-design.md) (SUPERSEDED; do not use to implement).
- [Lightweight Windows architecture](../system/architecture.md) — source ownership, one Sandboxie engine and resource-efficiency strategy.
- [Active two-client procedure](../validation/sandboxie-two-client-feasibility.md) — official Desktop executable, sandbox evidence and manual licensed login.

**Legacy-only setup:** [Base Image Setup](base-image.md) and [One Guest Feasibility](../validation/client-feasibility.md) describe the paused VMware approach. Do not treat their guest provisioning instructions as part of the active pilot.

The actual lifecycle authority resides in the relevant Rust/Tauri source, not these documents. Read [Validation](../validation/README.md) for the distinct proof levels.
