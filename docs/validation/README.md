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

For the **default efficient first local acceptance**, start with the
[13-scenario progressive local smoke plan](efficient-local-testing.md).
The explicit read-only `DEV.cmd plan --profile local-smoke --json` works
after approved portable Node setup; it does not confuse a docs-only latest
commit with completed first-run acceptance. Source quick check, game acceptance
and optional installer remain separate approvals. Optional `DEV.cmd smoke
start/record/status` stores bounded operator-reported case outcomes only
under ignored `workspace/runs`—not automatic gameplay acceptance.

For a **standard desktop clone, first UI run and optional native package**,
start at the [Windows first-run checklist](local-pc-first-run.md).
It records the missing reviewed-DLL prerequisite and distinguishes
preflight, UI smoke tests, packaging and real game acceptance.

For the **complete future testing inventory**, use the
[master operator guide](lazy-tester-test-master.md), the
[198-case standard product matrix](lazy-tester-test-cases-standard.md), and
the [108-case MCP/AI/QA matrix](lazy-tester-test-cases-experimental.md).
Each case has a unique ID, priority, action, expected observation and
reporting rules. **All 306 cases remain NOT_EXECUTED** at authoring;
these docs do not authorize execution or create a new runner or status database.

**Current AI Computer Use validation is routed only through the
[controlled Windows local-test guide](ai-computer-use-localtest.md).**
It distinguishes standard source preflight, opt-in native compilation/ABI,
MC01-human/MC02-AI gameplay and later four/five-client concurrency.
Earlier VMware/Sandboxie G0 notes are historical, not this test path.
No current Minecraft background-input acceptance has been executed.

**Probe QA is currently source-only.** The
[one-entry QA workflow and proof gates](../system/map-aware-probe-qa.md)
distinguish an approved Probe ZIP, local log file and native window from
the **unverified** admin Player, loaded QA world and live Content Log
source identity. A Probe-reported all-PASS does not certify map gameplay.
This is a future runtime acceptance gate, not a request for local
testing now.

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

## Native five-profile acceptance sequence (not yet executed)

For one exact `Local` candidate SHA, verify existing Windows source and native
package checks before claiming an installed build. On the target Windows host:

1. First run: open Lazy Tester and manually launch MC01; authenticate with an
   independently licensed Minecraft Education account and enter a responsive world.
2. Parallel sessions: manually open MC02–MC05 with distinct licensed accounts;
   confirm five playable processes and isolated profile identities.
3. Workspace: inspect selected live window, four passive DWM previews, focus,
   resize and minimize/restore without altering unrelated game windows.
4. Lifecycle: check normal CloseOne/CloseAll, pending game dialogs, app exit
   with retained game ownership and reconnect after reopening the manager.
5. Persistence: confirm profile/account/world state after game restart and
   reviewed install/repair/uninstall/reinstall; no silent data migration/deletion.
6. Measure RAM/FPS/startup only after the same real workloads are confirmed;
   existing read-only collectors may be used, never inferred from source.

Record observed PASS/FAIL/UNKNOWN against the exact revision and environment
in the existing handoff/evidence owner. Source, browser fixtures and installer
hashes alone do not establish Windows, Minecraft or five-client acceptance.
No new runner, hosted CI, status registry or automatic game operation is required.

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
