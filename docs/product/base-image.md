---
id: document.product.base-image
class: DOCUMENT
domain: product
role: GUIDE
authority: CANONICAL
lifecycle: ACTIVE
---

# Virtual Clients Base Image — historical VMware setup

> **Paused legacy implementation.** The Windows Base/Virtual-01/02/03
> setup below is retained to preserve compatibility and engineering context,
> **not** authorized as the current product path. The active experiment is
> one native plus one Sandboxie Plus Minecraft Education Windows client:
> [selected product contract](four-concurrent-accounts.md).
> Do not change existing guest disks, snapshots, worlds or Microsoft sessions.

Base is the immutable parent for Virtual-01, Virtual-02, and Virtual-03.

## Approved Windows desktop implementation

This Base is the one approved VMware Workstation Windows desktop implementation for the [four-account objective](four-concurrent-accounts.md). Before creating an account-clean Base or linked clones, prove the guest game can boot, install, sign in and play responsively through [One Guest Feasibility](../validation/client-feasibility.md).

## Prerequisite: prove one Minecraft Education guest

Before investing in Base finalization, linked clones or Microsoft account-bearing snapshots, use the [Single Guest Client Feasibility Gate](../validation/client-feasibility.md) on a **disposable, separate** Windows VMware VM. Minecraft Education's published minimum OpenGL 4.4 vs VMware's documented guest OpenGL 4.3 is a compatibility warning. Source configuration of 3D acceleration is not evidence of an actually rendered Minecraft world.

Do not reuse an interactive, signed-in feasibility scratch guest as the immutable account-clean Base. A successful pilot identifies a candidate environment for later deliberate Base preparation, not a production-ready clone image.

## Required

Base must keep:

- supported Windows installation;
- VMware Tools;
- Virtual Guest Agent with a launch-compatible protocol (current runtime requires v2; status-only v1 is insufficient);
- working virtual GPU / 3D acceleration;
- keyboard, mouse and controller input;
- networking;
- Microsoft organization sign-in dependencies;
- Minecraft Education;
- normal shutdown/restart/suspend behavior.

## Minecraft installation policy

For managed Windows Base images, prefer the official Minecraft Education desktop installer with automatic updater creation disabled.

Goal:

```text
Native version changes
→ Base is deliberately rebuilt/updated
→ register-base
→ Virtuals are reprovisioned
```

Do not let Virtual-01 / Virtual-02 / Virtual-03 independently auto-update Minecraft Education.

The Base provenance record stores the intended Minecraft version and one Base generation identity. Each provisioned Virtual stores that exact Base generation. The Guest Agent later proves the live Virtual version and protocol compatibility.

## Preparation flow

```text
install supported Windows
→ install VMware Tools
→ install Virtual Guest Agent
→ install Minecraft Education
→ disable independent Minecraft auto-update in the managed Base
→ keep Base free of Microsoft/Minecraft user sessions
→ require AzureAdJoined = NO and WorkplaceJoined = NO
→ apply conservative background cleanup
→ configure guest QA graphics/resolution target
→ verify network + input + rendering
→ shut down Base
→ register-base
→ Base state = REGISTERED
→ boot Base only to run C:\ProgramData\M-Bedrock\VirtualClients\finalize-base.ps1 -ConfirmGeneralize
→ Sysprep /generalize /oobe /mode:vm
→ Base state = FINALIZED
→ shutdown
→ never boot finalized Base again
→ provision Virtuals
→ complete first-boot Windows specialization/OOBE per Virtual
→ start all three
→ verify-identities
```

## Virtual profile

Backend-enforced:

```text
memory ceiling  4096 MB
vCPU            2
3D acceleration enabled
```

Guest QA configuration targets, not backend truth:

```text
resolution      1280x720
graphics        low
render distance QA-appropriate
frame target    ~30 FPS
```

## Per-Virtual preparation

## Interactive user launch boundary

The Base may contain the SYSTEM Guest Agent binary/service task, but it must not
contain a user-specific interactive launcher registration.

Interactive Minecraft UI launch belongs to each provisioned Virtual after
Windows OOBE establishes that Virtual's user profile:

```text
Base
→ SYSTEM Guest Agent only
→ Sysprep / clone

Virtual
→ OOBE / interactive Windows user exists
→ register the narrow interactive launcher for that user
→ sign in to Minecraft manually
→ QA_READY
```

Do not bake a Base username, browser profile, Microsoft identity, Minecraft
session, OAuth token, cookie, or per-user launcher registration into the Base.

The SYSTEM Guest Agent may request only the fixed Minecraft Education launch
action. The Interactive Launcher may listen only on localhost for its fixed
PING and MINECRAFT_EDUCATION protocol; it must not expose a host/LAN listener,
arbitrary command execution, executable/path arguments, or credential handling.

Identity verification happens before account sign-in:

```text
start Virtual-01 + Virtual-02 + Virtual-03
→ live Guest Agent parity must pass
→ verify-identities
→ vmIdentity = UNIQUE for all three
→ windowsIdentity = UNIQUE for all three
→ configure/sign in each Virtual with its own licensed identity
→ confirm the signed-in session survives stop/start
→ stop
→ set-ready
```

`QA_READY` belongs to the Virtual, not Base. It is created only after identity proof and account setup. Daily start/stop/restart/suspend/resume must preserve the guest disk and account state.

## Conservative optimization

Prefer reversible reductions:

- remove unnecessary third-party startup apps;
- avoid browsers/cloud sync inside Base;
- disable unnecessary third-party auto-updaters;
- reduce visual effects where useful;
- keep Minecraft graphics low.

Do not broadly disable Windows services merely to lower idle RAM.

## Disk hygiene

Do not auto-compact linked clones with active snapshots.

If a Virtual delta becomes unhealthy, prefer `reset` to the existing `QA_READY` checkpoint first.

If a full rebuild is required:

```text
stop Virtual
→ reprovision Virtual --destroy-account-state
```

Base remains the immutable parent. Virtuals are reprovisionable, but reprovision intentionally destroys that Virtual's saved guest/account state and QA_READY snapshot.
