---
id: document.validation.four-client-failure-matrix
class: DOCUMENT
domain: validation
role: REFERENCE
authority: CANONICAL
lifecycle: ACTIVE
---

# Virtual Clients — Four-Client Failure Matrix

> **Historical VMware-only procedure.** This document describes the preserved
> guest-VM implementation and its G0–G4 evidence campaign, **not** the active
> Minecraft Instance Manager + Sandboxie Plus Client 02 workflow. Do not
> provision, start, delete or reset a VM as part of the current pilot.
> Active procedure: [Sandboxie two-client feasibility](sandboxie-two-client-feasibility.md).
> Its signed-in gameplay and resource measurements are still unverified.

Status: REMOTE_GITHUB scenario model. This is not runtime proof. These scenarios are **supporting assertions** for the [five and only five product gates G0–G4](acceptance.md), not additional micro-test milestones. Apply the first-owner and stop rules below; do not create a runner or separate scenario-status store.

Target:
`Native + Virtual-01 + Virtual-02 + Virtual-03` running Minecraft Education
concurrently on one physical Windows host.

| Domain | Scenario / trigger | Expected product behavior | First owner | Proof |
| --- | --- | --- | --- | --- |
| Interactive launch | SYSTEM Guest Agent receives launch | Handoff to an interactive-user launcher; never claim visible UI from session 0 | Guest Agent / guest Windows setup | Windows |
| Interactive launch | No user logged on | Keep VM running; report interactive session unavailable | Guest Agent | Windows |
| Interactive launch | Multiple Windows user sessions/RDP sessions in one Virtual | Reject acceptance; one Virtual owns one interactive Windows user session so localhost launcher routing is unambiguous | guest setup | Windows |

| Interactive launch | Interactive helper missing/stopped | Loopback readiness is false; keep VM running and direct user to per-user launcher setup | Guest Agent IPC | source + Windows |
| Login | Minecraft opens auth surface | Auth surface stays inside the same Virtual Windows session | Minecraft/Windows session boundary | Windows |
| Login | Browser/WebView opens on another monitor | User can still reach it; do not assume fixed tab/window location | Window UX | Windows |
| Login | MFA / Conditional Access / federation | Remain manually usable; Virtual Clients never intercept credentials | account boundary | manual |
| Login | OAuth/deep-link callback | Return to the same VM/Minecraft instance | Windows/Minecraft | Windows |
| Login | Session expired | Minecraft requests sign-in normally; process-running is not presented as signed-in proof | UI semantics | manual |
| Identity | Same account used concurrently where license/policy forbids it | Surface account/license failure separately from VM/network health | Minecraft/account | manual |
| Isolation | V1/V2/V3 configured with distinct accounts | Stop/start/reset preserve only the selected Virtual's session | recovery + guest disk | Windows |
| Isolation | Base image contains user auth | Block acceptance; Base must remain account-clean | Base preparation | manual |
| Process | Minecraft already running | Launch request is idempotent; no duplicate instance | Guest Agent launcher | source + Windows |
| Process | Minecraft launch times out | VM remains running; offer Launch Minecraft retry | runtime + UI | source + Windows |
| Process | VM console open fails after launch | Report console-open failure without claiming Minecraft failed | provider/open | source + Windows |
| Concurrency | Start three Virtuals | VM and Minecraft launches remain staggered; no launch storm | runtime pressure policy | source + Windows |
| Concurrency | Native + 3 Virtual all active | All four menus remain interactive concurrently | host capability | Windows |
| CPU/RAM | host pressure becomes CRITICAL | Block new starts; do not kill existing clients | resource admission | Windows |
| GPU | three VMware 3D guests + Native | Observe renderer stability; do not invent GPU thresholds without evidence | acceptance | Windows |
| Input | switch focus V1→V2→V3→Native | No stuck VMware capture; overlay never takes focus | VMware + overlay | Windows |
| Audio | four clients produce audio | Workflow remains usable; mute policy only if evidence requires it | acceptance | Windows |
| Display | true fullscreen entered | Record behavior; initial supported layout target is windowed | Window Layout | Windows |
| Network | all guests have Internet | Do not infer multiplayer readiness from Internet alone | network acceptance | Windows |
| Network | VMware NAT | Record NAT behavior for Native↔VM and VM↔VM host/join | provider/network | Windows |
| Network | bridged/custom network | Evaluate only if NAT evidence fails required multiplayer scenarios | provider/network | Windows |
| Multiplayer | Native hosts, V1/V2/V3 join | All four remain connected and controllable | Minecraft/network | Windows |
| Multiplayer | V1 hosts, Native/V2/V3 join | Reverse topology also works | Minecraft/network | Windows |
| Multiplayer | one VM disconnects | Other three remain stable; disconnected VM can rejoin manually | Minecraft/network | Windows |
| Suspend | joined V2 suspended | Other clients continue; resume outcome recorded honestly | lifecycle | Windows |
| Reset | V2 QA_READY reset | V1/V3/Native unaffected; V2 account state returns to its own checkpoint | recovery | Windows |
| Host reboot | physical host restarts | Each Virtual returns to its own configured account/session when still valid | persistence | Windows |
| Version | Native updates | stale Base/Virtual lineage blocks daily start until rebuild/reprovision | version authority | source + Windows |
| Networking | Base network mode differs by machine | Base preflight exposes `networkConnectionType`; acceptance records proven mode | base preparation | source + Windows |
| Window Layout | one Minecraft missing | Arrange uses discovered windows and reports missing; no background reflow daemon | Window Layout | Windows |
| Overlay | overlay fails | Arranged windows remain valid; warning is nonfatal | Screen Overlay | Windows |

## Stop rules

Do not add a new subsystem merely because a scenario is unproven.

Promote a source change only when:
1. the scenario has a concrete first owner;
2. existing owner cannot express the required behavior; or
3. target-machine evidence demonstrates a specific failure.

Do not automate Microsoft credentials, MFA, OAuth tokens, browser cookies, or
Minecraft account sessions.

Do not treat these as equivalent:
`VM running`, `Minecraft process running`, `Minecraft menu usable`,
`account signed in`, and `multiplayer ready`.

## Resource interpretation

The Virtual profile's 4 GB memory ceiling is an intentional concurrency/QA
trade-off, not a claim that it matches Minecraft Education's recommended
per-device memory. Keep the existing host-pressure authority and low-graphics
profile until target-machine evidence shows that a different ceiling improves
three-guest concurrency without destabilizing the host.



## Network design principles

Virtual Clients keeps networking deliberately narrower than a general application gateway.

Adopted principles:
- local-only helper transport stays bound to loopback;
- Guest Agent transport remains authenticated per Virtual and exposes bounded product actions only;
- health/readiness and gameplay capability remain separate observations;
- ambiguous mutation outcomes are reconciled from provider truth before any retry;
- mutation requests are never automatically replayed when their outcome is unknown;
- malformed or unsupported protocol input fails closed;
- Internet connectivity is diagnostic evidence only and never proves Minecraft multiplayer readiness.

Do not copy a generic HTTP/MCP gateway into Virtual Clients. Add transport machinery only when an existing bounded owner cannot express a proven product requirement.
