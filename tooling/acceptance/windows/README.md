# Windows Acceptance Tooling

## Independent native AppContainer proof (synthetic only)

### Rust decoder for the existing synthetic child record

The native fixture writes `Observation` records containing exactly two
little-endian Windows DWORDs (`version=1`, bitmask flags). A compile-time
assertion fixes this size at eight bytes; the pure Rust decoder in
`engine/runtime-core/src/independent_engine/fixture_wire.rs` rejects
truncated, oversized, future-version or contradictory records. Decoded
`UnboundFixturePaths` contains three sanitized path statuses and a
first-run/reopen phase only. Pair decoding prevents mixing initial and
reopened fixture phases.

The **binary record alone is not authenticated IPC**. It contains no
PID, FILETIME, OS process handle, raw profile path, SID or account. The
existing fixture parent still owns and verifies its children. A future
Rust observer must bind file provenance to that known child handle
before converting anonymous fixture path observations into diagnostic
process-specific data. This decoder is never production readiness.

### Self-observed A/B paths — schema 2

The native fixture now records Local AppData Known Folder, APPDATA environment,
and Roaming Known Folder from within **each** synthetic AppContainer child.
The parent verifies token identity and emits separate sanitized
`processPaths` for fixtures A and B. No raw SID, PID, creation time, account
or path appears in JSON. The Node reviewer rejects mismatches between those
child observations and the older aggregate fields. The child labels are not
a verified production `ProcessIdentity` (`processIdentityBound=false`).
A path under the synthetic profile is NOT proof of real Minecraft data or
MSAL broker isolation. Positive fixture reports now use schema 2; the
historical negative-control failure keeps its existing schema-1 shape.

### Persistent private roaming-like storage proof (synthetic, opt-in)

The same AppContainer fixture now writes a canary under a freshly created
`synthetic-roaming` directory **inside each synthetic AppContainer profile**.
After both children exit normally, two new children with their original
AppContainer identities must read their own previously written canaries while
being denied access to the other profile. This checks storage persistence across
**process restarts**, not Windows reboots, Minecraft profiles or Microsoft login.

The report fields `syntheticPrivateRoamingWritten`,
`syntheticPrivateRoamingPeerDenied`, and
`syntheticPrivateRoamingPersistsAfterRestart` are only emitted as true
after actual fixture-side checks. `minecraftRoamingCompatibility` always
remains `NOT_EXECUTED`. The existing independent `APPDATA` environment
and `FOLDERID_RoamingAppData` checks are path observations only and may
still point outside the container. No synthetic test writes to those
host-roaming paths; a synthetic custom directory is **not** a redirection
of the Win32 Known Folder API. Do not use these booleans to unblock a provider.



The offline `evaluate-appcontainer-isolation.mjs` reviewer classifies a
positive, sanitized fixture JSON report without treating it as verified
Windows acceptance. `isolatedLaunchAllowed` remains false even if both
roaming paths appear private. It rejects unexpected fields (including raw
profile paths/SIDs), incomplete reports, malformed booleans and false gameplay
claims. Its Node tests are included in `npm run test:native-feasibility`.

After an explicit Windows fixture run, inspect the generated JSON using:

```sh
node tooling/acceptance/evaluate-appcontainer-isolation.mjs workspace/appcontainer-probe-<id>/result.json
```

This review is offline and never reads or writes Microsoft accounts, Minecraft
worlds or Windows profiles. It does not replace direct runtime observation.

From the repository root, explicitly run:

```powershell
./tooling/acceptance/windows/test-appcontainer-isolation.ps1 -VerifyNegativeControl
```

Requires installed x64 MSVC Build Tools; downloads nothing. The script compiles
`appcontainer-probe.cpp` with `/W4 /WX`, then launches only copies of that same
fixture. It uses documented Windows AppContainer APIs, no Sandboxie code, DLL,
service calls, VM or game launch. It creates two uniquely named temporary
`Mivubi.Probe.*` AppContainer profiles and gives their SIDs read/execute rights
only on its new fixture directory and binary. No installed file, existing profile,
ancestor directory or account data is reconfigured. Network capabilities are absent.

