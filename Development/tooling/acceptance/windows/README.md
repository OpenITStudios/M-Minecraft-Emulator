# Windows Acceptance Tooling

## Current Windows-native RAM baseline (0–5 profiles, source prepared)

The existing read-only collector now has a native-profile mode that reuses its
process identity, private commit/private working set, total-system memory,
optional direct-child observation and atomic file writer. It **does not**
start, stop, suspend or modify Minecraft. The mode samples 0–5 already-running
licensed clients, in increasing MC01–MC05 slot order. Default labels are
MC01 through MC0N; use matching -ProfileSlots 1,4 for sparse MC01 and MC04,
for example. Do not supply a process ID for an absent profile. A baseline with no clients can still observe the
system and optionally an existing manager. It deliberately does not infer
gameplay, host health, GPU physical usage, memory ownership or RAM savings.

~~~powershell
# 0 clients; optional manager process.
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 -NativeProfileBaseline -MemoryComposition -ManagerPid <MANAGER_PID> -DurationSeconds 60 -SampleIntervalSeconds 5

# 2 existing clients + manager; no new game processes are started.
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 -NativeProfileBaseline -MemoryComposition -ProfilePids <MC01_PID>,<MC02_PID> -ManagerPid <MANAGER_PID> -DirectChildren -DurationSeconds 600 -SampleIntervalSeconds 5

# Sparse MC01 and MC04; labels must be explicit when MC02/MC03 are absent.
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 -NativeProfileBaseline -MemoryComposition -ProfilePids <MC01_PID>,<MC04_PID> -ProfileSlots 1,4 -DurationSeconds 60 -SampleIntervalSeconds 5

# 5 selected clients; IDs must be unique and supplied in slot order.
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 -NativeProfileBaseline -MemoryComposition -ProfilePids <MC01_PID>,<MC02_PID>,<MC03_PID>,<MC04_PID>,<MC05_PID> -DurationSeconds 600 -SampleIntervalSeconds 5
~~~

New schema **7** contains ordered slot labels, permanently pinned process
identity/loss flags, CPU and memory samples, whole-system memory and optional
direct-child aggregate measurements. Its native process readings also include
the cumulative Win32 `PageFaultCount` from the **same process-memory query**;
this is a count, not MiB, not a hard-fault/disk-I/O counter, and is never
interpreted as proof of reclaimed RAM. The reviewer accepts older schema-7
reports without this field and withholds numeric comparisons when coverage
differs. Decreasing/wrapped counters report an unknown endpoint delta. No PID, login, world or account data is
written into the JSON. Keep the historical two/four/five-client command modes
and schemas 1–6 for comparisons with older diagnostic files; they are not the
preferred current native five-profile capture. This new path is source-prepared
only; Windows execution and RAM claims remain NOT_EXECUTED.

Schema-7 reports may be reviewed and compared with the existing Node
evaluator. Comparisons are descriptive numeric differences only; they
never prove a matched workload, physical RAM savings, game readiness,
or the safety of a proposed internal allocator.

~~~sh
node Development/tooling/acceptance/evaluate-native-performance.mjs native-memory-baseline-before.json
node Development/tooling/acceptance/evaluate-native-performance.mjs native-memory-baseline-before.json native-memory-baseline-after.json
~~~

The older `performance-contract.json` describes the retired VMware-era
performance gate; it does not override the current native schema 7.
This source change does not enable CI, scheduled sampling or runtime tests.

## Existing MC01 reactivation/lifecycle diagnostics (explicit opt-in)

The same [collect-mc01-startup.ps1](collect-mc01-startup.ps1) now has a separate
reactivation diagnostic path. Default invocation and ordinary -Record retain
their existing first-startup behavior and refusal of existing game/helper
processes. This opt-in is not a product launch command, isolation provider or
authorization to run a second game executable.

Use 64-bit Windows PowerShell 5.1. Source preparation does not itself authorize
game activity:

~~~powershell
# Pure guard checks; no game/registration observation or activation.
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-mc01-startup.ps1 -SelfTest
# Actual observer control: creates/releases only its own hidden PowerShell child.
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-mc01-startup.ps1 -LifecycleSelfTest
# Read-only reactivation preflight, requiring one existing official MC01.
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-mc01-startup.ps1 -Reactivation
~~~

Only after explicit approval of **one new registered MC01 reactivation and a
30-second metadata observation**, add -Record to -Reactivation. The caller must
be unelevated in an interactive session. The tool never elevates or starts WPR
in this mode. It verifies the existing registered Desktop image/signature/hash,
Start-app GUID/path binding, absence of the Store edition and exactly one
readable same-session game. It retains the original process handle/creation
FILETIME and repeats binding checks before the one Explorer AppsFolder request.
There is no direct-EXE fallback, retry, arbitrary target/PID/arguments, game
termination, sign-in/out, DLL/driver operation or product admission change.

