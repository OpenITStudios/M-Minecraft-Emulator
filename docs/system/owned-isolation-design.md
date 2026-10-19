---
id: document.system.owned-isolation-design
class: DOCUMENT
domain: system
role: DESIGN
authority: WORKING
lifecycle: HISTORICAL
---

# App-owned isolation design — decision and incremental implementation

> **HISTORICAL v1.18.5 design/inventory snapshot.** The only current
> owned v1.18.6 source/build route is [Isolation Source Tooling](../../tooling/isolation-source/README.md).
> External checkout paths, old fork procedures and pending tests below are
> not current `Local` commands.


**Selected direction (2026-10-14):** Personal educational/noncommercial isolation engine, integrated into Minecraft Instance Manager and eventually usable without installing official Sandboxie-Plus. The first implementation is Sandboxie-derived, not a clean-room substitute. Upstream source and license audit: [pinned inventory](../validation/owned-isolation-upstream-inventory.md).

## Source acquisition now pinned in Git (2026-10-14)

The application repository now contains a Git **submodule pointer** to the authentic upstream Sandboxie v1.18.5 commit at `engine/isolation-upstream/Sandboxie`. The `.gitmodules` URL targets the public upstream repository, and a tested `bootstrap-fork.mjs --local-seed` path creates a **separate full-history checkout** for engine development after initializing the submodule. This removes the need to copy kernel source files into this app repository. The pointer is a real Git tree entry, not an installed runtime; the actual submodule bytes have not been downloaded into the current execution environment.

**Non-claims:** a Git submodule is not a published MIVUBI fork; an independent checkout is not a custom signed driver; a compiled driver is not the same as a working, independently authenticated Minecraft Education session. The active official Sandboxie launch behavior and account-bearing profile remain unchanged.

## First owned runtime payload layout — source only

A new private assembler combines the pinned upstream kernel driver, service, DLL, Win32 companions and required helpers into a coherent isolated directory, preserving original licensing and hashing copied bytes. This is deliberately marked `ARTIFACTS_STAGED_NOT_INSTALLABLE`. No actual Windows artifacts, driver installation, runtime selection or gameplay validation exists in this ChatGPT session; production signing and coexistence remain separate gates.

## Closed payload inventory and read-only path consistency — 2026-10-14

The existing candidate inspector now enumerates its user-selected payload
directory with a bounded 96-entry, six-level traversal, rejecting any
undeclared file, directory, symbolic link, Windows reparse-point redirection
or special filesystem entry. Inventory must match the manifest's hashed
artifact entries plus only `engine-provenance.json` and
`NOT-INSTALLABLE.txt`. Optional PDBs remain supported only at known paths;
the assembler's existing payload format and the active provider are unchanged.

Small manifest (256 KiB) and marker (1 KiB) reads now inspect the **opened**
file handle's metadata, cap actual bytes read and check length/mtime again
after reading. The underlying artifact verification still compares the
hash of opened file bytes to the manifest entry. The UI describes a
**folder snapshot**, not a stable verified engine. Ordinary concurrent file
changes can invalidate a later result; the inspector does not freeze
filesystem contents and must be rerun before any future action.

**Boundaries:** traversal/metadata checks reduce accidental path drift and
certain replacement risks; they are not a complete Windows adversarial
filesystem-race proof. No launch or privileged action consumes this
snapshot, so it cannot independently authorize service installation or
switch the official Client 02 backend. Windows reparse/runtime behavior and
Rust integration tests remain NOT_EXECUTED in the current environment.

## Read-only staged candidate provenance tightening (2026-10-14)

The existing Rust-owned candidate inspector now accepts only the exact
packaging-owner output-to-source path mapping for required binaries, helpers,
resources, three upstream licenses and the pinned optional debug symbol
locations. A copied file with a matching checksum but a swapped or invented
upstream `source` path is rejected. The QSbieAPI LGPL notice is mandatory.
The `NOT-INSTALLABLE.txt` marker must match the canonical message for the
upstream or private exclusive state, not merely exist. Both metadata files
are read with bounded input lengths before allocation. Existing Tauri,
TypeScript and UI continue to report integrity only, never authorization.

**Important limit:** these checks show that staged bytes agree with the
user-selected manifest and expected layout. They do NOT establish
independent signature trust, authenticity of compiled upstream code,
kernel-driver loadability, safety of the selected directory or Minecraft
compatibility. The official Sandboxie Client 02 provider remains active.
Rust and Windows executable tests remain unexecuted in the GitHub-only
source review context.

