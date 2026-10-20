# Isolation Source Tooling — Active Route (v1.18.6)

This README is the **single operator entrypoint for native isolation source
verification/build**. The desktop has an explicit developer owned-runtime adapter;
these build commands do not sign, install, load or select an engine. A source
build is separate from the provider currently active on Windows.

## Exactly one current source/build path

| Role | Canonical owner | Allowed use |
| --- | --- | --- |
| Unchanged comparison source | `engine/isolation-candidates/sandboxie-v1.18.6/` | Read-only byte/tree reference (1,627 files) |
| Editable native source | `engine/isolation-owned/` | One owned v1.18.6 tree (1,627 files; 15 C/C++ changes plus four exact build-project path fixes) |
| Build component contract | `engine/isolation-native/owned-runtime/build-profile-v1186.json` | Exactly 3 stages, 16 native binaries, 5 resources and 4 original licenses |
| Source verification | `tooling/isolation-source/verify-owned-v1186.mjs` | Read-only exact-source and upstream provenance check |
| Windows build (optional, build-only) | `tooling/isolation-source/build-owned-v1186.ps1` | Compile only; 3 MSBuild stages, WDK required |
| Staging (optional, NOT-INSTALLABLE) | `tooling/isolation-source/assemble-owned-v1186.mjs` | One immutable payload and provenance manifest |
| Desktop inspector | `apps/desktop/src-tauri/src/engine/owned_isolation.rs` | Read-only package inspection; cannot activate the provider |

**Read-only source check:**

```powershell
npm run isolation-source:verify
```

**Optional native Windows compiler build** (not required for first UI
smoke or normal `DEV.cmd dev`, and not proof of working Minecraft):

```powershell
# From the repository root; use a new report/package name each run.
New-Item -ItemType Directory -Force .\workspace\builds, .\workspace\packages | Out-Null
$runId = Get-Date -Format 'yyyyMMdd-HHmmss'
$report = Join-Path (Get-Location) "workspace\builds\owned-$runId"
$payload = Join-Path (Get-Location) "workspace\packages\owned-$runId"
.\tooling\isolation-source\build-owned-v1186.ps1 -ReportDirectory $report -PackageDirectory $payload
```

Never reuse an existing report or package directory. The builder and
assembler refuse paths outside `workspace/builds/` and
`workspace/packages/`, respectively, and enforce clean source,
bounded artifact provenance and reparse-point checks. Neither command
installs or starts a driver, replaces official Sandboxie, creates boxes,
changes accounts or proves multiplayer.

## Current Windows build evidence — 20 October 2026

The owned-engine campaign prioritizes build/signing before switching the desktop.
At `f529d59fda778be50cd3e874374a5546c32dce29`, all three actual Windows compiler
stages pass and all 16 binaries plus five resources/four licenses stage successfully.
MFC v143, WDK 26100.6584 and its VS2022 integration are installed after operator
authorization. The package is **MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE**.
Production signing, activation, owned runtime operations and gameplay remain
unexecuted; no driver was installed and no security policy changed.

The builder explicitly disables WDK's default test signing and checks the driver
is actually `NotSigned`. An earlier implicit test-signed attempt was rejected and
its generated temporary certificate/private key removed. The final clean unsigned
candidate is `workspace/packages/owned-20261020-150022/`, with its source/hash-bound
report in `workspace/builds/owned-20261020-150022/`. Resources normalize checkout
CRLF only if the result matches exact committed Git blob bytes; edited configs
or licenses remain rejected. Preserve the successful candidate; do not install it.

Four reviewed project changes quote MASM, import-library and message-compiler
paths, including checkout paths containing spaces. The source verifier reconstructs
these exact transformations from pristine upstream separately from the original eleven
C/C++ security/identity modifications. Unexpected edits still fail verification.
Standalone C/C++ policies pass actual MSVC compilation and all three CTest tests:

```powershell
npm run isolation-native:test-owned-start
```

That command does not compile or load the real kernel driver. Full campaign
results and remaining work live in the existing
[Windows validation procedure](../../docs/validation/native-pilot-localtest.md).

## Explicit real-payload desktop inspection

