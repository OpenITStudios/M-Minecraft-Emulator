---
id: document.validation.native-pilot-localtest
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Minecraft Instance Manager — Windows local test (native + Sandboxie)

## Owned Windows engine campaign — 20 October 2026

**Build/staging COMPLETE; activation, owned operations and four Minecraft NOT_EXECUTED.**
The operator selected owned v1.18.6 first: one native Client 01 and three owned
persistent profiles 02–04. The official two-client desktop pilot below remains
the mounted implementation, not an automatic fallback for this campaign.
No owned provider has been activated.

Executed source revision: `f529d59fda778be50cd3e874374a5546c32dce29`.
Baseline: `8c1c6b459b8d6a69bb5683200d1d1ef120247460`.
This section owns campaign results; the dependency report retains dependency scope.

| Check | Result | Actual evidence / reason |
| --- | --- | --- |
| Remote and checkout identity | PASS | Clean Local matched remote baseline before edits |
| Exact source provenance | PASS | 1,627 files; pristine upstream unchanged; 11 C/C++ changes plus four exact project quoting fixes |
| Full source pipeline | PASS | 663 passed: 262 Node, 161 Vitest, 240 Rust; 0 failed, 4 existing file-symlink skips |
| Native C/C++ policy executables | PASS | MSVC Release build and three CTest tests: launcher parser, core identity/ABI matching, kernel box policy |
| MFC v143 x86/x64 setup | PASS | User authorized system setup; installer exit 0 and afxres.h present |
| WDK 26100.6584 setup | PASS | Microsoft signature Valid; installer exit 0, kernel headers present, no reboot |
| WDK integration into VS2022 | PASS | Official RunAs retry after operator clarification exits 0; kernel toolset registered; no UAC/security-policy bypass |
| Native Win32 engine build | PASS | Real canonical builder stage exit 0; both required Win32 binaries present |
| Native x64 user-mode build | PASS | Real canonical builder stage exit 0; thirteen required x64 user-mode binaries present |
| Native x64 kernel build | PASS | Real canonical stage exit 0; driver Authenticode status NotSigned |
| Sixteen binaries / five resources / four licenses staged | PASS | 25 artifact records bound to source revision, actual build report and SHA-256; state MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE |
| Dependency audit | PASS with warnings | Prior source batch: zero vulnerability-class findings; manifests unchanged; two GTK informational warnings retained |
| Four-process telemetry | PASS | Prior batch at 8928602: actual 30-second disposable Node captures on PowerShell 7/5.1; six samples each; exited Client 03 stays UNKNOWN |
| Production signing / activation / owned operations | BLOCKED | Candidate deliberately unsigned; no Windows-accepted production signature or active owned adapter; no driver/service/account changes |
| Owned gameplay: one, two, then four | NOT_EXECUTED | Runtime unavailable; no login, world, independent input, recovery or multiplayer observation |
| Four-client 60-minute soak, active 60 FPS | NOT_EXECUTED | No four-client game session or displayed-frame capture |
| Background cap near 10 FPS | NOT_EXECUTED | No supported, measured vendor/API route selected; no FPS mutation added |

The full engine attempt used VS2022 Build Tools 17.14, MSVC 14.44.35207,
MSBuild 17.14.40.60911 and Windows SDK 10.0.26100.0. The earlier RC1015 missing-MFC
failure is resolved. MFC and WDK setup were explicitly authorized by the user and
completed. VS2022's WindowsKernelModeDriver10.0 integration is now installed;
all three native build stages pass. Production signing credentials and Windows
driver acceptance were not assessed; the unsigned candidate cannot be activated.

