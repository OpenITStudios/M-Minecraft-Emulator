---
id: document.validation.rea-source-audit-2026-10-13
class: DOCUMENT
domain: validation
role: AUDIT
authority: WORKING
lifecycle: ACTIVE
---

# REA-aligned Full Audit — M-Minecraft-Emulator

## Private core REA evidence campaign — 2026-10-14

**State: SOURCE PLAN READY; NATIVE REA NOT_EXECUTED.** The current Local
branch contains committed-source verification, private-build report contracts
and a read-only package inspector, but still no genuine compiled private
Windows core binaries. Analyzing original upstream source is not REA binary
analysis or live Windows proof.

**Actual source proof:** all seven authentic original source files from
Sandboxie v1.18.5 at abd4c240031cfb46666daa74ca47ed4e48b561e4
passed strict source-patch verification (7 hunks), with two unexpected
source mutations refused. No MSBuild, ABI or driver observation follows
from that result.

### Binary investigation once artifacts genuinely exist

1. Preserve separate untouched upstream and private build outputs with their
   source Git HEAD/tree, private patch SHA-256, compiler/WDK/SDK versions,
   binary SHA-256 and build logs. Work on disposable artifact copies, not
   installed Sandboxie or account-bearing Minecraft processes.
2. Discover an actual available provider and capabilities. For Ghidra,
   developer-side commands are:

~~~text
rea providers --json
rea capabilities --json
rea doctor --provider ghidra --json
rea inspect ABSOLUTE_BINARY_PATH --provider ghidra --json
rea search ABSOLUTE_BINARY_PATH SbieApi_GetVersionEx --kind procedures --provider ghidra --json
~~~

3. Analyze copies of SbieDll.dll, SbieSvc.exe, Start.exe and SbieIni.exe
   first; expand to SbieDrv.sys only if the selected REA provider explicitly
   supports that PE/driver target. Match findings to exact upstream source,
   not filename guesses.
4. For a resolved symbol use the documented REA function/trace commands
   with its exact observed address or name. If a provider reports incomplete
   coverage or unsupported host/target, retain UNKNOWN rather than inferring
   absence. Capture Evidence IDs, limitations and artifact hashes.
5. Use portable Evidence export/compare only between matching evidence
   kinds and coverage. REA comparison is not runtime equivalence or a
   performance/security benchmark.
6. No REA tool output can authorize Windows driver installation, Windows
   credential access, the app backend switch, or Minecraft gameplay.

| Candidate | Investigation target | Pinned upstream source owner |
| --- | --- | --- |
| SbieDrv.sys | Driver ABI/version and process-to-box enforcement | Sandboxie/core/drv/api.c |
| SbieSvc.exe | Service, broker and driver IPC boundary | Sandboxie/core/svc/DriverAssist.cpp |
| SbieDll.dll | API exports and driver/DLL handshake | Sandboxie/core/dll/sbieapi.c |
| Start.exe | Launch and box selection path | Sandboxie/apps/start/start.cpp |
| SbieIni.exe | Configuration access path | Sandboxie/apps/ini/main.c |
| kmdutil.exe | Installer command semantics (static-only) | Sandboxie/install/kmdutil/kmdutil.c |

Official command guidance: https://github.com/morluto/rea/blob/main/docs/cli.md
and https://github.com/morluto/rea/blob/main/docs/native-investigation.md .
REA stays a developer-only investigator, never a production game dependency.

## Additional source-only REA-aligned crosscheck — 2026-10-13

This second audit pass traced `-BoxedImageProofPid` from the PowerShell
producer through Rust and TypeScript IPC parsing to Focus/Place admission.

**Confirmed source-contract defect (new P0, corrected in source):**
`$signedOutside` formerly selected all signed Minecraft Education images
without restricting `inSelectedBox`. During targeted boxed ImageProof,
the collector could publish a nonzero `officialImageOutsideSelectedBoxCount`
while Rust's per-candidate reconciliation correctly counted zero outside
images. Rust would reject that observation, blocking the requested Focus/Place.
The collector now requires both signed evidence **and**
`inSelectedBox -eq $false` for that original outside-only field.
Adversarial frontend fixtures cover the signed-only-boxed case and a
schema-plausible but identity-contradictory counter; tests are **authored
but NOT_EXECUTED**.

