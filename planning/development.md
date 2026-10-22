# Current Development Handoff — M-Minecraft-Emulator


## Independent COW core — canonical tombstone compaction and private writes, 23 October 2026

Corrected a tangible full-quota COW bug in the existing source owner:
`delete`/`delete_tree` previously rejected a deletion at full
tombstone capacity even when the target was masked by a parent or
multiple child tombstones could be replaced with ONE prefix marker.
The same unnecessary capacity refusal also affected single-leaf
rename. Shared private `preflight_prefix_mask` and
`install_prefix_mask` now unify leaf/tree deletion and rename,
coalescing redundant masks while preserving host suppression and
unrelated file/registry surfaces. Quota failures occur before any
private-map or tombstone mutation.

Removed avoidable full-buffer clone for already-private COW
`write_at` and `truncate_file`. Both now calculate future
per-item/8-MiB private-byte and entry quotas before mutation, then
mutate the owned Vec in place; only actual host fallback is copied.
Host-derived provenance remains tagged until an explicitly authored
full `write`. Added five focused Rust unit regressions for
full-capacity compaction, ancestor masking, private Vec pointer
identity, host immutability, error atomicity, and independent
registry/file mask handling.

**Proof:** pinned GitHub source/static review. Rust compilation/tests,
native Windows isolation, actual Minecraft worlds and four-client
acceptance NOT_EXECUTED. No new provider, IPC, service, driver, CI,
game client launch or account/world/profile mutation. Existing
MC02–MC04 Launch/Provision remains BLOCKED.

**Next:** finish a source-only audit of COW path/name ambiguities,
atomic rename edges and host/private boundary semantics before
selecting a Windows enforcement adapter.


## Independent COW core — atomic synthetic subtree rename, 23 October 2026

Implemented one complete missing domain operation in the canonical Rust
`engine/runtime-core/src/independent_engine/overlay.rs`: `rename_tree`
moves visible file/registry subtrees within one process-bound MC02–04
overlay. Source and destination cannot overlap, the destination must
have NO visible subtree, rewritten names must remain valid, and all
private item/byte/tombstone capacity and host-derived private-only
constraints are evaluated BEFORE mutation. Existing private Vec
allocations are transferred rather than cloned; only visible bounded
immutable host fixture leaves are copied. One tombstone masks the old
prefix, coalescing old nested tombstones so hidden host leaves cannot
reappear. Added five focused native-independent Rust regression tests
for private allocations, host/base immutability, collision/depth
atomicity, quotas, tombstone behavior and provenance.

**Proof:** pinned exact-source GitHub review; Rust compilation/tests
NOT_EXECUTED. This remains a bounded synthetic COW domain, NOT a
Windows filesystem/registry virtualizer, Minecraft launcher, service,
driver or independent provider. Existing admission continues denying
MC02–MC04 Launch/Provision. No real profile, credential, world,
Windows installation or user data is accessed.

**Next:** complete a bounded same-owner audit of remaining synthetic
COW operations for namespace/atomicity and quota consistency; then
return to independent native enforcement once a supported OS strategy
is established.


## P0 evidence-consumer audit — separate legacy snapshot from engine readiness, 23 October 2026

Audited the canonical Rust independent-engine evidence reviewers
(`independent_engine/evidence.rs`, `inspection.rs`, `paths.rs`,
`image_provenance.rs`), the scoped proof summary, the new package
family comparison and both desktop admission tables. No observed
path/package metadata can directly grant MC02–MC04 Launch/Provision:
the storage result remains `PendingNativeAcceptance`, the publisher
review remains `PendingImageProvenance`, the independent provider is
disabled and `instance_catalog.rs` still rejects new sessions.

**Concrete consumer bug corrected:** the ACTUALLY MOUNTED
`UnifiedManagerApp.svelte` used `checked = !!snapshot` (a successful
read-only legacy owned-session observation) to render
`Engine: Connected · verified for this session` and a green
`ready` label. This falsely suggested independent-engine readiness
despite all provider gates remaining blocked. The UI now explicitly
labels the observation as a legacy diagnostic snapshot, shows
`Independent engine: Not implemented` regardless of snapshot presence,
removes the readiness-style binding and clarifies the legacy data
directory label. Empty snapshots no longer advertise an engine
connection. One existing static UI regression was updated and one
focused false-promotion regression added in the same test owner.

**Proof:** exact GitHub pinned source/consumer/static review of
the affected mounted UI and bridge; authored frontend Vitest tests
NOT_EXECUTED. No Svelte/TypeScript build, Rust/Windows/game execution
or four-account runtime acceptance. No Tauri IPC, driver, service,
installer, Microsoft login, Minecraft data, Sandboxie revival, CI,
`main` or Launch/Provision permission changed.

**Next:** review Windows-native isolation feasibility based on
documented, executable product proof criteria; do not add more
diagnostic status flags or a new provider without an enforceable
per-client filesystem/HKCU/broker mechanism.


## P0 independent engine — edition-specific isolation route decision, 23 October 2026

Reviewed official Minecraft Education Windows data paths alongside
Microsoft AppContainer, MSIX, Win32 app isolation (preview), sparse
package identity, ProjFS, RegLoadAppKeyW/RegOverridePredefKey, legacy
UAC registry virtualization, RunAs/WAM and native kernel registry
callbacks. The official Desktop edition's world location is beneath
Roaming APPDATA, exactly the class whose synthetic unpackaged
AppContainer Known Folder/environment was observed outside the
private profile. Store world data is beneath the published per-user
package LocalState, which does NOT imply a separate instance per PID.
A second package or Win32 app-isolation identity cannot be attached
to the unchanged official distribution as an assumed shortcut. Kernel
minifilters can scope I/O but do NOT solve login broker, game singleton
or presentation compatibility merely by existing.

**Decision:** no new independent provider or driver is authorized/
selected. This is NOT a universal impossibility result, nor a
new runtime gate. Existing `instance_catalog.rs` still blocks
MC02–MC04 Launch/Provision; retired Sandboxie and legacy VMware
routes remain inactive. Keep original data untouched.

**Proof:** exact GitHub repo source/static consistency plus official
Microsoft/Minecraft public contracts. No native Windows or game
executions on this SHA; actual Education I/O, Microsoft account
broker, two-client gameplay and four-client acceptance NOT_EXECUTED.
No CI, installers, service/driver, package registration, profile,
account or world writes; `main` unchanged. Feasibility criteria
and sources recorded in `docs/system/independent-isolation.md`.

**Next:** source-review existing process/storage evidence consumers
for a concrete path/package-to-isolation promotion error; change
only the canonical owner if one exists.


## P0 independent feasibility — compare live Windows package families, 23 October 2026

Added a **read-only, unmounted** Windows Package Family comparison to the
EXISTING desktop process identity owner. Two distinct OS processes must be
pinned by full PID/creation FILETIME. Native GetPackageFamilyName is called
with an explicit bounded UTF-16 buffer and the same strict package status
preflight as the existing GetPackageFullName observer. Both HANDLEs remain
live through the comparison. An absent package is distinct from a failed
query; raw package strings never leave the native function. Only four
sanitized relation categories are returned. Added negative unit tests
for PID collisions/invalid keys and a Windows self-process consistency
fixture (all tests authored, NOT_EXECUTED).

**Decision:** this narrows the MSIX feasibility question without reviving
the retired AppContainer/fork routes. MSIX package identity and private
storage rules are package/user scoped, NOT per Minecraft PID. Matching
package families never proves Windows-user identity, independent
filesystem/HKCU/WAM access, verified official publisher or a separate
licensed game client; even a different family is not an accepted
replacement provider. No game launch, service, driver, account/profile
write, Tauri IPC registration, CI, release or main branch change.

**Proof:** exact pinned GitHub source/consumer/static review only.
Rust/Windows execution, actual Minecraft packaging observation, game
sign-in and four-client acceptance NOT_EXECUTED.
Launch/Provision MC02–MC04 remains BLOCKED.

**Next:** examine supported process-bound Roaming/HKCU/broker enforcement
approaches and their documented limitations before selecting a provider.


## Independent engine — parent-HANDLE-bound disposable roots, 22 October 2026

Hardened the EXISTING, test-only Windows NT file backend rather than
introducing a new provider. Fixture roots are now created via
NtCreateFile(FILE_CREATE) as a single checked child of a retained parent
directory HANDLE. An existing root can no longer be adopted by presenting
an arbitrary directory HANDLE after a path-based CreateDirectory. The
factory retains parent/root volume+file-index identity, reopens the same
relative child and checks the full process PID/FILETIME before and after
fixture operations. Root directory HANDLEs now explicitly request
READ_CONTROL, which the existing GetSecurityInfo/AccessCheck fixture needs.
The cross-process shared-root regression still demonstrates that two
same-user Windows tokens do NOT provide separate permissions.

One additional Windows-only negative fixture regression was authored for
pre-existing-root collision, tampered parent identity, changed child-name
lineage and successful bounded marker I/O after restoring the expected
capability. The fixture temp parent is NOT an attested service-owned
security anchor; its ancestor chain, inherited ACL, peer denial, WAM,
Minecraft file routing and registry interception are not proved.
This is source hardening, NOT transparent Windows I/O isolation.

**Proof:** exact GitHub source/static review at this revision. Native
Windows Rust compilation and fixture tests NOT_EXECUTED; Minecraft and
four-client runtime acceptance NOT_EXECUTED. No service activation,
driver, Sandboxie provider, credential/world/profile mutation, CI,
packaging or MC02–MC04 Launch/Provision change.

**Next:** validate this fixture on Windows and establish a genuinely
service-owned, token/ACL-attested parent root before considering any
production private-file capability.


## Independent engine — avoid full-overlay copy in COW rename, 22 October 2026

Optimized the canonical runtime-core independent_engine/overlay.rs
rather than growing fail-closed IPC signing checks. Previously one
rename cloned all private COW bytes (up to 8 MiB) and tombstone maps.
Rename now checks source/destination, private-only provenance, item
and tombstone quotas, future item count and private byte budget
before mutation. It moves one existing private Vec directly or
copies one bounded synthetic host fixture value, keeping original
tombstone and derived-data tag. Public API and slot/process gates
are unchanged. One focused regression authored: full item capacity,
identical Vec allocation after rename, and atomic capacity refusal
with full tombstones.

**Proof:** pinned exact GitHub source/static review. No Rust compiler
or cargo currently available; unit test NOT_EXECUTED. Windows
file/HKCU interception, SCM service, Microsoft WAM/broker isolation,
Minecraft gameplay and four-client acceptance NOT_EXECUTED.
No service installation, driver, CI, Sandboxie integration, profile
data mutation, or Launch/Provision MC02-MC04 activation.

**Next:** improve first Windows private-root HANDLE provenance gate
using the already existing disposable file/registry fixtures,
without authenticating a game process or granting privileged IPC.



## Independent service — skip unsigned/unconfigured image work and add native mapping fixture, 22 October 2026

Corrected an avoidable cost in the original opt-in service IPC owner.
As of this Local HEAD, the reviewed MIVUBI desktop signer leaf policy
is EMPTY, so every request is necessarily non-authorizing.
ipc_worker::compare_native_peer now refuses requests BEFORE process
HANDLE opening, WTS/token work, WinTrust or NtQueryInformationProcess.
The canonical signer-policy helper rejects empty, all-zero or duplicate
DER-leaf hashes; tests cannot introduce production pins. This is an
efficient fail-closed preflight, not a new authentication bypass.

A Windows-native self-process fixture test was authored under
peer_image.rs to require exact ProcessImageFileMapping(44) success
against its own retained read+execute executable FILE HANDLE, plus
the existing FILE_OBJECT identity stability check. An unsupported
Windows native class must fail the test rather than passing using
path equality. A pure policy test covers malformed and duplicate
signer pins. Both newly added tests are NOT_EXECUTED.

**Proof:** exact pinned GitHub source/static review and Microsoft
NtQueryInformationProcess warning plus public native class-44
reference. No Rust compiler, Windows-native tests, SCM or Minecraft
tests executed. CI remains disabled. The signer policy stays EMPTY,
readonly-ipc-pilot stays OFF by default, service not installed,
DACL remains admin-only, independent provider disabled and
MC02-MC04 launch/provision still BLOCKED. No installed Sandboxie,
account/world/credential, registry or driver mutation.

**Next:** run the self-process mapping fixture and review the actual
signed desktop release signer/section identity when a Windows
environment becomes available, before enabling an IPC client.



## Independent service — exact executable section-to-file negative check, 22 October 2026

Enhanced the ORIGINAL service-owned peer_image.rs rather than adding
a second trust manager. Retained pipe-peer PID/full FILETIME native
process HANDLE now requests PROCESS_QUERY_INFORMATION (query-only)
and opens the OS-reported executable pathname read-only with
FILE_EXECUTE + SYNCHRONIZE. Before AND after WinVerifyTrust and
disk FILE identity/path recheck, dynamically resolved
NtQueryInformationProcess(ProcessImageFileMapping=44) checks the
opened file HANDLE against the process image SECTION object.
Only NTSTATUS_SUCCESS 0 qualifies; STATUS_UNSUCCESSFUL means
different/missing file and all other statuses fail closed with no
fallback to queried path, file name or generic Authenticode.
A common System32-only DLL RAII owner handles wintrust.dll and
ntdll.dll without introducing a service driver or loader route.
The WinTrust leaf digest still requires a reviewed policy which
is intentionally EMPTY. Tests authored for NTSTATUS gate, explicit
no-write FILE access mask, and section+signer dual requirement.

**Proof:** pinned GitHub source/static Win32 API review; class-44
semantics corroborated by public NT internals research, NOT an
official stability guarantee. Microsoft explicitly warns native
NtQueryInformationProcess may change. Compiler/Rust tests, Windows
actual section/file matching, digital signature verification, SCM,
Minecraft and four-client acceptance NOT_EXECUTED. This feature
still does not authenticate a real desktop publisher, resist
same-logon duplicated pipe handles or establish filesystem/HKCU/WAM
isolation. readonly-ipc-pilot remains default OFF, service uninstalled,
admin-only DACL unchanged, Launch/Provision MC02–04 blocked.

**Next:** Windows-native validation of ProcessImageFileMapping and
real signed MIVUBI installed desktop publisher/source provenance,
then independently review handle-duplication resistant IPC identity
before activation. No GitHub Actions or PC tests were started.



## Independent engine — service-owned native desktop Authenticode preflight, 22 October 2026

Added the Windows-only peer_image.rs observer to the standalone
independent-service feature-gated source owner. It reads the actual
OS-connected (PID + FILETIME checked) pipe-client process HANDLE
and uses QueryFullProcessImageNameW to open the observed executable
on disk. The read-only file HANDLE is pinned across WinVerifyTrust
with WTD_UI_NONE, whole-chain + CACHE_ONLY_URL_RETRIEVAL, and
verified primary signer leaf DER SHA-256 extraction through
System32 wintrust.dll WTHelper exports. The WinTrust provider state
always closes when allocated; file object identity/path and process
liveness are rechecked before returning sanitized, private evidence.
Unsigned/invalid/missing providers, changed files and unavailable
process identities fail closed.

