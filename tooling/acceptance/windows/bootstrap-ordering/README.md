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
no target-path/PID CLI option, process discovery, shell activation, cross-bitness
helper, game launch or service/driver operation. The private Rust preparation
seam below is reachable only through the explicit same-manager startup mode;
ordinary MC02–04 admission stays closed.
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

The target accepts only the four fixed self-test switches and its internal child
argument, and rejects unsupported architectures. The fixed
compatibility import path currently requires ASCII; there is no lossy conversion
or alternate-path fallback. Keep source and build directories separate.

## Process-bound storage configuration

The original compatibility code now has a separate, private
[Education bootstrap configuration](./education-config.h). This source seam
removes the receipt, numeric mutex-name and sibling-directory requirements
from the **selected storage/HKCU** installation path. It does not select a
Minecraft provider. The desktop Rust preparer now publishes this configuration
for two fresh slots. Its optional private enrollment seam requires separately
reviewed DLL bytes. Its explicit private startup observation mode now pins the
operator-authorized development candidate; tests keep the game policy empty.

The target reads a snapshot from an unnamed section through a read-only
handle; a writable handle is rejected. The controller fills it while its
new child is suspended. PID, creation FILETIME, interactive session, version,
size and scope are checked; logical/private directory IDs are rechecked from
the supplied handles. Nested/overlapping roots are refused. The ordinary
registry backing must match the private directory's volume/file ID and slot
under the current user's fresh Education-pilot subtree. The current process's TokenUser SID metadata must match the supplied current-user
key path; no existing account values are queried. Formatting follows the official
[SID string syntax](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/c92a27b1-c772-4fa7-a432-15df5f1b66a1).

