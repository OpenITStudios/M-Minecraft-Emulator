---
id: document.validation.sandboxie-two-client-feasibility
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: HISTORICAL
---

# Sandboxie Plus — two-client application-isolation experiment (historical)

> **HISTORICAL / RETIRED PROVIDER.** This dated two-client experiment
> is not the current production route or an instruction to run Sandboxie.
> Lazy Tester uses the five-profile native controller. Use the
> [current validation router](README.md) and
> [single handoff](../../planning/development.md) for current proof.

**Work:** emulator/sandboxie-client02. This is the **single selected Windows
isolation experiment**: one normal Minecraft Education Desktop game and one
launch requested inside the persistent Sandboxie Plus `MinecraftClient02` box.
It is not a proven multi-account product or a production provider. Do not
stop/delete VMware guests, modify sign-in data or duplicate game binaries.

## Why this small experiment exists

The target is one official Minecraft Education Windows Desktop installation
and four concurrently playable accounts, with **no full Windows guest VMs if
possible**. Development is currently restricted to **two Windows Desktop
clients**, one host native and one inside the preconfigured `MinecraftClient02` box.
No Android emulator, controller, multi-provider engine or 4-client scheduler
is developed in this experiment.

Sandboxie Plus provides:
- SbieIni.exe query SECTION SETTING to read box configuration;
- Start.exe /box:BOX /listpids to enumerate PIDs reported inside a box.
  Start.exe is a GUI subsystem program; redirected stdout may not work
  on every configuration. No result means UNKNOWN, not zero processes.
  The helper's redirected stdout is consumed while it runs, with bounded
  memory and a total 5-second helper deadline, rather than waiting for exit
  while the output pipe fills. The CLI's two separately parsed PID snapshots
  must agree (including a genuinely empty box), or the collector tries
  the existing signed read-only DLL fallback. All reported box PIDs must
  also be present in the Windows process inventory; otherwise membership
  is UNKNOWN. These observations are not atomic proof against concurrent
  processes launching after inspection. The count includes all sandbox
  processes, not just game candidates: the collector accepts up to 256
  listed PIDs with a 4096-character output limit, while the Minecraft
  candidate list remains limited to 16.
  When that CLI output is missing or invalid, the same short-lived collector
  may fall back to the verified installed `SbieDll.dll`
  `SbieApi_EnumProcessEx` read-only export. The fallback must confirm
  stable count and PIDs across separate bounded reads; otherwise membership
  stays UNKNOWN and Client 02 launch is rejected. No game data, identities
  or sign-in tokens are accessed.

  The collector also requires every Sandboxie-reported PID to be present in
  the current Windows process snapshot **with a readable process name and
  integer SessionId** that matches the current interactive session. Missing,
  unreadable, duplicate, foreign-session or unclassifiable boxed process
  identity downgrades membership to `UNKNOWN` even if Sandboxie reported
  the PID. The source caches only the validated boxed PID name and session
  within this collector invocation to avoid re-reading volatile name/session
  getters when creating the Minecraft candidate list. Optional window-handle access failures retain
  a name-matched candidate without a window; they do **not** make the box
  empty. A Windows/Sandboxie enumeration remains a time-bounded observation,
  not an atomic guarantee against another launch or exit.

These calls remain **read-only** in the collector. The active desktop adds
one separate, explicitly requested launch action via the official
`Start.exe /box:MinecraftClient02 <installed Education EXE>` command, without
creating, deleting, resetting or configuring Sandboxie boxes. The app never
copies Microsoft account data or modifies Minecraft executable files.

