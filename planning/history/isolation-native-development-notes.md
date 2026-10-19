# Archived native-isolation development notes

> **HISTORICAL / NOT ACTIVE EXECUTION INSTRUCTIONS.**
> Preserves the 15 October source research from the former
> `engine/isolation-native/README.md`. Its v1.18.5 overlay commands
> and external D: checkout examples must not be followed for current Local
> development. The current owner is
> [v1.18.6 source pipeline](../../tooling/isolation-source/README.md).

# MIVUBI native isolation ABI probe — source-first

## Current v1.18.6 compile-only path

The real editable `engine/isolation-owned/` source is now eligible for **build-only** review through `tooling/isolation-source/build-owned-v1186.ps1`. The script requires exact v1.18.6 Git/source identity, clean inputs and fresh output paths, then compiles Win32 support, x64 core and x64 WDK driver to a report without signing, installation or activation. No Windows build has been executed yet. The official installed Sandboxie service/driver and account-bearing Client 02 remain the only active pilot.



## Private SbieIni update status fix — 15 October 2026

The pinned upstream `Sandboxie/apps/ini/update.c` returns exit code zero
for non-password configuration failures, even when the underlying update
API returns an error. The private fork now returns **3** for such failures,
preserving **0** for success, **1** for malformed commands and **2** for
wrong passwords. The canonical source overlay now modifies 11 upstream files
(24 patch hunks), including the actual updater compiled into `SbieIni.exe`.
The Rust private candidate inspector uses the same 11-file source closure.

This fixes error signaling needed by persistent `MivubiEdu02/03/04` box
provisioning; it does not prove settings are persisted until a Windows
runtime read-back test is performed.


## Cold-start native Start.exe broker bootstrap — 15 October 2026

The private `/mivubi_client:02/03/04` mode now invokes the existing
Sandboxie `CallSbieSvcGetUser` broker startup/wait path before its
private `SbieApi_GetVersionEx` handshake. Previously the driver
identity query ran before `Validate_Box_Name` could start the broker;
on a clean cold boot, that ordering could reject every owned launch
with `STATUS_SERVER_DISABLED`. The identity check remains mandatory
after service startup and still rejects official/foreign drivers.

This is a source-level functional first-launch fix. It does not bypass
service signing, install a driver, create sandbox configuration profiles,
or prove that the full Windows private core runs.


This is product-specific **Windows-native source** for the prospective M-Isolation-Engine.
It consumes the original Sandboxie-Plus v1.18.5 `Sandboxie/common/my_version.h`
from pinned upstream commit `abd4c240031cfb46666daa74ca47ed4e48b561e4`.
Driver ABI: `0x57230`; driver version: `5.73.6`.

## MIVUBI kernel-enforced sandbox namespace — 15 October 2026

The private fork now modifies `Sandboxie/core/drv/box.c` itself and
also checks both process-start paths and the box-path query in
`Sandboxie/core/drv/process_api.c`.
For model-PID launches it rejects foreign or missing model boxes before
`Box_Clone`, releasing the process-list lock and lowering IRQL first.
For named launches it denies foreign names after safe copying but BEFORE
service-token impersonation or BOX allocation. Rejected owned-engine box
requests return `STATUS_ACCESS_DENIED` instead of a misleading resource error.

The private fork now modifies `Sandboxie/core/drv/box.c` itself.
Immediately before `Box_Alloc` allocates an actual kernel `BOX`
object, its C policy `mivubi_kernel_box_policy.h` allows only
`MivubiEdu02`, `MivubiEdu03`, `MivubiEdu04`
(case-insensitive ASCII) and returns `STATUS_ACCESS_DENIED` for
all other names. The restriction also applies when sandbox contexts
are cloned for descendant processes, since both paths use
`Box_Alloc`. Official upstream builds are untouched. The private
fork closure now has ten source files (22 Git hunks).

The **actual header source** was compiled and tested in 35 native C
accept/reject cases: GCC and Clang, Release and ASan/UBSan, all passed.
The nine pinned upstream C/C++ source files were passed through the
fork's exact-content source transform in ChatGPT V8; nine targeted
source regression bodies passed. This DOES NOT yet establish a
loadable Windows `SbieDrv.sys`, actual box provisioning, or licensed
Minecraft sign-in. Because `SbieSvc`/`SbieDrv` names remain exclusive,
never install this private fork alongside the official account-bearing
Sandboxie installation.