**Reviewed publisher policy is deliberately EMPTY.** No verified
MIVUBI installer desktop leaf fingerprint or running mapped PE
section-to-file binding exists, so no generic signed executable
can claim trusted MIVUBI origin. The opt-in pilot now consults
this preflight after native WTS/token checks but still grants no
new endpoint, reply or privileged operation. Three regression tests
are authored for empty pin list, unknown signer, null handle.
Existing SCM session-change revocation, admin-only DACL, default-
OFF feature and game Launch/Provision denial are unchanged.

**Proof:** pinned GitHub source review and Microsoft
QueryFullProcessImageNameW/WinVerifyTrust/WINTRUST_DATA contracts.
Windows Rust compilation, signature fixture tests, installed SCM
and Minecraft native/multiclient acceptance NOT_EXECUTED. No user
account, world, game profile, Windows driver, service installation,
Sandboxie runtime or main branch was modified.

**Next:** prove native WinTrust signing and section/file binding
against a legitimately signed installed MIVUBI desktop release,
before any nonadmin IPC admission.



## Independent service — fail-closed on Windows session-change notification, 22 October 2026

Updated the existing independent-service/windows_host.rs SCM owner
and feature-gated ipc_worker.rs. With readonly-ipc-pilot compiled
(OFF by default), Running now opts into SERVICE_ACCEPT_SESSIONCHANGE.
Any ServiceControl::SessionChange event (logon/logoff, lock/unlock,
console/RDP connect/disconnect, etc.) sends a bounded nonblocking
signal through the SAME existing Stop/Shutdown receiver; duplicate
signals coalesce and the SCM event's SessionId is not used as
authorization. The worker reports StopPending before cancelling
and awaiting overlapped I/O, disconnects the single admin-only
named pipe and transitions to Stopped. It also checks the stop
channel directly before and after native peer/token inspection.
The ordinary default SCM host does not subscribe to session-change
and its startup/shutdown behavior is unchanged. Two Windows-only
service-route regressions authored.

**Proof:** pinned GitHub exact-source/static review and verified
windows-service 0.8.1 ServiceControl/ServiceControlAccept API +
Microsoft SERVICE_CONTROL_SESSIONCHANGE documentation. Native
Windows build, control-delivery tests, compiler/test execution,
SCM installation, and Minecraft gameplay NOT_EXECUTED. SCM
notifications may be missed before registration; current native
WTS/token rechecks are still required. No nonadmin DACL applied,
service/driver installation, credentials/world/profile mutation,
Sandboxie runtime dependency or MC02–MC04 launch enabled.

**Next:** design signer-pinned service-side desktop executable
provenance and verify source/Windows SCM lifecycle before issuing
per-logon endpoints or attempting filesystem/HKCU enforcement.



## Independent service — native effective AccessCheck for per-logon pipe candidate, 22 October 2026

Corrected the existing SDDL candidate by adding explicit SYSTEM
Owner/Group (O:SYG:SY); Windows AccessCheck requires both. Within
the canonical service_identity.rs owner, added non-mutating native
descriptor conversion, checks for owner/group and non-NULL protected
DACL, DuplicateTokenEx of a verified WTS primary user into a
query-only SecurityImpersonation token, and Windows AccessCheck for
the minimal FILE_WRITE_DATA|SYNCHRONIZE granted rights versus
FILE_CREATE_PIPE_INSTANCE denial. The privilege set is capped at
4096 aligned bytes and all ambiguous/error results fail closed.
If additional WTSActive sessions exist, their Windows primary user
tokens are checked against the same candidate for expected denial;
missing evidence or active-session churn refuses this diagnostic.
No simultaneous peer means no peer-denial proof. Two native
regressions authored for actual current-token AccessCheck and
missing descriptor components; no tests executed.

**Proof:** pinned GitHub exact-source static review and Microsoft
AccessCheck, SDDL, DuplicateTokenEx and named-pipe documentation.
Rust/Windows compile, interactive token tests, SCM installation,
ACL access on real pipe handles and Minecraft gameplay NOT_EXECUTED.
The candidate is NEVER installed, and the admin-only pilot DACL,
readonly-ipc-pilot default-OFF feature, provider disabled state,
MC02–MC04 Launch/Provision block, account/world and Sandboxie
runtime retirement remain unchanged.

**Next:** Windows ABI/runtime check of candidate grant and cross-logon
denial, then design verifiable desktop publisher/session enrollment
and revocation prior to any actual nonadmin service endpoint.



## Independent service — per-logon least-rights ACL candidate and relogon recheck, 22 October 2026

Within the existing SCM identity source owner, added a private,
non-authorizing SDDL candidate constructor sourced ONLY from the
Windows-resolved service SID and WTS-verified unique enabled logon
SID. SYSTEM and the verified service SID have GA; the single
session's logon SID gets precisely FILE_WRITE_DATA|SYNCHRONIZE
(0x00100002), never FILE_CREATE_PIPE_INSTANCE/GENERIC_WRITE.
Canonical service/logon syntax refuses SID/ACE injection, broad
BUILTIN Users/Everyone and cross-logon reuse. This does NOT bind,
update or open a per-logon pipe. The current pilot's administrator
DACL remains unchanged and the build feature stays default-OFF.

The WTSActive diagnostic additionally re-queries the user's
native PRIMARY token after the second session enumeration,
requiring the same User SID, SessionId, and unique enabled logon
SID. This reduces exposure to session-number reuse but never
promotes the result to production authentication or bypasses
the known duplicated-handle provenance limitation. All handles
are closed and candidate SDDL is dropped. Two source regressions
cover precise client mask/DACL content and cross-logon/SID spoofing.

**Proof:** pinned GitHub source/static review + Microsoft Windows
named-pipe ACL/SERVICE_SID/TokenGroups documentation. No Rust
compiler in this environment, native code/tests/SCM and Minecraft
NOT_EXECUTED. No service/driver installation, existing Tauri routes,
Sandboxie runtime, accounts/worlds or MC02-04 Launch/Provision changed.

**Next:** native Windows source compile and effective ACL tests,
followed by a trustable desktop-origin and logon-revocation design
before any actual nonadmin endpoint, filesystem or registry enforcement.



## Independent service — unify bounded SID reads and cross-check WTS user, 22 October 2026

Corrected two real validation gaps within the existing independent
SCM service owners, without creating an alternate lifecycle manager.
service_identity.rs now owns the only bounded native User SID reader
used by the service IPC. Both TokenUser and TokenGroups reuse the
same function, which verifies SID pointers, header and full body
within the returned GetTokenInformation allocation BEFORE any
IsValidSid/GetLengthSid call. The previous second TokenUser
implementation in ipc_worker.rs was removed. Active WTS console/RDP
observations now compare User SID from the native process primary
token to the WTSQueryUserToken primary user, in addition to
session, logon SID and existing last-message authentication checks.
Tests authored for foreign/truncated SID memory and TokenUser C ABI.

**Proof:** GitHub exact-source/static review and official Windows
GetTokenInformation/WTS token contracts. Rust/Windows compilation,
native tests, installed SCM and Minecraft all NOT_EXECUTED.
The opt-in pilot stays disabled by default with the same protected
admin-only DACL. Nothing enabled or installed; no account/world
mutation, driver, Sandboxie dependency, game Launch/Provision or
new user IPC rights. WTS evidence remains diagnostic, not binding
to the exact process that wrote a shared pipe message.

**Next:** Windows-native compiler/ABI tests, then verified per-logon
endpoint issuance with strict access rights and no untrusted caller
SID or PID as initial authority.



## Independent service — enumerate active console and RDP sessions per OS peer, 22 October 2026

Corrected the source-only physical-console startup gate. The service
must be able to bind before any user logs on: moved session checks
from named-pipe creation to each received read-only IPC message.
The existing native service identity owner now calls local
WTSEnumerateSessionsW, frees its buffer with WTSFreeMemory, and
accepts only bounded unique WTSActive console/RDP IDs. No WinStation
names or broker-provided IDs are used. A real connected process
token supplies its TokenSessionId and logon SID; WTSQueryUserToken
for that active ID must yield the same session and logon SID as a
primary OS token. Re-enumeration rejects inventory changes. Missing
LocalSystem/SeTcbPrivilege, service SID, invalid session states and
unavailable tokens fail closed. Two source tests added.

**Proof:** pinned exact GitHub static source review and Microsoft
native WTS docs; Rust compiler and Windows runtime unavailable.
All compile, WTS/SCM, token and Minecraft acceptance NOT_EXECUTED.
Default readonly-ipc-pilot remains OFF, named pipe remains admin-only,
no ordinary-user enrollment/privileged IPC authorization is granted,
and no Sandboxie runtime, game world or account data was changed.

**Next:** Windows-native validation of enumeration/peer-token matching,
then trusted per-logon DACL enrollment independent of wire claims.


## Independent engine — Windows-attested physical-console candidate, 22 October 2026

Added a read-only WTS user-session candidate check to the existing
opt-in service identity owner, invoked before named-pipe creation.
The OS provides the physical console session, not an IPC claim.
Reject zero/detached sessions, WTSQueryUserToken failure,
non-primary token, wrong TokenSessionId, missing enabled logon SID,
or changing console ID. WTS user token is closed immediately; no
account, SID, token or session ID is stored or used to widen DACL.
RDP and secondary sessions are not covered. Two new negative tests
were authored.

**Proof:** pinned GitHub source/static review and Microsoft WTS docs.
Rust compiler unavailable in ChatGPT environment; Windows build,
SCM/WTS queries, tests and Minecraft NOT_EXECUTED. The source-only
feature stays OFF by default, service uninstalled, Minecraft launch
blocked, Sandboxie retired and existing user data untouched.

**Next:** validate native WTS session identity on Windows before
designing an authenticated nonadmin/multi-session IPC endpoint.


## Independent service — SCM SID and last-message logon SID, 22 October 2026

Created service_identity.rs under the feature-gated standalone
independent-service crate. It checks actual Windows LookupAccountNameW
for NT SERVICE\MIVUBIIndependentHost and verifies that SID is enabled
(not deny-only) in the actual SCM process TOKEN_GROUPS. Endpoint
creation now fails closed unless this is true. The protected DACL
adds the verified service SID to SYSTEM and the existing limited
Administrators client grant, without Users/Authenticated Users.

The last-message and connected-process access tokens now also need
matching, unique enabled SE_GROUP_LOGON_ID S-1-5-5-X-Y groups.
Existing comparisons of User SID, Windows session, logon LUID
and impersonation level remain required, with mandatory RevertToSelf.
No raw SID/token value is sent to desktop or written to the repo.
Three native regression tests authored for SID validation and ABI.

**Proof:** exact GitHub source/static Win32 contract review only.
Windows compile/SCM/pipe tests, signing and Minecraft NOT_EXECUTED.
The pilot stays OFF by default, admin-only, uninstalled, and cannot
perform privileged/game operations. Trusted ordinary-user SID
enrollment and same-logon pipe handle provenance remain unresolved.
No Sandboxie runtime or existing world/account changed.

**Next:** native Windows validation of service SID, token group and
effective DACL before considering nonadmin client authorization.


## Independent SCM IPC — read and compare the last-message Windows token, 22 October 2026

Extended the **existing** opt-in, service-owned IPC worker (not a new
service/manager). Once the 32-byte request and actual pipe peer PID plus
creation FILETIME are checked, it performs native OpenProcessToken and
ImpersonateNamedPipeClient on the last message. OpenThreadToken and
GetTokenInformation compare User SID, Windows session, logon LUID
(TOKEN_STATISTICS.AuthenticationId), token type and impersonation level
with the connected process token. A pre-impersonated service thread is
refused, all errors fail closed and a RevertToSelf guard runs on
every successful impersonation path (fatal on failed revert). Final
pipe PID/liveness are checked again, and mismatches are discarded.
Token/SID/logon values are not serialized, persisted, logged or used
for credentials/Minecraft. Two regression tests cover all four negative
evidence gates and the native ABI layout.

**Proof:** GitHub pinned source and Microsoft Win32 API/ABI review.
Rust/Windows build, SCM startup/shutdown, named-pipe token tests and
Minecraft gameplay NOT_EXECUTED. Even full token equality is ONLY a
pending diagnostic: duplicate pipe handles from the same logon still
prevent strong message provenance, and service/logon SID DACLs remain
missing. The readonly-ipc-pilot stays OFF by default, admin-only,
unpackaged and uninstalled. No Sandboxie runtime dependency or
MC02-04 Launch/Provision authorization was introduced.

**Next:** review native Windows compilation and logon/service-SID pipe
policy before allowing non-admin clients or real engine operations.


## Independent service IPC — publish StopPending before drain and normalize oversized messages, 22 October 2026

Corrected the feature-gated service-owned IPC pilot:
SCM Stop/Shutdown now advances HostLifecycle/SCM StopPending
BEFORE CancelIoEx completion can block in a worker. A failed SCM
status update becomes an error, not a clean STOPPED transition;
RAII still drains pending kernel IO before freeing buffers.
Immediate and asynchronously completed ERROR_MORE_DATA both
reject an oversized request without falsely marking the host
as failed or accepting the truncated payload. Two pure Rust
regressions were authored for stop status announcement and
status-report failure. No new endpoint, privileges or opcodes.

**Proof:** pinned GitHub source/static review. Rust/Windows tests,
SCM execution, privileged peer-token authentication and Minecraft
runtime NOT_EXECUTED. Default service remains inert; pilot is
OFF by default and no account/world/driver data were modified.

**Next:** Windows compilation and SCM stop/cancellation acceptance
of the feature-gated service-owned worker before enabling any
privileged operations.


## Independent engine — service-owned opt-in IPC ingress with stop-safe cancellation, 22 October 2026

The separate independent-service crate now supports the explicit
compile-only feature `readonly-ipc-pilot` (OFF by default).
With that feature, its SCM host creates one protected inbound-only,
local-only named pipe *inside the service process* BEFORE reporting
SCM Running. The DACL admits only SYSTEM and Administrators, with no
generic-write pipe-instance rights for the admin client. The worker
uses the existing canonical 32-byte broker wire (optional dependency
on runtime-core), checks the OS-connected PID and creation FILETIME,
then discards messages without replies or privileged effects.

