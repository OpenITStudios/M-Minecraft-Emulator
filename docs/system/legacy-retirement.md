---
id: document.system.legacy-retirement
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Selected Windows isolation pilot and inactive legacy inventory

**Historical decision 2026-10-12 (superseded below):** Use one official Minecraft Education Desktop installation and one selected **native + Sandboxie Plus** two-client experiment before extending to the four-account target. This outcome is
**UNPROVEN**. The current desktop pilot does not claim four working clients.

This document is the **single deprecation register**. Executable command
authority is engine/runtime-core/src/command.rs; desktop entry is
apps/desktop/src/main.ts; Tauri command registration is
apps/desktop/src-tauri/src/app_bootstrap.rs. Retired code is not removed.

## RETIRED: Sandboxie and the derived owned fork — 21 October 2026

The operator superseded the earlier fork selection: the replacement must be our
own implementation using supported Windows facilities, without Sandboxie code,
DLLs, services, driver, box configuration or feature certificates. Rebranding a
fork does not satisfy this requirement. **The replacement is not implemented.**


| Dependency / owner | Retirement boundary |
| --- | --- |
| `engine/isolation-owned/`, `engine/isolation-candidates/` | Historical fork/reference source; no further product development or activation |
| `engine/isolation-native/owned-runtime/`, `tooling/isolation-source/` | Historical build/test/transaction tools; standalone developer scripts remain, not a replacement provider |
| Desktop `owned_provider.rs`, embedded `owned-manager.ps1` / `probe-v1186-runtime.ps1` | Still depend on Sbie DLL queries, service identity and box settings; retained for existing-session inspection/recovery/close |
| Desktop `instance_catalog.rs` | Rejects isolated Launch/Provision on official-pilot and owned-fork paths before provider execution, including direct IPC/helper requests |
| Desktop `workspace_surface.rs` and Svelte workspace | Reusable presentation mechanics; retained session authority still comes from the old provider and must be replaced |
| Installed Sbie services, payloads, box roots and saved accounts/worlds | Preserved; no uninstall, reset, certificate changes or data migration in source retirement |

New isolated sessions 02–04 are unavailable; the UI must say so. Existing observed
sessions may be selected, restored or normally closed. Native launch 01 remains
available at its existing command boundary. This is retirement of product launch
admission, **not physical removal of every dependency or a completed replacement**.
The currently running old executable is not changed by a source commit.


### Replacement boundary and first implementation proof

Use the existing lifecycle/admission owner, not a parallel manager or credentials
store. An independent provider must own launch identity and per-instance lifetime;
window composition must consume verified identities rather than discover games by
name. Switching tabs changes presentation only; it must never terminate another
process. Preserve PID/start-time/HWND checks and recovery contracts.

Before implementing a launcher, establish whether supported Windows facilities can
provide both independent Education process instances and separate persistent
Microsoft authentication contexts on the same interactive desktop. Different game
directories or environment variables alone do not prove either property. Separate
Windows users/tokens/profiles are an investigation candidate, not an accepted
solution: session placement, brokered authentication, singleton scope, graphics,
input, permissions and persistence require evidence. A fresh Rust wrapper cannot
be assumed to create these properties. Do not patch Minecraft, copy tokens or
change the old driver licensing checks.