## Source-first owned candidate inspection (2026-10-14)

The desktop now includes `apps/desktop/src-tauri/src/engine/owned_isolation.rs`, a private, read-only Rust module validating the staged engine payload's complete required x64/Win32 driver, service, DLL, COM helpers, manifests and license files against SHA-256. It exposes `desktop_inspect_owned_engine_candidate` through an isolated Tauri command, strict TypeScript parser, and an explicit read-only candidate-selection control in Settings. The result **always** declares `providerSelected=false`, `driverLoadVerified=false`, `gameplayVerified=false`, and `installPermitted=false`. The official Sandboxie backend remains selected. The candidate inspection prepares the application's native source while Windows compilation/signing and actual activation are validated only later.

Source-first policy: continue implementing provider management and disposable-environment lifecycle source regardless of absent WDK, but retain actual compilation, signing, driver loading, login, concurrency and rollback as separate evidence gates. No work in this source batch is an installed or live custom engine.

## Windows service-name admission source (2026-10-14)

The pinned upstream kernel/service source still defines `SbieDrv`/`SbieSvc` service identities. This is not independent enough to coexist safely with the official installation. The app now includes a read-only Windows SCM namespace probe and a fail-closed capability decision: present names block, unknown states block, an empty namespace also blocks pending signed driver/service, IPC and disposable-host evidence. The result cannot enable the owned provider and cannot infer Microsoft account or gameplay readiness. This is a **source-first activation prerequisite**, not an installed new isolation engine.

## Disposable Windows deployment transaction — source-first (2026-10-14)

The standalone `tooling/isolation-source/provision-disposable.mjs` source now models a bounded provisioning transaction using the **actual upstream KmdUtil** service/driver semantics, including minifilter altitude `86900`. It requires a staged pinned v1.18.5 payload, the read-only host assessment showing both `SbieSvc` and `SbieDrv` absent, and a target declaration excluding account-bearing boxes and the official Sandboxie installation. It emits explicit operations for kernel/service installation, signature checks, readback/ABI acceptance and reverse-order rollback with receipt/path ownership requirements.

**No Windows mutation is possible from this module:** it only constructs a noninstallable transaction plan with `installationPermitted=false`; a self-reported target type is not proof of a disposable VM. No Tauri or UI command invokes it. Partial/ambiguous receipts stop for manual reconciliation rather than automatically stopping/deleting any service. An independently reviewed executor, Windows signing and actual ownership checks remain required before this plan can ever become actionable.

## Disposable Windows execution contract and audit journal (source-first)

`tooling/isolation-source/execute-disposable.mjs` now implements the **transaction sequencer and write-ahead ledger** that a future Windows driver/service transport will use. It records each intent as `UNKNOWN` before calling the supplied simulation adapter; crashes, timeouts, unexpected responses, retries and changed plan fingerprints fail closed. A new exclusive journal directory is created, receipts are committed through file fsync/atomic replacement, and recorded transactions are never automatically deleted. The current transport accepts only an in-memory simulation and cannot invoke KmdUtil, modify the host, or claim successful installation.

The companion `windows/probe-disposable-readiness.ps1` is a **read-only Windows adapter** for service-name observations and Authenticode status of the staged kernel/service/helper files. A detected virtualized model is explicitly a heuristic, not proof of disposal: `disposableEnvironmentVerified=false`, `installationPermitted=false`, and `windowsKernelDriverLoadVerified=false` are always returned. Host trust, identity, binary signing and kernel acceptance need a dedicated privileged executor and independent Windows tests before any actual system mutation. The existing Minecraft 02 provider remains official Sandboxie.

## Windows transport binding — source-only (2026-10-14)

The disposable provision plan can now be compiled into a pinned `KmdUtil` command contract instead of invoking an arbitrary command line. The compiler restricts the payload path, validates the exact upstream step order, pins binary SHA-256 references and retains rollback image-path/receipt requirements. A separate PowerShell probe reads signatures and the `SbieSvc`/`SbieDrv` SCM namespace without changing Windows.

