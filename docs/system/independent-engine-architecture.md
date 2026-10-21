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