**Additional source mitigations:** repeated and sorted `Start.exe /listpids`
snapshots (fall back to signed `SbieDll` on unknown/churn); concurrent
bounded stdout draining before waiting for helper exit; Rust bounded I/O
worker reservations before child spawn; and clearing stale UI Refresh
success notices. These do not prove stable Microsoft-account isolation,
gameplay, worker termination or Sandboxie binary/ABI compatibility.

**REA proof ceiling:** no REA CLI/MCP runtime provider or shipped Windows
binary was available. This is a source-and-architecture comparison using
REA's evidence/unknown discipline, **not** a claim that REA decompiled or
observed the live application. No executable commands, Windows source build,
sandboxed Minecraft sign-in or local test were run. The user currently
defers local testing.

Current issue and next owner are in [development handoff](../../planning/development.md);
the underlying pinned original audit findings below retain their historical
source revision and should not be misread as the current binary state.

## Follow-up remediation on `Local` — 2026-10-13

The findings below are a **historical baseline** pinned to
`d6f9da73b24c79ded0cdf0f75526feacb43f8f9f`. Subsequent source work:
- **F02:** collector now downgrades box membership to UNKNOWN if the Windows
  process list fails or any Sandboxie-reported PID cannot be reconciled.
- **F06:** Sandboxie helper stdout is now read into bounded buffers with one
  total EOF deadline; no unbounded helper `ReadToEndAsync()`.
- **F12:** Client 02 launch button and invocation now require the same
  complete, empty, observed box snapshot; backend always re-inspects.
- **F09:** pure UI projection behavior tests were added, and stale verifier/test
  assertions updated; this is **partial test hardening**, not end-to-end proof.
- **F01/F03/F04/F05/F07/F08/F10/F11/F13–F16:** still require Windows/source
  execution, additional coding, compatibility evidence, gameplay, or release
  decisions as specified below. Stage progress is not inferred from source.
- **Execution ceiling:** 14 targeted source-contract predicates and 24 documentation
  metadata/link inspections were run and matched; `npm --prefix Development run verify:source`,
  PowerShell, Svelte/Vitest, Rust, Windows, Minecraft and REA artifact/runtime
  analysis remain **NOT_EXECUTED**. Follow the [Windows local test](native-pilot-localtest.md).
  The current handoff is in `planning/development.md`.



- **Date:** 2026-10-13 (Asia/Jakarta)
- **Target repository / branch:** MIVUBI-STD/M-Minecraft-Emulator @ `Local`
- **Pinned source HEAD audited:** `d6f9da73b24c79ded0cdf0f75526feacb43f8f9f`
- **REA reference:** morluto/rea @ `a97b713f740932339a5f1036ad2213168c421270`; `.agents/skills/reverse-engineer-anything/SKILL.md`
- **Audit type:** read-only source/contract/architecture inspection, GitHub Actions history review, official REA / Minecraft Education / Sandboxie documentation comparison.
- **Execution:** No REA server or CLI exposed to the audit environment; no app PE, package, JS bundle, running Windows desktop, Rust toolchain or account-bearing host available. **No REA artifact evidence, executed current-HEAD build result, authenticated Minecraft gameplay, RAM/CPU/GPU benchmark or 4-client result was produced.**
- **Evidence convention:** OBSERVED_SOURCE = code or CI log seen; INFERRED_RISK = conditional flaw requiring a reproducer; NOT_EXECUTED = missing runtime proof. A historically failed workflow never establishes a failure of the newer pinned commit.