The worker owns all Win32 events, a boxed stable OVERLAPPED and
bounded 32-byte receive buffers. On SCM Stop/Shutdown it cancels
the specific operation and waits for the final OS completion before
dropping any memory; service StopPending/Stopped is not published
until the worker returns. A pathological kernel that never settles
cancellation may prevent prompt stop, but cannot cause an I/O
use-after-free. Invalid or idle connections are discarded with
bounded retry. No service install, startup feature enablement,
driver, world, Microsoft account, Tauri provider or Minecraft launch
was changed. Default builds remain the previous inert SCM host.

**Proof:** pinned GitHub source/static review only. Cargo compilation,
Windows SCM/IPC execution and native security tests NOT_EXECUTED.
The pilot is NOT authenticated for non-admin users: separate logon
SID, service SID and message impersonation/token checks are missing.
This new source is not safe to install or ship as an isolation engine.

**Next:** perform a Windows-compatible build and review peer SID/logon
token authentication for the service-owned pipe before activation.


## Independent engine — original standalone Windows SCM service host, 22 October 2026

Created a standalone crate under
engine/isolation-native/independent-service/. It uses Windows SCM
through windows-service 0.8.1 and adds an independently authored
service-private StartPending -> Running -> StopPending -> Stopped
lifecycle with nonblocking Stop/Shutdown signaling. Three pure Rust
regression tests cover ordered stop, invalid transitions and
terminal failure. The service name is distinct from SbieSvc.

This component is an inert SCM process only: NO service registration,
installation, Tauri/desktop connection, named pipe, privilege use,
filesystem/registry mutation, account/session data, driver loading,
Minecraft launch or Sandboxie dependency. SCM RUNNING does not mean
the game sandbox engine works. Existing Launch/Provision rejection
for MC02–MC04 stays unchanged.

**Proof:** exact GitHub source/static review, tests authored but
NOT_EXECUTED. Rust compiler, Windows SCM, packaging, signing and
Minecraft remain NOT_EXECUTED. GitHub Actions stays disabled.

**Next:** verify a native Windows build of this standalone service
host, then implement an authenticated cancel-safe service-owned
IPC worker without reusing the desktop fixture trust boundary.


## Independent engine — exact oversized/short overlapped pipe message rejection, 22 October 2026

Follow-up correction in the existing Windows fixture pipe owner:
ERROR_MORE_DATA is now mapped to InvalidData whether ReadFile returns it
immediately OR GetOverlappedResult reports it after a pending operation.
One additional Windows test rejects a complete 31-byte message without
treating it as a valid diagnostic frame. Earlier tests still cover
33-byte oversized messages, forged FILETIME, missing clients and silent
clients. All Windows tests authored NOT_EXECUTED; static source review
only. No new service, Minecraft launch, Windows driver or credentials
were touched. The fixture remains internal and non-authorizing.

**Next:** native Windows compilation and fixture execution when a
compatible Windows environment becomes available; continue source
design of a durable service worker without granting any production
privileges until the enforcement boundary is proven.


## Independent engine — cancel-safe overlapped IPC fixture, 22 October 2026

Corrected the one-client native pipe fixture to use FILE_FLAG_OVERLAPPED,
manual-reset events, bounded ConnectNamedPipe/ReadFile timeouts (1500 ms
each), targeted CancelIoEx and completion acknowledgment before
releasing OVERLAPPED or input bytes. For the disposable fixture only,
an IO cancellation that fails to complete within 2500 ms aborts rather
than risking use-after-free. Added native tests for a missing client and
a client that connects but never sends a message; prior forged-identity,
oversized and valid-message tests remain.

**Proof:** GitHub pinned-source/static review only. Windows and Rust
compilation, native IO cancellation tests and Minecraft gameplay are
NOT_EXECUTED. No production service, kernel driver, game launch,
account data, installed sandbox binaries or Tauri route changed.
The fixture still cannot authenticate a privileged client or enforce
filesystem/registry redirection. Aborting the process on stuck IO
is not a suitable long-lived service recovery model.

**Next:** design a durable service-owned named-pipe worker with
independently owned IO memory, cancel-safe shutdown and an authenticated
client/root trust boundary.


## Independent engine — explicit-DACL, last-message-token named-pipe fixture, 22 October 2026

Added apps/desktop/src-tauri/src/engine/independent_pipe_server.rs:
a fresh, one-instance, noninheritable, inbound-only, local-only
message pipe protected by an explicit SDDL DACL (current fixture
Windows user, SYSTEM, Administrators; no default Everyone/Anonymous
grants). Its fixed 32-byte read rejects malformed, long and short
messages before calling the existing OS client PID/full FILETIME
observer. It then impersonates the **last successfully read message**
and checks the SID/session of the thread's impersonation token
against the original current-user token. Every successful impersonation
is followed by RevertToSelf via an RAII guard; an unrevertable thread
aborts. All outputs are explicitly PENDING diagnostics. Three authored
Windows-native tests cover same-process local transport, oversized
payload rejection and spoofed FILETIME denial.

**Scope/proof:** exact GitHub static source review. Windows compiler
and test execution NOT_EXECUTED; no installed service, authenticated
privileged broker, deadline-safe overlapped server or game launch
is claimed. Same-user fixture DACL grants are unsuitable for a
production SYSTEM service. This remains unmounted, never receives
accounts, worlds, tokens, runtime configuration or Minecraft
processes. MC02–MC04 Launch/Provision remains BLOCKED; Sandboxie
runtime is not linked or activated.

**Next:** design and implement a service-owned, cancellable
named-pipe endpoint with a restricted service/client SID policy
before permitting any privileged operation or integration.


## Independent engine — typed read-only broker wire and native pipe peer inspection, 22 October 2026

Added one original, versioned 32-byte broker diagnostic request contract
in engine/runtime-core/src/independent_engine/broker_wire.rs. Exactly two
read-only operations exist (observe process; inspect root ACL). The
LE parser rejects malformed size/magic/version, unknown operations,
MC01, zero/invalid PID/FILETIME/correlation and nonzero reserved bytes;
Rust regression tests cover stable roundtrip and adversarial inputs.
The frame is explicitly UNTRUSTED and cannot launch or configure
Minecraft. No separate manager, credentials store or lifecycle state
was added.

An unmounted Windows-only adapter at
apps/desktop/src-tauri/src/engine/independent_pipe_peer.rs checks
GetNamedPipeClientProcessId against the frame's claimed PID, reopens
the OS client's process with query-only rights, verifies full process
creation FILETIME and live state, and rereads the pipe peer PID.
Its result is ONLY a pending candidate; a pipe handle or message
can be duplicated, and no client SID/session/DACL/impersonation
validation or trusted server exists yet. It never creates a listener
or alters an account, service, driver, installed Sandboxie or world.
MC02–MC04 Launch/Provision stays denied. No Sandboxie code linked.

**Proof:** GitHub exact-source/static review. Rust/Windows compiler
and native IPC execution NOT_EXECUTED; Minecraft runtime NOT_EXECUTED.
The two pure-Rust tests are authored only, not represented as PASS.

**Next:** build the independently owned, read-only named-pipe server
fixture with restrictive ACL and fail-closed client token verification
before introducing any privileged service operations.


## Independent engine — read real Windows ACL rights for two owned fixture processes, 22 October 2026

Introduced apps/desktop/src-tauri/src/engine/independent_root_access.rs:
read-only GetSecurityInfo OWNER/GROUP/DACL plus OpenProcessToken,
DuplicateTokenEx (SecurityImpersonation) and AccessCheck for two
distinct engine-owned, still-live process HANDLEs. The file fixture
root is opened with READ_CONTROL. Owner/peer directory list/create
results are sanitized booleans; no account, SID, ACL, token or path
data are serialized. Null DACL, missing owner/group, same process,
expired process, unfamiliar rights or native API failure fail closed.
The existing native file fixtures now check that different same-user
processes retain equivalent effective root rights, so a private-looking
folder does NOT yield an isolation claim. Tests cover classification,
same-process refusal and stale-peer refusal. No provider is promoted.

**Proof:** exact GitHub source/static review. Windows native compile
and runtime tests NOT_EXECUTED; Minecraft gameplay NOT_EXECUTED.
This is actual Windows authorization-API source, NOT a trusted
per-client root or real filesystem/registry interception. A sibling
test run with positive peer access is an explicit NO-GO for promoting
this candidate. App data/account state and the installed Sandboxie
driver/service remain untouched. MC02–MC04 Launch/Provision is BLOCKED.

**Next:** validate the Windows native fixture tests and prove
trusted service-owned ACL/peer boundaries before designing production
filesystem/registry enforcement.


## Independent engine — bind native fixture storage to a retained owned process, 22 October 2026

The two independently authored Windows primitives, Job Object process
ownership and handle-relative NT storage, are now joined for
**fixture-only lifetime enforcement**. The native launcher exposes only
a borrowed, suspended, self-created process capability. File roots
cannot be constructed without that capability and a single MC02/03/04
slot. Every storage operation rechecks the process HANDLE's full
PID/FILETIME plus non-exited status; cross-slot reuse is denied. Rust
borrows prevent owner resume/destruction while a root or file lease
is held. The native test suite now includes three file tests:
distinct live owners/shared root, wrong-slot rejection, and write/read
denial after the owned fixture terminates, plus previous traversal/
collision checks. No process name, user-supplied PID or path creates
a storage capability.

Source/static verification only; Windows compilation and fixture
execution NOT_EXECUTED. This does not give a private SID/DACL,
same-user peer access denial, transparent Windows IO interception
or WAM/Minecraft account separation. No production provider, driver,
service, installed dependency, Tauri command, existing account/world
file or MC02–MC04 Launch/Provision was changed.

**Next:** implement and validate a trusted native service-owned root
with Windows token-based effective access/peer-denial proof before
selecting a filesystem/HKCU enforcement mechanism.


## Independent engine — native handle-relative per-slot file I/O, 22 October 2026

Added an original Windows NT filesystem adapter in
apps/desktop/src-tauri/src/engine/independent_private_files.rs.
It holds a pinned root directory and opens one checked child
relative to an OS HANDLE via NtCreateFile, using FILE_CREATE/FILE_OPEN,
FILE_OPEN_REPARSE_POINT, strict file type/volume/link inspection and
directory HANDLEs that do not permit sharing DELETE. It creates
separate MC02–MC04 slot directories and writes fixed, nonsecret,
eight-byte fixture markers, flushes to real files and reopens them.
MC01, invalid/DOS-device/traversal names, missing files and
existing-file overwrites fail closed. Two Windows-only regression
tests authored NOT_EXECUTED; no Windows target compile or Minecraft
game test was run. No Sandboxie runtime/dependency is used.

**Proof boundary:** This is original native disk I/O, not a file
virtualization provider: no hook/minifilter connects to Minecraft.
Fixture roots are freshly created but not process-SID/ACL-attested;
the implementation is unmounted and remains unsuitable for user
worlds/accounts. Existing MC02–MC04 Launch/Provision and production
driver/service operations remain blocked; no existing data changed.

**Next:** implement and prove the OS-enforced per-process file/
registry redirection mechanism, its privileged owner and recovery
contract before attempting Minecraft multi-account gameplay.


## Independent engine — native application-hive I/O under a pinned Windows directory, 22 October 2026

Added an unmounted original Rust Win32 adapter at
apps/desktop/src-tauri/src/engine/independent_private_hive.rs.
It holds an ordinary directory HANDLE (no final reparse follow,
no FILE_SHARE_DELETE) and checks the same directory volume/file identity
across operations. Registry hives are loaded using RegLoadAppKeyW into
unlisted handle-only namespace A; the adapter never opens or overrides
HKCU/HKU. Only the constant MC02/03/04 disposable test names exist.
Binary canary writes/reads are restricted to the returned HKEY.
Two Windows-only tests exercise separate native hive files,
close/reopen persistence and refusal of native MC01 or pre-existing
hive files. Both tests are authored NOT_EXECUTED; no Windows compiler
was available. Rust Cargo feature Win32_System_Registry was added.

**Safety and limitation:** This module cannot transparently virtualize
the unmodified Minecraft process registry. The fresh test root lacks
a trusted SID/ACL/ancestor trust chain, and registry file creation
is not yet handle-relative. It must remain a fixture until per-process
ownership and native enforcement are validated. No service/driver,
UI commands, Sandboxie binary/dependency, account/world data or
production Launch/Provision changed. Client 02–04 remain blocked.

**Next:** source-design and prove a process-bound Windows filesystem/
HKCU interception mechanism compatible with unmodified Education
and the Microsoft authentication broker before enabling the provider.


## Independent engine — first original Win32 Job Object process primitive, 22 October 2026

Added a Windows-only independently authored process/job primitive in
apps/desktop/src-tauri/src/engine/independent_job.rs. It creates an OS
Job Object, launches only the **own Rust test executable** suspended,
assigns it to the job, obtains exact PID/full creation FILETIME, then
resumes the main thread. Two Windows fixture tests are authored for
actual OS job-membership and cross-job rejection. Failed enrollment,
identity or resume terminates only that disposable child.

This is a real native process lifecycle primitive, NOT a sandbox. It
does not intercept files, HKCU, WAM, COM/IPC or game singleton state.
Kill-on-close is restricted to disposable fixtures; a persistent game
requires a durable service owner. No Sandboxie service, driver or DLL
is linked or called. No provider, Tauri command, Minecraft launch, box,
user data, installed driver or signing state is changed. MC02–MC04
Launch/Provision remain BLOCKED. Windows compile and fixture tests
NOT_EXECUTED; four-client gameplay NOT_EXECUTED.

**Next:** design and implement a bounded, process-identity-bound native
filesystem/registry enforcement adapter before permitting game launch.


## Independent engine — track synthetic host-derived copy-up provenance, 22 October 2026

Strengthened private-only routing against a copy-up provenance bypass.
`SyntheticSlotOverlay` now internally tracks host-derived resources
when base values are copied into private memory, partially overwritten,
or truncated. Moving a host-derived item through another nonprivate
path retains its origin. Renaming it into a private-only prefix is
denied even when the source is already in the private map. Full
explicit overwrite is treated as newly authored synthetic bytes and
clears the marker; deletion and subtree deletion release markers
alongside private entries. Two Rust regressions cover direct and
multi-step copy-up/rename and partial-write/truncate propagation.

**Boundary:** This is *synthetic provenance*, not information-flow
control of real Windows processes or memory; an application can still
copy bytes through its own buffers. Real sandbox data policies need OS
enforcement and a separate trust boundary. Rust tests authored
NOT_EXECUTED, no Windows/host/game access.

**Next:** Stop expanding pure-memory proof scaffolding and design
the independently implemented native virtualization/enforcement
layer needed to intercept unmodified Minecraft file/registry/IPC
operations. Do not claim a live sandbox from in-memory tests.


## Independent engine — deny untrusted synthetic host imports into private-only prefixes, 22 October 2026