Upstream references:
- https://sandboxie-plus.github.io/sandboxie-docs/Content/SbieIniCommandLine/
- https://sandboxie-plus.github.io/sandboxie-docs/Content/StartCommandLine/
- https://github.com/sandboxie-plus/Sandboxie/blob/43b629be6c17628e6298befccfec279774895ba5/Sandboxie/core/dll/sbieapi.c
- https://github.com/sandboxie-plus/Sandboxie/blob/43b629be6c17628e6298befccfec279774895ba5/Sandboxie/core/dll/sbieapi.h
- https://github.com/sandboxie-plus/Sandboxie/blob/43b629be6c17628e6298befccfec279774895ba5/Sandboxie/core/drv/process_api.c
- https://github.com/sandboxie-plus/Sandboxie/blob/43b629be6c17628e6298befccfec279774895ba5/Sandboxie/apps/start/start.cpp
- https://sandboxie-plus.github.io/sandboxie-docs/Content/SandboxieIni/
- https://sandboxie-plus.github.io/sandboxie-docs/Content/SandboxRemoval.html
- https://github.com/sandboxie-plus/Sandboxie/blob/master/Sandboxie/install/SbieSettings.ini
- https://github.com/sandboxie-plus/sandboxie-docs/blob/main/docs/PlusContent/supporter-certificate.md
- https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide

## Fallback trust and proof boundary

The pinned upstream C implementation accepts five arguments:
`(box_name, all_sessions, which_session, boxed_pids, boxed_count)`.
The current driver uses a separately supplied count pointer and may return
a **truncated but successful** PID list when the buffer fills. The older
documentation describes a legacy first-array-element count, which **must
not** be used for this version's five-argument binding. The collector
supplies an allocated WCHAR[34] box-name buffer, current-session mode,
a 256-process acceptance bound and buffer headroom. It independently checks
two sorted PID inventories and a final count before accepting the result.
Even a stable snapshot can race with an out-of-band process launch and cannot
prove Microsoft WAM isolation or Minecraft gameplay.

Fallback is requested only after the CLI read has failed. The collector
requires the SbieDll signature to match its trusted Start.exe, uses the
vendor export in a *short-lived PowerShell process*, and returns UNKNOWN
if validation/ABI/loading/enumeration fails. Loading the vendor DLL on a
real Windows machine, its actual resource cost and crash behavior are
**NOT_EXECUTED**. The app's 30-second collector deadline bounds the
parent wait; it does not establish that vendor DLL initialization is safe
for every Sandboxie version. The persistent Tauri manager never loads it.
The separate ImageProof-only per-PID QueryProcess attribution remains
manual-only.

## Canonical, generic naming

- Manager: **Minecraft Instance Manager** (developer UI only).
- Native process: **Client 01**.
- Isolated process: **Client 02**, fixed Sandboxie box `MinecraftClient02`.
- Pilot state: `%LOCALAPPDATA%\MinecraftInstanceManager\NativePilot`.
- Reports: `%TEMP%\MinecraftInstanceManager\SandboxieFeasibility`.

An older developer experiment may have created a differently named box.
The manager does **not** rename, erase, or import data from it. Prepare a
new, clean, persistent `MinecraftClient02` box manually in Sandboxie Plus;
never clone saved Microsoft login data into the new box.

## Performance without process-count shortcuts

The active source includes a read-only, opt-in process sampler at
[`tooling/acceptance/windows/collect-native-performance.ps1`](../../Development/tooling/acceptance/windows/collect-native-performance.ps1).
It accepts two **manually verified** existing PIDs and records bounded
CPU/working-set observations for up to two hours. Missing/reused PIDs stay
UNKNOWN, not zero; the report includes no credentials, process IDs or world
content. Because operator-selected PIDs are not independently attested by
this sampler, results are diagnostic only. Use separate PresentMon/Windows
Performance Recorder observation and real player input to judge smoothness,
including frame-time spikes and recovery. A full 60-minute run and all
real-device performance proof remain **NOT_EXECUTED**.

## Single desktop experiment — Client 01 + Client 02

The native + Sandboxie pilot is tested from a **clean `Local` checkout
of one exact SHA**, on Windows x64 with the pinned development toolchain:

```powershell
.\DEV.cmd preflight  # source checks; no Minecraft, sandbox or package mutation
.\DEV.cmd dev        # only after preflight succeeds; launch Tauri dev UI
```