## 0. Actual target and architecture
The approved outcome is four independently playable Windows Minecraft Education clients using four distinct licensed accounts on one PC, with no modification of the game or Microsoft account tokens. The selected experiment is **one official native Client 01 + one Sandboxie Plus Client 02**. VMware, Fusion, guest-agent and App Sandbox power paths are retained as **inactive** source and preserved user data; their mutations are blocked. This two-client feasibility stage is not a four-client architecture.

Entry points: `Development/apps/desktop/src/main.ts` → `NativeFeasibilityApp.svelte` → `app/bridge/nativeFeasibilityApi.ts` → `src-tauri/src/commands/native_clients.rs` → `src-tauri/src/engine/native_clients.rs` → Windows PowerShell collector / Sandboxie Start.exe. The `Development/engine/runtime-core/src/provider/process.rs` runner executes bounded subprocesses. `Development/engine/runtime-core/src/command.rs` pauses legacy mutation commands. `Development/apps/desktop/scripts/prepare-package-binaries.mjs` deliberately refuses the obsolete VMware NSIS installer.

The source inventory at pinned HEAD contains 189 tracked files (~1.64 MB in Git tree blob size); the active native pilot must not be conflated with legacy code still present in the repository.

## 1. Findings ledger

| ID | Priority | Evidence class | Finding and implication | Canonical first owner | Required closure |
| --- | --- | --- | --- | --- | --- |
| F01 | P0 | OBSERVED_SOURCE + NOT_EXECUTED | No successful **current SHA** Windows source build. Two Oct 10 workflows failed at older SHA `515e076...`: Svelte TypeScript in source verification and Rust compilation in package smoke. Corresponding current source lines have changed, so these historical errors are not claimed to remain. | `.github/workflows/verify.yml`; `planning/development.md` | Execute `npm --prefix Development run verify:source` on exact HEAD, retain full step logs and failure owner. |
| F02 | P0 | INFERRED_RISK | Collector uses `Get-Process -ErrorAction SilentlyContinue` to derive candidates, while backend admits launch when `isolated_candidate_processes == 0` after box membership observation. Missing/denied/incomplete process enumeration could be misinterpreted as a clear game candidate inventory; no boxed-PID-to-visible-process completeness reconciliation is part of the admission contract. **No duplicate launch has been observed.** | `Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1`; `src-tauri/src/engine/native_clients.rs` | Introduce a single fail-closed enumeration completeness condition; test inaccessible PID, exited PID, mixed box processes, game without HWND, empty box, delayed launch. Preserve account privacy. |
| F03 | P0 | NOT_EXECUTED + EXTERNAL | `OpenCredentials=OFF` and scanned `OpenProtectedStorage` flags do not establish independent Microsoft WAM/WinCred/authentication contexts. Sandboxie templates (including nested or built-in effective settings) are not exhaustively resolved by the collector. | Collector + two-client acceptance procedure | Manually prove two different licensed accounts with two independently usable worlds, then close/reopen/reboot; never copy or read tokens. |
| F04 | P0 | OBSERVED_SOURCE + NOT_EXECUTED | Successful Sandboxie `Start.exe` exit/accepted launch is not evidence of a second Minecraft process, separate window, login or rendering. The source deliberately labels this as a REQUEST, which is correct. | `native_clients.rs`, UI status, pilot acceptance | Observe two game windows and interactive worlds; record elapsed launch, failure point, and whether the official app refocuses an existing instance. |
| F05 | P1 | INFERRED_RISK | Rust bounded runner owns direct child but spawns detached pipe-reading and stdin-writing threads. If a child exits while a descendant holds its stdout/stderr open, the call times out yet detached reading threads can remain waiting for EOF. Repeated use may accumulate workers; this is conditional, not a confirmed leak in the live app. | `Development/engine/runtime-core/src/provider/process.rs` | Add deterministic Windows test with inherited pipe handles, verify bounded worker lifetime and no killing Minecraft descendants. |
| F06 | P1 | INFERRED_RISK | `Invoke-BoundedReadOnlyTool` calls `ReadToEndAsync()` then enforces the 4096-character cap only **after** reading. Excessive output may allocate memory before rejection; a 5-second command deadline is not an allocation bound. | Sandboxie collector | Bound stdout collection during read; add verbose/malformed/never-EOF process tests. |
| F07 | P1 | NOT_EXECUTED | No per-client memory, CPU, VRAM, launch-latency, input, network and 60+ min soak comparison on target Windows hardware. “Lightweight” remains a design objective, not measured performance. | `Development/tooling/acceptance/windows` + `docs/validation` | Establish one-client baseline and two-client profile on same hardware; collect safe aggregate metrics, no private worlds/accounts. |
| F08 | P1 | INFERRED_RISK | Launch checks current-user `Get-StartApps` by three exact English display names. An official installed entry exposed under another name/language may be rejected even when its AppID/install is valid. Registry/Start AppID link to custom APPDIR executable also remains unresolved. | `native_clients.rs` | Verify at least English + Indonesian/localized Windows and official custom Desktop APPDIR; test candidate binding without relaxing signer/edition checks. |
| F09 | P1 | OBSERVED_SOURCE | Source tests cover strict IPC parsing and launch admission, but a significant subset only checks that source strings are present (`toContain`). No executed end-to-end Windows/Sandboxie test on this SHA. | `Development/apps/desktop/test/native-feasibility.test.ts`, `Development/engine/runtime-core` tests | Add behavior-level mocks/fakes and OS integration probes that exercise failure, retry, invalidation and process/window churn. |
| F10 | P1 | OBSERVED_SOURCE | Existing NSIS package is intentionally blocked to protect old VMware data and identifier. This is **not a defect to bypass**. A distinct native prototype install/recovery route remains unimplemented. | `prepare-package-binaries.mjs`, `docs/system/legacy-retirement.md` | Only after source and 2-client proof, authorize separate prototype ID/payload/data directory and install/uninstall coexistence tests. |
| F11 | P1 | INFERRED_RISK | Sandboxie tools are identified primarily by installation path and valid Authenticode signatures; Start.exe, SbieIni.exe are not pinned to an expected Sandboxie publisher/product in the main launch path. DLL comparison to Start.exe thumbprint exists. A `Valid` certificate does not independently attest vendor identity or path↔installed product provenance. | `native_clients.rs` and collector | Assess actual vendor signer metadata and file-identity/installation binding on a known-good installation; avoid speculative certificate pinning. |
| F12 | P2 | OBSERVED_SOURCE | UI allows a Client 02 launch attempt after manual selection/license checkbox even before a successful Refresh. Backend then rejects incomplete state; the user gets avoidable friction. | `NativeFeasibilityApp.svelte` | Present an explicit Check setup → Launch state, or make launch run its own visible preflight with actionable stage-specific recovery. Backend must remain authoritative. |
| F13 | P2 | OBSERVED_SOURCE | A single global `busy` disables all desktop buttons during launch/inspection; operations share a 30-second collector ceiling. User may perceive a hung app without stage progress. | `NativeFeasibilityApp.svelte` | Add short stage labels and safe cancellation/return only where side effects are understood; never retry an ambiguous launch automatically. |
| F14 | P2 | OBSERVED_SOURCE | Active package and dependency graph retain legacy guest/VM source and old product identity. Deliberate for data safety, but the app binary size, cold-start and maintainability costs are unmeasured. | `legacy-retirement.md`, Cargo manifests, Tauri config | Measure once prototype exists; archive or decompile no legacy code until rollback/migration conditions are proven. |
| F15 | P2 | INFERRED_RISK | Focus/Place verifies PID, FILETIME, HWND and signed-file provenance, but signed-file verification may fail due to AV, Sandboxie interception, reparse/long paths or localized publisher metadata. The fail-closed result is safe but could make Focus/Place unreliable for real users. | `native_clients.rs` + `collect-sandboxie-feasibility.ps1` | Capture failure categories and latency on actual Windows; allow manual Alt+Tab as fallback, not weaker identity checks. |
| F16 | P2 | OBSERVED_SOURCE | The source evaluator's `OPERATOR_REPORTED_COMPATIBLE` remains based on operator declarations plus candidate windows; its separate `productAcceptance` stays `NOT_EVALUATED`. Correctly labeled but it must not be promoted to proof of four clients. | `Development/tooling/acceptance/evaluate-sandboxie-feasibility.mjs` | Preserve human/machine separation in all summary displays and release decisions. |