The command transport currently **cannot install, start, stop or delete** a service: no trusted signed disposable-lab attestation certificate is pinned, and service ownership needs independent evidence before any mutation. The transport never becomes active through Tauri, and the official Sandboxie Client 02 launch stays unchanged. This separation enables continued source work without falsely treating self-reported receipts or Authenticode `Valid` alone as trust or safe driver deployment.

## Signed lab evidence and SCM ownership review (source-first)

`tooling/isolation-source/windows/attestation-gate.mjs` validates Ed25519-signed, short-lived lab evidence bound to the exact upstream SHA, transaction ID, nonce digest, host fingerprint, snapshot UUID, plan hash and journal hash. It does not trust public keys supplied inside the signed JSON, and the *production trust key remains blank*: only test fixtures can exercise positive verification using generated ephemeral keys.

`service-ownership.mjs` correlates that signed evidence with the matching Windows SCM service image/path, observed executable hash and a journal receipt that claims prior absence and a matching creation step. `probe-owned-service-image.ps1` performs read-only Windows queries without leaking the raw machine UUID. A positive correlation explicitly still returns `rollbackPermitted=false`, `installationPermitted=false`, and `windowsServiceOwnershipIndependentlyAttested=false`, because JSON/SCM correlation cannot establish safe ownership by itself.

The KmdUtil command compiler is now tightened: self-declared `ownerProofVerified=true` and `signedReceiptAttested=true` cannot generate STOP/DELETE descriptors while there is no pinned lab attestation public key. The PowerShell transport is still sealed in Apply mode, and the app never selects or installs an owned driver.

## Operator review and ownership ledger (source-first, 2026-10-14)

The existing `execute-disposable.mjs` journal now has one durable owner for both provisioning stages and later recovery review. `openTransactionJournal` reopens an existing journal without resuming or replaying any operation; `replace` preserves prior step outcomes, operator intent, and SCM evidence prefix. `ownership-ledger.mjs` records a per-transaction **review-only** operator acknowledgment and at most two read-only SCM image observations (one driver, one service). Each event is linked by a SHA-256 hash chain and restricted to the pinned upstream plan and transaction ID. The journal deliberately avoids raw account identities or private world data.

The event hash chain is **not a MAC or digital signature**: an attacker with write access could recalculate it. This is continuity/audit evidence, NOT independent SCM service ownership or a disposable-machine attestation. The explicit operator acknowledgment is likewise NOT an authorization to install, stop, delete or rollback. `review-ownership.mjs` is a developer CLI disconnected from Tauri, Windows KmdUtil and Minecraft 02. Every returned operation permission remains `false`.

## Reconciliation of persisted journal and live SCM observations (source-first)

`tooling/isolation-source/windows/reconcile-ownership.mjs` now reads the **existing** single transaction journal file, hashes its actual bytes (not user-supplied JSON metadata), verifies the ledger's existing hash chain, and independently compares both `SbieDrv` and `SbieSvc` live read-only SCM observations with the payload paths, expected binary SHA-256 and recorded signatures. It enforces a matching hashed machine fingerprint across both observations; only the correct pinned v1.18.5 plan and simulation journal may be evaluated. Missing/duplicated service observations, altered journal contents, outside-payload image paths and mismatched signed images fail closed.

The production-signed variant binds the transaction ID, actual plan SHA-256, current journal-byte SHA-256, host fingerprint, nonce digest and snapshot ID to `verifyProductionLabAttestation`. Its source-pinned public trust key is still EMPTY: no caller-supplied key can enable it. Even future signature verification would only correlate evidence; `installationPermitted`, `canStopService`, `canDeleteService` and `rollbackPermitted` remain **false**. The read-only CLI does not accept any signer override and does not connect to Tauri, an installer or the official Client 02 launcher.

The evidence chain remains audit continuity, not Windows kernel trust or proof the service was created by MIVUBI. A disposable-host attestation, signed native driver, verified service ownership and safe recovery require actual Windows evidence before privilege can be considered.

## Rollback diagnosis without Windows mutations (source-first, 2026-10-14)

The canonical `reconcile-ownership.mjs` now also classifies the current `SbieDrv` and `SbieSvc` registrations against the recorded creation steps and previous read-only SCM observations in the existing journal. A simulated creation step marked `CONFIRMED` is explicitly **not** treated as proof that a real Windows service was created. Unknown results, partial service registration, changed image paths/hashes and conflicting host fingerprints are detected or rejected for manual reconciliation.

