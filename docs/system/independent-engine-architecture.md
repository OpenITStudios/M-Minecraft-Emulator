---
id: document.system.independent-engine-architecture
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Independent Minecraft Session Engine — source-first architecture

## Product invariant and licensing boundary

### Root-owner SID and effective peer ACL checks (23 October 2026)

The existing Windows-only `independent_root_access.rs` now inspects the
directory's ACTUAL owner SID from its `GetSecurityInfo` HANDLE-bound,
READ_CONTROL-guarded, non-NULL DACL descriptor, and the original two
processes' `TokenUser` SIDs from bounded `GetTokenInformation` buffers.
All SID pointer/buffer sizes and validity are checked before `EqualSid`.
Nothing is serialized: only equality booleans leave the native function.
The root's owner must match its purported owning process user and must
NOT match the other process's user before any candidate denial can be
reported. This explicitly prevents equal-user MC02/MC03 fixtures from
appearing isolated merely because a directory has a chosen slot name.

The Windows HANDLE-based core is kept as a private, non-test
`review_process_handles` function while the only current entrypoint
remains a Job-owned fixture that verifies exact process lifetimes and
retained file-object lineage. Future service code must separately
attest its process handles and SERVICE SID; this function does NOT
authenticate arbitrary callers or confer access.

The pre-existing read-only `AccessCheck` owner now also queries each
peer's create-subdirectory, traverse, file attributes, extended
attributes, security-descriptor read, delete-child, delete-directory,
WRITE_DAC and WRITE_OWNER rights in addition to list and create-file.
Any **evaluated** right granted to the peer blocks
`PeerDeniedCandidateOnly`, even if it cannot list/create files.
This checks specific directory rights, not every way privileged
code can access a root or its children. ACL evidence for both the
retained root and the actual child slot directory is examined in the
same source-only disposable native fixture path. Root/slot FILE HANDLE
lineage and exact process identity are rechecked around these calls.
Pure tests cover owner/peer SID candidate policy and each unsafe peer
right; the Windows fixture tests remain opt-in test-only.

**Critical limitation:** comparing TokenUser with file owner is a
strict preliminary provenance check, NOT proof of a future SCM
service's unique SERVICE SID/group, trusted ancestor ownership,
privilege-restricted access, inherited ACLs on files, a race-free
security descriptor, game-path interception or Microsoft WAM. Both
test processes normally use the same Windows user and can access
both roots. These diagnostics must NEVER activate Client 02–04.
Windows compilation/native execution remains NOT_EXECUTED in ChatGPT;
CI remains disabled.

### Windows HANDLE-relative file I/O consumes canonical Rust storage plans (23 October 2026)

The original Windows-only `apps/desktop/src-tauri/src/engine/independent_private_files.rs`
fixture now calls the canonical
`engine/runtime-core/src/independent_engine/storage.rs::plan_private_file_access`
instead of relying only on its native single-component parser. An actual
disposable job-owned OS PID/full creation FILETIME, the fixed MC02–MC04
slot, `LocalFiles` domain and read/create action determine the route
request. The returned `PendingPrivateFilePlan` is NOT an access grant:
before any `NtCreateFile` marker operation, the adapter checks its
publicly mutable fields against a FRESH trusted fixture owner/process
observation and regenerates the plan using that owner; altered slot,
process, domain, action or component list is refused. Only ONE checked
relative marker component is allowed by this fixture's native HANDLE.
Arbitrary paths, nested plans and host-account destinations remain
outside this I/O interface.

The retained `SlotFiles` HANDLE now also reopens its fixed `mc02`,
`mc03` or `mc04` directory relative to the pinned original root
HANDLE, comparing the reopened volume/file index to its retained slot
object before and after bounded marker I/O. This closes the missing
slot-level parent/child lookup check without treating a path string as
filesystem ownership. Two Windows-only native regressions were
authored for valid owner-bound read/create, forged plan fields,
unsupported nested paths, slot-name substitution and file identity
mismatch. Existing fixture tests use the same canonical path.

**Evidence ceiling:** exact GitHub source/static review; Windows Rust
compilation and native tests NOT_EXECUTED. Roots remain disposable and
created beneath the caller's temp ancestor, WITHOUT service-owned
provenance, verified exclusive peer DACL rights or transparent Minecraft
Roaming/HKCU I/O interception. It cannot authorize new sessions,
Microsoft login or four concurrent clients. No Tauri command or
production launch/provider route is enabled.

### Parent-HANDLE-bound disposable private roots (22 October 2026)

The unmounted Windows native private-file fixture now creates its fresh
root with NtCreateFile(FILE_CREATE) relative to a retained parent directory
HANDLE. The root factory rejects MC01, pre-existing root-name collisions,
final reparse objects, unexpected volume changes and stale fixture process
identity. It retains the parent and root's OS volume/file-index identities
and reopens the exact relative root on each bounded fixture operation,
denying a changed parent-child HANDLE relationship. Directory open rights
include READ_CONTROL for the already existing GetSecurityInfo/AccessCheck
review; same-user peer tokens still demonstrate shared access, not
isolation. A Windows negative regression covers adoption refusal and
identity/name mismatch; source authored, NOT_EXECUTED.

This is a test-only, one-level HANDLE lineage bound. The parent itself is
opened from the disposable Windows temporary path and is NOT attested as
a trusted service-owned ancestor. A root path, token label, or held HANDLE
does not supply cross-process filesystem virtualization or Microsoft
authentication separation. No production provider, SCM IPC activation,
driver, existing Minecraft data, HKCU/WAM or Launch/Provision changes.
A future production capability requires attested root ancestry, owner SID,
effective cross-token peer denial and actual game-path enforcement.

### Host-bound full writes and file-leaf hierarchy invariants (23 October 2026)

The pure synthetic COW owner `engine/runtime-core/src/independent_engine/overlay.rs`
now requires the immutable `SyntheticBaseView` when performing a full
`SyntheticSlotOverlay::write`. Previously full writes had NO baseline input,
so they could silently create a file named like an existing visible
directory or place a file below an existing host/private file leaf, even
though the later rename path rejected this shape. Both the baseline
`insert_fixture` and full-write path now reject parent/child **File**
leaf collisions in either direction. Full writes remain newly authored
private data (without a host-derived tag); they do not read or overwrite
the real host. File prefix tombstones and private-only masks determine
which synthetic baseline descendants are actually visible.

The same non-mutating preflight is reused by `copy_up`, nonempty
`write_at`, and `truncate_file`. RegistryValue remains a deliberately
flat synthetic namespace; case-normalized, bounded ASCII keys are NOT
Win32 paths. Deliberately malformed states required by older negative
rename regressions are now injected directly by test code, never by
the public full-write operation. Three focused regressions added:
baseline insertion order, host/private full-write collisions,
and partial/copy-up refusal. The full-write signature change is
source-only and has no active runtime provider consumer.

**Proof:** exact GitHub source/static ownership check only.
Rust compilation/tests NOT_EXECUTED (no Rust toolchain here).
This does not supply Minecraft Education file redirection,
registry/WAM virtualization, a service or launch permission.

### Copy-up single-allocation and zero-byte partial writes (23 October 2026)

The original pure Rust `independent_engine/overlay.rs` now performs
an explicit copy-up by checking a visible synthetic host resource,
per-item/1,024-entry/8-MiB private quotas and process ownership
BEFORE copying ONE immutable fixture leaf directly into the private
map. The previous copy-up copied the complete byte payload once
and then passed it to `write()`, which cloned it a second time.
The new path retains the host-derived tag; repeat copy-up on an
existing private value is a no-op and cannot change provenance.

A range-validated `write_at` with an empty byte payload is now a
no-op. It never grows files beyond EOF, creates empty private keys,
copies host fallback or changes the host-derived flag. Callers can
explicitly create an empty synthetic file with `write()`.
Three focused tests cover provenance/idempotency/peer separation,
both private quotas and missing host atomicity, and empty writes.

**Boundary:** source/static review only; Rust tests and Windows
gameplay NOT_EXECUTED. This is synthetic in-memory data only, not
actual Minecraft filesystem interception, HKCU, WAM or Launch authority.

### COW private-only policy compaction and rename namespace safety (23 October 2026)

The original bounded synthetic `independent_engine/overlay.rs` now applies
the same canonical prefix-set capacity math to both tombstone masks and
`private_only_prefixes`. A parent private-only boundary can compact 1,024
redundant child rules into one without losing host-read suppression.
Requesting a rule already enforced by a parent is a no-op even when the
slot has subsequently authored private children; creating a genuinely
NEW wider policy over existing private bytes is still rejected for
ambiguous provenance BEFORE the policy set changes. Prefix comparisons
remain case-folded ASCII and resource-surface bounded.

For the fixture File surface, `rename` and `rename_tree` now reject
destinations nested below a visible file leaf, visible destination
subtrees and mixed source leaf-with-descendants that a real filesystem
could not represent. Single-leaf rename also refuses ancestor/descendant
self-overlap. RegistryValue remains a flattened synthetic value
namespace, not a real Windows registry hierarchy. Tests authored for
full-capacity rule coalescing, redundant rules, no host/private leak,
rename source/destination collisions and failure atomicity.

**Proof:** exact GitHub source/static review and source-level
regression coverage; native/Rust execution NOT_EXECUTED. This is a
synthetic namespace improvement only: it does not implement Windows
filesystem interception, HKCU/WAM routing, a Minecraft launch provider
or authorize any client 02–04 operation.

### Canonical tombstone masks and in-place private buffer writes (23 October 2026)

The existing synthetic `independent_engine/overlay.rs` now shares ONE
quota-preflight/compaction rule across leaf and prefix deletion plus
single-leaf and subtree rename. Full-capacity tombstones that already
cover a path never cause a redundant allocation; nested tombstones
can be consolidated under a prefix instead of falsely failing when
the original 1024-entry budget is full. Prefix coalescing preserves
underlying host masks, unrelated namespaces and private children
that are subsequently authored. Failed quota preflights are atomic.

`write_at` and `truncate_file` now perform per-item/private-byte
quota checks and original host-view checks BEFORE mutation, then
reuse a previously owned private Vec in place. A host fallback
allocates only its own bounded fixture data once; host-derived tags
survive partial operations until an explicit full replacement.
Focused regression source covers pointer stability, 8-MiB full quota,
entry and tombstone limits, host immutability, profile isolation
and registry/file-surface separation.

**Proof ceiling:** exact GitHub source/static review. Rust and native
Windows tests remain NOT_EXECUTED. This in-memory ASCII synthetic
model has no real Windows file or registry hooks, service provider,
Minecraft credentials/worlds or Launch/Provision permission.

### Atomic visible-subtree COW rename (23 October 2026)

The existing original `engine/runtime-core/src/independent_engine/overlay.rs`
now supports `SyntheticSlotOverlay::rename_tree` for **bounded synthetic
file-directory and registry-key prefixes**. It validates exact slot/PID/
creation FILETIME, matching resource surface, disjoint source/destination,
strict ASCII component/depth limits and an entirely vacant destination
subtree. The operation enumerates visible private+base leaves, preflights
private-item/byte and tombstone quotas, and retains host-derived provenance
when a moved value originated from the immutable synthetic baseline.
Crossing into a private-only namespace with host-derived bytes is rejected.

Owned private Vec allocations are moved rather than copied; at most the
visible, bounded host fallback leaves are copied. Nested deletion masks
at the source are coalesced into one prefix tombstone, preventing unrelated
host values from reappearing. Recoverable refusal happens before any overlay
mutation, including directory collisions, cyclic prefixes, quota exhaustion
and oversized rewritten names. Regression source covers per-slot
noninterference, private Vec identity, source masking, destination collision,
depth/capacity, private-only provenance and tombstone coalescing.

**Boundary:** this is COW *domain logic* for a static synthetic namespace,
not an OS directory/registry move. It does NOT copy real filesystem
metadata, handle Windows links, intercept Minecraft I/O, select a provider,
isolate HKCU/WAM or confer Launch/Provision permission. The platform adapter
and real Education acceptance remain separate. Rust tests are authored but
NOT_EXECUTED in this ChatGPT+GitHub execution context. CI remains disabled.

### Independent COW rename without full-state clone (22 October 2026)

The existing executable runtime-core independent_engine/overlay.rs
handles a synthetic copy-on-write file/registry-value namespace.
Previously every rename cloned the entire SyntheticSlotOverlay,
potentially copying its full 8 MiB private-data budget plus all
private keys and tombstones simply to move one small value.

Rename now checks all expected failure cases first: slot/process,
surface, private-only/provenance, source visibility, destination
collision, per-item bytes, tombstone capacity, destination entry
count, and checked future private-byte budget. It then transfers
the owned private Vec without cloning its allocation, or copies
at most ONE synthetic immutable host value. A tombstone still masks
the old logical source, and host-derived data remains tagged
through rename. Failures during preflight leave the state untouched.
An authored regression checks that the same Vec pointer is retained
at maximum private-entry count and that item/tombstone quota denial
does not partially change the namespace.

