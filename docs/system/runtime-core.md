---
id: document.system.runtime-core
class: DOCUMENT
domain: system
role: CONTRACT
authority: WORKING
lifecycle: HISTORICAL
---

# Virtual Clients Runtime Core — paused VMware implementation

> **Historical VMware/Guest Agent design only.** The VM lifecycle code
> described below is absent from active `Local`. The current runtime core
> contains platform-neutral native-profile contracts and bounded helpers,
> not a VMware or Sandboxie game provider. See [current architecture](architecture.md)
> and [legacy retirement](legacy-retirement.md).

## Product boundary

M-Bedrock Virtual Clients is a local multi-instance environment for Minecraft Education.

Player input and gameplay remain manual.

```text
Native
→ authority

VirtualClients
├── Runtime Profile
├── Lifecycle
├── Resource Pressure
├── Runtime Telemetry
├── Identity Health
└── Guest Compatibility
        ↓
     Provider
        ↓
VMware Workstation / VMware Fusion
```

Rust owns runtime truth.

## Ownership

- `runtime.rs` — observation collection, lifecycle orchestration and mutation serialization.
- `lifecycle_admission.rs` — pure lifecycle eligibility shared by action projection and execution; no provider, filesystem or Guest Agent access.
- `profile.rs` — Native Minecraft detection, Base provenance and version parity.
- `guest.rs` — bounded Guest Agent status and fixed Minecraft-launch protocol. Its existing Windows binary `virtual-guest-agent.rs` now accepts segmented HTTP headers with a 4096-byte and two-second total bound, limits active request handlers to eight, and permits only one interactive Minecraft launch at a time. Concurrent status requests remain independent of launch processing; saturation returns HTTP 503. This does not change the wire schema, token authentication, request ID or supported commands.
- `resources.rs` — host pressure policy.
- `client.rs` — public instance/status contract.
- `doctor.rs` — provider, Base, capacity and parity readiness.
- `provider/` — VMware mechanics and VMX inspection. Existing `persistence.rs` transactional text replacement now owns VMX writes for cloned hardware, Base state, and Guest Agent tokens. VMX value reads and provider/Doctor/Base-preparation presence checks recover an interrupted metadata rename from the existing adjacent backup before treating a VMX as missing. Invalid or unsafe VMX files fail closed in provider operations. This does **not** validate linked disk chains or prove Windows/VMware runtime behavior. VMX key reads now accept mixed casing/spacing, require exact left-hand key matches, and reject ambiguous duplicate values for Base state, identity and Guest Agent token. Intentional hardware policy writes deduplicate the affected keys and replace the cloned Base token in the same VMX write. Provisioning preflights all existing VM states and lineage before cloning missing clients. This prevents *some* inconsistent intermediate states; it does not establish crash-atomic VMware disk-chain recovery.
- `virtual-guest-agent` — guest-side read-only runtime probe.
- `virtual-clients` — thin operator CLI.

No parallel runtime state database exists.

## Version invariant

Native is the version authority.

```text
Native Minecraft
= registered Base Minecraft
= live Virtual Minecraft
```

A mismatch is not a warning-only state for boot operations.

Verified daily boot paths require:

1. Native version detectable;
2. registered Base profile present;
3. Native/Base versions equal;
4. Guest Agent reachable after boot;
5. Guest Agent protocol supports the backend's fixed Minecraft launch request (protocol v2 at this checkpoint; status-only v1 is insufficient);
6. Virtual Minecraft version equals Native.

## Base provenance

The Base has one canonical lifecycle marker in VM configuration:

```text
guestinfo.virtualclients.baseState
REGISTERED → FINALIZING → FINALIZED
```

`register-base` performs live verification while the Base is still reusable, then marks it `REGISTERED`. `finalize-base.ps1 -ConfirmGeneralize` runs Windows Sysprep generalization inside the Base, verifies the marker becomes `FINALIZED`, then shuts Windows down. `provision` and `reprovision` accept only a stopped `FINALIZED` Base. A finalized Base must never be booted for re-registration.

