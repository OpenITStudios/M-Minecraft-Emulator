---
id: document.validation.router
class: DOCUMENT
domain: validation
role: ROUTER
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester — validation and proof routes

**Current target:** up to five separate native Minecraft Education
Windows profiles (MC01–MC05) with distinct licensed accounts.
No VMware/Sandboxie provider is selected in the active product.
GitHub Actions CI is disabled; all optional verification commands
remain on-demand, not hosted workflows.

## Select evidence matching the claim

| Claim | Current existing owner or evidence source | What it does not prove |
| --- | --- | --- |
| Exact current branch/source, ownership and doc links | GitHub `Local`, `Development/tooling/repository/` | Compiler pass or gameplay |
| Changed-source checks and regression routing | `DEV.cmd`, `Development/tooling/developer/`, existing frontend/Rust tests | Full installed native acceptance |
| Windows compiler / native DLL / Tauri integration | Existing `Development/tooling/windows-toolchain/` and `acceptance/windows/` fixtures, when actually executed | Microsoft sign-in, playable five-client world |
| Native installer and data preservation | `Development/engine/distribution/`, Tauri package checks, real Windows package verification | Signed release or user-profile migration without observed proof |
| Account persistence, DWM pixels, normal close/reconnect, FPS and shared-world gameplay | Actual installed Windows/Minecraft sessions and exact-revision evidence | Nothing inferred from static files or synthetic tests |

**Current status:** the last source handoff in
[planning/development.md](../../planning/development.md) distinguishes
implemented source from tests **NOT_EXECUTED** at the current revision.
Do not infer local Windows access or ask for a user-owned PC to complete
independent GitHub source analysis.

## Retained historical experiments (not active tests)

- [Earlier two-client Sandboxie feasibility](sandboxie-two-client-feasibility.md)
  and [first two-client gameplay/crash observation](two-client-first-success-2026-10-14.md)
- [Retired native/Sandboxie pilot notes](native-pilot-localtest.md)
  and [four-client failure matrix](four-client-failure-matrix.md)
- [Legacy VMware campaign G0–G4](acceptance.md),
  [old static readiness](static-readiness.md), and
  [old client/VM feasibility](client-feasibility.md)
- [Archived native-isolation research](native-multi-instance-feasibility.md)
  and [historical source audit](rea-source-audit-2026-10-13.md)

Their dated reports retain their original evidence ceilings and
source SHAs. They are not current execution plans, alternate runtime
providers or approval to reenable CI.
