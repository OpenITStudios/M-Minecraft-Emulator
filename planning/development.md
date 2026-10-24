# Current Development Handoff — M-Minecraft-Emulator

**Authority:** `Local` is the only active development branch. This is the **single current goal, blocker and next action**, not a chronological TODO file. The previous 3,009-line ledger with 105 historical `Next:` entries is [preserved at commit `d239a299e04c`](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/d239a299e04c23d2324b52325714600c00c34498/planning/development.md). All its former next actions are superseded; Git history preserves implementation decisions.

## Goal — playable four-client Minecraft Education engine

**Four simultaneous unmodified official Minecraft Education Windows Desktop clients MC01–MC04 on one PC**, each with a different legitimately licensed Microsoft Education account, separate persistent worlds, effective filesystem/HKCU/auth contexts, responsive GPU/input/window controls, required multiplayer and safe recovery. Users sign in normally in the game. No game patch, Microsoft credential or token copying, bypass of licensing, or implicit deletion/migration of existing account/world data.

**Near-term delivery = MC01 + independent MC02**, not four window tabs. Two real independently signed-in games must play concurrently, keep separate world writes across normal close/relaunch and avoid host/peer registry or authentication crossover. Scale the same verified provider to MC03/MC04 only after this is proven. “Lighter than Sandboxie” requires comparable CPU/RAM/GPU/frametime/soak measurements; it is not yet a fact.

## Exact source-state baseline (`d239a299e04c`)

| Capability | Status |
| --- | --- |
| Official MC01 registered-game detection, launch request, native window presentation and Windows-window fallback | Implemented source; recent Windows/gameplay **NOT_EXECUTED** |
| PID/FILETIME, Windows user/profile and `Minecraft.AdalServer.exe` candidate observations | Read-only, NOT game authentication/isolation evidence |
| PSEC capability query; original native Job/file/hive and synthetic COW/broker primitives | Diagnostics/fixtures, NOT a production Minecraft process/data provider |
| Independent MC02–MC04 launch with per-client Roaming, HKCU and Microsoft broker | **NOT_IMPLEMENTED**; existing Launch/Provision admission stays closed |
| Native preview installer | Source-defined only; Windows compile/install/gameplay **NOT_EXECUTED** |
| Four-client licensed gameplay and measured overhead relative to Sandboxie | **NOT_VERIFIED** |

The historical official Sandboxie 2-client pilot later encountered a crash; it is not independent-engine acceptance. Official Sandboxie, derived fork and VMware are **retired new-launch providers**, preserved for old session recovery. PSEC/MXC is only a Windows containment candidate: a file access denial or API export does not redirect Minecraft’s Roaming path, virtualize HKCU, isolate WAM or prove game concurrency.

## P0a source-feasibility decision — 23 October 2026