Use the mounted **Minecraft Instance Manager** interface, not the historical
VMware NSIS installer or `DEV.cmd g0`/`DEV.cmd package` (both release-blocked).
The current UI has **Check status**, **Set up**, **Open**, **Switch to** and
contextual **Move window**; it has no "Refresh both clients" or "Launch isolated
Client 02" action. The one Check status button inspects Client 02 Sandboxie
evidence; it does not confirm native Client 01 login or gameplay.

1. In licensed Sandboxie Plus, prepare exactly one persistent box named
   `MinecraftClient02`; AutoDelete/AutoRemove must be OFF. Review the
   credential-bypass/protected-storage settings as described below.
2. In **Instances**, press **Check status** before attempting Client 02 Open.
   If status is unknown or setup is incomplete, use **Set up** and correct the
   indicated Sandboxie settings before explicitly checking again. Unknown
   membership is not evidence of an empty box.
3. With the box positively inspected as prepared, use **Set up** → **Browse…**
   to choose the **installed** Minecraft Education Windows Desktop EXE,
   not its downloaded installer. Then explicitly confirm Sandboxie licensing
   and the persistent box. A different EXE selection resets that confirmation.
   The current pilot requires valid Authenticode signing by **Microsoft
   Corporation** and Education product metadata. The native and boxed launch
   routes also require the Desktop installation's documented registry version,
   one matching registered Minecraft Education Start app for this user, and
   successful Microsoft Store package enumeration. A registered Start AppID
   is not an authoritative EXE path. If a legitimate Education release has
   different signing evidence, review it before proposing source changes:
   never bypass signature verification simply to make the test pass.
4. Only when the UI presents **Open** for Minecraft 02, request its launch.
   Rust again checks the chosen EXE, explicit licence confirmation, Sandboxie
   persistence/configuration and a **fresh OBSERVED boxed process list with
   no Minecraft-name candidates**. Unreadable `/listpids` fails closed. A
   successful Start.exe helper exit means only that Open was requested, not
   that Minecraft launched, signed in or rendered. A timeout is UNKNOWN; do
   not blindly request a second launch.
5. Press **Check status** again. The collector distinguishes candidates
   inside `MinecraftClient02` from those outside it; outside-box does **not**
   prove native Client 01 identity. The collector uses `-SummaryOnly`,
   without writing an evidence file. With a unique eligible boxed window,
   **Switch to** requests a fresh backend-verified focus; open its **Details**
   for conditional **Move window** → **Left/Right** → **Apply**. Neither
   action proves authentication or moves Client 01.
6. Independently observe the officially launched native Client 01 and the
   boxed Client 02: two distinct licensed Microsoft Education logins,
   actual window/world rendering and input, multiplayer where required,
   and persistence. Mark missing, ambiguous or failed evidence explicitly;
   do not record account identifiers, tokens, world names or private images.

A launch or PID count does not override a manual game observation; a failed
launch is diagnosed without altering account/VM data.