`register-base` records the currently detected Native Minecraft version beside the stopped Base.

`open-base-finalization` is the only app-owned path that intentionally reopens a registered Base. It requires the canonical state to be exactly `REGISTERED`, requires Native/Base parity, starts the VM through the provider, and opens the provider UI. A `FINALIZING` or `FINALIZED` Base is rejected. If provider UI launch fails after start, the backend attempts to stop the Base again.

That record means:

> this Base was intentionally prepared for this Native version.

It is not live guest proof. Live Virtual proof comes from the Guest Agent after boot.

## Runtime states

```text
NOT_PROVISIONED
STOPPED
SUSPENDED
RUNNING
ERROR

Native:
MANUAL
```

`QA_READY` is a checkpoint, not a lifecycle state.

## Resource policy

Each Virtual:

```text
memory ceiling = 4096 MB
vCPU           = 2
```

Runtime telemetry keeps configured memory separate from observed process resident memory.

Host pressure:

```text
NORMAL
PRESSURE
CRITICAL
```

Virtual Clients never silently kills existing running clients because of host pressure.

## Reliability

- OS-level operation lock.
- Transactional linked-clone staging.
- Cleanup of abandoned staging data.
- Immutable Base requirement.
- Native/Base version parity gate.
- Live Virtual/Native version parity gate.
- Guest Agent protocol compatibility gate.
- Unique VM UUID/MAC identity check.
- VM identity and Windows machine-identity proof across all three running Virtual instances.
- Verified identity fingerprints are stored only as client provenance for later QA_READY gating.
- Base Microsoft device-registration preflight rejects Entra/Workplace-joined source images.
- Pressure-aware staggered boot.
- Batch rollback on start failure.
- Rollback failure visibility.
- Graceful-only stop with bounded observation, never automatic hard power-off or implicit resume of a suspended VM.
- Long timeout for clone/suspend/resume/snapshot operations.
- Suspend/fast-resume path.
- Clean QA_READY reset.
- Selected-instance reprovision.

## Account persistence invariant

Account state is guest-owned, not backend-owned.

```text
Base       = no Microsoft/Minecraft user session
Virtual-01 = its own persistent guest/account state
Virtual-02 = its own persistent guest/account state
Virtual-03 = its own persistent guest/account state
```

Virtual Clients never stores Microsoft passwords, access tokens, refresh tokens, or equivalent account secrets in runtime/profile data.

Normal lifecycle operations (`start`, `stop`, `restart`, `suspend`, resume through `start`, and `open`) must not revert a snapshot, recreate the VM, or otherwise replace the selected Virtual's persistent guest disk. Stop is graceful-only (90-second command/observation windows); a suspended VM must be resumed explicitly first. Stop-all and Suspend-all check all target admissions before changing any VM. Restart/Reset guest verification failures preserve the current VM and report observed power state; there is no hidden stop.

`QA_READY` is created only after `verify-identities` has persisted verified VM + Windows identity fingerprints and that Virtual completes one-time account setup. `reset` therefore returns to the configured checkpoint. Microsoft/Minecraft may still invalidate or expire a session independently; that is a real account condition, not backend state.

`reprovision` is destructive to the selected Virtual's guest/account state because it creates a fresh clone from Base. The public runtime contract requires an explicit destructive confirmation, and the CLI requires `--destroy-account-state`. It is recovery/setup, not normal daily lifecycle. **Reprovision remains disabled at both runtime and provider levels** until an account-preserving candidate/rollback mechanism has been independently verified; a confirmation flag is not a bypass.

## Disk policy

Do not automatically shrink or compact Virtual disks while snapshot chains exist.

Safe cleanup:

```text
fully stop Virtual
→ reprovision Virtual --destroy-account-state
→ rebuild from Base
```

`reset` is non-destructive to the intended QA baseline: it restores `QA_READY`, boots the selected Virtual, and returns `RUNNING`.

## Emulator feature gate

Do not expose a feature unless a provider primitive can enforce it measurably.

Not exposed today:

- fake Eco mode;
- live RAM resize;
- automatic kill under pressure;
- automatic snapshot-chain disk shrink;
- unverifiable FPS/GPU throttles.

