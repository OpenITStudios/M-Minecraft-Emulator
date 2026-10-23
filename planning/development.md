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

Public Microsoft API contracts were compared with the existing read-only PSEC adapter:
`IsProcessSecurityEnvironmentVersionSupported(DWORD, BOOLEAN*, DWORD*)`,
`QueryProcessSecurityEnvironmentSupport(PROCESS_SECURITY_ENVIRONMENT_SUPPORT_FLAGS*)`
and `PSE_SUPPORT_FS_DENY`/`PSE_SUPPORT_NETWORK_PROXY` agree at the signature/declared-flag level. **No Windows SDK compilation or Windows execution occurred.**
References:
- https://learn.microsoft.com/en-us/windows/win32/api/processmodel/nf-processmodel-isprocesssecurityenvironmentversionsupported
- https://learn.microsoft.com/en-us/windows/win32/api/processmodel/nf-processmodel-queryprocesssecurityenvironmentsupport
- https://learn.microsoft.com/en-us/windows/win32/api/processmodel/ne-processmodel-process_security_environment_support_flags

**Candidate decisions, not runtime verdicts:**
- PSEC/MXC ProcessContainer: **INCOMPLETE**, native filesystem/policy containment alone has no demonstrated transparent per-game Roaming/HKCU/ADAL-WAM redirection; do not promote the capability probe to Launch authority.
- MXC IsolationSession: **NO-GO for current directly visible local Minecraft GUI** per its own supported-workloads guide (no display server inside that session); not a usable drop-in to the native workspace. https://github.com/microsoft/mxc/blob/main/docs/isolation-session/oneshot.md
- `RegOverridePredefKey`: **NO-GO as an external transparent HKCU switch**; Windows documents that it affects only the calling process. https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regoverridepredefkey
- MSIX AppData/HKCU redirection: **NOT independently per game PID** (per user/per app/package); no permission to repackage or re-register third-party Education. https://learn.microsoft.com/en-us/windows/msix/desktop/desktop-to-uwp-behind-the-scenes

**Concrete source correction:** the earlier read-only PSEC capability FFI passed a pointer to a 32-bit support mask. The official Microsoft MXC PSEC V2 implementation uses a 64-bit output mask, while public Win32 documentation names an enum pointer without a guaranteed output allocation width. The adapter now reserves a zero-initialized 64-bit mask, so a 64-bit OS write cannot overrun a four-byte Rust local. Existing low-bit diagnostic results and the frontend wire schema remain unchanged. Source reference: [Microsoft MXC secenv.rs](https://github.com/microsoft/mxc/blob/ffecec04357747d6649714f473436dfb77111be8/src/mxc-sdk/src/backends/process_container/common/secenv.rs). Native SDK compilation and runtime **NOT_EXECUTED**; no new game isolation proof.

**Outcome:** none of these mechanisms alone meets MC02's complete game-bound contract. An isolated process is not yet an isolated Minecraft session. Do not add another generic primitive or disable admission on the basis of this review. Seek one supported complete path; otherwise retain **NO-GO/UNKNOWN** without fake progress.

## SINGLE ACTIVE PRIORITY — P0: real MC02 isolation provider

**One decision question:** Is there a supported Windows mechanism to start a **second unchanged official Minecraft Education Desktop** client next to MC01 with separate *effective* Roaming (environment AND Known Folder), Local data, HKCU reads/writes, authentication helper/broker, singleton/window behavior and playable GPU/input?

1. Validate actual SDK/Win32 ABI and build first—particularly the newest native PSEC implementation—and OS/version support. Do not confuse code review or export presence with a functioning engine.
2. Select/reject **one concrete enforcement and mediation route** based on the real game’s required filesystem, registry, broker, IPC and window behavior. Separate directories, RunAs labels, PID ancestry, AppContainer canaries, synthetic COW and policy-only denial do NOT establish effective MC02 separation.
3. When supported, implement one **small end-to-end vertical slice** through existing Rust independent provider port, native Windows adapter and single admission owner. Do not unlock MC02 until game-bound Windows evidence confirms legitimate sign-in, independent world persistence, cross-access denial, concurrent gameplay and normal restart.
4. When unproven, record the exact first **NO-GO/UNKNOWN** incompatibility and stop that candidate. Do not create more generic sandbox layers, speculative DLL hooks, fake-ready providers, new monitoring UI, services or test collections.

**P0 exit:** MC01 and MC02 truly playable and independently licensed at the same time; separate real world writes persist after relaunch; effective HKCU/auth remain non-crossing; host/peer cannot read protected client state; interactive controls work. Source tests are supporting evidence ONLY.

## Ordered outcomes (not new test-status gates)

| Stage | State | Exit |
| --- | --- | --- |
| **P0a Provider feasibility** | **ACTIVE** | One supported unmodified-game MC02 mechanism is established or decisively rejected with an explicit incompatibility. |
| P0b Two-client vertical slice | BLOCKED by P0a | Implement/validate actual MC01+MC02 private data/auth/gameplay; maintain Launch denial until ready. |
| P1 Four-client extension | WAITING | Reuse the SAME provider for MC03/MC04; verify four licensed worlds, normal input, multiplayer/recovery. |
| P2 Efficiency against Sandboxie | WAITING | Comparable real-game workloads for RAM, CPU, GPU, frametime, launch/switch latency and stability; optimize measured bottlenecks. |
| P3 Distribution | WAITING | Signed native Windows package, safe install/repair/uninstall/rollback preserving worlds/accounts; release needs separate approval. |

Old G0–G4 docs concern the retired VMware campaign, not these native delivery outcomes.

## Codex working rule

Read root `AGENTS.md`, `GITHUB_RULES.md`, `.agents/skills/emulator-development/SKILL.md`, then this single handoff; pin exact `Local` SHA. Start with `engine/runtime-core/src/independent_engine/` (domain/provider ports), `apps/desktop/src-tauri/src/engine/instance_identity.rs` and `native_clients.rs` (native Windows observations), and `instance_catalog.rs` (isolated Launch deny gate). `engine/isolation-native/independent-service/` is unbundled and inert/diagnostic. Retired Sandboxie/fork/VM routes are NOT active engineering tasks.

Prefer proof of game-compatible isolation over generic Windows experiments, more mock files, many microtests, UI polish and broad documentation. No second lifecycle/store, credential access, unsolicited driver/service install, release/main changes, unsafe profile operations, or GitHub Actions. Execute targeted build/tests if actually available; if Windows/game execution is unavailable, label it `NOT_EXECUTED` and complete only source work with a real path to P0. Do not ask the user to run a PC merely for source-verifiable tasks.

On each meaningful bounded commit, replace the **current** blocker/next status in this file instead of appending a new chronology. Commit to `Local` with `Work/State/Decision/Proof/Unresolved/Next` trailers, keeping original runtime unknowns explicit.

## Current owner and next action

**State:** ACTIVE P0a. **Blocker:** no validated process-bound Roaming/HKCU/Microsoft broker mediation and real gameplay for a second official Education client. The PSEC output-buffer width is corrected in source, but native Windows SDK compilation and actual PSEC/Minecraft runtime compatibility are still UNKNOWN.

**Next:** Determine whether any documented, non-modifying Windows mechanism can provide **all** MC02 game boundaries together (effective Roaming, HKCU, ADAL/WAM broker, interactive GUI). Select one for a real game-bound vertical slice only if the missing mediation can be demonstrated; otherwise record a precise NO-GO and keep isolated Launch/Provision denied. Windows SDK compilation of PSEC remains NOT_EXECUTED.
