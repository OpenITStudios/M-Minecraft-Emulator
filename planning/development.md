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
`e2905a3e0bb3c4ae94693417b90d3002ecc35c7d`, plus the earlier handoff input blobs below.
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

Earlier create/play/save/clean-close manager:
`eaa65ac1997bf6227b374bd2dce893346d7258c3fced46ed4973ac966d059687`.
Current manager exercised by two-client runtime (not release acceptance):
`5912f5dad64a3730c0df9b90a3ba0ea781059d556ef181b349fc21b762fb40e8`.
Earlier game-tested compatibility DLL:
`a59fb1803ff3d957b9691017f24dd7211a9b3f88b6a7b13988432210d67e02ec`.
Earlier game-tested enroller:
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

### Existing implementation: one owner with a test-only descendant handoff module

The earlier cfg(test)-only OwnedHelperTransaction in the existing bootstrap
owner creates a fixed own helper from the same slot, borrows parent/payload
pins, publishes fresh config and uses the reviewed enroller before one resume
attempt. It checks PID/birth/session/image, retained directory/path and registry
object bindings. Changed parent birth/image/registry/path and helper birth/slot
refuse before resume. Abandoned/unstarted helpers and parent-exit races cancel
only the exact own unstarted child; attempted parents are preserved.

The new internal module
`apps/desktop/src-tauri/src/engine/independent_bootstrap/owned_handoff.rs`
now accepts original suspended own descendants. The existing owner remains
responsible for slot policy, enrollment and resume/cancel. Request transport and
provenance checks stay in the small internal test-only module. Native wire
layout is centralized in `owned-helper-handoff.h`, 88 bytes with C++/Rust size
and offset assertions, separate from game config and public schemas.

An enrolled own C++ parent creates its fixed self-image suspended, then holds
original process/thread handles while reporting their sender-local values.
The Rust manager duplicates them from its retained original parent handle;
it does not reopen by PID. Before constructing a cancellable Pending child it
checks parent identity/slot, exact helper receipt object, child PID/thread/birth,
session, fixed pinned image and native parent PID. It then publishes a fresh
same-slot config, calls the existing reviewed enroller and acknowledges transfer.
The sender closes original handles without resume/termination after acceptance;
the existing manager transaction owns the single resume attempt. On refusal or
request drop the sender still owns cancellation. No receiver handle value is
published; therefore a late sender cannot leak new handles into the receiver.

The new integration test transfers actual own descendants from two live enrolled
parents. Wrong receipt and birth are rejected. A complete valid live peer-child
PID/thread/birth/image tuple is also rejected by native ancestry without killing
the peer or either sender. Accepted descendants run with separate private slot
writes; parent/host canaries remain intact and both helpers exit normally.
Replay refuses. A second test drops requests before creation and after handle
publication: the sender reports a refused fixture (exit 86), cancels its own
already-created suspended child (exit 90), and committed private bytes plus host
canaries remain unchanged. An allocation-order fix also retains the proof
handle before mapping the request, so a failed handle clone cannot leak that
mapping.

**Executed on these inputs:** 18 Rust bootstrap tests PASS, all four C++ modes
PASS, MSVC 19.44 x64 Release /W4 /WX build PASS, preview manager build PASS,
rustfmt and diff whitespace PASS. New handoff Rust code is cfg(test)-only.
Compatibility DLL/enroller and game config/public schemas remain unchanged.
No Computer Use or game process/input operations occurred in this batch.

