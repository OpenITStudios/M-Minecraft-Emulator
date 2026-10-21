---
id: document.system.independent-engine-architecture
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Independent Minecraft Session Engine — source-first architecture

## Product invariant and licensing boundary

Build a lightweight, **independently authored** Windows engine specifically for
four concurrently playable Minecraft Education Desktop sessions, each with its
own legitimately licensed account, persistent world/data and browser-like
presentation tab. One official game installation is preferred. The game
binary and Microsoft sign-in are not modified. Selecting another tab never
closes, stops, signs out or reprovisions another running session.

This is inspired by the **capabilities** of Sandboxie, not a derived Sandboxie
distribution. Historical upstream/fork files under `engine/isolation-owned/`,
`engine/isolation-candidates/`, and `tooling/isolation-source/` remain for
provenance and are **NOT** dependencies of this engine. Do not port, translate,
copy, link or bundle Sandboxie driver/DLL/service code into this implementation.
Source ownership, third-party dependency licenses and contributor assignments
must be audited before any proprietary distribution. Using documented Windows
APIs does not transfer ownership of Microsoft's implementations.

## One owner per responsibility

| Responsibility | Canonical owner | Boundary |
| --- | --- | --- |
| Domain contract, lifecycle/admission policy | `engine/runtime-core/` | Pure rules; no Windows handles or tokens |
| Session identity and source model | `engine/runtime-core/src/independent_engine/identity.rs` | Fixed 01–04 labels and full-width PID/creation time; value objects are not runtime proof |
| Operation effect classification | `independent_engine/operations.rs` | Inspect, present, launch, close and provision are distinct; not another admission system |
| Storage and authentication surface matrix | `independent_engine/storage.rs` | Local, roaming, registry, named objects, peer/host access, restart persistence, broker; no credential values |
| Native provider and adapter interface | `independent_engine/ports.rs` | Abstract ports only; DisabledIndependentProvider rejects everything |
| Evidence tier policy | `engine/runtime-core/src/independent_isolation.rs` | Synthetic, native Windows, Minecraft runtime are separate; not an acceptance registry |
| Windows process identity and future adapter | `apps/desktop/src-tauri/src/engine/instance_identity.rs` | Pin handles, validate PID/FILETIME, query tokens; return sanitized observations |
| Native window presentation/recovery | Existing `apps/desktop/src-tauri/src/engine/workspace_surface.rs` and Svelte workspace | Reversible presentation, no implicit lifecycle changes |
| Authoritative route admission | Existing desktop `instance_catalog.rs` and canonical runtime policy | Isolated Launch/Provision remains blocked until an independent provider is genuinely accepted |
| Synthetic tooling and offline review | `tooling/acceptance/windows/`, `tooling/acceptance/evaluate-appcontainer-isolation.mjs` | No game/account access; observations never authorize launch |
| Current continuation | `planning/development.md` | Decisions/next work only, not a runtime data database |

### Bound storage evidence contract — source-only

`engine/runtime-core/src/independent_engine/evidence.rs` validates diagnostic
storage facts against one exact `SessionSlot` and one full PID/FILETIME
`ProcessIdentity`. Its typed method/source matrix separates:

- synthetic fixture canaries from native per-process observations;
- path queries from actual canary access and restart persistence;
- registry and named-object observations from file-system behavior;
- host/peer access denial from ordinary private writes;
- real Minecraft sign-in observation from Windows token/MSAL metadata.

Unrecognized combinations, duplicate surfaces, mismatched identities and
synthetic positive proofs are rejected. Broker conclusions require
`MinecraftRuntime`-class observations. These are **unattested facts supplied
by future collectors**, not independently verified evidence or launch permission.
External platform crates can no longer call `StorageObservations::observe`
directly; they must use the reviewed converter. No secret/path fields are
transported by this contract, and no runtime probes or profile writes occur.

## Read-only source integration

`independent_engine/inspection.rs` combines `ProcessObserverPort`
and `StorageObserverPort` without adding a lifecycle owner. The typed
function `inspect_existing` rejects an observed PID/creation-FILETIME
mismatch before calling the storage adapter. Negative Rust tests use
in-memory fakes to check stale identity, denied process inspection, and
missing roaming/authentication evidence. This is a source integration
contract only; the real Windows adapter is not yet selected and no
Tauri command calls it.

## Logical runtime flow (not yet wired to production)

1. **Inspect target:** identify the registered, signed official Education image without modifying or duplicating the installation.
2. **Verify process ownership:** open and retain the exact process handle, compare PID, creation FILETIME, user/AppContainer identity and Windows session; reject mismatch or exited processes. A PID/title alone is insufficient.
3. **Determine isolation capability:** inspect named-object scope, local/roaming/registry view, peer/host canary denial, persistence and broker behavior. Distinguish path strings from actual file access and game credentials.
4. **Admit one bounded operation:** reuse canonical lifecycle/admission; refresh relevant evidence immediately before mutation; reject unknown/unsupported providers, untrusted binaries and stale identities. No synthetic fixture flag can unlock launch.
5. **Execute through one native adapter:** only a future verified Windows implementation may launch or normally close **its own** process. No global kill, broad ACL grants, credential cache manipulation or guessed session adoption.
6. **Present without mutating gameplay:** the existing window workspace switches selected HWND/focus and rolls back failed presentation. Background Minecraft processes and simulations remain alive.
7. **Recover explicitly:** on manager failure restore game windows; on process failure keep other sessions untouched. Account-bearing profile roots are never deleted by diagnostic fixture cleanup.
8. **Validate later:** separate actual two-game sign-in/render/input/network proof from four-client gameplay and installer acceptance; never promote source/tests into product readiness.