## Windows CRLF private checkout compatibility — 15 October 2026

**Concrete Windows-source fix:** the upstream Sandboxie .gitattributes
marks .c/.cpp files as Git text but does not force LF in those source
working-tree paths. On Windows with Git `core.autocrlf=true`, a clean
private checkout can contain CRLF lines. Earlier
`buildPrivateSourcePatch` unconditionally rejected CRLF, so private
fork source application could stop before MSBuild even begins.

The canonical source owner now accepts consistent LF or CRLF source,
normalizes the *in-memory patch input* to LF, and leaves both the actual
checkout and the user's Git configuration untouched. Mixed/lone-CR
source fails closed. `git apply --index` preserves checkout EOL through
Git's attributes conversion; this was verified on a disposable Git repo
configured for Windows CRLF, with check+index apply both succeeding.
The exact eight Sandboxie v1.18.5 GitHub upstream sources produced
byte-identical LF/CRLF patches (16 hunks), and the committed expected
source verifier preserves CRLF if its input is CRLF.

**Evidence boundary:** candidate pure source regressions passed in
ChatGPT V8; a new checked-in disposable-Git integration test exercises
all eight fixture files under `core.autocrlf=true`. This is not
execution of the pinned Node24 runner, Windows MSBuild, a signed kernel
driver, or Minecraft gameplay.

## Private-exclusive MIVUBI source fork (source-first)

The **first planned private runtime** preserves the pinned upstream's
`SbieDrv`, `SbieSvc`, `SbieDll`, IPC object names and minifilter altitude.
This is intentional: the first MIVUBI distribution is **exclusive**, not
coinstallable with official Sandboxie. Independent deployment is a different
target from simultaneous namespace coexistence.

The source transformation under [fork-overlay](../../engine/isolation-native/fork-overlay/apply-private-source.mjs)
uses a single exact-context Git patch against eight upstream source owners:
the common version/ABI header plus driver API, DLL API, service driver assist,
launcher, the separate asynchronous DriverAssistStart owner, SbieIni and KmdUtil. It installs compile-time private-build/ABI guards
**and a functional, opt-in MIVUBI Client 02–04 launch mode inside Start.exe**,
but **does not modify upstream copyright, licensed attribution, kernel service
names, device/IPC names or signing requirements**. It only accepts a separate
clean full-history checkout descending from pinned v1.18.5, not the application
submodule or official installation.

~~~powershell
# From this application's root, after separately bootstrapping source:
node engine/isolation-native/fork-overlay/apply-private-source.mjs --fork-root 'D:\Source\M-Isolation-Engine'
# The overlay is STAGED. Review all modified native paths, then commit
# the standalone fork in a linear branch descending from pinned v1.18.5:
git -C 'D:\Source\M-Isolation-Engine' diff --cached --check
git -C 'D:\Source\M-Isolation-Engine' diff --cached --stat
git -C 'D:\Source\M-Isolation-Engine' commit -m "feat: MIVUBI controlled three-client launch"
node engine/isolation-native/fork-overlay/apply-private-source.mjs --verify-committed --fork-root 'D:\Source\M-Isolation-Engine'

# Produces private binaries and source-specific hashes; does NOT install:
.\engine\isolation-native\fork-overlay\build-private.ps1 -ForkRoot 'D:\Source\M-Isolation-Engine' -ReportPath '.\workspace\isolation-private-build.json'

# Build native ABI probe against the separately patched source:
cmake -S engine/isolation-native -B workspace/isolation-native-private -G "Visual Studio 17 2022" -A x64 -DMIVUBI_PRIVATE_FORK_ROOT="D:/Source/M-Isolation-Engine"
cmake --build workspace/isolation-native-private --config Release
~~~

The source patch stages eight modified files (14 Git hunks) in the fork Git index for review. The
private build script only permits these exact source modifications and uses
upstream's Win32 -> x64 -> kernel build order. It does not sign, install,
load, stop, launch or migrate anything. A private source build is **not**
an unmodified upstream build and its report is deliberately separate.
These Windows scripts are authored source, **not executed** here.

No proprietary/private credentials, world files or copied original Sandboxie
binary builds are committed to `Local`.

## Hand off a private core build to the existing package owner

