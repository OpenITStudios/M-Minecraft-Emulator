---
id: document.validation.platform-prerequisites
class: DOCUMENT
domain: validation
role: EXTERNAL_REFERENCE
authority: REFERENCE
lifecycle: ACTIVE
---

# Minecraft Education and VMware Platform Prerequisites

Last external review: 2026-10-10. These are **platform constraints**, not evidence that a VMware guest meets them.

## Minecraft Education operating system and hardware

Minecraft Education's official system requirements document lists Windows 10/11 **64-bit** as supported desktop Windows systems. As of its February 17, 2026 revision, Windows 32-bit is no longer supported. Its RAM figures for Windows/Mac/Chromebook devices are **2 GB minimum** and **8 GB recommended**; minimum RAM can still result in slow frames, lag and multiplayer issues.

Source: https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements

The managed Base installer is the official **Windows Desktop .exe** with `/qn INSTALL_UPDATER="NONE"`; official documentation marks its machine version registry detection as a **32-bit app on 64-bit clients**, so both the Base preparation script and Rust backend must explicitly use the 32-bit machine registry view. Store/AppX and Desktop installations remain mutually exclusive.

The existing Virtual Clients policy caps each virtual guest at 4096 MB and configures two vCPUs. This is a purposeful multi-guest trade-off **below the published device-level RAM recommendation**, not proof of acceptable frame times, renderer compatibility, server hosting, or four-client concurrency. Measure real guest/host RAM, graphics and responsiveness before tuning policy.

The official requirements also caution that using emulators to run Minecraft Education on Linux is unsupported. This project does not claim official Minecraft Education support for running the game inside a VMware guest; Windows VMware support and Minecraft usability require actual compatibility evidence.

## Critical graphics compatibility gap — first acceptance gate

Minecraft Education lists integrated Intel HD Graphics 4400 / AMD Radeon R5 with **OpenGL 4.4** in its published PC GPU minimum (or supported discrete equivalents). VMware's documented guest 3D capability is **OpenGL 4.3** and **DirectX 11**. These specifications are not equivalent, and neither proves a particular Minecraft Education build fails or succeeds. Rendering may use another graphics path. Do **not** mark the product compatible merely because `mks.enable3d = TRUE`.

Authoritative references:
- Minecraft Education system requirements: https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements
- VMware Workstation/Fusion graphics capabilities: https://www.vmware.com/products/desktop-hypervisor/workstation-and-fusion
- Dedicated Server is VM-supported **unlike the client**: https://edusupport.minecraft.net/hc/en-us/articles/46913335157140-Dedicated-Server-System-Requirements

**Proof-first action:** Use [Client Feasibility Gate](client-feasibility.md) on **one disposable interactive Windows VMware guest** before creating the account-bearing immutable Base or three clones. Directly querying a guest OpenGL context is diagnostic; live Minecraft menu, input and a rendered world are the critical observations. Lack of guest-graphics evidence stays `UNKNOWN`, not PASS.

## Account identity, tenant and network

Minecraft Education multiplayer requires the same Minecraft Education version for all participating clients, and regular peer-to-peer players are expected to use accounts in the same Microsoft 365 tenant. A valid individual Minecraft Education license is assigned to the relevant account. Merely having network access or a shared join code does not prove a usable session.

Sources:
- https://edusupport.minecraft.net/hc/en-us/articles/360047555531-How-To-Set-Up-A-Multiplayer-Game
- https://edusupport.minecraft.net/hc/en-us/articles/360047119092-FAQ-Availability-Pricing-and-Licensing

Cross-tenant server invitations are a separate administered server feature; do not misrepresent them as a bypass for peer-to-peer tenant restrictions. Source: https://edusupport.minecraft.net/hc/en-us/articles/51711271699092-Enabling-Cross-Tenant-Play

**Product behavior:** keep Microsoft sign-in manual in each isolated guest, and treat account/license readiness separately from VM/process state.

## VMware provider

VMware announced Workstation and Fusion free for commercial, educational and personal use in November 2024. That licensing choice does **not** supply Windows guest OS licenses, Minecraft Education accounts, VM host GPU support or 3D renderer proof.

Source: https://blogs.vmware.com/cloud-foundation/2024/11/11/vmware-fusion-and-workstation-are-now-free-for-all-users/

The first acceptance target should be a real Windows host using VMware Workstation, with Windows guest version/architecture verified. The macOS Fusion backend is present in source, but its package and guest compatibility are separate unknowns.

## Evidence rule

Do not infer four interactive Minecraft Education sessions from configured 3D acceleration, four running VM processes, memory availability, or official minimum specifications. Use the canonical exact-machine acceptance and failure matrix under [Validation](README.md), especially the [Acceptance](acceptance.md) and [Failure Matrix](four-client-failure-matrix.md).