The selected observer uses WMI intrinsic __InstanceCreationEvent and
__InstanceDeletionEvent with a fixed one-second polling interval. This is an
explicit weaker observation choice, **not an automatic fallback**: the initial
Win32_ProcessStartTrace control was denied access on this host, so that native
trace mechanism is not used for the new unelevated mode. The old elevated
first-startup collector remains unchanged. Intrinsic polling can miss a process
that appears and exits between polls. Passing controls or observing no additional
process **does not prove complete coverage, shell-only reuse or a game mutex
rejection**.

Before any activation, a fixed hidden PowerShell child waits on its own input
pipe. Start and stop must both be observed within bounded waits, matched to its
original process identity/session and WMI creation time (microsecond precision).
Only this self-created control may be stopped on control failure. Control failure
or failed revalidation prevents the activation request. Self-test combinations
with activation/recording are rejected. Cleanup removes only this invocation's
GUID subscriptions and queued events and checks for remaining owned subscribers.

The local receipt records caller-side request start/end UTC and QPC ticks,
submission outcome, initial MC01 identity and positively observed process
candidates/new snapshots. The planned attempt is journaled before the call;
activationSubmitted means only that Explorer submission returned successfully,
not that a second game started or that a game PID was returned. Events carry
allowlisted names/PID/parent/session/birth/event/delivery times; snapshots store
registration-match flags rather than full paths. Parent PID is not actual
creator evidence. The collector does not read CommandLine/SID properties or export the whole WMI
TargetInstance object. No credentials, token/cache/world contents or
IPC payloads are accessed. Receipts remain private in a new GUID TEMP directory;
at most 256 selected events and 256 new lifetime candidates are retained.
The original game is never terminated or signed out by this observer; a concurrent
exit or identity/collection error makes the result incomplete, without retry.

OBSERVATION_COMPLETED means the timed observer finished. activationTargetPid,
windowResult, lifecycleCompleteness and singletonDecision remain UNKNOWN;
authentication is NOT_EXECUTED and providerReady=false. An actual game-bound
API/object/result/rejection still requires separately reviewed instrumentation.

**Executed source proof — 24 October 2026 WIB:** exact baseline
d8fd9a6a85e2d9338ab0366fbef388a1ba0f309c; final collector Git blob
41d377bd1a5ac152c51c6bc3a97efc4bc23de5ac, SHA-256
d272cadb92781afa8fc5806f6ce5edbcb9b707bfd479c2aacd340e77d3680a56.
Windows PowerShell 5.1: -SelfTest exits 0 with 19 guard/native-error checks;
-LifecycleSelfTest exits 0 with both real intrinsic control notifications and
cleanup. Two conflicting-switch invocations refuse. The legacy startup body
from its original preflight onward is byte-identical to the baseline.