This follows Windows [unnamed-section handle sharing](https://learn.microsoft.com/en-us/windows/win32/memory/sharing-files-and-memory)
and [mapping access rights](https://learn.microsoft.com/en-us/windows/win32/memory/file-mapping-security-and-access-rights).
A read-only target handle does not authenticate a hostile same-user controller,
prevent the parent from updating its own mapping, or prove full containment.

Initialization uses token/handle metadata rather than opening a user registry
root, loading a hive/profile, performing COM setup or looking up Known Folders
under loader lock. Windows [DllMain restrictions](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-best-practices)
still require game-specific loader validation beyond this controlled marker graph.

The DLL installs the existing bounded file/Known Folder/native HKCU hooks from
this configuration during attach. The inherited ordinary key is immediately
available to those hooks; the application does not need to call
BootstrapRegistry, RegLoadAppKey or RegOverridePredefKey from main.
No mutex is selected or rewritten in this storage-only scope. All earlier
unsupported file aliases/operations, cached registry operations, direct native
bypasses, helper propagation and authentication uncertainties remain.

The one integration control is:

~~~powershell
& ./workspace/bootstrap-ordering-build/Release/bootstrap-ordering.exe --run-configured-storage-self-test
~~~

It launches only its own executable with two fresh private directory/key stores,
using directory names outside the old fixture layout. Dependency/TLS/main access
the private registry before any main callback; Roaming/Local file views and
Known Folder paths use the selected store. Two rounds check persistence and
unchanged synthetic host/peer markers. Five malformed process/slot/handle
bindings are rejected; a shared mutex deliberately remains shared.
The probe receipt is private version 8; the independent configuration is
private version 1, 1,136 bytes. Neither is a public product wire schema.

**Evidence ceiling:** this is an integration of our configuration producer,
DLL and controlled processes. It is not Minecraft activation, licensed sign-in,
world persistence, helper isolation or general host/peer access protection.
Every result retains providerReady=false and minecraft=NOT_EXECUTED.

## Rust owner to native configuration bridge

The private AMD64 desktop module
[independent_bootstrap.rs](../../../../apps/desktop/src-tauri/src/engine/independent_bootstrap.rs)
now prepares MC01 and MC02 through the same registered-image creation path.
It uses actual Roaming/Local Known Folder metadata, requires their supported
common AppData parent, and creates new GUID-named pilot storage beneath the
Windows profile, outside that logical AppData tree. Each slot has its own
nonvolatile ordinary registry key bound to the private directory ID and slot.
Existing profiles, account caches and worlds are not copied or migrated.
Fresh data is preserved on failure/drop; no readiness/profile index is added.

Normal unstarted preparation inherits only the read-only configuration section.
The explicit enrolled experiment also inherits the bounded diagnostic section below.
Directory/registry handles are duplicated directly into the original suspended
child with inheritance disabled. The parent publishes PID/FILETIME/session,
root IDs and the current-user native key path, then discards its writable
section view/handle. The receiver disables further section inheritance.
These mechanisms follow the Windows [handle-list contract](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute)
and [Unicode environment rules](https://learn.microsoft.com/en-us/windows/win32/procthread/changing-environment-variables).
APPDATA/LOCALAPPDATA use canonical handle-derived paths, including expansion of
8.3 aliases; inherited overrides are replaced only in the child block.
ASCII/local-drive and current mediation limits still apply.

The private `prepare_registered_enrolled` seam now connects that registered-image
preparer to the same native enrollment operation used by the own-process control.
It first requires a source-reviewed pair of SHA-256 DLL digests. The exact
startup-only candidate is now pinned in non-test builds after explicit operator
authorization. Tests deliberately retain **None**, so --include-ignored cannot
start Education. There is no environment, settings, path or public-command
override. Missing or changed DLL bytes refuse before target/profile preparation.

The fixed location is `independent-bootstrap/` beside the manager executable,
with the fixed selected `bootstrap-enroll.dll` and `bootstrap-compat.dll`
inputs. Before loading code, the owner opens every path ancestor without delete
sharing, rejects reparses/unsupported paths, pins both ordinary DLL files against
write/delete replacement, and hashes those same handles (32 MiB maximum each).
LoadLibraryEx uses the full DLL path with System32-only dependency search; loaded module
file identity and the fixed `EnrollPreparedEducationV2` symbol are checked.
This is exact reviewed-byte loading, not an approved publisher/release policy
or proof of a hostile-host security boundary.

The private ABI receives the original process/thread, PID/birth, read-only
configuration, original pinned image handle and optional observation section. Native validation checks their
image identity/session/AMD64/config/registry binding before inserting the one
fixed sibling compatibility DLL. It does not read the fixture receipt or create
fixture keys, reopens PIDs, accepts target/DLL paths, resumes a child or launches
a helper. Fixture canary preparation is a separate test export. The prepared pair
cancels unstarted children before releasing module/file/directory pins. Only its
explicit private observation transaction below can attempt resume. No ordinary
Launch action or public Tauri command invokes it.

Microsoft Detours remains the existing external development-only static dependency
at the pinned revision, built only with BOOTSTRAP_ORDERING_TEST_ONLY=ON. Its bytes
are covered by the reviewed enroller digest, not dynamically resolved from PATH.
No dependency, installer resource, bundle rule or release format was added.
Production dependency/payload adoption remains undecided.

The default Rust checks keep their own executable suspended. Five explicitly
ignored checks accept the trusted built own fixture through
MIVUBI_BOOTSTRAP_CPP_READER. They exercise the no-DLL ABI receiver, wrong DLL
hashes/write exclusion, and complete enrollment through the shared loader/entry.
The enrollment check prepares separate cooperative instrumentation, then invokes the
same receipt-independent enrollment operation used by the private game seam.
A positive unhooked baseline sees host canaries; enrolled dependency/TLS/main
perform private native registry reads/writes/enumeration. Roaming/Local environment
and Known Folder paths are checked, and logical file writes arrive in private
backing. Both stores retain distinct markers over two relaunches; the unhooked
parent checks fresh host markers unchanged. Wrong birth/pinned-image controls
are refused and original children cancelled. Receipts remain cooperative proof.

On a compatible project toolchain after building the C++ development target:

~~~powershell
$env:MIVUBI_BOOTSTRAP_CPP_READER = (Resolve-Path ./workspace/bootstrap-ordering-build/Release/bootstrap-ordering.exe).Path
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml independent_bootstrap::tests -- --include-ignored --nocapture --test-threads=1
~~~

Executed against baseline `66cea89add4d32163ceb4ae4cb10c4f24b2d25db`: MSVC
19.44 x64 Release /W4 /WX build, all twelve Rust checks (five ignored checks
explicitly included), and all four existing C++ modes pass. The exact Rust
module was compiled without warnings in an isolated Rust 1.96/sha2 0.11.1/
windows-sys 0.61.2 harness whose Education resolver always refuses.
Full Tauri build remains NOT_EXECUTED (project requires Rust 1.98).
Exact own-image CIM check: zero remaining fixture processes.

| Executed input | Git blob |
| --- | --- |
| independent_bootstrap.rs | `1d01c5c7f533de8c79844ca8d51ac25cd8614e66` |
| startup-observation.h | `7184d279d25c0cdebdc98813a034c25c3830896e` |
| compat.cpp | `1c26d77bc99f7cf5728f4a22149f8e350a64d164` |
| enroll.cpp | `4d75fbe9cd3150b1304af6230ef0820d98dcbe9b` |
| probe.cpp | `d8076314cce943e27d19f289f08e91405900d750` |

No registered game was looked up, created, injected or resumed by these checks.
Actual singleton, helper/broker context, native/file bypass coverage, game loader
ordering, sign-in, world persistence and containment remain UNKNOWN /
NOT_EXECUTED. Public commands/provider admission stay unchanged; providerReady=false.

## Bounded startup call observation

The enrolled source path now supplies a separate, anonymous, process-bound
[observation journal](./startup-observation.h). Private journal v1 is 30,784 bytes:
a 64-byte header and 64 fixed 480-byte records. Education configuration v1 and
fixture receipt v8 are unchanged; private enrollment ABI is v2. The native
enroller initializes PID/birth/session and frequency before import insertion.
The DLL validates that binding, clears further inheritance and starts a fixed
30-second QPC window at attach. Capacity is finite; overflow, recursive-logger
loss and clock/late returns have explicit loss flags. Hooks remain installed
for the process lifetime when recording ends; there is no live detach or
observation-triggered game/helper termination.

Only three selected calls are observed in Education mode:

| Call | Recorded metadata |
| --- | --- |
| NtCreateMutant / NtOpenMutant | Desired access, object attributes, counted UTF-16 object name (192 characters maximum with truncation/unreadable flags), raw root/output handle, initial-owner flag, NTSTATUS and ambient LastError. Native LastError is not the outer CreateMutexW wrapper's final error. |
| CreateProcessW | Creation/inheritance flags, BOOL/LastError and returned original process/thread identities plus child birth from those original handles. No command line, application string or environment is inspected. Child image/role/authentication remain UNKNOWN. |

Each record includes thread, raw immediate caller return address and start/end
QPC values, bound to the journal's original game lifetime. No stack, module
provenance or application rejection branch is inferred from that address.
Relative object namespace interpretation can remain UNKNOWN. Original call
arguments/pointers, return value and post-call LastError are preserved. These
observers do not rename objects, add CREATE_SUSPENDED, enroll helpers or select
an authentication account. Existing selected storage hooks still apply.

This is incomplete coverage by design: alternate process APIs, brokers/COM/
activation, calls in other processes, other IPC/file/window singleton routes
and calls before attach are unobserved. Zero records/loss flags cannot establish
absence of a helper or prove a complete lifecycle. Names/addresses/identities
may contain private identifiers; raw bytes remain local and are not committed
or exposed through a public command. The parent copies only after the original
child exits, validates the header, and refuses a live writer. Observation is
cooperative diagnostics; its writable journal never authorizes launch.

The existing Rust-to-native integration control now exercises created/existing/
opened/missing mutex outcomes and successful/failed CreateProcessW calls in
each slot over two relaunches. It checks original errors and returned child
PID/TID/birth, truncation, saturated capacity/loss flags and after-exit copy.
Expiry is exercised by moving only the own fixture's clock into the past after
checking the 30-second boundary; this is not a 30-second Minecraft capture.
The sole new internal child argument exits only our own fixture immediately;
it supplies a process-creation control and accepts no target path.

### Explicit startup transaction and exit extraction

The private prepared pair has one consuming
`resume_for_authorized_observation` operation. It preflights both original
process lifetimes, calls ResumeThread once per slot and stops on error. Only
previous suspend count 1 is Resumed; unexpected counts or API errors are
Unconfirmed, with later slots NotAttempted. These are startup-call outcomes,
never game readiness. The Windows [ResumeThread contract](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-resumethread)
defines those counts; there is no retry to force an unknown child to run.

Before the OS call the child irreversibly disables cancellation. Partial
failure, observation expiry and owner Drop cannot terminate any attempted
child. Preflight errors and untouched children retain unstarted cleanup.
The returned owner retains the same handles/config/stores/journals and payload
pins. Closing the manager releases its handles/pins and loses journals that
were not exported; it preserves attempted processes, following Windows
[CloseHandle semantics](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-closehandle).
An uncertain call may leave a suspended child; this is reported without an
automatic retry or kill. Preservation does not prove successful startup.

After the operator closes a game normally, `save_after_exit(slot)` polls
the original process without a blocking wait. Live writers refuse; other wait
errors propagate. A signaled exit permits header-validated snapshot extraction
and the observed exit code. [GetExitCodeProcess](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getexitcodeprocess)
does not by itself prove normal user closure, and code 259 is not used as a
liveness test. No crash/normal-close inference is made.

Extraction revalidates/pins the existing slot directory and all ordinary
ancestors, then writes/syncs `startup-observation-v1.bin` with create-new
semantics. No arbitrary output path, overwrite, profile migration or deletion.
A failed write can leave a partial local file and later attempts refuse it.
Raw metadata stays local, outside Git and public commands. Keep the owner alive
until both games are closed and each journal is extracted.

The existing two-slot/two-round integration uses this transaction and checks
after-exit persistence, invalid slots and overwrite refusal. Two additional
own-executable ownership checks cover preflight/NotAttempted cleanup and partial
resume uncertainty. Optional ready/finish barriers use existing private receipt
events solely in the Rust-owned C++ control; they prove the first child remains
live after owner Drop while the second remains suspended. The test alone recovers
that second control and releases both normally. They also verify live-writer,
cancel and repeat-resume refusal. No new CLI, receipt version or observer was
added.

### Operator-authorized same-manager invocation

The same desktop manager has one fixed internal argument:
`--education-startup-observation-once`, dispatched before UI startup. Extra
arguments refuse before any game lookup; there are no target/PID/output-path or
hash options. A session-local Windows mutex serializes experiments, and the
existing native workspace inspection refuses when a registered game is already
running. That game is preserved for normal operator closure. No new launcher,
Tauri command, driver/service, account automation or public admission is added.

Production builds pin only the reviewed development DLL pair: enroller SHA-256
`7646bd2e46a5f3948454984fc81dc11342691e69ce8603c329f15b12140ece83`,
compatibility `292e1c4c472d25c5934a210739cc8e9f3029bfd4160250a87c9e67ecc5f6ac06`.
These files are manually staged in the fixed sibling directory, with the
external Detours license; they are not installer resources or an approved
production provider. The compatibility DLL directly imports SHGetKnownFolderPath
from Shell32, so it does not assume the target already loaded that module.
No Known Folder invocation or LoadLibrary is added inside DllMain.

The runner retains the existing returned owner and polls after-exit extraction.
A permanent journal error in one slot is reported while the other slot remains
observed. Status-only JSON reports resume-call outcomes, exit codes and journal
save/error; no raw names, paths, environment, credentials or game content.
No timeout kills a game. Keep the runner alive until normal closure; ending it
early preserves attempted children but loses journals not exported.

In the actual Rust 1.98.1 preview-feature debug Tauri build (locked dependencies,
no default features), all twelve existing bootstrap tests pass with the exact
new module; the integration now checks both successful exports and continued
second-slot handling when both old journal paths reject overwrite. The same
built manager rejects extra arguments with exit 2/InvalidData and an existing
registered MC01 with exit 2/AlreadyExists; PID/birth metadata confirms MC01
preserved. This is build/control evidence, not an Education injected startup,
sign-in or GUI/GPU/input acceptance. No new fixture is added. Existing build
warnings remain. Runtime authorization and the sole current next action belong
in planning/development.md.

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
overflow, invalid stages and malformed transport fail. Our compatibility DllMain never waits for a controller handshake or performs
registry/profile/COM/Known Folder setup. Controlled dependency/TLS native registry
probes below deliberately exercise the installed gate, without high-level setup.

The parent reads only after confirmed child exit, with bounded waits. Cleanup
uses the original process handle; no PID reopen or unrelated termination occurs.
Failure receipts distinguish incomplete operations and cleanup uncertainty.
JSON contains fixed stage/case labels and booleans/counts, with
providerReady=false and minecraft=NOT_EXECUTED. It emits no raw PID, FILETIME,
HANDLE, username, installed game path, credentials, caches or world contents.

In application-hive mode, selected registry open/query/set calls are denied from
first import until the explicit post-loader activation below. The separate
ordinary inherited-key mode supplies private values before dependency/TLS.
Only the selected synthetic directory tree is redirected. In the default ordering mode,
"canary" means a marker at a would-be protected call site and "arm" is a shared
flag. In mutex mode the flag follows installation of file/IPC/Known Folder and eight registry entry-point hooks in one transaction.
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

The private **version-6** receipt (2,048 bytes, not a product wire schema) carries
native/file/registry masks and the file round/root handles. It still binds the original child PID/creation FILETIME,
slot/test name and eight explicitly inherited handles: section, four phase events,
logical/private directories and one fresh synthetic HOST registry key. Both children hold
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
receipt is version 6; no product wire schema changes.

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
remain required. The current private receipt version is described above; product schemas remain unchanged.

This is process-local selected API compatibility, not Windows profile registration
or complete Known Folder virtualization. SHGetFolderPath, WinRT, shell/COM object
interfaces, packaged redirection flags, LocalLow, other IDs, inherited child
contexts, environment mutation and arbitrary hardcoded profile aliases remain
unimplemented/unverified. The process can still mutate its own environment.
The bounded post-loader predefined HKCU view is described below. Complete
registry/authentication isolation is unproved; no game was launched/injected.

Contracts:
[SHGetKnownFolderPath](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath),
[KNOWN_FOLDER_FLAG](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/ne-shlobj_core-known_folder_flag),
[Known Folders](https://learn.microsoft.com/en-us/windows/win32/shell/known-folders).

## Cached registry enumeration — 24 October 2026 WIB

Exact baseline: 4ad32abda10be250223d508099ece58d2cdf5b3d.
NtEnumerateKey and NtEnumerateValueKey now use the same existing cached-handle
route as query/set, installed in the first-import transaction. A selected cached
HOST handle reopens the slot key only after checking its original granted
KEY_ENUMERATE_SUB_KEYS or KEY_QUERY_VALUE right. The original native function
receives the caller's index, information class, buffer and length unchanged;
status, required length, partial bytes and LastError are preserved. Unsupported
routes and impersonation refuse; unmapped handles retain native behavior.
Application-hive preactivation denies enumeration; ordinary inherited backing
supplies private entries before dependency/TLS/main.

The same startup controls now include native value/subkey enumeration. An
unhooked positive control sees its fresh synthetic HOST value and HostOnly
subkey. Protected ordinary startup sees its slot's advancing private value and
no HOST subkey; relaunch startup accepts the existing private Native/Settings
subkeys without assuming a registry enumeration order. Application-hive startup
denies both calls with unchanged output/length. No real registry data is seeded
or read.

Both existing pair modes reuse one enumeration regression: compare a cached
HOST handle against its genuine private handle for two APIs, information classes
0/1/2/99, indices 0/1/2/99 and lengths 0/1/16/64/256. It compares native status,
returned length and all 256 output bytes, including untouched bytes. Win32
RegEnumValueW/RegEnumKeyExW return only private names. Query-only and
enumerate-only duplicated handles allow their respective operation and deny the
other with equivalent outputs; cached enumeration under impersonation refuses.
The parent rechecks the unchanged HOST value and sole HostOnly subkey.
No new controller, backing store, transport field or switch was added.

**Executed proof:** Windows x64 SDK 26100, MSVC 19.44.35226, MSBuild 17.14.40,
CMake 4.3.2, Release /W4 /WX /permissive-. Configure/build and all three fixed
commands exit 0. Both pair modes complete two rounds with registryChecks=65535
(8191 prior checks + 8192 native buffer/status equivalence + 16384 Win32 private
enumeration + 32768 rights/impersonation). Ordinary earlyPrivate=7/7;
application-hive earlyDenied=7/7 and unhooked earlyAllowedControl=7. File/AppData,
mutex, invalid-hive, invalid-backing, slot mismatch, ordering/failure/timeout and
persistence controls retain their expected results. Receipt remains v7,
2,064 bytes. No remaining bootstrap-ordering.exe was observed.

| Executed source | Git blob |
| --- | --- |
| compat.cpp | a43dad57e9e25258f75555fa108057c8695c2ced |
| probe.cpp | aacb8af96e20a3483e6e47fbddaff6ba12b13ce1 |
| receipt.h | d4c4a84d5641c8ff563cb29569d4bd3bc97c0fc0 |
| dependency.cpp | f33dcf4125062fd63992d42d78729845560282f2 |
| CMakeLists.txt | 8aebd659bf90c05aaf6c2f166610c36834268a53 |
| compat.def | 0bca64aeff66d33b9ab881152563370774906219 |

Local evidence: workspace/tmp/registry-enumeration-1792815909405.
SHA-256:

| Artifact | Hash |
| --- | --- |
| bootstrap-ordering.exe | 12b1bf590c1fc91b90dff6ea06d7bd562272bec6e7761b5a248f77928c970ffc |
| bootstrap-compat.dll | b2cd85f02df42e5f1b8e4378469902eb9f93f964a1781bf476b8d283dfb59a62 |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | 8da2467f825ce1343f084a56fd5b1632b98ab669cac411fd17feab8b327126a4 |
| inherited.jsonl | bc110847007c386f2f00442da4182cd97bf8be3c971f4219b2a96668f8803a2e |

The Microsoft [enumerate-key contract](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-zwenumeratekey)
and [enumerate-value contract](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-zwenumeratevaluekey)
describe the distinct rights, buffer/status results and use of Nt names in user
mode. No driver/kernel invocation or blanket syscall support is inferred.

**Limits:** selected, resolved cached-handle enumeration is mediated; unknown
key-name queries, multi-value reads, deletion/rename/security/transactions,
concurrent lifetime changes and direct-syscall bypass remain unresolved.
This is neither complete containment nor authentication isolation. No actual game
launch/trace/injection, account/token/world access or service/driver action
occurred. Full product, Minecraft and two/four-client acceptance NOT_EXECUTED;
production provider and isolated Launch remain disabled. Historical proof below
belongs to its recorded source revisions.

## Ordinary inherited registry startup — 24 October 2026 WIB

Exact baseline: 919fbdae2a8d04a1a35dcda10a8e2d159de8c9c3.
The new **test-only ordinary-key candidate** supplies synthetic private values
during dependency DllMain, EXE TLS and the first main canary. It does not inherit
an application hive or claim an exception to RegLoadAppKey's process rules.

~~~powershell
& ./workspace/bootstrap-ordering-build/Release/bootstrap-ordering.exe --run-inherited-registry-self-test
~~~

The existing controller creates two fresh REG_OPTION_NON_VOLATILE ordinary
registry roots under its unique synthetic Software test prefix. Each root name
binds the existing slot's directory volume/file ID and A/B label; the child
validates the inherited key's native name and granted rights against those
existing file/slot identities. The fixed handle allowlist adds only the own-slot
key (nine handles total); the peer key is not inherited. Missing keys, a wrong
slot's key and an event handle masquerading as a key reject attach before arm.

The same DLL registry owner publishes the validated ordinary key before the
existing hook transaction is armed. It does not load/create a hive, override
HKCU or perform profile/COM setup in DllMain. Native host-user path opens and
cached host-key query/set use the existing translation. Its own backing subtree
bypasses host-path remapping so it cannot recursively map into itself; direct
container/peer backing paths under that reserved prefix are unsupported.
Main-thread BootstrapRegistry then applies the existing predefined HKCU mapping,
idempotently. The application-hive mode retains its distinct main-thread load,
early denial, invalid-hive and REG_PROCESS_APPKEY load-lock controls.

The startup canary checks both a native logical open and the inherited cached
HOST handle against a slot-specific value, then advances that private value at
each stage. Dependency/TLS/main are bits 1/2/4 in earlyPrivate. Both children
remain live through the existing ready/go/done/finish phases and exercise the
shared registry regression checks. After normal exit, the parent closes and
reopens each ordinary root without recreation/reseeding, verifies both slot
values and absent peer-only values, and launches the second round. HOST remains
unchanged. Receipt v7 is 2,064 bytes; this is private fixture transport only.

**Executed proof:** Windows x64, SDK 26100, MSVC 19.44.35226, MSBuild 17.14.40,
CMake 4.3.2, Release /W4 /WX /permissive-. Configure/build and all three final
commands exit 0. Ordinary earlyPrivate=7/7 and registryChecks=8191/8191 in both
rounds; all three invalid-backing controls reject. In ordinary mode registry bit
64 means reserved backing-container rejection, **not** application-hive
load-lock isolation. The original ordering positive/failure/timeout controls
and mutex/file/AppData/application-hive pair also pass unchanged expectations:
earlyDenied=7/7, earlyAllowedControl=7, registry=8191/8191,
file=1048575/1048575, Win32 mutex=127/127, native mutex=511/511.
No remaining bootstrap-ordering.exe was observed after the final runs.
The first wrong-object control accidentally repeated a handle in the Windows
inheritance list; replacing it with a distinct event made it exercise DLL
rejection rather than process-creation failure. Temporary diagnostics were
removed before the final build/runs.

Executed source blobs:

| File | Git blob |
| --- | --- |
| compat.cpp | ebcc7d7ac0f55e4a406ff160a766062e42d7439c |
| probe.cpp | b307460c2ddcc1d06f6f3593507b541ff575a6da |
| receipt.h | 07159770f6870925105409b99ac31fc70e6701cd |
| dependency.cpp | f33dcf4125062fd63992d42d78729845560282f2 |
| CMakeLists.txt | 8aebd659bf90c05aaf6c2f166610c36834268a53 |
| compat.def | 0bca64aeff66d33b9ab881152563370774906219 |

Local evidence retained under workspace/tmp/inherited-registry-1792815456654;
no raw key names, file IDs, user paths or handles are published.
SHA-256:

| Artifact | Hash |
| --- | --- |
| bootstrap-ordering.exe | 5b6118ebf7c7879234aa8d970d1785e790f0465005bfadbb5327485740ce899b |
| bootstrap-compat.dll | 2221d748088ef85c7c8c646f18c4db8ba5f8a201f035f4be3f513d131268308e |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | b34ec1559e3fb3727ad7e336550455067fab6119338d49d6016ac527ae994707 |
| inherited.jsonl | aaaa57f829d12344d0cdf2ac239d8ad1167ad4477b24a15becf87653ab9da05d |

**Limits:** these are ordinary keys in the Windows user's registry, not portable
HKCU.dat files. Fresh synthetic keys are retained; no migration or deletion of
old stores is performed. This candidate has no REG_PROCESS_APPKEY load lock and
does not establish peer ACL denial, secrecy from the host user, complete
mediation or production readiness. Cached enumeration/multiple-value/delete/
rename/security/transaction operations, unresolved paths and direct syscalls
remain uncovered. The existing test AppData/IPC hooks are installed, but this
new command asserts only registry behavior, not a new combined game boundary.
No authentication broker, game process or world was accessed; full product,
Minecraft, reboot and two/four-client acceptance remain NOT_EXECUTED.
DisabledIndependentProvider and MC02–04 admission remain closed.

Official contracts:
[RegCreateKeyExW](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regcreatekeyexw)
documents ordinary nonvolatile keys and SECURITY_ATTRIBUTES handle inheritance;
[Handle Inheritance](https://learn.microsoft.com/en-us/windows/win32/sysinfo/handle-inheritance)
includes registry keys. Those contracts support this backing choice, not
Minecraft compatibility or complete user-mode hook enforcement.

## Inherited application-hive enrollment decision — 24 October 2026 WIB

Reviewed exact source: 1910a662711080834d29382f48b2eeea148da3e4.
**Decision: do not select parent-loaded application-hive handle inheritance alone
as a documented production mechanism for private data before dependency/TLS.**
This closes that specific support claim; it is not a runtime finding that Windows
cannot inherit or use such a handle.

The [RegLoadAppKeyW contract](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regloadappkeyw#remarks)
requires each participating process to call RegLoadAppKey. REG_PROCESS_APPKEY
also prevents another caller loading the active hive. Generic
[registry-key handle inheritance](https://learn.microsoft.com/en-us/windows/win32/sysinfo/handle-inheritance)
does not document an exception to the application-hive participation requirement.
An exported function, successful handle duplication, or a single-host fixture
would not establish that exception.

| Proposed step | First unresolved contract |
| --- | --- |
| Parent loads slot HKCU.dat and passes its handle to a suspended child | Inheritance alone does not satisfy the documented per-process application-hive load step. |
| Child loads before dependency/TLS | Current bootstrap runs in main; no supported earlier load boundary has been established without introducing prohibited loader-time registry setup. |
| Remove REG_PROCESS_APPKEY | Does not remove the per-process load requirement; it also discards the load-exclusivity property tested by the current implementation. |
| Treat early denial as readiness | Prevents selected host accesses but does not supply private values needed at startup or establish game compatibility. |

**Proof ceiling:** source/official-contract review only. Application-hive inherited
handle behavior, new builds/runtime and Minecraft acceptance: NOT_EXECUTED.
Actual OS behavior and broader support remain UNKNOWN. Existing successful
controls belong to their recorded source snapshots and are not promoted.

Ordinary non-application registry keys are a distinct candidate:
[RegCreateKeyExW](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regcreatekeyexw)
documents inheritable returned handles and nonvolatile storage. That does not
approve a provider. Any next implementation must reuse the existing DLL/slot
path, use only fresh synthetic ordinary keys, bind the inherited key to its slot,
exclude its private subtree from host-path translation and prove early private
values/persistence with the same dependency/TLS controls. Do not add a second
fallback/readiness store, copy real account registry data, weaken old tests or
migrate/delete existing test/game stores. Current continuation remains solely in
planning/development.md.

## Early registry boundary — 24 October 2026 WIB

The original DLL now installs RegOpenCurrentUser, NtOpenKey, NtOpenKeyEx,
NtCreateKey, NtQueryValueKey and NtSetValueKey alongside its existing hooks in the
first-import transaction. Before privateHive is published, selected current-user
opens and cached-handle value query/set return ACCESS_DENIED. There is no host
fallback. Unselected namespaces/unresolved shapes retain their documented limits.

The controller obtains current-user root-name metadata before launching children
and transports it only in their anonymous PID/creation-bound section. No SID or
native user path is written to logs/files/Git. Initialization in our DllMain
checks/copies that bounded metadata and resolves already-loaded exports; it does
not open a user profile, load a hive, override HKCU or initialize COM.

Only one new synthetic HOST branch handle is inherited, never the real user root.
Dependency DllMain, EXE TLS and the start of main each attempt three native
operations: absolute open of the fresh HOST test branch, query of its HOST marker
through the inherited cached handle, and a different marker write. All must be
denied with LastError preserved and the query output/length untouched. The parent
checks that the synthetic HOST value remains intact. These are controlled native
probes, not a claim that arbitrary registry APIs are loader-safe.

A separately controlled child without the compatibility DLL executes the same
three probes successfully at all three stages. Its write keeps the exact HOST
value, and it exits before file/hive tests. This positive control detects missing
or malformed probes; it is never a production fallback or provider launch route.
Stage masks use 1=dependency, 2=TLS, 4=main: protected earlyDenied=7/7, earlyAllowed=0;
unhooked control earlyAllowed=7, earlyDenied=0. Original four ordering cases remain.

The main-thread BootstrapRegistry then loads/maps its slot hive and publishes
the retained handle atomically. No second registry-hook installation occurs.
Once active, cached HOST-branch query/set calls reopen the corresponding private
key for that operation. NtQueryObject basic metadata verifies the original
handle's granted query/set rights, preventing a read-only host handle from gaining
write access through translation. Temporary private handles close on normal and
protected-exception paths. Tests duplicate the inherited handle read-only and
check query success/write denial after activation.

This establishes denial then private routing only for the tested graph and
selected entry points. It does not prove that a game tolerates denied early
registry requests, that the gate covers code before this import, or that private
registry data is available to dependency/TLS. Cached enumeration, multiple-value
queries, delete/rename/security/transaction paths, unrecognized handles, direct
syscalls, other users and external helpers are not covered here.

API contracts:
[Handle inheritance](https://learn.microsoft.com/en-us/windows/win32/sysinfo/handle-inheritance),
[RegCreateKeyEx](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regcreatekeyexw).
Registry-object metadata is not registry value/authentication content.

## Post-loader private HKCU — 24 October 2026 WIB

The same DLL now exports **test-only BootstrapRegistry**, called by the fixed
owned child's main thread before its registry/file/AppData tests. It validates
the PID/creation-bound receipt, MutexCase/armed state and the configured private
root handle/file ID, then loads Registry/HKCU.dat beneath that same A/B root
with RegLoadAppKeyW(KEY_ALL_ACCESS, REG_PROCESS_APPKEY). Windows creates an empty
hive when that file is absent. No account/profile hive is copied or imported.

The current-user namespace is already bound to controller-supplied ephemeral
metadata by the early gate. A real root handle opened relative to the application
hive becomes this process's HKEY_CURRENT_USER through RegOverridePredefKey. That
mapping and hive handle live until exit. Successful main-thread activation
publishes the hive atomically; failed load/map returns FALSE and the owned child
stops without opening registry test data on the host. Repeated initialization
on the same owned main thread is idempotent. A failed load/map returns FALSE; the controller-owned child
stops before any test registry operation, with no host-HKCU fallback.

This is an explicit **post-loader** step: no hive loading, HKCU remapping or
COM setup was added to DllMain/TLS. Only the controlled native probes above run
in dependency/TLS callbacks. The existing Armed bit now denotes installed hooks and the selected early denial
gate; it is NOT proof that private HKCU data was available during loader/TLS. There is no product provider
readiness flag, production registry API, arbitrary hive-path/PID option or
background service. Advapi32 is an SDK link, not an installed dependency.

The controller creates exactly one fresh uniquely named synthetic HOST key under
HKCU/Software for each run, refusing an existing test key, and writes a HOST DWORD.
Children access that same logical path through remapped HKCU. New hives initially
lack it, then contain independent A/B markers, slot-only values and a relative
Settings subkey. RegFlushKey explicitly persists this test data. No existing
account/auth key/value is inspected or copied. The fresh HOST test key and TEMP
hive trees are retained; existing host registry data is neither removed nor
recursively traversed. “Host unchanged” means the fresh HOST marker stays intact,
not that the run creates no new synthetic registry key.

Registry mask **8191** per child/generation:

| Bit | Actual check |
| --- | --- |
| 1 | In-child DLL initialization succeeds twice on the owned main thread. |
| 2 | HOST test path is absent in a new private hive; relaunch sees its own previous marker. |
| 4 | Create/open disposition, writes and exact value enumeration contain only Marker plus own OnlyA/OnlyB value. |
| 8 | Read-only handles deny writes, typed reads agree and relative Settings writes work. |
| 16 | Explicit flush of the private hive succeeds. |
| 32 | After both children are ready, own marker is unchanged and peer-only value is absent. |
| 64 | Peer application-hive load is refused while the other owned process holds it. |
| 128 | RegOpenCurrentUser returns a private root with requested read access; relative lookup sees the same private marker. |
| 256 | NtOpenKey and NtOpenKeyEx on an explicit native user path return own data and preserve LastError. |
| 512 | Native zero-length counted relative open and direct query/set through the inherited synthetic HOST branch map privately; caller data stays unchanged and duplicated read-only handles cannot write. |
| 1024 | Native create disposition and writes agree with HKCU and persist as a Native subkey. |
| 2048 | Selected link options, invalid-buffer, missing and conflicting-root shapes are controlled; impersonation is rejected and reverted; unrelated root and oversized unrelated-path results remain native. |
| 4096 | A native-opened handle still reads own data after both children are ready. |

The parent verifies its HOST marker during/after each pair, confirms it cannot
load either live REG_PROCESS_APPKEY hive, then independently loads both hives
after normal child exit to read persisted A/B values. Closing those observation
handles lets the next generation reopen the same private hive. This observed
API load exclusivity is not a claim of raw-file confidentiality from the host user.

A separate negative control places only an invalid synthetic hive file in fresh
slot A. The owned child exits with its operation-failure code before registry
tests (registryChecks=0), while HOST stays unchanged. After confirmed exit the
controller renames only that new invalid test file, preserves it and proceeds
with a newly created empty hive. No corrupt-file fallback or privilege escalation.

RegOpenCurrentUser now opens a new handle relative to the private hive. It honors
requested access and returns caller-owned handles, preserving entry LastError.
Any thread impersonation token is rejected; no token user/group/credential data
is read. The synthetic self-impersonation control checks refusal and successful
RevertToSelf. This does not implement impersonated-user profiles.

Native routing matches the cached current-user root at a namespace boundary.
Absolute counted names and relative names reconstructed from NtQueryKey parent
metadata map to the same private root. The current bounded ASCII component policy
is reused for selected suffixes. OBJECT_ATTRIBUTES is copied; flags, access,
class/title, output/disposition pointers are preserved. Selected nonzero open/
create options (including registry link operations), malformed conflicting
absolute/root shape and impersonation are refused. Other namespaces and
unresolved relative roots pass to originals. Native NTSTATUS and entry LastError
are preserved, including protected parameter-read exceptions. The parent root
metadata lookup is not an authenticated handle enrollment mechanism.

Before bootstrap the protected children cannot read/write the synthetic HOST
marker; the separate unhooked positive control can. After activation, selected
native/current-user and cached-handle query/set routes see private data instead. The controller also reads native-created
subkeys directly from both persisted hives after exit. User-root paths/handles exist only in the anonymous test transport and process
memory; none are written to source, log files or evidence artifacts.

**Limits:** private data availability before main, cached enumeration/multiple-value/delete/rename/security operations,
other-user roots, unresolved/long relative roots, transacted native entry points,
other predefined keys, registry links outside the selected routes and descendant/
external helper propagation remain UNKNOWN/unimplemented. Direct syscalls,
hook tampering, concurrent enrollment/all-thread quiescence and game compatibility
are unproved. This remains cooperative selected mediation, **not a security
boundary**. The single owned-main-thread bootstrap still starts after loader/TLS;
the Armed marker implies the selected denial gate, not private data at startup. No actual
Minecraft/helper/authentication access or game injection was performed.

Contracts:
[RegLoadAppKeyW](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regloadappkeyw),
[RegOverridePredefKey](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regoverridepredefkey),
[Application hive operations](https://learn.microsoft.com/en-us/windows-hardware/drivers/kernel/filtering-registry-operations-on-application-hives),
[RegOpenCurrentUser](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regopencurrentuser),
[Native key creation](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-zwcreatekey),
[Native extended key open](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/nf-wdm-zwopenkeyex).

## Evidence

### Executed early gate and cached-value slice — 24 October 2026 WIB

Exact baseline: 9a0e90e94064229957d5b797f1b4b4c07d2e5784.
Changed executed input blobs:

| Input | Git blob |
| --- | --- |
| compat.cpp | 123ddbb88ecf3c6e9b23b8298d4dab3bd553f6ff |
| probe.cpp | 03278d782adfa2603996714d3829eb6cc84fb549 |
| receipt.h | eb07b43e3d66357df2a8dd046f56e6d12b0107bf |
| dependency.cpp | f33dcf4125062fd63992d42d78729845560282f2 |

Private receipt v6 is 2,048 bytes. Build config/export inputs and clean pinned
external Detours are unchanged. Windows x64 26100 / CMake 4.3.2 / MSVC 19.44.35226
Release /W4 /WX configure/build and both final commands exit **0**. Protected
earlyDenied=7/7; separate unhooked earlyAllowed=7. Cached query/write isolation and
granted-access checks pass, together with registry 8191/8191, file/AppData
1048575/1048575, native mutex 511/511, Win32 mutex 127/127, two-generation
persistence, invalid hive, mismatches, junctions and all four ordering controls.
No remaining test image in CIM lookup; tracked checkout clean.

The positive control initially exposed an invalid native name at TLS: formatted
wide-string construction was not reliable before EXE CRT initialization. The
probe now constructs its bounded path directly, without a formatter/CRT startup
dependency. Final diagnostic line/status logging was removed. No test expectation
or native error was weakened. Full product, actual game startup/authentication and
two/four licensed clients remain NOT_EXECUTED.

Local artifact SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| bootstrap-ordering.exe | 78db7166709718c5b447f6b25a7c08cc6f3d2c41fc309963114ad2732ea20540 |
| bootstrap-compat.dll | 004511f864041afbbfc5e5290bd38b3487315668d2df40cb175950fd358ecefe |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | b34ec1559e3fb3727ad7e336550455067fab6119338d49d6016ac527ae994707 |

Earlier evidence retains its historical SHA/scope.


### Executed current-user/native registry slice — 24 October 2026 WIB

Exact baseline: 39dc07dcad1f8e232f44dc1a6d16cd88ca301005.
Changed executed blobs: compat.cpp 7700d375428e9c716e6751d9bb5a978909707d1f;
probe.cpp 9b3f67487e9bb0c2c9447f9b7a0159ad00021117.
Receipt v5, exports/build inputs and clean pinned external Detours are unchanged.

Windows x64 26100 / CMake 4.3.2 / MSVC 19.44.35226 Release /W4 /WX configure/build
and both final commands exit **0**. Registry 8191/8191, file/AppData 1048575/1048575,
native mutex 511/511 and Win32 mutex 127/127 pass across two concurrent pairs
and relaunch. Explicit native absolute/relative/ex opens and native create agree
with HKCU/RegOpenCurrentUser; persisted native Slot values and original HOST
markers are independently verified. Impersonation/refusal/revert, unselected
short/oversized namespace controls, invalid hive and all prior controls pass.
Independent CIM lookup finds no remaining test image; tracked checkout is clean.
Full product, pre-main registry, game/authentication and two/four-client NOT_EXECUTED.

Local artifact SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| bootstrap-ordering.exe | 5e338bf3b68578c1aa8c34995e14cb7c65cb9e8a5e02d0138372690777fe145a |
| bootstrap-compat.dll | 62abbc622978a2bd3913d84068583f73fe7302bf2e3e4b574f50aa17d33c256b |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | e6e99ebc9a52dc735265f023319a282ca933746130f49e654b827d9fabf691b8 |

Earlier evidence retains its historical SHA/scope.


### Executed predefined HKCU slice — 24 October 2026 WIB

Exact baseline: 4469a7281cc214cbfbf2664b762b8d9dd3774781.
Executed changed inputs:

| Input | Git blob |
| --- | --- |
| compat.cpp | eed657a3ef8e7196ec33bf1746e11a74716cf8a1 |
| probe.cpp | ca5be511d64755e1a9f0d8bd1e83f16f4e051931 |
| CMakeLists.txt | 8aebd659bf90c05aaf6c2f166610c36834268a53 |
| receipt.h | 2a2e0844bb32656165efad05f5416be75a06150b |
| compat.def | 0bca64aeff66d33b9ab881152563370774906219 |

Receipt advances to v5; the new LONG occupies prior alignment padding, so size
remains 1,000 bytes. dependency.cpp is unchanged; external Detours remains clean
at 1dbd4ea281d73d478aa8d09a8b83fbd265f715f7. Windows x64 26100,
CMake 4.3.2 / MSVC 19.44.35226 Release /W4 /WX build succeeds. An initial
signed/unsigned comparison warning was corrected; no check was disabled.

Both final self-test commands exit **0**. Registry masks 127/127, file masks
1048575/1048575, Win32/native mutex masks 127/127 and 511/511 pass in two
simultaneous pairs across relaunch. Independent persisted-hive/host checks,
live load-exclusivity and invalidHiveRejected=true pass, alongside all prior
AppData/file/junction/mismatch controls and four ordering/failure/timeout cases.
No remaining test image in CIM lookup; tracked checkout clean. Full product,
pre-main registry, Minecraft/authentication and two/four-client NOT_EXECUTED.

Local artifact SHA-256:

| Artifact | SHA-256 |
| --- | --- |
| bootstrap-ordering.exe | 57a42cbb1a1efd4ed7293ab352bdeb1f730657a1b8d6ceb1420e08d9747b81d6 |
| bootstrap-compat.dll | 0c575a85f1e37ad7d4e07c1f527908a72d09e54644c874a1160c3a5f51f779e5 |
| ordering.jsonl | 40927efb7a951dda0eccd4323ec78f5b80d713673bee402d9ebd632e0f460c36 |
| pair.jsonl | 265d8e97f0619208f3d7ca393a84ef3ad855251ba3c41fc84280737c00d3fe3f |

Earlier evidence retains its historical SHA/scope.


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
