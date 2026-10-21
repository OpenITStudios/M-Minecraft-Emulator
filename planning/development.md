# Current Development Handoff — M-Minecraft-Emulator

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