### Code anchors
- F02: `Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1` approx. lines 433–462; `Development/apps/desktop/src-tauri/src/engine/native_clients.rs` approx. lines 353–376, 724–865.
- F05: `Development/engine/runtime-core/src/provider/process.rs` approx. lines 54–196.
- F06: `Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1` approx. lines 60–95.
- F08: `Development/apps/desktop/src-tauri/src/engine/native_clients.rs` approx. lines 134–191, 747–791.
- F09: `Development/apps/desktop/test/native-feasibility.test.ts` approx. lines 69–209.
- F10: `Development/apps/desktop/scripts/prepare-package-binaries.mjs` lines 1–27.
- F12/F13: `Development/apps/desktop/src/NativeFeasibilityApp.svelte` approx. lines 8–149, 270–295.

## 2. Positive controls to preserve

1. Current UI mounts only native feasibility, with old VM mutation handlers unregistered.
2. Public VM CLI mutation commands are paused and fail before acting on account-bearing disks.
3. IPC process observations have strict schemas, bounded counts, identity checks and no account tokens/window titles.
4. UNKNOWN Sandboxie membership refuses launch rather than inferring an empty box.
5. The backend re-inspects before launch and rechecks exact PID/time/window for Focus/Place; it distrusts the UI snapshot.
6. File verification checks signature/ProductName and pins file identity for selected boxed Focus/Place actions.
7. Root subprocess uses a bounded stdin payload to avoid CreateProcess command-line limits; output has a bounded retained buffer.
8. Packaging guard prevents deploying a new native pilot under the old VMware installer identity.
9. Evaluator distinguishes observed candidates, operator reports and product acceptance; an authored test is never a runtime PASS.