The same CLI accepts `--rollback-report`; it is entirely read-only, preserves the official Sandboxie Client 02 setup, and emits no executable stop/delete commands or rollback authority. This is diagnostic source, not a privileged lifecycle manager. Signed-driver loading, verified ownership and independent engine operation remain blocked until Windows-specific proof exists.

## Native C++ driver/service/DLL ABI handshake source — 2026-10-14

A first **native Win32 executable** now exists under `engine/isolation-native/`. Unlike the earlier JS planning and diagnostic layers, this is C++ source compiled against the actual pinned upstream header `Sandboxie/common/my_version.h` with ABI `MY_ABI_VERSION=0x57230`. Its CMake build refuses an unpinned Git source checkout and builds a standalone x64 Windows diagnostic.

`--offline` inspects the candidate bundle's core file layout with **no** Windows SCM access or upstream DLL load. An explicitly requested, isolated-lab `--abi` checks exact `SbieDrv`/`SbieSvc` service image paths and Authenticode for the three core binaries **before** loading `SbieDll.dll` by full path and invoking `SbieApi_GetVersionEx` to compare the live driver version/ABI. The binary performs no service mutations and is not registered in Tauri. It always reports driver signing policy, installer readiness, backend selection and Minecraft gameplay as unproven.

This is core-runtime integration source, **not** a fork of the upstream driver. The original upstream service/driver/IPC namespace remains unmodified and conflicts with an official Sandboxie installation; a separately compiled, signed, correctly namespaced fork and disposable Windows validation are still required before the app can remove its external dependency.

## First private-exclusive fork core source (2026-10-14)

`engine/isolation-native/fork-overlay/apply-private-source.mjs` produces a real
seven-file source patch for a separately checked-out Sandboxie fork, covering
the common ABI header and direct translation-unit guards in driver API, DLL
API, service driver assist, launcher, SbieIni and KmdUtil. The patch uses
`git apply --check --index` before one Git-index application, preserving
the original upstream ancestry/license notices. Source markers do not change
the original `SbieDrv`, `SbieSvc` or `SbieDll` runtime identifiers,
`MY_ABI_VERSION=0x57230` or minifilter altitude `86900`.

This implements **build consistency and source ownership for a private,
exclusive deployment**; it is not a complete co-installable namespace fork.
The dedicated `build-private.ps1` builds Win32 support, x64 core and x64
kernel from only the seven intentional fork source changes, then hashes
artifacts under a private-build-specific report. The ABI probe CMake may now
consume the patched checkout and reports whether the private marker is built.
No actual signed/loaded driver or independent Minecraft gameplay is proven.
Any future simultaneous coexistence with official Sandboxie requires
separate complete kernel, service, DLL, device, IPC, install and signing
namespace migration, not a partial header rename.

## Private core binaries -> owned desktop inspection (2026-10-14)

The native private-core builder's distinct output report is now consumable
through `tooling/isolation-source/assemble-private-engine.mjs`, which
reuses the **single existing** binary staging owner. Its contract checks the
pinned upstream ancestor and seven source files, validates the private
Win32/x64/kernel build report and rehashes binary file contents.
The resulting payload is labeled `PRIVATE_CORE_STAGED_NOT_INSTALLABLE`;
the manifest retains original upstream attribution and includes the private
fork revision, source marker and `canCoexistWithOfficialSandboxie=false`.

`apps/desktop/src-tauri/src/engine/owned_isolation.rs` now recognizes
this private identity as a **candidate inspection only**, with the full
required runtime closure and checksum checks. Its existing Tauri and typed
frontend return states and official Sandboxie launch remain unchanged:
`providerSelected=false`, `driverLoadVerified=false`,
`installPermitted=false`. No parallel provider registry, driver
installation or migration is added. Windows compiler/kernel signing,
source-of-trust and Minecraft acceptance remain separately unexecuted.

## The two distinct independence goals

1. **Deployment independence (approved target):** our app can install, diagnose, manage and update an appropriately licensed and signed app-owned isolation backend without external SandMan/official Sandboxie installation. This still contains appropriately credited Sandboxie-derived GPL components.
2. **Source independence (NOT approved/NOT planned as first fork):** zero Sandboxie-derived code, independently designed isolation internals. Forking or renaming upstream does not achieve this. A later clean-room project would require its own security engineering and compatibility work.

## Baseline and scope