This performance and atomicity correction is limited to domain COW
logic and does NOT intercept real Minecraft filesystem/registry I/O,
authorize accounts, mount a production engine provider, or alter OS
ACLs. Rust tests, native Windows and four-client Minecraft acceptance
remain NOT_EXECUTED. The default-off independent service, blocked
MC02-MC04 Launch/Provision and retired Sandboxie route remain intact.

Next: strengthen the trusted Windows storage-root capability boundary
using the existing handle-relative file and hive fixtures, without
touching live Minecraft worlds or a user's registry.



### Early signing-policy denial and self-process image-mapping native fixture (22 October 2026)

The default-OFF, read-only native SCM IPC pilot now checks its **actual
reviewed MIVUBI desktop signer policy BEFORE requesting any pipe-client
OS process HANDLE, WTS token, WinTrust verification or native NT class-44
image binding**. The checked-in reviewed DER-leaf fingerprint allowlist
remains EMPTY, so the worker returns a non-authorizing denial immediately
rather than allowing repeated administrator-sent diagnostic frames to
trigger potentially expensive cryptographic verification. The
static signer-policy validator also rejects all-zero and duplicate
leaf fingerprints, in addition to an empty allowlist. This protects
the future path if a reviewed release policy is ever introduced;
it does NOT populate or guess a certificate.

One explicit Windows-only **native fixture test** now compares the
actual test process's own image section with a read+execute HANDLE
opened from QueryFullProcessImageNameW, using the existing
NtQueryInformationProcess(ProcessImageFileMapping=44) preflight.
A positive result demands EXACT STATUS_SUCCESS and stable
FILE_OBJECT identity; if a Windows version lacks the unsupported
native information class, the test must fail honestly rather than
promote a path match. Another source test checks the all-zero,
duplicate and missing signer-policy cases. Both tests are authored
but NOT_EXECUTED; neither test installs/runs a service, launches
Minecraft, reads accounts/worlds or modifies user files.

This revision deliberately DOES NOT change the named-pipe DACL,
enable the readonly-ipc-pilot feature, introduce desktop signer
credentials, mount per-logon endpoints or grant client authority.
The process-image section and signer check are still source-only
until an actual compatible Windows environment and legitimate signed
MIVUBI desktop release can validate them. Microsoft warns that
NtQueryInformationProcess is an internal API subject to change;
the native mapping interface remains fail-closed on unsupported OS
versions, not a cross-version production contract.

Next: validate the native self-process mapping fixture and obtain a
reviewed signed MIVUBI desktop build identity before permitting
any external per-logon IPC use. Meanwhile independent Minecraft
filesystem/HKCU/WAM enforcement remains absent and Launch/Provision
MC02-MC04 blocked.



### Native executable image SECTION versus opened file object (22 October 2026)

The independent service-owned peer_image.rs has been strengthened
against pathname/file-replacement ambiguity. The previous WinTrust
observation only proved a signature on a read-only on-disk file at
the *queried* process pathname. Now, while retaining the exact
PID/FILETIME-checked native IPC-client process HANDLE, the service
opens that file with explicit FILE_GENERIC_READ | FILE_EXECUTE,
which includes FILE_EXECUTE and SYNCHRONIZE but NO write access.
It checks the SAME file HANDLE against the kernel process executable
image section using NtQueryInformationProcess information class
44 (ProcessImageFileMapping) both BEFORE and AFTER the offline
WinVerifyTrust and file-object identity review. This class compares
the process creation image's SectionObjectPointer with the opened
file object. The service's existing OS process handle now requests
PROCESS_QUERY_INFORMATION (query-only) in addition to prior limited
query/synchronize rights. No VM_READ or process mutation is needed.

**Stability warning:** ProcessImageFileMapping(44) is an NT internal,
not an official stable user-facing Windows API contract. Microsoft
warns NtQueryInformationProcess can change in later Windows builds.
The service therefore dynamically loads NtQueryInformationProcess
from **System32 ntdll.dll** and requires EXACT NTSTATUS_SUCCESS (0).
STATUS_UNSUCCESSFUL (file/section mismatch or source deleted),
missing export, insufficient rights, unsupported class and all other
statuses FAIL CLOSED. It does NOT replace failure with a filename
comparison, generic trusted signature or hash-of-path workaround.
One owned System32 DLL loader is reused for both NTDLL and WinTrust
export lifetimes, and every borrowed provider pointer is discarded
before WinTrust state CLOSE. Tests authored for exact status gating,
explicit non-writing FILE access rights and section+signer pairing.

The existing static approved MIVUBI desktop leaf fingerprint list
remains EMPTY, so even an NT kernel section match plus generic
Authenticode success **does NOT authorize** desktop origin or any
nonadmin/privileged IPC endpoint. This is a source-only negative
gate, not a live image attestation, and a same-logon duplicated
pipe client handle can still masquerade as a different message
sender. No service installation, Minecraft game access, package
signing, user data or Sandboxie runtime changed. The opt-in pilot
remains OFF by default; Rust and Windows execution and four-client
Minecraft acceptance remain NOT_EXECUTED.

Next: on a compatible Windows target compile and verify the
native class-44 ABI/STATUS behavior against a pinned current process,
an alternate file and a replaced/renamed executable, and separately
review a real signed MIVUBI desktop publisher/install identity before
ever allowing production per-logon IPC. The code must fail closed
rather than enable fallback on unsupported Windows versions.



### Service-owned native desktop-image Authenticode preflight (22 October 2026)

The standalone, source-only Windows SCM service now includes an
original [peer_image.rs](../../engine/isolation-native/independent-service/src/peer_image.rs)
native observer only when `readonly-ipc-pilot` is explicitly enabled
(default feature list remains empty). It is NOT a Tauri bridge
or a second executable launcher. After the service's existing
OS-connected PID/creation-FILETIME and last-message token/WTS checks,
it uses the **same retained process HANDLE** to query the real Win32
process executable pathname via QueryFullProcessImageNameW, opens the
named disk file read-only and keeps its file HANDLE across
WinVerifyTrust. The file's volume/file index/size/last-write metadata
is checked before/after, and the queried process image pathname is
also rechecked. Directories/reparse objects, changed file objects,
exited processes and malformed image paths fail closed.

WinVerifyTrust uses generic Authenticode V2, WTD_UI_NONE, whole-chain
revocation and cache-only URL retrieval. Its provider state is closed
using WTD_STATEACTION_CLOSE even on rejection, and only a successful
provider state can yield the leaf DER certificate SHA-256. The
WTHelper signer APIs are loaded exclusively from System32's
wintrust.dll and the hash is calculated before the state is closed.
No Windows signing trust evidence is ever supplied by IPC request
bytes, logged or sent to desktop. Source tests assert the absent
release-signer policy, wrong/untrusted/all-zero signer refusal and
null process-HANDLE rejection.

**CRITICAL boundary:** no MIVUBI desktop signer leaf is approved or
embedded in this repo, so the static signer allowlist is EMPTY and
the optional read-only pilot refuses publisher-origin promotion even
when a file has a valid generic Authenticode signature. A file found
at QueryFullProcessImageNameW's pathname is NOT automatically the
actual PE section mapped into the running process. Signer
pinning alone cannot fix that section/file identity gap or
same-logon pipe-HANDLE duplication. This change does not introduce
trusted desktop enrollment, nonadmin endpoint access, service
installation, process launch, filesystem/HKCU/WAM enforcement,
Minecraft account permissions or any Sandboxie runtime.
Windows native compilation, signer collection, SCM and Minecraft
acceptance remain NOT_EXECUTED.

Next: validate service-native Rust/WinTrust ABI and prove a signed
desktop release signer/section-file identity against the actual
installed package before any per-logon endpoint is enabled.



### SCM session-change kill switch for opt-in diagnostic IPC (22 October 2026)

The standalone service now subscribes to **SERVICE_CONTROL_SESSIONCHANGE**
only when Cargo feature readonly-ipc-pilot is explicitly compiled
(default feature set stays empty). This uses the existing windows-service
0.8.1 ServiceControl::SessionChange handler and
ServiceControlAccept::SESSION_CHANGE; it does not register a
Tauri handler or a Windows desktop/window notification listener.
Every SCM session-change event (including logon, logoff, lock/unlock,
physical or RDP connect/disconnect) coalesces into the existing bounded,
nonblocking Stop/Shutdown channel. The service's current
HostLifecycle transitions to StopPending when the worker observes
the signal, cancels/drains pending overlapped I/O, closes its named
pipe, and only then reaches Stopped. The numeric SessionId from the
SCM notification is **deliberately not used** for auth, endpoint
selection, privileged token access, or DACL enrollment.

The IPC worker also checks the existing stop channel after parsing a
complete read-only frame and again after native peer/token inspection,
reducing the window where a diagnostic can be processed during
a concurrent session change. If another message has already been
queued, the control callback stays nonblocking and one stop signal
is sufficient. The service is *not* automatically restarted and
does not reopen a user's access; the conservative pilot simply stops.

**Limits:** the Windows SCM can miss session events occurring before
handler registration, and a synchronous Windows token/WTS query
cannot be instantly interrupted. Therefore this is a fail-closed
source-only *whole-pilot shutdown policy*, not complete, per-logon
revocation or authentication. Fresh OS WTS/token checks for every
message remain required. Default service binaries remain inert,
the pipe remains admin-only, nonadmin candidate DACLs are still
unmounted, and Minecraft Launch/Provision stays blocked. Two
Windows-only regression tests check exact session-change acceptance
and coalescing semantics; Windows compilation, SCM events and
Minecraft runtime are NOT_EXECUTED.

Next: implement and review service-owned signed desktop process
image attestation with an independently reviewed allowlist before
issuing per-logon endpoints, then validate revocation and ACL denial
on a Windows target without activating real game privileges.



### Native effective AccessCheck of candidate per-logon pipe policy — 22 October 2026

The previously string-only logon DACL candidate is now an actual,
**read-only Windows AccessCheck review** inside the existing
service_identity.rs source owner (reusing the Win32 ABI pattern from
independent_root_access.rs without coupling the SCM process to Tauri).
The candidate SDDL now includes an explicit SYSTEM Owner and Group,
`O:SYG:SY`, because AccessCheck rejects descriptors without them.
The source converts its internally validated SDDL to a self-relative
Windows security descriptor, checks that owner, group, and a non-NULL
protected DACL exist, then duplicates the already-verified WTS
user PRIMARY token into a SecurityImpersonation query-only token.
It invokes Windows AccessCheck for exactly two pipe-specific rights:
minimal FILE_WRITE_DATA|SYNCHRONIZE **must be allowed** to the intended
logon; FILE_CREATE_PIPE_INSTANCE **must be denied**. The privilege
set has a bounded, correctly aligned buffer and unexpected API output
fails closed.

When other local WTSActive sessions exist, the same native descriptor
is compared against each other Windows-returned primary user token:
both requested write rights and create-instance rights must be
denied. Token/logon mismatches or unavailable OS queries fail closed.
The active session set is enumerated again to detect changes. If
there are no simultaneous peer sessions, effective peer denial is
**not observed** and must not be claimed. Real per-logon pipe objects,
originating-client Authenticode proof, handle-duplication resistance,
revoke-on-logoff and filesystem/HKCU/WAM enforcement still do not exist.

This reviewer applies NO security descriptor to any named pipe,
installs NO service, emits NO authority/ACL strings to desktop and
creates NO Minecraft session. The opt-in IPC feature remains OFF by
default; the currently bound pilot pipe retains its administrator-only
DACL. Two native Windows fixture tests are authored for AccessCheck
on an interactive token and for refusing missing owner/group/DACL;
they are NOT_EXECUTED without a Windows toolchain and machine.
Exact-source static checks do not substitute for native proof.

Next: compile and verify real Windows AccessCheck results including
other interactive logons, then design the independently trusted
desktop-origin and endpoint lifecycle before any nonadmin bind.



### Candidate per-logon IPC ACL policy — source-only, 22 October 2026

The existing independent Windows SCM identity owner now derives a
strictly non-authorizing **per-logon named-pipe DACL candidate** from
the OS-resolved enabled NT SERVICE SID and a unique enabled
S-1-5-5-X-Y logon SID from WTSQueryUserToken. The candidate permits
SYSTEM and the exact service SID full control and permits the ONE
observed interactive logon only FILE_WRITE_DATA (0x2) and
SYNCHRONIZE (0x100000). It explicitly excludes
FILE_CREATE_PIPE_INSTANCE (0x4) and avoids GenericWrite,
BUILTIN Users, Authenticated Users and Builtin Administrators client
ACE grants. Canonical string validation rejects extra SDDL ACEs,
noncanonical SID text, missing/foreign logon subauthorities, and
different logons produce different candidate DACLs.