The positive case checks distinct actual AppContainer tokens, concurrent survival,
the same synthetic mutex name created separately, readable own canaries, denied
peer/host canaries, normal exit of one process while the other survives, and
successful removal of only the two profiles it just created. Existing profile
collisions abort; they are never adopted or deleted. The optional negative control
deliberately reuses token A and must fail at `actual-distinct-container-tokens`.
Its expected FAIL is not a product failure; another failure stage is an error.

Results are sanitized booleans below ignored `workspace/appcontainer-probe-*`.
Environment/known-folder path comparisons do not read profile contents and do
not imply auth isolation. On the observed host, local AppData was redirected,
but roaming environment and known-folder paths were not under the container.
Those optional observations are reported independently from the core fixture PASS.
See [independent engine findings](../../../docs/system/independent-isolation.md).

Each fixture has a 20-second lifetime and the script bounds its parent to 30
seconds. Exceptional cleanup may terminate only exact self-created fixture handles.
Abrupt parent termination can leave synthetic profiles; the inspection-only UTF-16
moniker receipt is written before creation and does not authorize deleting a
profile. Never clean profiles by wildcard. No automatic retry, elevation, global
ACL grant or existing-service removal is performed. Existing legacy services
remain installed; independence from an installed legacy driver on a clean host
was NOT_EXECUTED. Graphics, input, COM/authentication, Minecraft singleton behavior,
account separation, multiplayer and four-client acceptance are NOT_EXECUTED.

## Bounded process lifetime diagnosis

`watch-workspace-process.ps1 -ProcessId <observed PID> -CreatedFileTime '<observed FILETIME>' -DurationMilliseconds 60000`
pins the original process handle and refuses a different creation time. Obtain
the identity from the existing provider observation, not a window title. The
collector waits at most 60 seconds and reports UTC start/end, elapsed time,
whether that process exited and its exit code if available. It never closes a
process, sends game input, reads credentials/worlds, or enumerates accounts.
The JSON omits PID, image path and window title. Keep local collection output
under ignored `workspace/`; record only sanitized results in the canonical report.

A timeout means no exit was observed during that interval. It does not mean the
window rendered, accepted input, or passed gameplay; exit code alone does not
identify the caller/cause of a closure. This collector grants no provider or
mutation authority. `npm run test:native-feasibility` includes its Windows fixture
test for stale identity rejection, bounded timeout and known exit code.

## Unified workspace feasibility (synthetic windows only)

Optional UI regression, from repository root with the frontend server on
127.0.0.1:1421: `node tooling/acceptance/workspace-ui-regression.mjs`.
It requires an available Playwright installation and Edge; optionally set
`MIVUBI_PLAYWRIGHT_MODULE` to that installation's absolute module directory.
It mocks Tauri, creates no game or engine processes, and writes only synthetic
UI screenshots below ignored `workspace/`. It covers resize errors retaining
recovery controls, native error events and three viewport sizes. Not gameplay proof.

The approved composition now has a developer product implementation in
`apps/desktop/src-tauri/src/engine/workspace_surface.rs`. Its opt-in regression
uses the actual debug executable and real synthetic windows. Build the frontend
and debug desktop first, then run:

```powershell
cargo build --locked --manifest-path apps/desktop/src-tauri/Cargo.toml
cargo test --locked --manifest-path apps/desktop/src-tauri/Cargo.toml actual_surface_helper_restores_on_release_pipe_loss_and_helper_crash -- --ignored --nocapture
```

The regression compiles the existing C# fixture and exercises the production
controller/helper for normal release, manager-pipe loss and helper termination,
both normal and maximized. It checks exact original style/rectangle, native
unowned HWND, stable composed bounds, rejected stale creation time/foreign
session and preservation of the current surface when replacement preparation
fails. It never launches Minecraft or uses a real profile; only its self-created
synthetic process is force-cleaned. It is ignored in the portable default suite.
Actual gameplay/input, simultaneous coordinator/helper failure, mixed-monitor
DPI and live previews require separate acceptance.

Run `powershell.exe -NoProfile -NonInteractive -File tooling/acceptance/windows/test-workspace-host.ps1`
from the repository. The opt-in probe compiles an x64 GUI fixture with the existing
Windows .NET Framework compiler, starts only its own synthetic client, checks
PID/start time/window ownership and equal PMv2 contexts, reparents the client,
then verifies original parent/style/geometry restoration and normal close.
It also injects a failure immediately after attachment and verifies restoration.
The third default case creates separate synthetic UI and game processes below
a surviving presentation-owner window. It forcibly terminates only the fake UI,
verifies the game HWND/process survives, then restores parent/style/geometry and
requests normal close. This does not test failure of the presentation owner.