The active released-to-developer prototype remains `Client 01 → official native Minecraft Education` and `Client 02 → official Sandboxie 1.18.5 / MinecraftClient02`. Operator reports distinct licensed accounts, playable concurrent worlds, multiplayer, and account retention after reopen; boxed Client 02 later crashed idle, crash owner UNKNOWN. Reboot persistence, 30–60 minute complete profile and Client 03–04 acceptance remain NOT_EXECUTED. Never transfer passwords/tokens, modify Minecraft game binaries, copy per-account profiles or silently modify current box settings.

Core premise: **the app owns desired actions and observations, not Microsoft's authentication state or Minecraft gameplay.** Existing backend source remains authority; this design does not implement a new runtime.

## Control plane boundary, without a second state store

~~~text
Svelte CompactManagerApp
    ↓ typed frontend IPC (preserved)
Tauri native client commands (thin)
    ↓
Rust native_clients.rs (current single authority)
    ↓ implemented inspect + explicit launch IsolationBackend boundary (official adapter)
    ├─ OfficialSandboxieAdapter → installed signed official 1.18.5 tools (ACTIVE)
    └─ OwnedIsolationAdapter  → independently built/signed/tested fork (FUTURE)
           ↓ service/driver/DLL matched as one release
    ↓
Persistent per-client profile, Windows process and HWND
~~~

This is an internal implementation boundary, not a new Tauri public schema, network API, daemon or parallel lifecycle manager. One backend selected for an entire local runtime **installation context**, not per-client free switching between incompatible drivers. Windows driver/service namespace, system settings and reused box paths must be proven nonconflicting before any cutover.

### Implemented source checkpoint: Inspection Adapter v0.1

The first **source-only** Rust boundary is implemented in the active desktop owner `apps/desktop/src-tauri/src/engine/native_clients.rs`. It privately defines `IsolationBackend::inspect` and `OfficialSandboxieBackend` as the **sole** active implementation. Existing UI status inspection, fast launch preflight and signed-PID window inspection now route through that interface. The actual sandboxed launch remains the **unchanged** official Sandboxie request path and still performs a fresh backend-admitted inspection before requesting `Start.exe`.

The public Tauri/schema-6 `SandboxieClientObservation` is intentionally preserved. This is still an upstream-specific observation; vendor-neutral models, launch abstraction and the separate owned runtime remain future tasks. No new daemon, driver, user setting, account migration or box mutation is added. A fake inspection adapter exercises timeout/incomplete evidence and a zero-PID refusal; authored tests are **NOT_EXECUTED** until Rust/Windows tooling becomes available. This checkpoint does not establish crash recovery, independent deployment, or new gameplay acceptance.

### Implemented source checkpoint: Launch Dispatcher v0.2

The internal `IsolationBackend` now includes `request_launch(executable, licensed)` alongside `inspect(mode)`. `request_sandbox_client_02` is retained as the existing public Tauri entrypoint, but delegates through the **single active** `OfficialSandboxieBackend`. The official provider's Windows launch body is preserved byte-for-byte: executable and license admission, signed Microsoft product check, box membership, service/tool verification, launch lock, and `Start.exe` timeout/retry gap remain unchanged. A test double records forwarded arguments and asserts exactly one request on a timed-out launch.

This is an internal API boundary, **not** a second isolation engine, switched backend, new service, installed fork, or gameplay evidence. `Ok(())` from a launcher remains a *request acknowledgment only*. Public schema 6, app identity, native Client 01, and existing Client 02 profile are unchanged. The future owned provider cannot be activated until independent source/build/signing/Windows acceptance is demonstrated.

### Future full interface (design-only; not implemented)

~~~rust
trait IsolationBackend {
    fn inspect_installation(&self) -> Result<InstallationObservation, IsolationError>;
    fn inspect_box(&self, box_id: &BoxId) -> Result<BoxObservation, IsolationError>;
    fn observe_processes(&self, box_id: &BoxId) -> Result<ProcessObservation, IsolationError>;
    fn request_launch(&self, request: LaunchRequest) -> Result<LaunchRequestReceipt, IsolationError>;
}
~~~