## Frontend boundary

The user-facing desktop lives at `Development/apps/desktop/`; this runtime core is owned by `Development/engine/runtime-core/` and is consumed through typed Tauri commands. It must not duplicate provider, lifecycle, parity, update, resource, identity, or recovery policy.

`DoctorReport.nextSetupAction` is the sole first-run decision owner. Desktop presentation may classify that action as app-owned, user-guided, client-manager, or blocked for UX purposes, but it must refresh backend truth after every step and must never persist its own setup-completion state.


## Public contract

The crate root is an internal integration surface for the CLI and Tauri adapter; it is not a third-party SDK stability promise. Do not remove root exports based only on repository-search absence—CLI/Tauri/tests and platform-gated code must be checked first. Machine-facing compatibility is defined by the versioned JSON/wire schemas below, not by preserving every Rust `pub use` forever.

The backend has one public contract schema for CLI/application-boundary JSON.

Success:

```json
{"schema":1,"data":{...}}
```

Failure:

```json
{"schema":1,"code":"INVALID_INPUT","message":"...","retryable":false}
```

Internal provider, path, schema-storage, guest-wire and persistence modules are not public API. Frontend code must consume the exported root contract only.


## Backend v1 freeze

Public contract schema `1` is frozen before frontend work.

Allowed before real-machine acceptance:

- correctness fixes backed by a failing test or concrete audit finding;
- security hardening;
- deterministic provider simulation;
- persistence/recovery fixes;
- verifier and documentation corrections.

Not allowed without new evidence:

- new runtime state owners;
- alternate lifecycle entrypoints;
- duplicate terminology/aliases;
- speculative provider features;
- automatic account handling;
- frontend-owned policy;
- compatibility shims for contracts that have no real consumer yet.

A public-contract breaking change must increment `PUBLIC_CONTRACT_SCHEMA`. A runtime-data format breaking change must increment `CURRENT_RUNTIME_SCHEMA`. These are separate version domains.


## Operability surface

`snapshot` is the canonical aggregate read-model for operator/frontend inspection.

`support-bundle` creates a diagnostic artifact from existing backend truth. It is not a state database and cannot influence lifecycle decisions. Bundle content excludes credentials, account identifiers, tokens, world content, and absolute runtime paths.


## Engine policy

Product/runtime limits have one read-only owner in `EnginePolicy`. Provider and frontend code must consume this policy rather than duplicate literals for maximum Virtual count, guest memory ceiling, vCPU count, Guest Agent port, or QA_READY snapshot name. Policy exposure does not make these values user-configurable.


## Health issue contract

`DoctorReport.issues` exposes machine-readable observed conditions with a typed code, severity, and optional Virtual client. Issues are diagnostic facts only; `nextSetupAction` remains the single recovery/setup decision owner. Frontend code must never derive a competing recovery flow by interpreting issue strings.


## Operation journal

Mutation operations write a bounded local history of the last 200 operations. Records contain only timestamp, typed operation, non-sensitive target, outcome, typed error code, and retryability. The journal is diagnostic-only: lifecycle decisions never read it, write failures are best-effort, and no error message, credential, token, account identifier, world content, or filesystem path is stored.


## Lifecycle action projection

`actions` exposes the lifecycle state machine as read-only per-Virtual action availability. Frontend controls must consume this projection instead of recreating Start/Suspend/Stop/Open/Restart/Set-ready/Reset/Reprovision state rules. The projection and mutation execution use the same `lifecycle_admission.rs::evaluate_lifecycle_admission` policy over observed `LifecycleFacts`. Facts are collected once per client for action projection and collected again under the operation lock before execution. They are never persisted as a second lifecycle state.

Admission includes power state, runtime schema, Base/client compatibility, saved VM identity, duplicate UUID/MAC detection, recovery-point availability, verified identity provenance, Base finalization for reprovision, and memory admission where applicable. Stop/Suspend do not depend on Minecraft compatibility; recreation can recover a stale client from a healthy finalized/stopped Base.