To reproduce the rejected direct-parent design, append `-IncludeUnsafeDirectCrash`.
This additional case kills its own synthetic parent and attempts recovery from
the independent coordinator. On the tested Windows host the client HWND is
destroyed: JSON reports **FAIL** and the script exits nonzero. Preserve this
failure; it is evidence against the candidate design, not a passing acceptance
case. It never targets an existing game. Native window loss can leave the fake
client's message loop alive, so this case separately rechecks its exact compiled
fixture path and start time before cleaning up that synthetic process only.

`-IndependentSurfaceProposal` adds a different, explicitly proposed presentation
strategy: the synthetic client keeps no parent/owner, retains native input, and
is styled/positioned inside the host's screen rectangle. After terminating that
host, the independent coordinator verifies the same client HWND survives,
restores style/geometry, and requests normal close (exit 0). This is **not
embedding** and does not alter the product. It does not prove actual Minecraft
input, z-order, dialogs, taskbar behavior, DPI, preview capture or simultaneous
coordinator failure. Approval of the design deviation is still pending.
The wrapper never accepts an external process/window. It retains evidence under
a unique ignored `workspace/workspace-host-*` directory; a stuck synthetic host
alone may be terminated by its creator. A separate 20-second thread timer exits
the synthetic child even if destruction of its HWND strands the Forms timer.
It does not modify or launch Minecraft, services, profiles or accounts.

This is bounded feasibility evidence, not the product hosting implementation.
Relative mouse, rendering, mixed DPI, accessibility, presentation-owner crash, game dialogs,
capture previews and actual Tauri surface composition need separate proof.
See the [workspace contract](../../../docs/product/unified-workspace.md).

**Selected active experiment:** [Client 01 native + Client 02 Sandboxie Plus](../../../docs/validation/sandboxie-two-client-feasibility.md).
Use `collect-sandboxie-feasibility.ps1 -Stage INSPECT -BoxName MinecraftClient02`
for an explicitly requested observation; the desktop uses its read-only
`-SummaryOnly` option. PID/window counts cannot prove licensed login,
Minecraft rendering or multiplayer.

## PresentMon frame-pacing review (offline, no game hooks)

For smoother **real gameplay** and future Eco-mode decisions, use a separate
PresentMon CSV recording from the actual Windows game session. MIVUBI does
**not** launch PresentMon, inject a capture DLL, modify game graphics, change
Minecraft priority or install any monitor. The offline reviewer reads only
an operator-selected existing CSV, using the target Minecraft process ID
from that capture:

```powershell
npm run frametime:review -- --csv "C:\captures\PresentMon.csv" --process-id 1234
```

The CSV must include `Application`, `ProcessID`,
`SwapChainAddress`, and `MsBetweenPresents` (or the older
`msBetweenPresents` case). These are actual **inter-present intervals**
rather than displayed frame intervals or independently confirmed FPS.
PresentMon CSV layouts have changed, including around the v2.3.1 release:
the reviewer deliberately refuses `FrameTime` alone rather than treating
a different metric as interchangeable.

The input is bounded to 32 MiB, 250,000 CSV rows, and one **Minecraft** PID
with exactly one swap chain. Mixed swap chains, PID attribution to another
application, unsafe CSV, insufficient samples or substantial missing values
are rejected rather than silently averaged. The output is a sanitized JSON
summary containing interval count, missing-data coverage, mean/p50/p95/p99,
and interval counts above 33.333 ms and 50 ms. **No raw PID, process name,
swap-chain address, file path, login, world or frame-by-frame log is emitted.**
The original file remains unchanged.

A CSV alone is not authenticated proof of the actual Windows machine or
active Minecraft account. Present intervals may differ from frames actually
displayed (for example, dropped/repeated frames or frame generation).
A single selected swap chain also does not measure other Minecraft helpers,
GPU power/VRAM, actual on-screen FPS, game input latency or four-virtual
stability. For a fair baseline, compare identical game versions, worlds,
graphic settings, loading state and operator-verified captured process
identities. Use p95/p99 regressions and long-frame counts as **diagnostic
clues**, not an automatic optimization PASS. In particular, an idle crash
cannot be diagnosed just from a CSV with missing frames.

