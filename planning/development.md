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

**Operator direction — 24 October 2026:** mature the original MC01/MC02 engine,
starting with storage correctness, stable profile reuse and failure handling.
**No Computer Use or game-window control while the operator is working.**
No fresh game launch/closure or login request is part of the current source batch.
Public MC02–04 admission remains denied; CI and retired providers remain off.

**Source baseline:** exact remote Local `15e7c6a56ce5260e27d96dd5bc86397ab5ed62a0`,
plus the literal input blobs below. Native coordination remains in
`apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`; its only experiment
consumer is the existing fixed startup argument. The existing C++ compatibility
owner and control are in `tooling/acceptance/windows/bootstrap-ordering/`.
No public API, production profile catalog, driver or service is introduced.

### Confirmed defects and corrections

- File routing rejected spaces in ordinary data paths, then nested directory
  enumeration with a final separator. Attribute queries and ordinary private
  delete/rename operations also had inconsistent coverage. Shared native routing
  now supports these operations, pinned private parents and actual OS results.
  Traversal/ADS/reparse restrictions and profile-root protection remain.
- Explicit WRITE_OWNER/WRITE_DAC opens on ordinary private children were refused.
  They now reach the selected actual private object; root protection remains.
  Nested enumeration and security-open controls failed before these corrections
  and passed afterward. They do not identify the exact old crash instruction.
- The former fixed journal filename caused a second successful run to report
  failure when saving over an earlier journal. The owner now uses
  `startup-observation-v3-{process-birth:016x}.bin` with create-new semantics.
  Two rounds preserve four distinct byte-verified journals; saving the same
  lifetime twice still refuses. No earlier artifact is overwritten or deleted.
- Nonzero child exits remain failures even when journals save. The original
  handles and exit outcomes are retained; dropping a resumed transaction does
  not kill attempted children. A fixture exit 86 still makes the runner fail.
- Numbered captions use immutable store slots and verified original-child window
  ownership, with a bounded message wait. Only a unique visible unowned root is
  labelled; ambiguous windows/popups are skipped. Actual Minecraft - 1/2 titles
  were observed. This does not enable MC03/MC04.

### Actual game evidence, separate from controls

The earlier space-only build failed world creation in both clients with
0xC0000005 and matching Windows-reported game-module offset 0x245bf61.
A later pre-enumeration run recorded MC01 0xC0000005 and MC02 exit 0.
These are historical failures; their exact causal instruction/call chain remains
**UNKNOWN**. Capacity alone was not established as the cause. No raw crash dump,
account cache, token or world content was inspected/copied.

The corrected compatibility DLL subsequently reopened the SAME two profiles
with distinct player identities automatically at their main menus, without
password entry or authentication-dialog automation. Both were playable; the
operator confirmed movement in each. The agent observed two players together
in one world and normal guest departure while the host survived.

Then two new distinct agent-created worlds, `UJI-MC01-Persistensi` and
`UJI-MC02-Persistensi`, reached concurrent gameplay. In-game commands placed a
gold block under MC01 and a diamond block under MC02, each reporting Block
placed. Both worlds saved through normal Save & Exit. MC02's My Worlds showed
its own test world and the preserved old world, not MC01's new test world.
Both applications subsequently closed through their normal confirmation UI:
**both original child exit codes 0, both journals saved, manager exit 0**.
This is creation/play/save/clean-close evidence, not post-restart durability.

A fresh same-store pair was launched for restart validation before the operator
stopped Computer Use. Its logs confirmed both resume calls only; reopened
accounts/worlds/markers were **NOT_VERIFIED**. Do not repeat or close this run
without checking its actual current lifetime and operator intent. The operator
has now explicitly requested source/automatic controls without window control.

Game-tested manager SHA-256:
`eaa65ac1997bf6227b374bd2dce893346d7258c3fced46ed4973ac966d059687`.
New journal-naming manager (built; game restart acceptance NOT_VERIFIED):
`af61d047772dff5bbe6cf8da61201f4857a07cf08c4d300ab340074b85c13730`.
Final source-only rebuild (not launched into Minecraft):
`8366bdaa916a099098473f4378853b5aa6d501beb3bbf3dd2a258eb1a51a46a5`.
All three use compatibility DLL
`a59fb1803ff3d957b9691017f24dd7211a9b3f88b6a7b13988432210d67e02ec`
and enroller
`18f4157b524f142f2bac85efd6d95b5df69d161a1e1a0da6c10788c94868f384`.
Official game image is pinned to
`04164c1c64458a5d984dbc1687d0e8023d1808c44ee7be03519cdf8355233eaa`;
image changes refuse before store access. These artifacts are not installers.

