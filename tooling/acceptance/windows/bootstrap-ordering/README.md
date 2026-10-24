# Self-owned early-import ordering experiment

This standalone Windows/MSVC x64 experiment answers one question: does an
in-memory first import arm a synthetic canary gate before the controlled
dependency's DllMain, the EXE TLS callback, and the EXE's main canary run?
The default ordering mode does not hook APIs. The separate mutex mode below
hooks a fixed mutex name, one synthetic bounded directory tree and two Known Folder IDs. Neither mode isolates general filesystem/HKCU views, identifies
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
flag. In mutex mode the flag follows installation of the four native mutex/file hooks plus SHGetKnownFolderPath in one transaction.
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

The existing owned-child pair command now exposes **recursive private directories**
through the same original DLL/native file hooks and A/B slot used by mutexes.
Only new synthetic TEMP trees are accessed. Existing game/account/cache/world
files are never read or changed. Trees, including the new test junctions, remain
local; no recursive deletion or traversal-based cleanup is performed.

NtCreateFile/NtOpenFile select absolute logical NT device/DOS paths, native own
device paths, or directory-relative paths. Root IDs identify configured roots;
for deeper directory handles, GetFinalPathNameByHandleW supplies a canonical NT
path beneath the configured logical/own root. The relative suffix is resolved
again from the inherited private root, without a handle registry/state store.
Unknown roots/aliases still pass through. Canonical descendant classification
is cooperative provenance, not authenticated enrollment or a hostile-race proof.

Relative paths are bounded to fewer than 512 UTF-16 units, at most eight
components of 1–64 ASCII characters. Each component starts with a letter/digit;
remaining characters may additionally contain dot, underscore or hyphen.
Dot traversal, streams, empty/intermediate/trailing components, trailing dots,
Unicode and excessive lengths/depth return STATUS_NOT_SUPPORTED.

Each intermediate component is opened independently through the original
NtOpenFile, with FILE_DIRECTORY_FILE/FILE_OPEN_REPARSE_POINT, OBJ_DONT_REPARSE,
non-delete-sharing handles and directory/reparse/case-sensitivity metadata checks.
Every ancestor handle stays open through the final operation, then closes even
on the protected exception path. No automatic parent-directory creation occurs;
missing directories retain failure status. Callers can explicitly create private
directories. Root itself allows FILE_OPEN only.

Final calls copy OBJECT_ATTRIBUTES, replace parent/name and add OBJ_DONT_REPARSE.
Access/share/disposition, EA/allocation/options and IO_STATUS_BLOCK are forwarded.
Caller memory stays unchanged; forwarded NTSTATUS is returned and entry LastError
restored. Early policy/parent-resolution failure does not synthesize caller
IO_STATUS_BLOCK. No host fallback or read COW is introduced.

DELETE, owner/DACL, GENERIC_ALL, non-directory execute, delete-on-close and
open-by-ID requests remain unsupported. FILE_OPEN_REPARSE_POINT supplied by the
caller is rejected except the observed CreateDirectoryW pattern: FILE_CREATE
plus FILE_DIRECTORY_FILE. This permits creating a new directory without granting
an existing-reparse open path. Root/non-directory contradictions are rejected.

**Enumeration needs no hook:** real private directory handles let Windows supply
native and Win32 entries, cursor/restart/filter behavior and end-of-enumeration.
There is no merged host view or synthetic query-result rewriting. The private
receipt stays version 4; no product wire schema changes.

File mask **1048575**, per child/generation:

| Bit | Actual check |
| --- | --- |
| 1–32 | Existing private missing/persistent file, Win32 read/write/flush, absolute native DOS open, counted relative OPEN_IF/access/inheritance, DELETE/stream rejection and unselected outside-file controls. |
| 64 | After both children are ready, flat and nested contents/listings still contain only own files. |
| 128 | Logical root open returns the own private volume/file ID. |
| 256–1024 | Native multi-entry, restarted single-entry, wildcard and Win32 enumeration match the exact private flat directory set. |
| 2048 | Flat peer/host-only files stay absent; traversal, stream and trailing-dot inputs are rejected. |
| 4096 | Explicit Win32 creation of level/deep private directories; existing directories are observed on relaunch. |
| 8192 | Deep native handle-relative create/open agrees with absolute Win32 reads; nested state/slot markers persist across relaunch and caller attributes/LastError/information are preserved. |
| 16384 | Deep native multi/single-entry enumeration sees only own state/slot files; peer file is missing. |
| 32768 | Newly created directory junctions are refused as final targets and intermediate components, including an attempted overwrite through them; traversal/stream/excess-depth requests fail. |
| 65536 | APPDATA/LOCALAPPDATA, default Known Folder and DONT_VERIFY return the same expected private paths for both IDs. |
| 131072 | Environment-path writes agree with Known Folder and logical-path reads, and private directory enumeration. |
| 262144 | AppData files start missing despite synthetic logical HOST files; prior slot markers persist after relaunch. |
| 524288 | Non-null tokens, unsupported flags and null output are rejected; the controller also rejects a deliberately misbound environment before main. |

