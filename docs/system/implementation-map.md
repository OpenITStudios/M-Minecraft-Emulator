---
id: document.system.implementation-map
class: DOCUMENT
domain: system
role: GUIDE
authority: CANONICAL
lifecycle: ACTIVE
---

# Implementation Ownership Map

This is a **navigation guide**, not a second implementation registry. The sole current v1.18.6 source/compile/assembly commands live in [Isolation Source Tooling](../../tooling/isolation-source/README.md); current work intent lives only in [planning/development.md](../../planning/development.md). Historical v1.18.5 entries below are for provenance and regression traceability, **not alternate active commands**. For an issue, inspect its current source owner and actual consumers.

## Current direction — 21 October 2026

Sandboxie and the derived fork are retired for new product sessions. The old
inventory below is historical dependency navigation, not authorization to activate
the fork or enable certificates. See [retirement register](legacy-retirement.md).
`apps/desktop/src-tauri/src/engine/instance_catalog.rs` blocks isolated Launch and
Provision in both old routes; existing-session recovery remains available.
**No independent replacement provider exists.** Its first read-only foundation is
`apps/desktop/src-tauri/src/engine/instance_identity.rs` →
`desktop_compare_process_contexts` → `app/bridge/processContext.ts` → optional
Settings diagnostics. It compares pinned process contexts without the retired
provider, secrets, profile-file reads or launch authority. See
[architecture](architecture.md) and [feasibility](independent-isolation.md).

`ScopedIsolationFindings` now rejects assigning a Windows PID to
more than one MC02–MC04 slot, including mismatched creation-time
attempts. Failed proof and tier-specific positive proof cannot be
rewritten within one process-bound review. The existing desktop-native
identity owner also contains an **unmounted read-only**
`inspect_independent_process_pair_source` that compares exact
Windows processes using existing pinned token/SID/profile context
checks and revalidates both live keys. Results are sanitized
diagnostics, not evidence of Minecraft WAM, HKCU, singleton,
world or official executable isolation.