The current WTSActive diagnostic now also repeats WTSQueryUserToken
AFTER the second active-session inventory: the rechecked primary
user token must have the same User SID, Session ID, and enabled logon
SID as the previous primary user token. This reduces (but cannot
eliminate) reuse of a session number across logoff/logon.

**Important: this is NOT an ACL applied to a pipe.** The DACL
candidate is built from verified native observations and immediately
discarded; no nonadmin endpoint exists, and the existing admin-only
pipe and default-disabled readonly-ipc-pilot are untouched. Even
an identical last-message SID/logon does not distinguish processes
within the same logon when pipe handles can be duplicated.
Before enabling a per-logon endpoint, Windows target verification
must establish trusted service-created endpoints, client application
provenance, active session revocation, actual AccessCheck denial
for other logons, and correct service/installer/signature rollback.
Windows tests, SCM activation, Minecraft multi-client and OS-enforced
filesystem/HKCU/WAM interception remain NOT_EXECUTED.

Next: verify native Windows logon revalidation and actual DACL
behavior, then add service-owned per-logon endpoint admission only
after a trustworthy desktop origin/renewal model exists.



### One bounded Windows User SID owner for service and IPC (22 October 2026)

The native Windows service IPC had a second TokenUser reader that
called IsValidSid on the returned SID pointer without first proving
that the entire SID lay within the live GetTokenInformation allocation.
This is now corrected in the existing
[service_identity.rs](../../engine/isolation-native/independent-service/src/service_identity.rs):
a single bounded SID copier is used by both TokenGroups and TokenUser.
It checks the returned byte count, header location, SID revision,
subauthority count, complete SID body and native IsValidSid/length
**in that order**. The duplicate pointer reader and unused TokenUser
FFI declarations were removed from ipc_worker.rs. No SID is exposed
to desktop, logged or accepted from a broker message.

The service's WTSActive console/RDP peer check now also compares the
actual process PRIMARY token's User SID to the OS-returned primary
WTS user token, alongside the existing exact session and unique
enabled logon SID checks. A different user sharing a session is no
longer an acceptable diagnostic candidate. Authored Windows-only
regressions cover foreign SID pointers, truncated return lengths,
valid canonical SID copy and native TokenUser ABI layout. They have
NOT been run: there is no Rust/Windows compiler in this ChatGPT
environment. Service SID/DACL, IPC StopPending, WTS flags, default-off
pilot and Minecraft Launch/Provision block remain unchanged.

This is security *validation source*, not non-admin endpoint
enrollment, Windows service installation, Minecraft file/registry
isolation or four-client acceptance. No privileged mutation,
account/world read or Sandboxie activation occurred.

Next: build and run native Windows ABI/token tests when available,
then design a scoped per-logon pipe with an OS-derived SID and
verified publisher/desktop origin before enabling client access.



### Active console and RDP logon observation per IPC message (22 October 2026)

The opt-in SCM worker no longer blocks named-pipe creation merely
because no physical console user is attached. It retains its protected,
admin-only service SID DACL and default-OFF feature. Only AFTER the
actual pipe peer process is opened and its PID/FILETIME verified does
the service inspect that native process token against local Windows
Terminal Services sessions using WTSEnumerateSessionsW. Physical
console and RDP sessions with WTSActive qualify as candidates;
Disconnected, Connected, Listen and other states do not. Inventory
size is capped at 128 records/16 active entries, with duplicate
session IDs, session zero as Active and malformed data rejected.
WTS allocations are released using WTSFreeMemory.

The peer token's TokenSessionId must be an active OS-enumerated ID.
WTSQueryUserToken (requires existing LocalSystem/SeTcbPrivilege) then
returns the matching session's actual primary user token; its
TokenSessionId and unique enabled logon SID must match the peer.
The active set is enumerated again to catch session changes.
No user token, SID, WTS name or credentials are stored or added to
the pipe DACL. Prior last-message impersonation and User SID/session/
logon checks remain. Two regression tests cover console plus RDP
candidate selection and invalid, duplicate or disconnected sessions.

This remains pending diagnostic evidence, NOT permission for
Minecraft or the service to perform privileged operations. Same-logon
pipe-handle duplication and session races are still unproven.
Windows compilation/tests, SCM execution and game acceptance
NOT_EXECUTED; Launch/Provision MC02–MC04 stays blocked.

Next: validate native WTS/SCM behavior on Windows and design
trusted per-logon IPC DACL enrollment without self-reported IDs.



### Physical-console token observation before gated service IPC (22 October 2026)

The existing service_identity.rs now uses WTSGetActiveConsoleSessionId
to identify only the OS-attached physical console, refusing session
zero and 0xFFFFFFFF (attach/detach). After native service SID
verification, WTSQueryUserToken must succeed using the service's
existing LocalSystem/SeTcbPrivilege rights; none are enabled or
acquired through this code. The returned user primary token must
match TokenSessionId and have an enabled Windows logon SID.
The console ID is rechecked after observation. All token handles
are closed immediately; no token, SID or session identifier is
persisted or included in an IPC message or pipe DACL.

This is NOT RDP/multiuser logon enrollment or authorization.
The feature remains opt-in and OFF by default; admin-only DACL
and non-authorizing IPC remain unchanged. A detached/unavailable
console fails closed before CreateNamedPipeW. No service install,
driver or Minecraft process was changed. Rust/Windows compilation,
native WTS queries, SCM and Minecraft tests NOT_EXECUTED.

Next: validate WTS candidate against Windows SCM and design a
trusted multi-session enrollment process before nonadmin IPC access.



### Native SCM service SID and per-logon SID binding (22 October 2026)

The original, feature-gated SCM IPC worker now refuses its endpoint
unless Windows resolves `NT SERVICE\MIVUBIIndependentHost` via
LookupAccountNameW AND its actual process access token contains that
S-1-5-80 SID as exactly one enabled, non-deny-only TOKEN_GROUPS entry.
Only then does CreateNamedPipeW receive the protected ACL including
the OS-resolved service SID, SYSTEM and the existing narrowly scoped
administrators-only client write grant. A missing/unconfigured service
SID fails closed before pipe creation. This source does NOT register
or configure SERVICE_SID_TYPE_UNRESTRICTED/RESTRICTED in SCM.

The last-message impersonation token and OS-connected process token
must also have the same unique, enabled SE_GROUP_LOGON_ID SID
(S-1-5-5-X-Y), in addition to User SID, Windows session, logon
AuthenticationId and impersonation level. The SID bytes are read from
bounded native TOKEN_GROUPS and never logged, persisted, returned over
the pipe, or copied from untrusted wire values. Three native
regressions cover service/logon SID shape, disabled/deny-only groups
and C pointer alignment.

**No ordinary-user logon SID has yet been enrolled in the pipe DACL.**
It would be unsafe to grant a SID from a self-reported PID or frame.
Admin-only access remains, and same-logon duplicated pipe handles
still prevent strong message-to-process provenance. The IPC pilot
is OFF by default; no desktop integration, service installation,
credentials, Minecraft launch or filesystem/HKCU/WAM interception.
Windows compile, native tests, SCM activation and four-client gameplay
remain NOT_EXECUTED.

Next: validate Windows native SID lookup, actual token groups and
effective pipe DACL before any nonadmin enrollment or privileged
service operations.



### Last-message Windows token correlation in service-owned IPC (22 October 2026)

The opt-in SCM IPC worker now compares its OS-connected client process
token against the **last message's impersonation token** using native
OpenProcessToken, ImpersonateNamedPipeClient, OpenThreadToken and
GetTokenInformation. It checks User SID bytes, Terminal Services session,
TOKEN_STATISTICS.AuthenticationId (logon LUID), token type and
impersonation level. A pre-existing service-thread impersonation token
or failure to impersonate fails closed. The service uses RevertToSelf on
every successful impersonation through a scope guard; failure aborts the
service process instead of allowing it to continue under a caller token.
The claimed PID/creation FILETIME is still checked against OS process
data, and the pipe process is rechecked afterward. No security identity,
SID, LUID, token HANDLE or credential is returned to desktop or persisted.

This is diagnostic **candidate** matching only: an attacker able to
duplicate a pipe client HANDLE inside the same Windows logon can still
defeat message-to-process provenance. It also does not establish a
proper service SID/logon SID pipe DACL or distinguish licensed Minecraft
accounts, Windows registry views, WAM contexts or peer filesystem access.
The existing service IPC feature remains explicitly DISABLED by default,
admin-only, unmounted and free of mutating commands; matching credentials
are never used as a launch authorization. Two tests for the required
four negative evidence bits and C TOKEN_STATISTICS layout are authored;
native Windows compilation and execution NOT_EXECUTED.

Next: verify native Rust/Win32 ABI on a Windows target and implement a
production-appropriate logon-SID and service-SID endpoint policy before
accepting ordinary desktop users or exposing any privileged operations.



### SCM StopPending before service-worker cancellation (22 October 2026)

Corrected the optional read-only SCM IPC pilot so an observed
Stop/Shutdown request moves the existing `HostLifecycle`
to `StopPending` **before** the worker waits for
CancelIoEx/GetOverlappedResult completion. The worker receives a
single-use stop-report callback from the SCM owner; an SCM status
error fails the operation closed, with pending kernel I/O drained
by its existing RAII lifetime owner. Default (non-pilot) service
behavior is unchanged. Oversized pipe frames now map to a rejected
message whether ERROR_MORE_DATA is returned immediately by
ReadFile or after asynchronous completion. Two pure Rust tests
cover stop announcement and failed reporting, while the previous
wire contract regression remains.

This is SOURCE REVIEW ONLY, native Rust/Windows tests NOT_EXECUTED.
No service installed, launched or granted additional access.
It remains admin-only, read-only and opt-in, with no trusted game
profile operations or filesystem/registry enforcement.



### SCM-owned, opt-in native IPC ingress pilot (22 October 2026)

The independent standalone service now has a Windows-only
`readonly-ipc-pilot` Cargo feature, **OFF by default**. The feature
adds a service-owned [ipc_worker.rs](../../engine/isolation-native/independent-service/src/ipc_worker.rs)
and imports the ONE canonical, fixed 32-byte read-only broker parser
from runtime-core instead of duplicating the schema. The default SCM
host remains inert. An explicitly compiled pilot creates a single
protected, first-instance LOCAL inbound named pipe that permits only
SYSTEM and BUILTIN ADMINISTRATORS; administrators receive the narrowly
scoped data-write/synchronize/read-control rights, not generic-write
pipe-instance rights. This cannot yet serve non-admin user desktops.

The worker holds its pipe, event HANDLE, boxed OVERLAPPED and receive
buffer for their entire kernel I/O lifetimes. Stop/Shutdown interrupts
the connect or read wait within a bounded poll interval, calls
CancelIoEx on the exact operation and waits for final completion
BEFORE releasing the buffer, then allows the existing SCM host to
transition through StopPending to Stopped. If a buggy kernel I/O never
settles, the service does not falsely report completion or deallocate
the still-referenced memory; this is not a guarantee of bounded
shutdown on a broken driver. Invalid, oversized, stale and
unrecognized requests are discarded without replies or side effects.
The service compares GetNamedPipeClientProcessId and OS-derived
process creation FILETIME, but **never** turns these observations into
authorization. The pilot has no account, world, root, registry or
process-management operations.

**Release boundary:** no source proof makes this an authenticated
privileged broker. It lacks logon-SID/service-SID policy, per-message
client token authorization and anti-handle-duplication assurances.
Feature activation, native compilation, Windows tests, SCM
installation, driver interaction and Minecraft gameplay are
NOT_EXECUTED; this new capability is not mounted in the desktop
or packaged by the installer. The previous desktop fixture remains
separate test code, never the service runtime. The first validation
priority is a real Windows build and token/session security review,
not enabling Launch/Provision.



### Original standalone Windows SCM service host — 22 October 2026

A separate original Rust crate now lives at
`engine/isolation-native/independent-service/`. It uses the
maintained `windows-service` 0.8.1 SCM API wrapper (no Sandboxie
runtime), with the unique service name `MIVUBIIndependentHost`.
The host registers with Windows SCM, reports ordered StartPending,
Running, StopPending and Stopped states, and receives Stop/Shutdown
via a bounded nonblocking channel. A private pure Rust lifecycle
policy rejects invalid transitions; three tests are authored.

**Security and status:** this source currently contains NO named-pipe
listener, registry or file access, process launcher, driver, privileged
action, service installation or installer integration. SCM RUNNING
describes only an inert operating-system process, never Minecraft
isolation readiness. Normal desktop/CLI invocation does not become
a background daemon. The module is not part of the existing Tauri
runtime or current distribution, and does not alter the legacy
Sandboxie retirement or Minecraft Launch/Provision denial.