A retained local-only mocked collector copy exercised the full 30-second
orchestration/receipt path with game, registration, events/control and activation
dependencies replaced: successful receipt, one mock submission, original
self-process preserved; forced control failure yields INCOMPLETE at control
stage, zero attempted/submitted activations and unchanged mock activation marker.
This is mocked orchestration, **not** an executed Minecraft request or complete
real-game preflight. Local artifacts:
workspace/tmp/reactivation-1792827565190; final mock copy SHA-256
663c3968d94bd38da5dcf0978009695d6d24b614d1b4b767da3c5441151fa205.
The canonical registered-target resolver was not changed.
At a subsequent separately authorized run of exact source
46cb8d4a9df9e35b4b30744b08ac3b4bc4ad9423, one real reactivation and its bounded
metadata observation executed: an additional registered-path process candidate
appeared then received a stop notification while the original MC01 survived.
Its role and exit/rejection reason remain UNKNOWN. See the
[actual reactivation evidence](../../../../docs/validation/native-pilot-localtest.md#authorized-existing-mc01-reactivation--24-october-2026-wib).
Only the actual-reactivation NOT_EXECUTED boundary is superseded; kernel
recording, singleton/auth and two/four-client acceptance remain NOT_EXECUTED.

Sources:
[WMI creation notification](https://learn.microsoft.com/en-us/windows/win32/wmisdk/--instancecreationevent),
[deletion notification](https://learn.microsoft.com/en-us/windows/win32/wmisdk/--instancedeletionevent),
[WITHIN polling](https://learn.microsoft.com/en-us/windows/win32/wmisdk/within-clause).
The [evidence sufficiency decision](../../../../docs/validation/native-pilot-localtest.md#singleton-evidence-sufficiency-decision--24-october-2026-wib)
explains the missing game-specific discriminator. Current continuation remains
solely in planning/development.md.

## First MC01 startup metadata (opt-in, no game launcher)

`collect-mc01-startup.ps1` defaults to read-only preflight. It refuses recording
if **any** `Minecraft.Windows` or `Minecraft.AdalServer` basename is already
running, including another session or unreadable identity. Preserve that session;
do not close/sign out just to collect evidence. In default/first-startup mode, the collector never launches,
restarts, injects or stops a game and never changes product admission.
Installation resolution reuses
[`Resolve-ManagerEducationTarget -RegisteredOnly`](education-target.ps1);
the operator's normal first launch belongs to the existing
[`open_native_minecraft`](../../../apps/desktop/src-tauri/src/engine/native_clients.rs)
owner. No direct EXE/Explorer activation is implemented here.

Use **64-bit Windows PowerShell 5.1**:

```powershell
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-mc01-startup.ps1
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-mc01-startup.ps1 -SelfTest
```

Only after separate approval for a new recording, and when no existing target
session needs preserving, run the same command with `-Record` from an explicitly
elevated Windows PowerShell. There is no automatic UAC/relaunch. Wait for
`MC01_STARTUP_READY_30_SECONDS`, then manually request **one first MC01 launch**
through the existing official manager route. Do not request MC02 or repeat
activation. A target appearing before ready, another game lifetime, wrong
registered image/session or unreadable game identity makes collection incomplete.
A concurrently launched game is preserved; the collector stops its own trace,
not that process. A race cannot be prevented by this observer.

The fixed, hash-checked [WPR profile](mc01-startup.wprp) requests Handle/Loader
metadata and create/close/duplicate stacks with **32 MiB collector buffers** in
**sequential file mode** for a 30-second observation window. The collector always
passes `-filemode`; the profile's required memory variant is not used by this
collector. The authorized memory capture lacked the beginning of startup despite zero loss
counters, consistent with circular overwrite; that mode is unsuitable here.
Visible ETL lengths in the private run directory are sampled during the window;
reaching **256 MiB** stops recording as incomplete. This is a stop threshold,
not a hard disk quota: buffering, polling and stop/rundown writes can exceed it.
Neither the threshold nor the 32 MiB buffer allocation limits total process
memory. File mode still requires review of coverage and loss before any verdict.
It excludes ProcessThread, file I/O, registry, network and memory-page providers.
Separate WMI subscriptions filter only the two basenames and persist
selected PID, parent PID, session and event-time fields, excluding SID and command
line. Snapshots pin available target PID/creation-FILETIME/session/path; on-disk
Microsoft trust/hash is not loaded-image binding. WMI event time is not process
creation time, parent PID is only a candidate, and transient/early-exit processes
may lack a pinned snapshot. Actual creator and completeness remain **UNKNOWN**.

Raw ETL is system-wide: unrelated object names, module paths and stack addresses
may be present. No credentials, token/cache/world contents, registry payload,
network payload or command lines are deliberately queried. Keep raw ETL,
WPR text and receipt private and local; publish only reviewed sanitized findings.
Output uses a new GUID directory under the current user's TEMP, never a tracked
report or overwrite of an older capture. The GUID WPR instance is supplied to
every start/status/stop/cancel; cleanup never cancels a global/other recording.
The observation window is timed; WPR start/save and finalization take additional
time. Ordinary exceptions enter cleanup; forced process termination can leave
its named trace requiring explicit recovery.

The profile retains `SkipMerge` from the previous authorized raw capture to avoid
normal merge enrichment. Microsoft describes TraceMergeProperty as internal use;
profile validation is not a promise of portable recording behavior. `-recordtempto`
requests the private run directory. On stop, primary/rundown filenames may differ
from `startup.etl`; the receipt inventories and hashes actual local ETL parts
without assuming one merged file. Missing files or failed stop make the result
`INCOMPLETE`; preserve diagnostics and temporary parts, do not silently retry or
declare success. Final status/cleanup output needs review. Collection state
`WINDOW_OBSERVED` means only that the timer completed and local ETL was saved;
`startupCoverage`/`singletonDecision` remain UNKNOWN and `providerReady=false`.

Sources: [Microsoft WPR commands](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/wpr-command-line-options),
[named instances](https://devblogs.microsoft.com/performance-diagnostics/controlling-the-event-session-name-with-the-instance-name/),
[WMI start metadata](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/krnlprov/win32-processstarttrace),
[logging modes](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/logging-mode),
[TraceMergeProperty scope](https://learn.microsoft.com/en-us/windows-hardware/test/wpt/tracemergeproperty).

The two separately authorized 30-second recordings at `aaf7183ecbcfe9da1554ff7207256e015ccbb896`
confirmed raw-part placement, successful named stop and final not-recording status
on this Windows build. The first observed no game; the operator had not clicked.
The second pinned one real first-launch lifetime and received one matching WMI
start candidate, but its circular buffer retained provider activity only from
**6.409 seconds after game creation**. Startup/singleton proof was not obtained.
No new recording is implied by this source correction. See the
[exact capture evidence](../../../../docs/validation/native-pilot-localtest.md#startup-capture-coverage-correction--24-october-2026-wib).

The revised source's thirteen guard/native-error/size-threshold self-checks passed
in Windows PowerShell 5.1; WPR 10.0.26100 accepted the File profile. A later
operator-authorized one-off coordinator completed one automatic first MC01
activation after recorder-ready from an unelevated session, using the pinned
official activation recipe. The Tauri product command/admission lock was not
invoked. No automatic launch mode was added to this committed collector.

That File recording saved one 275.75 MiB raw ETL, stopped its named session, and
retained the main-image Load plus first target activity 14.7091 ms after process
creation. The 256 MiB stop threshold did not fire and is not a quota. The
offline decoder left 59 object property failures unresolved; no singleton or
auth verdict follows. See [automatic capture evidence](../../../../docs/validation/native-pilot-localtest.md#automatic-file-mode-startup-capture--24-october-2026-wib).
**NOT_EXECUTED:** threshold-triggered stop, failed-stop cancellation/recovery,
product-command admission in that one-off run, or game isolation/acceptance.
Current blocker/next action belongs only to
[planning/development.md](../../../../planning/development.md).

## Early compatibility bootstrap ordering (self-child only)

The standalone [bootstrap-ordering experiment](bootstrap-ordering/README.md)
checks one first-import candidate against dependency/TLS/main canaries, with
baseline, attach-failure and timeout controls. It builds only with an explicit
opt-in and a pinned external Microsoft Detours source checkout. It accepts no
target PID/path, is not linked into the product, and cannot approve Minecraft
isolation. At source `d9a2fb8`, the Release build and all four self-child controls
executed successfully on Windows x64. The same standalone tool now also checks
real CreateMutexW/OpenMutexW translation on two self-owned children; its host,
peer, create/open/error/access/inheritance-flag and cleanup checks passed.
Direct peer backing access remains possible, so this is not a security boundary.
See linked exact-source evidence. Minecraft mediation and acceptance remain
**NOT_EXECUTED**.

## Independent native AppContainer proof (synthetic only)

### Optional two-process application-hive load-lock probe

Use `test-appcontainer-isolation.ps1 -VerifyRegistryAppHivePeer`
to run an additional source-only native fixture mode
`--apphive-peer-fixture`. The parent first builds two **new**
disposable hive files and checks fixed synthetic markers and
close/reload persistence. It then starts two short-lived, separate
instances of the **same fixture executable**. Each child receives
only `A` or `B` as its argument and inherits the new hive-root
directory as its working directory, not a caller-controlled hive path.
Each child loads/reads its **unlocked own hive** while the parent
holds `REG_PROCESS_APPKEY` on the opposite one. Only a recognized
access/sharing/busy/lock refusal for the opposite file counts as
an observed *process-level load lock*; unexpected success or an
unrecognized error fails the test. Both directions are required.

The schema-2 sanitized receipt reports
`bidirectionalProcessLoadLocks=OBSERVED` only after both children
return successfully. It keeps `peerProcessDenial=NOT_EXECUTED`,
`hostHKCUIsolation=NOT_EXECUTED`, and
`minecraftAcceptance=NOT_EXECUTED`, because a load lock is NOT
proof of registry ACL protection, separate HKCU namespaces, separate
accounts or Minecraft compatibility. The children are ordinary
synthetic test processes—not signed Minecraft or AppContainer clients.
The original `-VerifyRegistryAppHive` mode remains unchanged.
All Windows/MSVC execution is **NOT_EXECUTED** in GitHub-only work.

### Opt-in RegLoadAppKey application-hive fixture

Use `./tooling/acceptance/windows/test-appcontainer-isolation.ps1
-VerifyRegistryAppHive` to run an **additional** fixture under a
newly created `workspace/appcontainer-probe-*/registry-fixture` output.
The C++ fixture creates two new files in its own unique subdirectory,
loads each with `RegLoadAppKeyW` and `REG_PROCESS_APPKEY`, writes
only fixed test DWORD markers through those returned hive `HKEY`s,
verifies independent handle views, flushes and closes both handles,
then reloads both files to check persistence after *handle unload*.
It never uses `HKEY_CURRENT_USER`, `HKEY_USERS`, `RegLoadKey`
or any path in a Microsoft/Minecraft account profile. Artifacts
remain inside the new fixture directory for inspection.

The receipt explicitly reports `peerProcessDenial=NOT_EXECUTED`,
`hostHKCUIsolation=NOT_EXECUTED` and
`minecraftAcceptance=NOT_EXECUTED`; it is not a game-ready decision.
It does not prove access denial between independent processes,
persistent state after a Windows restart, or transparent HKCU routing.
The switch is disabled by default, and Windows/MSVC execution remains
NOT_EXECUTED during GitHub-only development.


### Optional native-to-Rust HANDLE handoff

`./tooling/acceptance/windows/test-appcontainer-isolation.ps1 -VerifyRustHandoff`
builds the feature-gated Rust helper from the existing `Development/engine/runtime-core`
Cargo package under a new workspace directory, and opts into
`--run-rust-handoff` in the original C++ fixture. Each of four synthetic
record reads (A/B, initial/reopened) can then be checked by a separate
bounded Rust subprocess. The C++ parent passes **only** two inherited
HANDLEs via the Win32 allowlist and an expected PID/FILETIME/phase,
not paths or credentials. An exit code other than zero fails the
fixture. No server/daemon, release binary or new GitHub Actions
workflow is created. This code requires Windows/MSVC/Rust x64 and has
not been run in the GitHub-only development context.

The test assumes the **original parent fixture** is the trusted owner
of freshly created temporary profiles and process/file handles. It
is not a production capability mechanism and cannot establish
Minecraft roaming or authentication readiness.


### Rust record HANDLE identity validation

The Windows-only Rust reader now applies
`GetFileInformationByHandle` to the caller's **already-opened**
fixture record handle before and after one eight-byte read. A
directory, linked object, wrong size, changing file identity or
last-write timestamp fails closed. The caller must still open only
its newly created fixture profile file using
`FILE_FLAG_OPEN_REPARSE_POINT` and exclusive sharing: a followed
link can otherwise hide the original reparse attributes. Even with
these checks, the Rust reader is **not authenticated** to the
C++ parent; it returns `UnboundFixturePaths` only.

### Owner-bound synthetic process observation

The C++ fixture now pairs its `Child` directly with the
fresh `Profile` supplied at successful launch. The observation
and stop helpers accept **only the owned Child**. They cannot be
called with a second arbitrary profile path or an independently
selected owner SID. This is an in-parent ownership improvement;
the separate Rust fixture reader still lacks a transferable
process/file capability and must return `UnboundFixturePaths`.
The existing synthetic fixture creates only fresh AppContainer
profiles and does not access installed account/world data.

### Original fixture parent ownership checks

The native fixture parent records each child process's creation FILETIME
from its original CreateProcessW handle, then checks liveness and SID
before and after reading only its fresh profile's observation file.
It refuses record reparse points, non-regular files and records of
unexpected length and rechecks the file identity/size. Rust also rejects
binary records missing the fixture's mandatory isolation control flags.
These are synthetic fixture checks only, not authenticated cross-language
IPC or Minecraft compatibility/launch permission.

### Bounded native Rust reader (not process-bound yet)

`Development/apps/desktop/src-tauri/src/engine/instance_identity.rs` now supplies
an internal, Windows-only read helper for an **already opened** fixture
record handle. It verifies exact process creation FILETIME and an
AppContainer token before and after one eight-byte read. Wrong record
length or non-regular file is refused. The output is intentionally
`UnboundFixturePaths`, since no link from the supplied file handle to
that process's original fresh profile is yet established. The native
fixture itself still owns/validates its child handles. This helper
does not open arbitrary paths, touch Microsoft accounts, spawn games,
enable a provider, or provide production evidence.

### Rust decoder for the existing synthetic child record

The native fixture writes `Observation` records containing exactly two
little-endian Windows DWORDs (`version=1`, bitmask flags). A compile-time
assertion fixes this size at eight bytes; the pure Rust decoder in
`Development/engine/runtime-core/src/independent_engine/fixture_wire.rs` rejects
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
node Development/tooling/acceptance/evaluate-appcontainer-isolation.mjs workspace/appcontainer-probe-<id>/result.json
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
See [independent engine findings](../../../../docs/system/independent-isolation.md).

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
127.0.0.1:1421: `node Development/tooling/acceptance/workspace-ui-regression.mjs`.
It requires an available Playwright installation and Edge; optionally set
`MIVUBI_PLAYWRIGHT_MODULE` to that installation's absolute module directory.
It mocks Tauri, creates no game or engine processes, and writes only synthetic
UI screenshots below ignored `workspace/`. It covers resize errors retaining
recovery controls, native error events and three viewport sizes. Not gameplay proof.

The approved composition now has a developer product implementation in
`Development/apps/desktop/src-tauri/src/engine/workspace_surface.rs`. Its opt-in regression
uses the actual debug executable and real synthetic windows. Build the frontend
and debug desktop first, then run:

```powershell
cargo build --locked --manifest-path Development/apps/desktop/src-tauri/Cargo.toml
cargo test --locked --manifest-path Development/apps/desktop/src-tauri/Cargo.toml actual_surface_helper_restores_on_release_pipe_loss_and_helper_crash -- --ignored --nocapture
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

Run `powershell.exe -NoProfile -NonInteractive -File Development/tooling/acceptance/windows/test-workspace-host.ps1`
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
See the [workspace contract](../../../../docs/product/unified-workspace.md).

**Selected active experiment:** [Client 01 native + Client 02 Sandboxie Plus](../../../../docs/validation/sandboxie-two-client-feasibility.md).
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

## Native client process performance sampling (manual, read-only)

`collect-native-performance.ps1` is an **optional on-demand** Windows
sampler for the original native pilot. It does not launch games, inspect
accounts/worlds, certify Minecraft ownership or test FPS. Independently verify
the selected existing process identities in an authorized Windows session.
The historical NativePid/IsolatedPid parameter names and schema-1/2 scope label
remain compatibility names for slots 01/02; they do not select Sandboxie or
attest either provider. Example:

```powershell
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 `
  -NativePid <CLIENT01_PID> -IsolatedPid <CLIENT02_PID> `
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

### Five-client CPU/RAM capture and optional GPU hints

Use the same collector after independently verifying all five existing client
identities in an available operator session:

```powershell
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 `
  -NativePid <CLIENT01_PID> -IsolatedPid <CLIENT02_PID> `
  -Client03Pid <CLIENT03_PID> -Client04Pid <CLIENT04_PID> -Client05Pid <CLIENT05_PID> `
  -DurationSeconds 3600 -SampleIntervalSeconds 5 -GpuSnapshot
npm run native-performance:review -- <CAPTURE_JSON>
```

Client05Pid requires Client03Pid and Client04Pid; all five PIDs must be distinct.
Schema 4 retains five separate CPU/RAM readings and identity-loss flags.
An optional single GPU query at the end covers the same five identities.
Without -GpuSnapshot, gpuDiagnostic is explicitly null and no counter query
runs. If ANY selected process loses identity during CPU collection, all GPU
hints remain UNKNOWN and the query is skipped. Creation times are checked
again after an executed GPU query. Missing/localized counters remain UNKNOWN.

The strict reviewer retains slot 01–05 attribution and refuses incomplete
rows, unexpected fields, identity resurrection, contradictory flags or observed
GPU hints after CPU identity loss. It never declares five-client readiness.
GPU values retain the same maximum-instance semantics and stale-memory warning
described below; actual physical VRAM, displayed FPS, process-tree totals and
Education Host World health still require independent evidence. Schema 4
supports individual review; multi-client paired comparisons remain refused.
Schemas 1/2/3 and their result shapes are preserved.

Executed own-data checks: 24 Node reviewer/CLI/source-reader tests pass.
Thirty-second captures of five disposable Node processes pass under Windows
PowerShell 5.1 and PowerShell 7; the latter includes the fifth process exiting
normally and verifies zero GPU queries afterward. Both produce six samples
and BOM-less reports accepted by the actual Node CLI. Six early invalid
configuration controls refuse before creating output directories. These
controls do not launch Minecraft or establish game performance.

### Opt-in two-client memory composition

Use `-MemoryComposition` with the existing two-client command when the
operator session is available and identities have been independently verified.
It does not start or control games. Without the switch, no extended memory API
setup/queries run and schemas 1/2/3/4 retain their existing shapes.

```powershell
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 `
  -NativePid <CLIENT01_PID> -IsolatedPid <CLIENT02_PID> `
  -DurationSeconds 600 -SampleIntervalSeconds 5 -MemoryComposition
npm run native-performance:review -- <CAPTURE_JSON>
```

Schema 5 wraps the existing two-client schema in `baseSchema` (1, or 2 with
`-GpuSnapshot`) and adds separate `memoryComposition` samples. It refuses
clients 03-05 before creating output. The extension uses query-only process
handles, validates creation time before/after the native query, closes handles
on every path, and never requests process-memory contents. It collects:

- **Private commit MiB:** PSAPI PrivateUsage, committed private virtual memory;
  this is not resident RAM or bytes currently stored in the pagefile.
- **Private working-set MiB:** PROCESS_MEMORY_COUNTERS_EX2 when supported;
  failed/unsupported extended queries remain UNKNOWN, while base private
  commit can remain available. This differs from total WorkingSet64.
- **Whole-system physical total/available and commit total/limit MiB:**
  GetPerformanceInfo page counts multiplied by its page size. These include
  other applications and cannot be attributed to Minecraft.
- **Memory-query duration:** elapsed time for the two process queries and
  system query in each row. This excludes setup and is not total collector
  CPU/RAM overhead. Queries are sequential snapshots; their timestamp follows
  that row's CPU/working-set reading, not an atomic joint snapshot.

Unavailable values use null plus a bounded reason; zero remains a legitimate
observation. A lost selected identity invalidates both private metrics for the
remaining capture. Helpers/process trees and the manager are explicitly
NOT_COLLECTED; engine overhead is not isolated by these process counters.
Shared brokers are not guessed into a game total. No metric changes game
graphics, FPS policy, working-set limits, scheduling or admission.

The reviewer reports per-metric coverage, missing reasons, mean/median/peak,
endpoints and endpoint difference. A pair of schema-5 reports supports
descriptive candidate-minus-baseline memory differences only for complete
metrics and matching existing capture conditions. Missing metrics do not
become zero differences. Equivalent graphics, focus policy, host workload,
physical machine and run variability still require separate evidence; no
automatic savings or gameplay verdict is issued.

Executed own-data evidence: 29 Node tests, including Windows PowerShell 5.1
and PowerShell 7 native-reader checks, creation-identity rejection, unavailable
API handling and bounded handle retention over 200 queries. Actual 30-second
captures with two disposable Node processes passed in both shells, six samples
each; the PowerShell 7 run includes normal exit of one target and permanent
identity loss. Both real output files passed the actual Node reviewer and
UTF-8-no-BOM check. This is collector evidence, not Minecraft performance or
engine memory savings. Raw captures remain local.

API semantics:
[GetProcessMemoryInfo](https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getprocessmemoryinfo),
[EX2 and its OS prerequisites](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex2),
[GetPerformanceInfo](https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getperformanceinfo).


### Optional manager and direct-child attribution

Add `-ManagerPid <MANAGER_PID>` and/or `-DirectChildren` to
`-MemoryComposition` to produce **schema 6**. Both options are off by default.
ManagerPid must be distinct from the two selected games and accessible at
baseline. These are operator-selected identities, not verified application
roles. The script never discovers a manager by executable name or starts one.

```powershell
powershell.exe -NoProfile -File .\tooling\acceptance\windows\collect-native-performance.ps1 `
  -NativePid <CLIENT01_PID> -IsolatedPid <CLIENT02_PID> -ManagerPid <MANAGER_PID> `
  -MemoryComposition -DirectChildren -DurationSeconds 600 -SampleIntervalSeconds 5
```

The manager has separate CPU/working-set and private-memory readings with
sticky identity loss. An omitted manager is null/NOT_REQUESTED; its memory is
never silently combined with a game. The CPU interval follows the manager's
own sample endpoints. Schemas 1-5 remain supported with unchanged shapes.

Direct-child attribution uses process-only Toolhelp snapshots, not heaps,
module enumeration, command lines, account/profile reads or process memory
contents. For each row it:

1. Rechecks each root's pinned creation identity, takes a candidate snapshot,
   and pins candidate creation times.
2. Takes a second relation snapshot AFTER those pins, queries candidate
   statistics and rechecks each child and root identity. Children older than
   their alleged current parent or appearing too late to pin are unresolved.
3. Excludes all selected roots and the collector from child totals, so a game
   started by the selected manager is counted once as a selected game.
4. Reports separate native/isolated/manager direct-child counts and private
   memory sums. A failed snapshot, unavailable root, changing relation or
   exceeded 64-candidate bound keeps the affected total UNKNOWN. A verified
   empty inventory is an observed zero. Individual memory-query failures can
   leave commitment available while private residency is unavailable.

This establishes only an observed **direct parent relation**, never exclusive
helper ownership or account isolation. Shared brokers, descendants beyond one
edge, exited-between-samples processes and background services are not a full
tree census. Helpers may serve multiple clients even with one observed parent.
Do not add a shared broker to every client's total or sum working sets as
unique physical RAM. The artifact contains counts/metrics, without PIDs,
creation identities, process names, paths or credentials.

Schema 6 adds per-row `manager` and `directChildren` fields and explicit
selection/manager-loss fields to memoryComposition. The query-duration window
now includes related-process enumeration and queries, still excluding setup
and total collector CPU/RAM cost. The reviewer preserves component coverage
and returns descriptive differences only for compatible modes, unbroken
selected identities and complete individual metrics. Unknown/partial child
totals remain null; no automatic engine-saving verdict is produced.

Own-data proof: 35 Node tests passed, including eight deterministic child
attribution cases in both Windows shells and early refusal before output for
invalid selections. Actual 30-second process-tree captures passed in Windows
PowerShell 5.1 and PowerShell 7, six samples each, including manager exit.
Independent Win32_Process parent metadata matched the native direct-child
counts and demonstrated that grandchildren were excluded. Node also creates
console-host children on this machine; those are valid direct processes,
not assumed extra Minecraft helpers. A first fixture check that assumed one
child was corrected against that independent evidence, rather than weakening
collector attribution. Final-code PowerShell 7 capture and actual Node CLI
single/paired review passed. These checks do not operate Minecraft or prove
resource savings. Raw captures and fixture runners remain local.

Toolhelp contracts:
[process snapshot](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot),
[parent identifier](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/ns-tlhelp32-processentry32w),
[enumeration completion/failure](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-process32nextw).


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

The optional GPU query is **skipped entirely** if any selected game
process became missing or inaccessible during the CPU/RAM capture. In that
case all selected GPU hints stay `UNKNOWN`/null and no new GPU workload is added.
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

Before the timed capture, the collector now resolves the output directory to
an absolute filesystem path and exercises the SAME atomic writer with one
GUID-named probe. It creates, closes and moves that owned file, then removes
only the successful probe. An unusable directory, legacy path-length limit or
move failure refuses before the capture timer starts. Existing files remain
protected by CreateNew and non-replacing File.Move; cleanup tracks staging
ownership so a failed CreateNew cannot delete a pre-existing path.

This follows an observed 900-second session whose final staging pathname was
273 characters: Windows PowerShell 5.1/.NET refused File.Open even though the
output directory existed. That attempt has no persisted measurement and must
not be credited as telemetry proof. Prefer a short output directory; the
preflight does not enable OS-wide long-path settings, guarantee future free
space or recover data from a later process/filesystem failure. PowerShell 7
may support a path that legacy .NET refuses.

The regression exercises the shared writer in both shells, preserves an
existing artifact on collision, checks owned staging cleanup and, when the
runtime rejects the long path, verifies the actual collector refuses before
timed sampling without leaving a partial report. All 36 reviewer/Windows
own-data tests pass. This is a diagnostic-output fix; game settings, profiles,
pacing, admission and public schemas are unchanged.

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

`npm run test:native-performance` checks schema-1/2/3/4/5/6, memory and GPU hint handling,
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
G0–G4 or proves account separation. Minecraft performance acceptance is still **NOT_EXECUTED** for this change;
the executed own-data Windows collector checks are scoped above.

## Historical VMware-only G0–G4 campaign (paused)

The stages and procedures below document the retained Windows-guest
acceptance implementation. They are **not** the active startup/installation
workflow and do not authorize changing account-bearing guests. The old NSIS
release is deliberately blocked. Do not create another test gate scheme.

This campaign is evidence collection, not simulation. A stage stays `NOT_RUN` until its target-machine action has actually been performed. **The only five product-test progress gates are G0–G4 in [Acceptance](../../../../docs/validation/acceptance.md)**; the existing stage IDs and recorder schema stay unchanged. Stage records are supporting evidence, not twelve independent release milestones.

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

The [Product Goal](../../../../docs/product/four-concurrent-accounts.md) and [One Guest Feasibility Gate](../../../../docs/validation/client-feasibility.md) specify evidence and stop criteria. The guest collector cannot observe account sign-in or game rendering directly: its operator declarations are **not automatically verified**. Do not record PASS until actual operator/game evidence has been reviewed.

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

Canonical guide: [Client Feasibility](../../../../docs/validation/client-feasibility.md). A scratch-guest failure stops further cloning and account setup until graphics/driver/VM compatibility is understood.

G0 has no Windows evidence-recorder stage: it is derived from executed source, package, payload and disposable installer-safety checks in [G0 Readiness](../../../../docs/validation/pre-local-test-readiness.md). `HOST` establishes a baseline; it does not prove G1 on its own. `SOAK` is supporting evidence and cannot replace G4's recovery and multiplayer observations.

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




## Explicit one-shot LFH reclamation experiment

`heap-reclaim.cpp` builds a standalone x64 action DLL and host. It requests
`HeapSetInformation(NULL, HeapOptimizeResources, ...)` once in the selected
Minecraft process, outside DllMain/Present. It does not hook game APIs, inspect
game allocations, trim working sets, change profiles/settings, install anything,
or register an automatic policy. This is an experimental allocator action,
not a Sodium-style renderer optimizer or a production RAM feature.

Build from an MSVC x64 developer environment, in a separate output directory
with a copy of this one source file:

~~~bat
cl /nologo /std:c++17 /W4 /WX /permissive- /EHsc /MT /O2 /DUNICODE /D_UNICODE /DWIN32_LEAN_AND_MEAN /DNOMINMAX /D_WIN32_WINNT=0x0A00 /DHEAP_RECLAIM_DLL /LD heap-reclaim.cpp /Fe:heap-reclaim.dll /Fo:heap-reclaim-dll.obj /link psapi.lib
cl /nologo /std:c++17 /W4 /WX /permissive- /EHsc /MT /O2 /DUNICODE /D_UNICODE /DWIN32_LEAN_AND_MEAN /DNOMINMAX /D_WIN32_WINNT=0x0A00 heap-reclaim.cpp /Fe:heap-reclaim.exe /Fo:heap-reclaim-host.obj /link psapi.lib
heap-reclaim.exe --self-test C:\compiled\heap-reclaim.dll
~~~

The self-check exercises the actual DLL action on the caller's heap, rejects an
invalid request and replay, and verifies a live allocation remains intact.
It never treats successful API return or a synthetic byte count as Minecraft
memory savings.

Only after explicit runtime authorization and fresh PID/creation verification:

~~~text
heap-reclaim.exe --process <PID> <creation-FILETIME-decimal> C:\compiled\heap-reclaim.dll
~~~

Pin the newly built source/DLL/host hashes before invoking. The host refuses
a mismatched process birth, a 32-bit process, an unexpected Education executable
path, a reparse-point DLL leaf, or an already-loaded action DLL. It holds the
action file against write/delete, resolves remote system functions by their
actual module/RVA, and writes/reads only its own request allocation. It unloads
the action after completion. If a remote thread times out, it leaves its arguments
and module alive rather than terminating a game/thread or freeing live arguments;
exit 10 is UNKNOWN and must not trigger a blind retry.

Output contains API status, main process-heap compatibility (not all heap coverage),
private commit immediately around the call and elapsed time. Check post-action
unload and actual game state. Private commit during concurrent game activity can
change independently; working-set drops are not attributed automatically.
No scheduled repetition or automatic focus trigger is justified by this tool.
The source currently targets the observed official Program Files Education path;
do not broaden target admission incidentally.