Installed components: MFC v143 x86/x64
(`Microsoft.VisualStudio.Component.VC.ATLMFC`) in the existing VS2022 Build Tools
and WDK 26100.6584 (bundle version 10.1.26100.6584; SDK/WDK path build 26100).
The WDK bootstrapper SHA-256 is
`ed82c46bd98e0f1d07fd6e5075900e42aedc3fa5e68c06a8764b3dc5303cff1b`;
Authenticode was Valid with Microsoft Corporation as signer before execution.
The VS component `Component.Microsoft.Windows.DriverKit.BuildTools` is installed.
After the earlier canceled consent attempt, the operator reported no visible
dialog; a normal official RunAs retry completed with exit 0. No UAC bypass,
manual system-file copying, driver installation, reboot or security-policy change
was performed.
Microsoft lists this kit for VS2022; no VS2026 toolchain migration was attempted.
Sources:
[Microsoft WDK downloads](https://learn.microsoft.com/en-us/windows-hardware/drivers/other-wdk-downloads)
and [Build Tools component IDs](https://learn.microsoft.com/en-us/visualstudio/install/workload-component-id-vs-build-tools?view=vs-2022).
System-component authorization does not include reboot, driver installation,
replacing Sandboxie, changing signing policy or accessing game accounts.

Builder repairs capture VS discovery status in a fresh strict PowerShell session,
wait for the compiler rather than inherited descendants, and retain the process
handle so Windows PowerShell 5.1 preserves its real exit code. Actual exit-7 and
VS discovery regressions pass. The verifier reconstructs all four project fixes
from pristine source; the eleven C/C++ transformation checks remain active.
The standalone identity test uses a bounded copy without suppressing warnings.
No upstream snapshot or driver runtime policy was modified.

### Native-build findings resolved in the final candidate

The first fully compiling attempt exposed WDK's implicit test-sign default:
it generated a temporary test certificate and test-signed the local driver.
That output was rejected for final use, was never installed or loaded, and its
generated certificate plus private key were removed. Targeted checks found no
remaining matching certificate in the user/machine My, Root or TrustedPeople
stores. No real signing credential was accessed or removed.

The builder now passes `SignMode=Off` and `EnableTestSign=false` to every stage
and checks the resulting driver's actual Authenticode status is `NotSigned`
before issuing a successful compile-only receipt. The final clean build meets
that check. A test-signed local binary is not production signing evidence.

Staging also exposed Git autocrlf expansion in configuration/license files.
The assembler now accepts original bytes or CRLF→LF normalization only when
the resulting Git blob ID exactly matches committed HEAD; it writes those exact
bytes and retains the final copied-blob check. Modified configuration or licenses
still fail even after an earlier clean-status observation. Regression tests cover
both accepted CRLF checkout resources and tampering. No tracked resource or
license content was rewritten to accommodate staging.

The existing sampler accepts four distinct explicit targets, pins creation times,
keeps lost identities UNKNOWN, and publishes sanitized schema-3 CPU/RAM reports.
Schemas 1/2 remain compatible. It neither authenticates owned Minecraft targets
nor measures FPS. Usage belongs to the
[Windows tooling guide](../../tooling/acceptance/windows/README.md).

Successful build logs and sixteen hashes are ignored under
`workspace/builds/owned-20261020-150022/`; the matching 25-file artifact manifest is
`workspace/packages/owned-20261020-150022/engine-provenance.json`.
The package is local and NOT_INSTALLABLE; it is not a desktop installer or release.
Earlier rejected output is preserved in `workspace/builds/owned-20261020-145802/`.
WDK setup and new source/policy logs are under
`workspace/dependency-maintenance/wdk-26100-install*`, `owned-toolchain-*`
and `owned-unsigned-*`;
earlier telemetry remains under `owned-four-*`. Earlier failed compiler outputs
were preserved beneath their build-report directories before clean retries.
No binary, account identity, private world, token, certificate or raw host report
is committed.

### Remaining implementation and acceptance sequence

1. Preserve this unsigned candidate and its exact source/build hashes for signing
   review. Native build/staging are complete; do not reinstall or rebuild merely
   to repeat a passing check. Any new source change requires fresh bound evidence.
2. Verify the complete candidate's hashes, ABI, Windows-accepted signatures,
   service provenance and recovery instructions. Review a concrete activation
   proposal before driver/service changes. Never silently replace official
   SbieDrv/SbieSvc. Reduced-containment settings require explicit review and do
   not prove WAM/account isolation.
3. Implement owned create/launch/inspect/stop, fresh provider/profile/PID/start-time/
   session attribution, cross-process admission and partial-failure handling in
   existing runtime/desktop owners. **Those operations and owned typed bridge
   integration are not implemented by this build/telemetry batch.** Slot admission
   stays blocked; caller flags, checksums or process names cannot authorize it.
4. Prove one owned client, then two, then four with manually signed-in distinct
   licensed accounts, responsive worlds and independent input. Test native↔owned
   and owned↔owned multiplayer, rejoin and sequential client restart without
   session loss or cross-account contamination. Stop escalation on failure.
5. Run four clients for 60 minutes with matching version, graphics and world
   conditions. Rotate focus through each client and capture displayed FPS during
   focused 60-second post-loading windows to evaluate the active 60-FPS target.
   Inter-present data alone cannot award displayed-FPS PASS. Record CPU/RAM,
   available GPU/VRAM diagnostics, crash/freeze and memory growth; no arbitrary
   RAM/GPU threshold is added. Background clients must remain responsive; a supported
   cap near 10 FPS is optional and reported separately.

**Delivery is partial, not full plan completion.** Build/signing/runtime/gameplay
prerequisites remain. GitHub Actions stays OFF, main untouched and VMware paused.

**Scope:** developer-mode local feasibility check for **one native Minecraft Education Client 01 + one Sandboxie Plus Client 02**, on a **single Windows 10/11 x64 PC**. Does not install/replace the legacy VMware manager. Does not start/delete/reset VMware guests and does not automate Microsoft sign-in. The goal of four independently playable licensed sessions is **NOT PROVEN**.

## Historical CI debug binary — UI-only smoke

**CI is disabled.** The following describes historical artifacts only and is not
a current download/build route. Use the existing manual toolchain for source work;
never restore an Actions workflow to obtain a binary.

A successful automatic Windows Full CI run for `Local` can upload a `localtest-debug-<SHA>` **unsigned debug executable** after all twelve source stages and compile-only pass. This is the **shortest first localtest**: no portable developer toolchain or MSVC/WDK build is needed to *attempt* UI startup. Use [CI Debug Candidate](./ci-debug-candidate.md) to download the exact-SHA artifact into `workspace/downloads/`, extract only the expected three files into a new `workspace/packages/debug-<SHA>/`, verify the manifest SHA-256, and try opening/closing the manager on Windows 10/11 x64 with WebView2. It is not an installer, signed release, Sandboxie distribution or Minecraft gameplay proof. Do **not** bypass OS security warnings. The normal full developer localtest route remains `DEV.cmd bootstrap` → `DEV.cmd doctor` → `DEV.cmd preflight` → `DEV.cmd dev`.

## Quick start — first Windows session

Choose one **short NTFS checkout root on a drive you control**, such as
`D:\M-Minecraft-Emulator`. Install Git for Windows as an OS prerequisite.
Open a normal Windows PowerShell or Command Prompt at that drive's parent
directory, then run:

```powershell
git clone --branch Local --single-branch https://github.com/OpenITStudios/M-Minecraft-Emulator.git
cd M-Minecraft-Emulator
.\DEV.cmd bootstrap
.\DEV.cmd doctor
.\DEV.cmd preflight
```

The `bootstrap` command runs through the built-in Windows PowerShell,
downloads the pinned Node/npm, PowerShell, and Rust prerequisites into
`workspace/downloads/`, and extracts/configures them only beneath
`workspace/toolchains/`. Dependency caches and temporary files belong
to `workspace/cache/` and `workspace/tmp/`. No separate `C:\Source`,
`D:\Source`, global Node installation, or global Rust installation is
required by this developer launcher. **Git and Microsoft C++ Build Tools
remain OS-level prerequisites.** Before opening the UI, inspect the paths
printed by `doctor`. Do not change Windows service/driver or installed
Minecraft/Sandboxie locations to make them appear portable.

For an existing checkout, inspect `git status --short` first, preserve
local edits, then use `git switch Local` and `git pull --ff-only origin
Local`. Never run `git reset --hard` as a shortcut. Execute the same
`bootstrap`, `doctor` and `preflight` sequence.

**Stop at the first failed stage.** If preflight passes, run the developer
UI in a terminal at the same repository root:

```powershell
.\DEV.cmd dev
```

This developer mode does not install/replace the old VMware manager.

## A. Before starting (stop on failures)

1. Use **branch `Local`** at a known commit (prefer latest reviewed `Local`). Confirm the checkout is clean with `git status --short` and record `git rev-parse HEAD`. Source changes or untracked edits can invalidate exact-head verification.
2. Run `DEV.cmd bootstrap` and `DEV.cmd doctor` from the checkout root. The portable setup uses the Node/npm/Rust/PowerShell versions in [toolchain.json](../../toolchain.json), including rustfmt and `x86_64-pc-windows-msvc`, under the checkout's `workspace/toolchains/`. Do **not** install these globally just for this project. Git, [MSVC Desktop development with C++ and WebView2](https://v2.tauri.app/start/prerequisites/) remain external Windows prerequisites for the developer-mode build. The `doctor` check proves tool versions/paths, not linker compatibility or that a Windows desktop opened successfully.
3. Use an [official Minecraft Education Windows Desktop installation](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide), not copied binaries or mixed Microsoft Store + Desktop editions. Have **two distinct properly entitled Education accounts** ready for manual sign-in. Do not place usernames, tokens, passwords, world files or screenshots exposing identities in Git.
4. Use a legitimately licensed [Sandboxie Plus installation](https://sandboxie-plus.com/downloads/) in the applicable environment. Check the vendor's business/educational certificate requirements where applicable. Create **one clean, persistent, enabled** box named exactly `MinecraftClient02` in Sandboxie Plus. Keep **AutoDelete**, **AutoRemove** and credential-bypass/protected-storage settings disabled; do not clone existing account data. The manager does **not** provision the box or its license. Check effective settings in Sandboxie Plus, including templates.

### Source verification (repository root)

```powershell
.\DEV.cmd bootstrap
.\DEV.cmd doctor
.\DEV.cmd preflight
```

For runs that reach pinned Node/toolchain checks, preflight automatically creates
a metadata-only report at `workspace/runs/<UTC>-<shortSHA>-preflight-<suffix>/manifest.json`.
The console prints its location. The report records the complete Git commit
and whether source checks passed or which phase failed. Raw terminal output,
account information and Minecraft worlds are **never captured automatically**.
Earlier Windows/Git/toolchain failures may not create a report. An interrupted
`RUNNING` report is not a PASS. Keep reports local and share only redacted
details; see [Artifact Storage Policy](../system/artifact-storage.md).

**Stop if this command fails.** `DEV.cmd preflight` prints four numbered stages, rejects unsupported Windows hosts, and confirms a clean committed `Local` SHA, checks pinned developer tools and the tracked Cargo lockfile, installs locked desktop npm dependencies, runs root `npm run verify:source`, and rechecks that tracked source/HEAD did not change. It does not open the desktop or package the application. If a stage fails, keep the first error and SHA; do not retry by disabling checks. The source verifier runs repository rules, evidence evaluator tests, frontend Svelte/TypeScript/Vitest/build checks, Rust checks/tests/format. A Git commit or an authored test is *not* evidence of a passing command. A clean verification result does not prove Minecraft playability.

**Do not use** `npm run package:windows`, `npm run build:app`, `DEV.cmd package` or `DEV.cmd g0` to obtain an installer: those legacy VMware packaging paths are deliberately blocked while this native pilot is active.

### Start the developer-mode application

```powershell
.\DEV.cmd dev
```

This runs the desktop development UI, **not** the release installer or a virtual machine. Leave the developer console visible; record only generic error classifications and timings. Do not edit game executables, profiles, or credential storage.

## B. Two-client manual acceptance sequence

Start with a short smoke test: verify that the manager opens, inspect the Client 02 Sandboxie box, open Client 01, then request Client 02 only after its setup is eligible. Observe two independent windows, two distinct licensed sign-ins and playable worlds before attempting multiplayer, restart persistence or sustained performance. **Check status** inspects Client 02 Sandboxie/process candidates; it does not establish Client 01 identity, account authentication or game playability. The mounted UI uses **Check status**, **Set up**, **Browse…**, **Open**, **Switch to** and contextual **Move window** (Left/Right → Apply).

| Step | Action | Expected observation | Decision |
| --- | --- | --- | --- |
| 1 | Open **Instances** and select **Check status** on the Minecraft 02 row (or the toolbar **Check status · 02**) before setup or launch | Sandboxie configuration and PID membership are observed, or an actionable UNKNOWN/refusal is shown | UNKNOWN is **never** evidence of an empty box; check Sandboxie service, box settings and inspection access |
| 2 | On Minecraft 02 choose **Set up** → **Browse…**, select the installed Education Desktop EXE (other EXE filenames/unsafe paths are rejected by the picker) and confirm Sandboxie licensing | **Open** is offered only after a positively observed empty box and complete UI setup | Correct any indicated setting and use **Check status** again; do not bypass rejected or ambiguous setup |
| 3 | On Minecraft 01 select **Open**, then inspect the actual Education window and interact with its menu; after any **Open requested** or error, review **Details** | Native game window responds to input; edition is correct | The manager cannot confirm Host 01 is closed or playable. Before any later launch request, check Windows for an existing window and explicitly clear the UI message; this does not terminate the game |
| 4 | On Minecraft 02 select **Open**, then **Check status** again | An independent candidate window is observed inside `MinecraftClient02` | Sandboxie Start.exe success is only a launch request. If the result is ambiguous, inspect first and **do not auto-retry** |
| 5 | Manually sign in to two distinct licensed Education accounts | Both sessions retain different intended identities without crossing | Stop if authentication cannot be independent; do not access or copy tokens |
| 6 | Open two playable worlds; optionally use **Switch to** and **Details** → **Move window** → Left/Right → Apply for an eligible Client 02 window | Both worlds render, accept separate input and remain responsive | Focus/movement is not proof of gameplay; if either game freezes or window identity changes, mark the result unverified or failed |
| 7 | Test approved multiplayer (only with valid permissions) | Both clients join and interact as intended | Record outcomes without account names, tokens, IP addresses or world contents |
| 8 | Close/reopen Client 02, restart Windows, select **Check status** and repeat the independent-account check | Sessions persist without account crossing or unexpected data deletion | Do not claim two-client acceptance if persistence is missing or UNKNOWN |
| 9 | Optional 30–60-minute sustained-use observation | Note candidate processes, CPU/RAM/GPU/VRAM trends, responsiveness and crashes | Do not invent a RAM/FPS threshold; measure the same PC and Minecraft version |

Host 01 does not use the Virtual 02 **Check status** probe. Its **Open requested** or failed-launch feedback remains until the user opens **Details** and, after checking Windows for an existing Education window, selects **I checked Windows · Clear message**. This is only a local UI acknowledgement that restores the explicit Open control. It does **not** check the running process, confirm gameplay, stop a game or launch another copy automatically. Do not clear the message and retry blindly when the native request outcome remains ambiguous.

The file picker checks only the selected executable's path, expected Education filename and existence. It does **not** prove publisher authenticity: the launch path performs the Microsoft product/signature check independently. An invalid selection clears any previous selected path and licence consent; cancelling the dialog preserves the prior selection. Do not treat “File selected” as proof of signing or gameplay.

When opening **Set up Minecraft 02** from Settings, an inconclusive box/process check or an observed isolated process intentionally hides the executable picker and licence checkbox. Use the explicit **Check status · 02** button in that panel to inspect again; do not assume an unknown result means a stopped game. An error panel also offers a read-only status recheck; do not retry **Open** after an ambiguous launch without verifying current process membership. MIVUBI does not create, delete, enable or reconfigure the official account-bearing `MinecraftClient02` box.

The optional **Switch to** and **Move window** controls apply only to an eligible observed Client 02 window. Windows may reject foreground focus; Alt+Tab is the manual fallback. This manager does not verify Client 01 sign-in, or read any account credentials.

A successful **Move window** request leaves a non-launch `Move requested` notice; the existing **Switch to** action stays available only while the last inspected Client 02 observation still identifies a unique candidate window. The backend always rechecks PID, window and executable identity before another focus/move request. A Client 02 **Open requested** notice persists through a quick check with no matching game process or UNKNOWN membership. A read-only empty-box snapshot is not proof the previous game will never appear. An observed boxed game candidate resolves the notice; otherwise, after a fresh positively observed safe-empty check, open **Details**, inspect actual Windows/Sandboxie windows, and explicitly select **I checked Windows · Clear message** to restore Open. This acknowledgment neither stops Minecraft nor proves it closed; the backend repeats launch admission and enforces its retry gap. An inspection error also retains the launch safeguard. **A failed or timed-out Client 02 Open request is also uncertain**: Minecraft might already have started. The error stays visible with the same pending Open safeguard, which survives empty/UNKNOWN status checks until a boxed game candidate is observed or an operator reviews a fresh safe-empty inspection and explicitly clears the notice. Never infer game readiness from either notice.

## C. Read-only collector (optional diagnostics)

```powershell
powershell.exe -NoProfile -File tooling/acceptance/windows/collect-sandboxie-feasibility.ps1 -Stage INSPECT -BoxName MinecraftClient02
```

The collector emits candidate/process and box-setting information, **not** verified account or game status. Some snapshots may correctly return `UNKNOWN` if Sandboxie processes exit mid-inspection or Windows blocks discovery. Source changes now reconcile every reported box PID against a complete Windows process snapshot; inaccessible or missing IDs do not authorize Client 02 launch.

For a manually confirmed two-game report, follow the existing commands in [Sandboxie two-client feasibility](./sandboxie-two-client-feasibility.md). **Only enter YES/NO for outcomes actually observed.** An operator-reported compatible outcome is still not automated login evidence or four-client acceptance.

## D. Failure classification and next action

| Symptom | Record / action |
| --- | --- |
| `DEV.cmd preflight` fails | Save the first failing command, compiler message, git SHA and tool versions; do not proceed to gameplay. |
| `Check status` fails or membership UNKNOWN | Report Sandboxie version, collector error category, whether the box exists, and whether its service is running; do **not** keep clicking launch. |
| Start Menu / Desktop edition rejected | Inspect official Windows installation/Start app mapping; record language and install-path category, **not** private path or credentials. Do not relax signer verification just to pass. |
| `Open` requested but no second window | Check whether Minecraft focused the first instance, an unsigned/unattributed process appeared, or Sandboxie reported an error; record result as inconclusive/failure as appropriate. |
| Distinct sign-ins fail | Stop the experiment; do not patch the executable, broker, tokens or license checks. |
| Game crashes / input broken / account crossing | Stop scaling; preserve existing worlds/profiles and record the first reproducible failure and hardware summary. |
| Installer request | Existing package path is deliberately blocked. Only developer mode is supported for this pilot. |

### Client 02 idle-crash isolation (one controlled session at a time)

The 14 October 2026 pilot proved two concurrently playable Minecraft Education sessions but subsequently reported Client 02 disappearing while idle. The first cause is **UNKNOWN**: the old two-hour Application event lookup returned no matching 1000/1001/1002 record, and the current manager has no continuous Minecraft monitor or idle-stop action. Do not patch an unproven Sandboxie/graphics cause or report the crash resolved.

Use the **same official Education installation, Windows user, licensed Client 02 account, world/scene, graphics options, idle duration (initially 30 minutes)** and Sandboxie box in these comparisons. Close Minecraft normally between runs, save world progress, and do not reset or recreate the persistent box.

1. **A — Sandboxie without MIVUBI manager:** launch Minecraft 02 using the already installed official Sandboxie Plus interface, then close the MIVUBI manager. Observe whether the boxed game stays responsive while idle. This isolates manager involvement, not Sandboxie from the game.
2. **B — Sandboxie with MIVUBI manager:** with the same closed/empty box and verified setup, request Client 02 from the manager, leave the manager running but do not click repeated checks, and repeat the same idle state. An accepted launcher request is not proof of gameplay.
3. **C — native control:** if A or B fails, run one authorized native Education session outside Sandboxie with the same scene and settings. Compare an equally long idle interval. Do not run this control simultaneously with A/B if doing so changes resource pressure.

For each run record the **start/end time including local timezone, whether the game exited, froze or remained responsive, and whether the Windows desktop/other processes remained alive**. After any exit, review Windows **Reliability Monitor** (`perfmon /rel`) and Event Viewer → Windows Logs → Application near that **exact time**. Check Application Error (1000) and Windows Error Reporting (1001), if present. Retain only the application name, faulting-module name, exception code and event time in the handoff; redact names, paths and account/world information. An absent matching event **does not** prove a clean exit or exclude a crash. Do not enable full crash dumps or persist raw reports without a separate privacy review.

Interpretation is conditional: **A and B fail, C passes** → prioritize Minecraft–Sandboxie compatibility; **only B fails** → investigate manager-launched process/working directory/interaction; **A, B and C fail** → investigate game, Windows or graphics baseline first. Other combinations and missing event data are **INCONCLUSIVE**. These are diagnostic routing rules, not proof of root cause. Only a reproduced failure with matching Windows/module evidence justifies a targeted code or configuration patch. Do not change driver signing, account isolation, templates or live profile data speculatively.

## E. Evidence handoff — no secret data

When reporting the **first** error, send a short manually redacted report instead of raw logs or account details:

```text
COMMIT SHA:
WINDOWS: 10/11 x64
PREFLIGHT: PASS / FAILED / NOT_RUN
FAILED STAGE: 1/4, 2/4, 3/4, 4/4 or NONE
FIRST ERROR: <failing command + redacted error>
DEV UI OPENS: YES / NO / NOT_TESTED
CHECK STATUS (CLIENT 02): OBSERVED / UNKNOWN / ERROR / NOT_TESTED
CLIENT 01 PLAYABLE: YES / NO / NOT_TESTED
CLIENT 02 WINDOW: YES / NO / UNKNOWN / NOT_TESTED
SEPARATE ACCOUNT SESSIONS: YES / NO / UNKNOWN / NOT_TESTED
```

Do not share usernames, access tokens, IP addresses, full filesystem paths containing personal names, private worlds, or screenshots with visible account identities. A source preflight PASS does not establish successful multi-instance gameplay.



Record the exact SHA, Windows version, official Minecraft version, Sandboxie version, app/toolchain versions, elapsed time for Check status and Open requests, observed window count, world responsiveness, distinct-account outcome as **YES/NO/UNKNOWN** (no identities), multiplayer result, persistence result, approximate CPU/RAM/GPU measurements, reproducible error message, and whether the observation is **machine-observed** or **operator-reported**.

**Proof ceiling:** Source checks at a different commit do not verify this revision. Until the exact-SHA preflight and Windows two-client tests actually execute, `SOURCE_EXECUTION`, `TARGET_WINDOWS`, `MINECRAFT_RUNTIME` and four-client acceptance remain **NOT_EXECUTED** for that revision. Review [REA-aligned audit](./rea-source-audit-2026-10-13.md) for still-open source risks. This document does not add a second state machine, test gate or service.
