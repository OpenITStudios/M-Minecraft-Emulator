---
id: document.system.architecture
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Lazy Tester — current Windows architecture

This document defines the **active architecture only**. The four-client,
Sandboxie, AppContainer, VMware and Guest Agent designs are historical,
recoverable from prior `Local` commits and the
[legacy retirement register](legacy-retirement.md). They must not supply
a fallback provider or change present-day launch decisions.

## Product and flow

Lazy Tester is a Tauri 2 / Svelte 5 desktop application for **five**
individually licensed Minecraft Education Windows profiles, MC01–MC05.
It uses one official Minecraft installation and persistent private
per-profile storage. The operator explicitly launches selected profiles;
starting the manager does not launch Minecraft.

```text
Svelte UI (presentation and user requests only)
  -> typed TypeScript bridge (validated session contracts)
  -> allowlisted Tauri commands + one manager resource gate
  -> retained native Rust controller (process/session authority)
     -> verified official game image + reviewed native DLL payload
     -> persistent private profile handles and normal lifecycle
  -> Windows workspace/DWM presentation (no lifecycle authority)
  -> real Minecraft Education processes (game owns login/world/render/network)
```

The selected running game is presented in the main workspace. Up to four
other independent native game windows contribute passive preview tiles.
Minecraft remains separate OS processes; Task Manager grouping is **not**
a product acceptance claim.

## Canonical end-to-end flow routing

Every active flow has ONE existing owner. An interface or source snapshot
may request/observe work, but cannot take ownership from its runtime owner:

| Flow | Entry -> responsible component | What it cannot establish |
| --- | --- | --- |
| App startup / manual Play | Root launcher -> Tauri/Svelte -> typed bridge -> retained Rust game controller -> official Minecraft processes | Launch/window state does not imply sign-in or world readiness |
| Window composition / session exit | Workspace/DWM owner -> original verified game window; game controller owns normal close and exact process lifecycle | A DWM tile cannot own or terminate Minecraft |
| External AI/MCP | Optional `Development/tooling/agent/mcp.mjs` (stdio) -> same-image CLI/authenticated original controller | Reading target identity or native ACK cannot prove gameplay input |
| Original Skills Probe | Reviewed ZIP/cache -> `probe_qa` local capture checkpoint -> externally authorized Player/admin command -> appended original runner markers -> frozen source report | Original Probe-reported PASS cannot verify a map/world or gameplay |
| Map-aware QA | Map-owned JSON -> `map-test-plan.mjs` -> bounded `probe_qa.mapPlan` and terminal `qaReport` | A source-only report shows original Probe outcome, missing evidence and zero executed map tests; never a gameplay PASS |
| Team module changes | Explicit local ZIP comparison -> approved team locks/source -> separately reviewed team sync | Lazy Tester does not install or fork analytics/scoring |
| Diagnostic evidence | Explicit local JSONL import -> ignored `Data/Diagnostics` -> bounded read-only MCP | File identity/checksum does not prove Minecraft world origin |
| Developer checks / package | `DEV.cmd` -> reviewed local tools -> existing check/package owners | Source checks or unsigned packages do not certify gameplay |

The recommended **one** QA interface is `probe_qa`. Its default mode is
source-only: it prepares the original admin command but **does not send
it**. Its `proofGate` keeps Player/admin delivery, exact QA world and
process/world-bound Content Log `NOT_VERIFIED`; only a separately
authorized real Windows/Minecraft observation could change those
runtime claims. No extra command dispatcher, game controller, QA engine,
daemon, hosted CI or automatic map state writer is in this flow.

For the minimal source owners and consumers, consult
[the implementation map](implementation-map.md). For exact QA status,
scope and failure handling, consult
[Map-Aware Probe QA](map-aware-probe-qa.md). The one current work/status
owner is [the concise development handoff](../../planning/development.md).

## One owner per responsibility

- Native lifecycle, launch authorization, close and uncertain outcomes:
  `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`
  and `independent_bootstrap/session_controller.rs`.
- Independent profile files, registry, settings and storage identity:
  the existing `independent_bootstrap/` modules; no credential store.
- Retained controller IPC and exact Windows peer admission:
  `engine/native_clients_session.rs` with its `pipe.rs` module.
- User actions and backend admission:
  `src/LazyTesterApp.svelte`, typed `src/app/bridge/` and
  `src-tauri/src/commands/native_clients.rs`.