**Evidence:** source/static review. Rust compilation, SCM launch/
stop, Windows native tests and Minecraft acceptance NOT_EXECUTED.
No CI or installer was enabled. It must not be installed or
distributed as a working sandbox based on source evidence alone.

Next: validate the host build and design a separately authenticated
read-only service-owned IPC worker with cancellable I/O and trusted
endpoint/service-SID policy. The SCM host must not reuse the desktop
fixture DACL or elevate the existing untrusted PID wire protocol.



### Bounded overlapped named-pipe fixture lifecycle — 22 October 2026

The original one-client named-pipe fixture in
[independent_pipe_server.rs](../../apps/desktop/src-tauri/src/engine/independent_pipe_server.rs)
now uses FILE_FLAG_OVERLAPPED for both ConnectNamedPipe and ReadFile,
with separate manual-reset events and native OVERLAPPED structures.
Connect and exact-message read deadlines are 1500 ms each. A timeout
requests cancellation of only the pending operation through CancelIoEx,
then demands actual completion before releasing the OVERLAPPED/event
or read buffer. If Windows cannot complete cancellation within the
2500 ms fixture-only grace period, this disposable **test executable**
aborts rather than permitting an asynchronous use-after-free.

Two new Windows-native tests cover absent clients and a connected but
silent client. Existing normal, oversized-message and forged identity
fixture tests remain. All tests and Windows compilation are
NOT_EXECUTED. This is still not a durable service listener; abort on
unsettled IO is specifically UNSUITABLE for a production service.
It does not implement filesystem/HKCU enforcement, WAM isolation or
Minecraft startup and does not install/activate any Sandboxie component.

Next: use the independently owned process lifecycle as a foundation
for a durable, service-owned, cancel-safe IPC worker with authenticated
tokens and failure recovery; never promote fixture passing to gameplay
or isolation acceptance.



### Explicit-DACL native named-pipe server fixture (22 October 2026)

The original, unmounted Windows backend
[apps/desktop/src-tauri/src/engine/independent_pipe_server.rs](../../apps/desktop/src-tauri/src/engine/independent_pipe_server.rs)
implements a disposable, **one-client inbound-only** named-pipe server.
Unlike the unsafe Windows pipe default, it builds a protected SDDL DACL
using the actual current-user token SID plus SYSTEM/Administrators,
creates a noninheritable pipe with FILE_FLAG_FIRST_PIPE_INSTANCE,
PIPE_TYPE_MESSAGE, PIPE_READMODE_MESSAGE and
PIPE_REJECT_REMOTE_CLIENTS, and permits only a fixed 32-byte
diagnostic frame (oversized/short/malformed messages are rejected).
The endpoint uses an exact fresh fixture name and does not run
a permanent listener or install a service.

The reader correlates the OS-reported pipe client PID and creation
FILETIME with the untrusted frame via the existing native peer
inspector, then impersonates the **last message's** caller using
ImpersonateNamedPipeClient. It compares the impersonation token user
SID and Windows session to the current fixture user, then uses an
RAII guard for mandatory RevertToSelf; failure to revert aborts rather
than leaving the server thread in an unsafe identity. The result
contains sanitized pending facts ONLY. Native tests cover a local
valid frame, a too-long message and forged process creation time.

**These are Windows-only source/tests, NOT_EXECUTED.** The current
user still has full access to this *test* pipe. That is not appropriate
for a privileged service: the permanent design will require an
independent service SID, per-client minimal access including a logon
SID policy, robust overlapped/deadline/cancellation handling and
message-to-handle authentication resistant to duplicated pipe
handles. The fixture uses synchronous I/O only when a client is
already connected; it must never become a persistent listener.
There are no mutating opcodes, Tauri routes, service/driver changes,
Minecraft launch permissions or Sandboxie runtime dependencies.

Next: define the service-owned endpoint identity and connection
lifecycle with cancel-safe overlapped I/O and its Windows-only
acceptance contract. Never promote PID/token resemblance to
sandbox readiness or license/account separation.



### Versioned independent broker diagnostic protocol (22 October 2026)

`engine/runtime-core/src/independent_engine/broker_wire.rs`
now owns one canonical, independently authored **32-byte little-endian
request wire contract** (magic MIVB, schema 1, read-only probe kind,
MC02–MC04 slot, claimed PID, full creation FILETIME, correlation ID,
and four reserved-zero bytes). The decoder refuses extra/short bytes,
unknown versions/ops, MC01, zero process values/correlation and
unrecognized reserved bits. There are **NO** launch, provision,
file/registry mutation, credential, token, or readiness opcodes.
A correlation ID is not an authentication nonce or an anti-replay proof.

The Windows-only, unmounted diagnostic adapter at
`apps/desktop/src-tauri/src/engine/independent_pipe_peer.rs`
accepts an **already connected server-side named-pipe HANDLE**, checks
the real client PID using GetNamedPipeClientProcessId, opens that
process with query/synchronize-only rights, checks complete creation
FILETIME and live status, and rechecks the observed pipe PID. Results
remain PENDING, never authorize a session, even on full PID/FILETIME
match. This does **not** create or install a named-pipe listener/service.

**Security boundary:** a future privileged service must use an
explicitly restricted pipe DACL, authenticated client token/session
and carefully bounded named-pipe impersonation with unconditional
RevertToSelf; it must prove which message was actually read and
bind that to a fresh, still-live OS process and verified profile.
GetNamedPipeClientProcessId and frame contents **alone do not
authenticate a caller**, especially with duplicated pipe handles.
Windows-specific source/tests NOT_EXECUTED. No new desktop command,
service, game launch, driver install, credentials or user profiles
were changed. MC02–MC04 Launch/Provision remains blocked.

Next: implement a bounded read-only named-pipe server fixture with
explicit pipe ACL and token impersonation/revert handling, then
source-validate the service peer-binding contract before privileged
storage admission.



### Native Windows effective root-access evidence (22 October 2026)

The original Win32 read-only inspector in
[apps/desktop/src-tauri/src/engine/independent_root_access.rs](../../apps/desktop/src-tauri/src/engine/independent_root_access.rs)
uses GetSecurityInfo (OWNER, GROUP, DACL), OpenProcessToken,
DuplicateTokenEx (SecurityImpersonation) and AccessCheck to evaluate
FILE_LIST_DIRECTORY and FILE_ADD_FILE against the same opened directory
HANDLE using **two distinct, engine-created, still-live process tokens**.
Results contain only four access booleans; no SIDs, raw descriptors,
tokens, paths or credential values escape. A NULL DACL, owner/group
omission, an AccessCheck failure, stale PID/FILETIME, exited child or
same-process comparison fails closed. Real Windows test fixtures
compare a root's ACL for two sibling processes that share the same
Windows user; identical rights must never be promoted to isolation.

The bounded test root requests READ_CONTROL explicitly and remains
borrowed from the suspended process owner. The review result can only
be OwnerCannotUseRoot, PeerAccessObserved or
PeerDeniedCandidateOnly. **Even PeerDeniedCandidateOnly is not a
production readiness grant**: it cannot prove denial to *all* peers,
privileged processes, host aliases, or game paths, nor replace a
service-owned root, trusted SID/ancestor checks, kernel mediation,
registry interception or WAM compatibility. Source/test code only;
Windows compilation and native fixture execution NOT_EXECUTED.
No driver, service, registry/config/account or game Launch path changed.

Next: validate this native reviewer on Windows, then independently
design a trusted privileged root/service and actual filesystem/
registry mediation for unmodified Minecraft. Do not infer isolation
from same-user ACL evidence or Job Objects.



### OS-owned process identity bound to private fixture file I/O (22 October 2026)

The existing native Job Object launcher and handle-relative file backend
now form a **single bounded process-lifetime fixture**. The file root is
only constructible from an actual engine-created, retained, suspended
Windows process handle. Creation checks full PID/creation FILETIME and
live process state. Every root and file operation rechecks that same
handle; a mismatched MC02/03/04 slot or an already-exited child is
rejected before file access. Borrowed Rust lifetimes prevent resuming
or dropping the fixture owner while its file root is in use.
Three Windows-native file tests cover independent process owners,
peer-slot rejection, persistence, invalid names/collisions and
post-termination denial. The existing Job Object tests remain.

This is **genuine manager-owned process-bound fixture I/O**, not
transparent interception or confinement of Minecraft. No Windows
token/ACL/effective peer denial or inherited AppData/HKCU/WAM/IPC
virtualization is established. The root's ACL/ancestors and OS-level
access by other same-user processes remain unverified. The native
launcher remains test-only, no Tauri IPC or production provider is
enabled, and no user data is touched. Windows compilation and tests
are NOT_EXECUTED. Do NOT treat the Rust borrow as a security boundary
against other OS processes.

Next boundary: trusted service-owned root creation and **effective**
AccessCheck/peer-denial with fresh process tokens, then a supported
per-process filesystem/registry mediation mechanism. Do not launch
unmodified Minecraft until these can actually be enforced.



### Native Windows handle-relative private file storage (22 October 2026)

The original Win32/NT adapter at
[apps/desktop/src-tauri/src/engine/independent_private_files.rs](../../apps/desktop/src-tauri/src/engine/independent_private_files.rs)
performs actual filesystem I/O via NtCreateFile with a held parent
directory HANDLE and a single validated relative component. It uses
FILE_CREATE or FILE_OPEN instead of overwrite, FILE_OPEN_REPARSE_POINT
plus GetFileInformationByHandle checks to reject reparse objects,
and rejects hard-linked regular files. Each MC02–MC04 gets a separate
held directory handle; MC01 and invalid/device/path-traversing names
are refused. The adapter writes and flushes bounded eight-byte
nonsecret fixture markers without reading from any shared host path.
Two OS-backed tests cover per-slot noninterference, persistence,
missing data, create collisions and unsafe names.

**Status:** source implemented; Windows compile and fixture tests
NOT_EXECUTED. The root currently comes ONLY from newly created
disposable test directories. Validated file identity is not a
trusted SID/ACL/ancestor capability, and this adapter DOES NOT hook,
redirect or intercept any unmodified Minecraft file operation.
No credential, world, host profile, driver, service, sandbox,
production registry or account operations are performed. Existing
Minecraft Launch/Provision remains blocked; no new provider/IPC route
has been enabled. This adds a REAL backend file primitive while
retaining the strict distinction from an OS sandbox. Do not widen the
single-component/8-byte fixture API into arbitrary game data before
enforcement, origin and crash-safe storage semantics are proven.

Next: explicitly design and validate the original process-bound
filesystem/registry enforcement layer and its privileged service or
kernel callback boundary; supported Windows authentication behavior
must still be proven before any production launch.



### Native Windows private registry application hive (22 October 2026)

The unmounted, independently authored Rust module
[apps/desktop/src-tauri/src/engine/independent_private_hive.rs](../../apps/desktop/src-tauri/src/engine/independent_private_hive.rs)
implements real Windows application-hive I/O using RegLoadAppKeyW,
RegSetValueExW, RegQueryValueExW, RegFlushKey and RegCloseKey.
The registry namespace is reached ONLY through an owned HKEY, never
HKCU or HKU. Two disposable Windows tests create fresh per-slot
files, check separate binary canaries, close/reopen persistence, and
reject native MC01/preexisting hive files. The parent directory is
opened as a Win32 HANDLE with OPEN_REPARSE_POINT and checked against
GetFileInformationByHandle before/after operations; only fixed
fixture-02/03/04 hive names are possible.

**Proof ceiling:** Windows compile/tests NOT_EXECUTED; this is a
non-production manager-owned fixture primitive. The ephemeral directory
has not been verified against a trusted process SID/ACL/ancestor chain;
the underlying API offers no transparent interception of ordinary
Minecraft HKCU accesses. RegLoadAppKeyW application hives live in
the unenumerable registry A namespace and require the returned HKEY
for access. Therefore it is NOT an independent Minecraft registry
virtualizer, storage-security boundary, WAM fix, or Launch permission.
No Sandboxie runtime or source is linked, no account files are read,
no Tauri command is registered and MC02–MC04 Launch/Provision stays
blocked.

Next step: identify and prove a Windows-supported way for the UNMODIFIED
game process to resolve private per-client filesystem and HKCU paths,
including effective ACL/peer denial and Microsoft authentication broker
compatibility. Do not connect this manager-side hive to the game
without real interception/enforcement proof.



### Independent Win32 process lifecycle (22 October 2026)

