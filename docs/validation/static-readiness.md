---
id: document.validation.static-readiness
class: DOCUMENT
domain: validation
role: REFERENCE
authority: WORKING
lifecycle: HISTORICAL
---

# Virtual Clients — Static Readiness Freeze (legacy VMware)

> **Historical source-review snapshot.** The VM lifecycle, Base, Guest
> Agent, recovery, resource policy and distribution assertions below apply
> to the paused VMware implementation. They do not certify the active
> [native + Sandboxie Plus Client 02](sandboxie-two-client-feasibility.md)
> system. See [Architecture](../system/architecture.md) for the currently
> selected implementation and [Legacy Retirement](../system/legacy-retirement.md)
> for the preserved inactive source inventory.

Status: REMOTE_GITHUB source boundary. This document is not target-machine proof. Source review supports **G0**, not a product-test PASS; only the [five campaign gates G0–G4](acceptance.md) track testing progress. Individual unit and regression tests remain diagnostic supporting evidence.

## Static-covered

The current source establishes these contracts without claiming Windows/VMware/Minecraft runtime acceptance:

- one Native Minecraft version authority with Base and Virtual lineage gates;
- bounded lifecycle admission owned by the backend;
- provider-state reconciliation for ambiguous lifecycle mutations where authoritative facts exist;
- fail-closed handling when destructive mutation outcomes cannot be proven;
- rollback isolation for multi-client start, stop and suspend flows;
- per-Virtual Guest Agent tokens rotated from inherited Base state;
- bounded Guest Agent request/response transport with request correlation;
- explicit Guest Agent status and launch protocol compatibility;
- loopback-only interactive launcher handoff;
- connection evidence separated from setup, sign-in and multiplayer readiness;
- resource ceiling separated from observed host working set and pressure;
- runtime schema downgrade protection;
- idempotent Guest Agent install/repair/uninstall lifecycle;
- bounded firewall policy for Guest Agent TCP 47831;
- trusted release identity, installer hash, Authenticode and provenance contracts;
- update staging without self-apply;
- privacy-bounded support evidence and Windows acceptance collector.

## Architecture freeze rule

Do not add another lifecycle manager, networking layer, retry framework, readiness state machine, credential mechanism, resource scheduler, or update executor without target-machine evidence that an existing owner cannot express the required behavior.

Source changes after this checkpoint should be limited to:
1. correctness defects with a concrete first owner;
2. contract/documentation drift;
3. security/privacy defects;
4. deterministic regression coverage for an existing contract; or
5. changes justified by LOCAL_WINDOWS / VMWARE / MINECRAFT acceptance evidence.

## Local proof required

The following remain intentionally unproven by source inspection:

- VMware Workstation can run Virtual-01, Virtual-02 and Virtual-03 concurrently on the target host;
- Native Minecraft Education plus all three Virtual Minecraft clients remain interactive concurrently;
- SYSTEM Guest Agent starts after guest boot and remains reachable;
- the per-user interactive launcher starts after Windows login and replaces stale processes correctly;
- Minecraft Education launches in the intended interactive Windows session;
- Microsoft authentication surfaces remain inside the correct Virtual and deep-link completion returns to that client;
- four distinct licensed accounts can coexist under the intended tenant/policy;
- QA_READY preserves the intended account/session state after reset;
- stop/start, suspend/resume and host reboot preserve each Virtual independently;
- actual CPU, RAM, disk and GPU behavior is acceptable with Native + three Virtuals;
- VMware 3D acceleration is usable in all three guests concurrently;
- NAT supports the required Native↔Virtual and Virtual↔Virtual multiplayer topology;
- disconnect/reconnect and suspend/resume multiplayer behavior is usable;
- Window Layout identifies and arranges the real Minecraft/VMware windows correctly;
- install/repair/uninstall scripts behave as designed on the canonical Windows guest;
- signed installer staging and release artifacts behave correctly on the target Windows machine.

## Acceptance entrypoint

Use `Development/tooling/acceptance/windows/collect-acceptance.ps1` for machine evidence. A collected field is evidence only for what it directly observes. In particular:

- `connectionHealth=MINECRAFT_RUNNING` means a Minecraft process was observed;
- `networkMode` records a provider network configuration only when one was observed;
- multiplayer remains unproven until an actual multiplayer acceptance stage passes; there is no synthetic verification boolean.

Do not promote LOCAL proof from documentation, source tests, package smoke, or hosted CI.


## Pre-local-test separation

Use [G0 Source & Package Integrity](pre-local-test-readiness.md): a single real exact-SHA source execution, Windows NSIS construction, installed payload inspection and disposable install-lifecycle safety observation are required before G1 game acceptance. Static repository inspection can identify defects but cannot mark G0 as PASS.

## Build gate status

The static architecture freeze does not imply a successful package build. Windows acceptance requires an **actually executed, same-SHA** source/package integrity result. GitHub's `workflow_dispatch` definitions on `Local` are **not currently runnable** while `main` remains intentionally README-only; do not suggest enabling them by modifying `main` or adding automatic CI.

Current source remains `STATIC_REVIEWED / BUILD_UNPROVEN` and all five product-test gates remain `NOT_EXECUTED` until real evidence exists. No workflow run means **no executable PASS**, not a build failure.


## Static security freeze

Security architecture is frozen at this checkpoint. Existing boundaries include bounded Guest Agent transport, per-Virtual tokens, fixed launch actions, runtime metadata redirection rejection, update URL/hash/signature/staging containment, privileged Guest Agent task verification, and signed release/package evidence.

Do not add a second authentication mechanism, privileged helper, firewall service, updater daemon, ACL framework or security database without target-machine evidence. Effective Windows ACLs, Scheduled Task ACLs, firewall enforcement, antivirus interaction, Windows session behavior and VMware guestinfo exposure remain LOCAL proof.


## Final static handoff

Static architecture, security, performance-policy structure and anti-slop cleanup are frozen here. Do not continue speculative refactoring after this checkpoint. Reopen source architecture only for a concrete correctness/security defect or target-machine evidence.

The highest-risk unproven boundaries are:

1. three VMware guests running concurrently without host instability;
2. Native + Virtual-01/02/03 Minecraft Education remaining interactive together;
3. VMware 3D acceleration actually functioning in every guest, not merely configured in VMX;
4. SYSTEM Guest Agent plus per-user interactive launcher crossing the Windows session boundary correctly;
5. four distinct licensed Microsoft/Minecraft sessions remaining isolated and persistent;
6. QA_READY preserving the intended per-Virtual account state without contaminating Base or another Virtual;
7. NAT/firewall/router behavior supporting the required multiplayer topology;
8. effective Windows ACL, Scheduled Task, firewall and antivirus behavior matching the static security assumptions;
9. actual RAM, CPU, GPU and disk contention validating the provisional automatic startup thresholds;
10. installer/repair/uninstall behavior on the canonical target Windows machine.

When LOCAL testing becomes available, exercise these boundaries before tuning performance numbers or adding fallback mechanisms. A failed boundary should be fixed at its existing owner first; do not introduce a second manager, scheduler, compatibility layer or state machine unless the existing owner cannot represent the proven requirement.