The mounted UI offers **Switch to** only when a single
boxed, visible Minecraft-name process candidate has a known creation FILETIME.
The Rust owner re-inspects Sandboxie membership on every Focus, then
requests an **on-demand Authenticode verification of only the selected boxed
PID** through the existing short-lived read-only PowerShell collector
(`-SummaryOnly -BoxedImageProofPid <PID>`). The collector requires
the selected boxed process to have a visible window and known creation
FILETIME. The collector obtains a minimal query-only process handle, uses
`QueryFullProcessImageNameW` to read its executable path, rejects nonlocal
or reparse-point path components, and holds a read-only file handle with
`FileShare.Read` during Microsoft Authenticode/ProductName verification.
Before accepting the result it rechecks process creation time/image path,
reopens the same path and compares the file's `FileIdInfo` 128-bit ID and
volume identity. If access, path identity, file ID or signing is
inconclusive, evidence is UNKNOWN and Focus/Place are refused.
The Rust backend compares the fresh, signed boxed candidate's PID,
creation FILETIME and HWND with the previous UI observation and still
checks the open process handle and window ownership immediately before
acting. Only source-level behavior is described; execution remains unproven.
The Windows call then verifies the process creation time, HWND's owning
PID, and that it is a visible, unowned root window (not an owned
dialog/popup) before requesting foreground activation. The HWND is converted
to a native pointer width without truncation and validated again after
restoring a minimized window. This supplies only evidence for the signed
executable file currently reported for the PID, not immutable attestation
of its mapped running image. File paths and file IDs are neither returned
in IPC nor saved in reports. The read-only file handle reduces replacement
risk but cannot eliminate every filesystem or in-memory image race.
Files with unsupported path/handle semantics fail closed. It does not
establish authenticated accounts or playable gameplay. If the process
changed, use Check status first; if Windows rejects foreground activation, click the
game window or use Alt+Tab. No keyboard simulation, hidden polling,
automatic process start, VM route or sandbox data mutation is used.
The contextual **Move window** panel's **Left/Right** and **Apply** controls request half of
Client 02's current monitor work area only, after the same identity check.
They neither identify nor move Client 01, keep the existing Z-order and
focus, and require the game window to be restored from minimized/maximized
state first. Immediately before placement, the backend rechecks the full
live window ownership, minimized/maximized state, and whether the window
moved to another monitor while its work area was inspected. A changing
window fails closed and asks for Check status. Windows can queue SetWindowPos
asynchronously, so even a successful request is not a guarantee of final
size/position or protection against every HWND race. Windows may adjust the
final size to Minecraft's constraints; the accepted request does not
certify rendering. Sandboxie Plus requires
a Business Certificate for business/educational use as described upstream.

## Official Desktop installation provenance