Do not remove these safeguards merely to get an apparent successful demo.

## 3. Ordered remediation batches

**Batch A — evidence-quality + correctness (first):**
- A1: Run exact-HEAD Windows `npm --prefix Development run setup && npm --prefix Development run verify:source`; record individual step outcomes, never reuse 2026-10-10 red runs as current.
- A2: Make boxed-PID/process inventory completeness explicit; fail closed on unresolved in-box PID enumeration, while preserving expected non-Minecraft helper processes.
- A3: Add targeted test cases for empty box, stale PID, inaccessible PID, game process without HWND, duplicate launch, launch races, Sandboxie API unavailable, timeouts.
- A4: Make UX display the first recovery action and avoid enabling a launch path prematurely; backend remains canonical.

**Batch B — two-client real Windows feasibility:**
- B1: Validate official Desktop installation, Store coexistence guard, English/localized Start app enumeration and custom APPDIR.
- B2: Validate Sandboxie box exists, is persistent, properly licensed, and exposes membership correctly (CLI and vendor DLL fallback).
- B3: Attempt a native + boxed window using different valid Education accounts; verify rendering, keyboard/mouse, world loading, multiplayer and independent sessions without accessing secrets.
- B4: Close/reopen/reboot; record exceptions, cross-account state, data preservation and failure recovery.
- B5: Measure cold/warm starts and sustained CPU/RAM/VRAM/network/interaction; don't invent arbitrary performance PASS numbers.