**Reviewed repository source:** `829e577980a1b5bd92aee17a6f93a151303ed9f7` (`Local`).
**Reviewed Microsoft MXC source:** [`ffecec04357747d6649714f473436dfb77111be8`](https://github.com/microsoft/mxc/tree/ffecec04357747d6649714f473436dfb77111be8).
**Decision: NO-GO for PSEC/MXC ProcessContainer alone as the complete MC02 provider under the current unchanged-game requirements.** This is a source-feasibility rejection of this concrete route, not an observed Minecraft failure or proof that every independent Windows route is impossible. Do not spend another batch adding generic PSEC/fixture layers or unlock isolated Launch.

### Existing owner and consumer trace

- [Independent provider port](../engine/runtime-core/src/independent_engine/ports.rs): `IndependentProviderPort` is a contract; `DisabledIndependentProvider` returns `NotImplemented`. It is not a Windows launcher.
- [Windows adapter](../apps/desktop/src-tauri/src/engine/instance_identity.rs): `WindowsProcessObserver` pins process identity; `UnknownStorageObserver` returns unknown storage observations. `inspect_native_psec_capabilities` loads System32 exports and reads version/support flags only; it neither creates nor attaches a security environment.
- [Desktop command consumer](../apps/desktop/src-tauri/src/commands/native_clients.rs): the PSEC command returns diagnostics through the manager operation lock. Slot operations call the existing [admission owner](../apps/desktop/src-tauri/src/engine/instance_catalog.rs) before a Windows helper; isolated Launch/Provision remains denied in both normal and owned-test routes.
- [Native game adapter](../apps/desktop/src-tauri/src/engine/native_clients.rs): MC01 uses the registered official game route. Auth-family inspection reports candidate helper relationships only; broker-launched helpers may be unattributed. It supplies no private authentication context.

### Complete MC02 requirements against the pinned candidate

| Required boundary | Mechanism found / first gap | Verdict |
| --- | --- | --- |
| Roaming environment AND effective Known Folder | MXC can supply an environment block, but its PSEC specification has access lists, not source-to-private-path mappings. No reviewed mechanism makes both the unchanged game's Roaming Known Folder and absolute-path accesses resolve to a persistent MC02 store. Environment substitution is insufficient. [M1][M2][W1] | **First blocking gap; NO-GO for this route** |
| Local data and persistence | Upstream documents that native PSEC can replace `LOCALAPPDATA` with a container-profile path. That does not prove the game's actual Known Folder/file view, stable profile reuse after relaunch, private world writes or host/peer access boundaries. [M2] | **Partial OS behavior documented; game persistence UNKNOWN** |
| HKCU reads/writes | The reviewed PSEC schema and MXC request builder contain no selectable per-game registry hive/remap. Capability grants are access permission, not evidence of a private HKCU. Our synthetic hive/COW code is not attached to Minecraft. Calling `RegOverridePredefKey` in the launcher affects that caller only. [M1][M3][W2] | **Missing mediation; NO-GO for this route** |
| Microsoft helper / authentication broker | No reviewed configuration binds each game and its actual helper/broker to a private persistent authentication context. WAM documentation describes Windows-account integration, not a per-PSEC broker switch; it does not establish which broker this Education build uses. A helper name, parent PID or different container SID is insufficient. [W3] | **Actual Education auth path and separation UNKNOWN** |
| Singleton and IPC | Containment and UI handle/atom restrictions exist, but the game's actual single-instance mechanism and any external helper activation are unidentified. No game-bound two-process lifetime evidence exists. [M1][M4] | **UNKNOWN** |
| GUI, GPU and input | ProcessContainer supports retaining Win32k/GUI access when `ui.disable=false`; job UI limits remain separate restrictions. This is not proof of Minecraft rendering, D3D resources, keyboard/mouse, focus/fullscreen or cross-job broker dialogs. [M3][M4] | **GUI policy exists; game compatibility NOT_EXECUTED** |
| Required multiplayer | MXC has egress/ingress/loopback policy, but no Minecraft host/join/reconnect acceptance. Opening network access does not isolate auth or establish multiplayer compatibility. [M1][M2] | **NOT_EXECUTED** |
| Two licensed playable clients and safe restart | No independent game provider or concurrent manual sign-in/world-persistence proof exists. | **NOT_IMPLEMENTED / NOT_EXECUTED** |

**Why this closes the candidate:** denying the existing Roaming/HKCU path can break a game; granting the host path can expose shared state. Neither action supplies the missing private mapping. No documented/configurable mediation was identified in this reviewed route. Missing schema fields do not rule out all implicit OS behavior; effective Minecraft behavior remains UNKNOWN. AppContainer does have real built-in profile, credential and object isolation; those properties do not by themselves establish this game's Roaming/HKCU/broker compatibility. Host/peer access protection also remains unverified: container isolation is not a claim of secrecy from the owning Windows user or an administrator. This conclusion is an inference from the pinned schema, request builder and API scope, not a runtime verdict. Adding broad capabilities, using audit mode that permits denied access, changing profile environment variables, or treating synthetic COW/broker fixtures as game integration cannot satisfy these gaps.

MXC **IsolationSession is a different backend**, not an implicit fix: its [pinned supported-workloads guide](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/docs/development/architecture/backends/isolation-session/oneshot.md#supported-workloads) excludes GUI applications because the session has no display server. Keep its existing NO-GO for the current native interactive workspace. Do not infer the same GUI restriction for ProcessContainer. MSIX repackaging, RunAs impersonation, game hooks and driver/service installation are not introduced.

### Primary sources and evidence ceiling

- **[M1]** [Windows PSEC FlatBuffers schema](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/external/windows-sdk/ProcessSecurityEnvironment.fbs): filesystem access lists, capabilities, UI and network policy; no explicit path/hive remapping fields.
- **[M2]** [MXC schema: environment and filesystem policy](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/docs/schema.md#environment): environment semantics, native Local AppData caveat and access-grant scope.
- **[M3]** [PSEC request builder](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/src/mxc-sdk/src/backends/process_container/common/base_container_helpers.rs) and [Windows runner](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/src/mxc-sdk/src/backends/process_container/common/base_container_runner.rs): policy serialization and PSEC attachment to `CreateProcessW`, rather than a game-specific storage/auth provider.
- **[M4]** [Implemented UI policy resolver](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/src/mxc-sdk/src/core/mxc_common/ui_policy.rs) and [UI policy guide (draft)](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/docs/backends/process-container/UIPolicy_Schema.md): distinguish Win32k permission, USER handles/atoms and input restrictions; policy is not game acceptance.
- **[W1]** [SHGetKnownFolderPath](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetknownfolderpath): Known Folder lookup is user/token scoped; launcher environment alone is not evidence of the game's lookup.
- **[W2]** [RegOverridePredefKey](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regoverridepredefkey): remapping applies only to the calling process.
- **[W3]** [MSAL.NET WAM documentation](https://learn.microsoft.com/en-us/entra/msal/dotnet/acquiring-tokens/desktop-mobile/wam): broker integration context only; not a statement that Education uses MSAL.NET or WAM.
- [Pinned Windows OS support table](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/docs/backends/process-container/os-version-support.md) defines backend release prerequisites; actual target OS/API compatibility remains **UNKNOWN**.
- Existing source correction at `829e577` reserves a zero-initialized 64-bit PSEC support output, matching the [pinned MXC V2 FFI](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/src/mxc-sdk/src/backends/process_container/common/secenv.rs). It remains diagnostics only.

**Proof performed:** exact-source owner/consumer and official upstream contract review, with pinned references; documentation consistency checks. **NOT_EXECUTED:** Rust/Windows SDK compilation, automated suites, PSEC launch, game/authentication/world accesses, GUI/GPU/input, multiplayer, clean-host and four-client acceptance. No source-execution or gameplay PASS is awarded. Only this handoff changes; public APIs, admission, CI, installed services, profiles and worlds remain untouched.

## SINGLE ACTIVE PRIORITY — P0: real MC02 isolation provider

**Operator direction — 23 October 2026:** build an original Minecraft-specific engine using Sandboxie's architectural ideas as reference, with independently authored code and no retired-runtime dependency. Our engine may provide transparent isolation; an official Education per-instance launch option is not a prerequisite. This supersedes the former vendor-contract-first next action, not the PSEC-only NO-GO. The operator later explicitly authorized original compatibility-DLL source development, superseding the no-injection design constraint; game files and ordinary manual login/licensing remain unchanged. Runtime injection and driver/service installation are not authorized by that source-development decision.

The [architecture mapping](../docs/system/independent-isolation.md#original-engine-architecture--operator-decision-23-october-2026) maps filesystem, registry, IPC/objects, process lifecycle, authentication and presentation onto existing owners. Runtime-core retains domain/provider policy, the desktop retains native coordination and presentation, and the existing admission remains authoritative. Synthetic COW/hives and the inert service are not native mediation.

**Native route assessment:** [source assessment](../docs/system/independent-isolation.md#native-mediation-assessment--23-october-2026) finds supported file-open redirection and registry operation substitution mechanisms, but no complete MC02 provider. A minifilter must bind private backing objects/per-open state, not classify all I/O by current PID; callbacks need live process/key provenance and correct operation results. The first uncovered subsystem is the actual game's singleton/IPC path, followed by external auth-helper attribution. Both remain UNKNOWN. PSEC-only NO-GO is retained; the original mediation route is NOT_SELECTED, not declared universally impossible.

**P0 exit:** MC01 and MC02 playable and independently licensed together; separate worlds persist after relaunch; effective HKCU/auth do not cross; contained game access cannot expose protected host/peer state; GPU/input and recovery work. Do not promise confidentiality against the owning host user/administrator. Source tests and synthetic controls remain supporting evidence only.

## Ordered outcomes (not new test-status gates)

| Stage | State | Exit |
| --- | --- | --- |
| **P0a Provider feasibility** | Native mechanism review complete; game singleton/auth evidence missing | One supported unmodified-game MC02 mechanism is established or decisively rejected with an explicit incompatibility. |
| P0b Two-client vertical slice | Source integration preparation active; game feasibility/acceptance unresolved in P0a | Integrate the two-slot experiment, then validate actual MC01+MC02 private data/auth/gameplay under applicable authorization; maintain product Launch denial until ready. |
| P1 Four-client extension | WAITING | Reuse the SAME provider for MC03/MC04; verify four licensed worlds, normal input, multiplayer/recovery. |
| P2 Efficiency against Sandboxie | WAITING | Comparable real-game workloads for RAM, CPU, GPU, frametime, launch/switch latency and stability; optimize measured bottlenecks. |
| P3 Distribution | WAITING | Signed native Windows package, safe install/repair/uninstall/rollback preserving worlds/accounts; release needs separate approval. |

Old G0–G4 docs concern the retired VMware campaign, not these native delivery outcomes.

## Codex working rule

Read root `AGENTS.md`, `GITHUB_RULES.md`, `.agents/skills/emulator-development/SKILL.md`, then this single handoff; pin exact `Local` SHA. Start with `engine/runtime-core/src/independent_engine/` (domain/provider ports), `apps/desktop/src-tauri/src/engine/instance_identity.rs` and `native_clients.rs` (native Windows observations), and `instance_catalog.rs` (isolated Launch deny gate). `engine/isolation-native/independent-service/` is unbundled and inert/diagnostic. Retired Sandboxie/fork/VM routes are NOT active engineering tasks.

Prefer proof of game-compatible isolation over generic Windows experiments, more mock files, many microtests, UI polish and broad documentation. No second lifecycle/store, credential access, unsolicited driver/service install, release/main changes, unsafe profile operations, or GitHub Actions. Execute targeted build/tests if actually available; if Windows/game execution is unavailable, label it `NOT_EXECUTED` and complete only source work with a real path to P0. Do not ask the user to run a PC merely for source-verifiable tasks.

On each meaningful bounded commit, replace the **current** blocker/next status in this file instead of appending a new chronology. Commit to `Local` with `Work/State/Decision/Proof/Unresolved/Next` trailers, keeping original runtime unknowns explicit.

## Current owner and next action

**Operator direction — 24 October 2026:** mature the original MC01/MC02 engine
without Computer Use, game-window input, fresh game launches or closures while
the operator is working. Preserve existing accounts/worlds. Public MC02–04
admission, retired providers and GitHub Actions remain disabled.

**Source baseline:** exact remote Local
`70cfae44112bb1afac306adab052209e122a6144`, plus the literal input blobs below.
Native preparation/enrollment remains in
`apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`; the private config,
enroller, compatibility DLL and own controls remain in
`tooling/acceptance/windows/bootstrap-ordering/`. No second lifecycle owner,
production profile catalog, public command/schema, driver or service is added.

### Confirmed defects already corrected

- Private path validation rejected ordinary spaces and nested enumeration's
  final directory separator. Attribute queries and ordinary delete/rename
  coverage also differed from open/create. Shared routing now uses pinned
  private parents and actual OS results; root/destructive/reparse restrictions
  remain. WRITE_OWNER/WRITE_DAC on ordinary private children reaches the actual
  private object; profile-root security/destructive operations remain refused.
  The enumeration/security-open reproductions failed before and passed after.
- Fixed journal names falsely failed later successful runs. Per-process-birth
  `startup-observation-v3-{birth:016x}.bin` names preserve four exact journals
  over two rounds; create-new still rejects a repeat save of one lifetime.
- Nonzero original child exits remain runner failures even if journals save.
  Original process handles are retained; attempted children are not killed on
  manager drop. Captions use immutable slots: actual Minecraft - 1/2 observed.

### Actual game evidence and its boundary

The earlier space-only build crashed both games at world creation with
0xC0000005 and Windows-reported game-module offset 0x245bf61. A later
pre-enumeration run recorded MC01 0xC0000005 and MC02 exit 0. The exact causal
instruction/call chain remains UNKNOWN; drive capacity was not established as
the cause. No raw crash dump or account/token/cache/world content was inspected.

The corrected DLL later reopened the SAME two profiles to their main menus
with distinct player identities and no password entry/authentication automation.
Both were playable; the operator confirmed movement in each. Two players were
observed in one world, followed by normal guest departure with the host alive.

Two new distinct test worlds, UJI-MC01-Persistensi and UJI-MC02-Persistensi,
then reached concurrent gameplay. In-game commands placed gold under MC01 and
diamond under MC02, each reporting Block placed. Both saved via normal Save &
Exit. MC02's world list showed its own test world and the preserved old world,
not MC01's new world. Both apps closed normally: both original child exits 0,
both journals saved, manager exit 0. This is create/play/save/clean-close evidence.
A later same-store launch confirmed resume calls only before Computer Use was
stopped. Reopened accounts/worlds/markers after that full process restart remain
NOT_VERIFIED. Do not repeat or close that run without checking actual lifetime
and operator intent; current work is source/automatic controls only.

Game-tested manager:
`eaa65ac1997bf6227b374bd2dce893346d7258c3fced46ed4973ac966d059687`.
Latest source-only manager build (not launched into Minecraft):
`9e93645f848f9b4780a6b156672b4a355f68e947d17d9a0873d86e0df34fa0db`.
Compatibility DLL:
`a59fb1803ff3d957b9691017f24dd7211a9b3f88b6a7b13988432210d67e02ec`.
Enroller:
`18f4157b524f142f2bac85efd6d95b5df69d161a1e1a0da6c10788c94868f384`.
Official game image:
`04164c1c64458a5d984dbc1687d0e8023d1808c44ee7be03519cdf8355233eaa`.
The image is checked before store access. No installer was built.

### Existing automatic storage and helper evidence

The configured C++ control uses generated canaries, not game databases. Its two
profiles/two normal rounds cover nested official-directory shape, Unicode/temp
names, metadata parity, flush, 64 atomic replacements, native relative rename,
delete/delete-on-close, same-store reopen, distinct backing and unchanged host.
Failed replacement preserves old/retry bytes; CREATE_NEW collision preserves
old data. Escaping rename/link and profile-root destruction remain refused.

Two synchronized own-writer interruption phases cover flushed pending data
before replacement (exit 91) and flushed mapped data after replacement (exit 92).
The unhooked parent verifies exact backing bytes, released handles/byte locks,
exclusive reopen, new-process same-store reopen, no retry names after commit,
peer survival and unchanged prior CURRENT/host markers. Overlapping byte locks
fail while a distinct range works. This does not prove mapped memory obeys
byte locks, torn-write/disk-full/power-loss durability or game recovery.

Fixed own helpers from the enrolled parents show private APPDATA/LOCALAPPDATA
inheritance alone does not supply private Known Folders or HKCU: unbound helpers
read only the generated host marker. Explicit low-level bootstrap makes both
Known Folders and both HKCU access forms private. Four unbound/four explicit
helpers are checked per configured run; the enlarged case passed eight repeated
runs, all four C++ modes and then-current 15 Rust tests. This identifies a
fixture gap, not the real Education helper/broker route.

The EnrollPreparedEducationV2 ABI refuses an already mediated caller even with
correct child config. That caller's effective RegOpenCurrentUser key name is
confirmed different from config.userKeyPath. Keep this manager-context check;
the low-level own-fixture positive control is not a production fallback.
`compat.cpp::observedProcess` merely forwards CreateProcessW arguments and
records metadata; it does not enroll children. Broker activation is separate.

### Current implementation: fixed helper transaction through the Rust owner

A cfg(test)-only OwnedHelperTransaction now borrows the original parent and
pinned BootstrapLibrary in the same native owner. It creates only the fixed
bootstrap-ordering.exe next to the pinned payload and requires the parent's
pinned image identity. Parent PID/birth/session/slot/config, logical/private
IDs and path, plus the exact retained/remote registry object must still match.
A fallible duplicate of the SAME slot's existing capabilities supplies a fresh
child config. Singleton caller RVA is cleared, observation is absent, and the
manager calls the existing reviewed enroller ABI. Resume rechecks both live
bindings, same-slot roots/image, then uses the existing single-attempt owner.
Lifetime borrows retain parent/payload ownership; Drop preserves attempted
children and cancels only original unstarted children.

The new integration test keeps two own enrolled parents live and runs one
manager-created helper per slot. Both helper enrollments, effective private
Known Folder/HKCU checks, second private writes and host-canary checks pass.
Unattempted parent, changed parent birth/image/registry/path and changed helper
birth/slot refuse before resume. Repeat resume refuses. Abandoned prepared
helpers exit 90 while both parents stay alive. Parent normal exit before helper
resume causes refusal and original unstarted-helper cancellation with exit 90.

**Executed on these inputs:** 16 Rust bootstrap tests PASS (including the new
explicit own-executable case), preview manager build PASS, rustfmt and diff
whitespace checks PASS. C++ source/DLLs are unchanged in this batch. This
transaction exists only in tests and creates a manager-owned sibling process;
it does not yet receive/adopt a game-created descendant. No real helper or game
was launched, activated, closed or injected during this batch.

Literal code-input blobs (documentation excluded):

| Input | Git blob |
| --- | --- |
| `independent_bootstrap.rs` | `07a2c681947e2d3b71835e99e1dacaa97e315a56` |
| `compat.cpp` | `400e4453a18b6b9bbefe9401ef798380c003d5d7` |
| `probe.cpp` | `0971a9776a74284de166f03248b3c6991d66c9f5` |

### Remaining proof and one next action

UNKNOWN/NOT_EXECUTED: real saved-world/marker reopen after restart, long
concurrent gameplay and game crash recovery, actual Education helper/broker
attribution and propagation, full HKCU/auth/host-peer containment, partial
writes/disk-full/power loss and comparative FPS/frametime/GPU/CPU overhead.
Prior raw peer-access control showed name translation is not a security
boundary. Four clients and distribution remain NOT_EXECUTED. Fixture coverage
does not establish full stability or a development percentage.

**One next action:** exercise original suspended own-descendant handle handoff
to this existing Rust manager transaction, with process/image/slot provenance
checks and no PID reopening. The present transaction creates its helper itself;
that is not captured-child enrollment. Keep game-helper propagation and public
admission disabled; actual Minecraft restart/soak acceptance waits for an
operator-selected window without unsolicited Computer Use.