**Not** an API that declares success or queries Microsoft credentials:
- `InstallationObservation`: provider kind, signed artifact provenance, service/driver version, supported capabilities, completeness.
- `BoxObservation`: box existence, persistence, compatibility-profile provenance, unknown fields kept UNKNOWN, no account content.
- `ProcessObservation`: bounded PID + creation FILETIME + HWND, reliable box attribution, image signer and verified file identity for privileged window actions.
- `LaunchRequestReceipt`: launch request submitted; separate later observation required for process/window; login/gameplay verified by operator independently.
- `IsolationError`: bounded categories `UNSUPPORTED`, `INCOMPLETE_EVIDENCE`, `PRECONDITION_FAILED`, `TIMEOUT_UNKNOWN`, `INTERNAL_FAILURE`; a timeout MUST NOT trigger an automatic second launch.

Provisioning, config edits, driver deployment, stop/kill, recovery and migration are **not added to the initial read/launch interface**. They require explicit approval/confirmation, new safety design and respective tests; no silent destructive methods.

### Capability set rather than confusing product states

First cut capabilities: `inspectInstallation`, `inspectBox`, `processMembership`, `requestLaunch`, `focusVerifiedWindow`, `persistentBox`. Unsupported != failed, and a provider-proclaimed feature != validated Minecraft gameplay. Current public desktop schema/box identifier remain unchanged until a reviewed compatibility migration.

### Persistence and compatibility

- Each client box is independently named/identified and has persistent storage. `MinecraftClient02` is an **existing occupied production-test box**: never reuse, rename, reset, overwrite, import, or point a custom kernel/service at it during experimental fork development.
- Maintain explicit provenance: official box/data versus new fork's disposable box. Reversible settings apply only to newly created disposable profiles until migration is independently reviewed.
- No auto-delete/auto-remove; no shared credentials or token-copy flow; preserve COMPARTMENT/OpenCOM/DropAdminRights compatibility as observed baseline rather than default for arbitrary untrusted applications.
- App close must not kill existing game processes; manager startup is read-only until user requests operations.
- Never infer native Client 01 account or Minecraft readiness from an outside-box PID.

### Fail-closed transition/cutover

1. **Phase A / inventory:** pinned upstream projects, source/licenses/build graph and separation boundary. Documentation only.
2. **Phase B / application contract:** add internal Rust adapter with official implementation first; source-level mocks for unknown/no-PID, PID reuse, multiple candidates, launcher timeouts and box drift. Existing UI unchanged.
3. **Phase C / source fork baseline:** separate upstream-history-preserving repository; compile unchanged pinned x86/x64 required artifacts, reproduce native interfaces, compare hashes and actual behavior; no driver install on account-bearing host.
4. **Phase D / defect attribution:** capture redacted crash metadata during matched native-vs-boxed idle/foreground comparison. Preserve UNKNOWN if not determined. Do not patch kernel hooks by speculation.
5. **Phase E / controlled owned adapter:** in a disposable Windows test environment, demonstrate signed driver install/uninstall, broker sign-in, gameplay, multiplayer, persistence after reopen/reboot, resource profile and failure recovery. Valid artifacts/signatures/privileged service scope required.
6. **Phase F / independent deployment:** review legal obligations, Windows driver signing and separate installer identity; support dry-run preflight and approved rollback without deleting existing profiles. Only then allow official dependency removal from app install requirements.
7. **Phase G / optional scaling:** after two-client stability, extend to 03 then 04 with exact same evidence obligations per instance.

No claimed fixed timeline, percentage complete, or automatic upgrade. Later stages only promote on direct evidence.

### Stop conditions

Stop before modifying the host if: required source/license provenance UNKNOWN, driver/companion hashes mismatched, unsigned or untrusted privileged component, two competing kernel service versions, conflicting box roots, unclear account data ownership, active Minecraft in a profile to be migrated, unresolved driver uninstall recovery, failed/ambiguous process enumeration or Windows capability uncertainty. Never weaken process-signer checks or Microsoft account isolation to pass a test.

### Developer evidence package

Each meaningful implementation batch records exact product SHA, upstream SHA, changed module owner, Windows/compiler/WDK versions where applicable, artifact digests/signatures, exercised tests, results, unknowns, rollback method, and explicit `NOT_EXECUTED` for unavailable layers. REA may analyze **owned build artifacts only if helpful**; it is a developer tool and not a runtime dependency.

**Next bounded task:** determine the upstream-history-preserving fork repository and reproducible unchanged build plan from pinned Sandboxie v1.18.5; include provenance and artifact-hash manifest before any driver changes. The installed official pilot and account-bearing profiles remain untouched.