`build-private.ps1` now accepts optional `-StageOutput`. Once Win32,
x64 and kernel source compilation succeeds it invokes the sole private payload
assembler, which verifies the private binary hashes and puts both architectures,
templates, and upstream licenses in an exclusive `PRIVATE_CORE_STAGED_NOT_INSTALLABLE`
bundle. It is NOT safe to co-install with the official Sandboxie driver/service.
The app's existing candidate inspector can read this bundle but cannot activate
it. See [private package source](../../tooling/isolation-source/assemble-private-engine.mjs).

## Build source

After initializing the `engine/isolation-upstream/Sandboxie` Git submodule,
these Windows Visual Studio 2022 x64 steps build the standalone native program:

~~~powershell
cmake -S engine/isolation-native -B workspace/isolation-native-build -G "Visual Studio 17 2022" -A x64
cmake --build workspace/isolation-native-build --config Release
ctest --test-dir workspace/isolation-native-build -C Release --output-on-failure
~~~

CMake verifies upstream HEAD and rejects another revision. No actual Windows
compilation or CTest is claimed for this GitHub-source batch.

## Three explicit commands

- `--selftest` checks header ABI/version without accessing Windows services.
- `--offline D:\Source\M-Isolation-Payload` checks core binary layout and the
  `NOT-INSTALLABLE.txt` marker without loading a DLL or querying SCM.
- `--abi D:\Source\M-Isolation-Payload` is a **lab-only, manually invoked**
  runtime query. It demands exact `SbieDrv`/`SbieSvc` service image paths
  within the candidate, then validates Authenticode for the driver, service
  and DLL before loading that DLL by full path to call
  `SbieApi_GetVersionEx(WCHAR[16], ULONG*)`.

`--abi` runs upstream DLL initialization and queries the live driver.
**Never run it on the account-bearing host or next to official Sandboxie.**
The path/signature checks do NOT attest signed-kernel load acceptance,
ownership, a disposable Windows machine, isolation or Minecraft gameplay.
Its JSON always sets `driverSigningPolicyAccepted=false`,
`installerReady=false`, `providerSelected=false`, and
`gameplayVerified=false`. No Tauri command invokes it.

The utility does not rename `SbieDrv`, `SbieSvc`, IPC names, binaries,
or inject hooks into Minecraft. Renaming files alone cannot create an
independent fork. The complete native driver/service/DLL remains sourced
from upstream, with licenses and attributions preserved.

## Required committed-source identity before private build (2026-10-14)

Patch preparation intentionally stages seven files in a separate full-history
checkout for human review. **A staged patch with HEAD still at upstream is not
a private Git commit.** Review the staged changes and commit exactly those seven
files on `isolation/baseline-v1.18.5`. The source history must descend linearly from the
pinned v1.18.5 source, with a clean index and worktree. Additional source fixes may be committed normally, but merge commits and intermediate changes outside the ten authorized native files are rejected. Final source bytes must still match the canonical fork overlay. Then verify:

~~~powershell
node engine/isolation-native/fork-overlay/apply-private-source.mjs --verify-committed --fork-root 'D:\Source\M-Isolation-Engine'
~~~

The verifier reconstructs the exact authorized source insertions from original
upstream Git blobs and refuses any extra changes. It returns
`privateForkHead`, `privateForkTree` and `privatePatchSha256`; these must
match the Windows build report, private payload and Rust read-only inspector.
`build-private.ps1` checks the identity both before and after compilation.

Source hashes ensure traceability, **not** repeatable binary compilation,
Authenticode trust, driver load approval or Minecraft gameplay.

## ABI probe CMake source admission (2026-10-14)

For the private-fork option, CMake invokes the **same canonical**
`apply-private-source.mjs --verify-committed` verifier used by
`build-private.ps1` and `assemble-private-engine.mjs`. A Git descendant with
only matching source-header markers is no longer accepted: it must contain
exactly the committed, reviewed seven-file patch, with a clean worktree and
no additional changes. Node must be available at CMake configure time.
This check is source admission only, not driver/API compatibility proof.

No CMake/MSVC execution is claimed from the ChatGPT/GitHub source review.
The unmodified pinned submodule requires exact pinned HEAD and a clean
tracked/untracked source checkout. It is not falsely treated as a committed
MIVUBI fork.

## Corrected patch application preflight (2026-10-14)