- Window layout, DWM previews and restoration:
  `engine/workspace_surface.rs` and its existing child modules.
- Pure portable contracts, not a provider or second state machine:
  `Development/engine/runtime-core/`.
- Official game resolver and reviewed compatibility DLL source:
  `Development/tooling/acceptance/windows/`.
- Distribution and account/world preservation:
  `Development/engine/distribution/lazy-tester-package.json`
  and native Tauri product config.

The [implementation map](implementation-map.md) gives precise routes
to source owners and affected consumers. This architecture is a
boundary contract, **not** a duplicate module inventory or task queue.

## Safety and lifecycle invariants

1. A Minecraft process PID, window or profile folder alone is not a
   signed-in, playable, independent licensed session.
2. No silent adoption of unknown games, token copying, automated Microsoft
   sign-in, implicit game launch or forced process termination.
3. Each profile retains its verified storage identity; migration or deletion
   requires its existing explicit authorization and preservation checks.
4. Native window presentation never becomes a second game lifecycle owner.
   Restore presentation before coordinated normal CloseOne/CloseAll.
5. Unknown or interrupted mutations require inspection/reconciliation,
   never automatic blind replay.
6. Foreground/background FPS values (first-run 144/10) are policy
   **ceilings**, not measured achieved FPS or memory savings.
7. Unsupported operating systems cannot invoke native Minecraft controls.
   macOS five-client support is not implemented or accepted.
8. Runtime and developer build caches are distinct from accounts, worlds
   and installer-preservation data. No unscoped cleanup is authorized.

## Pending IPC result provenance and disconnected worker (P35)

A request to the retained Rust controller has **one locally owned
worker and one one-shot response receiver**. The mutex and pending
request gate refuse another exchange while that original worker is
unresolved. The unique receiver is sufficient to distinguish an old
local worker's result from a new request; inventing an unrelated
public transaction identifier or making the caller replay a mutation
would not improve controller-side acceptance.

Previously the `recv_timeout(45s)` branch handled both a live,
still-pending worker and `RecvTimeoutError::Disconnected` identically
as `TimedOut`. This misreported a terminated worker/sender as if
it were still working. The original IPC owner now separates:

- **Completed:** a complete worker result is consumed from its own
  receiver; a fresh successful session Inspect (not Settings) is
  still the only route that can clear prior uncertainty.
- **StillRunning:** preserve the original worker/receiver and signal
  timeout; do not create a replacement or replay the request.
- **ResultChannelLost:** join and retire the terminated worker; return
  a distinct broken-pipe error, but **retain uncertainty** because a
  native effect may have happened before the local result disappeared.
  The existing one-shot native CLI maps this transport result to its
  fixed `OUTCOME_UNCERTAIN` code, not `NATIVE_UNAVAILABLE`, so MCP
  cannot treat it as a guaranteed non-executed request.

A late result drained by a subsequent caller remains linked only to
its original one-shot receiver, is never returned as that caller's
result and never clears uncertainty automatically. No request wire
field, game-session owner, polling thread, retry, public result
schema or installation path changed. One pure Rust fixture covers
late verified/native-refused replies, live wait, transport failure,
independent channels and disconnected sender.
Fixtures are AUTHORED_NOT_EXECUTED; real IPC/Windows/Minecraft
behaviour remains NOT_VERIFIED.

## Pending worker uncertainty and Settings inspection (P34)

The original native session client retains a timed-out IPC worker
instead of replaying its request. Subsequent operations first check
that the original pending worker has settled. An unresolved worker
still blocks additional work, and the result of a late worker is
not returned as if it belonged to a later caller.

The prior uncertainty gate admitted both `Inspect` and
`InspectSettings` as read-only operations, but also allowed **either**
successful reply to clear `Controller.uncertain`. The desktop can
request `InspectSettings` simply by displaying or refreshing its
Settings page. A settings response is not a sufficient basis for
reconciling the outcome of a lost launch, close, or AI lease response.

The gate now permits both reads without mutation, but only a fresh,
successful **session `Inspect`** may clear the transport's uncertain
state after the old worker has settled. The old worker's late
completion is consumed without replay and explicitly retains the
uncertainty until that new session inspection. An unsuccessful
session Inspect, Settings refresh, or disconnected pending worker
never turns an unknown mutation into an accepted result. A session
snapshot is a current observation, not a proof that the original
mutation succeeded; native input lease/epoch and per-client recovery
checks still independently control input authority.