`ActionAvailability.reason` is an optional additive explanation. Existing `allowed` and `blocker` fields keep their schema-1 shape and enum values. Absence of a reason remains valid. Frontend code displays the explanation and must not interpret it as policy.

Every frontend command response now requires a payload validator in addition to the public envelope/schema check. Required consumed fields, enums, nullability, complete client/action lists, and the allowed/blocker relationship are checked before entering presentation state. Extra backend fields remain compatible.

STOPPED is displayed as Stopped, Native as Managed externally, and completed setup as Setup complete. These labels do not establish Minecraft gameplay readiness. Returning to the visible Clients page refreshes backend truth; no periodic heavyweight diagnostic polling or new persisted state is introduced.


## Naming and presentation boundary

Machine-facing schema-1 commands, state enums and the `QA_READY` checkpoint identity remain stable. Presentation may translate them into user-facing labels such as Recreate virtual client, Save recovery point, Restore recovery point, Refresh, Setup complete, and Pause/Resume. These labels are presentation only: they do not create aliases, new commands, or persisted state.

## First-time setup boot and command activity

`start-setup <Virtual-01..03>` is the explicit first-boot operation for a fresh
client. It uses the same operation lock, admission policy, provider startup and
rollback owner as daily startup. It requires a readable client profile with both
identity proofs absent, independently unique VM identity, no recovery point, a
finalized stopped compatible Base, current client lineage, and memory admission.
Partial/unreadable provenance is not treated as a fresh profile.

The operation starts/resumes the selected VM and opens its VMware window. It
returns observed power state with unknown Guest Agent/Minecraft/Windows identity
status, without waiting for OOBE or writing identity proof. Missing Guest Agent
during this explicit setup operation does not trigger the daily 90-second
verification rollback. Actual provider start/open failures still use rollback;
a pre-existing running client remains running on UI-open failure.

`verify-identities` remains the only identity-proof writer and still requires
all three running guests to pass live compatibility and uniqueness checks.
Normal Start/Resume/Restart/Reset verification remains unchanged. The desktop
shows first-time setup from optional `actions.startSetup` availability, prevents
fallback to daily Start when that setup action is blocked, and keeps batch daily
Start out of the identity-setup screen. The host's recommendation is displayed
beside the all-three-running requirement; it is not a capacity guarantee.

Command activity has an optional, per-invocation observer and versioned DTO:
schema 2, operation name, phase EXECUTING / SUCCEEDED / FAILED. It reports command dispatch and
serialized result only, not internal VM stages or percentage completion.
The existing CLI entry delegates to the same implementation without an observer.
The Tauri adapter forwards at most two messages through a per-invocation Channel;
channel-send failure cannot fail or retry the operation. The frontend validates
messages, ignores late callbacks after settlement, and keeps busy until normal
result parsing and state reconciliation finish. Invalid telemetry means progress
unavailable, not operation failure. No global event bus or persistent progress
state is introduced. See the [Tauri channel API](https://v2.tauri.app/develop/calling-frontend/).

Source implementation is not build, OOBE, transport-delivery or multiplayer proof.
Detailed internal stages, measured read-performance, and target acceptance remain
pending.

## Readiness evidence consistency

ClientProfile owns saved VM-fingerprint comparison and the stored identity
milestone. Doctor and lifecycle admission use the same helper. Both saved proofs
must be nonempty and the current VM identity must match; unknown/changed VM
identity routes Doctor back to VERIFY_IDENTITIES. This checks current VM
provenance, not live Windows identity: live Guest Agent checks remain required.
An identity mismatch does not unlock first-time setup for an established client
or relax daily startup. Manual VMware inspection may be needed before rechecking
identities; do not silently recreate a client to fix its identity.

UPDATE_STAGED now requires an installer file to exist and its saved version,
platform, SHA-256 and signer metadata to match the validated current manifest.
A missing installer or changed manifest returns UPDATE_AVAILABLE. This is
staging metadata consistency, not a fresh cryptographic verification of bytes at
check time. Stage download still verifies bytes/signature; self-application
remains disabled.