The initial `apply-private-source.mjs` unified-diff generator included only
the insertion anchor as context. This generated syntactically plausible hunks
that Git could reject with `patch does not apply`, even against the exact
expected file contents. The source owner now includes the **following
original source line as trailing context** and adjusts old/new hunk lengths
to two original context lines. A clean Windows CRLF working tree is now accepted and normalized in
memory into an LF Git patch. Mixed EOL or lone-CR input is still refused. The existing disposable-Git regression does real
`git apply --check --index --whitespace=error` and `git apply --index`
against its fixture checkout.

The regression was additionally reproduced and corrected with disposable
Git on the ChatGPT execution container. Seven fixture hunks now apply
atomically in the source checkout; no real Windows binary build, driver
load or account data was involved. Real upstream patch application remains
an additional integration obligation on the exact v1.18.5 source checkout.

## Private SbieDll atomic verified-handle state — 15 October 2026

**Implemented source change:** private SbieDll now keeps its
verified-device-handle cache in a `PVOID volatile` variable. It uses
`InterlockedCompareExchangePointer` to read and
`InterlockedExchangePointer` to approve, close, or invalidate a handle.
These operations remove ordinary unsynchronized accesses to this shared
cache across threads. The existing driver identity and ioctl ABI are
unchanged.

**Evidence:** eight authored pure source regression bodies passed against
the candidate in ChatGPT V8. Eight actual pinned upstream files generated
16 nonoverlapping Git hunks and passed exact expected-byte comparison.
The pointer Interlocked type-contract compiled with `clang-cl /TC /W4
/WX /c` to a Windows COFF object. Actual full MSVC/WDK Sandboxie
DLL/service/driver builds remain NOT_EXECUTED.

**Important limitation:** the atomics synchronize this private *cache
variable*, not the pre-existing upstream
`SbieApi_DeviceHandle` open/close lifecycle. In-flight ioctl versus
close/reopen races require actual Windows testing and may need a
broader core-lifecycle correction. This is not complete driver
authentication or proof of four Minecraft clients.

## Native Start policy Release-test integrity — 15 October 2026

**Confirmed failure:** the native `mivubi_owned_start_policy.test.cpp`
previously relied on C/C++ `assert()`. In Release builds, `NDEBUG`
disables those checks. I reproduced this with an intentionally incorrect
expected result: the old Release test exited **0 (false PASS)**.
The repaired test uses a small always-active `require()` helper that
aborts on failure. The exact same deliberately incorrect expectation
now exits with a nonzero status. Correct test source has 26 cases,
including short and truncated `/mivubi_client:` input.

The C++ parser was also refactored into sequential character checks
for easier review. A sanitizer run of the original combined expression
did **not** reproduce an out-of-bounds access: its boolean operators
already short-circuited on a missing required character. This parser
refactoring is **not** presented as a confirmed memory-safety fix.

**Executed:** the *exact committed candidate blobs* for the modified
header and C++ test were independently matched against `git hash-object`
in the ChatGPT container. GCC and Clang each passed the 26-case test
in Release with `-DNDEBUG` and under AddressSanitizer/UBSan.
Both CMake Release builds passed CTest, and the modified policy was
embedded and expected-byte verified against all eight pinned Sandboxie
source files (16 nonoverlapping patch hunks).

**Not executed:** native MSVC/WDK Windows DLL, service, driver, GUI,
real isolation, signing or Minecraft multi-account gameplay.

## Bounded native driver-version comparison — 15 October 2026

The shared `mivubi_core_identity.h` now compares the private identity
against the **explicit capacity** of the `WCHAR[16]` version response.
The previous `wcscmp` could read beyond an unterminated or malformed
foreign-driver version buffer. The replacement compares the literal
version marker *including its NUL terminator* only after confirming the
buffer is large enough. `SbieApi_GetVersionEx`, its direct-handle
verification path, and the owned `Start.exe` launcher pass the precise
16-WCHAR capacity. The same code is compiled into the private kernel API
and userspace source, without adding an ioctl or altering ABI.

**Execution:** GCC and Clang each compiled and executed C11 bounded-identity
tests with AddressSanitizer and UndefinedBehaviorSanitizer enabled,
including short nonterminated buffers, a full 16-WCHAR nonterminated
response, invalid ABI, missing input and truncated capacity. The actual
eight-file pinned Sandboxie overlay passed 16-hunk exact-source checks,
and seven existing authored JavaScript source-test bodies passed in
ChatGPT V8. This is NOT Windows MSVC/WDK kernel/driver execution.
Version matching is non-cryptographic and does not establish safe
kernel-driver authenticity.