The original Windows-only code in
[apps/desktop/src-tauri/src/engine/independent_job.rs](../../apps/desktop/src-tauri/src/engine/independent_job.rs)
creates an unnamed Job Object, sets fixture-only KILL_ON_JOB_CLOSE,
launches our own Rust test executable SUSPENDED, assigns the exact retained
child HANDLE to the job, verifies PID/creation FILETIME, then resumes it.
Failure kills only the newly created fixture. Two OS-backed tests verify
true job membership and separate job identity.

This is **not** a sandbox or a production provider. Job objects group and
control processes; they do not virtualize real Windows filesystem, HKCU,
named IPC, Microsoft authentication or Minecraft singleton state. The
kill-on-close fixture policy MUST NOT be used for persistent game clients.
No Tauri command or Minecraft launch is connected. No Sandboxie code is
linked or called. The original DisabledIndependentProvider and Launch/
Provision gates stay closed. Windows compilation and tests NOT_EXECUTED.

Next technical boundary: per-process filesystem/registry enforcement with
real Windows handles and an explicit independent service ownership design;
prove identity, compatibility and recovery before enabling game launches.



### Synthetic COW private-only profile boundary (22 October 2026)

`SyntheticSlotOverlay::make_private_only` blocks host baseline fallback
under selected file or registry-value prefixes during in-memory read,
copy-up and merged enumeration. Private values still take precedence;
another MC02–04 slot and the original base are unaffected. This is
a deterministic testable policy behavior, NOT real OS filesystem or
HKCU interception, and it is not a safety barrier for unmodified
Minecraft processes.


### In-memory copy-on-write namespace core (22 October 2026)

`independent_engine/overlay.rs` implements a real bounded, test-only
copy-on-write namespace for synthetic file leaves and registry values:
immutable base fallback, per-slot private shadows, host-masking leaf
and subtree tombstones, deterministic merged enumeration, atomic rename,
quotas, slot and PID/FILETIME guards. The base is supplied as owned bytes,
not read from Windows. Actual game filesystem/registry I/O would require
real Windows enforcement, Unicode/path/reparse/stream/ACL semantics,
trusted profile root ownership and WAM compatibility. No native adapter
or Launch/Provision authorization exists.


### Package metadata is not reviewed official package origin (22 October 2026)

`image_provenance.rs` never clears `PackageOriginNotReviewed` merely
because a diagnostic supplied `ProcessPackageMetadataObserved`.
`GetPackageFullName` proves only Windows package-identity presence.
Official origin requires a separate authenticated package identity,
release and signing policy, which has not been implemented. This
prevents future source collectors from treating generic packaged
processes as verified Minecraft images and does not alter the existing
production launch denial.


### Exact-process package-form diagnostic (22 October 2026)

The native owner now supports an unmounted, read-only `GetPackageFullName`
check on a PID/creation-FILETIME-pinned process HANDLE. It returns only
`Packaged`/`Unpackaged` in `PendingProcessPackageObservation` and
discards the full package name after a bounded Windows query. An error is
never interpreted as `Unpackaged`. This is useful for distinguishing
Windows packaging paths; neither result establishes an Education publisher,
a supported MSIX migration, private Roaming/HKCU, Microsoft authentication,
or production launch authority.


### OS-derived AppContainer LocalAppData root comparison (22 October 2026)

For a pinned process with an actual AppContainer SID, an unmounted native
observer first refuses thread impersonation and compares the target's
TokenUser to the manager's current Windows user. Only in matching-user
context does it call `GetAppContainerFolderPath`, then compare the OS
location's opened volume/file ID to the caller's already-pinned root and
perform the existing ancestor diagnostic. The response is a private
optional boolean, never an OS access capability or stored account path.
A missing/mismatched user context never becomes proof. Even an OS
LocalAppData match does NOT demonstrate game RoamingAppData/HKCU,
registry, WAM/MSAL or session singleton isolation; Launch/Provision
remains fail-closed.


### Root-relative Windows NT directory handle chain (22 October 2026)

The unmounted Windows ancestor probe now opens the local drive root once,
then opens **one component per NtOpenFile call** relative to an already-open
parent directory HANDLE. The code requests only metadata/traverse rights,
sets `FILE_OPEN_REPARSE_POINT`, rejects non-directory/reparse results, and
retains each ancestor handle without share-delete until file-ID/name
rechecks finish. This removes repeated absolute prefix reopening but is
still NOT a verified root-origin or protected filesystem capability:
the first drive root is path-derived, NT ABI must be compiled on Windows,
effective access and Microsoft/Minecraft app routing remain UNKNOWN.
No operation can use this diagnostic to grant game launch or storage writes.


### Opened-root ancestor inspection (22 October 2026)

From an existing pinned root HANDLE, Windows-only, unmounted source now
checks up to 32 cumulative local DOS path components with no-follow
attribute-only directory handles, refusing reparse points and final
file-object mismatch. This is a bounded **diagnostic**, not a
race-free access capability: the opened path, changing ancestors,
ACL effectiveness, trusted owner/token and actual Minecraft data path
routing must still be independently established. No file mutations,
user-provided path, Tauri route, or launch permission is introduced.


### Read-only process SID ↔ directory-owner/ACE correlation (22 October 2026)

The existing unmounted root inspector compares a live pinned process's
TokenUser/AppContainer SIDs with the owner and basic allow ACEs on a single
already-opened directory. Only sanitized private booleans are produced,
never credentials, SID bytes or a permission grant. No ownership admission,
effective Windows access, complete ancestor traversal, game filesystem
redirection or Microsoft WAM separation is inferred from a matching ACE.


### Process-scoped pending private-root observation (22 October 2026)

The existing Windows read-only root inspector is now composable with the
canonical pinned-process observer: MC02–MC04, exact PID/creation FILETIME
and AppContainer token category are checked before/after directory ACL
inspection. The resulting `PendingPrivateRootInspection` is a diagnostic,
**not a directory access capability** or an identity join between an
AppContainer SID and a directory owner. A future original owner must
authenticate that relationship and its complete ancestor chain before
using filesystem paths or moving any game data.


### Bounded private-root ACL denial review (22 October 2026)

The unmounted Windows directory HANDLE observer rejects known broad
access-allowed ACEs (Everyone, Authenticated Users, Anonymous, Builtin Users
and Guests). Only simple access-allow/deny ACEs with valid bounded SID layout
are parsed; unfamiliar callback/object ACEs fail closed. The scan is
bounded to 64 entries and 8192 ACL bytes, and never changes DACLs.
This is a **negative-risk filter, not effective-permission proof**:
future trusted native process/slot ownership and an OS authorization check,
secure ancestor traversal and game access observations remain separate
requirements before any storage mutation or Launch/Provision.


### Read-only private-root owner and DACL negative check (22 October 2026)

The existing Windows directory-handle inspector checks `GetSecurityInfo`
OWNER/DACL on the same opened file object and rejects missing owner, NULL
DACL, failure, or changing volume/file identity. It frees the security
descriptor with `LocalFree`. A non-NULL DACL is **not** proof of isolation:
a separate effective ACE/access check, trusted expected owner/token, secure
ancestor traversal, and actual per-process game file view must be established.
The result stays unmounted and non-authorizing.


### Bounded Windows private-root handle inspector (22 October 2026)

A new **unmounted** read-only inspector accepts an already-open directory
HANDLE and checks regular-directory type, final reparse attribute and
volume/file index. It does not open a caller-supplied path or authorize
writes. Future trusted native owners must still open a verified private
root with non-following reparse semantics, validate parent traversal and
DACL/owner, and ensure process/token/slot binding. The helper cannot
prove account privacy, registry virtualization or the game's path view.


### Manager-owned private file plan — source only, 22 October 2026

`independent_engine/storage.rs::plan_private_file_access` creates a
**non-authorizing** bounded plan for manager-owned metadata under one
MC02–MC04 Local/Roaming profile, using a fixed allowed component namespace.
It rejects cross-slot/host/official-install targets, traversal, ambiguous
Windows device names, drives and UNC paths. It never accepts arbitrary
Minecraft world filenames or returns an absolute path. A future native
adapter must still prove the opened private root's identity and access
control, reject junction/reparse traversal, and bind OS process/token
identity before any read/write. The Rust plan alone does NOT provide
filesystem redirection, Roaming Known Folder separation, registry
virtualization or Microsoft account isolation.


Build a lightweight, **independently authored** Windows engine specifically for
four concurrently playable Minecraft Education Desktop sessions, each with its
own legitimately licensed account, persistent world/data and browser-like
presentation tab. One official game installation is preferred. The game
binary and Microsoft sign-in are not modified. Selecting another tab never
closes, stops, signs out or reprovisions another running session.

This is inspired by the **capabilities** of Sandboxie, not a derived Sandboxie
distribution. Historical upstream/fork files under `engine/isolation-owned/`,
`engine/isolation-candidates/`, and `tooling/isolation-source/` remain for
provenance and are **NOT** dependencies of this engine. Do not port, translate,
copy, link or bundle Sandboxie driver/DLL/service code into this implementation.
Source ownership, third-party dependency licenses and contributor assignments
must be audited before any proprietary distribution. Using documented Windows
APIs does not transfer ownership of Microsoft's implementations.

## Implementation language and ownership policy — 21 October 2026

**Approved: Rust-first hybrid architecture, not a mandatory 100% Rust rewrite.**
The goal is an independently authored, small and auditable Minecraft-specific
engine, not a percentage target for a programming language. Language selection
must preserve the existing source owner and avoid speculative compatibility
layers.

| Boundary | Standard | Condition |
| --- | --- | --- |
| Pure domain, lifecycle/admission, isolation policy, persistence rules and recovery | **Rust** in existing `engine/runtime-core/` | No new lifecycle manager or alternate ready database |
| Windows process identity, AppContainer inspection, path/registry observation, HWND and bounded native calls | **Rust** under existing `apps/desktop/` | Keep `unsafe` behind small audited Windows API wrappers; pin PID/creation FILETIME and return sanitized facts |
| Privileged user-mode service/broker, if it becomes necessary | **Rust preferred** | Add only after a supported requirement, threat model, bounded IPC and installation/rollback design |
| Filesystem minifilter/kernel helper, if Windows user-mode is insufficient | **Minimal independent C/C++ permitted** | First document why it is necessary and sufficient; WDK compatibility, signing, threat model, rollback and target-Windows proof required |
| Synthetic Windows experiments | **Existing standalone C++ may remain** | Fixture code is test-only and cannot be packaged or represented as the production engine |
| User interface | **Existing Tauri + Svelte** | No rewrite or duplicated desktop manager |
| Retired Sandboxie fork, its binaries, driver, DLLs, services and configuration | **Reference history only** | Never port/translate its source into the proprietary engine or relabel GPL-derived code as independently owned |

Prefer existing reviewed `windows-sys` / Windows SDK bindings to adding another
Win32 wrapper crate for the same calls. Each native wrapper must specify handle
ownership, lifetime, rights, error handling, size/time bounds and non-Windows
behavior. Separate observation from mutation; no synthetic or path-only fact
can authorize Minecraft launch, account isolation or a profile operation.

Do **not** rewrite the existing C++ AppContainer fixture or add a kernel
driver just to standardize languages. Reconsider C/C++ in production only
when there is a documented Windows capability gap and a narrowly scoped
interface. A C/C++ helper is not a shortcut around auth broker, singleton or
game compatibility. Memory safety, CPU/RAM overhead and tab latency must be
measured later on the actual target; Rust alone does not guarantee lower
resource use than C++.

This is a source-level engineering decision, **not proof of Windows compatibility
or an authorization to install a driver/service**. The proprietary licensing
goal additionally requires attribution/dependency and contributor-rights
review before distribution; an original Rust implementation does not license
third-party components on its own terms.

## One owner per responsibility

| Responsibility | Canonical owner | Boundary |
| --- | --- | --- |
| Domain contract, lifecycle/admission policy | `engine/runtime-core/` | Pure rules; no Windows handles or tokens |
| Session identity and source model | `engine/runtime-core/src/independent_engine/identity.rs` | Fixed 01–04 labels and full-width PID/creation time; value objects are not runtime proof |
| Operation effect classification | `independent_engine/operations.rs` | Inspect, present, launch, close and provision are distinct; not another admission system |
| Storage and authentication surface matrix | `independent_engine/storage.rs` | Local, roaming, registry, named objects, peer/host access, restart persistence, broker; no credential values |
| Native provider and adapter interface | `independent_engine/ports.rs` | Abstract ports only; DisabledIndependentProvider rejects everything |
| Evidence tier policy | `engine/runtime-core/src/independent_isolation.rs` | Synthetic, native Windows, Minecraft runtime are separate; not an acceptance registry |
| Windows process identity and future adapter | `apps/desktop/src-tauri/src/engine/instance_identity.rs` | Pin handles, validate PID/FILETIME, query tokens; return sanitized observations |
| Native window presentation/recovery | Existing `apps/desktop/src-tauri/src/engine/workspace_surface.rs` and Svelte workspace | Reversible presentation, no implicit lifecycle changes |
| Authoritative route admission | Existing desktop `instance_catalog.rs` and canonical runtime policy | Isolated Launch/Provision remains blocked until an independent provider is genuinely accepted |
| Synthetic tooling and offline review | `tooling/acceptance/windows/`, `tooling/acceptance/evaluate-appcontainer-isolation.mjs` | No game/account access; observations never authorize launch |
| Current continuation | `planning/development.md` | Decisions/next work only, not a runtime data database |

