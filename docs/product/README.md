---
id: document.product.router
class: DOCUMENT
domain: product
role: ROUTER
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester — current product and UX

The current product is one Windows desktop workspace for **five**
separately licensed Minecraft Education profiles (MC01–MC05).
Users manually launch the profiles they need, select the one to
play, and see up to four other native game windows as passive
previews. Starting the app does not automatically start Minecraft.

**Current source and contracts:**

- [Unified workspace](unified-workspace.md) — active first section
  defines operator actions, profile identity, native windows, recovery
  and data preservation. Sections explicitly labeled historical
  remain background only.
- [Data layout](../system/data-layout.md) — current `App/`, `Data/`
  and profile/persistence contracts.
- [Desktop README](../../Development/apps/desktop/README.md) —
  currently mounted UI, typed bridge, native backend and limits.
- [Architecture](../system/architecture.md) — product/runtime ownership
  and source-only versus real-game proof boundaries.

**Historical references, not active product specifications:**
[four-client account pilot](four-concurrent-accounts.md),
[old Compact Manager design](compact-manager-implementation.md),
[previous UX draft](compact-manager-ux-spec.md) and
[VM Base image](base-image.md). They must not override the
current five-profile Lazy Tester UI or restore Sandboxie/VMware.

Minecraft itself owns authentication, worlds, rendering and network.
A process or thumbnail is not proof of an authenticated playable
session. Installed Windows and five-client Minecraft acceptance
remain separate from source review.