## Private DLL ioctl admission and service failure shutdown — 15 October 2026

**New functional core work:** the private SbieDll now verifies that the
*opened device handle* belongs to the MIVUBI kernel before its first
non-`API_GET_VERSION` ioctl. It sends the existing `API_GET_VERSION` ioctl
directly on that handle using upstream's synchronous native ioctl path,
checks the pinned ABI and `5.73.6-MIVU` identity, then caches approval
for that handle only. Subsequent game/process ioctls do **not** pay for
a version probe on every call. When the DLL closes or reopens the
device handle, cached approval is explicitly reset—including handle-value
reuse—and the next non-version ioctl must be checked again.
`SbieApi_GetVersionEx` still independently checks its own outputs.
No new ioctl code, driver object name or ABI layout is introduced.

**Service lifecycle fix:** a foreign-driver mismatch in private
`DriverAssistStart.cpp` now invokes upstream `AbortServer()` instead
of returning early from the asynchronous thread. This preserves the
existing failure shutdown path and prevents the private service from
pretending that driver startup succeeded.

**Actual source regression:** multiple insertions in `GetVersionEx`
must be represented by ONE combined Git hunk; adjacent overlapping
hunks were reproduced as `git apply --check` failures. The corrected
patch generator emits one combined version hunk plus nonoverlapping
handle-open, close and ioctl-admission hunks. A bounded local Git
fixture for the combined-version hunk passed `git apply --check --index`
and `git apply --index`. The exact pinned eight-file upstream source
patch passed V8 byte verification, with 16 hunks in total; all seven
eligible authored pure source regressions passed. The full pinned
Windows source/driver integration remains **NOT_EXECUTED**.

**Scope limit:** this is non-cryptographic identity admission for the
private DLL's normal ioctl entry point; it is not a kernel security
audit, Windows driver signing, replacement of all upstream clients, or
a guarantee against an adversarial kernel spoofing the version response.
No service/driver was installed, and the official account-bearing
Sandboxie instance was not touched.

## Private DLL ↔ service ↔ driver identity (15 October 2026)

The private C++ launcher version check is now extended to the **existing
SbieDll version API and the correct asynchronous service startup owner**:

- `Sandboxie/core/dll/sbieapi.c`: private `SbieApi_GetVersionEx` ALWAYS
  requests the full driver marker and ABI, even if a caller passes
  `NULL, NULL`. It rejects a non-MIVUBI kernel with
  `STATUS_REVISION_MISMATCH`, and copies version/ABI back to the caller only
  after the private core identity matches. The original public two-argument
  API, ioctl and buffer layout remain unchanged.
- `Sandboxie/core/svc/DriverAssistStart.cpp`: the asynchronous driver
  startup checks this specific mismatch before attempting `NtLoadDriver`.
  With an already active official Sandboxie driver, it refuses the private
  startup instead of trying to load another driver under the same service
  identity. Driver startup after a genuinely absent kernel follows the
  existing upstream sequence.
- `Sandboxie/core/drv/api.c`: retains the private version marker
  `5.73.6-MIVU` and pinned ABI `0x57230` on its existing version API.
  Private `Start.exe` still checks both before accepting an owned launch.

The approved fork closure now changes **eight** exact upstream files,
including `DriverAssistStart.cpp`, with **16** Git hunks. This is
significant source functionality—not just metadata—but it is not yet
a compiled/signed service, DLL, or kernel runtime. It is a
**version/handle-based fail-closed check**, not cryptographic runtime
attestation or a guarantee for alternate upstream clients bypassing the
private DLL. A complete Windows
MSVC/WDK build and disposable service/driver execution remain mandatory
before any app cutover. Never install this fork beside the official
Sandboxie with identical `SbieSvc`/`SbieDrv` names.

## Private kernel–launcher runtime identity (2026-10-14)

