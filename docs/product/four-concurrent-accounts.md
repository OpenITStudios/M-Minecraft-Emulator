---
id: document.product.four-concurrent-accounts
class: DOCUMENT
domain: product
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Four concurrent Minecraft Education Windows desktop clients

**Current approved target (23 October 2026):** four concurrently playable, unchanged official Minecraft Education Windows Desktop clients with four distinct licensed accounts, private persistent worlds/registry/authentication, normal graphics/input and multiplayer on one PC. Only active near-term P0: MC01 plus one real isolated MC02. All Sandboxie, owned-fork and VMware routes described below are **superseded pilot history**, not current Launch providers. See [one current goal and next action](../../planning/development.md).

## Education login compatibility parity — private profile plan (15 October 2026)

The private Client 02–04 profile planner now includes the exact box
compatibility settings that made the official Sandboxie Plus 1.18.5
two-account local test usable: `NoSecurityIsolation=y`,
`DropAdminRights=y`, `Template=OpenCOM`, `AutoDelete=n`,
`AutoRemove=n`, `OpenCredentials=n`, and
`OpenProtectedStorage=n`; `Enabled=y` is applied last. Previously
the private planner omitted the first four compatibility settings,
risking a repeat of the `0x80070424` Education sign-in failure even
after successful driver compilation.

**Tradeoff:** `NoSecurityIsolation` (App Compartment) and `OpenCOM`
reduce containment; they are compatibility choices, not secure account
identity guarantees. The plan requires explicit operator approval,
never reconfigures an existing box, does not execute commands or copy
credentials, and records read-back verification as false. A working
private driver and service, live box setting read-back, and independent
Minecraft sign-in remain unproven.

## First owned-fork feature — source only (14 October 2026)

The MIVUBI private fork now includes a functional native C++ source
change to upstream `Sandboxie/apps/start/start.cpp`: the canonical
[owned Start policy](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/b1daa1ecf6679234f9433020aea7f11d93c2e613/Development/engine/isolation-native/owned-runtime/mivubi_owned_start_policy.h)
is inlined into the fork and provides an opt-in `/mivubi_client:02`,
`:03` or `:04` launch path for `MivubiEdu02/03/04`, limited to one
quoted Minecraft Education Windows executable. Its 20-case C++ test
compiled and passed with GCC and Clang in ChatGPT. The accompanying
[owned profile contract](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/b1daa1ecf6679234f9433020aea7f11d93c2e613/Development/engine/isolation-native/owned-runtime/minecraft-profiles.mjs)
generates three persistent, non-auto-deleting sandbox configuration sections
and exact corresponding launch arguments.

This **does not** mean the private Sandboxie engine has compiled, the boxes
were created, or Minecraft supports three isolated signed-in sessions.
Source behavior and in-memory command planning have test coverage, but the
active product still launches Client 02 via the separately installed
official Sandboxie Plus provider. Signing, driver/service exclusivity,
actual Microsoft WAM credential separation, REA native artifact evidence,
and full four-client acceptance remain unexecuted.

## Historical pilot architecture — superseded (2026-10-12)

Four concurrent, independently playable **Minecraft Education Windows Desktop**
sessions with four distinct legitimately licensed accounts on one Windows PC.
The **historical pilot architecture selected at that time (now retired)** was one native Minecraft
Education Windows Desktop process plus Minecraft Client 02 launched using
Sandboxie Plus box `MinecraftClient02`. Both must use distinct legitimately
licensed accounts, preserved data and normal Windows keyboard/mouse/GPU.
The future four-account outcome is not implemented; no Android, controller
emulation, Windows VM, account copying or custom resident launcher service
is part of the current development. See the canonical
[Lightweight Windows Architecture](../system/architecture.md).

**Implementation and proof status (updated 2026-10-14):** the active
Tauri/Rust/Svelte route supports explicit launch requests and read-only
process/box observations. Operator-confirmed Client 01 + Sandboxie Client 02
playability, independent Education accounts and permitted multiplayer were
observed, followed by a Client 02 idle crash. Stable two-client acceptance
remains FAILED/INCOMPLETE; reboot persistence, four-client gameplay and the
private owned isolation engine are **NOT_EXECUTED**. See the
[first success/crash evidence](../validation/two-client-first-success-2026-10-14.md).
No resource benchmark has established an ADB-sized
footprint; the manager's minimal-overhead design is a goal, not evidence.
Legacy VMware source and saved accounts remain inactive and untouched.
Avoid game EXE modification, injection, authentication bypass or storage of
account secrets.