The parent creates private nested/escape **junctions pointing only to a fresh
synthetic guard directory**, not any user/game location. It verifies the guard,
logical flat/nested HOST files and outside file stay unchanged, plus both flat
and nested private files per slot, during and after both pairs. A/B relaunch
retains previous markers. Existing marker order, slot-mismatch rejection,
Win32/native mutex controls and normal exits remain required.

**Evidence ceiling:** the mount-point tests establish refusal of these junction
paths on this Windows host. The general OBJECT_ATTRIBUTES documentation defines
OBJ_DONT_REPARSE, but complete user-mode/native-function and all-reparse-tag OS
compatibility is not established. File symlinks, other tags, hardlinks, handle
rename/delete, inherited/duplicated data handles, hostile races, real namespace
presentation, Unicode and general filesystem behavior remain UNKNOWN.
This is cooperative compatibility, **not a security boundary**; same-user direct
private backing access remains possible. The narrow AppData lookup path is described below. Effective Minecraft
Roaming/Local coverage, HKCU, external authentication and game injection are not established.

API contracts:
[OBJECT_ATTRIBUTES](https://learn.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-_object_attributes),
[NtCreateFile](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntcreatefile),
[NtOpenFile](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntopenfile),
[NtQueryDirectoryFile](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/ntifs/nf-ntifs-ntquerydirectoryfile),
[Reparse points and file operations](https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points-and-file-operations).

## AppData environment and Known Folder — 24 October 2026 WIB

The same controller now supplies a fresh child environment block with APPDATA
and LOCALAPPDATA replaced by the slot's existing private Roaming/Local directories.
It removes previous case-insensitive definitions from that copy; it does not set
the parent/system environment. No existing profile data is seeded or copied.

DLL initialization derives those two expected paths from the validated private
directory handle and rejects disagreeing environment values. It resolves the
SHGetKnownFolderPath export from the already-imported Shell32 module and attaches
it in the same Detours transaction as file/IPC hooks. There is no LoadLibrary,
COM initialization, allocator call or actual shell lookup in our DllMain.
Shell32/Ole32/Uuid are Windows SDK links, not downloaded or bundled dependencies.
This controlled import graph does not prove general loader/game compatibility.

For FOLDERID_RoamingAppData and FOLDERID_LocalAppData, current-user (null token)
lookup returns the private DOS path. Flags 0 verify existing private directories;
KF_FLAG_DONT_VERIFY skips that metadata check. The caller owns a CoTaskMemAlloc
copy and frees it with CoTaskMemFree. Entry LastError is restored and failures
clear the output. Non-null tokens (including the Default User pseudo-handle)
return E_ACCESSDENIED; null output returns E_POINTER. Other flags, including
CREATE and DEFAULT_PATH, return E_NOTIMPL rather than host/default-profile paths.
Unselected IDs forward to the original API unchanged.

The native file selector also recognizes canonical own DOS paths so environment/
Known Folder file opens use the same original private component traversal.
The test initializes COM outside loader lock, checks both IDs/flags, writes each
environment file and reads it through Known Folder and logical host-path mapping.
Only newly created logical HOST markers and private A/B markers are accessed.
Parent raw reads during/after both process generations verify no crossover,
persistent private values and unchanged logical/guard/outside markers.

A second negative binding control configures slot A/private A with the synthetic
logical directory's AppData environment. It fails before Armed/main. Existing
A/private B mismatch, ordering/failure/timeout, native/Win32 file and IPC controls
remain required. Receipt version/layout remain unchanged.

This is process-local selected API compatibility, not Windows profile registration
or complete Known Folder virtualization. SHGetFolderPath, WinRT, shell/COM object
interfaces, packaged redirection flags, LocalLow, other IDs, inherited child
contexts, environment mutation and arbitrary hardcoded profile aliases remain
unimplemented/unverified. The process can still mutate its own environment.
HKCU and authentication are not isolated; no actual game was launched/injected.

Contracts:
[SHGetKnownFolderPath](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath),
[KNOWN_FOLDER_FLAG](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/ne-shlobj_core-known_folder_flag),
[Known Folders](https://learn.microsoft.com/en-us/windows/win32/shell/known-folders).

## Evidence

### Executed AppData lookup slice — 24 October 2026 WIB

Exact baseline: d85fe6d794fbeea8b006bd8cf32cf2de18027817.
Changed executed input blobs:

| Input | Git blob |
| --- | --- |
| compat.cpp | 711c50d7885b7bfd5d2299a4085c2bbf668b0534 |
| probe.cpp | 17e43f7dc6bef13fbb2822bb9f25263aeafa64d1 |
| CMakeLists.txt | c69a5ad5127e5116f63bdfb7200f4fb16cf7c2f7 |

Receipt v4 and other source inputs are unchanged. External Detours stays clean
at 1dbd4ea281d73d478aa8d09a8b83fbd265f715f7. Windows x64 26100,
CMake 4.3.2, MSVC 19.44.35226 Release /W4 /WX configure/build succeeds.
An initial shadowed-local warning was corrected without weakening /WX.

Both final fixture commands exit **0**. Four original ordering controls and
summary pass; two simultaneous children across two generations pass file masks
1048575/1048575, Win32 mutex 127/127, native mutex 511/511. AppData environment/
Known Folder/logical-path agreement, persistence, unchanged synthetic host files,
slot mismatch and separate AppData environment mismatch rejection pass.
No remaining test image in independent CIM lookup; tracked checkout clean.
Full product checks, Minecraft injection/login and two/four-client NOT_EXECUTED.

Local artifact SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| bootstrap-ordering.exe | 5aa24485e6abfb6b535e350f92c02adb028f331ea509df2cfc8e8fe4d6d51939 |
| bootstrap-compat.dll | 0e0fef07f70bafcfa300fd735ac6d374b93106c9654bd7e0a71e5ca6eab44474 |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | 0816ca916f59501edf124669b0bb649a97fbe88daaca539df99b1bfdb3134799 |

Earlier evidence retains its historical SHA/scope.


### Executed recursive directory slice — 24 October 2026 WIB

Exact baseline: 08ec394d81fecea18553dfd50c8ddd4471ef11b6.
Changed executed blobs: compat.cpp ce83c2b3c79e331640a5661f5f2a4922b7b15e96;
probe.cpp 9f52cc26d8e45f482938c0197a4fef2b5b1b3a9f.
Receipt v4 and other build inputs remain unchanged; external Detours clean at
1dbd4ea281d73d478aa8d09a8b83fbd265f715f7.

Windows x64 26100 / CMake 4.3.2 / MSVC 19.44.35226 Release /W4 /WX
configure/build succeeds. Both final explicit commands exit **0**: four ordering
controls and summary pass; pair file masks 65535/65535, Win32 mutex 127/127,
native mutex 511/511, two generations, persistence/host preservation and
slot-mismatch rejection pass. CIM inspection finds no remaining test image;
tracked checkout stays clean. Full product/game/two/four-client NOT_EXECUTED.

The first pair attempt rejected CreateDirectoryW's FILE_OPEN_REPARSE_POINT
creation flags; a local-only diagnostic pinpointed that policy mismatch.
The final implementation permits only the new-directory pattern described above,
with all rejection controls retained. An observed failure-cleanup race was also
corrected: if the owned child exits before TerminateProcess, cleanup rechecks the
same original handle rather than falsely reporting uncertain cleanup. Diagnostic
line/argument logging was removed before the final build. No arbitrary process
or file target, public API, product admission, driver/service or CI changed.

Local artifact SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| bootstrap-ordering.exe | a700a5c1d71ab460ce6607b4c43f24a4802cfdf8399ca470f7e5df4f69cf6b8d |
| bootstrap-compat.dll | 04274fea02bb67d209e39f6ea4e2b6df33938452d64f3542cdf6d0185096e01e |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | a27fae028d1f9966ae734ff5b376f0fa553784b2aa7c974169fa9bb7717f2645 |

Earlier evidence retains its historical SHA/scope.


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