Tightened `independent_engine/overlay.rs` against two policy-order
bypasses. Installing a private-only prefix after existing private bytes
is rejected with `AmbiguousPrivateProvenance`: without a trusted
copy-up provenance log the engine cannot know whether preexisting
bytes came from the shared base. A rename that reads a host-backed
source and targets a private-only prefix is rejected with
`HostCopyIntoPrivateBoundary`, while an already privately owned
source may be renamed there atomically. The original base and other
slots never change. Two Rust regressions authored NOT_EXECUTED.

These checks model desired confidentiality rules. They are not native
Windows filesystem security, a real registry virtualizer, or game
authentication proof. No privileged ops or game data were touched.

**Next:** Assess the true Windows enforcement requirement to intercept
unmodified Education filesystem/registry operations. Do not equate
synthetic COW with functional four-instance isolation.


## Independent engine — per-box private-only host fallback boundaries, 22 October 2026

Extended the original synthetic `overlay.rs` COW core with an explicit
`make_private_only` prefix policy. File and registry-value namespaces
remain separate. Beneath such a prefix the in-memory merged read and
enumeration no longer fall back to any synthetic host baseline;
copy-up rejects absent private sources, while explicitly written
private values remain readable. A peer session's policy and the immutable
base bytes remain unchanged. The policy is bounded, per-slot, and
bound to the existing process PID/FILETIME. Two Rust regression tests
cover denied reads/copy-up, private writes/listing, cross-surface
independence and peer/host integrity; authored NOT_EXECUTED.

The prefix policy is a **simulation of desired sandbox read behavior**,
not enforcement of Windows host filesystem, ACL, registry or broker
operations. No launcher or kernel hook is registered, and no actual
Minecraft account/world data was touched.

**Next:** finish source review of COW model limitations before selecting
a Windows API interception/enforcement mechanism with explicit
compatibility, signing, process lifetime and rollback proofs.


## Independent engine — preserve COW subtree tombstones and implement bounded partial writes, 22 October 2026

Fixed an edge case in the new synthetic COW engine: private write
formerly removed an equal-key tombstone, which could make unrelated
host descendants reappear after deleting a subtree. Tombstone
masking now survives private writes (private entries still win during
lookup). Added `write_at` and `truncate_file` for byte-offset
copy-up from the immutable synthetic host view. Sparse writes
zero-fill PRIVATE memory only; operations reject registry-value
offsets, capacity overflow and invalid process/slot keys before
mutation. Two Rust regression tests cover subtree resurrection,
partial writes, sparse zero-fill, truncation and peer/host preservation,
authored NOT_EXECUTED.

This remains a synthetic in-memory behavioral core. It does not
intercept or modify real Windows filesystem or registry calls,
publish a launcher or establish Minecraft compatibility.

**Next:** Source-audit COW namespace invariants and implement a bounded
synthetic directory/key enumeration contract before considering any
privileged native adapter.


## Independent engine — functional synthetic copy-on-write overlay core, 22 October 2026

Added original Rust source in the existing canonical
`engine/runtime-core/src/independent_engine/overlay.rs`. This is an
**executable in-memory overlay algorithm**, not another observation-only
check: it merges an immutable synthetic base with per-MC02–MC04 private
entries, performs copy-up and private full-value writes, tombstone deletes,
prefix/subtree masking, deterministic merged enumeration and transactional
same-surface renames without ever mutating the shared base. It handles
both file leaf resources and registry value leaves using independently
normalized namespaces; all operations validate exact slot and process
PID/FILETIME, deny MC01, bound path structure, individual byte counts,
private memory quota and item count. Seven Rust unit regressions authored
for case normalization, traversal/device denial, independent COW,
registry separation, subtree tombstones, mismatched process/peer, atomic
rename and quotas.

**Critical limit:** `SyntheticBaseView` is deliberately an immutable
in-memory fixture, NOT real Windows host data. Resource keys are strictly
synthetic/ASCII: they cannot represent arbitrary Minecraft Unicode
world filenames, ACLs, streams, hardlinks or true registry trees.
This module does **not** hook APIs, redirect AppData/HKCU, own a
trusted filesystem root, start a process, modify accounts or authorize
Launch/Provision. The semantics are a useful concrete engine building
block; they do not make the product sandbox operational.
Rust compiler/tests and target Windows execution NOT_EXECUTED.

**Next:** Integrate a safely authenticated private resource adapter that
enforces real Windows file/registry semantics, or pause that mutation
until signed native enforcement and broker compatibility are proved;
do not promote the synthetic overlay to a provider.


## Independent engine — Windows provider feasibility and all storage escape denials, 22 October 2026

Verified Microsoft's supported Windows behavior across unpackaged
AppContainer, RegLoadAppKeyW, process-local RegOverridePredefKey,
LOGON_WITH_PROFILE user profiles, WAM interactive session requirements
and MSIX's virtualized packaged desktop forms. Compared those against
the single-desktop, unmodified Education 4-account requirement in
docs/system/independent-isolation.md with direct official links.
No complete, proven provider exists. Package form alone is not MSIX
virtualization or game-account proof.

Corrected the first wrong source owner,
tooling/acceptance/evaluate-appcontainer-isolation.mjs: its existing
synthetic candidate review now classifies any observed Local/Roaming
environment or Known Folder escape as NO-GO, rather than only Roaming
Known Folder escapes. Roaming Known Folder takes priority, so the
historical pilot diagnosis remains identical. A missing Known Folder
without any other observed escape remains UNVERIFIED; an observed
APPDATA environment escape with a missing Known Folder is NO-GO.
No schema changes, duplicate admission/lifecycle state or provider
selection are introduced. MC02–04 production Launch/Provision is untouched.

Added three adversarial JavaScript tests plus improved the missing-Known-
Folder scenario. All evaluator callback tests were executed via ChatGPT
V8 and the existing minimal assertions against the exact candidate
function; Node's native test runner, Windows source/native compilation
and Minecraft runtime remain NOT_EXECUTED.

**Next:** Research genuine official Store-packaged Education runtime and
manifest support without installing/repackaging games or enabling a
production provider before Roaming/HKCU/WAM proofs.


## Independent engine — never promote observed package metadata to reviewed release origin, 22 October 2026

Fixed a semantic evidence gap in the canonical Rust
`independent_engine/image_provenance.rs` reviewer. Previously a
caller-reported `PackageIdentityEvidence::ProcessPackageMetadataObserved`
could remove `ImageProvenanceGap::PackageOriginNotReviewed` even without
an official signed-package allowlist or origin binding. After introducing
the read-only `GetPackageFullName` package-form observer in the desktop
owner, that interpretation would be misleading. The reviewer now
**always retains the package-origin gap** until an independently reviewed
official-package source contract is implemented; an observed package
name, an unreviewed hash or a generic Microsoft signature cannot supply
it. Existing regression claiming zero gaps was corrected and a
negative test covers both absent and observed package metadata.
Pure Rust tests authored NOT_EXECUTED because a Rust toolchain is not
available in the ChatGPT source-execution environment; Windows target
compiler and Minecraft acceptance also NOT_EXECUTED.

This changes only diagnostic gap semantics, not lifecycle/admission,
package installation, profiles, secrets or launch. No new signer list,
trust policy, installer, CI or `main` modification.

**Next:** Source-review exact image/package provenance when a
legitimately reviewed official package origin is available, alongside
supported Roaming/HKCU/WAM feasibility. MC02–MC04 Launch/Provision remains
blocked.


## Independent engine — execute offline evaluator tests in ChatGPT and repair false-positive privacy assertion, 22 October 2026

Ran the exact committed pure JavaScript AppContainer evaluator and eight
existing test callback bodies inside ChatGPT's V8 isolate, replacing only
Node import/CLI wrappers with an equivalent minimal assertion harness.
Baseline at `b616bf74259f56be54ece1d09d888ab37b83a87a`: 7 PASS,
1 FAIL. The failing test looked for the raw substring `SID` anywhere
in serialized JSON, misclassifying the legitimate public enum value
`OUTSIDE_EXPECTED_PROFILE` as a leaked SID.

Replaced the assertion in the canonical
`tooling/acceptance/evaluate-appcontainer-isolation.test.mjs` with an
exact output-key allowlist, an exact nested child schema/value whitelist,
and checks for explicitly named sensitive fields. Re-ran the full eight
test callback bodies against the unchanged evaluator using the same V8
harness: **8 PASS, 0 FAIL**. The repository's normal Node `node --test`
runner itself was NOT_EXECUTED; neither Windows nor Minecraft tests were
run. This does not upgrade any AppContainer candidate's Roaming/HKCU/WAM
or gameplay readiness.

**Next:** continue source-only independent Windows engine compatibility;
the new package-form source observer still needs a Windows target compile
and OS execution. Keep isolated Launch/Provision blocked and GitHub
Actions fully disabled.


## Independent engine — observe real Windows package form without reading package data, 22 October 2026

Implemented a new read-only, unmounted package-form collector in the canonical
desktop Windows identity owner: `inspect_process_package_form_source`. It
uses the existing PID/FILETIME-pinned process HANDLE and documented
`GetPackageFullName` kernel32 API to distinguish `Packaged` versus
`Unpackaged`. The buffer-size query uses documented
`APPMODEL_ERROR_NO_PACKAGE` (15700) and `ERROR_INSUFFICIENT_BUFFER`
(122) semantics. Unknown errors and unbounded names fail closed; a second
bounded query validates returned UTF-16 termination and discards the value
without logging or exporting package full name. Final process liveness is
rechecked on the SAME handle. Output is a `PendingProcessPackageObservation`
with process key and only a category, never approval to launch or claim a
trusted Microsoft/Minecraft package. Native ABI is scoped inside the
existing Windows owner. No new dependencies, IPC command or launcher.

Added a cross-platform pure classifier regression, invalid PID/FILETIME
regression, non-Windows fallback regression, and Windows-only same-process/
stale-key regression. Source tests authored NOT_EXECUTED. Windows MSVC
compiler, observed installed Minecraft package form, roaming/HKCU/WAM
and game acceptance remain NOT_EXECUTED.

This source enables a real distinction needed for evaluating packaged
Windows capabilities: **package identity is not sufficient proof that an
unmodified Minecraft executable uses MSIX VFS/registry virtualization**.
The official signer allowlist and loaded-image provenance are unchanged.

**Next:** Run source-level pure regressions in an available ChatGPT execution
environment if possible, then review actual package metadata and supported
Windows isolation semantics without assuming that this permits MC02–MC04
Launch/Provision.


## Independent engine — classify AppContainer roaming NO-GO before provider work, 22 October 2026

The canonical offline synthetic fixture reviewer in
`tooling/acceptance/evaluate-appcontainer-isolation.mjs` now returns a
freshly derived `candidateDisposition` and bounded `mechanismBoundaries`.
If a cooperative fixture actually queries FOLDERID_RoamingAppData and
reports it outside its own AppContainer, the candidate is classified as
`NO_GO_OBSERVED_ROAMING_BOUNDARY`. A failed/missing query or apparently
private-looking paths remain `UNVERIFIED_NO_GAME_ACCEPTANCE`. The review
explicitly keeps HKCU registry and Microsoft broker `NOT_OBSERVED`, and
Minecraft game filesystem `NOT_EXECUTED`, regardless of synthetic
private folder canaries or a redirected APPDATA environment variable.
No new readiness state, platform provider, launch permission, or extra
fixture schema field is created. The existing desktop gate stays blocked.

Regression source added for historical outside-roaming, all-private path
reports, missing roaming query, and synthetic private-storage persistence
that must NOT hide the roaming Known Folder incompatibility. JavaScript
tests authored NOT_EXECUTED; no fixture, Windows or game execution.
This is evidence interpretation, not a claim to have fixed redirection.

Confirmed from Microsoft's RegLoadAppKeyW contract: an application hive
cannot be addressed through ordinary HKCU/HKU namespaces. It therefore
cannot transparently redirect unmodified Minecraft's existing HKCU calls.
Microsoft's MSIX filesystem/registry virtualization descriptions apply
to **packaged applications**, not proof of equivalent behavior for the
current unpackaged AppContainer prototype. Neither supplies an approved
Minecraft-specific workaround.

**Next:** choose and source-evaluate an independently supported Windows
mechanism that meets the real Roaming/HKCU/broker requirements before
adding any Minecraft-specific independent Launch/Provision adapter.
Do not turn a NO-GO mechanism into production by extending synthetic tests.


## Independent engine — audit OS AppContainer LocalAppData root by SID, 22 October 2026

The existing unmounted process-bound private-root inspection now obtains
a source-scoped, **non-authorizing** OS-root observation. Using the pinned
target's TokenUser and TokenAppContainerSid, it first rejects thread
impersonation, then compares the target user SID with the current manager
Windows user before calling Windows `GetAppContainerFolderPath`. The API
resolves the caller user's AppContainer folder, not arbitrary target-user
profiles; a user mismatch remains `None` (unknown) and does not open a
root path. Its returned CoTaskMemAlloc UTF-16 path is bounded and released
via `CoTaskMemFree`. For matching Windows users, a preexisting directory
is opened with read-attributes/traverse/control access, final reparse
following disabled, and its volume/file ID compared to the already-pinned
root handle. An exact match also undergoes the existing handle-relative
ancestor diagnostic; mismatch is explicit `Some(false)`. The result is
recorded privately as `os_container_local_app_data_match: Option<bool>`
in `PendingRootSidRelations`. No path, SID or token leaves the native
owner; no provisioning or filesystem mutation is introduced.

The AppContainer binary SID-to-text conversion is strict for
`S-1-15-2-...` and has a new Windows-only negative regression authored
NOT_EXECUTED. This is stronger file-identity evidence for one OS-selected
LocalAppData location, **not** an original profile ownership certificate,
NT effective permission test, Minecraft Roaming Known Folder/HKCU
redirection, Microsoft WAM account separation, or authorization to launch.
The native Windows compiler and real Windows/Minecraft execution remain
NOT_EXECUTED. No changes to CI, `main`, installer, driver or account data.

**Next:** confirm Win32 Userenv/ole32 ABI and folder semantics on an
authorized Windows build; design how production can independently own
and safely use a selected root without mistaking AppContainer LocalAppData
for Minecraft Roaming/HKCU isolation.


## Independent engine — permit valid dot-prefixed NT directory components, 22 October 2026

Removed an overbroad early guard in the new root-relative NtOpenFile
child component validator: a legitimate directory such as `.private`
is now accepted while `.`, `..`, path separators, alternate data stream
syntax, trailing dots/spaces and control characters remain denied.
The already authored disposable Windows synthetic regression now also
creates/opens one freshly owned `.private` directory and the opened-name
parser checks an equivalent dotted component. This prevents a false
negative on valid private-profile directory structures, not an
authorization to access arbitrary installed game folders.

**Proof:** exact source-level regression review; Windows MSVC/native tests
NOT_EXECUTED, Minecraft game acceptance NOT_EXECUTED. No user data, game
file, account, installer, CI, driver or Launch/Provision change.