## Historical proposed fork scale-out — superseded, not active

One host-native Education Client 01 plus up to three separately profiled,
concurrent isolated Windows Desktop Clients 02–04 using a single Sandboxie-derived
runtime service/driver. Each account requires its own legitimate Microsoft
Education entitlement and independent interactive sign-in. No four-account
performance, isolation, persistence or multiplayer acceptance exists yet.
Initial owned core source is intentionally exclusive and must **not** coexist
with the official Sandboxie installation on the same account-bearing host.

## Existing VMware implementation (legacy, not new product direction)

One Windows PC runs **four separate Minecraft Education Windows desktop clients concurrently**, each with its own licensed Microsoft 365 Education account: **one native Windows client + three isolated Windows 10/11 VMware Workstation guests**. The game must install, sign in, render, accept input and remain responsive in each guest. No Android, BlueStacks, multi-mode launcher or account automation is in scope. Four active VM processes are not evidence of four playable game windows.

```text
Windows host ─ Minecraft Education Desktop ─ Account 1 (native)
          └ VMware Workstation
              ├ Windows guest 01 ─ Minecraft Education Desktop ─ Account 2
              ├ Windows guest 02 ─ Minecraft Education Desktop ─ Account 3
              └ Windows guest 03 ─ Minecraft Education Desktop ─ Account 4
```

VMware is infrastructure only. The application should present a simple **Start clients**, **Open/Arrange**, and **Stop** workflow after one-time setup. It must not expose a second Android/native-instance product route.

## Historical VMware acceptance procedure (legacy only)

1. **Boot:** VMware Workstation opens one Windows 10/11 64-bit guest to a responsive interactive desktop, with working VMware Tools, display and input.
2. **Install and sign in:** the actual Windows desktop installer or Microsoft Store Minecraft Education build installs **inside that guest**, matches host version, and a distinct correctly entitled organization account signs in manually to the real game menu. Do not install both package variants or copy a signed-in Base.
3. **Play responsively:** a real world renders in that Windows guest, camera and keyboard/mouse work, there are no blocking graphics errors or crashes, and the observed session meets the actual machine's usability criteria. A process counter, static 3D flag or configured RAM limit never establishes this.
4. **Scale only after one guest works:** use an **account-clean** immutable Windows Base; create isolated Virtual-01/02/03, finish each Windows OOBE independently, sign into three different licensed accounts and retain account-bearing disks. Native + three guests must be playable concurrently on one host, not merely booted.
5. **Daily operation:** normal restart/stop/start/suspend must preserve separate sessions; only explicitly confirmed reprovision may erase an account-bearing guest. Record measured host RAM/CPU, VMware guest GPU behavior, launch times and interactivity. Game state and multiplayer remain separate evidence.

No credentials, account tokens, private worlds or Windows user-profile data go into GitHub. No automated Microsoft sign-in or anti-single-instance bypass is authorized.

## Key compatibility risk

Minecraft Education supports Windows 10/11 64-bit desktop installations, but [Microsoft/Minecraft Education's Dedicated Server documentation](https://edusupport.minecraft.net/hc/en-us/articles/46913335157140-Dedicated-Server-System-Requirements) says VM-hosted Dedicated Server has official support **unlike the Client**. Its [PC requirements](https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements) list OpenGL 4.4 GPU capability, while documented VMware guest OpenGL limits may differ. Thus **VM desktop-client gameplay remains unproven until actually observed**. Do not equate VMware 3D configuration with successful Minecraft rendering.

The official [Windows installation guide](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide) explains the two mutually exclusive installer variants.

## Proof and continuation

Follow [current development handoff](../../planning/development.md) only for active work. The [owned fork design](../system/owned-isolation-design.md) and [VMware G0–G4 campaign](../validation/acceptance.md) are historical reference, not current tasks. Never infer four-client readiness from process/window counts.
Source commits do not establish Windows/driver/gameplay acceptance.
All development stays on `Local`; `main` remains untouched.
