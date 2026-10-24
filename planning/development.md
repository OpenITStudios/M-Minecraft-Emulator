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

**Operator direction — 24 October 2026:** continue to the actual two-client goal in complete integration batches. Original code only; Sandboxie is a behavioral reference. The operator authorized the private compatibility-DLL/game experiment and manual distinct-account sign-in. Public admission remains denied; no driver/service, legacy runtime, CI or main change is authorized by this experiment.

**Source baseline:** exact remote Local `7d11554256973a52007235ac1374a617d5666a45`, plus the input blobs recorded below. The sole native owner is `apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`, consumed only by the fixed existing manager startup argument. Existing C++ config/enroller/compatibility code in `tooling/acceptance/windows/bootstrap-ordering/` implements the selected storage/HKCU/IPC experiment. No public provider readiness is awarded.

### Observed result and current blocker

- Focused transparent startup observation verified a main-image CreateMutexA return RVA `0x2cdc30`, a GUID-shaped name and actual session BaseNamedObjects parent identity. The process that exited saw Win32 ERROR_ALREADY_EXISTS (183); the survivor saw 0. Native informational STATUS_OBJECT_NAME_EXISTS is not itself a failed API call. Application branch disassembly remains UNKNOWN.
- The scoped original translation uses an exact official-image SHA-256, that caller RVA and one process-local bound name. Native create/open maps only this selected name in the verified directory to a stable per-store suffix. Results are actual OS results; unrelated objects are unchanged.
- With this translation, **two responsive official Education windows remained active**. The operator explicitly confirmed **both windows successfully signed in manually using different accounts**. This establishes that limited observed milestone, not complete authentication containment or a production provider.
- **Both clients then failed to create a world**, showing “You are almost out of data storage space”. The screenshot and operator report establish the symptom; gameplay/world persistence has not passed.
- Both games closed normally; the manager retained original child handles and saved both startup journals, with child exit codes 0 and manager exit 0. Both translated native mutex creates had real status 0.
- A verified reopen of the SAME private directory/registry pair succeeded, but the operator reported **both accounts required sign-in again**. Login persistence therefore failed this run. Do not report two persistent accounts.
- The v3 storage diagnostic replay found hundreds of rejected/missing operations in both processes: success/path/access/parent/backing counts `[3,138,30,141,78]` and `[3,136,30,141,78]`. First path sample: directory create, relative length 35, STATUS_NOT_SUPPORTED. The metadata contains no filename, so no exact game-call attribution is claimed.
- A concrete source defect was found and corrected: path validation rejected spaces, including the documented Desktop data directory `Minecraft Education Edition`. The existing two-round own-process private-file control now uses that exact directory shape and persisted marker. It failed with exit 2 before correction and passed with exit 0 after allowing interior spaces. The later consolidated batch also supports ordinary Unicode/base64/dot-prefixed names. Traversal, ADS, ambiguous trailing spaces/dots, bounded depth/length and reparse restrictions remain.
- **Latest game outcome: FAILED.** The operator reported both clients crashed when creating a new world on the space-only correction. Both original process exit codes were 3221225477 (0xC0000005). Windows Application Error metadata independently recorded the same game-module offset 0x245bf61 in both crashes. No crash dump, account cache or world content was read. Storage counts were [86,93,41,168,136] and [90,93,49,168,136]. This confirms continued failures, not the causal crash instruction. The runner incorrectly returned 0 because it counted journal saving only; the new source now returns failure for any nonzero child exit and retains both exit outcomes.

**Operator correction: stop piecemeal manual testing.** No further game launch or login request was made after this instruction. Work continued autonomously in the existing two-profile integration control. Source found two more concrete gaps: attribute-by-name queries bypassed the redirected open calls, and ordinary DELETE/rename/delete-on-close workflows were rejected. The current batch adds native attribute/full-attribute routing, per-private-handle rename mediation, ordinary private deletion, safe filename components including leading dots/base64 punctuation/Unicode, and coherent synthesized create/open error outputs. Rename/link/short-name/case-mode escape operations and profile-root deletion remain denied; unsupported namespace/bypass coverage remains UNKNOWN.

The existing configured two-slot/two-round control now exercises the official data-directory shape, world-like nested database paths, dot-prefixed temporary files, Unicode names, GENERIC_ALL on private data, flush/atomic replace, native relative rename, metadata matching, free-space query, deletion/delete-on-close, same-profile reopen and 64 replacement cycles. Its unhooked controller checks each backing marker and an unchanged host tree. Rename/hard-link escape attempts and profile-root deletion are rejected. The complete control passes. This is generated canary data, not a real Education world/database or proof that the Minecraft crash is fixed. The latest full correction has NOT_EXECUTED game acceptance.

Drive C:, which hosts the current pair, had approximately 6.5–7.6 GiB available during inspection. Capacity alone has not been established as the cause. Do not delete worlds, caches, accounts or prior pilot roots, falsify free-space reporting, migrate profiles or copy tokens to remove the symptom.

### Source boundaries and executed proof