Source/format references:
[PresentMon console output](https://github.com/GameTechDev/PresentMon/blob/main/README-ConsoleApplication.md)
and [upstream CSV field layout](https://github.com/GameTechDev/PresentMon/blob/main/PresentMon/CsvOutput.cpp).
`npm run test:presentmon` exercises the deterministic offline CSV and CLI
fixtures. The test runner does not validate live Windows/ETW capture,
Minecraft rendering or account isolation.

## Active two-client process performance sampling (manual, read-only)

`collect-native-performance.ps1` is an **optional on-demand** Windows
sampler for the current native + Sandboxie experiment. It does not launch games,
inspect accounts/worlds, certify which process belongs to Minecraft, or test FPS.
The operator must first identify two existing game PIDs via the approved
Sandboxie inspection and verify them in the actual Windows session. Example:

```powershell
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 `
  -NativePid <NATIVE_PID> -IsolatedPid <BOXED_PID> `
  -DurationSeconds 3600 -SampleIntervalSeconds 5
```

### Four-client CPU/RAM capture

After independently verifying four existing game identities in an authorized
Windows session, extend the same collector with both additional PID parameters:

```powershell
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 `
  -NativePid <CLIENT01_PID> -IsolatedPid <CLIENT02_PID> `
  -Client03Pid <CLIENT03_PID> -Client04Pid <CLIENT04_PID> `
  -DurationSeconds 3600 -SampleIntervalSeconds 5
npm run native-performance:review -- <CAPTURE_JSON>
```

Both extra parameters are required together and all four PIDs must be distinct.
The capture uses schema 3 and retains separate CPU/RAM metrics for slots 01–04;
each identity is pinned to its initial creation time. A missing/inaccessible target
stays UNKNOWN for the rest of the capture. Output includes no PID/account data.
The old two-client commands keep schemas 1/2 unchanged. Schema 3 supports individual
review only; comparison with another capture and the two-client `-GpuSnapshot`
option are rejected rather than producing incomplete four-client evidence.

This measures four manually selected processes, without provider attribution,
process-tree totals, FPS, account isolation or multiplayer certification.
Actual 30-second collector checks with disposable Node processes passed in
PowerShell 7 and Windows PowerShell 5.1, including permanent identity loss after
one process exited. Those checks are **not Minecraft or a 60-minute acceptance run**.

### Optional GPU counter snapshot (diagnostic only)

The default command above remains lightweight, uses **schema 1**, and does
**not** query any GPU performance counters. On an existing authorized Windows
session, opt in to **one GPU counter snapshot at the END of the timed CPU/RAM
capture**:

```powershell
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 `
  -NativePid <NATIVE_PID> -IsolatedPid <BOXED_PID> `
  -DurationSeconds 60 -SampleIntervalSeconds 5 -GpuSnapshot
```

This produces **schema 2** (same CPU/RAM samples plus `gpuDiagnostic`). The
optional snapshot requests exactly two rate-counter samples once using
`GPU Engine(*): Utilization Percentage` and `GPU Process Memory(*):
Dedicated Usage`, and reads the last sample. It reports each selected
process's largest observed **3D engine instance percentage** and largest
**dedicated-memory counter instance** in MB. These are **not** total GPU
utilization or actual aggregate VRAM. Windows performance-counter names may
be unavailable/localized, GPU drivers may not expose the data, and Microsoft
documents cases of stale/incorrect GPU Process Memory values. In any such
case the affected result is `UNKNOWN` with nulls, not a fabricated 0.
Original PID creation times are rechecked to reduce PID-reuse attribution;
the JSON contains no PIDs, device IDs, paths, user names or account data.

All GPU output is explicitly an **unverified hint**, with
`gpuPerformanceVerified=false`, `frameTimingVerified=false`, and no
performance verdict. The reviewer still lists `GPU_VRAM` and `FRAME_TIMES`
among missing acceptance evidence. It does not measure FPS, presentation
latency, full process trees or GPU power. For real frame timing use a separate
trusted frame-present capture where supported; test comparable worlds and
graphics settings and do not treat GPU counters alone as an Eco-mode PASS.
The `Get-Counter` diagnostic may briefly add CPU load after collection;
do not enable it by default or run it at high frequency.

Technical references: [Microsoft performance-counter sampling guidance](https://learn.microsoft.com/en-us/windows/win32/perfctrs/about-performance-counters)
and [Microsoft known incorrect GPU Process Memory values](https://learn.microsoft.com/en-us/troubleshoot/windows-client/performance/gpu-process-memory-counters-report-wrong-value).

### Capture stability and report compatibility

The optional GPU query is **skipped entirely** if either selected game
process became missing or inaccessible during the CPU/RAM capture. In that
case both GPU hints stay `UNKNOWN`/null and no new GPU workload is added.
This does not mislabel a missing game PID as a crash or accidentally sample
a replacement process.

The Windows PowerShell 5.1 collector now writes report JSON as UTF-8
**without a byte-order mark**. It writes to a new GUID-named temporary file
in the output directory, closes it, and moves the completed file to a new
final destination; an existing report is never replaced. Failed runs only
clean up their own temporary file. This prevents incomplete diagnostic
captures from appearing as completed reports after ordinary write failures.
A sudden process/OS termination can leave a temporary file; it must never
be treated as a valid final report.

The Node reviewer also accepts legacy PowerShell 5.1 captures containing a
single UTF-8 BOM, while applying the same exact JSON schema validation.
The compatibility fix is for evidence files **only**; it cannot change
Minecraft, Sandboxie boxes, running drivers or account data.

To review the generated JSON without declaring gameplay PASS:

```powershell
npm run native-performance:review -- "C:\path\to\native-process-performance-....json"
```

The same reviewer can compare an existing baseline and candidate capture
**offline**, without launching another sampler or additional background agent:

```powershell
npm run native-performance:review -- "C:\\path\\baseline.json" "C:\\path\\candidate.json"
```

The reviewer now reports observation coverage, the first missing sample
time, mean/peak selected-process CPU and mean/peak working set. A process
reported UNKNOWN could have exited, become inaccessible or been replaced;
this is **not** automatic proof of a crash. A pair of reports must match their
CPU-count, requested duration, sampling interval, sample count and observed
duration, with both selected processes continuously readable, before the
reviewer emits *numeric differences* (candidate minus baseline). Even then
the comparison is **descriptive only**: matching CPU counts do not establish
the same physical PC or workload. It cannot infer FPS, GPU demand, full
sandbox descendant-process cost or confirmed Minecraft optimization.

`npm run test:native-performance` checks schema-1/schema-2 GPU hint handling,
privacy, missing-data, numeric comparison and false-PASS constraints.
Authored tests do not prove that the Windows sampler has actually executed.

The command is **not executable until real client processes are available**.
It captures relative elapsed times, per-process working set and process CPU
percentage normalized to the logical CPU count, into a timestamped JSON file
under the user's local TEMP folder. Output intentionally excludes PIDs,
process paths, user/account names, world data and credentials. An inaccessible,
terminated or replaced PID becomes UNKNOWN for the remainder of that capture.
No thresholds are declared; `automaticVerdict=NOT_PROVIDED` and the result is
**PROCESS_TELEMETRY_ONLY**. Do not upload raw private device captures to Git.

For the actual 60-minute acceptance review, pair the process sample with
independently recorded PresentMon frametimes (if supported by the real renderer),
GPU/VRAM telemetry, user-observed input responsiveness, independent licensed
worlds and multiplayer evidence. Compare single versus two-client runs on the
same PC, resolution and game settings. No performance sample alone promotes
G0–G4 or proves account separation. Windows/PowerShell execution of the sampler
is currently **NOT_EXECUTED**.

## Historical VMware-only G0–G4 campaign (paused)

The stages and procedures below document the retained Windows-guest
acceptance implementation. They are **not** the active startup/installation
workflow and do not authorize changing account-bearing guests. The old NSIS
release is deliberately blocked. Do not create another test gate scheme.

This campaign is evidence collection, not simulation. A stage stays `NOT_RUN` until its target-machine action has actually been performed. **The only five product-test progress gates are G0–G4 in [Acceptance](../../../docs/validation/acceptance.md)**; the existing stage IDs and recorder schema stay unchanged. Stage records are supporting evidence, not twelve independent release milestones.

## Stages

| Stage | Supports gate | Purpose | Evidence owner |
| --- | --- | --- | --- |
| CLIENT_FEASIBILITY | G1 | **Before production VM setup:** prove one disposable Windows guest on the candidate provider renders Minecraft Education; capture actual OpenGL and D3D11 hardware when using GPU-PV | operator + read-only guest collector |
| HOST | G1 (baseline) | Record Windows, VMware, CPU, memory, GPU, hypervisor/VBS and Native Minecraft facts | collector |
| SINGLE_VM | G2 | Prove one Virtual lifecycle on the target provider | operator + collector |
| GUEST | G2 | Prove VMware Tools, Guest Agent and interactive launcher progression | collector |
| MINECRAFT | G2 | Prove Minecraft launches in the intended interactive session without duplicate launch | operator + collector |
| ACCOUNT | G2 | Prove one licensed account can sign in and persist through the intended recovery workflow | operator |
| THREE_VMS | G2 | Prove Virtual-01/02/03 coexist with unique identities | operator + collector |
| FOUR_MINECRAFT | G3 | Prove Native + three Virtual Minecraft clients coexist and remain interactive | operator + collector |
| FOUR_ACCOUNTS | G3 | Prove four distinct licensed sessions remain isolated | operator |
| MULTIPLAYER | G4 | Prove required Native↔Virtual and Virtual↔Virtual join paths | operator |
| RECOVERY | G4 | Prove stop/start, suspend/resume, reset and host reboot behavior | operator + collector |
| SOAK | G4 (supporting) | Record long-run stability and repeated lifecycle cycles | operator + collector |

## Historical Windows guest route (not currently selected)

The product target is **one Windows-native Minecraft Education desktop client plus three isolated Windows guest clients**. The installed production code remains VMware; the approved App Sandbox GPU-PV direction is limited to a disposable pilot until G1 proof. Do not run Android, BlueStacks or second-native-instance probes. Start with a disposable scratch Windows guest, before production Base preparation. Confirm **VM opens → Minecraft Education installs and signs in → a world is playable and responsive**.

The [Product Goal](../../../docs/product/four-concurrent-accounts.md) and [One Guest Feasibility Gate](../../../docs/validation/client-feasibility.md) specify evidence and stop criteria. The guest collector cannot observe account sign-in or game rendering directly: its operator declarations are **not automatically verified**. Do not record PASS until actual operator/game evidence has been reviewed.

## Mandatory one-Windows-guest proof

Before creating three account-bearing Windows VMware clones, record these **in order** on a disposable scratch Windows guest:

1. **VM opens** — guest boots to a responsive Windows desktop; VMware Tools and input function.
2. **Desktop Education installs and signs in** — choose exactly one official Windows installer type, launch the real Minecraft Education client, sign in manually with a distinct legitimately licensed organization account.
3. **World plays responsively** — load a Minecraft world, test camera/keyboard/mouse and 3D rendering for an initial 10 observed minutes without an obvious crash, freeze or severe input lag.

The `collect-client-feasibility.ps1` `INTERACTIVE` stage now asks for explicit operator observations (`-VmDesktopOpened`, `-EducationInstalled`, `-EducationSignedIn`, `-CrashOrFreezeObserved`, `-ObservedMinutes`) alongside the previous rendering flags. These are claims to review, not machine-certified PASS. Never submit identities, license keys or credentials. Stop escalation if any gate fails.

## Early client-graphics feasibility pilot

Use `collect-client-feasibility.ps1` **inside one interactive Windows scratch guest** with `-Provider VMWARE` (default) or `-Provider APP_SANDBOX_GPU_PV`. It is read-only except its temporary report and does not require Guest Agent, account-name capture or a configured three-VM application. Collect `ENVIRONMENT`, then manually open Minecraft Education, observe a usable world and collect `INTERACTIVE` with explicitly operator-attributed observations.

For GPU-PV, the collector measures a **D3D11 hardware FL11_0** device and WGL OpenGL version/vendor/renderer and the evaluator refuses a reported-playable GPU-PV outcome when hardware evidence is missing. Do not substitute VMware Tools, host GPU names or App Sandbox flags for guest graphics observations.

Review the JSON with `../evaluate-client-feasibility.mjs`. The reviewer never certifies official client support; it reports `OPERATOR_REPORTED_PLAYABLE`, `OPERATOR_REPORTED_FAILURE` or `INCONCLUSIVE`, never a synthetic PASS. The existing `new-stage-result.ps1 -Stage CLIENT_FEASIBILITY` records a **separately reviewed** decision with an actual evidence reference.

Canonical guide: [Client Feasibility](../../../docs/validation/client-feasibility.md). A scratch-guest failure stops further cloning and account setup until graphics/driver/VM compatibility is understood.

G0 has no Windows evidence-recorder stage: it is derived from executed source, package, payload and disposable installer-safety checks in [G0 Readiness](../../../docs/validation/pre-local-test-readiness.md). `HOST` establishes a baseline; it does not prove G1 on its own. `SOAK` is supporting evidence and cannot replace G4's recovery and multiplayer observations.

## Result vocabulary

**Safety:** `collect-acceptance.ps1` is observation-only by default. Optional `-VerifyIdentities` may update identity evidence and is blocked unless all three Virtual clients are provisioned and RUNNING. Every CLI response must have a valid schema-1 success envelope; malformed responses fail closed.

Use only:

- `NOT_RUN` — target-machine action has not been performed.
- `OBSERVED` — evidence was collected, but the stage does not have enough evidence for pass/fail.
- `PASS` — every required assertion for that stage was actually exercised and supported by evidence.
- `FAIL` — at least one exercised required assertion failed.
- `BLOCKED` — the stage could not be exercised because an earlier dependency failed.

Do not infer a **gate PASS** from a stage result, source tests, CI, documentation, configured NAT, process existence, or a successful collector run. The recorder's `NOT_RUN` means `NOT_EXECUTED` in the human-facing five-gate summary; partially observed stages do not promote their gate.

## Campaign order

Use the stages as **one campaign grouped by G1–G4**, not as a new test activity after each code patch. Stop escalation on a real failed required outcome; diagnose the first wrong owner and revisit affected/dependent gate outcomes. Do not re-run the entire campaign for source-only edits or repeatedly exercise already evidenced scenarios without a relevant change.

Use `collect-acceptance.ps1` before and after meaningful runtime transitions. The collector must support HOST/baseline **before any VM is provisioned**: `resourcesCollected=false` and null resource measurements explicitly mean *not yet available*, not zero or success. Only after all three Virtual clients are provisioned may the optional `resources 3` command be queried. No unobserved multiplayer or provisioning-ready flags may be created. Preserve each output as evidence for the campaign rather than overwriting previous captures.


## Performance evidence

Developer workspace probes live in `../probe-live-workspace.mjs`. Supply
`MIVUBI_PLAYWRIGHT_MODULE` if Playwright is not resolvable, and use a separately
started debug manager with a loopback-only CDP endpoint on port 9223. Never enable
that endpoint in saved settings or release configuration. `--live` may launch
one native game and exercises Settings suspend/resume. `--live --four` requires
four already running, uniquely attributed responsive game windows and performs
12 tab selections without launching, signing in, sending game input or opening
worlds. It verifies identity/responsiveness, not rendered gameplay or taskbar
appearance. Close the debug manager normally after restoring windows, then run
normally to remove the endpoint. Canonical results belong in
`docs/validation/native-pilot-localtest.md`.

Use `performance-contract.json` during target-machine performance acceptance. Record measurements as observations, not conclusions.

The purpose is to calibrate the single automatic performance policy with real Native + Virtual-01/02/03 evidence. Do not change configured RAM, vCPU, GPU policy or startup thresholds from an incomplete run. Missing measurements remain unknown; they are never treated as zero or PASS.

At minimum, capture the host baseline, each Virtual VM start duration, Guest Agent readiness duration, Minecraft readiness duration, observed host working set, 3D-graphics observation, resolution, peak host CPU during sequential startup, concurrent Minecraft count and operator responsiveness.

Performance acceptance must distinguish:
- capacity: what the hardware should reasonably support;
- pressure: whether another client should start now;
- experience: whether Native and the running Virtual clients remain usable together.

The benchmark harness does not automatically declare a performance PASS. Threshold calibration is a separate evidence review after representative target machines have been measured.
