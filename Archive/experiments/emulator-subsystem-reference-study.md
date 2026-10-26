# Emulator Subsystem Reference Study

Status: RESEARCH / DESIGN ONLY — 2026-10-11
Source baseline: MIVUBI-STD/M-Minecraft-Emulator, GitHub `Local` @ `7dc6bd65af82cbe804c9931e9a95a46db16fc4e6`.
Authority: `AGENTS.md`, `GITHUB_RULES.md`, `docs/system/implementation-map.md` and current source. This is a bounded comparative study under `Archive/experiments/`, **not** a product contract, second implementation map, work-status database, architectural migration approval, or runtime acceptance.

## Outcome and constraints

Improve the existing four-client architecture (one Native Minecraft Education Windows client and three distinct VMware Workstation Windows guests). Preserve one canonical Rust lifecycle/readiness owner, one Tauri/Svelte desktop adapter, current product compatibility identities, account-bearing guest data, and the G0–G4 acceptance definition. No new hypervisor, distributed controller, persistent state cache, cloud service, OAuth/account automation, plugin framework, CI, background daemon or speculative GPU optimization is approved by this study.

Select by problem, not by technology prestige: (1) identify a concrete first owner and source behavior, (2) compare one external implementation/contract, (3) choose **KEEP / ADAPT IN OWNER / DEFER / REJECT**, (4) trace callers, destructive consequences and public schemas, (5) only implement a complete, source-justified change, (6) apply proportional evidence after a coherent batch. An external mechanism is not a new dependency by default. No overall subsystem score, readiness percentage or additional test gate.

## Product feasibility — first-class risk, no invented workaround

Minecraft Education lists an OpenGL 4.4 GPU requirement for Windows: https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements.
VMware's Workstation 17 announcement specifies guest OpenGL 4.3: https://blogs.vmware.com/cloud-foundation/2022/11/17/announcing-vmware-workstation-17-pro-and-player/.
On 23–25 September 2026 a Broadcom community discussion describes OpenGL 4.3 as the reported guest version for 25H2/26H1; a Broadcom employee notes partial coverage of higher-version extensions: https://community.broadcom.com/vmware-cloud-foundation/question/any-plans-for-opengl-44-support-in-guests.

This is a **potential compatibility mismatch, not proof of failure or a supported workaround**. Keep G1 one-disposable-guest boot → Minecraft Education installation/manual licensed sign-in → real rendered world/input acceptance before promising G2–G4. Source work can continue; source review, OpenGL flags, process running and reported guest version cannot substitute for gameplay proof. Do not add GPU spoofing or a competing platform route.

## Canonical cross-domain data and command flow

~~~text
UI intent + ephemeral presentation (Development/apps/desktop/src)
 -> typed invocation / response validation (Development/apps/desktop/src/app/bridge, contracts.ts)
 -> thin Tauri command (Development/apps/desktop/src-tauri/src/commands)
 -> single public command (Development/engine/runtime-core/src/command.rs)
 -> runtime observation + lock + shared lifecycle admission (runtime.rs, lifecycle_admission.rs)
 -> existing VMware provider or bounded authenticated Guest Agent (provider/, guest.rs)
 -> re-observed facts / typed status, error, actions, Doctor and snapshot
 -> frontend projection (never a second lifecycle/setup/readiness decision)
~~~

Daily and setup actions must remain separate. `start-setup` can open fresh guests before identity/agent readiness; daily start needs compatible lineage and guest launcher. `QA_READY` is a guest checkpoint, not gameplay/account proof. Provider RUNNING, Guest Agent reachable, interactive launcher ready, Minecraft process and real account/game/multiplayer acceptance are non-equivalent observations. Operation journal and support bundle are read-only diagnostics; neither governs lifecycle. Network peer play remains real-world acceptance, not status-inference code.

## Comparative inventory: 23 analytical subsystems, NOT 23 implementation services

The identifiers below are **research labels only**. They must not become new package names, engines, registries, folders, aliases or external APIs.