### Bound storage evidence contract — source-only

`engine/runtime-core/src/independent_engine/evidence.rs` validates diagnostic
storage facts against one exact `SessionSlot` and one full PID/FILETIME
`ProcessIdentity`. Its typed method/source matrix separates:

- synthetic fixture canaries from native per-process observations;
- path queries from actual canary access and restart persistence;
- registry and named-object observations from file-system behavior;
- host/peer access denial from ordinary private writes;
- real Minecraft sign-in observation from Windows token/MSAL metadata.

Unrecognized combinations, duplicate surfaces, mismatched identities and
synthetic positive proofs are rejected. Broker conclusions require
`MinecraftRuntime`-class observations. These are **unattested facts supplied
by future collectors**, not independently verified evidence or launch permission.
External platform crates can no longer call `StorageObservations::observe`
directly; they must use the reviewed converter. No secret/path fields are
transported by this contract, and no runtime probes or profile writes occur.

## Read-only source integration

`independent_engine/inspection.rs` combines `ProcessObserverPort`
and `StorageObserverPort` without adding a lifecycle owner. The typed
function `inspect_existing` rejects an observed PID/creation-FILETIME
mismatch before calling the storage adapter. Negative Rust tests use
in-memory fakes to check stale identity, denied process inspection, and
missing roaming/authentication evidence. This is a source integration
contract only; the real Windows adapter is not yet selected and no
Tauri command calls it.

## Logical runtime flow (not yet wired to production)

1. **Inspect target:** identify the registered, signed official Education image without modifying or duplicating the installation.
2. **Verify process ownership:** open and retain the exact process handle, compare PID, creation FILETIME, user/AppContainer identity and Windows session; reject mismatch or exited processes. A PID/title alone is insufficient.
3. **Determine isolation capability:** inspect named-object scope, local/roaming/registry view, peer/host canary denial, persistence and broker behavior. Distinguish path strings from actual file access and game credentials.
4. **Admit one bounded operation:** reuse canonical lifecycle/admission; refresh relevant evidence immediately before mutation; reject unknown/unsupported providers, untrusted binaries and stale identities. No synthetic fixture flag can unlock launch.
5. **Execute through one native adapter:** only a future verified Windows implementation may launch or normally close **its own** process. No global kill, broad ACL grants, credential cache manipulation or guessed session adoption.
6. **Present without mutating gameplay:** the existing window workspace switches selected HWND/focus and rolls back failed presentation. Background Minecraft processes and simulations remain alive.
7. **Recover explicitly:** on manager failure restore game windows; on process failure keep other sessions untouched. Account-bearing profile roots are never deleted by diagnostic fixture cleanup.
8. **Validate later:** separate actual two-game sign-in/render/input/network proof from four-client gameplay and installer acceptance; never promote source/tests into product readiness.

## Planned platform adapters (implementation slots, not a promise of compatibility)

| Adapter concern | Preferred Windows facility or direction | Stop condition |
| --- | --- | --- |
| Process identity/launch | Win32 process and token APIs with exact creation identity | Cannot prove own process, official signed target or singleton compatibility |
| Namespace separation | Native AppContainer named-object scope, only if compatible | Actual Education singleton still conflicts |
| Local/private storage | AppContainer profile and tested filesystem permissions | Peer/host access is possible |
| Roaming Known Folder and registry | Test documented OS mechanisms; design independent virtualization only if truly required | Unmodified Education resolves sensitive paths to shared host |
| Microsoft authentication | Game-owned login; investigate supported MSAL/WAM behavior, no tokens exported | Two independent licensed sign-ins cannot persist |
| Window/focus | Existing Win32 presentation and recovery owner | Blank window, input, fullscreen/DPI or failed rollback |
| Privileged service/driver | **Not selected** until a demonstrated requirement and security review | Unsupported signing, broker semantics or unsafe access |

AppContainer is a **candidate**, not the mandated final provider. If roaming or
Microsoft broker compatibility cannot be achieved with documented APIs, record
NO-GO for that candidate rather than inserting unreviewed hooks, bypasses, or
a large speculative custom driver.

## Implemented source adapter (read-only)

The existing desktop identity owner now implements the
`ProcessObserverPort` through `WindowsProcessObserver` and
`inspect_independent_source`. It calls the previously implemented
process-handle pin and creation-FILETIME check, then returns a typed
`ProcessObservation` with both official-image signature and visible
window evidence explicitly **false**. The matched process is a Windows
process, not an accepted Minecraft account or a verified signed game.

`UnknownStorageObserver` currently returns `Unknown` for **every**
storage and broker surface, including Roaming Known Folder, registry,
process namespace, peer/host data and authentication. This is an
intentional fail-closed platform-port placeholder, not a filesystem
sandbox, second state owner or production implementation. A future
observer must replace it with verified OS-specific evidence. The
integration is not registered with Tauri; admission continues to
deny isolated launches.

## Exact-process storage binding and token metadata

The port contract now requires `StorageObserverPort::inspect_boundaries`
to receive the exact `ProcessIdentity` (PID and full-width creation
FILETIME). An eventual native storage implementation must re-pin or retain the
same handle before inspecting a process-specific storage view. Domain tests
verify the requested identity reaches the observer, and that mismatched or
denied process observations prevent storage inspection entirely.

The desktop's existing process identity owner checks the live token's
AppContainer flag and SID through its already pinned process handle. Only
`ProcessContainment::NoAppContainerToken` or
`ProcessContainment::AppContainerTokenObserved` leaves the native observer.
Raw SIDs, tokens, filesystem paths and credentials remain private; the
all-`Unknown` storage adapter is unchanged. Observing an AppContainer SID
is not proof of separate named-object namespaces, RoamingAppData,
registry, WAM/MSAL accounts or game compatibility. No launch route is enabled.

## Process-local path evidence — source-only contract

`independent_engine/paths.rs` accepts a sanitized, exact-process
`ProcessPathSnapshot` with three independent observations:
LocalAppData, `APPDATA` roaming environment, and
`FOLDERID_RoamingAppData` Known Folder. Each is `Unavailable`,
`UnderExpectedProfile`, or `OutsideExpectedProfile`. The collector
must establish that it ran within the intended process context; an
external host query for another user's Known Folder is not equivalent
to the unmodified game's actual view. No raw path is stored.

The mapping distinguishes `PathOnly` from
`PathOutsideProfile`. Both remain unresolved because neither proves
actual file/registry access or account isolation. A reported outside
path is a compatibility warning, **not a demonstrated host-data leak**.
No game, token, registry or profile file is opened. Tests cover
stale PID/FILETIME, wrong slot, all-private-looking path reports,
missing Known Folder data and invalid provenance.

## Per-process synthetic path evidence (source-only)

The existing native fixture collects paths from both cooperating synthetic
AppContainer children and emits bounded schema-2 A/B observations. The parent
checks token identity; individual paths are never exposed. The offline
evaluator verifies child/aggregate consistency but explicitly reports
`processIdentityBound=false`, since fixture labels are not PID/FILETIME.
Production binding must continue to use the canonical pinned Windows
process observer and `independent_engine/paths.rs`. This is not a game
launch, isolated Windows roaming solution, or local runtime acceptance.

## Bound cooperating-process path capture — source contract

`CooperativePathObserverPort` and `inspect_cooperative_paths` require a
previously selected `SessionSlot` and exact `ProcessIdentity`
(PID + full 64-bit creation FILETIME). The process is inspected **before and
after** collecting `ProcessPathSnapshot`; stale PID/creation identity,
child exit, denied observation, wrong slot or mismatched probe identity
invalidates the complete result. A successful inspection still contains only
`PathOnly`, `PathOutsideProfile` or `Unknown` observations, never
proof of actual account isolation or provider launch readiness.

The eventual native collector must obtain the actual path queries from a
**cooperating process** and validate its peer against the pinned OS handle.
It cannot fabricate a process key from an A/B fixture label, call Known
Folder APIs in an unrelated host process and attribute those values to the
game, inspect user token caches, or inject hooks into Minecraft. This
module provides the contract and deterministic fake-adapter tests only.
No IPC, launcher, profile access, Windows service, or production collector
is implemented by this batch. The existing `UnknownStorageObserver`
remains the active diagnostic placeholder.

## Cross-language synthetic observation decoder

The existing native fixture already writes one packed-by-layout, **8-byte**
`Observation` record per cooperating child (two little-endian Windows DWORDs:
`version=1` and `flags`), under the *fixture-owned* AppContainer profile.
Rather than introducing another daemon, IPC endpoint or binary protocol,
`engine/runtime-core/src/independent_engine/fixture_wire.rs` decodes this
existing record in pure Rust. The C++ fixture now asserts the exact record
size at compile time. The decoder accepts **exactly eight bytes**, validates
the version, known bits, mutually exclusive phases and the consistency of
optional roaming-query flags, and produces only `UnboundFixturePaths`.
Paired child records must agree on their first-run/reopen phase.

**Crucial trust boundary:** an 8-byte observation file has no authentication
or embedded PID/creation FILETIME. Its bytes alone do not prove that a given
process wrote it or that Minecraft uses the same paths. Only the fixture's
own parent currently has the original child process handles and verifies the
child's AppContainer token. The Rust decoder deliberately cannot construct
`ProcessPathSnapshot` or implement `CooperativePathObserverPort` from
an anonymous record. A future Windows adapter must independently pin the
actual child handle, constrain its source to that child-owned profile, enforce
bounded I/O and reject stale identities before using the full domain path
inspection flow. No account-bearing paths, broker tokens, installed game,
driver, service or production launch code are added.

## Bounded, unbound fixture record reader — source-only

The existing desktop-native identity owner now contains
`inspect_unbound_synthetic_fixture_record`, compiled only on Windows.
It accepts an **already-opened** `std::fs::File` handle and exact
`ProcessIdentity`, not an arbitrary path or an application account ID.
It verifies the process's PID/creation FILETIME and AppContainer token
both before and after reading, checks the file is regular and precisely
eight bytes, performs a single bounded `read_exact`, then calls the
canonical `fixture_wire.rs` decoder. Source tests reject an ordinary
non-AppContainer process and a stale creation identity.

This checks the process **and** the record, but it does **not** establish
a relationship between them. The next adapter must validate that the
specific open record handle belongs to the newly created profile of the
original self-created fixture child, and guard against concurrent
replacement/writes. Neither two valid independent checks nor a matching
AppContainer flag proves that the process wrote the supplied bytes.
Accordingly the reader returns **UnboundFixturePaths**, never a bound
`ProcessPathSnapshot`, a trusted `CooperativePathObserverPort` result,
or an authorization to launch an installed Minecraft executable.
No Tauri command, service, filesystem/registry virtualization, background
task or profile cleanup is implemented here.

## Synthetic fixture record ownership safeguards

The original C++ fixture parent now retains the creation FILETIME of each
**self-created child process handle**, and rechecks live state, creation
identity and actual AppContainer SID against its freshly created profile
both before and after each record read. The file is opened from that
profile's fixed observation name with reparse-point following disabled;
a reparse/link, directory or non-eight-byte file is refused. The same
open handle's file ID, volume and size are checked again before accepting
its data. The Rust decoder also rejects an eight-byte record if required
own/peer/host/mutex/synthetic-roaming control bits are missing.

These checks strengthen **synthetic parent-local provenance** and prevent
accepting arbitrary short records as meaningful fixture evidence. They do
not establish cryptographic authorship, proof that unmodified Minecraft
uses the same paths, or a Rust transport from the original parent. The
separate Rust helper still returns only `UnboundFixturePaths`, because
it does not own the parent-created profile and process handles. A trusted,
bounded capability handoff must be designed before creating a bound
`ProcessPathSnapshot`.

## Owned synthetic child/profile capability — 21 October 2026

The C++ synthetic fixture parent now binds its newly created
`Profile` to a noncopyable `Child` at the exact successful
`CreateProcessW` call, after capturing the full creation FILETIME.
`observe(child)` and `stop(child)` no longer accept an independent
profile parameter: both derive the profile folder and expected AppContainer
SID from the child's original borrowed owner. Before each record read
or normal stop, they verify the owned process handle, creation FILETIME,
liveness and SID; record files remain fixed-name, exclusive-read, non-reparse
eight-byte fixture artifacts. The owner cannot be changed by a caller
passing a different profile to the observation routine.

