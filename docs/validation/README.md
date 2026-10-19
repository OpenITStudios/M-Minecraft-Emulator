---
id: document.validation.router
class: DOCUMENT
domain: validation
role: ROUTER
authority: CANONICAL
lifecycle: ACTIVE
---

# Validation and Evidence

## Selected native + Sandboxie two-client experiment (2026-10-12)

The approved *new product outcome* is four Minecraft Education Windows
sessions with four independent accounts, **without VM guests if feasible**.
The current engineering experiment is one native Client 01 plus one
Minecraft Education Desktop launch requested inside a persistent
[Sandboxie Client 02](./sandboxie-two-client-feasibility.md).
Native-only simultaneous login is not the active implementation route.
The evaluator distinguishes process candidates from manually reported
two-account gameplay. Its *final two-client verdict* still requires observed
multiplayer and account persistence after reopening and reboot; no two-client
result establishes four-client product acceptance.
This is an evidence collector and evaluator only; it does not imply native
multi-instance works or supersede any passed legacy VM package G0 record.
Existing VMware G1–G4 remain unexecuted and unrelated to native viability.
The former VMware NSIS bundle is blocked from shipping while the native
feasibility desktop is mounted; no native distribution migration is proven.
The active Client 02 launch is explicitly user-requested. Its shared
Sandboxie collector remains a read-only *inspection of host configuration*,
returns **no-write SummaryOnly** evidence to the desktop; a separately invoked full CLI collector writes a bounded JSON report. Neither can verify Microsoft sign-in.

| Question | Authoritative reference |
| --- | --- |
| What is the **current** v1.18.6 owned source/build path? | [One native source/build owner](../../tooling/isolation-source/README.md) — unsigned, noninstallable and not the current runtime provider |
| What was the old v1.18.5 build inventory? | [Historical pinned upstream inventory](./owned-isolation-upstream-inventory.md) — research snapshot only |
| What passed in the first two-client run, and what remains unstable? | [First success and subsequent crash, 14 Oct 2026](./two-client-first-success-2026-10-14.md) — operator-confirmed gameplay, observed compatibility probes and unresolved idle crash |
| How do I prepare and execute the current Windows local test safely? | [Native + Sandboxie local test](./native-pilot-localtest.md) — exact-HEAD build, manual two-client steps, failure handling and evidence limits |
| Can one native Minecraft and one sandboxed Windows Minecraft run independently? | [Sandboxie Client 02 feasibility](./sandboxie-two-client-feasibility.md) — single active experiment, not game-proof |
| What did the 13 October source audit observe? | [Historical REA-aligned audit](./rea-source-audit-2026-10-13.md) — former source snapshot, not a current release verdict |
| What was the earlier native-only experiment? | [Native two-client feasibility](./native-multi-instance-feasibility.md) — historical evidence route; not active architecture |
| What were the historical VMware-only product gates? | [Legacy VMware G0–G4](./acceptance.md) — preserved five-gate VMware campaign, **not** the active native acceptance procedure |
| What did historical VMware G0 require? | [Legacy VMware G0 Source & Package Readiness](./pre-local-test-readiness.md) — old package route remains blocked for native pilot |
| Can one Windows VMware guest boot, install/sign in and play Minecraft Education responsively? | [Client Feasibility Gate](./client-feasibility.md) — before Base cloning |
| Where is the old VMware target-machine acceptance? | [Historical VMware Acceptance](./acceptance.md) — not needed to start the selected native two-client experiment |
| What is statically covered versus unproven? | [Static Readiness](./static-readiness.md) |
| What specific four-client failures must be exercised? | [Four-Client Failure Matrix](./four-client-failure-matrix.md) |
| What does the Minecraft/VMware platform officially require? | [Platform Prerequisites](./platform-prerequisites.md) |

The existing Rust tests, Vitest, static checks, and Windows stage recorder provide supporting evidence. They do not introduce more product-progress gates; no new status schema or runner is used.

## Historical VMware evidence hierarchy (inactive for the native pilot)

The sequence below documents the preserved VMware implementation. The
current Sandboxie pilot first checks two independently playable Windows clients without VMs;
it does **not** require guest cloning or an NSIS package.

```text
Exact GitHub source / package verification
→ one interactive Windows VMware guest: boot
→ Minecraft Education Windows desktop install and distinct licensed login
→ real world rendering, input and responsive gameplay
→ prepared account-clean Base + three isolated Windows guests
→ Native + Virtual-01/02/03: four separate licensed sessions concurrently
→ measured load, recovery and optional multiplayer acceptance
```

A lower level cannot authorize the next. Machine collector source lives in `../../tooling/acceptance/windows/`; a completed collection proves only the fields directly observed. Player/gameplay acceptance still requires real Minecraft evidence.

**GitHub Actions CI disabled** by explicit operator decision. No automatic or manual Actions workflows are active in `Local`; the former `main` workflow registration is retired. Existing `npm run verify:source`, individual tests and debug-build commands are retained for *explicit local/manual use only* when an appropriate environment exists. Historical CI artifacts remain evidence for their original SHAs, not the current HEAD. [Current handoff](../../planning/development.md) distinguishes source review from actual Windows/gameplay proof.