The first **kernel-facing functional source change** is now included in
the eight-file MIVUBI private overlay. The pinned upstream kernel
`Sandboxie/core/drv/api.c` replies to its EXISTING `API_GET_VERSION`
request with the MIVUBI-specific version string `5.73.6-MIVU`,
preserving the original driver ABI `0x57230`, ioctl ID, buffer layout,
and upstream behavior for an unmodified build. The shared
[`mivubi_core_identity.h`](../../engine/isolation-native/owned-runtime/mivubi_core_identity.h)
is embedded verbatim into both private kernel API and launcher source.

When the explicitly opted-in MIVUBI launch mode is requested, private
`Start.exe` calls `SbieApi_GetVersionEx` and refuses to route an owned
box unless the connected kernel reports **both** the private marker and
pinned ABI. An official Sandboxie driver replies with its original
version and therefore fails this private launch admission. This change
does **not** rename device/service objects; official/private coexistence
remains unsupported. It is a version identity gate, not cryptographic
attestation, signer verification, service ABI proof or kernel-load proof.

The existing native owned-runtime CMake suite now builds **both**:
C++17 Start policy admission tests and C11 driver-identity/16-WCHAR buffer
tests. The two binaries passed under GCC and Clang using CTest in
ChatGPT's Linux container (2/2 on each toolchain). The actual kernel,
DLL, Start.exe and SbieSvc still require Windows MSVC/WDK compilation,
signing and exclusive disposable-lab integration. An upstream consumer
that requires an exact version string may need deliberate compatibility
work once compiled; do not claim Windows compatibility from this source
evidence.

## Functional core: native-owned Start.exe profile admission

**Implementation owner:** [mivubi_owned_start_policy.h](../../engine/isolation-native/owned-runtime/mivubi_owned_start_policy.h).
The private overlay INLINES this exact C++17 policy into upstream
`Sandboxie/apps/start/start.cpp` and adds one opt-in dispatch point after
upstream command-line parsing obtains the argument string. We no longer keep
an independently authored copy of the path/box parser inside the JavaScript
patch generator. The committed private-fork source still changes exactly
seven pinned upstream files, now with eight Git hunks (six ABI guard sites,
the shared header, and the native dispatch site).

The launch policy accepts only:

~~~text
Start.exe /mivubi_client:02 "C:\Program Files\Microsoft Studios\Minecraft Education\Minecraft.Windows.exe"
Start.exe /mivubi_client:03 "C:\Program Files\Microsoft Studios\Minecraft Education\Minecraft.Windows.exe"
Start.exe /mivubi_client:04 "C:\Program Files\Microsoft Studios\Minecraft Education\Minecraft.Windows.exe"
~~~

These select separate box names `MivubiEdu02`, `MivubiEdu03`,
`MivubiEdu04`. Non-owned `Start.exe` command lines follow upstream code
unchanged; owned requests reject unsupported client IDs, executable names,
UNC/relative paths, traversal, unwanted shell arguments and malformed quotes.
A requested box is passed to upstream `RestartInSandbox()` and
`SbieDll_RunSandboxed()`. It is NOT a new implementation of the driver or a
Microsoft-signature verifier, and is not proof of WAM/SSO isolation.

**Native executable evidence (ChatGPT, Linux):** the same portable C++ header
was compiled with GCC and Clang using C++17, `-Wall -Wextra -Werror -pedantic`.
Each CMake/CTest test executable passed in the isolated ChatGPT container.
The checked-in [native test](../../engine/isolation-native/owned-runtime/mivubi_owned_start_policy.test.cpp)
contains 20 positive/negative admission cases. This is **native C++ evidence**,
not Windows/MSVC compilation of the fork's complete `Start.exe`, service,
DLL or kernel driver.

Reproduce in a supported developer environment:

~~~powershell
cmake -S engine/isolation-native/owned-runtime -B workspace/owned-start-policy
cmake --build workspace/owned-start-policy --config Release
ctest --test-dir workspace/owned-start-policy -C Release --output-on-failure
~~~

The separate [profile plan](../../engine/isolation-native/owned-runtime/minecraft-profiles.mjs) constructs
non-overwriting per-client `SbieIni` settings and private `Start.exe` launch
arguments, but never executes them. The current official Client 02 sandbox
remains active and unchanged. The private fork still retains the upstream
`SbieSvc`/`SbieDrv` names and cannot coexist with the installed official
Sandboxie service. Real independent operation requires a separately tested
disposable Windows lab with signed/native core artifacts; legal distribution
review is mandatory for publishing this Sandboxie-derived fork.