**Batch C — REA artifact and runtime campaign:**
- C1: Only once build succeeds, retain original Windows PE/Tauri resources and frontend distribution for the exact same SHA; hash artifacts.
- C2: On an approved local REA host run `rea capabilities` and `rea providers` to record actual tool availability; for the packaged app use `rea inspect <absolute app artifact>`, and `rea analyze-javascript-application <absolute extracted frontend directory>` where supported.
- C3: Map executable entrypoints/IPC/native imports, package resources, credential boundaries and launch/error paths back to canonical source owners; retain the REA Evidence IDs, raw digest, coverage, failed requests and UNKNOWNs.
- C4: Compare two verified versions by portable REA Evidence bundles using `rea compare <left-evidence.json> <right-evidence.json>` where applicable. Static binary similarity is not behavioral equivalence.
- C5: Runtime observation (if supported and approved) must be passive, bounded, on the user's own app; no credentials, tokens, world content, DLL injection into Minecraft or game binary modification.

**Batch D — four-client scaling:** Only after Batch B demonstrates real two-client independent gameplay and persistence, decide whether one additional isolated instance is technically warranted. Stop rather than multiplying an unproven approach.

## 4. REA audit integration: not a second runtime engine

REA is a **developer-side evidence tool**, not a Minecraft lifecycle service. Keep REA scripts, Evidence bundle references and investigation notes under `docs/validation/` and ignored private `workspace/` outputs. Do not add REA as an app dependency, resident daemon, duplicate state machine or automated authority for licensing and gameplay. The canonical runtime, acceptance evaluator and product status remain unchanged.

Minimum retained audit record per artifact: `source_sha`, `artifact_sha256`, `artifact_kind`, `os_and_tool_versions`, `rea_tool_and_version`, `evidence_id`, `operation`, `scope`, `observations`, `inferences`, `unknowns`, `limitations`, `reproducer`, `reviewer_decision`. Unknown evidence stays UNKNOWN; **absence of a match in partial analysis never establishes feature absence**.

## 5. Exit/stop decisions

- **Source-ready:** exact SHA built and verified by actual Node/Rust/PowerShell checks, with proof logs.
- **Two-client candidate:** two observable windows PLUS two authenticated independent game worlds with responsive controls; process counts alone insufficient.
- **Two-client suitable:** repeatable session isolation, persistence through reopen/reboot, multiplayer, acceptable actual host resource usage and no account crossing.
- **Four-client release-ready:** all four independently playable accounts simultaneously, repeatable recovery and monitored stability, reviewed packaging/data migration; never extrapolate from two windows.
- **No-go:** app refocuses existing window, Microsoft login crosses accounts, Minecraft won't render or control independently, sandbox persistence loses data, or real resource pressure makes gameplay unusable.

## 6. Citations / authoritative evidence

- GitHub source pin: https://github.com/MIVUBI-STD/M-Minecraft-Emulator/tree/d6f9da73b24c79ded0cdf0f75526feacb43f8f9f
- Historical failed source workflow (older SHA): https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/38067894851
- Historical failed package workflow (older SHA): https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/38067894820
- REA source skill: https://github.com/morluto/rea/blob/main/.agents/skills/reverse-engineer-anything/SKILL.md
- REA evidence workflow: https://github.com/morluto/rea/blob/main/.agents/skills/reverse-engineer-anything/references/evidence-workflows.md
- Official Minecraft Education Windows installer: https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide
- Sandboxie Start.exe CLI: https://sandboxie-plus.github.io/sandboxie-docs/Content/StartCommandLine/
- Sandboxie SbieIni CLI: https://sandboxie-plus.github.io/sandboxie-docs/Content/SbieIniCommandLine/
- Sandboxie credential setting: https://sandboxie-plus.github.io/sandboxie-docs/Content/OpenCredentials/
- Sandboxie protected storage: https://sandboxie-plus.github.io/sandboxie-docs/Content/ProtectedStorage/
- Sandboxie template architecture: https://sandboxie-plus.github.io/sandboxie-docs/Content/TemplateArchitecture/
- Windows Get-StartApps reference: https://learn.microsoft.com/en-us/powershell/module/startlayout/get-startapps

**Completion state:** This is a completed **source and evidence-gap audit**, with a staged full REA binary/runtime campaign prepared but **NOT_EXECUTED**. No code or account-bearing data is changed by this report.