Proof readiness observations are now **scoped separately per
isolated slot (MC02–MC04) and exact PID/creation FILETIME** at
\`engine/runtime-core/src/independent_isolation.rs\`.
The pure diagnostic matrix requires native evidence for the actual
Roaming Known Folder and registry view, and game-runtime evidence
for Microsoft broker account continuity. No synthetic
\`RegLoadAppKeyW\` lock may satisfy those checks. Reused PIDs,
process restarts and a successful result from another slot cannot
become four-client evidence. No second lifecycle owner or
production provider has been introduced.

The optional `-VerifyRegistryAppHivePeer` source route now
tests `REG_PROCESS_APPKEY` **load-lock behavior across two separate
synthetic processes**. One hive is unloaded and readable by the
child while the other is held open by the parent; then the roles
reverse. Only a recognized load refusal counts, and the JSON
receipt continues to mark **real peer registry ACL isolation and
HKCU/Minecraft separation as NOT_EXECUTED**. No host registry keys,
accounts or game worlds are accessed.

An opt-in synthetic application-hive phase has been added to
`tooling/acceptance/windows/appcontainer-probe.cpp`, reachable
through `-VerifyRegistryAppHive` in its existing PowerShell runner.
It creates two new registry hive files under a disposable workspace
and uses only returned `RegLoadAppKeyW` handles to seed/read canaries
and recheck them after close/reload. **Cross-process denial and HKCU
redirection remain NOT_EXECUTED**; no host HKCU/HKU key or Minecraft
data is accessed. The application-hive phase is not a provider.

The documented Windows `RegLoadAppKeyW` facility is the next
**candidate** for an opt-in, temporary application hive experiment:
it returns an `HKEY` rooted in an application hive instead of
writing an ordinary HKCU/HKU path. Its private-handle semantics
do **not** redirect unmodified Minecraft's normal HKCU calls.
See the canonical independent engine architecture for stop
conditions and proof requirements.

A new pure Rust synthetic registry reviewer in
`independent_engine/registry.rs` refuses host HKCU,
unowned/preexisting hives, cross-slot ownership and stale/reused
process identities. It accepts only fixture-hive canary reports and
never approves live registry isolation or Minecraft launch.
The route inspector now separates path-outside-profile
`path_risks` from shared-access `dangerous` findings.
No Windows registry I/O is performed or authorized.

The read-only routing inspector is now
`independent_engine/inspection.rs::inspect_storage_route`. It applies
the canonical route policy before Windows observation, checks exact
process identity/token category before and after storage inspection,
and returns only **Denied** or **PendingNativeAcceptance** with explicit
required/unresolved/dangerous surfaces. The desktop's existing
`instance_identity.rs` exposes an unmounted source-only composition
with the current all-Unknown storage adapter, plus a separate Win32
read-only query for legacy UAC token virtualization flags. Those
flags are NOT proof of HKCU registry isolation or official Minecraft
sign-in, and the isolated Launch/Provision gate stays blocked.

Independent filesystem and registry route **preflight** now lives
in the existing `independent_engine/storage.rs` owner. Pure Rust
`preflight_storage_route` explicitly rejects host/account data,
cross-slot owners, unknown targets, stale process keys, and writes to
the common official installation. MC01 remains under its existing
native owner. Remaining private local/roaming/registry/namespace
requests return **NeedsNativeVerification** with distinct required
OS proof surfaces; no runtime filesystem or registry mapping is
implemented and desktop Launch/Provision remains blocked.

An opt-in, **test-only Windows HANDLE handoff** is implemented by
the existing C++ AppContainer fixture plus
`engine/runtime-core/src/bin/synthetic_fixture_handoff.rs`. The
owner duplicates only child-process QUERY/SYNCHRONIZE and fixed
record-read handles into an explicitly allowlisted Rust helper.
The helper verifies exact PID/FILETIME, live token, file identity and
fixture frame and returns only an exit status. Requires explicit
`-VerifyRustHandoff`; not part of the Tauri app or isolated
Minecraft Launch. Runtime Windows execution still NOT_EXECUTED.

The unbound Windows Rust fixture record reader now checks the
underlying **file HANDLE** using `GetFileInformationByHandle` before
and after its eight-byte read, rejecting invalid file type,
reparse attributes, length and changed file identity. It remains
non-authoritative: only the original C++ fixture owner can establish
the fresh profile + handle provenance, and no cross-language
capability has been issued.

The synthetic C++ fixture now uses one `Child` owner binding:
a freshly created `Profile` is paired with its original process
handle/FILETIME during `launch`. `observe(child)` and `stop(child)`
derive the exact fixture profile from that owner rather than accepting
a second, potentially mismatched profile argument. This is
synthetic-parent ownership only, not a Rust cross-process handoff
or an independent Minecraft runtime provider.

The original native C++ fixture parent now pins self-created process
creation FILETIME and rechecks exact AppContainer SID/liveness around
fixture record reads. Its open record handle must refer to a regular,
non-reparse, exactly eight-byte file; identity and size are rechecked.
The Rust decoder rejects missing core synthetic control flags. This is
fixture-side hardening, not a trusted Rust file-to-child transport.

A bounded *unbound* fixture record reader is now implemented under the
existing desktop process-identity owner:
`apps/desktop/src-tauri/src/engine/instance_identity.rs`.
It takes an already-opened file handle, checks PID/FILETIME and a
Windows AppContainer token before/after one eight-byte read, and returns
`UnboundFixturePaths`. The reader does **not** prove that the file was
written by this process; it is not a production collector, session provider,
profile manager, new Tauri route or game authentication evidence.

The existing C++ test fixture's **eight-byte Observation record** now has
a strict pure-Rust decoder at
`engine/runtime-core/src/independent_engine/fixture_wire.rs`.
It rejects malformed version/length/flags, decodes independent path states,
and requires A/B records to come from the same fixture run phase. The
result is explicitly `UnboundFixturePaths`, **not** a verified
`ProcessPathSnapshot`: there is still no live Rust transport or privileged
process launch. Exact-handle provenance is the next platform boundary.

The cooperating-process path contract is
`independent_engine/paths.rs::inspect_cooperative_paths`, consuming
`CooperativePathObserverPort` and the current `ProcessObserverPort`.
It verifies exact PID/FILETIME before and after a path probe, rejects
mismatched fixture binding and preserves path-only uncertainty. No A/B
fixture label can independently establish process ownership. No native
collector or desktop launch route is registered.

The agreed production language split is **Rust-first hybrid**:
`engine/runtime-core/` and `apps/desktop/` own independent Rust logic and
user-mode Windows APIs; C++ remains for synthetic fixtures, with any
separately authored kernel C/C++ component strictly conditional on
demonstrated need. See [Implementation language and ownership policy](independent-engine-architecture.md#implementation-language-and-ownership-policy---21-october-2026).
No code conversion, service, or driver is authorized by this decision.

The synthetic native AppContainer fixture emits separate path-query outcomes
for each cooperative A/B process (JSON schema 2), with strict offline review.
No raw process identifier or profile path is serialized; A/B labels are
not authoritative PID/FILETIME evidence. Existing game launch remains blocked.

The new `independent_engine/paths.rs` performs source-only, exact
slot/PID/FILETIME review of three process-local path outcomes. It
distinguishes `PathOnly`, `PathOutsideProfile` and `Unknown`
without calling host filesystem APIs or treating path strings as
successful isolation. No production Windows collector is wired to it yet.

Bound native storage evidence mapping is owned by
`engine/runtime-core/src/independent_engine/evidence.rs`. It binds
each fact to exact slot/PID/FILETIME, probe method, and evidence source
before an observation can be recorded. Synthetic positive flags and
path-only data cannot assert actual Minecraft storage or broker separation.
Only pure Rust contracts/tests are added; no Windows I/O or launch path.

The independent `StorageObserverPort` now accepts exact
`ProcessIdentity` along with the slot, rather than slot alone. The
existing pinned Windows process observer also categorizes its actual
AppContainer token without emitting raw SID. Storage remains all-Unknown
and the verified token category grants no launch or game readiness.

The existing desktop identity owner
`apps/desktop/src-tauri/src/engine/instance_identity.rs` now supplies
a read-only `WindowsProcessObserver` and `inspect_independent_source`.
They reuse the pinned PID/FILETIME Windows handle check and leave image,
window and all storage/broker evidence explicitly unknown. These are
unmounted Rust functions, not an alternate Tauri handler or launch path.

A read-only orchestration closure, `independent_engine/inspection.rs`,
joins the process-observer and storage-observer ports. It rejects mismatched
PID/FILETIME observations before inspecting any storage boundary; no Tauri
command, provider admission or persisted status has been created.

The modular, independently authored source architecture is
[Independent Engine Architecture](independent-engine-architecture.md).
The domain contracts live in `engine/runtime-core/src/independent_engine/`
(`identity.rs`, `storage.rs`, `operations.rs`, `ports.rs`). They describe
the four fixed slots, full-width PID/start-time identity, the storage-surface
evidence matrix, effect classification, and abstract Windows adapter ports.
The deliberately disabled provider rejects all actions. Nothing registers
a new desktop launch route, state store, service or driver; old admission
remains fail-closed.

The independently written proof-classification policy lives in
`engine/runtime-core/src/independent_isolation.rs`: pure in-memory findings and
tier-separated evidence gaps for synthetic, native Windows, two-game, and four-game
claims. It is NOT provider readiness, lifecycle persistence, or launch admission.
The existing desktop launch block remains authoritative.

The desktop's existing pinned-process identity owner now also provides a separate
`compare_appcontainer_identities` read-only Rust API. It queries actual Windows
AppContainer tokens and compares SIDs strictly in memory, returning only two
booleans; this is neither a credential comparison nor a provider launch path.
No Tauri/UI command is registered for the new API.

The synthetic AppContainer experiment now also contains a two-process,
two-phase **process-restart persistence** check for private
`synthetic-roaming` canaries. Actual `RoamingAppData` API redirection is
still separately unproven; this is not a game-ready storage provider.
The Windows runner `test-appcontainer-isolation.ps1` validates the
synthetic-only output fields and an explicit negative token-control path.

Offline diagnostics for the synthetic native fixture are owned by
`tooling/acceptance/evaluate-appcontainer-isolation.mjs` and its Node tests.
The reviewer reports only path observations, rejects unexpected private
fields, and cannot authorize a new game launch even if synthetic data passes.
It is not a second lifecycle state store or runtime provider.

The current isolation experiment is a native AppContainer fixture in
`tooling/acceptance/windows/appcontainer-probe.cpp`, explicitly compiled/run by
`test-appcontainer-isolation.ps1`. It launches only itself, validates two native
token namespaces and storage canaries, and leaves game acceptance unexecuted.
Roaming-path redirection remains a material unresolved compatibility question.

## Locate work by problem

**Historical fork source (before retirement on 21 October 2026):** the editable Sandboxie-derived
v1.18.6 implementation is **complete as source** in `engine/isolation-owned/`
(1,627 files, 15 directly modified C/C++ files plus four reviewed project-path
quoting fixes). The unmodified reference is
`engine/isolation-candidates/sandboxie-v1.18.6/`. The active exact-source
verifier is `tooling/isolation-source/verify-owned-v1186.mjs` and the
**build-only** Windows entrypoint is `tooling/isolation-source/build-owned-v1186.ps1`.
The full native build and staging pass on 21 October 2026 at `e2c80ad`:
three Windows stages, 16 unsigned-build binaries, five resources and four licenses.
The resulting 25-file owned package remains NOT_INSTALLABLE. A separate locally
test-signed copy was activated after the operator's reboot/UAC approval;
production signing and four-account gameplay remain unexecuted. See the
[current Windows evidence](../validation/native-pilot-localtest.md).
Standalone C/C++ policy tests compile and pass with MSVC; they are not the
real driver build. Developer driver activation is observed; owned-engine gameplay
acceptance remains unexecuted.
See [current implementation sequence](../../planning/development.md).
The pinned v1.18.5 Gitlink, independent fork scripts, overlays, assemblers
and Windows transaction prototypes listed elsewhere in this guide are
**historical/reference-only** until they are explicitly migrated to v1.18.6;
they must never be chosen automatically as current build or activation code.

| Question / fault | Primary implementation owner | Consumer / companion |
| --- | --- | --- |
| Classic credential fallback, Protected Storage host fallback and compartment bypass | `engine/isolation-owned/Sandboxie/core/dll/cred.c`, `pst.cpp`, `ipstore_impl.cpp` and `core/svc/pstoreserver.cpp` | Exact current delta in `tooling/isolation-source/owned-credential-policy.mjs`, reconstructed by the v1.18.6 verifier; compiled synthetic routine regression and build-only evidence in [native pilot report](../validation/native-pilot-localtest.md). WAM/SSO, game cache and account acceptance remain unproven. |
| Four-tab native composition, shell eligibility and window recovery | `apps/desktop/src/NativeWorkspace.svelte` → `app/bridge/workspaceSurface.ts` → `src-tauri/src/engine/workspace_surface.rs`, gated by existing owned provider | [Workspace contract](../product/unified-workspace.md); retained checkpoints, same-executable recovery helpers, coalesced manager foreground updates in `app_bootstrap.rs`; playable rendering/input and single-taskbar acceptance pending |
| Unified dashboard, saved locations and canonical Minecraft targets | `apps/desktop/src/UnifiedManagerApp.svelte` → `manager_settings.rs` | Existing NativePilot data directory; `engine/isolation-native/owned-runtime/minecraft-targets.json`, shared `education-target.ps1`, existing owned provider; old Compact Manager is unmounted |
| Explicit debug-manager owned engine selection and four-slot operations | `apps/desktop/src-tauri/src/engine/owned_provider.rs` → embedded `tooling/isolation-source/windows/owned-manager.ps1` | Existing catalog/resource lock, typed `ownedProviderApi.ts`, mounted `OwnedManagerPanel.svelte`; one-shot Create-only helper in the same executable; no production activation or service installation |
| Diagnose Minecraft present-interval stutter without adding a game hook or daemon | `tooling/acceptance/analyze-presentmon.mjs` + `tooling/acceptance/analyze-presentmon.test.mjs` | Offline operator-supplied PresentMon CSV, one selected Minecraft PID and swap chain, p50/p95/p99 `MsBetweenPresents` only; output excludes PID/account data, never claims FPS/VRAM/gameplay PASS. See `tooling/acceptance/windows/README.md` |
| On-demand GPU counter hint plus stable CPU/RAM report writing | `tooling/acceptance/windows/collect-native-performance.ps1` (`-GpuSnapshot`) → `tooling/acceptance/evaluate-native-performance.mjs` | Default schema-1 remains; optional schema-2 is skipped after PID identity loss; complete UTF8-no-BOM JSON published through exclusive temporary-file move, historical PowerShell 5.1 BOM read; no FPS/VRAM or gameplay claim, see `tooling/acceptance/windows/README.md` |
| Opt-in Windows foreground event counter (zero Minecraft changes) | `apps/desktop/src-tauri/src/engine/instance_focus_events.rs` → existing `commands/native_clients.rs` | One out-of-context WinEvent hook on a dedicated blocking message thread, manual Start/Read/Stop in Settings Diagnostics, same-thread unhook on exit; count-only, no PID/title storage, no client attribution or FPS cap |
| Avoid overlapping high-cost manager PowerShell, dialog, game launch and package hashing | `apps/desktop/src-tauri/src/commands/native_clients.rs` (`MANAGER_RESOURCE_LOCK`) | Existing Svelte `isResourceOperationBusy` in `app/presentation/compactManager.ts` and `CompactManagerApp.svelte`, on-demand only; game CPU/GPU/FPS quotas not implemented |
| Select a single owned Minecraft virtual slot (plan-only, 02/03/04) | `engine/isolation-native/owned-runtime/minecraft-profiles.mjs` (`planOwnedMinecraftVirtual`) | Same persistent Sandboxie setting sequence, with no driver activation; UI readiness explanation belongs to `app/presentation/compactManager.ts` and existing Rust host probe |
| Inspect candidate Sandboxie section names, including disabled boxes (read-only; no ownership proof) | `tooling/acceptance/windows/collect-virtual-boxes.ps1` → `apps/desktop/src-tauri/src/engine/virtual_inventory.rs` → existing Tauri `desktop_discover_existing_virtuals` | `nativeFeasibilityApi.ts` and mounted `CompactManagerApp.svelte`; inspection UI lives in Settings → Diagnostics, official-pilot section candidates only, no Create/Configure/driver activation or account proof |
| User-facing Virtual Manager and internal isolation readiness | `apps/desktop/src/CompactManagerApp.svelte` + `apps/desktop/src/app/bridge/nativeFeasibilityApi.ts` | One mounted UI: Minecraft 01–04 overview first, technical Sandboxie/owned-engine checks collapsed under Settings → Diagnostics; existing Rust inspector (`apps/desktop/src-tauri/src/engine/owned_isolation.rs`); v1.18.6/25 and v1.18.5/24 metadata agree, Create Virtual remains blocked until a signed owned provider exists |
| Compile actual **v1.18.6 MIVUBI-owned** native core without installation | `engine/isolation-owned/` + `tooling/isolation-source/build-owned-v1186.ps1` | `engine/isolation-native/owned-runtime/build-profile-v1186.json`, `verify-owned-v1186.mjs` and `build-owned-v1186.test.mjs`; three-stage real Windows build passes at `e2c80ad`; not installed |
| Native PID/HWND candidates without accounts, tokens or VM | `apps/desktop/src-tauri/src/engine/native_clients.rs`, `apps/desktop/src/app/bridge/nativeFeasibilityApi.ts` | [Native feasibility](../validation/native-multi-instance-feasibility.md); candidate-only |
| Active desktop and blocked legacy VM routing | `apps/desktop/src/main.ts`, `apps/desktop/src/CompactManagerApp.svelte`, `engine/runtime-core/src/command.rs` | [Legacy Retirement](./legacy-retirement.md); old Tauri handlers not registered |
| Prove two Minecraft Education Desktop sessions can run natively without VMs | `tooling/acceptance/windows/collect-native-feasibility.ps1`, `tooling/acceptance/evaluate-native-feasibility.mjs` | [Native feasibility](../validation/native-multi-instance-feasibility.md); historical alternative, not selected launch architecture |
| Build/package the current owned v1.18.6 engine (unsigned, not installable) | `engine/isolation-native/owned-runtime/build-profile-v1186.json` → `tooling/isolation-source/build-owned-v1186.ps1` → `tooling/isolation-source/assemble-owned-v1186.mjs` | One exact profile, actual compiler exit/16 binary hashes/5 resources/4 licenses; package remains NOT_INSTALLABLE; Windows build passes at `e2c80ad` |
| Inspect the owned v1.18.6 payload in the desktop (read-only) | `apps/desktop/src-tauri/src/engine/owned_isolation.rs` | Existing Settings → Diagnostics candidate inspection accepts owned schema-2 package but never selects or installs the engine |
| Historical private v1.18.5 assembler (superseded) | `tooling/isolation-source/assemble-private-engine.mjs` → `assemble-engine.mjs` | Retained only for regression/reference; not an active source build or package route |
| Apply a coherent private-source build marker and ABI compiler guards across seven pinned upstream owners in a separate Git fork checkout | `engine/isolation-native/fork-overlay/apply-private-source.mjs`, `build-private.ps1` | Private exclusive fork build route, keeps upstream namespace/ABI intact, private build report distinct from unmodified baseline; no install |
| Compile a native Windows executable that can compare an isolated candidate driver/service/DLL ABI to pinned Sandboxie source | `engine/isolation-native/src/abi_probe.cpp`, `engine/isolation-native/CMakeLists.txt` | Pinned upstream `my_version.h`, read-only offline mode, explicit lab-only `SbieApi_GetVersionEx` runtime handshake; never installed or wired into Tauri |
| Diagnose simulated creation steps against partial/unknown current SCM registrations for manual rollback review | `tooling/isolation-source/windows/reconcile-ownership.mjs` via `reconcile-ownership-cli.mjs --rollback-report` | Per-service state, drift and provenance, never grants stop/delete or driver install |
| Reconcile the canonical on-disk journal hash with current driver/service SCM evidence and production Ed25519 bindings | `tooling/isolation-source/windows/reconcile-ownership.mjs` and `reconcile-ownership-cli.mjs` | Read-only cross-check of both pinned image hashes, actual persisted journal bytes and same-host fingerprint; production signed gate sealed and all action flags false |
| Record operator review intent and append read-only SCM observations in the canonical transaction journal | `tooling/isolation-source/execute-disposable.mjs`, `windows/ownership-ledger.mjs`, `windows/review-ownership.mjs` | One canonical journal, no automatic resume or service deletion; chained events aid reconciliation but do not prove ownership |
| Cryptographically correlate short-lived disposable-lab claims and read-only SCM service image evidence without enabling rollback | `tooling/isolation-source/windows/attestation-gate.mjs`, `service-ownership.mjs`, `probe-owned-service-image.ps1` | Ed25519 envelope verification with production key intentionally unconfigured, journal+host+snapshot binding, verified path/hash review; ALWAYS nonauthorizing |
| Compile pinned Windows KmdUtil operation arguments and inspect driver/service signatures without enabling installation | `tooling/isolation-source/windows/transport-commands.mjs` and `kmdutil-transport.ps1` | Validated v1.18.5 install/stop/delete transport descriptors; Apply sealed pending signer/host/ownership acceptance |
| Journal disposable engine provisioning attempts before any future privileged side effect | `tooling/isolation-source/execute-disposable.mjs`, `simulate-disposable.mjs` | Exclusive write-ahead simulation ledger; source hook for future Windows transport, no Tauri integration |
| Read-only signing and service-registration diagnostics for a candidate owned Windows runtime | `tooling/isolation-source/windows/probe-disposable-readiness.ps1` | File signature and collision evidence; virtual-machine detection heuristic NOT host attestation, install always blocked |
| Design a disposable-only driver/service provisioning and rollback transaction before any Windows mutation | `tooling/isolation-source/provision-disposable.mjs`, `plan-disposable.mjs` | Deterministic upstream KmdUtil install/rollback sequence; always `SOURCE_PLAN_ONLY_DRIVER_INSTALL_BLOCKED`; not an active Tauri runtime |
| Check Windows `SbieSvc`/`SbieDrv` registration without PowerShell/WMI startup | `apps/desktop/src-tauri/src/engine/owned_isolation.rs` (native SCM `OpenSCManagerW`/`OpenServiceW`) | Same Tauri/UI host-inspection contract; no service mutation, denies unknown/occupied names; `tooling/isolation-source/windows/probe-host-service-names.ps1` is retained as historical read-only reference only |
| Inspect a staged owned Sandboxie-derived engine candidate without installing or selecting it | `apps/desktop/src-tauri/src/engine/owned_isolation.rs` via `native_clients.rs` | Read-only Tauri command `desktop_inspect_owned_engine_candidate`, typed desktop bridge; staged payload is NOT_INSTALLABLE |
| Request Client 02 launch in one persistent MinecraftClient02 Windows Sandboxie box | `apps/desktop/src-tauri/src/engine/native_clients.rs` → Tauri native command | [Sandboxie Client 02 experiment](../validation/sandboxie-two-client-feasibility.md); explicit, non-destructive launch request only |
| Attribute signed Education image candidates without claiming login or native ownership | `tooling/acceptance/windows/collect-sandboxie-feasibility.ps1` → `apps/desktop/src-tauri/src/engine/native_clients.rs` | Explicit **Check status** (`-ImageProof`) alone invokes the per-PID `SbieApi_QueryProcess` attribution; a failed CLI enumeration can load a trusted vendor DLL in any on-demand inspection. Focus/Place use only a targeted boxed PID Authenticode check, not `QueryProcess`. Launch remains a fast signed-EXE preflight |
| Inspect current Client 02 box membership, credentials and persistent settings, without touching accounts | `tooling/acceptance/windows/collect-sandboxie-feasibility.ps1`, `tooling/acceptance/evaluate-sandboxie-feasibility.mjs` | [Sandboxie feasibility](../validation/sandboxie-two-client-feasibility.md); active collector/evaluator, candidate observations are not proof of login |
| Focus or place Client 02 window using fresh PID/FILETIME/HWND, Sandboxie membership and selected signed Education file evidence | `apps/desktop/src-tauri/src/engine/native_clients.rs` → canonical PowerShell collector with `-BoxedImageProofPid` | Only one requested boxed PID gets Authenticode proof at action time; name-only UI cannot authorize a move, and Win32 may refuse foreground; no legacy VM route |
| Embedded collector execution without oversized Windows command lines | `engine/runtime-core/src/provider/process.rs` bounded stdin runner → `apps/desktop/src-tauri/src/engine/native_clients.rs` | Source is streamed as UTF-8 from the embedded collector into a short-lived PowerShell child. No temporary script, daemon, or schema change; Windows execution still unproven |
| Resource-efficient native/isolated management, no VM or new daemon | `apps/desktop/src-tauri/src/engine/native_clients.rs` and [Architecture](./architecture.md) | `apps/desktop/src/CompactManagerApp.svelte`; one on-demand read-only Sandboxie candidate observation, not independent Client 01 authentication, no governor |
| Prove one disposable Windows guest can boot, sign in and render Minecraft Education on VMware or App Sandbox GPU-PV, with provider-specific graphics evidence | `tooling/acceptance/windows/collect-client-feasibility.ps1`, `tooling/acceptance/evaluate-client-feasibility.mjs` | [Client Feasibility](../validation/client-feasibility.md); no production VM mutation |
| VM status, start/stop/suspend/restart/reprovision and rollback | `engine/runtime-core/src/runtime.rs` | `lifecycle_admission.rs`, provider implementation |
| Lifecycle eligibility / guard decisions | `engine/runtime-core/src/lifecycle_admission.rs` | `runtime.rs`, `client.rs` |
| Legacy VMware client-owned setup wizard (inactive) | `apps/desktop/src/app/setupFlow.ts` and `apps/desktop/src/App.svelte` | Historical `shouldShowClientControls` regression; not mounted in native pilot |
| Legacy VMware first-run setup decisions (inactive) | `engine/runtime-core/src/doctor.rs` | Unmounted `src/app/setupFlow.ts`, `src/app/surfaces/SetupSurface.svelte` |
| Base provenance, immutable finalized image and parity | `engine/runtime-core/src/profile.rs`, `base_preparation.rs` | `engine/guest/windows/prepare-base.ps1`, `finalize-base.ps1` |
| VMware Workstation/Fusion operations and VMX | `engine/runtime-core/src/provider/` | `runtime.rs`, `diagnostics.rs` |
| Explicit disposable App Sandbox pilot (`inspect/start/graceful shutdown/display`) | `engine/runtime-core/src/provider/app_sandbox_pilot.rs` | [GPU-PV Disposable Pilot](../validation/gpu-pv-disposable-pilot.md), CLI only; no production VM controls |
| Identity and guest disk isolation | `engine/runtime-core/src/identity.rs`, `provider/vmx.rs` | `profile.rs`, `engine/guest/windows/` |
| Guest Agent authenticated status/launch | `engine/runtime-core/src/guest.rs`, `src/bin/virtual-guest-agent.rs` | `engine/guest/windows/install-guest-agent.ps1` |
| Native Minecraft detection and executable constraints | `engine/runtime-core/src/minecraft_runtime.rs`, `profile.rs` | `guest.rs`, `doctor.rs` |
| RAM/CPU resource pressure and start admission | `engine/runtime-core/src/resources.rs`, `policy.rs` | `runtime.rs`, `capacity.rs` |
| State schema, durable profile and recovery | `engine/runtime-core/src/schema.rs`, `persistence.rs` | `journal.rs`, `runtime.rs` |
| Update policy, signed-asset manifest, installer integrity | `engine/runtime-core/src/update.rs`, `engine/distribution/` | Tauri package configurations, package smoke |
| Open host-native Windows Minecraft / inspect candidate HWNDs without claiming account or gameplay proof | `apps/desktop/src-tauri/src/engine/native_clients.rs`, native Tauri adapter | Active `apps/desktop/src/app/bridge/nativeFeasibilityApi.ts`; legacy `virtualClientsApi.ts` unmounted |
| Active desktop navigation / transient visual state | `apps/desktop/src/UnifiedManagerApp.svelte`, `apps/desktop/src/app/presentation/compactManager.ts` | `apps/desktop/src/main.ts` mounts Unified Manager; `App.svelte` and `NativeFeasibilityApp.svelte` are unmounted legacy components |
| Typed frontend/backend IPC and contract validation | `apps/desktop/src/app/bridge/` | `apps/desktop/src-tauri/src/commands/` |
| Display DPI bootstrap (active); legacy VMware layout/overlay (inactive) | `apps/desktop/src-tauri/src/engine/dpi.rs`; unmounted `window_arrangement.rs` and `screen_overlay.rs` | Active Client 02 **Move window** is in `native_clients.rs`; legacy layout Tauri commands are unregistered |
| Windows package contents/install lifecycle | `engine/distribution/package-contract.json`, `install-lifecycle-policy.json` | `apps/desktop/src-tauri/tauri*.json`, `scripts/prepare-package-binaries.mjs` |
| Target Windows machine acceptance collector | `tooling/acceptance/windows/` | [Validation](../validation/README.md) |
| Root scripts, tooling pin/metadata and source checks | `package.json`, `toolchain.json`, `tooling/repository/verify.mjs` | `tooling/windows-toolchain/dev.ps1`, `.github/workflows/` |

## Domain boundaries

```text
ACTIVE: Svelte UI → typed bridge → thin Tauri command
→ native_clients.rs → official Windows launcher / Sandboxie Plus
→ bounded process + Sandboxie observation → candidate-only UI

PAUSED: Rust VM runtime → VMware provider / bounded Guest Agent
→ old VM status schema and desktop surfaces (unmounted)
```

UI never decides or stores whether the Base, guest or signed-in session is ready. The runtime does not own Windows window layout presentation. Documentation never replaces either.

## Route by work type

| Work | First navigation |
| --- | --- |
| Change engine behavior | [Runtime Core](./runtime-core.md) → listed Rust owner → source tests |
| Improve desktop | `apps/desktop/README.md` → affected Svelte/Tauri owner |
| Base image or guest Windows concern | [Base Image](../product/base-image.md) → `engine/guest/windows/` |
| Package/release safety | `engine/distribution/README.md` → contract JSON → Tauri consumer |
| Investigate VM/game acceptance | [Acceptance](../validation/acceptance.md) → [Failure Matrix](../validation/four-client-failure-matrix.md) |
| Continue unfinished repo work | `planning/development.md` + current GitHub `Local` HEAD |

## Exact lookup rule

Identify one first wrong owner and only its caller/test dependency closure. Search the entire repository only when a concrete cross-domain reference remains unresolved. Source, machine-readable contracts and actual runtime evidence take precedence over overview text. Keep obsolete implementation names in Git history, **not** duplicated source folders.