The desktop's existing Rust inspector can be exercised against a selected real
v1.18.6 payload, without opening the app, loading a DLL/driver or changing services:

```powershell
$env:MIVUBI_TEST_OWNED_PAYLOAD = (Resolve-Path 'workspace/packages/owned-20261020-150022').Path
try { npm run isolation-source:inspect-candidate }
finally { Remove-Item Env:MIVUBI_TEST_OWNED_PAYLOAD -ErrorAction SilentlyContinue }
```

Select an existing absolute payload directory from the matching build report;
the example is the completed unsigned candidate, not an automatic latest selector.
The command runs one opt-in Rust test, checks all 25 declared artifacts with the
production inspector, and observes the current SCM namespace using its existing
read-only probe. It asserts provider selection, driver-load/gameplay proof and
installation/activation remain false. Printed service-name registration is not
service-owner authentication. A successful checksum inspection never grants
permission to replace an occupied namespace or load an unsigned driver.

The real-payload test is ignored by default because source verification must not
depend on machine-local build artifacts; explicit invocation fails if the variable
is missing or the payload is invalid. This is developer verification, not a new
installed-app CLI, IPC schema, runtime provider or readiness database.

## Local developer test preparation — v1.18.6 only

`windows/prepare-owned-v1186-test.ps1` prepares a **separate test-signed copy**
of an existing successful build. It never installs, starts, stops or deletes a
service, launches Minecraft, reboots Windows, or selects a production provider.
Use Windows PowerShell elevated through normal UAC, on branch `Local`:

```powershell
$session = Join-Path (Get-Location) 'workspace/owned-tests/my-new-session'
./tooling/isolation-source/windows/prepare-owned-v1186-test.ps1 -Stage Prepare `
  -SessionDirectory $session `
  -CandidateDirectory 'workspace/packages/owned-20261020-150022' `
  -BuildReport 'workspace/builds/owned-20261020-150022/build-result.json'