| Research area | Current canonical owner | External pattern to assess | Decision now |
| --- | --- | --- | --- |
| A1 Desktop UI | `Development/apps/desktop/src/App.svelte`, `surfaces/` | coherent operator feedback | KEEP; defer speculative redesign |
| A2 Setup Experience | `setupFlow.ts` consuming `doctor.rs` | guided recovery UX | KEEP; backend owns next step |
| A3 Typed IPC | `app/bridge/`, `contracts.ts` | Tauri permission/progress contracts | KEEP; adapt only for proven discrepancy |
| A4 Window Layout | `src-tauri/src/engine/window_arrangement.rs`, `screen_overlay.rs` | Microsoft FancyZones window/monitor/DPI handling | DEFER Windows-specific features |
| B1 Orchestration | `Development/engine/runtime-core/src/runtime.rs` | controller-runtime read/reconcile/idempotence | ADAPT principles, not controller daemon |
| B2 Lifecycle Admission | `lifecycle_admission.rs` | policy and mutation same facts | KEEP existing single evaluator |
| B3 Doctor | `doctor.rs` | causal prerequisite ordering | KEEP; review ambiguous blockers |
| B4 Resource / Capacity | `resources.rs`, `capacity.rs`, `startup.rs` | admission with workload observations | DEFER numeric tuning pending evidence |
| B5 Schema / Policy | `policy.rs`, `client.rs`, `schema.rs` | explicit wire/state compatibility | KEEP; no new state aliases |
| C1 VMware Provider | `provider/workstation.rs`, `provider/fusion.rs` | Packer VMware builder & vmrun semantics | ADAPT source-grounded error discipline |
| C2 Base Factory | `base_preparation.rs`, `profile.rs`, `Development/engine/guest/windows/` | Packer deterministic image preparation | KEEP strict finalized, stopped Base |
| C3 Clone / Storage | `provider/storage.rs`, `vmx.rs`, `persistence.rs` | VMware linked-clone backing chains | PRIORITY investigation |
| C4 Recovery / Snapshot | `runtime.rs`, `provider/` | crash-safe cutover and snapshot semantics | KEEP reprovision disabled |
| D1 Guest Transport | `guest.rs`, `bin/virtual-guest-agent.rs` | QEMU GA request correlation/bounds | KEEP bounded HTTP; compare failure modes |
| D2 Interactive Launch | `bin/virtual-guest-agent.rs`, guest Windows scripts | Microsoft service/session isolation | KEEP per-user launcher boundary |
| D3 Minecraft Parity | `minecraft_runtime.rs`, `profile.rs` | official Minecraft version/install rules | KEEP Native as authority |
| D4 VM/Windows Identity | `identity.rs`, `profile.rs`, `provider/vmx.rs` | immutable provenance and isolation | KEEP account owner in guest |
| E1 Diagnostics | `diagnostics.rs`, `doctor.rs` | typed causal errors | KEEP; avoid duplicate health state |
| E2 Journal / Support | `journal.rs`, `support.rs` | bounded, redacted diagnostic records | KEEP existing append-only history |
| E3 Evidence / Acceptance | `Development/tooling/acceptance/`, `docs/validation/` | outcome-based proof | KEEP G0–G4 only |
| F1 Build | root/toolchain, Rust/Node manifests | Cargo locked dependency reproducibility | OPEN G0 source issue |
| F2 Packaging | `Development/engine/distribution/`, Tauri NSIS | payload integrity/data preservation | KEEP safety contract |
| F3 Trusted Update | `update.rs`, distribution policy | TUF anti-rollback/freeze trust logic | STUDY ONLY; self-apply stays disabled |

## High-value upstream references and specific boundaries

1. **VMware Packer plugin**: https://github.com/vmware/packer-plugin-vmware and https://developer.hashicorp.com/packer/integrations/vmware/vmware/latest/components/builder/vmx. It distinguishes full and linked clones, Base images, snapshots, and shutdown. A linked clone **depends on its parent disks**. Source caveat: `provider/workstation.rs` intentionally clones directly to final VMX because relocating a linked clone can invalidate backing paths; `provider/fusion.rs` follows a different staging/promote route. Do not copy Packer's build framework into our app.
2. **Broadcom Virtual Disk API backing-chain documentation**: https://developer.broadcom.com/xapis/virtual-disk-api/latest/vddkFunctions.6.7.html. Study recursive parent-child backing dependencies and restore hazards; this is vSphere-oriented reference material, **not evidence** that the API is available or appropriate for Workstation. Never treat VMX presence plus profile existence as a verified disk chain.
3. **controller-runtime FAQ**: https://github.com/kubernetes-sigs/controller-runtime/blob/main/FAQ.md. Reconciliation should be idempotent and based on observed state. Our runtime already reconciles timeout/partial VM mutations; study only specific missed failure paths, not queue watchers, Kubernetes or a second controller.
4. **QEMU Guest Agent**: https://www.qemu.org/docs/master/interop/qemu-ga-ref.html. Request synchronization after timeouts protects long-lived byte-stream protocols. Our guest boundary uses bounded per-request HTTP with a request ID: **do not port QEMU's stream delimiter** without proving an equivalent stale-response defect. Compare idempotence, timeouts, protocol authentication and concurrent launch admission only.
5. **Microsoft interactive services**: https://learn.microsoft.com/en-us/windows/win32/services/interactive-services. Service Session 0 is not user-interactive; retain existing per-user interactive launcher. Do not introduce automatic account or session-token handling merely because APIs exist.
6. **PowerToys FancyZones architecture**: https://github.com/microsoft/PowerToys/blob/main/doc/devdocs/modules/fancyzones.md. Study multiple monitors, mixed DPI, and window ownership only against current Win32 `window_arrangement.rs`, not a new permanent window-reflow service.
7. **TUF specification**: https://theupdateframework.github.io/specification/v1.0.36/. Study rollback/freeze/mix-and-match threats against current staging metadata. Do not adopt TUF's complete infrastructure or enable self-apply without a concrete trust-model gap and explicit release authorization.
8. **Minecraft Education official requirements**: https://edusupport.minecraft.net/hc/en-us/articles/360047556591-System-Requirements. Product feasibility is not source-level optimization.
9. **Existing Tauri v2 framework**: https://v2.tauri.app/. Prefer native typed command/channel/security primitives already in use; no new desktop transport or plugin manager.