The registered official image is bound to SHA-256 `04164c1c64458a5d984dbc1687d0e8023d1808c44ee7be03519cdf8355233eaa`. An update refuses before store access. Private config v2 remains 1,136 bytes; journal v3 is 31,968 bytes with 64 startup records of 496 bytes and 160 bytes of fixed storage counters/first numeric failure samples. Startup recording is still 30 seconds/64 entries; storage counts span the process lifetime. Journals are read only after the original writer exits. No credentials, token/cache/world contents, storage paths or call stacks are collected.

The same owner reopens one explicitly reviewed experimental pair via a hash-pinned 216-byte local capability: bounded root basename and root/MC01/MC02 directory IDs, ordinary pinned ancestors, non-overlapping logical/private roots, existing backing keys and existing Roaming/Local directories. Missing or substituted stores fail without creating a new profile. No “latest root” discovery, token/world copying or second production state store. Raw capability, directory IDs and journals stay local; tests keep real-game policies disabled.

Actually executed on the current code inputs:
- MSVC 19.44 x64 Release /W4 /WX build PASS.
- All four C++ control modes exit 0, including both-round storage/registry controls.
- Rust 1.98.1 actual Tauri binary: **14 bootstrap tests PASS**, including the six explicitly enabled own-executable integration tests. Scope selection checks actual host/private object distinction and consistent reopen; replay rejects bad hash/directory identity/missing stores; partial-resume lifetime and journal overwrite rejection remain covered.
- Preview manager build PASS; no installer/package/distribution run.
- `git diff --check` PASS. Source/blob binding and final Local SHA are recorded with the delivery commit.

Executed original two-login artifacts:
- Manager SHA-256 `a82376e52737ad61507c2b4063edc7bb056ad8169f9cd00741058ea51218a73e`.
- Enroller `62bd34fc1add9704f1a0723b84b1d7ffb447380d20cdc3abf32a28994471180e`.
- Compatibility DLL `1ff0e3596763f947c7eadb98df94258bd9d9e260cab88997578b811fcfb5c0d2`.

Executed storage-diagnostic replay artifacts (before space correction):
- Manager `784ffad4a8b8520192e9e6feea1fbdb307aa3fa9779d9d5983f999269a160cb4`.
- Enroller `18f4157b524f142f2bac85efd6d95b5df69d161a1e1a0da6c10788c94868f384`.
- Compatibility DLL `192584862a584c7ee7cd881de7c4c54bf68444fcedf3e746a71e7dbd20262c36`.

Failed space-only correction artifacts:
- Manager `b3df7fe3ec0db4abe85ab3ceb5e29fca1912707ab091dcf8f990c4df0ea5c2c8`.
- Same diagnostic enroller as above.
- Compatibility DLL `55bdd46ca90f5504c7db8516c284d709fa0436b367bd96cadb39b748cd032d97`.

**Proof ceiling:** actual Windows compilation/tests and limited MINECRAFT_RUNTIME two-window/manual distinct-login evidence at the original artifact snapshot. **FAILED:** world creation and initial login persistence. **NOT_EXECUTED:** game acceptance of the consolidated storage correction. **UNKNOWN:** complete helper/auth/HKCU containment, all required file-operation coverage and host/peer protection. **NOT_EXECUTED:** successful two persistent worlds/gameplay acceptance, multiplayer, four clients, comparative performance, installer/distribution. Name translation is not a security boundary; prior own controls explicitly showed raw peer access. A sign-in success does not erase that limit.

Consolidated storage-correction build (not launched into Minecraft):
- Manager SHA-256 `7a8e3de13b8470a6787eabaab54283ae2fa88f3da017628718b7dfa59c04306c`.
- Compatibility DLL `b3046fe7f8344f43bdbb1c34079cb7120647518979df97bc5c319ddd96866423`.
- Enroller remains `18f4157b524f142f2bac85efd6d95b5df69d161a1e1a0da6c10788c94868f384`.


Executed code-input blobs (literal file bytes; documentation is excluded):

| Input | Git blob |
| --- | --- |
| `independent_bootstrap.rs` | `877acb23733eb51c912f08232688299494c0866e` |
| `compat.cpp` | `692a2be511ebedd6541d404ae5df187c03ce27fc` |
| `education-config.h` | `b72a2682e1e1ae6e923c6d9b46ef3f2aac7ec52d` |
| `enroll.cpp` | `856c6af7728d0c327b633eeca34a3d723c5f029c` |
| `probe.cpp` | `4fa13d1d808a21469a64361d9e0e300aa66dc21a` |
| `startup-observation.h` | `db76edfa1af742442b00c62a2acbedb240c7b1cd` |


**One next action:** one consolidated two-client acceptance on the corrected build covering distinct manual login, world creation, concurrent gameplay, normal close and same-store account/world reopen. Do not ask for more incremental click/login tests, reset the stores, or promote the passing canary workflow to game acceptance. The observed crash cause and full auth/helper containment remain UNKNOWN until actual evidence resolves them; public MC02–04 Launch stays closed.