No new IPC field, controller, retry, polling task, lifecycle command,
UI action or release step is introduced. The existing read-only
Settings path remains usable during uncertainty. A pure Rust
admission fixture is **AUTHORED_NOT_EXECUTED**; native Windows IPC,
Minecraft Education and five-client acceptance remain NOT_VERIFIED.

## Confirmed native IPC reply versus handshake acknowledgement (P33)

The authenticated retained controller writes a bounded, typed response
before the client sends the final `ack` message used to close the
local named-pipe exchange. Previously, a failure **only** while
writing this final client ACK caused the client to report a transport
failure even after it had received a complete response from the
verified, still-live controller. This could make a completed
native mutation appear ambiguous and encourage an unsafe manual retry.

The existing `native_clients_session/pipe.rs` now preserves the
fully received `SessionReply` when the *trailing ACK alone* fails.
The ACK remains best-effort cleanup after a verified peer and complete
parse, not a second commit or gameplay acceptance. This never turns
a truncated/unparseable response, exited/unverified peer, pending
45-second worker, or lost request into success: those continue to
report uncertainty, retain existing state/handles and require
operator inspection instead of automatic replay. No second
controller, retry queue, request ID, public IPC schema, lifecycle
mutation or identity bypass was added. A pure Rust fixture covers
ACK success, ACK write failure on an already-confirmed success,
an explicit native rejection, and malformed response exclusion.
AUTHORED_NOT_EXECUTED; Windows/IPC/game execution not verified.

## Native AI input and Windows-session boundary

The existing native controller's named-pipe admission binds the same Windows
logon/session, user, integrity and executable at both peers. A validated
MC01–MC05 PID/HWND is a **read-only identity**, not a separate keyboard,
relative-mouse or gamepad input channel. AI gameplay dispatch remains blocked.
An alternate Windows desktop within the same interactive window station is
not independent gameplay input: Windows has only one active input desktop
per window station at a time.

No multi-session provider is currently admitted. Minecraft Education's
Windows **client** is not supported on Windows Server; Windows Enterprise
multi-session is not licensed for production outside Azure Virtual Desktop;
the current operator does not use cloud or VM as a fallback. Community
RDPWrap-based multiseat projects modify unsupported OS session behavior and
are not an approved native dependency, driver or installer route.

Windows Agent Workspace is **not an admitted gameplay backend**. Windows
ODR can run a properly packaged executable MCP server within a separate
agent-account session, and Microsoft's containment documentation permits
that server to launch executables *inside its own session*. This is narrower
than a supported per-game keyboard/relative-mouse/screenshot control API.
The public ODR documents reviewed do not establish game input acceptance,
four independently addressable game sessions, session-specific GPU capture,
Minecraft Education sign-in/P2P or a Codex-to-ODR host integration.

ODR containment requires an .exe with MSIX/package identity and static MCP
tool registration. The existing unpackaged Node stdio MCP is not such a
connector; directly connecting to a regular stdio MCP from Codex does not
create an isolated Windows agent session. Disabling connector protections
is not an accepted workaround. Windows 365 for Agents separately exposes
desktop control on **cloud PCs**, which is outside this local-only design.

No active native lifecycle, identity, desktop or IPC code is being replaced
by speculative agent-session components. Revisit only when a supported
local-session desktop input channel and the required game acceptance
evidence are independently established.

A future architecture change requires separate operator approval and an
explicit lifecycle/security/packaging compatibility review. In particular,
do not install RDPWrap, switch foreground desktops, weaken IPC identity,
copy Microsoft tokens or create a second native Minecraft controller as
a source-only shortcut. The current approach remains unchanged.

## Acceptance and engineering discipline

`Local` is the only development branch; `main` is release-reserved.
GitHub Actions CI is entirely disabled. Source reading can verify current
contracts and program structure but cannot establish Rust/Tauri compilation,
Windows installation, native DWM pixels, independent five-client sign-in,
gameplay, networking or performance. Record those as `NOT_EXECUTED`
unless exact-revision evidence exists.

Use the existing `DEV.cmd` / `Development/tooling/` checks where an
appropriate execution environment is actually available; they are not
required to finish independent GitHub source review. Do not create a new
orchestrator, daemon, driver, provider, alternate state ledger or test runner
without a concrete current-product need.

Current work and the **single next step** belong only to
[planning/development.md](../../planning/development.md).
