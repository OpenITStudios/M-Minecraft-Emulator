# Self-owned early-import ordering experiment

This standalone Windows/MSVC x64 experiment answers one question: does an
in-memory first import arm a synthetic canary gate before the controlled
dependency's DllMain, the EXE TLS callback, and the EXE's main canary run?
The default ordering mode does not hook APIs. The separate mutex mode below
hooks a fixed mutex name and one synthetic flat directory. Neither mode isolates general filesystem/HKCU views, identifies
Minecraft's singleton, separates brokers, or approves a provider. The marker graph is intentionally
small; success cannot establish order for every Windows/game dependency.

The controller only launches its **own absolute executable path**, suspended,
with a fixed internal child argument. It updates the resulting original process
handle via DetourUpdateProcessWithDll and resumes the original thread. There is
no target-path/PID option, process discovery, shell activation, cross-bitness
helper, game launch, service/driver operation, account access or product wiring.
The sibling compatibility DLL is fixed, and the output directory must be a
trusted build directory. This is not an authenticated production injector.

## Source and build boundary

Our receipt, controller, compatibility DLL and dependency marker are independently
authored. Import insertion uses a **test-only external Microsoft Detours source
dependency**, pinned to 1dbd4ea281d73d478aa8d09a8b83fbd265f715f7.
Nothing is downloaded or vendored by this target. CMake checks the external
checkout's revision/clean src and builds its static library directly; production
adoption is undecided. Retain the upstream checkout's LICENSE.md when handling
its source/artifacts. No Sandboxie code or runtime is used.

From an existing compatible developer environment, configure only this folder,
explicitly opting in and supplying the trusted pinned external checkout:

~~~powershell
cmake -S tooling/acceptance/windows/bootstrap-ordering -B workspace/bootstrap-ordering-build -A x64 -DBOOTSTRAP_ORDERING_TEST_ONLY=ON -DDETOURS_ROOT=C:/path/to/reviewed/Detours
cmake --build workspace/bootstrap-ordering-build --config Release
~~~

Build does not execute the experiment. A separately requested manual run is:

~~~powershell
& ./workspace/bootstrap-ordering-build/Release/bootstrap-ordering.exe --run-self-test
~~~

The target accepts only the two fixed self-test switches and its internal child
argument, and rejects unsupported architectures. The fixed
compatibility import path currently requires ASCII; there is no lossy conversion
or alternate-path fallback. Keep source and build directories separate.

## Ordering controls and receipt

Each case creates a fresh anonymous section; only that HANDLE is inherited through
the Win32 handle allowlist. The parent binds PID and creation FILETIME while the
child is suspended. Marker code validates them before using the fixed-size
receipt. The environment value transports a handle number; it is **not** a
production capability. The controlled child can write its receipt, so this is
cooperative instrumentation, not adversarial isolation evidence.

A failed normal-order result does not skip controls. A fatal API/transport error
stops the remaining cases and emits an incomplete-operation receipt:

| Case | Expected self-check |
| --- | --- |
| baseline | No import insertion; dependency/TLS/main all record an unarmed canary. This positive control detects missing instrumentation. |
| first-import | Attach and arm recorded once; all three canaries recorded after arm. Any early canary, missing marker, exit or timeout fails. |
| attach-failure | DLL returns FALSE during initial attach; child exits unsuccessfully and never reaches main. |
| missing-arm-timeout | DLL deliberately does not arm; all three canaries remain unarmed and main sleeps outside loader lock. Parent must detect timeout and stop only its own child. This intentionally insecure control is never a production fallback. |

Stages: 1=compat attach, 2=arm, 3=attach rejected, 4=arm suppressed,
5=dependency DllMain canary, 6=EXE TLS canary, 7=main canary.
Each event includes the observed arm bit. At most 16 records are accepted;
overflow, invalid stages and malformed transport fail. DllMain never waits
for a controller handshake or calls registry/COM/known-folder APIs.