./tooling/isolation-source/windows/prepare-owned-v1186-test.ps1 -Stage EnableTestMode -SessionDirectory $session
```

Choose a **fresh** session directory. The preparation checks 25 artifact hashes,
the matching build report and committed source tree, snapshots the existing
official service registrations, and signs only the copied 16 binaries using a
new, nonexportable, 90-day local developer key. Public certificate trust is
installed in LocalMachine Root and TrustedPublisher, with exact thumbprint and
store tracking in ignored `session.json`. The unsigned original is untouched.
Receipts and registry exports remain local; never commit signing material,
machine configuration or private profiles. Preparation failures preserve partial
evidence rather than guessing that cleanup succeeded.

Boot preparation refuses Secure Boot, running HVCI, enabled BitLocker protection,
unknown boot entry/value, or changed signed artifacts; it never disables these
protections. `EnableTestMode` records `REBOOT_REQUIRED_NO_SERVICE_CHANGES` after
readback. **Save work and reboot manually.** This checkpoint is not driver-load
or gameplay evidence. `Inspect` prints the preparation receipt, not live driver
attestation. Service takeover, all-session process-idle proof, ABI checks and
owned Minecraft launch still require the subsequent reviewed activation step;
the historical v1.18.5 Apply transport stays sealed.

To cancel preparation **before any service takeover**, run elevated:

```powershell
./tooling/isolation-source/windows/prepare-owned-v1186-test.ps1 -Stage RestoreTestMode -SessionDirectory $session
```

This restores the recorded boot setting and removes only the recorded MIVUBI
test certificate/trust/key; manually reboot to apply boot changes. It does not
restore services or delete profiles, worlds, payloads or evidence. Do not use
this cancellation as rollback after installing a driver. Test signing is for
local development, never a production release or account-isolation guarantee.
See [Microsoft test-signing boot configuration](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/the-testsigning-boot-configuration-option).

## Explicit v1.18.6 runtime test and service restoration

After the manual reboot, use elevated Windows PowerShell and the **same session**:

```powershell
./tooling/isolation-source/windows/test-owned-v1186-runtime.ps1 -Stage Inspect -SessionDirectory $session
./tooling/isolation-source/windows/test-owned-v1186-runtime.ps1 -Stage Activate -SessionDirectory $session
# Close any test clients normally before restoring:
./tooling/isolation-source/windows/test-owned-v1186-runtime.ps1 -Stage Restore -SessionDirectory $session
```

This opt-in test route preserves the existing SCM registrations, startup type,
account, ACL, dependencies and minifilter configuration. It authenticates the
original paths/hashes/signers against preparation, verifies altitude 86900, then
uses the upstream KmdUtil stop/start operations with bounded waits. SCM
`ChangeServiceConfig` changes **only ImagePath**, with original images recorded
in the existing session receipt before changes. No service is deleted/recreated;
the old v1.18.5 installer is not activated. Inspect and activation use a separate,
short-lived signed-DLL probe, release its handles on exit, and query all sessions
with a null box filter. Actual code-integrity Test Mode and a reboot after
preparation are required. Any active/unknown sandbox inventory blocks takeover.

Activation verifies `5.73.6-MIVU` and ABI `0x57230` through the actual loaded
driver before reporting `OWNED_TEST_DRIVER_ACTIVE_NO_GAME`. It does not create a
profile, launch Minecraft, or enable the desktop owned provider. A named mutex
and receipt file lock serialize operations. Partial failure preserves the first
error and journal; inspect before retry, and run Restore if ownership is proven.
Restore refuses foreign images or active sandbox processes, restores original
ImagePath values, starts the original services and verifies their driver identity.
It leaves profiles/worlds/configuration, payload and test signing untouched.
Only after restoration may the earlier `RestoreTestMode` cancellation remove the
certificate and restore the boot setting, followed by another manual reboot.

Exit the official Sandboxie Plus/Control manager **through its tray Exit menu**
before activation. An open manager can prevent driver unloading despite an empty
sandbox-process inventory. Activation now refuses a running SandMan/SbieCtrl
before stopping any service. A failure before ImagePath redirection restores
the original user service without retrying the failed original-driver unload.
Completed restoration permits an explicit activation retry; previous attempts
remain in `runtimeHistory` inside the same local receipt.

For the first owned client only, after private driver activation passes:

```powershell
# Elevated: refuses existing MivubiEdu02 or any previous partial creation.
./tooling/isolation-source/windows/test-owned-v1186-runtime.ps1 -Stage ConfigureClient02 -SessionDirectory $session
# In a NORMAL, UNELEVATED PowerShell session:
./tooling/isolation-source/windows/test-owned-v1186-runtime.ps1 -Stage LaunchClient02 -SessionDirectory $session `
  -GameExecutable 'C:\Program Files\Microsoft Studios\Minecraft Education\Minecraft.Windows.exe'
```

The adapter consumes the existing `minecraft-profiles.mjs` owner, disables the
new section first, and requires a matching query after every write, enabling it
last. Existing/partial profiles are preserved and never overwritten or deleted.
The explicitly approved App Compartment/OpenCOM compatibility mode is not an
account-isolation guarantee. Launch rechecks actual owned driver/service identity,
all-session idle and every final profile setting; it accepts only the observed
Microsoft-signed official desktop executable and uses the private `/mivubi_client:02`
entrypoint. An accepted request is not proof of a running or playable game.
Sign in manually; no credentials, existing account state or tokens are copied.
The default official pilot and production provider are not promoted by these
commands. The debug manager now has an explicit local-test selection described
below; these CLI commands remain a separate operator entry into the same engine.

## Four-slot manager integration — developer mode

Run the actual Tauri development app. Startup detects the current owned service
payload and registered Education installation, authenticates them through the
existing provider, and remembers verified locations in the existing manager data
directory. On this activated test host, use **Clients → Open**; no folder picker
or repeated licensing checkbox is needed. Choosing Open confirms use of a separate
licensed account. Settings reports automatic detection; technical locations are
read-only under Diagnostics. The ordinary UI has no file/folder chooser.