**Next:** keep the new NT ABI read-only until Windows compiler verification,
and separately establish trusted root provenance and actual filesystem/
registry/Microsoft broker compatibility before any product launch.


## Independent engine — root-relative NT handle ancestor traversal, 22 October 2026

The unmounted native root ancestor checker was changed from reopening each
absolute DOS path prefix to opening **only the local drive root by name**,
then each existing directory component through user-mode
`NtOpenFile` with `OBJECT_ATTRIBUTES.RootDirectory` set to the
previously-opened directory HANDLE. `FILE_OPEN_REPARSE_POINT` is set for
each single child name, and a native `GetFileInformationByHandle` check
rejects files, reparse points, missing file IDs, cross-volume boundaries
and final-file-identity mismatches. All ancestor directory handles are
kept alive without FILE_SHARE_DELETE until the original opened root
identity/name is checked again. The call requests metadata/traverse and
synchronize access only; it cannot create, enumerate, read or write
Minecraft data. The NT ABI structs and FFI are scoped to the existing
Windows desktop identity owner, not a service/driver. An additional
Windows-only disposable fixture test was authored, NOT_EXECUTED.

**Proof:** exact GitHub source review plus Microsoft `NtOpenFile`,
`OBJECT_ATTRIBUTES` and `FILE_OPEN_REPARSE_POINT` documented interfaces.
Windows MSVC compilation and fixture execution are NOT_EXECUTED. There
is no evidence of private root ownership, effective ACL, Roaming/HKCU
redirection, Microsoft broker support or game acceptance. The initial
volume-root DOS name remains an untrusted namespace input; file systems
and future races can still be unsupported. The result is always a
non-authorizing pending inspection. No new UI/Tauri route, profile
mutation, installer, driver, CI or `main` change.

**Next:** Audit native NT FFI ABI against the actual supported Windows
target and design how a trusted profile owner supplies a validated root
HANDLE to the relative walker. Never enable MC02–MC04 Launch/Provision
using this diagnostic.


## Independent engine — scan opened root's Windows ancestor chain, 22 October 2026

Implemented a bounded, read-only `inspect_opened_ancestor_chain` in the
existing desktop-native Windows identity owner. It derives the opened-name
DOS path from an already-open root HANDLE using GetFinalPathNameByHandleW
(FILE_NAME_OPENED), rejects unsupported UNC/device/ambiguous path forms,
and inspects every cumulative directory prefix (maximum 32 components).
Each existing ancestor is reopened only for FILE_READ_ATTRIBUTES with
FILE_FLAG_BACKUP_SEMANTICS and FILE_FLAG_OPEN_REPARSE_POINT. Reparse
points, non-directory handles, cross-volume traversal, invalid file IDs,
and mismatch against the pinned leaf directory HANDLE are rejected.
The root's identity and opened name are checked again after the scan.

The existing unmounted `inspect_directory_for_process` now applies this
check after its process-SID/DACL relation, before final liveness inspection.
It reads no directory contents, accounts or worlds and changes no profile.
An authored Windows pure parser regression covers UNC, traversal and path
ambiguity; the pre-existing disposable-directory regression now also
exercises the ancestor check on supported DOS paths. Both NOT_EXECUTED.

**Boundary:** GetFinalPathNameByHandleW path reconstruction and path-based
reopening have TOCTOU and mount semantics limitations. This is a *pending*
negative-risk diagnostic, not safe relative-handle access, root ownership,
effective ACL proof, game roaming isolation or launch authorization.
Windows MSVC/native tests and Minecraft acceptance remain NOT_EXECUTED.

**Next:** Review Windows ABI and race-resistant root-relative handle
traversal design before any real private profile file operations. Keep
MC02–MC04 Launch/Provision blocked.


## Independent engine — read process TokenUser without profile lookup, 22 October 2026

Extracted `user_sid_from_token` from the existing native Windows
`context_from_token`, retaining exactly one bounded TokenUser query
implementation. The process-bound root/DACL correlation now reuses
the process TOKEN_QUERY handle to read only TokenUser and AppContainer SID,
without calling `GetUserProfileDirectoryW` or querying the unrelated
Windows session/profile path. The original context comparison still
uses this exact helper and preserves its established public booleans.
One Windows-only regression checks that the SID-only and full-context
readers agree; it is authored NOT_EXECUTED.

This is a read-only source efficiency/reliability correction, NOT
proof of root provenance, effective permissions, Minecraft roaming,
registry/HKCU isolation or Microsoft account separation.
No launch, installer, account, driver, service, CI or `main` change.
**Next:** Determine a trusted directory-ancestor ownership boundary;
keep production MC02–MC04 Launch/Provision blocked.


## Independent engine — correlate pinned process SIDs with private root DACL, 22 October 2026

The unmounted native private-root observer now uses the same pinned process
and one read-only TOKEN_QUERY handle to compare the current process user
and AppContainer SIDs with the already-opened directory's OWNER and bounded
basic DACL ACEs. Only three private booleans are recorded: owner matches
process user, owner matches container, and a simple allow ACE names the
container SID. Original directory identity, ACL structural checks and
before/after process identity checks remain unchanged. Raw SIDs, paths,
token handles and ACLs never enter an IPC, log or persistent state.
A Windows-only Rust negative test for foreign SID, deny ACE and zero
mask is authored NOT_EXECUTED.

These source-scoped booleans cannot grant file access, confirm an
original private root, resolve junctions or certify effective Windows
permissions. The routine is not registered to a product command.
MC02–MC04 Launch/Provision remains denied; no localtest, world,
credentials, `main`, CI, service or installer was changed.

**Next:** Source-design handle-bound ancestor traversal for a future
trusted profile owner. Windows compiler/ABI and actual Minecraft
storage/login compatibility remain NOT_EXECUTED.


## Independent engine — process-bound pending root observation, 22 October 2026

Added a Windows-only `inspect_private_root_for_process_source` to the
existing desktop identity owner. The adapter refuses native MC01, zero
PID/FILETIME and a non-AppContainer process before inspecting a directory
HANDLE, then rechecks the full process observation after ACL inspection.
A stale/reused process or changed containment makes the entire result
invalid. The output is strictly `PendingPrivateRootInspection`, carrying
slot, process key and directory file identity privately. It is unmounted,
returns no path, credential, ACL bytes or root access permission and cannot
select/launch/provision an isolated Minecraft session.

Two Rust source regression tests cover MC01/invalid keys, stale PID/FILETIME,
non-AppContainer and category change, authored NOT_EXECUTED. Windows compiler
and game/runtime execution remain NOT_EXECUTED. An observed AppContainer
category DOES NOT prove its SID matches the root's DACL owner or that
the root is the game's Roaming/HKCU view.

**Next:** Authenticate expected container SID and actual directory owner,
with safe ancestor traversal and Windows effective access proof, before
allowing any private profile file operation.


## Independent engine — bounded private directory ACE denylist, 22 October 2026

The existing unmounted Windows directory HANDLE inspector now validates
DACL structure with `IsValidAcl`/`GetAclInformation` and enumerates a
bounded 1–64 entries using `GetAce`. A pure conservative reviewer rejects
any nonzero access-allowed ACE for Everyone, Authenticated Users, Anonymous,
Builtin Users or Builtin Guests, regardless of read/write mask; unsupported
object/callback/conditional ACE formats fail closed. It bounds ACL length
and each ACE size, validates basic SID layout and keeps Windows
GetSecurityInfo/LocalFree ownership in the same scope. Two pure Rust tests
were authored for broad grants, deny ACE, unsupported and malformed ACEs.

A syntactically acceptable DACL is NOT **effective Windows access**. This
filter does not authenticate the expected session owner, group SIDs, inherited
ancestor permissions, network filesystems, Windows privileges or real
Minecraft game filesystem routing. It never returns `Ready`, opens a
Minecraft path, mutates a profile or permits Launch/Provision.

**Proof:** GitHub exact-source static review and Microsoft GetAce /
GetAclInformation / ACE layouts; Rust/Windows compilation and gameplay
NOT_EXECUTED. No CI, main, installer, service or driver change.

**Next:** Identify the trusted native root owner and ancestor traversal
mechanism before any active storage operation; do not treat this conservative
ACE filter as isolation certification.


## Independent engine — fail closed on invalid directory DACL/owner, 22 October 2026

The unmounted Windows private-root inspection now reads OWNER and DACL
information from the **same already-open directory handle** using the
documented `GetSecurityInfo` `SE_FILE_OBJECT` ABI. The returned descriptor
is released via `LocalFree`, including failure paths. A missing or invalid
owner SID, NULL DACL, failed READ_CONTROL query, or changed directory object
identity causes rejection. The routine does not read files or profiles and
does not modify ACLs. The check is deliberately limited: a non-NULL DACL
may still contain broadly permissive ACEs, and ancestor reparse points,
actual root ownership, live Windows token/slot binding, and Minecraft
filesystem visibility remain unverified. It never produces a launch grant.
The existing disposable directory and regular-file tests are unchanged;
one new pure policy regression was authored, NOT_EXECUTED.

**Proof:** GitHub source review and Microsoft GetSecurityInfo / LocalFree
API contract; Windows ABI compilation, Windows tests and Minecraft runtime
NOT_EXECUTED. No service, driver, installer, Tauri route, account or world
mutation, and no GitHub Actions reactivation.

**Next:** Define an independently verified effective-permission and
ancestor traversal assessment for a trusted private-root owner, without
letting an arbitrary directory handle authorize session file access.


## Independent engine — bind pending file plan to exact process key, 22 October 2026

Follow-up hardening on the new private metadata path planner: the pending
result now retains the full PID/creation FILETIME from the already validated
request. Future native adapters must re-pin this identity before acting and
must never treat old plans from a restarted or reused PID as reusable.
Valid-plan regressions assert the carried full identity and explicit owned
String components. This is a source-only correction, NOT a launch grant or
filesystem virtualization.

**Proof:** exact GitHub source review; Rust tests NOT_EXECUTED.
No game data, account files or OS paths were read or changed.
**Next:** implement only a verified owner-bound native storage capability
after supported Windows/private-profile enforcement is established.


## Independent engine — read-only pinned private root handle inspection, 22 October 2026

Added a Windows-only, unmounted `inspect_private_root_handle_source` to the
existing `apps/desktop/src-tauri/src/engine/instance_identity.rs` owner.
It accepts a caller-owned, already-open directory HANDLE; retrieves
volume/file-index identity via GetFileInformationByHandle; refuses regular
files, final-component reparse points and unavailable file indices.
The caller remains responsible for opening with no reparse following and
for actual root ownership, ancestors, ACL, process and token evidence.
This is a bounded observation only, never an OS access capability and NOT
proof of private Minecraft filesystem/HKCU/WAM routing.

Two Windows-only Rust tests authored (regular-file rejection and disposable
new directory identity stability). NOT_EXECUTED; no temp files or profile
changes performed in ChatGPT or GitHub. No Tauri registration, launch,
provision, installer, driver or service changes. Windows compiler and
Minecraft gameplay/isolated storage remain NOT_EXECUTED.

**Next:** Unify the trusted, open-directory capability with an actual native
profile owner only after a proven Windows isolation mechanism exists; do not
promote this observation into storage access or game readiness.


## Independent engine — manager-owned private file access plan, 22 October 2026

Implemented `plan_private_file_access` in the existing canonical Rust
`independent_engine/storage.rs` domain. It accepts only MC02–MC04
private-session Local/Roaming file metadata, exact PID/creation FILETIME
binding, and bounded portable relative path components; it rejects host,
cross-slot, official-install and registry/named-object targets. Windows
reserved device filenames, traversal, backslash/drive/UNC paths, whitespace,
unsupported characters and excessive depth/length fail closed. No user path
is resolved, directory created, file opened or account-bearing data touched.
The return type is explicitly `PendingPrivateFilePlan`, NOT an OS access
capability or evidence of Minecraft roaming isolation. It is for
manager-owned metadata only, not arbitrary Minecraft file names.

Three new Rust unit tests cover valid MC02–MC04 plans, forbidden ownership,
and hostile cross-platform Windows path syntax. Authored NOT_EXECUTED.
No driver/service, IPC, Tauri route, launch provider, CI, installer or
target Windows/Minecraft execution. Launch/Provision remains denied.

**Next:** Define and source-review an independently verified native private
root handle/ACL/reparse validation boundary before any file operation;
keep roaming/HKCU/MSAL actual game compatibility UNKNOWN.


## Independent engine — reuse pinned Windows token per native pair, 22 October 2026

Refactored the existing `instance_identity.rs` read-only Windows pair
inspector: each exact pinned process now opens one `TOKEN_QUERY` handle,
reused to inspect TokenUser, TokenSessionId, profile directory,
TokenIsAppContainer and TokenAppContainerSid. The old stand-alone
comparison endpoints still delegate to the same token-based helpers.
The combined pair reads now require two OpenProcessToken calls instead
of four, with existing process liveness/FILETIME checks and original
sanitized boolean output preserved. No raw SID, token or user path
is serialized, and Microsoft broker isolation is still UNKNOWN.

**Proof:** exact GitHub source/consumer review. Existing Windows
parity/stale-key regressions cover combined-vs-separate results but were
NOT_EXECUTED. No Windows target, Minecraft or account acceptance was run.
No new service, launcher, Tauri command, installer or admission bypass.

**Next:** inspect remaining Windows signer and process context contracts
for target-compiler compatibility; keep independent Launch/Provision
blocked pending actual isolation/account evidence.


## Independent engine — reject invalid process keys before native observation, 22 October 2026

In the canonical pure Rust `independent_engine/inspection.rs` and
`independent_engine/paths.rs` orchestration paths, a zero PID or zero
creation FILETIME now fails immediately with `NativePortError::InvalidIdentity`
before opening a process observer, storage observer or cooperating path
collector. Other route and pair preflight paths already rejected such keys.
Two negative regression tests confirm no process or storage/path port is
called for invalid input. No new adapter, operating-system call, launch
permission, IPC route, state store or account access was introduced.

**Proof:** pinned GitHub source and call-site review. Rust unit tests authored
NOT_EXECUTED; native Windows and Minecraft execution NOT_EXECUTED.
**Next:** Continue source verification of native process observation and
signature lifetime; isolated Launch/Provision remains blocked.


## Independent engine — repair Windows-sys 0.61 FreeLibrary binding, 22 October 2026

Corrected the native WinTrust signer inspector's DLL lifetime boundary:
`windows-sys` 0.61 LibraryLoader exports LoadLibraryExW/GetProcAddress,
but not a FreeLibrary binding in that module. The DLL owner now declares
the exact one-function `kernel32` Win32 ABI locally and releases only a
non-null module handle acquired using LOAD_LIBRARY_SEARCH_SYSTEM32.
The module ownership remains scoped to signer inspection; no new
dependency, runtime service, launcher or installation action was added.

