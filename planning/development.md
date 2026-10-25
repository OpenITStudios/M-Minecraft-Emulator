# Current Development Handoff — M-Minecraft-Emulator

**Authority:** `Local` is the only active development branch. This is the **single current goal, blocker and next action**, not a chronological TODO file. The previous 3,009-line ledger with 105 historical `Next:` entries is [preserved at commit `d239a299e04c`](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/d239a299e04c23d2324b52325714600c00c34498/planning/development.md). All its former next actions are superseded; Git history preserves implementation decisions.

## Primary goal — A / B / C Minecraft efficiency

**Resumed and explicitly established by the operator — 25 October 2026.**
These three outcomes supersede earlier priorities. Source tests, diagnostics,
fixture counts and minor launcher/HUD savings are not substitutes for them.

| Track | Required outcome | Owner and boundaries | Current evidence |
| --- | --- | --- | --- |
| **A — saved Minecraft settings** | Keep the approved Simple preset independently per original profile and across normal restart. Render distance **at least 12**, simulation distance unchanged; preserve resolution, accounts/worlds and 144/10. | Minecraft's existing options file owns the settings; existing bootstrap owns persistent per-slot Roaming/Local routing and identity-capability replay. Do not introduce a duplicate preferences store or rewrite options while a game is live. | Both original per-slot options files retained graphics_mode=0, gfx_viewdistance=192, gfx_msaa=1 and the five approved effect switches=0 after both game exits. Disk persistence is observed for both; the restarted MC02 UI also reads Simple / 12 chunks. The fresh MC01 main menu shows its original account; its complete post-restart video UI has not been rechecked. |
| **B — internal Minecraft efficiency** | Reduce unnecessary mesh/buffer/asset/cache or renderer work using a real Education-compatible integration, inspired by Java optimizers. | A new native internal optimization needs one demonstrated resource owner/lifetime seam. Existing Present mediation owns pacing/HUD, not game meshes/heaps. A generic allocator/hook fixture or wrapper micro-optimization does not satisfy B. | Existing exact-source Sodium/FerriteCore research identifies concrete algorithms; a compatible internal Education resource/mesh ownership seam remains UNKNOWN. Do not promise a Java mod port or a RAM percentage. |
| **C — inactive clients first** | Prioritize lower RAM/CPU/GPU demand while a client is not controlled; keep foreground 144/background 10 ceilings, prompt focus recovery and continuing Education host/network/saves. | Focus/pacing retain their current owners. Host role is separate from focus. No whole-process suspension, arbitrary memory/CPU caps, periodic trimming, token/profile sharing, fabricated occlusion result or freeing game-owned resources. | The existing limiter is implemented and prior 10-FPS observations are historical. Further game-owned background work reduction, its resource benefit and host health require a concrete intervention; no new savings are claimed yet. |

**Working method:** choose one real intervention in A/B/C, implement it or
explicitly reject its concrete integration boundary, then use only checks that
directly establish its benefit and failure risks. A before/after reading needs
equivalent world/workload and metric semantics. Do not expand collector APIs,
generic sandbox layers, synthetic milestone flags or ceremonial test loops.
Use available source evidence without making repeated PC sessions prerequisites
for source work. Current user permission allows full PC control, but avoid
interfering with concurrent user input. Preserve all account/world data.

**Immediate findings on resume:** at first discovery both earlier game identities
were still present; subsequent discovery found no game processes. The existing
manager log records journal-saved/exitCode=0 for both, with no agent closure in
this resumed turn. The approved eight options entries above remain on disk in
each original private Roaming path. No account/token/world/cache content was
read and no settings were rewritten. The absence of Code Builder while games
are closed is not evidence of helper savings. The operator subsequently confirmed coding is unused and authorized disabling
Code Builder. After both clients saved/exited the multiplayer world normally,
the existing host world's offline Edit World → Coding & Scripting UI was used
to switch only Code Builder off and Save changes. Education displayed its saved
confirmation. This is world-scoped; other worlds/defaults were not changed.
No helper was killed and no RAM reduction is inferred from the toggle.

**C minimization candidate — closed without automatic rollout:** the original
guest joined the existing MC02 Education host world. A normal guest minimize
was confirmed by the window API's minimized-state response; restore returned to
the same multiplayer pause surface with two participants. The host continued
through an observed night-to-day transition. This is a bounded recovery
observation, not a long soak or tick/network performance guarantee.

The confirmed-minimized 10.018-second reading used 1.406 CPU-seconds, working
set 1412.87 MiB and private commit 2432.90 MiB. The earlier background 10.015-second
reading used 1.547 CPU-seconds, working set 1411.00 MiB and commit 2433.99 MiB.
The guest was in its pause UI for the valid minimize reading, unlike the earlier
background reading; therefore CPU/GPU benefit cannot be attributed to
minimization. Working set did not decrease. An intervening minimize request
while mouse capture was active was not confirmed and is excluded. **No meaningful
RAM saving is established; do not implement an automatic minimization policy
from this result.** No GPU/frame trace or collector extension was introduced.
Both world sessions were then saved/exited normally for the authorized Code
Builder change; games remain open in menus. B internal ownership remains UNKNOWN.

Retain independently authored code, Local only, CI disabled, no Sandboxie/VMware
or game-file modifications. Five clients remain deferred until two-client
stability and worthwhile savings are demonstrated. The existing research report
remains technical evidence; this file is the sole current goal/handoff.

## Goal — five clients with foreground rendering policy

**Operator direction — 25 October 2026:** five independent, legitimately licensed
official Education clients (Minecraft - 1 through Minecraft - 5) on i7-14700
without iGPU, 32 GB RAM, RTX 3070 8 GB VRAM and 512 GB SSD. One account uses
Education **Host World**; four join that same world through Education multiplayer.
This supersedes the four-client target. Host role stays with its account when
foreground focus changes. The controlled client has a **144 FPS ceiling**;
all other clients, including a background host, have a **10 FPS ceiling**.
Neither number is a guaranteed achieved rate. Each game's client area also needs a
small, clearly readable **FPS counter at the top right** (operator addition).
It must show a measured per-window rate, remain correctly anchored across
resize/DPI changes, preserve input and have measured low overhead. Metric
semantics must be explicit; unavailable measurements show an unavailable state.
HUD source and own-control checks now exist; actual game acceptance remains
NOT_EXECUTED. Background rendering must preserve
host simulation, saves and networking; that separation remains UNKNOWN until
measured in the actual game. Resolution, monitor refresh and free SSD capacity
are not established.

Retain the original provider and existing accounts/worlds, normal licensed login,
no token copying, no Sandboxie/VMware, Local only and CI disabled. The operator subsequently authorized full PC control and normal closure of
optional applications, retaining Minecraft, Codex and their dependencies. This
supersedes the earlier no-Computer-Use restriction for this efficiency session;
it does not require disruptive game restarts or account/world changes. Use the existing
native/bootstrap owners and resource admission; no second manager or readiness
store. Preserve the proved MC01/MC02 restart result while developing MC03–MC05.
Public admission stays closed until the relevant runtime acceptance.

**Actual starting point at `ad7ad7446b1a104879ca8b6a2655435872c57abc`:** the
operator confirmed two different accounts sign in automatically after restart,
their gold/diamond world markers persist, gameplay is smooth, and Chromium no
longer opens with worlds. Long soak and full authentication containment remain
unproved. Five clients, a per-client renderer limiter and 144/10 FPS switching are
**NOT_IMPLEMENTED / NOT_EXECUTED**; successful fixtures do not establish these.
The historical sections below preserve prior evidence and rejected candidates;
their older scope/next instructions do not replace this goal.

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

### Five-client prerequisite: bounded deep storage traversal

Source baseline: `ad7ad7446b1a104879ca8b6a2655435872c57abc`. The existing
compat.cpp validator no longer rejects paths solely for exceeding eight
components. Its existing 511-character relative-path bound supplies a 256-handle
parent capacity; each ancestor remains pinned, and every opened directory retains
its non-reparse/case checks. Create/open/query/rename use that same walker.
Traversal, ADS, malformed names, length and profile-root protections remain.
The shared registry validator also stops imposing the artificial eight-level
limit; this is not new authentication acceptance.

Long private paths need the canonical root plus the relative suffix. The existing
parent/source identity buffers now accommodate both, avoiding an unclassified
native fall-through when a legitimate private canonical path exceeds 511
characters. Deep rename also exposed INFO_LENGTH_MISMATCH for a one-character
destination: the translated request now retains the padded native structure
minimum, and undersized incoming requests remain refused.