The parent reads only after confirmed child exit, with bounded waits. Cleanup
uses the original process handle; no PID reopen or unrelated termination occurs.
Failure receipts distinguish incomplete operations and cleanup uncertainty.
JSON contains fixed stage/case labels and booleans/counts, with
providerReady=false and minecraft=NOT_EXECUTED. It emits no raw PID, FILETIME,
HANDLE, username, installed game path, credentials, caches or world contents.

No registry operations are mediated. Only the selected synthetic directory below is redirected. In the default ordering mode,
"canary" means a marker at a would-be protected call site and "arm" is a shared
flag. In mutex mode the flag follows installation of all four native mutex/file hooks in one transaction.
This test cannot observe kernel accesses before notification, all loader work,
external authentication, other module/TLS graphs or enforcement under bypass.

## Real mutex-name translation on self-owned children

The optional `--run-mutex-self-test` mode reuses the same fixed executable,
early-import DLL and pinned external Detours. **NtCreateMutant and NtOpenMutant**
are now the single interception layer; the separate CreateMutexW/OpenMutexW
hooks have been removed. Existing Win32 W calls and explicit native calls use
that layer. This is original code; no Sandboxie code/runtime is imported.

Only the exact configured test basename from `Local\\MivubiMutexProbe_` and
the child's actual session BaseNamedObjects directory can map to the A/B suffix.
Counted UNICODE_STRING input is bounded; caller memory is not rewritten.
Absolute names and RootDirectory-relative names (including Local aliases) resolve
their parent through NtOpenDirectoryObject. CompareObjectHandles checks identity
against a retained handle to the child's session directory, rather than comparing
handle numbers or matching a basename in every directory. The already-loaded
system modules supply checked exports; missing exports/initialization reject
attach. No NtQueryObject on arbitrary handles, shared hierarchy creation, extra
service or driver is introduced.

Names outside that exact selection, unnamed calls, unsupported lengths/shapes
and unresolved/different parent contexts pass to the original API. This is
**not fail-closed mediation for arbitrary callers**. Selected calls copy
OBJECT_ATTRIBUTES, preserving root/flags/security pointers and replacing only
the counted name. Native NTSTATUS is returned unchanged; entry LastError is
restored even when internal lookup/comparison changes it. Protected parameter
reads translate access violations into their native exception status.
The initialization/parent handles and hooks belong to this process-lifetime
fixture; no live DLL unload or authenticated production slot capability exists.

The private **version-4** receipt (1,000 bytes, not a product wire schema) carries
native/file masks and the file round/root handles. It still binds the original child PID/creation FILETIME,
slot/test name and seven explicitly inherited handles: section, four phase events,
logical directory and the slot's private directory. Both children hold
their own object while the parent holds the host logical object and checks the
A/B identities. Existing Win32 controls remain bitmask **127**: fresh/existing
create, recursive open, unnamed/missing behavior, access/inheritance flags and
raw-peer wait/cleanup.

Direct native controls in each child are bitmask **511**:

| Bit | Actual check |
| --- | --- |
| 1 | Absolute native open resolves to the Win32-owned private object. |
| 2 | Relative directory-handle and Local-alias opens reach that same object. |
| 4 | Counted name ignores a non-terminator tail; caller string/attributes remain unchanged. |
| 8 | OBJ_CASE_INSENSITIVE opens the selected object; case-sensitive lowercase returns name-not-found. |
| 16 | OBJ_OPENIF duplicate create returns STATUS_OBJECT_NAME_EXISTS and honors OBJ_INHERIT. |
| 32 | Unnamed native create remains a distinct usable object. |
| 64 | Limited access/inheritance is honored, wait is denied without SYNCHRONIZE, and missing open preserves native name-not-found. |
| 128 | Same basename under the unrelated KnownDlls directory is not redirected. |
| 256 | Invalid name-buffer access returns STATUS_ACCESS_VIOLATION, without changing LastError. |

Every direct native API check captures LastError immediately. These are narrow
cooperative controls, not proof of every invalid input, ACL/security descriptor,
handle race, abandonment or hostile bypass. The controller still requires exact
early-import canary ordering, normal child exits and disappearance of both
backing objects. Same-user raw peer access **still succeeds**.

Manual invocation, separately from the four ordering controls:

~~~powershell
& ./workspace/bootstrap-ordering-build/Release/bootstrap-ordering.exe --run-mutex-self-test
~~~

Receipts retain providerReady=false, minecraft=NOT_EXECUTED and
securityBoundary=false. ANSI/Ex variants, other object types, inherited/duplicated
handle containment, general filesystem/HKCU views, real game hooks and singleton/auth behavior
remain outside this slice. Real cache/input/diagnostic names are never selected.

API references:
[CompareObjectHandles](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-compareobjecthandles),
[NtOpenDirectoryObject](https://learn.microsoft.com/en-us/windows/win32/devnotes/ntopendirectoryobject).
The native export may change/unavailable on future Windows; successful checks on
this host do not establish universal OS compatibility.

## Coherent synthetic file backing — 24 October 2026 WIB

The same mutex pair command runs two generations against one newly created TEMP
root, now exposing a **flat private directory** instead of one fixed file.
Only fresh synthetic markers are accessed. No existing host/game/account/cache/
world files are opened. Test trees stay local, without recursive cleanup.

NtCreateFile/NtOpenFile select the configured logical root itself or a flat leaf
under it, then open the private A/B directory or its leaf. The same receipt.slot
selects backing and mutex suffix. Root opens use an empty counted name relative
to the inherited private directory handle; their returned handles identify that
private directory. Directory-relative leaf access recognizes both logical and
private root volume/file IDs, including independently opened handles.

All four hooks still install atomically. Caller OBJECT_ATTRIBUTES is copied;
leaf spelling/counts, access/share/disposition, allocation/EA/options and
IO_STATUS_BLOCK are forwarded. NTSTATUS and entry LastError are preserved.
Missing private files stay missing without copying/falling back to logical HOST.
Root opens allow FILE_OPEN only; directory creation/truncation is unsupported.

**Enumeration needs no additional hook:** NtQueryDirectoryFile and Win32
FindFirstFileW/FindNextFileW operate on the real private directory handle returned
by the mapped open. Windows supplies entries, cursor/restart/filter behavior and
NO_MORE_FILES. There is no host/private merge, synthetic query-result rewriting,
shared enumeration cursor or second directory-state store.

Attach retains canonical ASCII path, volume/file-ID, non-reparse and
case-insensitive-root checks; A/private B binding fails before Armed/main.
Allowed leaves are 1–64 ASCII letters/digits, with dot/underscore/hyphen permitted
after the first character; trailing dots are rejected. Empty leaf selects root.
Subpaths, dot traversal, streams and other leaf syntax return
STATUS_NOT_SUPPORTED under recognized logical/private roots. DELETE,
owner/DACL, GENERIC_ALL, leaf execute, delete-on-close, open-by-ID and
open-reparse operations remain rejected; directory option is allowed only at
the root, and non-directory option only for leaves. Policy rejection does not
synthesize IO_STATUS_BLOCK.

This is cooperative routing, **not a security boundary**. Unknown roots/aliases
pass through. Recursive directories, arbitrary Unicode, leaf reparse/hardlink
attacks, handle rename/delete, inherited/duplicated data handle containment,
filesystem identity presentation, read COW and adversarial races are missing.
Direct same-user private backing access remains possible. Loader safety beyond
this controlled import graph and general Windows compatibility are unproved.

File mask **4095** per child/generation:

| Bit | Actual check |
| --- | --- |
| 1 | Initial private state.bin is missing despite HOST; relaunch reads previous own marker. |
| 2 | Win32 create/open, write, truncate and flush use private state.bin. |
| 4 | Absolute native DOS-form open preserves status/information/LastError and reads own marker. |
| 8 | Counted relative OPEN_IF preserves input, inheritance, information and read-only access. |
| 16 | DELETE access and an alternate stream are rejected without a handle. |
| 32 | Unselected synthetic outside file passes through unchanged. |
| 64 | After both children are ready, own contents and private directory entries remain unchanged. |
| 128 | Win32 logical-directory open returns the own backing's volume/file ID. |
| 256 | Relative native creation writes slot-A/B.bin; native multi-entry enumeration sees exactly state.bin plus own slot file. Relaunch first reads its previous marker. |
| 512 | Restarted single-entry native enumeration on the same directory handle yields the same exact set and terminates. |
| 1024 | Fresh-handle native wildcard filtering and Win32 enumeration see only own files. |
| 2048 | Own handle-relative peer file is absent; absolute own leaf reads agree; traversal/stream/subpath/trailing-dot calls are rejected; host-only.bin stays absent. |

The parent independently reads both private files per slot and unchanged
logical state.bin/host-only.bin plus outside.bin during and after both pairs.
Second-generation checks include previously persisted peer files, so absence
cannot be inferred merely from timing of first creation. Native/Win32 mutex
controls, mismatch rejection, marker ordering and normal exits remain required.
Device-form mapping is implemented; the explicit absolute native test uses DOS
form. Roaming/Local/Known Folder, HKCU and authentication are not established.

API contracts: [NtCreateFile](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile),
[NtOpenFile](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntopenfile),
[GetFileInformationByHandleEx](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getfileinformationbyhandleex),
[NtQueryDirectoryFile](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ntquerydirectoryfile),
[FILE_NAMES_INFORMATION](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/ns-ntifs-_file_names_information).

## Evidence

### Executed flat directory slice — 24 October 2026 WIB

Exact baseline: 661c79121d607f3c2ecfb086764048b9a8228d1e.
Changed compiled blobs: compat.cpp e9f53325d424971e7b202adf22735de9dd42bfcb;
probe.cpp 70d54cb23e7a1ccb1377a60fc7a92e5af5fb4d73.
Receipt v4 and all other build inputs remain unchanged. External Detours stays
clean at 1dbd4ea281d73d478aa8d09a8b83fbd265f715f7; no new dependency.

Windows x64 26100, CMake 4.3.2, MSVC 19.44.35226 Release /W4 /WX configure/build
and both explicit fixture commands passed (exit 0). Four ordering controls and
summary pass. Pair: file 4095/4095, Win32 mutex 127/127, native mutex 511/511;
two generations, private persistence, unchanged host markers and slot-mismatch
rejection pass. Independent CIM lookup finds no remaining test image. Tracked
checkout remains clean. Full product checks/game/two/four-client NOT_EXECUTED.

Local artifact SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| bootstrap-ordering.exe | 585dc0e40630040231f739b3eaec5242ffe1c97e2025ca40b418d7037ffed9ee |
| bootstrap-compat.dll | 3c94c02655e86b49e82bd9bb288fc0e571e86b61786ea9f6505785d3ee135647 |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | 06180b0a270b0db4065de5f313b3885e565f4e6b072d1cad70dc02f6ab33ad22 |

Earlier evidence below retains its historical SHA/scope.


### Executed coherent file slice — 24 October 2026 WIB

Base source: 1a6ddb032c5d9a3572160375f588401586de979e.
Changed compiled blobs:

| Input | Git blob |
| --- | --- |
| compat.cpp | 9f7ab8891d639ded9b2b2119540133b24b96aaed |
| probe.cpp | 0986fc4f98c21b025ad9910c42d36ed38ea5a09c |
| receipt.h | 72bd06eb87833100a5a6ce6ac63176cb97630a12 |

Other build inputs and the clean pinned external Detours source are unchanged.
Windows x64 26100 / CMake 4.3.2 / MSVC 19.44.35226 Release /W4 /WX configure
and build succeeded. Both explicit commands exited **0**. All four ordering
controls passed. Pair output: file masks 127/127, mutex 127/127, native 511/511,
rounds=2, persistent=true, hostUnchanged=true, slotMismatchRejected=true.
Independent CIM lookup found no remaining test image. Tracked checkout stayed
clean. Full product checks and Minecraft/two/four-client acceptance NOT_EXECUTED.

Local artifact SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| bootstrap-ordering.exe | 60d41245a11761bb09c6c6952f067d00b30853ea64d039f77f270cb686458e54 |
| bootstrap-compat.dll | cdbacf2c3ba57e3baf78a53530a74fa9a284f5cbb2ff04f6a572a9b948dc6c41 |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | acf79c3778c14514992e70d3724746cb121f3a0b30266f94c7337784641d088a |


### Executed native IPC slice — 24 October 2026 WIB

Base source: `d5452a90bedb9a65e20a6db8e0f5138f7681bbe8`.
Compiled changed blobs:

| Input | Git blob |
| --- | --- |
| compat.cpp | `1b1d632fce0a53d83c655264700686d858100419` |
| probe.cpp | `88fb2291d537b7081cddd4c728616d3636d4873c` |
| receipt.h | `aaef568b1fec93403967a276826ce1e218ebdd16` |

CMakeLists.txt, dependency.cpp and compat.def remain unchanged. Trusted external
Detours remains the same pinned clean source. Windows x64 26100, CMake 4.3.2,
MSVC 19.44.35226/MSBuild 17.14.40 Release `/W4 /WX` build succeeded.
An unavailable SDK KernelBase import-library link was replaced with a checked
export from the already-loaded KernelBase module; no dependency was installed.

Both self-test commands exited **0**. The two-child run reported Win32 masks
`[127,127]` and native masks `[511,511]`; host/A/B ownership, normal exits and
backing-object cleanup passed. All four original ordering/failure/timeout
controls and summary selfCheck=true passed. Independent exact-image observation:
zero remaining owned images, zero fixture candidates with inaccessible paths.
No game process or account/world/cache contents were inspected or changed.

Local SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| bootstrap-ordering.exe | `f8e034003f26bbf9875943fbc7d94ac93a4ee4d72cc6ea7ff6a2a10ee7a12bb7` |
| bootstrap-compat.dll | `990940a804254300d3c13097735ce8e632129dfe2c00e917df08c7279dd16249` |
| ordering.jsonl | `40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36` |
| mutex.jsonl | `6863b9b373caef73064423f65827a761f914bf2974855108c0f7b4a1200c8dd1` |

Artifacts remain ignored local output. Raw peer access is still possible;
production security, real game injection, private file/HKCU/auth and two/four
licensed playable clients remain **NOT_EXECUTED / UNKNOWN**. The historical
Win32-only scope below describes its earlier SHA, not this native revision.

### Executed mutex slice and bootstrap regression — 23 October 2026

Base source: `3b2ad6872a1d460de49096b84089ca631c030909`, with the exact
compiled inputs below. Their blob IDs bind the executed snapshot to this
change's source, independently of documentation updates.

| Compiled input | Git blob |
| --- | --- |
| CMakeLists.txt | `fe177777862844314b42ad866cef13144601b0ba` |
| compat.cpp | `4bfe6accc47805131f18573bda3088805dcd6556` |
| compat.def | `b73e48f5312ce48430ffe85f101d7736b8011ac9` |
| dependency.cpp | `aa522f6d72672caab69234e21308234f40a106e4` |
| probe.cpp | `1cb5a6a69c7e9a3819d8f4a85b566e6cc5432e8b` |
| receipt.h | `9b3a75f4dc37c03f565157cd267f6084551f33ea` |

Windows x64 26100, CMake 4.3.2, MSVC 19.44.35226, MSBuild 17.14.40.
Configure and Release build with /W4 /WX succeeded. Pinned external Detours
remained clean. Both commands returned native exit **0**:

- `--run-self-test`: all four original cases and summary selfCheck=true;
  baseline detected three early markers, normal detected zero, failed attach
  prevented main and the timeout control stopped its owned child.
- `--run-mutex-self-test`: both children reported bitmask **127**; controller
  verified distinct held host/A/B objects, each child's exact early-import
  marker order, normal exits, and disappearance of both backing mutexes.
- The deliberate raw peer open **succeeded**: rawPeerAccessible=true,
  securityBoundary=false. This is an observed limit of name translation,
  not a successful peer-access denial.
- Independent exact-image CIM check: **0** test processes remaining.
  Five ordering receipt rows and one mutex row validated. Evidence captured
  at 16:47 UTC / 23:47 WIB. No game process, cache or world was accessed.

Local artifact/receipt SHA-256 (ignored workspace output; binaries not committed):

| Artifact | SHA-256 |
| --- | --- |
| build/Release/bootstrap-ordering.exe | `c7c8f0a2f0910ddcce229a78fae5be99db45b4084d4e028193a7be53c14b7149` |
| build/Release/bootstrap-compat.dll | `137c3c8148eaafd9ca9985d62654c91b523132c52ee5bd6f75f01e29bc78de5d` |
| build/Release/bootstrap-dependency.dll | `dfb45bc9736a7c16b73e5d6ba0bb28f559631ab406353d5a73c327d5c8872cd8` |
| build/Release/probe-detours.lib | `ff0079422f20ea68667eccf9714e4604820f70a983de1aac9eb9542ce94ab284` |
| ordering.jsonl | `40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36` |
| mutex.jsonl | `12105d2d49fd5763fb7af7d2c1caca6e039983a5a2232f6ef67b5fe720d5ca46` |

Only the documented Win32 W-function path and controlled object graph were
exercised. Handle **inheritance flags**, not a grandchildren inheritance test,
were checked. Production enforcement, direct native/ANSI/Ex call coverage,
crash/abandonment semantics, real Education singleton, persistent file/HKCU/auth
and two/four-client gameplay remain UNKNOWN / NOT_EXECUTED.

### Executed on 23 October 2026

Exact fixture source: `d9a2fb80ad7ca362b4a9cb8a42cdb90e08cf88ae`.
All seven materialized source/readme blobs matched GitHub before compilation.
External Detours: clean `1dbd4ea281d73d478aa8d09a8b83fbd265f715f7`.
Windows x64 build 26100; CMake 4.3.2, MSVC 19.44.35226,
MSBuild 17.14.40. Configure and Release build succeeded with the committed
/W4 /WX settings. No source correction was needed for this build.

One explicit self-test run returned native exit code **0**:

| Case | Observed stages / arm bits | Result |
| --- | --- | --- |
| baseline | dependency, EXE TLS, main: all 0 | selfCheck=true; three unarmed canaries detected |
| first-import | attach(0), arm(1), dependency(1), TLS(1), main(1) | selfCheck=true; zero early canaries in this graph |
| attach-failure | attach(0), rejected(0); no main | selfCheck=true; failed attach prevented main |
| missing-arm-timeout | attach/suppressed/dependency/TLS/main: all 0 | selfCheck=true; timeout detected, owned child stopped with expected code |
| summary | SELF_CHILD_BOOTSTRAP_ONLY | selfCheck=true, providerReady=false, minecraft=NOT_EXECUTED |

The initial shell wrapper ended with status 1 from its empty Get-Process lookup;
the native test explicitly reported PROCESS_EXIT=0. A separate successful CIM
check found **0** remaining processes at the exact test-image path. Receipt
validation found exactly five rows, all self-checks true, no provider readiness
and no Minecraft acceptance. Verification recorded at 16:37 UTC / 23:37 WIB;
no game process was inspected or modified.

Artifacts remain local in ignored `workspace/tmp/bootstrap-run-*/`.
SHA-256 values identify this particular build/run, not reproducible-build proof:

| Artifact | SHA-256 |
| --- | --- |
| build/Release/bootstrap-ordering.exe | `240f4005056dc54b74f4c96334338da5bf564ab63fa2822626d578283156414f` |
| build/Release/bootstrap-compat.dll | `1bc1615588f1e9899243c2eca105b902fd038f5ff25b07dad71144fd7856ccbc` |
| build/Release/bootstrap-dependency.dll | `66f85d93cbb0701e933fe9ead106b6b5df414dab3f06f755067fe70d52e4f39f` |
| build/Release/probe-detours.lib | `5c527a336a95ef1440b2b9dc2c7df8140348ff0f85939bc35da5ba649aea99bf` |
| receipt.jsonl | `40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36` |

This establishes SOURCE_EXECUTION plus limited TARGET_WINDOWS evidence for the
specific self-owned synthetic graph and controls. It does not establish
universal loader ordering, installed hooks, actual private file/HKCU/IPC views,
Education singleton/broker compatibility, game injection or two/four-client
acceptance. Those remain **NOT_EXECUTED / UNKNOWN**. Current continuation belongs
only in `planning/development.md`.