**Proof:** cross-checked against published Windows-sys 0.61.2 API index
and Microsoft FreeLibrary requirements; exact source-only review.
Rust Windows compiler and Minecraft execution NOT_EXECUTED.
No new tests were executed. MC02–MC04 admission remains BLOCKED.

**Next:** validate the complete Windows process-signature collector using
a target Windows compiler when permitted, while continuing source-only
hardening without any false publisher or game readiness claim.


## Independent engine — coalesce native process pair reads, 22 October 2026

Optimized the existing desktop read-only pair inspector to keep one pair of
pinned Windows process HANDLEs for both Windows user/profile/session and
AppContainer SID comparisons. The shared pure comparison helpers prevent
diverging interpretations between the original individual IPC calls and
the combined internal pair observation. The independent pair route now
performs six process-handle opens instead of eight (two before, two inside
the combined comparison, two after), while still checking exact PID/FILETIME
and liveness before and after and returning only sanitized booleans.

One Windows-only parity/stale-key regression is authored NOT_EXECUTED.
This is neither a kernel/process isolator nor a Microsoft broker/account
check, and does not enable Launch/Provision or read game data.
No installer, Tauri command, service, CI or game profile mutation.

**Next:** preserve native pair snapshot invariants and continue examining
viable per-process Windows isolation without claiming app/gameplay proof.


## Independent engine — refuse zero process keys in storage evidence, 22 October 2026

The canonical pure Rust `review_bound_storage_facts` now rejects PID=0
or zero creation FILETIME before any observation review, even when its
input array is empty. A specific InvalidProcessIdentity error prevents
an Unknown storage summary from being attached to an impossible
process. The duplicate-surface bitmap is sized from the authoritative
StorageSurface::ALL instead of a second hardcoded count. One negative
test covers empty and nonempty invalid identity reports.

**Proof:** exact source review, one Rust regression authored NOT_EXECUTED.
No new provider, launcher, registry/network call, account write, runtime
admission, Windows test or Minecraft gameplay.
**Next:** audit future native collectors and their exact-process evidence
before any provider activation.


## Independent engine — harden WinTrust signer lifecycle and chain checks, 22 October 2026

Refined the existing read-only desktop image trust source owner.
The native Windows path now rejects a primary signer with an error or empty
chain, and a leaf certificate with a provider error, before computing its
DER SHA-256. A bounded pure Rust adjudicator requires successful WinTrust
verification, a usable nonzero leaf digest, an allocated state and a
successful release. Even failed WinTrust verification cannot ignore a failed
provider-state close or return alleged signer bytes. Native paths always
close provider state before returning an inspection result.
One negative regression authored NOT_EXECUTED.

No official Education signer allowlist, loaded-PE binding, package attestation,
isolation provider, Tauri command or launch admission is enabled. Windows
compiler/MSVC ABI and Minecraft evidence remain NOT_EXECUTED.

**Next:** Audit exact Windows process/image identity around Trust and
review source-level non-authorizing isolation contracts; keep MC02–MC04
Launch/Provision denied.


## Independent engine — root navigation corrected for source-first development, 22 October 2026

The root README no longer routes new development through the retired
Sandboxie-owned fork or suggests automatic GitHub Actions compilation.
The active architecture is Rust-first independent Windows process and
isolation work. Isolated Launch/Provision stays disabled and localtest is
explicitly deferred by operator instruction. This is navigation-only;
no engine capability was promoted and no CI, installer or account data changed.

**Proof:** Exact GitHub source/authority comparison; source tests NOT_EXECUTED.
**Next:** Continue implementation of independent Windows source boundaries
and compiler-ready tests without re-enabling retired providers.


## Independent engine — token-bound cooperative path inspection, 22 October 2026

The Rust `independent_engine/paths.rs::inspect_cooperative_paths` now
checks the entire process observation again after a cooperating path probe,
not only PID/FILETIME. A changed AppContainer containment category is
rejected even when PID and creation time are unchanged. The existing
source only collects non-authorizing, privacy-minimal path findings.
One Rust regression added; tests authored NOT_EXECUTED.
No additional runtime authority, launcher or Windows mutation.

**Next:** Audit read-only native collectors for verified process lifetime
and ABI correctness before approving any independent provider.


## Independent engine — recheck process lifetime after storage inspection, 22 October 2026

The canonical Rust `independent_engine/inspection.rs::inspect_existing`
now re-observes the exact PID/creation FILETIME and token containment
after the bounded storage collector. Process exit/reuse or token-category
changes fail closed, as they already did in route and pair inspectors.
Two new regression tests cover the positive two-observation path and
the stale/token-change path. This is diagnostic-only; no launcher,
profile mutation, new lifecycle state or Sandboxie dependency was added.

**Proof:** GitHub source/consumer review; Rust tests authored NOT_EXECUTED.
Windows/Rust compilation and Minecraft acceptance NOT_EXECUTED.
**Next:** Continue source-first independent engine implementation and
native compatibility review; never treat storage snapshots as launch proof.


## Independent engine — pinned WinTrust signer certificate fingerprint, 22 October 2026

Updated unmounted Windows desktop image inspection to hash the DER bytes of
the primary signer leaf certificate from the *successful* WinTrust provider
state, before state CLOSE. Wintrust.dll helper exports are resolved from
System32 rather than an application-relative path; the helper library and
provider state are released on all paths, including extraction failures.
Missing trusted signer data fails closed, never promotes a generic signature
to Minecraft official publisher. A nontrusted file reports no signer digest.

The existing source-scoped image provenance reviewer receives that hash as
an audit-only claim. The official release signer allowlist stays EMPTY and
both loaded-PE backing equality and package trust remain UNVERIFIED.
Launch/Provision for MC02–MC04 stays BLOCKED; no accounts, game binaries,
worlds, registry, Tauri routes, CI, driver or installer are touched.
Rust negative regression authored; NOT_EXECUTED. Windows MSVC, Windows-sys
ABI, signed-game and multi-client runtime tests NOT_EXECUTED.

**Next:** Validate Windows Wintrust.dll helper ABI and signer lifetime
on a supported target only when execution is authorized; meanwhile continue
source-first image binding/package identity design without a launch grant.


## Independent engine — wire pinned-image candidate to publisher review, 22 October 2026

Added unmounted `instance_identity.rs::inspect_process_image_provenance_source`,
reusing the pinned Windows image trust observation and the canonical
non-authorizing `review_image_provenance`. The adapter supplies
no signer certificate fingerprint, audited publisher allowlist,
loaded PE-section/file match or package claim; all remain explicit
gaps. Generic signed file TRUSTED does not make
`OfficialGamePublisher` attested or enable Launch/Provision.

Hardened WinVerifyTrust cleanup: when verification succeeds, missing
provider state or failed `WTD_STATEACTION_CLOSE` causes a native
error instead of silently accepting a positive signature result.
Three desktop negative tests authored; Windows/Rust/Minecraft
execution **NOT_EXECUTED**. No package versions, WAM/account data,
kernel/service, installer, Tauri route, CI, `main` or worlds changed.

**Next:** verify exact Windows-sys ABI on Windows target and implement
an audit-only signer leaf certificate SHA-256 read from actual
WinTrust provider state, with lifecycle-safe cleanup. Independently
review legitimate official Minecraft Education signer/package
policy and backing image semantics before ever permitting an
official-image proof.

## Independent engine — signer identity and running image policy, 22 October 2026

Added `engine/runtime-core/src/independent_engine/image_provenance.rs`
to the existing Rust domain: an explicitly non-authorizing,
source-scoped image/signature review. It rejects mismatched process
identity, synthetic signer fingerprints and zero digest, and
requires individually audited Authenticode trust, signer leaf
SHA-256 fingerprint, reviewed official release policy, true
mapped-PE file identity and package provenance. No issuer name or
certificate hash is invented or activated; the allowlist starts
empty. Six Rust regression tests authored **NOT_EXECUTED**.

**Important:** the current Windows adapter still reports
`OfficialGamePublisher::NotAttested`; it does not obtain the
signer fingerprint nor prove the loaded process section is
identical to the inspected disk file. All conclusions remain
`PendingImageProvenance` and Launch/Provision stays blocked.
No account, registry, WAM, world, installer, driver, Tauri route,
`main` or GitHub Actions changes.

**Next:** use reviewed WinTrust provider state to collect a signer
certificate fingerprint from the exact pinned file, then address
loaded-image/file provenance on supported Windows. Do not assume
that signer display name alone establishes official Minecraft
publisher authority or silently trust user-supplied fingerprints.

## Independent engine — bounded WinVerifyTrust image inspection, 22 October 2026

**Source-first native owner:** added unmounted
`instance_identity.rs::inspect_process_image_trust_source`, querying
only a pinned process's on-disk image and using silent, cache-only
`WinVerifyTrust` against the opened read-only file handle.
The code checks file and process identity before/after one trust
assessment and always returns `OfficialGamePublisher::NotAttested`,
even for a trusted generic Authenticode signature. No claim is
made about loaded PE section equivalence, Microsoft/Mojang signer
identity, Store package or catalog signatures.

**Proof:** exact-HEAD GitHub static source/consumer review;
three Rust negative tests authored **NOT_EXECUTED**.
Windows compilation/signature verification, Minecraft executable
publisher verification and licensed game acceptance are NOT_EXECUTED.
Only two `windows-sys` API features were added (WinTrust and
Cryptography); no new package dependency version. The canonical
ProcessObservation official-image flag remains false; launch,
provision, CI, `main`, accounts/worlds and broker caches unchanged.

**Next:** independent certificate signer identity/chain and official
publisher policy review plus process image-to-file binding on the
target Windows edition. Preserve all UNKNOWN until actual native
test. Add no game modifications or speculative providers.

## Independent engine — stateless two/four-client Minecraft acceptance, 22 October 2026

Implemented the pure Rust domain
`independent_engine/acceptance.rs::review_game_acceptance`
using canonical slots and pinned PID/creation FILETIME values.
Two-client assessment requires MC01+MC02; four-client assessment
requires all MC01–MC04. Missing slots and unobserved outcomes stay
visible; duplicate slots, process alias/reuse, invalid identities
and synthetic/Windows-only login claims fail closed. Client-local
publisher signature, licensed sign-in, input/render, reconnect,
world persistence and singleton observations are separate from
joint simultaneous gameplay, account distinctness, cross-client
multiplayer and session survival. Joint outcomes must bind every
required slot's exact process key, preventing old 2-client or
pre-restart evidence from standing in for 4-client gameplay.
Ten Rust tests authored
**NOT_EXECUTED**; only GitHub source/static review.

The API deliberately only returns `PendingGameAcceptance`
regardless of reported pass count. It collects no credential,
account identifier, world content, file paths or login cache,
and neither verifies signatures nor launches the game itself.
No second lifecycle/admission owner, Windows service, driver,
Tauri route, CI, installer, main or profile mutation.

**Next:** source-first add a narrow read-only signed-official-image
verification contract to the existing desktop process identity
owner, keeping actual WinVerifyTrust/native execution deferred.
Then specify real-game manual acceptance order without exposing
accounts, including two-client before four-client continuity.

## Independent engine — singleton and broker pair diagnostics, 22 October 2026

Implemented `inspect_pair_boundaries` plus a read-only
`PairBoundaryObserverPort` in the canonical Rust inspection owner.
Two exact processes are checked before/after a bounded pair reading;
stale PID/FILETIME, token-category changes, cross-wired slots and
inadequate proof sources fail closed. Windows token/AppContainer
metadata or synthetic fixture flags cannot prove absence of an
unmodified Minecraft singleton conflict or WAM/MSAL account leakage.
The only return type is `PendingPairInspection`.

The existing desktop `instance_identity.rs` now composes this with
`WindowsProcessObserver` and `UnknownPairBoundaryObserver`,
deliberately reporting both singleton/broker as Unknown.
Eight core Rust + three desktop tests authored, NOT_EXECUTED;
no compiler/Windows/Minecraft run on this HEAD.

No account token, registry, user profile, game binary, Tauri route,
service, driver, independent lifecycle manager, CI, `main`,
installer or Minecraft world was changed. Launch/Provision
remains blocked.

**Next:** design real-game signed-image and manual WAM/session
continuity acceptance without reading credentials, while keeping
the two-process and four-process requirements distinct.

## Independent engine — pairwise singleton and broker source contract, 22 October 2026

The canonical Rust inspection owner now exposes `inspect_pair_boundaries`
and a narrow `PairBoundaryObserverPort`: two distinct PID/FILETIME keys
are revalidated before and after cooperating observations. Source claims
from Windows metadata or synthetic fixtures cannot assert absence of
Minecraft singleton or Microsoft broker account conflict; positive
results require independently verified real-game runtime observations.
The result is always `PendingPairInspection`, never launch readiness.
This is only a source contract: no registry, token, account or named-object
enumeration and no production provider or Tauri command.
Rust/Windows/Minecraft execution NOT_EXECUTED.

**Next:** add a default all-Unknown desktop port and focused fail-closed
tests, then document real-game compatibility acceptance without secrets.

## Independent engine — prevent duplicate-slot PID and evidence laundering, 22 October 2026

**Source-first hardening:** The existing `ScopedIsolationFindings`
now rejects one Windows PID being bound to more than one MC02–MC04
slot, including the same PID with a different creation FILETIME.
A failed check remains failed in the same ephemeral review, and
a synthetic pass cannot be relabelled native/game proof by a second
write. Three pure Rust negative regression tests authored.

**Native integration:** A new unmounted
`instance_identity.rs::inspect_independent_process_pair_source`
compares two *different* logical slots/processes with the existing
pinned Win32 observer and anonymous Windows user/profile/session +
AppContainer SID comparisons. It revalidates both exact process
identities and token categories after comparison. Identical slots,
reused PIDs and zero keys are refused without native calls.
Two additional source tests authored; no Tauri command, new Windows
service, driver, WAM API, HKCU access, Minecraft launch or profile
mutation introduced. The reported distinction is NOT broker,
registry, singleton or signed-image proof.

**Evidence:** GitHub exact-head static review. Rust native/Linux
compilation, Windows process comparisons, Minecraft account/runtime
and four-client acceptance all **NOT_EXECUTED**. No changes to
`main`, CI, installer, accounts or existing worlds.

**Next:** source-first define safe read-only *singleton and broker
capability gaps* (without exposing token/cache data) and realistic
per-process Windows observations; preserve launch blockade pending
actual Minecraft compatibility evidence.

## Independent engine — scoped Minecraft proof and compatibility gaps, 22 October 2026