The own-data control creates 16 and 255 parent directories in both private slots,
checks Win32 create/query, native root-relative and deep-directory-relative reads,
native rename to/from a one-character name, distinct slot bytes and persistence
over two process lifetimes. A 511-character suffix succeeds and an overlength
suffix refuses. The host view is checked unchanged. The strengthened control
failed before correction. These are real Windows own-executable operations,
not Minecraft execution or a newly proved cause of the old crashes.

Executed: MSVC 19.44 x64 Release /W4 /WX build, all four C++ control modes and
37 scoped native-client Rust tests pass on the final inputs. Existing Rust
warnings remain. No frontend/Rust API, game payload pin, installed game file,
profile or live candidate is changed. Updated DLLs are source/control artifacts
only; runtime deployment and Minecraft acceptance for this repair are
**NOT_EXECUTED**.

### Per-client pacing core and actual DXGI attribution

Source baseline `91ae21cacb63366f59f05421ac9d2cbceef24026`. The timing core
lives beside the existing compatibility experiment, not in a second lifecycle/
resource manager. `frame-pacing.h` has one rendering consumer and an event-driven
policy producer. Fixed 144/10 ceilings use QPC and a high-resolution one-shot
waitable timer. Policy changes and stop interrupt a pending wait; there is no
polling, global timer-resolution change, suspension, priority manipulation or
catch-up burst. Construct outside loader lock. It is **not yet connected to
the compatibility DLL or Minecraft**.

The opt-in `frame-pacing-control` target creates five own hidden, non-activating
D3D11 WARP swap chains. Synthetic foreground policy rotates through all five.
Final MSVC x64 Release /W4 /WX build/control passed: 554 release samples over
3.056 seconds, 0.062 process CPU seconds, maximum policy promotion 0.019 ms,
joined stop 4.408 ms. Every adjacent release obeyed its selected ceiling.
Cancellation while waiting, a deliberately delayed frame without catch-up and
invalid-argument exit 64 passed. These observations describe this small
own-renderer fixture, not displayed/hardware FPS or Minecraft performance.
Neither actual focus tracking nor host ticking is proved by this fixture.

The operator explicitly selected a runtime window and authorized opening the
prior two-profile candidate without Computer Use. No game was initially active.
The original manager was started with retained process/wait/output handles after
verifying its 5912f5da manager, 2091e9d2 enroller, 4cc1a121 compatibility and
41dbc7db replay hashes. This is the previously tested ad7ad74 runtime;
**91ae21c's storage repair is not deployed there**. Both games responded; the
operator reported both saved worlds open.