## Planned platform adapters (implementation slots, not a promise of compatibility)

| Adapter concern | Preferred Windows facility or direction | Stop condition |
| --- | --- | --- |
| Process identity/launch | Win32 process and token APIs with exact creation identity | Cannot prove own process, official signed target or singleton compatibility |
| Namespace separation | Native AppContainer named-object scope, only if compatible | Actual Education singleton still conflicts |
| Local/private storage | AppContainer profile and tested filesystem permissions | Peer/host access is possible |
| Roaming Known Folder and registry | Test documented OS mechanisms; design independent virtualization only if truly required | Unmodified Education resolves sensitive paths to shared host |
| Microsoft authentication | Game-owned login; investigate supported MSAL/WAM behavior, no tokens exported | Two independent licensed sign-ins cannot persist |
| Window/focus | Existing Win32 presentation and recovery owner | Blank window, input, fullscreen/DPI or failed rollback |
| Privileged service/driver | **Not selected** until a demonstrated requirement and security review | Unsupported signing, broker semantics or unsafe access |

AppContainer is a **candidate**, not the mandated final provider. If roaming or
Microsoft broker compatibility cannot be achieved with documented APIs, record
NO-GO for that candidate rather than inserting unreviewed hooks, bypasses, or
a large speculative custom driver.

## Implemented source adapter (read-only)

The existing desktop identity owner now implements the
`ProcessObserverPort` through `WindowsProcessObserver` and
`inspect_independent_source`. It calls the previously implemented
process-handle pin and creation-FILETIME check, then returns a typed
`ProcessObservation` with both official-image signature and visible
window evidence explicitly **false**. The matched process is a Windows
process, not an accepted Minecraft account or a verified signed game.

`UnknownStorageObserver` currently returns `Unknown` for **every**
storage and broker surface, including Roaming Known Folder, registry,
process namespace, peer/host data and authentication. This is an
intentional fail-closed platform-port placeholder, not a filesystem
sandbox, second state owner or production implementation. A future
observer must replace it with verified OS-specific evidence. The
integration is not registered with Tauri; admission continues to
deny isolated launches.

## Exact-process storage binding and token metadata

The port contract now requires `StorageObserverPort::inspect_boundaries`
to receive the exact `ProcessIdentity` (PID and full-width creation
FILETIME). An eventual native storage implementation must re-pin or retain the
same handle before inspecting a process-specific storage view. Domain tests
verify the requested identity reaches the observer, and that mismatched or
denied process observations prevent storage inspection entirely.

The desktop's existing process identity owner checks the live token's
AppContainer flag and SID through its already pinned process handle. Only
`ProcessContainment::NoAppContainerToken` or
`ProcessContainment::AppContainerTokenObserved` leaves the native observer.
Raw SIDs, tokens, filesystem paths and credentials remain private; the
all-`Unknown` storage adapter is unchanged. Observing an AppContainer SID
is not proof of separate named-object namespaces, RoamingAppData,
registry, WAM/MSAL accounts or game compatibility. No launch route is enabled.

## Process-local path evidence — source-only contract

`independent_engine/paths.rs` accepts a sanitized, exact-process
`ProcessPathSnapshot` with three independent observations:
LocalAppData, `APPDATA` roaming environment, and
`FOLDERID_RoamingAppData` Known Folder. Each is `Unavailable`,
`UnderExpectedProfile`, or `OutsideExpectedProfile`. The collector
must establish that it ran within the intended process context; an
external host query for another user's Known Folder is not equivalent
to the unmodified game's actual view. No raw path is stored.

The mapping distinguishes `PathOnly` from
`PathOutsideProfile`. Both remain unresolved because neither proves
actual file/registry access or account isolation. A reported outside
path is a compatibility warning, **not a demonstrated host-data leak**.
No game, token, registry or profile file is opened. Tests cover
stale PID/FILETIME, wrong slot, all-private-looking path reports,
missing Known Folder data and invalid provenance.

## Source-first implementation sequence

1. **Domain baseline (authored):** typed identity, operation classification,
   storage-surface observations and disabled provider interfaces in runtime-core.
2. **Platform observation (next):** narrowly scoped independent Windows-native
   inspection in the existing desktop owner. Pin process creation identity and
   sanitize results; no new desktop manager or app-wide polling service.
3. **Read-only evidence mapping:** inject test doubles into the interface to
   exercise unknown, shared, revoked, stale and contradictory observations.
4. **Private storage design:** transactional, explicitly owned profile paths;
   no auto-delete of any profile that could hold game/account state. Host/peer
   denial and Known Folder routing remain separately checked.
5. **Launch/stop design:** after a supported approach is identified, specify
   verified official image launch, profile ownership, bounded graceful stop,
   cleanup/rollback and timeouts. Keep the actual implementation unregistered
   until supported by Windows and Minecraft observations.
6. **Workspace integration:** reuse existing window and process ownership
   logic; no relaunch per tab, fake running-state cache or second GUI.
7. **Source-only review:** module ownership, Rust/TypeScript contracts,
   deterministic negative tests and static checks without enabling CI.
8. **Later local acceptance (deferred by operator):** run synthetic Windows
   fixtures, then two signed-in Minecraft clients, then four independent
   playable clients, persistent worlds, multiplayer, window recovery,
   measured CPU/RAM/GPU and clean-host packaging.

**Current evidence ceiling: REMOTE_GITHUB source architecture only.** The new
domain modules are not an isolation implementation or runtime provider. Rust
tests are authored but not yet compiled/executed; no Windows or Minecraft
readiness is claimed. Branch `Local` only; `main` and GitHub Actions remain
untouched.