**Source-first Rust contract:** the existing non-authoritative
`engine/runtime-core/src/independent_isolation.rs` owner now
distinguishes three new mandatory domains: actual
`FOLDERID_RoamingAppData` routing, actual Windows private registry
view, and stable, separate Microsoft broker account context observed
with the real Minecraft client. Proof contexts must **match exactly**
rather than promoting a higher-named tier to an unrelated OS fact.

**Cross-session correction:** `ScopedIsolationFindings` tracks the
three independent MC02–MC04 slots under separate full-width
PID/creation FILETIME identities. Reused PIDs, stale process records
and host MC01 registration are rejected. New processes require a
fresh ephemeral ledger. One synthetic or even all-positive MC02
review cannot clear MC03/MC04. Five Rust regression tests authored,
**NOT_EXECUTED**; source and GitHub head reviewed. No new Windows
registry APIs, WAM calls, token/cache contents, player sign-in,
service, driver, launcher or desktop readiness store.

**External technical context:** Microsoft Learn `RegLoadAppKeyW`,
`Registry Virtualization`, and `Acquire a token using Web Account
Manager` distinguish explicit app-hive handles, limited UAC
virtualization, and broker-managed identity. These are candidate
mechanisms/background, not proof that Minecraft supports them.

**Next:** source-first design of native per-slot Windows OS
identity/registry capability observations and Minecraft broker
compatibility acceptance (without reading secrets). Do not
promote the synthetic lock test, activate Launch/Provision,
or write host HKCU/roaming.

## Independent engine — two-process application-hive load-lock trial, 22 October 2026

**Source-first extension:** the existing C++ application-hive
fixture accepts explicit `--apphive-peer-fixture` and spawns two
short-lived read-only child processes. Each loads its unlocked
newly owned hive, reads the fixed own marker and attempts to
load the opposite hive while the original parent holds it with
`REG_PROCESS_APPKEY`. Only known access/sharing/busy/lock
refusals count as observations; unexpected success or unrelated
API errors fail the fixture. Both A/B directions are required.
The child requests `KEY_QUERY_VALUE` only; marker writes remain
exclusively in the original fixture parent.
An opt-in PowerShell switch `-VerifyRegistryAppHivePeer` selects
this mode. Exact schema-2 receipt validates
`bidirectionalProcessLoadLocks=OBSERVED` while
`peerProcessDenial`, `hostHKCUIsolation` and
`minecraftAcceptance` remain `NOT_EXECUTED`.

**Evidence ceiling:** all source committed to Local; C++/PowerShell
compilation, native Windows process-hive checks, Minecraft and
four-client acceptance **NOT_EXECUTED**. This proves no registry
ACL/HKCU redirection, Microsoft broker or game isolation. No
additional registry writes outside new disposable hive files;
no account or world access, services, drivers, installer, main
or GitHub Actions changes.

**Next:** review Windows compilation compatibility, strengthen
real AppContainer cross-process evidence without touching host
HKCU, then evaluate whether any documented mechanism can
transparently isolate unmodified Minecraft's registry usage.

## Independent engine — opt-in application-hive fixture source, 22 October 2026

Implemented a bounded optional `--apphive-fixture` mode in the
existing C++ Windows synthetic fixture and
`-VerifyRegistryAppHive` switch in its PowerShell runner.
Two new, uniquely rooted `RegLoadAppKeyW` application-hive
files are created only within the new per-run workspace. Fixed
test DWORD values are written/read through the returned HKEY
handles; flush, close and reopen tests preserve distinct markers.
No host HKCU/HKU access or account/world registry keys are touched
by this authored route. PowerShell validates an exact sanitized
receipt and refuses any claim of independent process denial,
HKCU virtualization, gameplay or launch permission.

**Evidence:** Source committed to GitHub Local; C++/MSVC, PowerShell,
native Windows application-hive I/O, Minecraft sign-in and
four-client acceptance **NOT_EXECUTED** in this environment.
The Rust registry reviewer is unchanged; no fake positive registry
probe is injected. Existing isolated Launch/Provision is blocked.
No driver, service, package, main or GitHub Actions change.

**Next:** add a genuinely independent cooperating synthetic Windows
process check against the disposable application hives (not HKCU);
only then consider peer-denial evidence, and keep ordinary Minecraft
HKCU routing as a separate UNSUPPORTED/UNKNOWN requirement.

## Independent engine — registry feasibility and diagnostic correctness, 21 October 2026

**Rust-only source milestone:** `independent_engine/registry.rs`
defines a strict synthetic private-hive review: denies host HKCU,
unknown/preexisting hive roots, native-slot ownership, cross-slot
registry targets, stale PID/FILETIME and invalid restart evidence.
It distinguishes missing probes, observed own markers, unexpected
peer readability and restart persistence without outputting paths,
values, SIDs, secrets or a production approval. Seven unit tests
authored, **NOT_EXECUTED**.

**Diagnostic correction:** `inspection.rs` now distinguishes
`path_risks` (outside-profile path string) from
`dangerous` (reported shared data access). Both remain unresolved.
No Windows registry API, canary write, account/world/profile mutation,
driver, service, Tauri command, provider activation, `main`, CI or
release was added. Windows and Minecraft acceptance NOT_EXECUTED.

**Documented native candidate:** Microsoft `RegLoadAppKeyW`
application hives are scoped by a returned registry handle rather
than HKCU/HKU paths and may be backed by a new temporary fixture
file. The canonical architecture now specifies opt-in creation,
marker read/write, peer denial, restart persistence and cleanup
**only within disposable synthetic hive files**. Importantly this
does not transparently redirect Minecraft's ordinary HKCU access,
so game compatibility remains UNKNOWN/NO-GO until observed. No
native registry I/O was added or executed by this documentation
update. References: Microsoft Learn `RegLoadAppKeyW` and
application hive semantics.

**Next:** implement an explicit opt-in native `RegLoadAppKeyW`
synthetic-file fixture, with no host HKCU/HKU edits, and confirm
that this does not imply automatic game registry redirection.

## Independent engine — process-bound route diagnostic inspection, 21 October 2026

The canonical Rust inspection owner now exposes `inspect_storage_route`:
it rejects host/peer/unknown/invalid routes without inspecting the OS,
then checks pinned PID/creation FILETIME process identity before and
after one storage observation. Any process exit or change of token
category invalidates the report. The result is strictly diagnostic:
`Denied` or `PendingNativeAcceptance`, carrying required, unresolved
and dangerous boundary findings; even all-positive self-reported
findings cannot authorize an OS operation. The existing desktop
Windows identity owner provides an unmounted native wrapper using
`WindowsProcessObserver` and `UnknownStorageObserver`.

A separate source-only native function queries the pinned process's
legacy `TokenVirtualizationAllowed` and `TokenVirtualizationEnabled`
flags as two sanitized booleans. These legacy UAC values cannot prove
private HKCU registry or AppContainer filesystem namespaces. No
registry hive, game data, account token, or filesystem path is read.
Rust fake-port and platform-negative tests are authored only,
**NOT_EXECUTED** (no Windows/Rust compiler here). No CI, driver,
service, `main`, Tauri route, launch, provision or profile mutation.

**Next:** implement a strictly cooperating synthetic Windows registry
boundary observation protocol and native-owned proof of isolation
without accessing the host's HKCU or treating manager-side views
as the target process's view.

## Independent engine — filesystem/registry routing policy, 21 October 2026

**Source-first implementation:** extended canonical
`engine/runtime-core/src/independent_engine/storage.rs` with typed
`preflight_storage_route`. MC02–MC04 deny cross-slot data, host
account/world/cache targets, unknown owner, unpinned PID/FILETIME,
and mutations of the single official Minecraft installation.
MC01 stays with the existing native route. Valid-looking private
local, roaming, HKCU registry and named-object requests produce
only `NeedsNativeVerification`, each carrying independently required
boundary observations. Roaming environment and Known Folder remain
separate checks. Seven Rust tests cover denial and necessary proofs;
**NOT_EXECUTED** (no Rust toolchain/Windows here). Existing JavaScript,
Windows and Minecraft acceptance not run on this HEAD.

**This is not an isolation provider:** no Windows calls, paths,
registry writes, service/driver/daemon, credential handling, or
authorization to launch. The retired Sandboxie source and account
data remain untouched; no CI, main, release or installer change.

**Next source-only milestone:** implement a Windows-owned, read-only
registry and filesystem boundary inspector returning sanitized
process-bound diagnostics, without assuming host manager path or
registry view equals the game's process view. Preserve all Unknown
until actual native observations exist.

## Independent engine — opt-in inherited fixture handle handoff, 21 October 2026

**Source-first native-to-Rust bridge:** the original C++ fixture parent
optionally duplicates just its owned child process query/sync HANDLE and
exclusive eight-byte record HANDLE into a test-only Rust helper through
`PROC_THREAD_ATTRIBUTE_HANDLE_LIST`. The helper validates PID/creation
FILETIME, AppContainer token, file identity and strict frame phase;
success means *synthetic parent-supplied handles were consistent*.
The existing `engine/runtime-core` binary is gated by Cargo feature
`synthetic-fixture-handoff`; optional Windows PS switch
`-VerifyRustHandoff` builds it under a fresh workspace output.
Rust source tests written, NOT_EXECUTED; Rust/MSVC/Windows/game
acceptance NOT_EXECUTED. Lockfile entries updated for existing
`windows-sys` v0.61 already present; no new framework dependency.

This does not authenticate arbitrary caller-provided PIDs, profiles
or Microsoft accounts, nor does it activate a service, driver,
Tauri route or provider. The production-independent path/storage
inspector and game launch remain unchanged and fail closed.

**Next:** inspect and close any handoff Windows compilation/handle
rights gaps via an actual opted-in Windows fixture run when available;
continue independent filesystem/registry design source-first.
No Minecraft local execution until deferred acceptance is resumed.

## Independent engine — strict Windows fixture file handle identity, 21 October 2026

The existing Windows-only Rust reader
`instance_identity.rs::inspect_unbound_synthetic_fixture_record`
now validates the exact open HANDLE via
`GetFileInformationByHandle` before and after the fixed eight-byte read.
It refuses unexpected file length, directory or reparse-point
attributes, unsupported handles, and file-ID/volume/size/last-write
changes. No path is opened; the original C++ owner still controls
exclusive/non-following file open and fresh synthetic profile creation.
One Windows negative test authored, **NOT_EXECUTED**. Cargo adds only
the existing `windows-sys` crate's FileSystem API feature; no
dependency version change.

**Trust boundary unchanged:** Rust does not receive original
`CreateProcessW`/profile capability and cannot bind a record to
that specific child. Result remains `UnboundFixturePaths`.
No game/account/world operations, Tauri route, driver, service,
CI, `main`, or release changes.

**Next:** implement an explicit test-only handle handoff from the
original C++ fixture parent to a Rust collector, with OS handle
rights/lifetime and fresh-profile provenance; do not treat anonymous
fixture files or paths as game isolation evidence.

## Independent engine — owned synthetic child/profile pairing, 21 October 2026

**Source-only fixture hardening:** the original C++ parent now binds
each self-created `Child` to its fresh `Profile` immediately after
a successful launch and pinned creation FILETIME. Observation and
normal-stop operations consume only the child lease; they cannot
take an unrelated profile argument. The source still verifies exact
AppContainer SID, handle/liveness, file record size and file identity.
No behavior from Rust's standalone reader is promoted to trusted
process-specific evidence.

**Evidence:** exact GitHub source-structure review; MSVC, Rust,
Windows, Minecraft and four-account execution **NOT_EXECUTED**.
This preserves the original standalone test fixture and does not
install services, drivers, add new Tauri routes, or alter game worlds.

**Next:** source-first implement an explicit, bounded capability
handoff design at the native/Rust boundary. It must transmit an
actual owner-validated handle and exact FILETIME; without that
verification the Rust reader must continue returning
`UnboundFixturePaths` and isolated Launch/Provision stays denied.

## Independent engine — synthetic fixture provenance hardening, 21 October 2026

The existing C++ AppContainer fixture parent now pins each created
process handle's FILETIME, and verifies live process, expected AppContainer
SID and creation time before/after a fixed observation read. The file
must be a regular, non-reparse eight-byte record with unchanged volume,
file ID and size. The Rust fixture decoder rejects missing required
own/peer/host/mutex/private canary flags. All source-only tests are
authored, but C++/Rust compilation and Windows/Minecraft execution are
NOT_EXECUTED on this HEAD.

This does NOT make the separate Rust reader an authenticated collector,
nor permit Minecraft launch, sign-in, driver/service activation, profile
migration or account/world access. Existing desktop admission stays
blocked and the independent provider remains disabled.

**Next source milestone:** design a narrow, explicitly synthetic
capability handoff from the original process/profile owner to the Rust
observer; never adopt pre-existing user profiles or trust an unbound file.

## Independent engine — bounded Windows fixture record reader, 21 October 2026

**Source-only Rust-first implementation:** the existing desktop process
identity owner implements an opt-in internal
`inspect_unbound_synthetic_fixture_record` helper. It accepts one already
opened regular file handle (never a profile path), requires exactly eight
bytes, decodes the existing C++ synthetic observation, and pins/rechecks
PID/FILETIME + AppContainer token before and after the read. An authored
Windows negative test rejects non-AppContainer/self process and stale
FILETIME. **No file-to-child provenance has been established**:
even a valid record alongside a live process is `UnboundFixturePaths`,
not a trusted production `ProcessPathSnapshot`. No new Tauri command,
engine provider, service, driver or account/world access. Native Rust and
Windows tests remain NOT_EXECUTED in GitHub.

**Next:** implement verifiable record-handle provenance tied exclusively to
a self-created synthetic fixture child/profile before any bound Rust
path observer is permitted. The existing isolated Launch/Provision
block remains authoritative; local Minecraft acceptance is deferred.

## Independent engine — bounded synthetic fixture wire decoder, 21 October 2026

**Source-only Rust/C++ integration:** `independent_engine/fixture_wire.rs`
now strictly decodes the existing native AppContainer fixture's 8-byte
version-1 observation, including explicit unknown/under/outside path status,
unsupported bits and impossible flag combinations. The fixture C++ adds
a compile-time ABI size assertion, not another protocol or launcher.
Rust negative tests are authored, NOT_EXECUTED here. This decoder does
not accept or manufacture PID/creation FILETIME and cannot authenticate
who wrote an observation file; it cannot be substituted for the existing
`CooperativePathObserverPort`. No account/world data, sandbox runtime,
service, driver, installer, Tauri IPC or `main` changes.

**Next:** implement a bounded, read-only Windows fixture-owned record reader
in the existing pinned process owner, with source/handle provenance checks,
before mapping path observations to `ProcessPathSnapshot`. No local
Minecraft acceptance until the operator resumes it.

## Independent engine — cooperating path identity contract, 21 October 2026

