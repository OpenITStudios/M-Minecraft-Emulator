# Current Development Handoff — M-Minecraft-Emulator

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