### Current automatic storage and recovery checks

The existing two-profile/two-round configured control uses generated canaries,
not actual world databases. It checks nested official directory shape, Unicode
and temporary names, native relative rename, metadata parity, flush, 64 repeated
replacement cycles, deletion, same-store reopen, distinct backing data and an
unchanged host tree. Escape rename/link and root destruction remain refused.

New failure-path checks keep a committed file open without SHARE_DELETE:
replacement fails with the same ERROR_ACCESS_DENIED as an independently executed
unhooked Windows baseline; old and retry data remain exact. After releasing the
reader, replacement and flush succeed and the retry source disappears.
CREATE_NEW collision also preserves the old bytes. Both slots/rounds pass.
This checks real Windows file semantics through mediation, not LevelDB crash
recovery, disk-full/power-loss durability or every game workload.

**Interrupted-writer batch:** after the two normal rounds, two own children
flush distinct generated checkpoints and wait on controller events. The parent
terminates only the original first self-child handle (known exit 91) before
checkpoint close/replacement. It independently verifies exact old and retry
bytes in the private backing tree and opens the retry file exclusively, proving
release of the dead writer's handle. The other child remains live, resumes and
completes its own replacement. A new enrolled first child reopens the SAME
profile and pending data, completes replacement and exits 0; the peer also exits
0. Parent checks both distinct final markers, removed retry names, preserved
prior CURRENT files and unchanged host data. Generated registry canaries alone
are reseeded to their existing startup baseline; no game store is read/reset.

The new configured case passed eight repeated runs. The other three C++ modes
and all 15 Rust bootstrap tests passed against the rebuilt own control. The
compatibility DLL and manager source are unchanged from the baseline commit;
this batch found no new production defect and adds regression evidence only.
No game process was launched, activated or terminated. This covers interruption
after an acknowledged flush and before replacement, not power loss, torn writes,
disk-full, post-replacement interruption or LevelDB/game crash recovery.

The replay owner opens only one hash-pinned, local-only capability's existing
root/MC01/MC02 directory identities and backing registry keys. Wrong capability,
replaced directory, missing Local directory, overlapping roots and unreviewed
payload are refused. There is no fresh-profile fallback or token copying.
Raw capabilities, IDs, journals, player identities and account data stay local.

Actually executed on the current code inputs:
- MSVC 19.44 x64 Release /W4 /WX build PASS; all four C++ modes exit 0.
- Rust 1.98.1: **15 bootstrap tests PASS**, including six explicitly enabled
  own-executable integration cases, journal uniqueness/overwrite protection,
  nonzero exits, partial resume and replay rejection.
- Preview manager build PASS; git diff whitespace check PASS.
- No Computer Use/game interaction during this source-only batch.

Literal code-input blobs (documentation excluded):

| Input | Git blob |
| --- | --- |
| `independent_bootstrap.rs` | `c9cb47840c04f0de1081e4fefa6d1300057b5b7d` |
| `compat.cpp` | `400e4453a18b6b9bbefe9401ef798380c003d5d7` |
| `probe.cpp` | `3817e4c570ef018ed8843a73e20a5a76084fe23b` |

**Remaining blind spots:** real saved-world/marker reopen after process restart;
long concurrent gameplay and recovery after a game crash; full helper/broker
and HKCU/auth isolation; raw-syscall/alias/host-peer protection; disk-full,
partial-write, post-replacement interruption and power-loss behavior; FPS/frametime/GPU/CPU overhead and
comparison with Sandboxie. Prior raw peer-access control showed that name
translation is **not a security boundary**. Four clients, installer and
comparative performance remain **NOT_EXECUTED**. No maturity percentage is
awarded from fixture coverage.

**One next action:** extend the same generated control to interrupt a writer
after replacement and verify reopen sees the committed new bytes, while checking
real byte-range locking/mapped-file compatibility. Keep game UI untouched;
actual Minecraft restart/soak acceptance waits for an operator-selected window.