Only the fixture parent owns these leases. They live in the same C++
process, and the new source does not add runtime account profiles,
an IPC service, or a Rust adapter that inherits OS handles. The existing
Rust `inspect_unbound_synthetic_fixture_record` is **still unbound**:
a C++ owner pointer is not a cross-language authenticated capability.
Therefore these improvements do not unblock a Minecraft launcher or
prove filesystem/registry/broker isolation. Before any Rust-bound
handoff, the original owner must supply an OS-verified child and record
handle using a bounded transport with explicit lifetime/rights checks.

## Rust fixture record handle identity hardening

The existing **Windows-only** `inspect_unbound_synthetic_fixture_record`
now uses the already-opened file HANDLE with Win32
`GetFileInformationByHandle` both before and after its one fixed
eight-byte read. It rejects directory/reparse-point file attributes,
incorrect lengths, unsupported handle types and changed volume/file
index, size, attributes or last-write timestamp. The Windows API call
and unsafe conversion stay in the existing desktop process identity
owner, using one additional `windows-sys` feature, not a new service
or lifecycle layer. An authored Windows unit test checks rejection of
a non-eight-byte executable.

**Security ceiling:** Win32 reports the object reached by an existing
handle; opening a symlink that follows to a regular target can conceal
the original reparse point. The original trusted fixture owner MUST
therefore use `FILE_FLAG_OPEN_REPARSE_POINT` plus exclusive sharing and
prove that the file belongs to its self-created AppContainer profile.
Metadata stability is not a cryptographic signature and cannot prevent
same-length concurrent malicious writes. Neither a PID check nor a
file identity check alone proves a specific process wrote that file.
The reader deliberately continues to return `UnboundFixturePaths`,
never process-bound launch evidence.

## Opt-in fixture-only inherited HANDLE handoff to Rust

A new test-only binary, `engine/runtime-core/src/bin/synthetic_fixture_handoff.rs`,
is **gated by the `synthetic-fixture-handoff` Cargo feature** and never
registered as a desktop command, runtime provider, service or driver.
The independent C++ fixture parent can optionally spawn it while each
fixture child remains alive using `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`.
It duplicates precisely **two inheritable handles**: the already owned,
read-only process handle (QUERY_LIMITED_INFORMATION + SYNCHRONIZE) and
the exclusively opened, non-reparse eight-byte observation file handle.
No profile path, username, SID or credential crosses the command line.

The Rust helper checks the actual inherited process HANDLE's PID,
creation FILETIME, liveness and AppContainer token, file HANDLE attributes,
fixed size, file index/volume and write timestamp before/after a bounded
read, and the existing pure-Rust fixture frame decoder's phase/flags.
It then rechecks the process and exits **0 or nonzero**, without
serializing account/path/OS identifiers. The C++ owner waits up to five
seconds, rejects a nonzero result and rechecks its original child. The
opt-in PowerShell switch `-VerifyRustHandoff` builds the helper with
`--locked` under a freshly created workspace output; ordinary
`--run` and negative controls remain unchanged.

**Trust boundary:** This is a capability handoff *from the trusted,
self-created C++ fixture parent* to a disposable test-only Rust helper,
not remotely authenticated IPC or a general-purpose process inspector.
Both inherited numeric HANDLE values are only meaningful in that
specific child process; arbitrary CLI values do not grant handles.
The parent still owns profile provenance. The helper does not prove
game/registry/broker isolation, does not issue
`ProcessPathSnapshot` or launch permission and cannot run Minecraft.
The current in-product standalone Rust reader still returns
`UnboundFixturePaths`; no admission or workspace behavior is changed.

## Independent filesystem / registry route preflight — source contract

`independent_engine/storage.rs` now owns a pure Rust
`preflight_storage_route` for MC02–MC04. This is **not filesystem or
registry virtualization**, but a fail-closed assessment that describes
what a future Windows adapter must verify before **considering**
access. Every request supplies the slot, exact expected and observed
PID/creation FILETIME, storage domain, logical target owner and action.

Host account/profile data, unknown targets, another session's data,
missing or stale process identity and attempts to adopt MC01's native
profile are denied at the policy level. The official signed game
installation may be considered **read-only**, only through the local
file domain, and still requires native signature/handle verification;
creation, update and deletion of the official install are rejected.
Private local files, roaming files, current-user registry and named
objects each carry **separate required Windows observations**.
Roaming requires both `APPDATA` and Known Folder proofs, cross-slot
denial, host denial and restart persistence. A registry canary is not
file proof, and named-object isolation is not a registry view.

The only non-denied outcome is `NeedsNativeVerification`: it is NOT an
OS access grant, a profile mapping, provider readiness, authentication
proof or launch permission. An arbitrary caller may claim a private
owner; the native adapter must independently establish actual
Windows token/root/registry ownership and reject reparse, junction,
namespace alias and unsupported API routing. Only the existing
admission owner can authorize future lifecycle operations, and it
continues to reject isolated launch/provision. No new state store,
driver, filesystem I/O, HKCU writing or account/world access is added.

## Read-only route inspection and legacy registry token diagnostics

The existing Rust `independent_engine/inspection.rs` now connects
`preflight_storage_route` to the canonical process and storage observer
ports through `inspect_storage_route`. Invalid logical targets (MC01,
another slot, host account data, unknown owner, unsupported official
installation operations or stale caller identity) are denied **before
either OS observer is invoked**. Candidate routes check exact PID +
64-bit creation FILETIME, request domain-specific storage findings, and
recheck process identity **and token category** afterwards. Process
exit/token-category change rejects the complete observation.
The result type is only `Denied` or `PendingNativeAcceptance`,
never an access grant, provision right, or launch authorization. Unknown
Roaming AppData, HKCU registry and namespace observations remain explicit.
Self-reported all-positive observations still cannot unlock routing.

In the existing Windows identity owner, read-only
`inspect_legacy_token_virtualization_source` queries
`TokenVirtualizationAllowed` and `TokenVirtualizationEnabled` on the
token of the **pinned exact Windows process** and returns two sanitized
booleans (or a failure). It does not open a registry hive, query
`HKCU` from the host manager, touch account/profile files or claim
a private registry view. These are **legacy Windows UAC file/registry
virtualization flags**, not equivalent to AppContainer namespace
separation or per-Minecraft account isolation. The native
`inspect_independent_storage_route_source` currently composes the
actual process observer with the deliberately all-`Unknown` storage
adapter and is **not mounted** as a Tauri command. Rust fake-port
regressions cover denied-before-I/O, stale process after storage,
token changes, roaming gaps, host leaks and false positive reports;
Windows/Rust execution is not yet available in this GitHub-only stage.

## Synthetic registry boundary contract (no host HKCU access)

The canonical Rust engine now provides
`independent_engine/registry.rs::review_synthetic_registry_probe`.
It accepts sanitized, self-reported results only for a
new disposable fixture-owned private hive: one exact process identity,
its logical owning slot, peer-access denial and a separately verified
restart process. Host HKCU, unknown/preexisting hives, cross-slot owners,
stale PID/FILETIME and false restart continuity are rejected.
Unexpected peer reads and missing canaries are different issues.
Even an empty issues list is **synthetic self-reporting**, not
proof of Windows registry virtualization or Minecraft isolation.

No registry I/O is implemented here. Creating a canary key in an
unverified AppContainer HKCU view could change host account-bearing
data, so a future test must first prove a disposable isolated hive by
a supported mechanism. If that cannot be established, keep the
registry boundary Unknown and mark the candidate NO-GO.
Legacy UAC token virtualization flags are not replacement evidence.

The route inspector also distinguishes `path_risks`
(OutsideExpectedProfile path strings) from `dangerous`
(reported SharedOrLeaking access). Both remain unresolved; only
the latter is a reported data-access failure. Neither grants access.

### Windows application-hive feasibility candidate

The documented Win32 `RegLoadAppKeyW` facility can load an application
hive from a dedicated file and returns an `HKEY` accessible by handle,
not an ordinary HKCU/HKU key path. A missing hive file can be created as
an empty hive. Handles closed with `RegCloseKey` cause automatic unload
when no handles remain. Microsoft documents `REG_PROCESS_APPKEY`
as a mode that prevents a second caller from simultaneously loading
the same application hive; this mode must not be misread as equivalent
to per-account registry virtualization.

Source-first native fixture design, **not yet implemented or executed**:

1. Parent creates a fresh owned fixture profile and uniquely named
   *new file* for each synthetic application hive; never use HKCU,
   HKU, a host user hive, a preexisting hive file or arbitrary path.
2. Only the returned application-hive `HKEY` may be used to seed and
   query synthetic marker values. Close the handle before testing a
   second process or restart, and track file rights and ownership.
3. A cooperating synthetic child opens its own hive file through
   `RegLoadAppKeyW` and reports sanitized own-marker, peer denial
   and persistence results; a failed/unsupported access is unknown
   or NO-GO, never a reason to loosen permissions on host profiles.
4. Reject creation failures, accidentally shared markers, files
   outside the disposable fixture root and any request to redirect
   real user HKCU. Never copy Microsoft credentials or game hives.
5. Only after native Windows proof, evaluate separately whether the
   unmodified Minecraft process actually accesses a private registry
   view. `RegLoadAppKeyW` alone does **not** replace or transparently
   redirect the application's normal HKCU accesses.

Sources: Microsoft Learn, *RegLoadAppKeyW* and *Filtering Registry
Operations on Application Hives*. This proposal does not select a
production provider or authorize installation.

### Implemented opt-in application-hive phase (source only)

The Windows synthetic fixture now has a separate
`--apphive-fixture` execution route, enabled only with PowerShell
`-VerifyRegistryAppHive`. In a newly created fixture-only directory,
it asks `RegLoadAppKeyW` to create two previously absent application
hive files. All marker writes and reads use only the two returned
application-hive `HKEY` handles—not `HKEY_CURRENT_USER` or
`HKEY_USERS`. Distinct fixed DWORD markers must remain separate
while both hives are loaded and must persist after `RegFlushKey`,
`RegCloseKey`, and reopening both hive files. A strict PowerShell
receipt checker rejects unexpected fields or any claim of gameplay,
host HKCU redirection or launch readiness.

**Important:** The current phase runs within one parent process.
Different root handles and persisted markers are not proof of
cross-process access denial. `REG_PROCESS_APPKEY` has documented
load restrictions, but they are not asserted as measured in this
phase. `peerProcessDenial`, `hostHKCUIsolation` and
`minecraftAcceptance` remain `NOT_EXECUTED`. The existing pure
Rust `registry.rs` reviewer remains independent; its
`RegistryCanaryRead::AccessDenied` must NOT be populated by
this fixture until an actually separate cooperating process probes
a fresh, isolated hive and its original child identity is verified.
The new files are retained only inside the test workspace, not
installed, reused or used as Minecraft profile data.

### Independent synthetic peer-process load-lock trial — source-only

The optional `--apphive-peer-fixture` extends only the
disposable `RegLoadAppKeyW` fixture. After verifying two fresh
hives and marker persistence, the original parent keeps hive B
loaded under `REG_PROCESS_APPKEY` and starts a short-lived child
that loads/reads **unlocked** hive A but must fail to load hive B.
It then reverses the roles. Both probes require an observable
own-marker read and a recognized Win32 access/sharing/lock/busy
refusal for the opposite hive; any successful peer load or unrelated
error fails the stage. The children take only the literal A/B
selector plus a fixture-owned inherited working directory, not
arbitrary registry paths or account credentials. Child hive loads
request `KEY_QUERY_VALUE` only. They never write
hives; only the original parent creates/writes disposable markers.

A schema-2 receipt may report
`bidirectionalProcessLoadLocks=OBSERVED`, but continues to report
`peerProcessDenial=NOT_EXECUTED` and
`hostHKCUIsolation=NOT_EXECUTED`. The former observation measures
exclusive **application-hive loading while another process holds
`REG_PROCESS_APPKEY`**, not protected HKCU, registry ACL data
isolation, process-specific application namespace, Minecraft sign-in
or four playable clients. On real Windows the absence of a recognized
lock outcome remains a failure/unknown, not a reason to widen access.
A later AppContainer-specific peer and true per-process registry-view
acceptance are separate milestones.

## Per-slot isolation proof ceiling and registry/broker boundary — 22 October 2026

The canonical non-authorizing proof owner at
`engine/runtime-core/src/independent_isolation.rs` now explicitly
tracks **PrivateRoamingKnownFolder** and **PrivateRegistryView** at the
Windows-mechanism evidence context, plus
**DistinctBrokerAccountContexts** at the actual Minecraft-runtime
context. A successful `RegLoadAppKeyW` synthetic hive lock or a
`TokenVirtualizationEnabled` flag cannot satisfy a production
registry view check. A private-looking path/environment is not a
verified `FOLDERID_RoamingAppData` process view. Distinct game account
sign-in alone is not proof that selecting/restarting another session
keeps broker account context separate.