[Minecraft Education's Windows installation guide](https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide)
documents the 32-bit-registry-view `Version` detection key and
mutually incompatible Desktop and Store installation methods. The
Desktop installer also supports a custom `APPDIR`; checking only
`Program Files` would therefore reject legitimate installs.

The active Client 01 and Client 02 launch requests now apply the same
registered Start-app uniqueness and valid AppID syntax checks, alongside
Desktop registry/version and current-user Microsoft Store package
inventory. Failure to enumerate the Store package is treated as a
preflight failure, not silently interpreted as no package. The chosen
Client 02 EXE additionally requires the existing Microsoft-signed
Education product metadata and filename rule. These checks do **not**
prove that the Start AppID resolves to the selected executable, because
`Get-StartApps` exposes AppUserModelIDs, not trustworthy EXE paths.
They also do not rule out Store packages for other Windows users.
The manager does not guess unknown installation directories, modify
Start registration, copy executables or treat a file signature as proof
of licensed gameplay. Ambiguous installation provenance stays unverified.

## Bounded collector transport

The collector is embedded in the Rust/Tauri binary but **is not passed
as a long `-Command` argument**. Large scripts can exceed Windows'
32,767-character process-command-line limit. The existing bounded
subprocess runner now accepts an optional UTF-8 stdin payload (128 KiB
maximum), and PowerShell uses a fixed short `-Command` bootstrap to
read it as UTF-8 and execute a script block. It retains one explicit
inspection, a 30-second command deadline, bounded stdout/stderr, direct
child cleanup, no background polling and no persistent script file.
Normal provider callers still use the original no-input command route.
This transport has not been exercised on a real Windows machine.

## Boxed executable attribution (source-only)

- **Manual Check status:** `-ImageProof` still checks visible outside-box
  Microsoft-signed Education file candidates and can invoke the manual
  vendor-DLL per-PID box lookup. It does not mark Client 01 truly native.
- **On-demand Focus/Place:** `-BoxedImageProofPid` verifies only one
  observed Client 02 PID, without invoking the manual outside-box DLL
  `QueryProcess` route. A prior UI `imageEvidence` value is never
  accepted as authorizing a window action.
- **Launch:** unchanged; box membership and persistent settings are
  preflight prerequisites, but executable path and signer checks remain
  in the existing launch request. No signed PID observation authorizes
  additional processes or proves successful launch.
- **Evidence:** an executable's valid Authenticode status, Microsoft
  publisher and Education product metadata are a *signed-file candidate*,
  not Microsoft sign-in, gameplay, graphics, multiplayer or a guarantee
  that a mapped running image cannot change. Signature checks happen on
  a path obtained from the bounded process handle, not a caller-supplied
  path. The collector pins that path read-only during verification and
  compares a freshly reopened file's 128-bit FileIdInfo and volume
  identity. Failure to resolve path, open the handle, reject reparse
  points, or keep the process/file identity stable returns UNKNOWN.
  This remains signed-file evidence, not in-memory image attestation.
  No credentials, token material, window titles, account data or private
  world files are read.
- **Source/proof boundary:** no schema version changes: the existing
  `imageEvidence` label now permits signed candidates either outside
  the selected box or inside its verified membership. The existing
  `officialImageOutsideSelectedBoxCount` counts *outside only*: a signed,
  targeted boxed process never increments it. The Rust consumer separately
  reconciles this count with the signed candidates attributed outside the
  selected box. This source-contract mismatch was corrected after the
  initial REA-aligned audit. Windows, PowerShell, Rust, frontend, Minecraft
  and performance checks have not been executed for these revisions.

References:
- https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.security/get-authenticodesignature
- https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-queryfullprocessimagenamew
- https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getfileinformationbyhandleex
- https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-file_id_info
- https://learn.microsoft.com/en-us/dotnet/api/system.io.fileshare

## Prerequisites, manually managed

Use the official **Desktop** version of Minecraft Education, not a copied
executable or a Store/Win32 dual installation. Leave native Minecraft world
files unchanged. Sandboxie Plus is optional and third party: its drivers,
installer, box creation and license are NOT managed by Minecraft Instance Manager.

Before running the two-game test, an authorized operator:
1. Verifies Sandboxie Plus is legitimately licensed for its intended use.
   Its own documentation requires a Business Certificate for business or
   educational environments; App Compartment options require an eligible
   Supporter Certificate. Do not assume access is licensed.
2. Creates a **test-only**, ordinary, persistent box named **exactly
   MinecraftClient02** using Sandboxie Plus SandMan. The active backend and
   collector use this single fixed name; other box names are not supported.
3. Reviews **Auto Delete**, **Auto Remove** and **Open System Protected
   Storage / OpenCredentials** in the selected box. AutoDelete/AutoRemove
   must be OFF. The collector resolves OpenCredentials in the box and
   inherited GlobalSettings; it also checks a box's explicit
   `OpenProtectedStorage` template from both the local box and
   `GlobalSettings` selections, plus an explicit direct flag if present.
   This cannot resolve arbitrary nested template settings. A known bypass or
   unreadable/UNKNOWN setting prevents launch. Even with OFF, intercepted
   WinCred operations can fall back to host credential reads; therefore
   OFF **does not independently establish Windows-account or WAM/SSO
   session isolation**.
   Do not enable profile copying, credential sharing or automatic cleanup.
4. Starts one official Minecraft Education Windows Desktop game natively,
   then requests the official game from within that selected Sandboxie box
   through the **Minecraft Instance Manager Client 02** launch action using the same signed,
   installed executable. Neither a launch request nor explorer.exe alone
   proves that Minecraft is actually sandboxed.
5. If *two real independent windows* appear, signs in to two distinct,
   properly entitled Education accounts **manually**. Checks both worlds,
   Microsoft sign-in redirects, input, multiplayer, closing and reopening,
   and a host reboot. No account/credential values go into the report.

If Minecraft cannot authenticate or open inside the box, mark it NO-GO.
Do NOT bypass Microsoft licensing, app single-instance guards, WAM/SSO
broker restrictions or Sandboxie driver protections.

## Read-only inspection (runs from repo root)

    powershell.exe -NoProfile -File Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1 -Stage INSPECT -BoxName MinecraftClient02

The collector checks signed installed Start.exe / SbieIni.exe,
Sandboxie service state, enabled-box/AutoDelete/AutoRemove settings,
the WinCred-hook bypass setting (including an explicitly configured
OpenProtectedStorage template), selected-box PID membership and Windows
candidate process windows. The historical JSON field
`credentialHostAccess` must be interpreted as a configuration/bypass
indicator, not a proof that host credentials are inaccessible. Its source is embedded in the development
Tauri backend so the inspector and launcher preflight reuse the same rules.
The current desktop summary is IPC schema 6 and includes a bounded
list of Minecraft-name process candidates with PID, optional Windows
creation FILETIME string, HWND, observed sandbox membership and image
evidence. The list is internally reconciled against visible/boxed counters,
including processes without a window. A process without accessible start
time remains a name-matched candidate only. Explicit desktop Check status
uses SummaryOnly -ImageProof: only visible candidates outside the
selected box are checked for valid Microsoft Corporation Authenticode
and Minecraft Education product metadata, using an invocation-local cache
without sending paths to the UI. Launch/Focus/Place omit -ImageProof to
avoid repeated signer checks. When a signed candidate exists, explicit Check status optionally loads
the installed SbieDll.dll inside its disposable PowerShell child, checks
its digital signature against the trusted Sandboxie Start.exe signer, then
calls the documented SbieApi_QueryProcess for at most 16 candidates.
The call is bounded to the current inspection process and checks the
candidate's Windows creation FILETIME before and after the lookup. A
successful returned box name different from MinecraftClient02 counts as
OTHER_BOX; a nonzero return, reused PID, missing start time, load failure
or unavailable API counts as UNKNOWN. **UNKNOWN must never be interpreted
as confirmed host-native**, including when Sandboxie did not register a
process. No sandbox names, executable paths or user credentials are
included in the IPC response. DLL initialization failures can end the
inspection subprocess and surface a Check status error; this has not been
run against an installed Sandboxie version. The persisted full JSON report remains schema 1
with earlier process field shapes for the existing evaluator.
The Rust backend serializes manager-initiated launch attempts and suppresses
rapid ambiguous retries; it does not control independent launches made
outside the manager. An UNKNOWN /listpids result remains UNKNOWN, not an
assertion of an empty sandbox.

It first queries the enabled box and enumerates its explicit box-local
setting keys using SbieIni.exe query BOX *. If that enumeration succeeds,
a missing AutoDelete/AutoRemove key means default OFF. An explicit key is
queried directly. A failed/ambiguous enumeration or invalid value remains
UNKNOWN, never OFF. GlobalSettings is deliberately not used for those
SandMan lifecycle controls. New reports record boxSettingsSource as
BOX_SECTION_ENUMERATED or UNKNOWN; legacy reports lacking provenance remain
readable but cannot be reported compatible. Old records that omit
credentialHostAccess likewise stay INCONCLUSIVE until new host evidence
is collected. Full evidence reports now contain this field (older reports
are not silently migrated or treated as complete).
It never records full process paths, CLI arguments, box filesystem paths,
world names, account identities, cookies or tokens. Its OpenCredentials
field is a Sandboxie configuration observation, not proof that Microsoft's
WAM helper processes are contained. The explicit CLI collector writes
an evidence file by default. The desktop's `-SummaryOnly` inspection
returns only a bounded response without writing an evidence file. All
unreadable probes stay UNKNOWN, never an invented success. An unsigned/custom source build
is NOT automatically invoked; it requires separate review.

Optional manually selected install directory:

    powershell.exe -NoProfile -File Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1 -InstallDirectory "D:\Tools\Sandboxie-Plus"

The -InstallDirectory option is only used for read-only inspection
of tools already installed. It never downloads or installs binaries.

After actually observing **both** login/gameplay sessions and completing
the close/reopen/reboot persistence check, record operator statements.
Replace each YES/NO with UNKNOWN unless personally observed:

    powershell.exe -NoProfile -File Development/tooling/acceptance/windows/collect-sandboxie-feasibility.ps1 -Stage TWO_CLIENT -BoxName MinecraftClient02 -LicensedForUse YES -NativeWorldPlayable YES -SandboxWorldPlayable YES -DistinctAccounts YES -IndependentInput YES -AccountCrossingObserved NO -ReopenAndRebootPersistent YES -MultiplayerJoined UNKNOWN -ObservedMinutes 10 -EvidenceReference isolated-local-001

The JSON is written under %TEMP%\MinecraftInstanceManager\SandboxieFeasibility. Previous reports remain at their prior paths; nothing is migrated or deleted.
Record no screenshots/account identifiers in Git. Evaluate the specific file:

    node Development/tooling/acceptance/evaluate-sandboxie-feasibility.mjs "<report.json>"

Pure Node fixture tests only:

    npm --prefix Development run test:sandboxie-feasibility

### Process changes between Check status and Switch/Move

Process/window counts in the UI are last-observed hints, never a continuous
claim that a game is running. If a boxed Minecraft candidate exits or its
PID/creation FILETIME/HWND changes, the existing Rust action boundary refuses
the stale target, and the UI clears the stale observation on failure.

**Additional source guard (2026-10-14):** a different failure mode existed
when the old target survived but a *second boxed game candidate* appeared
after inspection. The frontend only offers actions for one boxed candidate,
but the earlier Rust `verify_fresh_focus_target` accepted any exact matching
target among a newer multi-candidate snapshot. Both Focus and Place now
require the freshly collected snapshot to contain **exactly one boxed
Minecraft candidate and one visible boxed candidate** as well as the
existing PID/FILETIME/HWND, signed executable and Win32 checks. A separate
outside-box native candidate is not grounds to reject the boxed one.

These are source and regression assertions, not evidence that the Windows
collector, Svelte, signed clients, Microsoft authentication or gameplay passed
actual runtime acceptance. The manager does not add periodic polling or
automatically start/stop Minecraft. If an action refuses an outdated state,
the user may explicitly use Check status to inspect again.

## Interpretation and safety

- CONFIGURATION_BLOCKED: AutoDelete/AutoRemove is ON, or box explicitly
  disabled; accounts must not be entrusted to this box.
- OPERATOR_REPORTED_FAILURE: operator observed failed sign-in, loss of
  persistence, cross-account reuse, failed world, or missing entitlement.
- OPERATOR_REPORTED_COMPATIBLE: candidate Windows PID/window
  inventory, verified box-local settings, and operator reports include
  both independent gameplay, multiplayer success, and reboot persistence.
  This is **not** automated Minecraft or account proof.
- INCONCLUSIVE: absent or incomplete evidence, including unknown storage
  settings, unsigned tools, service failure, or unverified PID membership.

The evaluator additionally distinguishes processFeasibility
(TWO_WINDOW_CANDIDATES_OBSERVED or INCONCLUSIVE) from accountFeasibility
(operator-reported, failure or inconclusive). Neither is a product
acceptance gate: productAcceptance remains NOT_EVALUATED for four clients.
An unknown reboot/persistence **or multiplayer** result does not erase
genuine intermediate account observations; it still prevents the final
compatible verdict. An explicitly failed multiplayer attempt yields an
operator-reported failure, not a compatible verdict.

A valid Authenticode signature proves binary-signature integrity only,
**not** the current license/certificate, game compatibility or account
isolation. Even a compatible two-client pilot is not proof that four
accounts, networking, Microsoft broker behavior after updates, or long-run
storage use work. An operator must independently review genuine game
evidence before any production architecture decision.

**STOP:** do not add an application-provider adapter, modify Rust VM
lifecycle, rewrite the UI, or remove guest data based on this collector.
Only the next bounded Windows compatibility observation is authorized
by this experiment.

## Proof ceiling

At authoring: REMOTE_GITHUB source only. Node regression fixtures
authored but not necessarily executed; actual signed Sandboxie runtime,
interactive Minecraft Education, two/four account use and persistence
are NOT_EXECUTED.