Read-only loaded-module metadata found D3D12/DXGI/D3D12Core in both games, with
no EGL/OpenGL module in that filtered snapshot. Module presence alone was not
promoted to presentation proof. The official
[PresentMon v2.3.1 console](https://github.com/GameTechDev/PresentMon/releases/tag/v2.3.1)
was downloaded only into ignored local tooling, verified against release
SHA-256 364e5d98d4d134bd54dd25c22ed2ca2f4883f8bc3ed6502bee0c151e3436d30c
and a Valid Authenticode signature. Non-elevated capture refused access; the
authorized UAC launch completed with original collector exit 0. No service,
overlay or input tracking was enabled.

The 20-second capture attributed 2,589/2,872 presents to the original game PIDs:
**DXGI only, one swap chain per PID, SyncInterval 1, Composed: Flip**. Mean
msBetweenPresents was 7.722/6.963; p95 7.632/7.504. This selects the DXGI adapter
candidate; it does not identify Present versus Present1 callsites, attest a
hook, prove displayed FPS or establish a foreground/background limiter. Raw
PIDs, addresses and CSV remain local.

The existing read-only process sampler completed 30.01 seconds/six samples
without PID birth-identity loss. Selected-game mean CPU was 12.86%/21.58% of
28 host logical processors; maximum working sets were 2397.70/2547.14 MiB.
Single-instance dedicated GPU-memory counter maxima were 758.33/989.35 MiB.
These omit full helper families and are not total VRAM proof. The sampler's
legacy Native/Sandboxie labels identify selected inputs only: **no Sandboxie
runtime was activated**. Foreground and host/guest roles were not recorded
for these samples; they are no controlled optimization/capacity comparison.
The games remain with the operator; no save, closure, game input or Computer Use
was performed.

### Window/chain-bound DXGI adapter

Source baseline `46bf3917a3192fa0b3c19689d7b2342fbfc71f34`. The native
`dxgi-pacing.cpp` adapter now consumes the existing pacing core through actual
Microsoft Detours interception of Present and Present1 in the own renderer
control. It is linked only into that explicit test executable. The compatibility
DLL, enrollment ABI, game payload pins, public APIs/admission and CI are unchanged.
No game process operation, Computer Use, token or world-content inspection was
performed in this batch.

An explicit quiescent installation validates and retains one own-process
window, swap-chain interface and canonical COM identity. The callback's code module is pinned against unload. Successful hook state
is process-lifetime, with no live unload/rebind API; failed installation releases
its temporary references/timer. Same-HWND chains with different identities are
excluded. Thread-local nesting avoids double pacing when the native Present1
implementation enters Present. A competing rendering consumer records a fault
and disables pacing instead of racing/serializing the single-consumer clock.
Concurrency fault handling exists in source but is not real-game acceptance.

TEST, DO_NOT_WAIT and invalid sync intervals bypass the clock. Both wrappers
restore caller LastError before the native call and retain native LastError/
HRESULT afterward. Sync/flags/Present1 parameter pointers are forwarded unchanged.
Stop disables waiting while forwarding the original presentation; no fake native
success is returned. A timing error is diagnostic failure, not policy readiness.

Final executed MSVC x64 Release /W4 /WX build and both control modes PASS:
- Five hidden D3D11 renderers retain 144/10 release timing; final run 549 samples,
  3.047 seconds, 0.047 process CPU seconds, maximum synthetic promotion 0.905 ms,
  joined stop 5.016 ms. No displayed/game FPS inference.
- Actual detours on D3D11 WARP and separately D3D12 WARP flip chains each recorded
  two paced Present calls, two paced Present1 calls and nine bypasses. Native
  failures/LastError, caller parameters, test/nonblocking behavior, wrong-window/
  duplicate-install refusal, unrelated and same-HWND chains, stop and resize
  retain their tested behavior.
- D3D12 queue work is fenced before resize. Its small hidden swap-chain API
  control does not render a representative scene or prove hardware performance.
- Invalid CLI input still refuses with original exit 64. No leftover control
  process was found after these synchronous runs.

The earlier local PresentMon CSV was also rechecked: all 5,461 captured real-game
presents used flags 0 and SyncInterval 1. This is compatible input to the adapter,
not proof of its installation in Minecraft. The old live runtime and accounts/
worlds are not altered; neither new source component is deployed there.

### Bootstrap creation observation and lifetime blindspot

Source baseline `de75985b21ac303366126d828325a35189589441`. The existing
compatibility/bootstrap owner now has opt-in **observation-only** factory/HWND
swap-chain enrollment. Only the exact private environment marker
MIVUBI_EDUCATION_PRESENTATION=observe is accepted after config validation.
Default off and malformed-marker refusal are tested. No game-facing command
enables it; the Rust child environment strips ambient markers. Config remains
v2/1136 bytes; public APIs/admission and CI remain unchanged.

The DLL joins its existing startup transaction for CreateDXGIFactory/1/2.
It creates no factory/device, timer or render thread in DllMain. Actual factory
creation supplies an object-local Factory7 table; writable storage and full
same-address interface support are checked. References/tables are bounded and
process-lifetime. Setup failures are diagnostics, not readiness. No shared
creation method is patched while rendering runs.

Read-only metadata on the two named game processes found an OGLES class in
each, plus auxiliary/IME/Chrome classes. Only own-process, top-level OGLES
candidates are counted. This is a filter, not HWND authority or actual game
enrollment proof. A private in-process Snapshot export supplies counters and
dimensions. No accounts, tokens, worlds or command-line content were inspected.

Own bootstrap children check all three factory export versions and both
CreateSwapChain/CreateSwapChainForHwnd methods on D3D12 WARP, auxiliary exclusion,
unchanged descriptions, failed creation and three eligible creation events.
After releasing a chain, same-HWND recreation succeeds: this observer retains
**no swap-chain reference** and installs no pacing. Two slots/two lifetimes and
the previous storage/helper controls remain intact.

The timing helper separately now exchanges only the selected object's complete
SwapChain4 table, avoiding shared Present-code patching. Native semantics and
resize controls pass; foreign chains no longer enter its callbacks.

**New negative evidence changes the next action:** native D3D12 flip recreation
succeeds before timing installation. Once that timing adapter retains the target
interface/identity, releasing the application's reference leaves recreation
refusing with E_ACCESSDENIED. The exact observed error differs from the initially
hypothesized DXGI_ERROR_INVALID_CALL. The helper explicitly reports
BLOCKED_BY_RETAINED_REFERENCES and swapChainRebindImplemented=false. Its selfCheck
validates the negative control, not stable recreation. The observer's successful
recreation does not fix the timing adapter. **Do not deploy the timing binding.**
This is an own-control limitation, not a newly proved cause of old game crashes.

Executed: MSVC x64 Release /W4 /WX build; all five bootstrap modes (including
opt-in creation observation); both timing modes; Rustfmt; 37 scoped native
Rust tests PASS. The test shell alone supplied an invalid ambient marker.
Removing the Rust strip reproduced failure in the enrolled own-child test;
restoring it passed all 37 tests. The C++ reader asserts marker absence.
Existing Rust warnings remain. No own control process remained after completion.

New DLL artifacts are build/control outputs only. Reviewed payload pins, the
old candidate/runtime, installed game files and account/world data are unchanged.
No Computer Use, game launch/input/restart/closure or credential inspection
occurred. Actual game enrollment/new-code gameplay remains **NOT_EXECUTED**.

### Lifecycle-safe typed timing ownership

Source baseline: `00e7ad99a352a77956fd1a10b09e9d59c0eb7933`. This supersedes
the retained-interface timing experiment above. The timing owner and its own
control now use an unpublished typed COM wrapper rather than replacing the
native object's vtable. One caller-owned reference transfers only on success;
failure preserves the original pointer/reference. Public swap-chain ancestry
through SwapChain4 and optional SwapChainMedia use one canonical IUnknown.
Wrapper aliases keep it alive; its final release drops all native references
before clearing the live binding. The process-lifetime policy retains clocks/
counters only. Recreated chains inherit current policy and sticky stop.

An intermediate, uncommitted native-table/private-data-lifetime attempt crashed
the own control on final native Release. A local debugger stack placed the access
violation in native DXGI deletion dispatch; the exact private-vtable dependency
was not established. That approach was discarded. No game was attached to it.
The final typed wrapper leaves native implementation storage/dispatch unchanged.

Unknown native-supported interfaces outside the wrapper's declared set are
explicitly refused/counted instead of returning a different native IUnknown
identity. The own controls observed zero such requests, which is not proof of
Minecraft's interface requirements. This compatibility gap prevents deployment
approval. The observer's factory tables/pins are unchanged by this timing fix.

Executed on final code: MSVC x64 Release /W4 /WX build; both timing modes exit 0.
D3D11 WARP and D3D12 WARP each pass four repeated same-HWND release/recreate
cycles, five created/five destroyed bindings and zero remaining live bindings.
An application IUnknown alias retains the wrapper until its own release.
All recreated chains retain 10 FPS release intervals. Native device/backbuffer
identity, private-data forwarding, resize, canonical identity (including optional
Media), native error/LastError and TEST/DO_NOT_WAIT bypass controls pass.

Two simultaneous callers on each backend complete eight Present/Present1 calls
without faults. The HWND owner pumps messages. Only permit acquisition is
serialized; the lock is released before native DXGI. This proves the exercised
concurrency path, not arbitrary multithread displayed-FPS spacing.
Each mode reports six paced Present, ten Present1 and seven bypasses, zero
unsupported-interface requests. The five-own-renderer control reports 550
release samples over 3.036 seconds, 0.062 process CPU seconds, 2.562 ms maximum
synthetic promotion and 4.431 ms joined stop. No displayed/game FPS inference.
Invalid CLI input still exits 64; no frame-control process remains.

Only dxgi-pacing.cpp/.h, its control and documentation change. The timing owner
remains linked solely into the explicit control executable. Compatibility DLL,
creation observer, Rust/public API, config ABI, payload pins and CI are unchanged.
No game launch/input/restart/closure, account/world inspection or Computer Use
occurred. New timing behavior in Minecraft is **NOT_EXECUTED**.

### Startup creation capability without factory pins

Source baseline: `1671bfebcca1f52fc93ce63fe5ff4f16ef7bc641`. This resolves
the observer's factory-table/lifetime prerequisite before connecting pacing.
The previous observer retained up to 32 factory interfaces/tables for process
lifetime. An expanded own control failed before correction after additional
factory lifetimes. The observer now leaves all native COM objects/vtables
unchanged and retains no factory or swap-chain reference.

The own controller resolves CreateSwapChain/CreateSwapChainForHwnd method RVAs
from a temporary native factory before creating/resuming each child. A separate
72-byte read-only capability binds PID/creation FILETIME/session, System32 DXGI
file ID, loaded PE size/timestamp and those two RVAs. The child refuses malformed,
writable, foreign-process/session/module or out-of-image capabilities and checks
executable image ranges. These checks bind cooperating parent metadata; they do
not establish complete mapped-image Authenticode provenance or compatibility
with unknown method implementations.

Both creation methods join the existing initial Detours transaction. No factory,
device, render thread or timer is created in DllMain, and no method patch is
installed later from factory callbacks. The three factory-export hooks count
native successes only. Native factory/chain identities, calls and release
behavior remain native. The observer is still default-off/observation-only;
the storage v2/1136-byte ABI and Snapshot layout stay unchanged. No game-facing
command supplies a capability. The actual game enrollment seam does not yet
prepare this new optional capability.

The own control checks a factory recovered from chain GetParent retains its
canonical native IUnknown identity and remains covered by the creation hook.
Thirty-six additional temporary factories create/release chains; each enrolled
child verifies forty eligible creations, with auxiliary exclusion and native
failure/description/recreation checks retained. Two slots and repeated process
lifetimes pass. Nineteen invalid creation-capability variants (plus the existing
five invalid storage configs and one malformed mode) refuse before main.

The first helper regression exposed the new capability variable leaking into an
explicitly enrolled own helper. Both C++ and Rust environment owners now strip
MIVUBI_EDUCATION_PRESENTATION_SECTION as well as the mode marker. The reader
asserts absence of both. Temporarily removing the new Rust strip reproduces
enrollment-test failure (own reader exit 86); restoring it passes.

Executed: final MSVC x64 Release /W4 /WX build; all five bootstrap modes and both
frame-control modes pass. Rustfmt passes. A local isolated Rust 1.99.1 harness
includes the exact independent_bootstrap module and own-helper submodule with
real-game entry points disabled; 18 relevant tests (including ignored own-DLL
interop tests) pass with both ambient presentation variables deliberately invalid.
The harness's initial full run failed only the unrelated caption-window test
because its native-window verifier was deliberately stubbed to refuse; that test
was filtered from the scoped pass, not awarded PASS. Full Tauri build and the
complete prior 37-test native suite were not rerun. No own control remains.

The timing wrapper remains separate/test-only and unmodified. This batch
does **not** connect FPS policy or OS focus. Inspection confirms the existing
instance_focus_events worker is count-only; it has no authenticated child policy
producer yet. No second observer/governor is introduced. New DLLs are local build
artifacts; reviewed live payload pins, original profiles/accounts/worlds, public
admission, CI and distribution are unchanged. No Computer Use or game
launch/input/restart/closure occurred.

### Single-worker focus leases and event-driven native policy

Source baseline: `bb00253a45f3118ec2e1cc19467fdc75fa58e3f1`. The existing
instance_focus_events owner now has an internal, non-command registration for
up to five live process HANDLE/PID/FILETIME identities in the same session.
One existing WinEvent worker serves all leases. Private mode reads transient
foreground HWND/PID with stability rechecks; count-only mode returns before
those reads. Manager focus events are included, and unregistered/unstable focus
demotes clients. Other leases are demoted before a selected lease is promoted.
No titles, account data or persisted policy/readiness store is added.

A policy lease owns level/change/stop events and publishes SYNCHRONIZE-only
reader handles plus a query/synchronize manager-process handle. Drop, explicit
worker Stop and loop teardown signal sticky stop; dead/stale identities cannot
promote. Duplicate and sixth registrations refuse. Public count-only status
cannot label private activity OBSERVE_ONLY: it returns an explicit unavailable
error while leases exist, preserving the four-field schema. Last-private-lease
cleanup ends that worker, while a previously started public observer remains.
Worker ownership/cleanup still uses the same lock, handshake and join owner.

The native Pacer(Input) duplicates reader capabilities, serializes one rendering
consumer and waits on policy change/stop/controller exit with its existing
timer. There is no child policy thread or periodic focus sampler. Render demand
reads the level event; the direct foreground setter refuses for an external
source. Stop/controller exit stays sticky and disables waiting while the DXGI
wrapper continues native forwarding. Partial/pseudo/aliased inputs refuse;
failed constructors and caller handle closure do not leak/invalidate owned waits.
Event type/reset-mode provenance is still a trusted-producer precondition:
a complete untrusted child capability decoder is not implemented.

The typed DXGI owner has one optional initialize(Input), outside loader lock and
before wrapping; a second initializer refuses. D3D12 WARP controls now actually
use native event input, including inherited background policy after recreation,
concurrent calls, external stop and original native error/identity forwarding.
D3D11 retains its direct policy control. The compatibility DLL/creation observer
and storage config ABI are unchanged; timing remains linked only into the
explicit control executable.

Executed: final MSVC x64 Release /W4 /WX build and both timing modes pass.
External-policy promotion measured 0.169 ms on the final native run; own
controller exit interrupts and permanently stops the wait. Thirty-two repeated
invalid/aliased constructor attempts retain the same process handle count.
A negative check discovered INVALID_HANDLE_VALUE is a process pseudo-handle
when duplicated; input now explicitly refuses pseudo-handles.

The initial 80 ms notification check falsely tolerated a missing wake. The
strengthened control checks both notification latency and time since the
previous frame. Removing notification from the consumer wait set then fails
(exit 1); restoring it passes. The five-own-renderer run yields 551 samples
over 3.051 seconds, 0.062 process CPU seconds, 4.442 ms maximum synthetic
promotion and 4.159 ms joined stop. These are permit/own-process measurements,
not displayed/game FPS or resource savings.

Four focus-owner tests pass: five own suspended-process selections, identity/
read-only/stop checks, single-worker registrations, duplicate/sixth refusal,
public Stop, last-lease self-cleanup, and preservation of an existing public
observer. Combined with prior bootstrap tests, the isolated exact-source Rust
harness passes 22 tests with the unrelated caption test filtered as previously
documented. Rustfmt passes; isolated-harness dead-code warnings remain. Full
Tauri/native suite and actual user-focus-driven game acceptance are NOT_EXECUTED.

**Integration boundary:** the Rust producer and native consumer are separately
exercised. Reader handles are not yet delivered through the process-bound
bootstrap capability, and game creation does not initialize/wrap timing.
Do not report an end-to-end Rust-to-game limiter. Reviewed payload pins,
original profiles/accounts/worlds, public admission, CI and distribution remain
unchanged. No Computer Use or game launch/input/restart/closure occurred.

**Additional operator requirement:** add a measured FPS HUD at the top right
inside each Minecraft client area, lightweight and input-safe, anchored through
resize/DPI changes. Its sampling must reuse the presentation owner and clearly
define the observed metric. HUD source, actual on-screen behavior and overhead
remain NOT_IMPLEMENTED / NOT_EXECUTED; this addition does not replace the
five-client, 144/10, persistence, Host World or stability requirements.

### Creation-to-pacing capability integration — 25 October 2026

Base reviewed: `8b6e731f85d8ff2c94f08732df3b3834e273c214` (Local).
The existing presentation enrollment owner now accepts a separate private
120-byte v2 policy capability in exact `pace` mode. Observe v1 (72 bytes),
storage v2 (1,136 bytes), public admission and reviewed game payload pins
stay unchanged. The opt-in C++ runner is its only capability producer so far.

The child checks exact target PID/birth/session, DXGI file/PE metadata and
executable RVAs, four distinct objects, exact read-only access, event
manual/auto/manual reset modes, and a live same-session controller PID/birth.
It removes inheritance. NtQueryObject/NtQueryEvent are resolved dynamically;
unexpected/missing native queries fail closed. NtQueryEvent reset-mode layout
is supported by primary System Informer declarations, not a Microsoft Win32
guarantee. Other Windows-build support remains UNKNOWN. README links sources.

On the first eligible creation, outside loader lock, InitOnce initializes the
existing process pacer, which duplicates wait-only event/controller copies.
The inherited source copies close after initialization. Both creation paths
transfer one caller reference into the typed wrapper; native chain/factory
objects and vtables remain unchanged. Initialization/wrapping failures preserve
native API results and record failure. A replacement chain inherits policy and
sticky stop. No factory/device/timer/thread is created by this integration
under loader lock. Rust reader event rights add EVENT_QUERY_STATE for this
decoder, without EVENT_MODIFY_STATE; bootstrap delivery is still unwired.

### Executed evidence for this source change

MSVC x64 Release /W4 /WX build passes. Ten unchanged compiled C++ inputs were
compared literally (after line-ending normalization) with the exact base.

- `--run-presentation-pacing-self-test`: own parent/hidden child/D3D12 WARP
  checks release intervals at 10/144/10/144, recreation during active policy,
  stop-event and distinct controller normal-exit cases, and sticky bypass on
  another recreation. Three wrappers per case balance to zero live bindings.
  Invalid snapshot sizes/null output refuse. All return native exit 0.
- Thirty-four invalid presentation capabilities refuse, including missing,
  aliased/pseudo/wrong-type handles, wrong reset modes, writable/query-missing
  rights, changed controller PID/birth, signaled stop, reserved bytes and v1
  downgrade. Five storage-invalid cases and malformed mode also refuse.
- The existing observe command passes 19 invalid presentation capabilities,
  40 creation lifetimes, both native creation methods and native factory
  recovery. The paced runner also retains those recreation/storage controls.
- The first policy-only receipt check exposed the storage fixture's round
  semantics: canaries expected a prior saved round. Dedicated fixed child
  arguments now select presentation controls while retaining first-round
  storage canaries; each case resets only its generated marker. No game data
  or acceptance assertion was weakened.
- Negative control removing HWND wrapping builds but the paced test fails
  (exit 2; child fails its pacing assertion). Restoring wrapping builds and
  passes. Final exact-path inventory finds zero remaining own test processes.
- Existing D3D12 typed-wrapper control passes; normal frame control passes
  event promotion (0.011 ms), D3D11 wrapper and five hidden own renderers
  (550 samples/3.056 seconds, 0.062 CPU seconds, max promotion 0.174 ms).
  These clock-release/software-renderer observations are not displayed FPS
  or Minecraft performance.
- Exact-source isolated Rust harness: 22 passed, one caption test filtered,
  zero failed; rustfmt passes. Full Tauri/product build is not claimed.

No Computer Use or game launch/input/restart/closure occurred. Original
accounts/worlds and old candidate remain unchanged; no binaries, credentials,
raw process metadata or worlds are committed. CI and distribution remain off.

### Rust focus lease through the existing bootstrap — 25 October 2026

Base reviewed: `193af7fe2c8c8c7efb4f310b171c52403259a5d5` (Local).
The existing Rust bootstrap owner now provides a private, payload-bound
constructor alongside its existing default preparation. It reuses the storage
configuration's section allocator for an opaque 120-byte presentation record,
inherits only its read-only section, removes ambient presentation settings and
adds the exact pace marker. After exact original process/image/session/storage
validation, it registers that child on the existing single focus worker.

The pinned parent enroller now exports PrepareEducationPresentationV2. Outside
DllMain it revalidates the same original process/thread/birth/storage/image,
resolves native System32 DXGI creation RVAs from a temporary native factory,
and releases that factory normally. It requires the controller to be the
calling parent, duplicates only query/wait event/controller rights into the
unstarted child, refuses a nonempty destination record, rechecks identity and
publishes once. Child-side validation remains authoritative for exact object
access/reset modes and DXGI metadata. Partial transfer failures are owned by
the existing original-unstarted-child cancellation guard.

Parent presentation-section handles and temporary transfer readers close after
construction; the existing worker retains its producer endpoints. PendingEducationBootstrap
retains only the focus lease alongside existing process/storage handles; its
Drop signals policy stop while preserving any resumed process. Existing
registered game routes keep opting out, reviewed game payload pins and
EnrollmentV2/storage/public schemas remain unchanged. The test-only descendant
constructor explicitly carries no policy. No second focus/lifecycle owner,
profile store, public command, game-data copy or third-party dependency is added.

### Executed evidence for the Rust delivery change

- MSVC x64 Release /W4 /WX build passes. An initial header declaration error
  for CreateDXGIFactory2 was corrected by including the existing fixture's
  DXGI 1.6 header before verification.
- New exact-source Rust/C++ test passes with two original own children.
  They validate actual Rust-issued sections/events, wrap native D3D12 WARP
  creation, check 10 FPS release intervals and recreation, signal an existing
  receipt barrier, then verify lease-drop sticky bypass through another
  recreation and exit normally. Parent confirms each child remains live
  after lease release. The last lease's single worker is stopped/joined.
- The harness now mirrors the real engine/native_clients/bootstrap nesting;
  canonical crate::engine::instance_focus_events resolves in both test and
  non-test compilation. Isolated offline non-test cargo check and rustfmt
  pass; all 23 harness tests pass, zero fail, one visible caption test remains
  filtered. This is not a full Tauri/product build.
- Negative control omitting parent capability publication fails the new
  interoperability test (cargo exit 101); restoring publication passes.
- Original C++ bootstrap ordering (including failed attach/owned timeout),
  observe creation/storage control and paced creation/storage control pass.
  The latter retains 34 invalid presentation-capability refusals, five
  storage-invalid refusals, 10/144/10/144 own-parent updates, sticky stop,
  distinct controller normal exit and 40 native creation lifetimes.
- Final exact-path inventory found zero remaining own C++/Rust test processes.

These checks do not change foreground focus or touch Minecraft. The new Rust
receivers are hidden and remain background; actual OS foreground promotion
through Minecraft is NOT_EXECUTED. Prior C++ synthetic transitions are supporting
evidence, not evidence of real game FPS. Native query compatibility outside the
tested Windows build remains UNKNOWN.

### Measured top-right FPS HUD source — 25 October 2026

Base reviewed: `aa6332a91ab2929f3dbd027219968fc0bed63b24` (Local).
The existing typed presentation owner now samples outer native Present/Present1
returns, counting exactly S_OK and excluding TEST/status/failure returns.
A successful DO_NOT_WAIT presentation counts; pending nonblocking work does
not. At least 500 ms of accepted intervals produce a measured rate. Initial
and two-second-stale values show `-- FPS`; it never displays a 144/10 policy
number as a measurement. Out-of-order concurrent completions clamp safely;
current observer timestamps are obtained under the sampler's read lock.
This measures accepted render presentations, not scanout/displayed frames.

The same process-lifetime presentation policy owns one HUD worker, reused
across chain recreation without retaining native renderer references. It
creates an owned, non-topmost layered window, with TRANSPARENT/NOACTIVATE/
TOOLWINDOW styles and transparent hit-test/no-activation responses. White
17 DIP text sits in a dark 128×32 DIP panel, 12 DIP inside the client top-right.
The worker alone performs GDI/Win32 drawing; render callbacks add only a
QPC/short counter update after the native result and preserve HRESULT/LastError.
Geometry follows own-process location events plus a 250 ms fallback while
bound. DPI conversion restores temporary thread context; no process-wide
DPI/timer change, input hook, game backbuffer drawing, account/world I/O or
persistent preference is introduced. Unbound windows are destroyed and the
worker waits indefinitely. The policy keeps its existing process lifetime;
standalone controls stop/join the worker on destruction outside loader lock.

### Executed evidence for the HUD change

- MSVC x64 Release /W4 /WX build passes. The initial control-only auto-declarator
  type mismatch was corrected before successful verification.
- New hidden HUD control passes eight construction/teardown cycles and rebinds
  each same worker, with balanced GDI handles. It checks actual own-window
  resize/owner-relative bounds, styles and message responses; deterministic
  sampler cases cover 144/10/actual-5 FPS, failures, occlusion, TEST, successful
  nonblocking work, initial/stale/reset values and out-of-order completion.
- Geometry vectors cover 96/144/192/288 DPI; raster generation also runs at
  96/192 DPI. Own GDI bitmap examples of 144.0 FPS, 10.0 FPS and unavailable
  values were rendered and visually inspected. No screen capture or Computer
  Use occurred. Actual visible clicking and real monitor-DPI transitions
  remain NOT_EXECUTED.
- Final HUD control: 8 cycles/10 draws over 3.389 seconds, reported 0.0000 CPU
  seconds (including 0.0000 over its 2.1-second idle portion). This accounting
  result does not prove zero overhead. One-million-call microbenchmarks measured
  10.43 ns/counter record and 23.63 ns including QPC/record in that own process;
  these exclude game hook/DWM/GPU cost and are not a Minecraft comparison.
- Existing D3D11 and external-policy D3D12 typed-wrapper controls pass, now
  checking actual native-result sampling and TEST exclusion. Removing native
  sampling makes the D3D12 control fail (exit 1); restoration passes.
- The Rust-owned receiver now uses a larger hidden own client area and checks
  actual HUD window/bounds/bitmap draws with zero HUD failures under the full
  compatibility DLL; malformed snapshot size/null output refuse. All 23
  exact-source isolated Rust tests pass; one visible caption test stays filtered.
- Existing observe/storage and paced creation/storage regressions pass, retaining
  19/34 invalid presentation-capability refusals, 40 creation lifetimes and
  original two-slot persistence/recovery canaries. Five-own-renderer timing
  control passes (550 releases/3.060 seconds, 0.031 CPU seconds, maximum
  promotion 0.138 ms, stop 5.320 ms); these are not displayed game FPS.
- Final exact-path own-process inventory is zero. No binaries, previews, raw
  process metadata, accounts or worlds are committed.

The private HUD snapshot is diagnostic only. Storage/creation capability
layouts, public schemas/admission, existing reviewed live payload hashes,
original account/world data and CI/distribution are unchanged. No game was
launched, closed, focused or controlled in this work.

### Reviewed same-profile pacing/HUD candidate — 25 October 2026

Base reviewed: `75f515c8f28e054eb1e47f877061d8ee1af0e0d1` (Local).
The existing private registered-enrollment route now supplies its pinned
BootstrapLibrary to the existing configured constructor for both replayed
slots. That constructor already owns the read-only presentation capability,
single focus-worker lease and pre-resume delivery. Plain source-only registered
preparation remains unpaced. A presentation request outside reviewed replay
refuses before registered-game or storage lookup; the own-payload regression
asserts that refusal. No new command, provider/store or public admission is added.

The fixed development pins now select the rebuilt pacing/HUD payload pair:
enroller SHA-256
`7b38a1c02f0d5169f74ebebc839d854d49e9a7e77b9933415b1ec8d5d539588a`;
compatibility SHA-256
`76a938f18ec6e1d03ad2e00f0ff37feba41834b2d453c1b612ce9904519c8389`.
Test builds still have no reviewed game/replay policy. Existing registered
image/callsite and 216-byte profile-identity capability pins remain unchanged;
missing/replaced roots refuse without allocating replacement accounts.

Executed: MSVC x64 Release /W4 /WX build, all 23 isolated exact-source Rust
tests (one visible caption test filtered), non-test isolated cargo check and
rustfmt check pass. The preview frontend and complete manager Release build
also pass with the existing distinct preview configuration and no installer.
The complete build reports existing unused/dead-code warnings and a pre-existing
GetSecurityInfo signedness declaration warning; these were not suppressed.
Compiled C++ and Rust/frontend inputs match the pinned source semantically
apart from this bounded Rust change (line-ending differences retained locally).

A separate ignored `acceptance-candidate-20261025-pacing-hud` directory contains
the complete preview manager (SHA-256
`d10334fd4fc74f232f19e02a15f7309ef44546c6957818106327107081acec4d`),
the exact DLL pair and the unchanged replay capability. Its local manifest
binds all bytes and the source blob. Only capability metadata was copied;
account/token/world directories were not copied or read. Every file hash of
the older exercised private-temp candidate was rechecked unchanged.
Executing the new manager with the private flag plus an invalid extra argument
returns exit 2 / InvalidData before lookup or game creation. This is refusal
proof, not game startup or pacing acceptance. No game was launched, closed,
activated or controlled; CI and public schemas/admission remain unchanged.

### Five-process resource collector — 25 October 2026

Base reviewed: `be24595e61f753a2dfedbdd72b462af90d7df748` (Local).
While the operator-session question remains unanswered, source work extends
only the existing manual `collect-native-performance.ps1` and strict Node
reviewer. The staged game's Rust/native payloads are unchanged.

Client05Pid requires slots 03/04 and five distinct selected PIDs. Schema 4
retains separate CPU/RAM metrics and pinned process-creation times for all five
slots; missing/inaccessible/recycled identities stay UNKNOWN through capture.
The optional one-shot GPU counter query covers those same five identities,
is skipped if any timed CPU identity was lost, and rechecks births after the
query. Unrequested GPU evidence is null. Counter values remain explicitly
unverified maximum-instance hints, including the stale-memory caveat; they do
not establish physical VRAM, displayed FPS, full process-tree cost or gameplay.
Existing two-/four-process schema/result shapes are retained. No product
command/admission, lifecycle owner, account/world I/O or CI change is made.

Executed SOURCE_EXECUTION / own-process TARGET_WINDOWS checks:
24 Node tests pass, including fifth-slot metrics, exact schemas, contradictory/
missing/revived identities and strict GPU slot attribution. A PowerShell AST
control executes the actual canonical GPU-reader functions with synthetic
counters, checking five-slot and legacy two-slot shapes, per-instance maximums,
recycled-identity refusal and unavailable counters. Two actual 30-second
captures of disposable Node processes pass in Windows PowerShell 5.1 and
PowerShell 7, producing six samples each and BOM-less reports consumed by the
actual reviewer CLI. The PowerShell 7 fifth process exits normally after eight
seconds: its final five samples stay UNKNOWN, other slots remain observed,
and an instrumented counter verifies zero GPU queries after that loss.
Both actual captures report all GPU hints UNKNOWN; positive GPU values were
checked only with synthetic counters, not the RTX 3070 or physical VRAM.
Six invalid configuration invocations across both shells refuse before output
directory creation. Only owned disposable processes were started/closed;
no Minecraft launch/input, account/world access or Computer Use occurred.

### Actual two-client pacing/HUD and Education world session — 25 October 2026

Game candidate: `be24595e61f753a2dfedbdd72b462af90d7df748`; resource
collector/reviewer source: `e2111e4a23bad4d1bccd131fe57e57f9d7cb56aa`.
The operator explicitly made a session available and authorized opening the
candidate. All four staged manager/DLL/replay capability hashes matched before
launch and after the observations. The existing manager resumed exactly two
original official children with the same reviewed profiles. Both loaded the
exact staged compatibility DLL, remained responding, and received immutable
Minecraft - 1 / Minecraft - 2 captions. Read-only window metadata found one
visible 128x32 HUD window and one visible OGLES game window per process.
No Computer Use, game input automation, token/world-content reads or game
termination occurred.

The operator then confirmed both existing accounts signed in, both games were
in the SAME Education Host World through official host/join, and the visible
FPS counters matched the intended policy. The response did not give numeric
readings or a timed focus sequence. Treat foreground 144 and switching as
operator-reported counter behavior, not independently measured displayed FPS.
Host/join success proves basic multiplayer in this session; long background
simulation/network/tick health remains UNKNOWN.

An initial non-elevated PresentMon attempt refused with access denied (exit 6);
no frame proof is credited to that attempt. The signed Intel PresentMon 2.3.1
binary (SHA-256
`364e5d98d4d134bd54dd25c22ed2ca2f4883f8bc3ed6502bee0c151e3436d30c`)
then ran one UAC-approved 30-second capture, exit 0, restricted to the verified
official game image name with input tracking disabled and a unique session.
Raw CSV stays local (SHA-256
`52a511ce2b8968c0c3f986bbe0d11272cda75662ad9b27a7359b80f4455e0001`).
The existing analyzer accepts both selected processes: each has one chain,
294 positive present intervals, no missing intervals and SyncInterval 1.
Slot 1 mean/p50/p95/p99 intervals: 100.302/100.277/100.550/100.706 ms.
Slot 2: 100.306/100.294/100.554/100.727 ms. These support approximately
10 present calls/second in that recording. Foreground transitions were NOT
time-aligned to this elevated capture, so it does not establish foreground
144 FPS, display scanout timing or focus-switch latency.

A separate initial 30-second CPU/RAM capture passed (six readings per process,
no identity loss). It began before operator world-ready confirmation:
slot 1 mean host CPU 21.45%, mean/max working set 1492.08/1776.68 MiB;
slot 2 20.43%, 1999.21/2164.11 MiB. Do not label this an optimized steady-world
benchmark. NVIDIA's available read-only device query reported whole-device
VRAM 2498-2733 MiB and GPU utilization 4-11% during that initial phase,
including other applications. Windows per-process GPU hints remained UNKNOWN.
Per-client physical VRAM attribution and controlled baseline comparisons remain
unproved. Raw process/counter/window artifacts remain local and uncommitted.

At the operator's request to conclude, no further test or game operation was
started. Both games were left responding/open. The local candidate manifest
now records partial two-client runtime acceptance rather than NOT_EXECUTED;
five-client readiness is not awarded.

### Two-client memory-efficiency research — 25 October 2026

The operator now prioritizes RAM/performance research and discussion BEFORE
implementing clients 03-05. This supersedes the previous expansion next action;
the full five-client 144/10 goal and preservation boundaries remain intact.
The operator further distinguishes game-setting tradeoffs from efficiency of
our original system, using Sodium as inspiration. Credit system gains only
under equivalent graphics, policy and workload; report settings gains
separately. Java renderer/data optimizations are architectural references,
not a proven native Education integration or permission to port a mod.

[Research findings](../docs/system/native-memory-efficiency-research.md) pin
this review to `5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5` and cite primary
Minecraft Education, Microsoft, NVIDIA and CEF sources. The current pacer
changes present frequency without releasing game content. The existing
collector measures selected-process CPU/working set but not private commit,
private working set, host commitment or attributable helper totals. Source
inspection finds bounded own HUD/journal buffers and a CLI route before
Tauri UI bootstrap; no GiB-scale source cause or game leak is established.

The first engineering task is composition/overhead measurement at unchanged
settings. The report separates engine-only, settings-only and combined
comparisons. A fixed Education graphics/render-distance preset is an optional
parallel candidate, with its visual/workload tradeoff reported explicitly. Live focus-driven Education graphics changes remain unverified.
Periodic trimming, arbitrary memory/CPU caps, whole-client suspension, helper
killing or global memory tweaks are rejected as default optimization paths.
The report defines a bounded metadata-only comparison with time-aligned focus,
frame tails and host health; no numeric savings or five-client capacity is
promised.

Proof: exact-source owner/consumer review plus primary documentation research
and Markdown/source-link/scope checks only. No new game/process observation,
benchmark, source-code implementation, setting change, memory dump, account/
world/cache read, Computer Use, CI or game closure was performed. All proposed
optimization gains and current memory composition remain UNKNOWN /
NOT_EXECUTED. The earlier two-client runtime result remains unchanged.

### First memory-composition increment — 25 October 2026

Source authority for this change: exact Local parent
`02bc701312e8ec641759fbafd95cfe6e76404c95`. The existing diagnostic collector
now has an opt-in two-client `-MemoryComposition` mode, with schema 5 retaining
base schema 1/2. It records private commit, supported EX2 private working set,
whole-system physical/commit counters, explicit metric availability/reasons and
memory-query duration. Schemas 1-4 and default extended-query behavior remain
unchanged. This is a measurement implementation, not an optimization or
public product API change.

The native query opens only query-limited handles, checks creation identity
before/after reading statistics, falls back to base commit when the extended
query fails, and closes its handles. The existing Read-Target now also disposes
its own System.Diagnostics.Process object after each reading rather than
leaving handle release to collection. Lost selected identities remain sticky;
unavailable private metrics never become zero. The existing offline evaluator
strictly validates the extension and reports coverage, mean/median/peak and
endpoint values. Paired differences require matching existing conditions and
complete individual metrics, and remain descriptive, without a savings verdict.

Proof: 29 Node tests passed, including native own-process/system reader checks
under Windows PowerShell 5.1 and PowerShell 7, wrong-creation-time rejection,
unavailable-API null handling and handle-retention checks across 200 reads per
shell. Two actual 30-second collector runs with disposable Node targets passed,
six samples each: PowerShell 5.1 retained both identities; PowerShell 7 included
normal exit of the second target and permanent identity loss. Both BOM-less
outputs passed the real Node reviewer. The older-OS EX2 fallback was reviewed
in source, not executed on an older unsupported Windows build. These are own-data collector checks,
not Minecraft runtime, host-world health or RAM savings. Raw captures and
fixture runners remain local, outside Git.

Boundaries: helpers/process trees and manager memory remain explicitly
NOT_COLLECTED; per-process memory does not isolate the compatibility DLL's own
allocations. Memory-query timing excludes setup and is not total telemetry
CPU/RAM cost. No game observation/control, Computer Use, account/world reads,
graphics/FPS/scheduling changes, production admission or CI changes occurred.

### Manager and direct-child memory attribution — 25 October 2026

Source authority: exact Local parent
`9994b88c4a5278c02f1061073e096603b31a0703`. The same collector/evaluator now
support opt-in schema 6: `-ManagerPid` adds a separately pinned selected manager,
and `-DirectChildren` adds per-root direct-child relation counts and private
memory sums. Both require MemoryComposition; invalid/overlapping selections
refuse before creating output. Schemas 1-5 and existing positional parameters
are preserved. No second measurement service, launcher or resource governor
was added.

The child adapter takes process-only Toolhelp snapshots, pins candidate births
before a second relation snapshot, checks child/root identities and rejects
children older than their alleged parent. Selected game/manager roots and the
collector are excluded from child totals. Failed snapshots, late/unpinned
children, unavailable roots or more than 64 candidates yield UNKNOWN rather
than a partial sum or invented zero. A verified empty inventory is zero.
Manager identity loss remains sticky. Query duration includes related queries,
but is not total collector CPU/RAM overhead.

These are observed direct parent relationships, not exclusive game/helper
ownership, authentication attribution or a complete process tree. Shared
brokers, background services and more distant descendants remain unassigned/
UNKNOWN. Process memory still cannot isolate compatibility-DLL allocations
inside the game's heap. Manager application role is operator-selected,
not machine-attested. All savings claims remain unproved.

Proof: 35 Node tests passed, including eight deterministic attribution cases in
Windows PowerShell 5.1 and PowerShell 7, invalid-option early refusals,
manager loss, coverage/null rules and conservative descriptive comparisons.
Actual 30-second own-process-tree captures passed in both shells, six samples
each; the PowerShell 7 case includes a normally exiting selected manager.
Independent Win32_Process parent metadata matched the direct-child inventory
and excluded grandchildren. The first fixture expectation incorrectly assumed
one child; independent inspection identified a real conhost.exe plus node.exe,
so fixture expectations were corrected against Windows metadata. Final-source
PowerShell 7 capture passed actual Node single/paired review. These are
collector tests, not Minecraft performance. Raw captures/fixture runners stay
local and uncommitted.

### First live memory baseline and output-path repair — 25 October 2026

**Exact sources:** sampler/evaluator at
`106117aa6b5851968fd3e7f69f5f5d585a61709a`; unchanged private game payload at
`be24595e61f753a2dfedbdd72b462af90d7df748`. Payload hashes and pinned live
identities were checked locally. At the operator's request the existing private
candidate was opened once through its observation CLI, without enabling public
Launch. The operator then reported both clients in the same Education Host World,
with **Minecraft - 2 as host**. Roles are operator-reported, not inferred from
foreground focus. No settings, pacing policy, account data or world content were
read or changed by collection.

The first 900-second attempt failed to publish its report under Windows
PowerShell 5.1: the nested staging pathname was 273 characters and File.Open
reported DirectoryNotFound although the directory existed. That attempt is
**NOT_PERSISTED**, with no performance metrics credited. A repeat to a short
workspace path completed without reopening games: 900.01 seconds, 170 samples,
no selected game/manager identity loss. Raw report SHA-256:
`42708180a5e9be26d2ebdcfdb7d91acc558df8f9107d66c98980ae5212e05e50`.
Raw process metadata stays local, outside Git.

The first five minutes are warmup. Steady review includes 113 complete intervals
inside 303.55–900.01 seconds; the interval crossing the boundary is excluded.
The original full report passed the existing evaluator before phase selection,
and phase statistics reuse that evaluator. The query cost extends requested
five-second spacing, so sample count is not an assumed 180.

| Steady metric (MiB except CPU) | Minecraft - 1, guest | Minecraft - 2, host | Candidate manager |
| --- | ---: | ---: | ---: |
| Mean private resident | 1291.53 | 1467.48 | 1.46 |
| Peak private resident | 1291.77 | 1469.99 | 1.46 |
| Resident endpoint change | -0.13 | +6.23 | 0.00 |
| Mean private commitment | 2488.80 | 2684.48 | 2.32 |
| Peak private commitment | 2490.70 | 2686.95 | 2.32 |
| Commitment endpoint change | -0.16 | +4.76 | 0.00 |
| Mean CPU, percent of whole host | 0.97 | 1.55 | 0.00 rounded |

Direct-child private resident means were 114.59 and 113.61 MiB; corresponding
commit means 151.31 and 150.81 MiB. Guest coverage was 113/113; host coverage
112/113 with one PARTIAL_RELATION kept UNKNOWN. Manager's verified direct-child
inventory was empty. These are direct relations only, not exclusive ownership,
all descendants or shared-broker attribution. In-process compatibility-DLL
allocations remain inseparable from game totals in this capture.

Mean/peak memory-query duration was 275.85/1072.25 ms. This is measurable
diagnostic cost, not total collector CPU/RAM or game overhead. System available
RAM rose about 4517 MiB within steady observation while system commit fell
about 8069 MiB; uncontrolled other-PC activity prevents crediting either change
to the engine. No graphics or matched candidate was changed, so this is a
composition baseline, **not an optimization A/B result**.

Inference: the current CLI manager is not a GiB-scale resident-memory target.
The host's roughly 6 MiB resident increase over ten minutes alone establishes
neither a leak nor long-term stability. Private commitment is materially larger
than private residency; reducing the latter by paging would not demonstrate
less game allocation. No observed gameplay was supplied: the operator explicitly
did not watch it during recording. Background host progression, connection
health, frame tails, focus recovery, GPU/VRAM and sustained active 144 FPS remain
UNKNOWN / NOT_EXECUTED for this session. Process survival is not gameplay proof.

**Whole-PC scope:** the operator permits normal closure of optional applications,
with Minecraft and Codex retained. Discovery must preserve Codex even though its
executable/display name is ChatGPT, plus game/authentication/tool dependencies.
A later metadata snapshot found optional high-memory applications already absent
before an agent exit action. No normal-closure saving is claimed. Available RAM
was about 16204 MiB of 32524 MiB at 07:33 UTC; game private commitment remained
about 5184 MiB combined despite lower instantaneous residency. This uncontrolled
snapshot is PC context, not matched engine savings. No process was force-killed,
cache deleted, account reset, world changed, service disabled or memory trimmed.

**Bounded repair:** the existing collector now tests its actual exclusive
write/flush/non-replacing-move path with one owned disposable probe before
starting timed sampling. The final writer uses that same function; owned
staging is cleaned on failure and existing reports are preserved. Output
capability failure asks for a writable shorter filesystem path immediately.
This does not enable Windows long paths or guarantee later disk capacity.
Report schema, public APIs, launch admission and game behavior are unchanged.

Proof for this repair: 36 Node tests passed, including collision preservation
and capability-dependent long-path early refusal in Windows PowerShell 5.1 and
PowerShell 7. An actual 30-second final-source own-process-tree capture under
Windows PowerShell 5.1 produced six samples, retained identities, and passed
single/paired Node review. These own-data checks are separate from the real-game
baseline at 106117a. Raw captures, fixtures and PC inventory stay uncommitted.

### First owned HUD allocation reduction — 25 October 2026

Source parent: `d9e8d60d147d6d104d33255fa4a2b9e752a584f9`.
The allocation-lifetime audit followed the existing compatibility hooks,
configuration/journal mappings, process presentation policy, swap-chain wrapper,
pacer and HUD. The reviewed routing hooks use bounded stack/static buffers;
configuration/journal views unmap through their existing owners, and final
chain release drops native COM interfaces. The process policy intentionally
retains one pacer/HUD worker and synchronization handles. No unbounded
game-content cache or GiB-scale owner was demonstrated. This is source review,
not exhaustive heap attribution, absence-of-leak proof or a ceiling on all
Detours/OS/game allocations.

A concrete duplicate-work cause exists in fps-hud.cpp: the worker rasterized a
DIB, copied its pixels into a vector, then allocated a second DIB and copied
the vector back for UpdateLayeredWindow. It now paints directly into the upload
DIB through one private shared painter. The preview raster function retains its
existing interface and uses that painter too. No new cache, process, observer,
governor, public API or persistent drawing resource was added. Pacing/focus,
HUD format and visibility, game settings and isolation paths are unchanged.

Per redraw, this eliminates one bitmap creation, one vector allocation and two
pixel copies. Peak explicitly owned pixel storage drops from two planes to one:
32 to 16 KiB at 96 DPI, or 800 to 400 KiB at the supported 480-DPI maximum.
Those are source-derived transient-buffer sizes, not whole-process residency,
commit savings or a bound on GDI/DWM/allocator overhead. The baseline game RAM
is orders of magnitude larger; do not advertise this as a GiB-scale fix.

Proof: exact-source MSVC Release /W4 /WX builds passed for bootstrap-compat,
fps-hud-control and frame-pacing-control. The extended HUD control passed eight
hidden-window bind/unbind cycles, 120 rasters across six DPI values (48–480),
invalid-DPI rejection, opaque output and balanced GDI handles. A disposable
API-counting probe drove the actual worker upload calls: old code failed the
one-DIB assertion for all 18 uploads; changed code passed all 18.
Old/new preview BMPs were byte-identical, SHA-256
`f9f13416d19765a374ec7f7c3aca413a625aa80b573a995be9b45c646c9db3ee`.
D3D11 and D3D12 WARP controls passed native-result/LastError preservation,
TEST exclusions, concurrent calls, resize and balanced chain recreation.
Five hidden fixture renderers are not five Minecraft clients. Probe artifacts,
builds and previews remain local and uncommitted.

No CPU speedup is claimed from short controls; changing coverage and host noise
prevent such an inference. Existing live games retained their original identities
and responded at the final read-only check. They were not reloaded with this DLL,
closed, controlled or reconfigured. The new payload's Minecraft behavior and
matched RAM/CPU/GPU benefit are **NOT_EXECUTED**. Previous game acceptance remains
bound to the old be24595 payload; no launch gate or CI change occurred.

### Updated HUD candidate integration and operator correction — 25 October 2026

Source parent `506aef24b002204b6670d37280da0cadc24da6e9`.
The existing independent_bootstrap.rs reviewed compatibility digest now pins
the built HUD DLL at SHA-256
`09d214cc48dfb5fac80c7ba15576582dddd49e2196ab694b4820cfbe02878813`.
Enroller and 216-byte profile-identity capability digests remain unchanged.
No hash override, new public launch route, copied account/world tree or provider
admission was added.

A separate candidate was built from the pinned source plus this digest update,
using Rust 1.99.1, locked offline dependencies and the existing distinct preview
configuration. Vite frontend and release executable builds passed. Rust's
registered_ filter passed four tests; one actual-installed-game discovery test
was ignored. Test compilation retains empty reviewed game/replay policies.
The touched Rust file passes rustfmt with edition 2021/skip_children. Whole-crate
format checking reports pre-existing differences in untouched files and is
not reported as passing. Initial sparse checkout lacked embedded script inputs;
adding those unchanged source directories resolved the build errors. Existing
compiler warnings were not silently treated as a clean warning-free build.

Staged manager SHA-256:
`1957dfabfaba5a61d3c1339f4e7c537416db6f7e7fad9f8647da54ac079ee4ed`.
The invalid-extra-argument smoke returned exit 2/InvalidData with no games
running. After the operator confirmed both games normally saved/closed, one
authorized startup opened Minecraft - 1 and Minecraft - 2. Loaded module
metadata matched the new DLL in both games and the unchanged enroller in the
manager. The operator confirmed both previous accounts automatically signed in,
FPS counters appeared, and both clients joined the same Education Host World
with Minecraft - 2 hosting. This confirms setup/replay on the new candidate,
not measured active 144 FPS, sustained host health or five-client capacity.

A follow-up RAM sampler was started, then explicitly stopped when the operator
corrected the work's focus. It published no final capture; its numeric
comparison is NOT_EXECUTED / NOT_PERSISTED. Do not use transient readings as
savings evidence. The separate PresentMon attempt failed with access denied
and produced no valid frame trace; no UAC, security/group change or input
automation was used. The collector stop left both original game processes
responding. All private logs/capability/builds stay local, outside Git.

The operator correctly identified that further recording and tiny wrapper
changes had displaced the intended Minecraft efficiency outcome. No further
hook micro-optimization was implemented. The existing 15-minute baseline is
sufficient to distinguish the small manager from game allocations. Official
Education performance guidance supports reducing render distance and decorative
effects, with visual/workload costs reported separately; it supplies no
guaranteed RAM reduction. An Education-compatible Sodium-style internal
optimizer has not been established. The operator subsequently allowed reduced effects/graphics while requiring
render distance at least 12 and simulation distance unchanged. The concrete
Minecraft-level intervention and integration findings follow below.

### Approved Minecraft graphics intervention and native-mod feasibility

The user explicitly redirected the goal toward Minecraft's own resource demand,
rather than continued engine telemetry or tiny wrapper changes. Render distance
must be at least 12, simulation distance unchanged; graphics/effects may be
reduced. Prior fixed-graphics constraints are superseded for this separate
settings track. This does not authorize treating visual reductions as engine
code savings or changing world simulation.

Using the operator's later full PC-control authorization, the installed Education
26.30.2 settings UI was inspected and changed in both current clients:

| Setting | Observed before, both clients | Applied, both clients |
| --- | --- | --- |
| Graphics Mode | Fancy | Simple |
| Render Distance | 12 chunks | 12 chunks, untouched |
| Fancy Leaves / Fancy Bubbles | On | Off |
| Render Clouds / Beautiful Skies / Smooth Lighting | On | Off |
| Anti-Aliasing | 2 | 1 |

Each value was confirmed in the live UI, then settings were exited normally.
Resolution, simulation/world settings, Code Builder, profiles, accounts and
144/10 code/policy were not modified. No option-file guesses, package installs,
registry tweaks, global memory trims or game-resource deletion were used.
No further recorded performance session was started. Both clients were observed
at the main menu during settings work; earlier operator-reported same-world
readiness does not establish continued world residency. Menu RAM therefore must
not be compared to the old world baseline. Current gameplay benefit and
restart-persisted graphics remain NOT_EXECUTED.

The [extended existing research report](../docs/system/native-memory-efficiency-research.md#minecraft-internal-efficiency-feasibility-sodium-principles-versus-education-integration)
now pins actual Sodium/FerriteCore source and official Education support.
Sodium's compact vertices, terrain arena reuse/compaction, culling and bounded
chunk-copy caches are real mechanisms. Their Education-native ownership and
lifetime integration seam remains UNKNOWN. The current Present adapter does
not own meshes or game heaps. Education officially does not support community
Java-style mods; official content Add-Ons do not prove a renderer SDK.
Pooling alone can retain more memory; do not construct another generic
allocator/hook fixture as a substitute for a real integration seam.
Code Builder has an official feature toggle, but helper teardown/RAM release
is not documented and no such feature change was applied.

### Remaining proof and one next action

The full five-client goal is incomplete. Two original accounts, basic same-world
Education multiplayer, visible HUD and intended FPS counter behavior are now
operator-confirmed for the new candidate. Native present intervals independently
support approximately 10/second during the captured interval. Five slot
bootstrap, independent active-144/transition timing, sustained host
health at background rendering rates, five-client CPU/RAM/GPU/VRAM comparisons,
restart persistence on this exact new candidate and soak/crash recovery remain
NOT_IMPLEMENTED / NOT_EXECUTED / UNKNOWN as applicable. The earlier private-temp
two-account/world restart proof remains historical evidence for its own payload.
Public production admission and CI stay disabled.

**One next action:** investigate one concrete Education-native renderer resource
ownership/lifetime seam for B using existing source and supported interfaces;
either identify a named integration with safe allocation/retirement semantics
or close that specific candidate as unsupported/UNKNOWN. Do not start another
collector, generic allocator fixture, wrapper-only optimization or automatic
minimization rollout. Code Builder is now saved off only in the existing host
world; its resource benefit is unproved. A/B/C remain the active goal.