The canonical catalog is `engine/isolation-native/owned-runtime/minecraft-targets.json`.
Automatic discovery never adopts official Sandboxie or an offline package. Local
test operations remain rejected in production builds. The backend independently
authenticates actual service paths, all 25 hashes/signatures, source manifest/resources,
private driver/ABI, kernel Test Mode and current Windows session on every operation.
Saved locations are preferences, not a new readiness flag or lifecycle authority.

Client 01 retains the registered official Windows launch route. The owned mode
attributes its native process through stable image/start-time identity and two
explicit not-boxed API results, without classifying unknown failures as native.
Clients 02–04 use canonical MivubiEdu profiles. The prepared Client 02 is adopted
only through its existing creation receipt plus fresh settings/provider evidence.
New profiles refuse existing sections, partial attempts and orphan default
data/hive roots; custom global storage roots require review. First Open on an absent profile asks compatibility consent, then prepares and opens
through the same existing operations. Cancellation creates nothing. Compatibility
consent acknowledges reduced containment, not account isolation.

Background PowerShell calls use the canonical bounded runner with CREATE_NO_WINDOW.
The game launcher uses direct ProcessStartInfo without ShellExecute, and observes
an attributed window before RUNNING. The opt-in test
`engine::owned_provider::launch_regression_tests::actual_manager_launch_observes_owned_minecraft`
is an actual launch, requires MIVUBI_TEST_OWNED_SESSION, and must not be run while
a client is active. It never closes the game or signs in automatically.

Profile creation is the only operation that requests UAC. It invokes this same
application's fixed Create-only helper with a bounded local request, never an
elevated mutable checkout script or a new background service. Current Windows
PowerShell modules are imported from the system PSHOME rather than inherited
module search paths. The same global runtime and per-session locks serialize
CLI, manager and elevated child operations. No driver/service/boot operations are
exposed through this adapter. Sign-in remains manual.

Open records a request and observes actual processes with a deadline. No automatic
launch retry is issued. A live previous launcher, nonempty profile, mismatched
PID/start time/session/HWND or provider blocks operations. An inconclusive launch
requires Refresh and an explicit guarded retry; the cooldown remains enforced
across app restarts through the existing session receipt. Focus/Place validate
the target immediately before Windows calls. Close sends WM_CLOSE only and does
not claim STOPPED until a subsequent inventory proves it; save/confirm inside
Minecraft when necessary. UI inspections are on demand, with no new polling loop.

The opt-in Rust test `engine::owned_provider::tests::actual_owned_session_inspects_three_slots`
accepts `MIVUBI_TEST_OWNED_SESSION` and verifies the actual selected backend read-only.
Default source tests never require an installed driver or game. Four-client UI,
manual account separation, multiplayer and soak results belong only to the
canonical [acceptance report](../../docs/validation/native-pilot-localtest.md).

For a selected actual Education PID, `probe-v1186-runtime.ps1 -DllPath <verified
owned SbieDll.dll> -Sha256 <its recorded SHA-256> -GameProcessId <PID>` adds a
read-only attribution observation. It verifies the official Microsoft-signed
game path, current private driver/ABI, twice-read MivubiEdu02 membership and
Windows session, and process start time before/after the query. PID reuse or
identity loss fails closed. Window presence/responsiveness is diagnostic;
manual sign-in, playable world, independent input and multiplayer require
separately attributed operator observations. Output excludes account/SID/title
data and remains local unless sanitized for the canonical acceptance report.

## Not active — historical implementations

The pinned v1.18.5 Git submodule
`engine/isolation-upstream/Sandboxie`, v1.18.5 fork overlay,
`bootstrap-fork.mjs`, `assemble-private-engine.mjs`,
`build-private.ps1`, source migration and disposable service/driver
prototypes remain **historical research/test fixtures only**.
They are **not fallback build commands** for the current v1.18.6 engine.
Do not remove these sources indiscriminately: regressions, source
provenance and licenses still refer to them.

Archived investigation details:
[legacy v1.18.5 isolation notes](../../planning/history/isolation-v1185-source-notes.md);
[native overlay history](../../planning/history/isolation-native-development-notes.md).
See the [source ownership map](../../docs/system/implementation-map.md)
and [active Windows localtest](../../docs/validation/native-pilot-localtest.md)
before selecting a command.