## Specific priority: account-safe reprovision and clone provenance

At inspected baseline:
- `runtime.rs::reprovision_inner` and the Workstation/Fusion provider reject reprovision; DO NOT bypass.
- `provider/storage.rs::cutover_staged_vm` is an unwired filesystem-only rename primitive requiring staged VMX and profile siblings and keeping an original backup; its source tests are authored, **not executed**.
- `provider/workstation.rs::provision` uses a linked clone placed directly at final destination to avoid unverified path relocation; `provider/fusion.rs` has a staged promote path. These are not automatically interchangeable.
- `provider/vmx.rs` and `persistence.rs` now protect specific VMX metadata writes/recovery, but do not prove the full linked VMDK chain.

Candidate contract to examine before any code change:
1. Preserve existing VM, snapshot chain, original guest/account disk and profile until an independently validated replacement exists.
2. Validate any candidate's complete VMX, profile provenance, unique UUID/MAC/agent token and **actual backing-disk references in the candidate's intended FINAL location**; a directory rename is not sufficient.
3. Decide explicitly whether a safe linked-clone candidate can be built for the final location without mutating the original. If not established, **defer candidate cutover**; do not substitute a generic storage abstraction or destructive fallback.
4. All ambiguous clone/copy/rename outcomes must be observed and fail closed. Do not automatically delete staging, backups, or originals.
5. Only after a proven viable layout, reason about rollback, same-filesystem cutover, crash recovery, provider admission and public UI action; keep one owner (`provider/storage.rs` for filesystem mechanics, `runtime.rs` for lifecycle, provider implementation for VMware semantics).
6. Changes require exact-SHA static dependency/consumer review and proportional regression proof; reopening reprovision requires stronger Windows/VMware disk safety evidence and separate approval.

## Delivery batches, not micro-tests

- **Study 1 — VM integrity (P0 source research):** compare current Workstation clone + cutover preconditions against official VMware linked-clone parent requirements and Packer; record KEEP / safe limited ADAPT / DEFER. No writes to runtime from this study.
- **Study 2 — Runtime/guest failure flow (P1):** trace one bounded source-proven error path through admission, provider, guest and presentation; reuse current owners; avoid extra state/transport.
- **Study 3 — Desktop/performance (P2):** use PowerToys and documented Windows APIs for proven focus/window errors; defer hard resource/GPU tuning until G1/G3 observed evidence.
- **Study 4 — Delivery/evidence (parallel maintenance only when source-justified):** Cargo-generated desktop lockfile and G0 build evidence when genuinely available; keep update signing, installer-data and G0–G4 semantics unchanged.

Do not turn these studies into additional product gates. One coherent source batch → narrow static check → targeted tests only where genuinely executable → exact SHA handoff. No repeated comprehensive acceptance campaigns after cosmetic edits.

## Current disposition and STOP

**Confirmed:** existing owner boundaries, safe reprovision guard, linked-clone relocation concern, G0 lockfile absence, OpenGL 4.4/4.3 feasibility mismatch.
**Research recommendation:** KEEP all canonical ownership; compare Packer + Broadcom disk-chain documentation first; adapt only a source-demonstrated VM integrity issue in its present owner.
**NOT_EXECUTED:** Rust/Node/Tauri builds/tests, installer, Windows VMware client, licensed login, game graphics, four-account gameplay and G0–G4.
**Next single task:** perform a read-only, line-level comparative audit of `provider/workstation.rs::provision`, `provider/storage.rs` and VMX/backing-disk consumers against Packer's linked-clone behavior. Produce a specific feasible/non-feasible cutover recommendation before changing source. Do not enable reprovision or ask for an ad-hoc machine test.