**Scope limits:** this is a cooperative fixed-own sender; its source uses
CREATE_SUSPENDED, and the control is not a hostile-process authentication or
independent thread-suspend-state proof. Native parent PID follows Microsoft's
[ProcessBasicInformation layout](https://learn.microsoft.com/en-us/windows/win32/api/winternl/nf-winternl-ntqueryinformationprocess),
which Microsoft marks internal and subject to change. Missing/mismatched native
results fail closed; there is no PID fallback. Broker activation need not have
the game as direct parent. Actual Education helper identity/policy is still
UNKNOWN; this control does not activate real-game child propagation.

Earlier descendant-handoff code-input blobs (documentation excluded):

| Input | Git blob |
| --- | --- |
| `independent_bootstrap.rs` | `eba522d4a21a74939b15e2a20ccb589e616d9962` |
| `owned_handoff.rs` | `25a1b567245d8036a91a1520c125bc6f996a84c7` |
| `probe.cpp` | `ceae3b0ae780ab5a65f7931e627b7bcf7f4373a3` |
| `owned-helper-handoff.h` | `5f87ced170887b25ba5d6890aebc4db9334f50d7` |

### Current audit: installed helper coverage and fresh-window refusal

Read-only installed-file and process/module metadata found Education 1.26.32.0
with the same official image hash recorded above. Its installed tree contains
Minecraft.Windows.exe, Minecraft.CodeBuilder.exe and MinecraftEducationUpdater.exe;
Minecraft.AdalServer.exe is absent. Two existing game processes were responding
and were preserved. Neither a live legacy helper nor a matching ADAL/MSAL/broker
module was observed; both game processes loaded libcef.dll. This snapshot does
not identify the actual sign-in route or exclude transient/OS broker activity.
No process command line, account/cache/world content or credentials were read.

The existing observer hard-coded the legacy helper name. It now refuses with
Unsupported/UNKNOWN before process inspection when that candidate is absent,
and rejects a non-file candidate. The existing UI explicitly labels its legacy
coverage and treats an empty candidate snapshot as inconclusive. Existence
alone does not establish signature, attribution or permission to enroll it.
There is no automatic helper fallback, new public API or game propagation.
The official [connection troubleshooting article](https://edusupport.minecraft.net/hc/en-us/articles/4404976926356-Troubleshoot-the-Can-t-Connect-to-the-Service-Error)
names the legacy helper, but does not establish its presence in this installed
version. The historical [1.19.53 update notes](https://edusupport.minecraft.net/hc/en-us/articles/13961617817236-What-s-New-in-The-Wild-Update-v1-19-53)
describe migration to MSAL; they do not specify a current isolation mechanism.

The scoped regression run also exposed an existing fresh-window guard defect:
it checked observed responsiveness but accepted a target marked nonresponding.
The original regression failed before correction. The existing matcher now
requires both target and fresh candidate to respond, retaining exact PID,
birth, session and HWND matching. Its focus and placement consumers therefore
refuse this stale target instead of presenting it. No window was operated.

Closed original-run journals were decoded only after their writers had exited.
Each slot recorded two unsupported path shapes and no access/options rejects;
the first sample was a relative create, length 121, status 0xC00000BB. The
metadata does not expose the exact path or establish which validation failed.
Missing-object parent/backing statuses can also be normal existence probes;
they are not automatically corruption. These residual operations remain
UNKNOWN; no path guard was relaxed on an unproved hypothesis.

Executed on this source: all 36 scoped native-client Rust tests PASS, including
18 bootstrap tests and the new missing/non-file helper control. These use
own generated fixtures, not Minecraft or the retired runtime. Frontend
typecheck PASS (zero errors/warnings; available Node 24.14.1, below the declared
24.22.0 range). No new game acceptance, installer, Computer Use, game launch,
closure, sign-in or window input occurred. Existing source warnings remain.

### Current change: distinguish the existing path rejection branches

Source inspection cannot distinguish the old length-121 samples: both parent
and composed-name validation share the same diagnostic class, and several
rules reject paths below the total length limit. No exact failing filename or
cause can be inferred. The existing validator now optionally returns only a
numeric reason, and native create/open attach the validation stage to the
first unsupported-path sample. No policy is relaxed and no new routing owner
or logger is introduced. Query/rename/registry callers retain their existing
boolean behavior. The private journal's existing three reserved words carry
detail schema 1/stage/reason, published before the sample commit flag. Its
size/version/offsets and Rust copying behavior remain unchanged; old zeros
mean unavailable. No filenames, path hashes or account/world content is logged.

An own-executable control sends a nine-component relative create followed by
an ADS-shaped name through the real mediated NtCreateFile. It verifies both
refusals, null handles, native status, unchanged LastError and first-sample
composed/depth detail surviving the later distinct failure. The existing
Rust-to-C++ two-slot/two-lifetime test failed with the original logger and
passed after publication. This is diagnostic regression evidence, not a
Minecraft storage fix or crash-resolution claim. The new C++ artifacts are
source/control builds only; reviewed real-game payload pins and staged/live
artifacts are unchanged. No game process, input or data operation is performed.

Executed on these inputs: MSVC 19.44 x64 Release /W4 /WX build PASS, all four
C++ control modes PASS, and all 36 scoped native-client Rust tests (including
18 bootstrap cases and the strengthened observation control) PASS. Diff
whitespace check PASS. This batch does not change Rust/frontend source or
claim a new preview-manager/frontend/game acceptance build.

### Current repair: directory traversal during native rename workflows

The audit found a concrete compatibility defect in compat.cpp::unsupportedFile:
FILE_TRAVERSE shares FILE_EXECUTE's bit, so an existing child directory opened
without explicit DIRECTORY_FILE was rejected. The unmediated reference passed
its mutation workflow, while the private child failed at that directory open
before correction. The existing native create/open owner now forces the actual
OS open to DIRECTORY_FILE for that access. NtCreateFile permits unspecified-type
inference only for FILE_OPEN, not create/overwrite. Explicit NON_DIRECTORY and
ambiguous creation requests remain refused; a new regular-file traversal check
and absent-after-refused-create check pass. No generic sandbox layer or new
public interface is added. Other-path calls retain their original options.

One shared own-data workflow checks six rename combinations (classic/Ex x
rootless/directory-handle-relative/NT absolute), both delete forms, unchanged
caller buffers/LastError and successful target file identity. It runs on actual
unmediated Windows, then both private slots over two lifetimes. Collision,
read-only/busy targets and missing DELETE access preserve both generations.
Classic replacement of an open target refuses on this host; Ex POSIX preserves
its old reader while new opens see the replacement. Delete/cancel survives
handle closure, insufficient rights cannot delete, and permitted deletion
removes the generated target. Both native NtOpenFile and Win32/NtCreateFile
directory traversal paths pass. Existing world-workflow canaries remain intact.

This fixes a demonstrated source/control defect, not a proved cause of the old
game crashes or length-121 path failures. The new native artifacts are not
deployed; real-game payload hashes, staged artifacts and live games remain
unchanged. Actual Minecraft acceptance on this repair remains NOT_EXECUTED.

Final executed checks: MSVC 19.44 x64 Release /W4 /WX build PASS; strengthened
configured-storage control PASS after its pre-fix private-directory failure;
all four C++ modes PASS; all 36 scoped native-client Rust tests PASS; diff
whitespace PASS. No Rust/frontend source or public schema changed in this batch.

### Previous candidate: reviewed payload pair and mandatory existing-profile replay

The native review traced BootstrapLibrary::reviewed/load, the read-only config
and enroller ABI, original suspended child ownership, DLL attach, replay and
one-shot resume consumers. The manager hashes both pinned ordinary DLL files
before loading code or preparing game stores, pins every ordinary ancestor and
binds the loaded enroller file identity. The enroller checks original process,
thread/birth/session/image and per-slot registry binding; DLL config validates
the pinned storage IDs. Helper propagation remains test-only; the game observer
does not automatically suspend/enroll children. These are cooperative source
and own-control properties, not full hostile-process or loaded-game attestation.

One persistency risk was corrected in registered_slots: observation previously
fell through to fresh_slots if REVIEWED_REPLAY was absent. It now refuses with
Unsupported before Known Folders, registry/store access or profile allocation.
The explicit preparation-only route retains its existing fresh-store behavior.
The new absent-policy regression passes with the deliberately empty test
policy, without touching real profiles. Existing replay tests still check
wrong capability hash/identity/missing stores and no fresh-profile fallback.

The current source/control DLL pair is now pinned for the private opt-in
acceptance path, without enabling public admission. New source-only artifacts:

| Artifact | SHA-256 |
| --- | --- |
| Manager | `790c5b774ef731ffea5fe314394daff69f32c8a0e95eadfa667f0462537062bb` |
| Enroller | `2091e9d25e63e90a4241c43f84fa9f2588e7ab6273612286e565d0e96ff932ed` |
| Compatibility DLL | `4cc1a12146b38aad58e1b0ad43926fbffdbdfe5d2b1c111e456e62f78801cb17` |

A separate local acceptance-candidate-20261025-e5f51a4 directory holds only
these new code artifacts and a byte/hash-verified copy of the existing 216-byte
replay metadata capability. Its local manifest records the exact code hashes
and source commit; no capability bytes, profile IDs, raw journals or machine
paths are committed. No profile directory, token/cache, account or world content
was read/copied. The old/live staged manager and payload remain untouched.
Installed game image hash still matches the recorded official hash.

Executed on these Rust inputs and the unchanged reviewed native pair: all 37
scoped native-client tests PASS (including 19 bootstrap cases), preview manager
build PASS, rustfmt and whitespace PASS. The actual staged candidate was run
only with an invalid extra argument: its original process exited 2 with
startup-refused/InvalidData before entering the game/store inspection path.
This tests refusal only, not loading the payload or game startup. A first GUI
process invocation lacked a waited exit result; its absence was confirmed
before the waited control was run. The waited process's exit was verified.
Read-only metadata confirms the same two existing responding game lifetimes;
no new game, restart, closure, window input or Computer Use occurred.

### Current repair and actual runtime: private Temp identity for CEF

On 25 October the operator selected an integrated acceptance window, finished
using the old games, and authorized the new pair. Both previous original game
processes exited 0 with saved journals; the old manager was absent, without a
retained exit-code handle for that older run. The e2905a3
candidate reopened the reviewed same pair and loaded its exact compatibility
DLL in both games. The operator reported smooth play, but opening worlds also
produced an unexpected full Chromium New Tab window (screenshot supplied).

Read-only process/module metadata found bundled Minecraft.CodeBuilder.exe
descendants of one game, loading the installed libcef.dll. Predicate-only WMI
queries classified GPU/renderer/utility roles and technical switch/path patterns;
no command-line strings, URLs, credentials, cache or world contents were read.
There was no untyped CodeBuilder browser process in that snapshot. The known
private pilot basename and MC01/MC02 patterns did not match after user-data-dir;
Temp/minecraftpe patterns did. These predicates do not expose an exact path or
alone attest its ownership. The helper lacks an individual Authenticode
signature; installed ancestry is not publisher approval or enrollment policy.

CEF branch 6723 (matching the installed Chromium 130 family) documents a
[root-cache process singleton and default Chrome-window relaunch behavior](https://raw.githubusercontent.com/chromiumembedded/cef/6723/include/cef_browser_process_handler.h).
Its [settings contract](https://raw.githubusercontent.com/chromiumembedded/cef/6723/include/internal/cef_types.h)
requires separate root-cache identities for independent instances. Source review
found APPDATA/LOCALAPPDATA were private but TMP/TEMP were inherited unchanged.
Native file redirection alone therefore could not supply a distinct temp path
identity to the unmediated CEF subprocesses or cache-based singleton decisions.

The existing SlotStorage owner now retains ordinary Local and Local/Temp handles
with FILE_LIST_DIRECTORY/READ_ATTRIBUTES and no delete sharing. An existing
Temp directory is opened, never cleared; only a missing empty Temp child under
the pinned Local is created. Missing Local still refuses replay, and reparses
are rejected by the existing directory validator. child_environment replaces
TMP and TEMP case-insensitively with that canonical private physical path.
Duplicated own-helper storage keeps the same pins. No game file, profile root,
wire schema, DLL hook, token/cache/world data or real helper enrollment changed.

The actual own-executable GetTempPathW/TMP/TEMP control failed before this fix
and passed in both slots/two lifetimes afterward. Replay verifies same Temp IDs
and retained distinct canary bytes, live pins refuse removal, and after dropping
only own fixture pins a missing Local still refuses without recreating it.
An attribute-only directory handle did not prevent removal on this Windows
host; adding directory-list access made the pin-removal regression pass.
All 37 scoped native Rust tests, preview manager build, rustfmt and whitespace
checks PASS. The final test-only pin-removal refinement removes only the own
canary first, so it proves denial on an empty Temp rather than a nonempty-folder
failure. Product runtime code is unchanged from the exercised manager.
The native DLL pair remains byte-identical to e2905a3: enroller
2091e9d25e63e90a4241c43f84fa9f2588e7ab6273612286e565d0e96ff932ed and
compatibility 4cc1a12146b38aad58e1b0ad43926fbffdbdfe5d2b1c111e456e62f78801cb17.

After normal user save/close, both e2905a3 original children and manager exited
0. With explicit operator authorization, the separate private-temp candidate
(current manager hash above, same DLL pair and replay metadata) launched the
same profiles. The operator then confirmed Chromium no longer appears in
either world, automatic sign-in succeeded, gameplay is smooth, and both gold
and diamond world markers survived restart. These account/world/gameplay facts
are operator-confirmed, not UI automation or token inspection. Exact duration,
long soak and forced game-crash recovery are not established. No Computer Use
was performed. The current games were preserved after the report.

Post-world metadata found six CEF descendants per original game; all matching
MC01 user-data-dir patterns belonged to the first game, all matching MC02
patterns to the second. Both original games remained responding. This plus the
controlled Temp change and disappearance of the popup supports the shared-temp/
CEF identity diagnosis, without claiming a traced CefInitialize call stack or
full helper HKCU/authentication containment. The local private-temp candidate
manifest binds its exact files/source commit; private runtime IDs remain local.

The closed e2905a3 journals finally identify the older two length-121 create
rejections in each slot: detail schema 1, stage ComposedPath (3), DepthLimit (4).
Counts were [314,2,0,46,46] and [322,2,0,358,46]. The writer identities and
headers were validated only after original exit; no filenames were collected.
The eight-component validator limit is now a demonstrated residual storage
restriction, not an unknown hypothesis. Its exact requested filenames remain
unknown and the current Temp fix does not change that depth policy.

### Remaining proof and one next action

UNKNOWN/NOT_EXECUTED: long concurrent gameplay and game crash recovery,
actual Education authentication helper/broker
attribution and propagation, full HKCU/auth/host-peer containment, partial
writes/disk-full/power loss and comparative FPS/frametime/GPU/CPU overhead.
Prior raw peer-access control showed name translation is not a security
boundary. Four clients and distribution remain NOT_EXECUTED. Fixture coverage
does not establish full stability or a development percentage.

**One next action:** correct the demonstrated eight-component storage depth
restriction in the existing validator/parent walker with a capacity derived
from the already bounded relative path length, and verify deep native/Win32
operations with own data. Keep traversal, ADS, reparse and profile-root guards;
do not suppress the rejection or hide browser windows. Preserve the current
games; further game restart/soak requires an operator-selected window. Actual
authentication attribution/crash recovery remain UNKNOWN, and public admission
stays closed despite the now-confirmed two-client account/world restart result.