Read-only API assessment (21 October 2026):
[CreateProcessWithLogonW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createprocesswithlogonw)
can create a process under another user's credentials and optionally load its
profile. `LOGON_NETCREDENTIALS_ONLY` retains the caller's local token and is
unsuitable for local identity separation. Full executable paths and deliberate
profile/environment handling are required. Windows account credentials are a
separate concern from Minecraft sign-in; no custom secret persistence is approved.
This API's existence is not Education compatibility proof.
[Kernel object namespaces](https://learn.microsoft.com/en-us/windows/win32/termserv/kernel-object-namespaces)
are global or per-session. Inference: a second user on the same desktop does not
automatically supply a separate named-mutex namespace. Education's singleton
mechanism is UNKNOWN; no mutex bypass or game modification is allowed.
**Verdict:** per-user launch remains a candidate with unresolved singleton,
authentication and desktop-permission constraints, not a production architecture.
Next: read-only Windows API/source feasibility review of separate Windows-user
process creation, profile loading and interactive desktop access. Identify a
supported two-session experiment and its constraints, or explicitly reject that
approach. Only after proof should the provider be replaced. Four-account login is
deferred until independent lifetimes, profile separation and recovery are proven.
Fullscreen, cross-monitor DPI, four-account play/network and long-duration
survival remain NOT_EXECUTED for the replacement.


## INACTIVE: keep source until removal is safe

| Root | State / reason |
| --- | --- |
| apps/desktop/src/NativeFeasibilityApp.svelte | UNMOUNTED historical native feasibility screen; retained for source regression and review only |
| apps/desktop/src/App.svelte and legacy app/surfaces, dialogs | UNMOUNTED; VMware wizard and VM controls not available to user |
| Tauri commands/virtual_clients.rs, setup_tools.rs, window_arrangement.rs | NOT REGISTERED and NOT DECLARED in the native desktop's Rust module tree; old source retained for reversible legacy support |
| engine/runtime-core/src/runtime.rs and provider/workstation.rs, fusion.rs, vmx.rs, storage.rs | VM MUTATIONS PAUSED at public command boundary |
| engine/runtime-core/src/provider/app_sandbox_pilot.rs | POWER/DISPLAY ACTIONS PAUSED; explicit inspect remains read-only |
| engine/guest/windows/, Base preparation and Guest Agent | LEGACY BUILD-ONLY; installed guest binaries not modified |
| tooling/acceptance/windows/collect-client-feasibility.ps1 and old VM G0–G4 scripts | HISTORICAL EVIDENCE ROUTE; not an active campaign |
| experiments/gpu-pv-migration-design.md | RESEARCH ARCHIVE, no production provider migration |
| engine/distribution/ and existing NSIS resources | COMPATIBILITY PRESERVED; old release assets still embed VM Guest Agent and scripts |
| apps/desktop/scripts/prepare-package-binaries.mjs | PACKAGE BLOCKED while the native-first pilot remains the active desktop; fails before building/staging old guest binaries |

The active desktop `commands/mod.rs` declares only `native_clients`. Its `engine/mod.rs` declares only `app_instance`, `dpi`, and `native_clients`. The legacy screen-overlay module was also unmounted: no active command could start it, so its shutdown hook was inert. These changes exclude unused modules from the active native desktop compile graph without deleting their files, tests, historical data or the legacy runtime-core dependency. Desktop-only legacy Rust tests will not be compiled while those modules are excluded; this is not proof that the old VMware implementation remains buildable.

The pause list at command.rs blocks actions, stage-update, register-base,
open-base-finalization, provision, reprovision, verify-identities, start,
start-setup, start-client, suspend, stop, restart, set-ready, reset, open,
launch-minecraft. It also blocks App Sandbox pilot start/shutdown/display.
Every mutation returns UNSUPPORTED before invoking a provider. There is
no UI toggle or user-selectable override. Unpausing requires a reviewed
source change.

Read-only legacy diagnostics are retained by explicit CLI:
product-status, doctor, policy, base-preflight, snapshot, status,
resources, diagnostics, history, support-bundle, check-update, and
app-sandbox-pilot inspect. Some may inspect existing VMware state when
explicitly invoked. The active desktop makes no such startup calls.

## Do not delete

- Existing VM directories; VMX/VMDK, VHDX, Base disks, snapshots,
  account-bearing guest profiles, worlds, saved Microsoft sessions.
- The persisted Windows runtime root at
  %LOCALAPPDATA%\M-Bedrock\VirtualClients and its provenance files.
- Installed identifier com.mivubi.mbedrock.virtualclients, release naming,
  signing/update contracts, guest protocol IDs and compatibility paths.
- Old test evidence, exact-SHA G0 proof, Git history and migration notes.

The canonical npm build:app flow is deliberately blocked with
PACKAGE_BLOCKED_NATIVE_FEASIBILITY by an explicit release hold, independent
of Svelte component names or runtime product-status. This is not
a missing-dependency failure or a completed native installer. A direct
unsupported invocation that bypasses package preparation is not
an authorized release workflow.

The pilot desktop cannot stop existing running VMs. If needed, an
authorized user must use the vendor's existing console to perform a
manual graceful shutdown. Installing this code does not stop or erase VMs.

## Separate native prototype package — design boundary, not a release

The installed Tauri identifier and current NSIS payload still belong to
the old VM manager. The canonical packaging command therefore remains
blocked while the native-first desktop route is selected. **Do not remove
this block** merely to obtain a downloadable test installer.

A future isolated prototype must first demonstrate all of these properties
through reviewed source and real package execution:

- A **different Tauri app identifier**, installer/product name, and
  installation/uninstall registry identity, with an approved migration
  policy if it ever replaces the old installed product.
- A distinct per-user launcher-state directory and single-instance marker.
  The native developer pilot now places its lock under
  `%LOCALAPPDATA%\MinecraftInstanceManager\NativePilot`; existing legacy
  VM profiles and account disks remain under their original paths.
  This source isolation does not approve a new installer identity.
- No VMware Guest Agent, Base/Sysprep scripts, HCS/App Sandbox provider
  binaries, VM management CLI, or legacy acceptance payload in the new
  installer. **A Tauri config overlay by itself may inherit resources
  from the base config**; inspect the final merged bundle contract.
- No automatic uninstallation, migration, reset or deletion of
  legacy runtime files, VM disks, snapshots or Microsoft sessions.
- No new Microsoft sign-in implementation, credentials store,
  patched Minecraft executable, or copied official app binaries.
- Independent verification of the actual Windows package file list,
  installation coexistence, upgrade, removal and preserved account
  data before calling a prototype installable.

This is a release-isolation specification, not authorization to rename
the current production identifier or implement another packaging system.
The current source pilot can still be developed and statically reviewed
without an installer. The old Tauri command/setup/window modules are excluded from the active Rust desktop module graph, but `virtual-clients-core` still compiles its legacy runtime and provider tree. Reduced package/RAM/storage size is **not yet demonstrated**.

## Deletion criteria — not yet met

1. Two independently logged-in Education sessions verified through actual
   game windows and real playable worlds, not candidate PIDs.
2. Four accounts playable concurrently, with multiplayer, persistence
   across close/reboot, acceptable RAM/GPU/storage and no account crossing.
3. Genuine Windows installer/upgrade/uninstall proof that preserves
   old account-bearing state and provides a practical recovery route.
4. Separate approved cleanup commit: remove old Svelte/IPC screens and
   obsolete UI tests while preserving useful regressions.
5. Separate approved provider removal commit: eliminate VMX, VMware,
   Guest Agent and packaging payloads only when migration is reversible
   and no guest data is automatically deleted.
6. Review installed product identifier and release contracts separately
   if an explicit compatible rename is ever warranted.

## Proof ceiling

At this stage GitHub source/tree inspection verifies only recorded source
paths and control guards. Rust, Node, Svelte, Windows NSIS, Sandboxie,
Microsoft authentication and real Minecraft sessions require independent
execution evidence and must not be described as passing.