Proof contexts are now **non-substitutable**: a higher-named tier
cannot stand in for evidence collected via another mechanism
(`MinecraftRuntime` cannot assert a Windows registry observation).
The in-memory `ScopedIsolationFindings` only binds to MC02–MC04
plus an exact PID/creation FILETIME. Wrong process, PID reuse, or
an attempted MC01 adoption is rejected. Each session's gaps are
reported separately. Even a fully positive MC02 fixture cannot
clear MC03/MC04. On process restart, the caller must use a new
ephemeral ledger; no stale proof carries into a new process.

The proof matrix intentionally remains **diagnostic, not readiness
state**: it contains no OS handles, registry reads, Microsoft account
tokens, credential caches, native provider registration, or route
authorization. Only actual future, independently collected Windows
evidence and real Minecraft acceptance can justify capabilities.
MC01's official/native execution is owned elsewhere; even zero
isolated-slot gaps does not establish four-client concurrency.
The desktop's canonical Launch/Provision gate stays disabled.

Relevant Windows SDK distinctions:
- Microsoft `RegLoadAppKeyW` loads an application hive accessible
  through an explicit returned HKEY. It does not rewrite ordinary
  `HKCU` access by an unmodified app.
- Microsoft registry virtualization applies only to a limited class
  of **32-bit** processes and certain `HKLM\\Software` writes,
  not a general 64-bit/per-user HKCU sandbox.
- Microsoft WAM/MSAL uses an OS broker and can facilitate access to
  accounts known to Windows; private directories alone do not prove
  broker session separation.

## Process exclusivity and read-only pair comparison — 22 October 2026

The in-memory, non-authorizing `ScopedIsolationFindings` now enforces a
unique Windows PID across MC02, MC03 and MC04, in addition to binding each
slot to the full PID/creation FILETIME. The same PID with a different
creation FILETIME represents process reuse, not two concurrent sessions;
such cross-slot aliases fail closed. Within one process-bound review,
a `Failed` finding cannot be overwritten with `Unknown` or `Passed`.
Similarly, a `Passed(SyntheticFixture)` cannot be relabelled
`Passed(WindowsMechanism)` by writing to the same ledger. Explicit
new verification requires a newly created review, never carry-over
from another process or proof tier. These are defensive checks on
untrusted diagnostic values—not a production readiness manager.

The existing Windows-native `instance_identity.rs` owner additionally
exposes **unmounted** `inspect_independent_process_pair_source`.
It refuses equal slots, equal PIDs even with different FILETIMEs,
zero identities and stale process observations before returning an
anonymous `NativeProcessPairInspection`. It reuses the existing
pinned-handle Windows user/profile/session comparisons and AppContainer
SID-category comparison, then rechecks both processes' PID/FILETIME
and containment category. Unlike account telemetry, no SIDs, paths,
tokens, broker cache values or player identifiers are returned.

A matching profile, distinct AppContainer SIDs, or differing Windows
users is **not** proof of private HKCU routing, named-object namespaces,
Microsoft WAM/MSAL account isolation, singleton compatibility, or two
usable Minecraft windows. Existing Win32 comparisons run in separate
calls rather than one atomic transaction, so the output remains
diagnostic and may be invalidated by subsequent process changes. This
new source does not register a Tauri endpoint or alter the canonical
blocked Launch/Provision route. Windows execution remains deferred.

## Singleton and Microsoft broker pair diagnostics — 22 October 2026

`independent_engine/inspection.rs::inspect_pair_boundaries` now uses
the existing pinned `ProcessObserverPort` and a minimal
`PairBoundaryObserverPort` for two different slot/PID/creation-FILETIME
keys. Both process identities and token categories are rechecked
before/after a cooperative observation. Invalid slots/PIDs and
wrong collector bindings fail closed without producing an acceptance
record.

Two separate findings are reviewed: **game singleton collisions**
and **Microsoft WAM/MSAL account-context conflicts**. Synthetic canaries
and Windows metadata may alert about an observed singleton conflict;
they cannot claim no conflict for the actual Minecraft game.
Broker status, either conflict or no-conflict, requires genuine
Minecraft-runtime observations. Even a positive game observation
reports `PendingPairInspection`: there is no ready/approved
state, no broker attestation and no launch permission. The source
of a fact is a claim until a future trusted collector verifies it.

The existing desktop `instance_identity.rs` now provides an
**unmounted** default `UnknownPairBoundaryObserver`, which intentionally
reports both findings as `Unknown`, while reusing the exact Windows
process observation port. No WAM token/API calls, user/profile reads,
global mutex enumeration, Minecraft injection, driver, service or
Tauri route were added. Eight pure Rust and three desktop source
tests cover false positives, wrong identity, process exit and
non-Windows behavior; native execution is not yet available.

## Stateless, privacy-minimal actual-game acceptance contract — 22 October 2026

The existing domain owner `engine/runtime-core/src/independent_engine/`
now includes `acceptance.rs::review_game_acceptance` as a pure,
**non-authorizing** checklist validator. For two-client trials, exactly
MC01/MC02 are expected; for four-client trials, MC01–MC04 are expected.
Partial reports retain explicit missing slots. Duplicate slots, the
same PID with different FILETIMEs, zero process identities and an
unexpected client are rejected, never interpreted as four players.

Each client's future manual observations separately cover signed
official **publisher** verification, responsive rendering/input,
licensed sign-in, tab switch survival, reconnect account continuity,
world persistence, multiplayer and Minecraft singleton conflicts.
A different set of **joint** observations covers simultaneous
playable windows, distinct licensed accounts, continuity of the
other clients during tab switch/reconnect and cross-client multiplayer.
Each such observation is additionally bound to the **exact canonical
set of MC01–MC04 process identities for that trial**; stale FILETIMEs,
missing peer processes and prior two-client captures are rejected.
Thus four individual PASS reports cannot stand in for four
**concurrently** playable clients. A publisher-signature observation
requires independent Windows process evidence; interactive/account
and all joint outcomes require real Minecraft-runtime observations.
Token/SID comparison, WAM metadata, synthetic fixtures and
private-looking folder paths never substitute.

No user names, account IDs, passwords, tokens, license secrets,
registry contents, world files, game executable paths or screenshots
are stored by this contract. Inputs remain claims until observed
by authorized acceptance participants: **even all reported passes
return `PendingGameAcceptance`, never readiness/launch permission**.
The evaluator has no Windows system calls, scheduler, persistence,
Tauri endpoint or provider ownership. Existing Launch/Provision
remains blocked, and all runtime acceptance is still NOT_EXECUTED.

## Read-only on-disk Authenticode image candidate — 22 October 2026

The existing desktop Windows process identity owner implements
`inspect_process_image_trust_source(expected)`. The source
pins an exact process by PID + full-width creation FILETIME,
retrieves its executable name with `QueryFullProcessImageNameW`,
opens that image **read-only** with restricted sharing, and uses
`WinVerifyTrust(WINTRUST_ACTION_GENERIC_VERIFY_V2)` on the opened
file handle. The call is silent (`WTD_UI_NONE`) and uses
`WTD_CACHE_ONLY_URL_RETRIEVAL` to forbid revocation fetches.
It closes provider state on all verification outcomes and
rechecks file identity, image pathname and process liveness afterwards.
The result never includes the raw executable path or certificate.

**Critical security ceiling:** A trusted generic Authenticode signer
is **not proof of Microsoft's/Mojang's authorized publisher for the
official Minecraft Education application**. Moreover, querying the
on-disk executable path does not prove that the already mapped PE
section in a running process references identical immutable bytes,
nor does it validate a Store package identity or catalog-signature
path. These distinctions are explicit in
`PinnedImageTrustCandidate`: the trust assessment may be
`AuthenticodeTrustedOnDisk` while
`OfficialGamePublisher` is **always `NotAttested`**. An absent,
failed or uncached revocation response is reported as
`NotVerified`, not success.

The existing `ProcessObservation::official_image_signature_observed`
remains `false`. No publisher allowlist, certificate subject
guess, arbitrary file-picker, sign-in, game binary modification,
Tauri command, launcher/provider activation, background thread,
HTTP request, driver or new lifecycle owner is introduced.
A future signed-image assessor must bind the signer certificate
chain and package/code provenance to an explicitly reviewed
publisher policy and the real process image before returning a
publisher-specific fact. Any such fact remains diagnostic until
actual Windows and Minecraft acceptance are separately executed.

## Publisher signer and loaded-image evidence gate — 22 October 2026

The independent Rust domain adds
`independent_engine/image_provenance.rs::review_image_provenance`.
This is a **pure, non-authorizing reviewer**. It requires an exact
process PID/creation FILETIME, on-disk Authenticode trust, a
Windows-reported SHA-256 fingerprint of the **leaf signer
certificate**, a reviewed policy derived from an independently
obtained official Minecraft Education release, a proven mapping
from the running process's PE section to the verified file object,
and independently reviewed package provenance.

No official certificate fingerprint or publisher name is guessed
or hardcoded. The default publisher policy is empty, and
`inspect_process_image_trust_source` currently does not extract
signer certificate identity or prove loaded section identity.
Therefore **all real source observations retain explicit gaps**.
On-disk signature trust, an unsigned basename/filename, a valid
Microsoft account session, or a generic Microsoft-signed system
binary is not proof that the running executable is official
Minecraft Education. Even a fabricated all-positive report returns
`PendingImageProvenance`, never a verified publisher/launch flag.

The next Windows adapter milestone is to extract the actual signer
certificate from a successfully verified WinTrust provider state
and inspect only its SHA-256 DER fingerprint while the trust state
is alive. That extraction must fail closed for missing signer,
multi-signature ambiguity, unsupported catalog/Store packages and
revocation uncertainty, and be tied to the exact file handle.
Separately, a supported loaded-image/file-object identity technique
is required before attributing the on-disk signature to a running
process. No signer parsing is attempted through file extensions,
certificate subject display strings, WAM, account caches or UI paths.

### Windows adapter to non-authorizing publisher review

The desktop's existing process identity owner now has the
**unmounted** `inspect_process_image_provenance_source` adapter.
It feeds `inspect_process_image_trust_source` into the canonical
Rust reviewer and deliberately leaves signer-certificate SHA-256
unobserved, the executable-to-mapped-section identity only
`QueriedExecutablePathOnly`, package provenance Unknown, and the
official signer policy EMPTY. Therefore a trusted generic Windows
signature still yields explicit publisher/image-binding gaps and
does not change any game admission operation.

The Windows trust wrapper additionally checks that a **successful**
`WTD_STATEACTION_VERIFY` created a provider state and that the
corresponding `WTD_STATEACTION_CLOSE` completed successfully.
Missing or unreleased state invalidates any otherwise positive
signature observation. The signed-file lookup remains read-only,
cache-only for revocation, bound to a pinned process identity and
rechecked for on-disk file metadata changes. No certificate identity,
publisher name, package policy, WAM account or credential is guessed.

## Source-first implementation sequence

1. **Domain baseline (authored):** typed identity, operation classification,
   storage-surface observations and disabled provider interfaces in runtime-core.
2. **Platform observation (next):** narrowly scoped independent Windows-native
   inspection in the existing desktop owner. Pin process creation identity and
   sanitize results; no new desktop manager or app-wide polling service.
3. **Read-only evidence mapping:** inject test doubles into the interface to
   exercise unknown, shared, revoked, stale and contradictory observations.
4. **Private storage design:** transactional, explicitly owned profile paths;
   no auto-delete of any profile that could hold game/account state. Host/peer
   denial and Known Folder routing remain separately checked.
5. **Launch/stop design:** after a supported approach is identified, specify
   verified official image launch, profile ownership, bounded graceful stop,
   cleanup/rollback and timeouts. Keep the actual implementation unregistered
   until supported by Windows and Minecraft observations.
6. **Workspace integration:** reuse existing window and process ownership
   logic; no relaunch per tab, fake running-state cache or second GUI.
7. **Source-only review:** module ownership, Rust/TypeScript contracts,
   deterministic negative tests and static checks without enabling CI.
8. **Later local acceptance (deferred by operator):** run synthetic Windows
   fixtures, then two signed-in Minecraft clients, then four independent
   playable clients, persistent worlds, multiplayer, window recovery,
   measured CPU/RAM/GPU and clean-host packaging.

**Current evidence ceiling: REMOTE_GITHUB source architecture only.** The new
domain modules are not an isolation implementation or runtime provider. Rust
tests are authored but not yet compiled/executed; no Windows or Minecraft
readiness is claimed. Branch `Local` only; `main` and GitHub Actions remain
untouched.