**Source-only continuation:** The `independent_engine/paths.rs` owner now
defines `inspect_cooperative_paths`, consuming exact-process observation
and a bounded cooperating-path port. It rejects missing/stale process
identity before a path read, rejects wrong process key from a probe, and
revalidates identity after the read. The resulting Known Folder and
environment observations remain path-only; A/B synthetic labels do
not authenticate a Windows process or grant launch rights.
Five Rust negative/contract tests authored, **NOT_EXECUTED** in
REMOTE_GITHUB. No native collector, Tauri route, driver, login or
world/profile operation. Rust-first hybrid policy unchanged.

**Next:** source-only design and implement a bounded native cooperative
test-process path transport that can bind to a pinned child handle
without modifying Minecraft or reading account-bearing host roaming.

## Rust-first hybrid decision — 21 October 2026

**APPROVED, architecture-only**: Rust is the primary language for the
independent engine domain, Win32 user-mode adapters and any later justified
user-mode broker. Tauri/Svelte remains the UI; the existing independent C++
AppContainer fixture stays test-only. An independently developed, minimal
C/C++ WDK driver may be considered **only after** proving a concrete kernel
capability gap, with security, signing and rollback review. Do not translate
the retired Sandboxie fork, initiate a mass C++ rewrite or add another
lifecycle authority. Canonical policy:
[Independent Engine Architecture](../docs/system/independent-engine-architecture.md#implementation-language-and-ownership-policy---21-october-2026).

**Implementation effect:** no provider activation, Windows service/driver,
packaging, user account, world or existing runtime data change. This
decision records future source standards, not test evidence. The prior
source-first task remains next: bind future synthetic process-local
path observations to independently verified PID/FILETIME and keep
auth/roaming unknown until target acceptance. Windows/Minecraft
acceptance remains NOT_EXECUTED.

## Independent engine — source-first A/B path collector, 21 October 2026

The existing native AppContainer synthetic fixture reports separate
per-child `processPaths` for Local AppData Known Folder, APPDATA environment,
and Roaming Known Folder. The offline evaluator checks schema-2 child
observations against aggregate flags and does not infer real process identity
or successful data isolation. Negative/source tests authored, NOT_EXECUTED.
No Minecraft, Windows driver, installation, old profile, `main`, or CI
changes. The historical negative-control failure remains schema 1.
**Next:** bind future native path collection to independently pinned
PID/FILETIME without treating synthetic A/B labels as game process identity.

## Independent engine — source-first architecture, 21 October 2026

**Operator decision:** finish modular architecture and source integration on
GitHub branch `Local` before executing Windows/Minecraft local acceptance.
New source-only domain contracts in `engine/runtime-core/src/independent_engine/`:
`identity`, `storage`, `operations`, `ports`. They define exactly four
canonical desktop slot IDs, full-width process identities, nine separate
storage/broker boundaries, presentation versus lifecycle side effects, and
separate native observer/provider interfaces. The default provider explicitly
returns NotImplemented; existing Tauri/admission paths remain unchanged.
The canonical design is [Independent Engine Architecture](../docs/system/independent-engine-architecture.md).
Tests are authored, with Rust compilation and Windows acceptance NOT_EXECUTED.

**Read-only orchestration closure:** `independent_engine/inspection.rs`
takes typed observer ports, rejects stale PID/creation-time identity before
storage probing, and exposes unresolved roaming/authentication findings.
Three negative/source-only Rust tests use fake adapters. This does not
connect to the UI, grant a profile, authenticate, or choose a provider;
Rust/Windows execution remains NOT_EXECUTED.

**Source adapter milestone:** The existing `instance_identity.rs` now
implements the typed `WindowsProcessObserver` using the canonical pin
and FILETIME check. `inspect_independent_source` composes that native
process observation with an explicit all-`Unknown` storage observer.
Signed Minecraft image, HWND, roaming, registry and broker are never
claimed on this path. Windows and non-Windows negative/source tests
are authored; compilation/execution remains NOT_EXECUTED here.
No Tauri command or new provider was registered.

**Current source contract:** Storage inspection now requires the same
full-width process identity as the process observer, preventing a future
adapter from silently relying on slot/PID alone. The existing Windows
pinned-handle observer returns only a sanitized AppContainer token
category, with all filesystem, registry, known-folder and authentication
surfaces still Unknown. Negative tests assert that a stale/missing
process cannot trigger storage inspection. Windows/Rust execution remains
NOT_EXECUTED; no Tauri launch/provider mutation or new service.

**Bound storage evidence source:** `independent_engine/evidence.rs` now
implements typed, exact-process validation for nine storage/broker
surfaces, rejecting wrong slot, changed FILETIME, duplicated surfaces,
synthetic positive claims, and path-as-file-access claims. The domain
storage observation setter is crate-private, so future desktop/native
adapters must use the reviewed conversion function rather than
arbitrarily assign verified fields. Negative Rust tests are authored but
NOT_EXECUTED; no Windows or Minecraft acceptance is claimed.

**Process-local path contract:** `independent_engine/paths.rs` now
maps three sanitized process-local LocalAppData/roaming observations
to explicit `PathOnly`, `PathOutsideProfile` or `Unknown`.
It binds slot and full-width process identity and never upgrades a
path string to private file access. New negative unit tests are
authored, but not executed. The existing native desktop storage
adapter still reports all Unknown and launch remains blocked.

**Next source-only milestone:** specify a native storage evidence adapter
with exact-handle/lifetime binding and explicit known-folder/registry/broker
unknowns; keep account-bearing paths untouched pending local acceptance.

## Independent sandbox engine — ACTIVE source foundation, 21 October 2026

**Operator explicitly resumed implementation in ChatGPT on 21 October 2026.**
Scope: independently authored, modular Minecraft-focused isolation engine with
OpenITStudios-owned source; Sandboxie runtime/fork remains retired. The initial
implementation is the pure evidence classification policy in
`engine/runtime-core/src/independent_isolation.rs`, covering synthetic Windows,
native filesystem/namespace/roaming and two-/four-game acceptance. It does not
activate a provider or create accounts, drivers, processes, profiles or new
stores. Its authored negative/unit tests are source only; source/Windows execution
is NOT_EXECUTED in remote GitHub. No imported Sandboxie source is used by this
module. Prior source/fork trees remain historical, not implementation inputs.

The existing desktop `instance_identity.rs` also adds a read-only comparison
of two pinned process AppContainer token identities (sanitized booleans only).
No login, game launch, Windows service/driver or profile operation was performed.
Windows tests are authored but NOT_EXECUTED in remote GitHub.

**Current bounded implementation:** extended the independent native
AppContainer synthetic fixture to create private roaming-like canaries,
stop both children, relaunch two new children under the same distinct
AppContainer identities, verify persistent own data and denied peer reads,
then clean only the fresh fixture profiles. The PowerShell runner asserts
synthetic-only result fields. Neither Windows Known Folder roaming
redirection nor Education broker behavior is inferred from these checks.
Updated MSVC/Windows execution is **NOT_EXECUTED** in ChatGPT/GitHub.

**Current source review:** `tooling/acceptance/evaluate-appcontainer-isolation.mjs`
is a strict-schema, offline, non-authoritative reviewer for the synthetic
JSON output. It separates privately persistent fixture canaries from
`APPDATA` and `FOLDERID_RoamingAppData` path observations. Even when
both paths look private, the reviewer never claims actual known-folder
file-access virtualization, Microsoft authentication or Minecraft readiness;
`isolatedLaunchAllowed` is always false. Negative/malformed evidence is
rejected; no raw account, path or SID fields are accepted. Node unit tests
are included in `test:native-feasibility`. These are source changes, not
new Windows or Minecraft acceptance.

**Next:** execute this opt-in Windows synthetic fixture on an isolated
Windows target and investigate supported private `RoamingAppData` routing;
if the Win32 path remains host-wide, record NO-GO for the unmodified game
unless a supported independent isolation mechanism is proven. Retain current isolated Launch/
Provision blockade; preserve old accounts/worlds/services. No GitHub Actions,
release, main, VM or certificate changes.

### Historical paused handoff (superseded by explicit resumption)

## Independent engine - PAUSED for operator discussion, 21 October 2026

**PAUSED at operator request.** Last implementation/proof commit:
`d823f68456e64298aedea38d879daabe3be4aa55` (`Local`). No further experiments,
game launches, service changes or implementation until explicitly resumed.
Start the ChatGPT discussion from the
[discussion brief](../docs/system/independent-isolation.md#bahan-diskusi-chatgpt---21-oktober-2026).
Architecture and retirement decisions remain in the existing linked owners there.
[Local source archive](../experiments/history/local-investigation-2026-10-21/README.md)
adds 29 historical snapshots plus sanitized fixture results for discussion only;
no runtime/implementation resumed.
Sandboxie and its fork remain retired for new product sessions; installed
services, old profiles/accounts/worlds and legacy VM data remain preserved.
Two synthetic native AppContainers passed token/namespace/canary/lifetime checks;
negative reused-token control failed as expected; fixture profiles removed.
Local AppData redirected, roaming environment/known-folder paths did NOT.
No independent Minecraft launch, auth or four-client acceptance exists.
**Candidate after discussion, not authorization:** prove private persistent
roaming storage without host access or game modification before game smoke tests.

## Windows-first efficiency — 20 October 2026

**State: DONE for bounded desktop process inspection.** Source:
`bb588902f472ba6413325c12ab522e0a9908a339`. The existing desktop instance owner
now requests only its target PID and basic identity fields, replacing a full
system/process refresh plus redundant refresh. PID/start-time identity and Windows
exclusive handle semantics are preserved; no dependency or release format changes.
Two real-process regressions were added. Full `npm run verify:source`: **657 passed,
0 failed, 4 existing file-symlink skips**; desktop 64 tests. TS6 compatibility,
formatting, Windows compilation and dependency audit also pass with the existing
two cross-platform GTK informational warnings. Those crates are absent from the
explicit Windows Cargo graph. The canonical [report](../docs/validation/dependency-modernization-2026-10-20.md)
records exact evidence and unchanged exceptions. Actual startup/RAM/FPS gains,
installer, gameplay and four-client acceptance remain NOT_EXECUTED. No CI, main,
driver or account operations were performed.

## Dependency modernization and structure cleanup — 20 October 2026

**State: DONE for supported Windows dependency/source maintenance**, with explicit
upstream/runtime exceptions. Source: `972b0e40440452d16ff5ee35678559afa66d84e8`.
The complete `npm run verify:source` passes: **655 passed, 0 failed, 4 existing
file-symlink tests skipped**. Native TS7 and TS6 checks have 0 errors/0 warnings;
both Rust packages pass formatting, checks and tests. The canonical
[report](../docs/validation/dependency-modernization-2026-10-20.md) owns versions,
change inventory, source commits and exact evidence.

Continuation fixed Windows lock-contention error mapping and journal retries
while removing fs2; actual cross-process regressions pass. Real Windows junctions
replace directory fixtures that previously skipped. Installs enforce supported
Node/npm engines. Deprecated atomic calls were migrated without an MSRV increase.
Source batches were pushed and remotely verified.

Npm/Cargo audit: zero vulnerability-class findings; two upstream GTK Linux
informational warnings remain. Four file-symlink execution cases, native installer
release approval and gameplay/four-client proof remain unexecuted/held. No CI,
main, driver, account or gameplay mutation occurred. Maintain dependencies through
existing audit/check commands; do not reopen features or legacy operations merely
to erase these separately scoped exceptions.


**Repository:** `OpenITStudios/M-Minecraft-Emulator`; all product changes on
branch `Local`. Current operator decision: **GitHub Actions CI OFF**.

## Current work: retire CI and preserve safe foreground observation

The operator explicitly revoked approval for GitHub Actions on
20 October 2026. Remove **both** workflow definitions from `Local`:
`.github/workflows/verify.yml` (automatic Full Source / manual Quick /
Full Source / Windows Build) and `.github/workflows/package-smoke.yml`
(manual installer/package testing). The separate registration-only
`main` workflow was removed in
[main commit cb106f9](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/cb106f9013206a73f6172c12fb62ee590a1c6b15).
The canonical `toolchain.json`, root rules, repository verifier and
validation router now reflect the CI-disabled policy. Never restore an
Actions workflow, third-party hosted runner, or scheduled CI without
fresh explicit user approval. Existing `npm run verify:source`, Rust, Svelte
and Windows scripts remain *available* for optional manual execution
when supported and requested. They do not run automatically.

This repository write is not the same as changing the repository-wide
GitHub **Settings → Actions** administrative switch: the connector exposes
source editing but not that settings mutation. To disable GitHub Actions
even against future workflow re-additions or other branches, an
administrator must turn Actions off in the repository settings. No new
Actions workflow definitions remain on current Local/main after this
batch if both deletion commits succeed. Previously started workflow
runs may finish; historical logs remain historical evidence only.

## Continued engineering: fail-closed observer recovery

The existing opt-in Windows foreground event hook is still ONE Rust worker
in `apps/desktop/src-tauri/src/engine/instance_focus_events.rs`, with
unbuffered startup handshake, same-thread unhook and explicit Stop. The
operator-facing Start/Read/Stop state previously had a blind spot: a
failed IPC could cause the UI to offer Start even when hook cleanup was
unverified, and a worker holding the lock could be misreported as stopped.

Changes in this batch:
- A Rust status read returns an error for uncertain worker ownership
  (installing, exiting, unreaped or busy), not a fabricated `enabled=false`.
- The Svelte UI blocks Start after any failed transition or status read,
  offers **Retry Stop/cleanup** and a separate explicit **Read event count**.
- A successful Stop or valid status read restores the appropriate controls.
- Transport checks and negative frontend/source contracts updated.
- No extra service, timers, process mutation, Sandboxie driver action,
  or Minecraft graphics/simulation/account setting change.

## Evidence / remaining blockers

The preceding `Local` commit `b5db7f6c4a026ab17a29edc012427b0d04cb05d9`
[failed desktop Vitest on GitHub](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39292712346):
one assertion sought a cleanup log string without its `MIVUBI` prefix.
The actual backend log was correct. Repository, native/isolation stages
and Rust checks passed on that prior SHA. The assertion has been fixed.

**No CI will run for this new commit**. Static review supports source
coherence only; the new Rust/UI behavior is **NOT_EXECUTED** in Windows.
Source compilation, hook lifecycle on a real desktop, active Client 03/04,
four independent licensed Minecraft Education sessions and 15–20 FPS
external limiting are still unverified or not implemented. Editable owned
v1.18.6 source exists but is not an installed, supported runtime.

**Next source task:** establish positive, provider-owned Client 01 and
03/04 process attribution; do not infer it from process names or generic
Sandboxie query failures. Keep the entire performance system external
to Minecraft and leave FPS mutation disabled until a supported vendor/API
route has been measured on Windows.

Historical work: [planning/history](./history/development-through-2026-10-19.md).
