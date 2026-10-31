# Windows AI test bridge — source implementation

The bridge exists for **Minecraft Education peer-to-peer client testing** using the five native profiles owned by Lazy Tester. It does **not** start a dedicated server, create a second game controller, inject input, move players in background windows, or modify account/world data.

## Exactly what works in source

- A local **stdio MCP** server in `mcp.mjs` exposes profile, Computer Use target, optional read-only visual observation, peer planning, and gameplay evidence tools.
- `inspect_profiles`/`launch_profile` delegate to the **same packaged `lazy-tester.exe`** under `--lazy-tester-agent-cli` which calls the existing authenticated native session controller. A returned `RUNNING` process observation does not prove the game is logged in, has joined a world, or accepts input.
- Evidence tools read completed `[ANALYTICS]` and `[GAME_RESULTS]` JSON lines from the existing `minecraft-bedrock-agent-skills` debug capture script's per-client output. No new debugger listener or protocol is implemented. File-to-MC slot mapping is **manual and unverified** until connected to an observed player/world identity.
- Two modes: `human-plus-four-ai` reserves MC01 for the human host (default); `five-ai` permits MC01 launch as an AI-controlled slot. MCP tools do not give the AI any new right to interact with the protected human's keyboard or window.

**Important:** This increment implements the *control-plane and evidence reading*, **not** parallel gameplay. Real independent WASD/mouse for MC02–MC05 while a person plays MC01 remains `NOT_IMPLEMENTED / NOT_VERIFIED` on Windows. The tools do not claim to host/join a world, send chat commands, or perform simultaneous Minecraft input. Claude/Codex computer use alone does not guarantee four independent background input streams. A separate, verified input backend is required before advertising those capabilities.

## Start for source development (Windows)

The installed app provides `App/lazy-tester.exe` after packaging. The operator must install/prepare Lazy Tester normally; **no account, world or game launches simply by starting the MCP server**. A local Node 24+ installation is required to run the current MCP script. The source-repository path of this script must remain accessible; it is **not yet part of the public three-file installer export**.

Example Codex/Claude MCP server configuration (replace paths with absolute local paths):

```json
{
  "command": "node",
  "args": [
    "C:\\projects\\M-Minecraft-Emulator\\Development\\tooling\\agent\\mcp.mjs",
    "--mode", "human-plus-four-ai",
    "--exe", "C:\\projects\\M-Minecraft-Emulator\\App\\lazy-tester.exe",
    "--evidence-dir", "C:\\projects\\M-Minecraft-Emulator\\workspace\\private\\agent-evidence"
  ]
}
```

Use `--mode five-ai` only when **all five clients are intentionally AI-operated**. `launch_profile` is a state-changing request; authorize it through your agent tool permission UI. The native controller retains its own admission gate and lifecycle checks.

`--lazy-tester-agent-cli inspect` refuses without a compatible retained controller; it never invents an empty five-client state. `launch` uses the reviewed original bootstrap to request a client launch, without silently opening or joining a Minecraft world.

## Phase 1: Codex-targeted Windows Computer Use (source only)

New MCP tools `get_computer_use_target({slot})` and
`revalidate_computer_use_target({slot,pid,windowId,createdFileTime})`
resolve one of the existing five native Minecraft windows through the same
authenticated session controller. The native Rust adapter verifies exact
PID, process birth FILETIME, Windows session and root HWND using the existing
window owner, and refuses unavailable, minimized or nonresponsive windows.

In `human-plus-four-ai`, MC01 is **never** offered as an AI action target;
in `five-ai`, MC01 can be targeted. The response contains a **Cua Driver**
window selector `{kind:"window",pid,window_id}`. No external software is
automatically installed or started. Configure Cua Driver as a *separate*
Codex/Claude MCP server only after explicit local-setup approval; this
repository currently adds only reliable native target discovery.

Proposed agent-side read-only workflow after installation approval:

1. Resolve `get_computer_use_target({slot:2})` in Lazy Tester MCP.
2. Call Cua Driver `get_window_state({pid,window_id})` using the returned
   exact selector. Inspect the screenshot and accessibility state.
3. Revalidate the same `pid/windowId/createdFileTime` after capture.
4. If the target changed, discard the image or action; do not silently
   follow a replacement process. In hybrid mode never escalate Cua to
   foreground or use global SendInput, even if the background action refuses.

**No screenshot is captured by Lazy Tester in this phase**. Cua Driver is
an optional external window-capture provider, not a Minecraft game-input
backend. The reported verification applies only at the instant of native
inspection; it is not an authorization token for arbitrary external tools.
Background screenshot availability, all game input, P2P host/join, and
multiple simultaneous AI players remain NOT_EXECUTED / UNVERIFIED.
The existing MCP controller, UI, world storage and disabled CI are unchanged.

Official API basis:
- https://cua.ai/docs/cua-driver/concepts/how-cua-driver-works
- https://cua.ai/docs/cua-driver/reference/mcp-tools/apps-windows-and-screen
- https://learn.microsoft.com/windows/win32/api/windows.graphics.capture.interop/nf-windows-graphics-capture-interop-igraphicscaptureiteminterop-createforwindow

## Phase 2: read-only visual observation via Cua Driver (source only)

An optional **`observe_client({slot})`** MCP tool produces one
`image/png` content block and compact metadata after selecting the exact
AI-owned Minecraft window via the retained native controller. This uses
the already documented Cua Driver read-only CLI, avoiding a new WinRT/D3D
capture engine and avoiding any input/control into Minecraft.

To enable this *after separate operator authorization for installation and
runtime acceptance*, add `--cua-exe` with the absolute path to an ordinary
`cua-driver.exe`, for example `C:\\Tools\\cua-driver.exe` (example
path only, not a claim about the installed location). A separately installed
Cua Driver daemon must already be running; this bridge never starts it.

`observe_client` performs:
1. Resolve the current exact PID/HWND/birth/session for the AI-owned slot.
2. Call **only** `cua-driver call get_window_state` with explicit
   `pid` and `window_id`, screenshot enabled and accessibility walk
   disabled; use Cua's documented `--screenshot-out-file` option.
3. Read one PNG capped at 1280 px long edge (strict absolute limit 1536 px
   and 6 MiB); erase the temporary file before returning the MCP image.
4. Resolve the exact target again. Reject the frame if PID/HWND/birth/session
   changed. Missing Cua, minimized/stale windows, missing frames and unexpected
   image data fail closed; no desktop-wide fallback or foreground switch.

MC01 remains protected in `human-plus-four-ai`; `five-ai` may observe
MC01. One capture is **not proof of a live frame or successful gameplay**;
stale, black or unobtainable game frames must be reported as uncertainty.
No keyboard, mouse, cursor overlay, computer-use action, world mutation, or
automated agent scheduler is introduced. Cua's read-only CLI must be
reviewed and accepted on the eventual Windows installation, and capturing
a game-specific GPU window is still `NOT_EXECUTED`.

The PNG is stored only temporarily while servicing the request, not in a
world, credential/profile folder or Git repository. An abrupt process
termination could leave a temporary file for the operator to clean later.
Only code is committed; no user screenshots are collected now.

Cua CLI reference: https://cua.ai/docs/cua-driver/reference/cli/serve-and-configure

## Phase 16: stable bounded Skills Probe log snapshots

The `readGameEvidence` reader previously took its tail offset from
`lstat(path)`, then separately `open(path)` without checking they
referred to the same file. An MC02 log replaced/rotated during this
interval could cause an offset from the old file to be applied to the
new file, producing incomplete or misattributed records. A log being
appended/re-written during an open read had no before/after snapshot
gate either.

The existing reader now pins an ordinary file handle and compares its
nonzero device/inode identity, size, nanosecond modification/change
timestamps, and pathname identity before and after a bounded positional
read. A changed, disappearing, or unverifiable source returns
`CAPTURE_FILE_UNSTABLE` with **zero events**; the summary preserves
that status as a missing/unaccepted source. It never guesses that the
previous or replacement log belongs to the same Minecraft player.
Filesystems without a verifiable file ID fail closed. The reader
retries short positional reads within its existing 256 KiB limit, and
still drops trailing incomplete lines; the returned metadata flags
`incompleteTrailingLineIgnored`.

This is a best-effort atomic *reader-side* snapshot check, not
proof that the external debugger wrote a transactionally consistent
record or that account/world attribution is correct. Deterministic
regression fixtures cover rotation before open, append during read,
short reads, partial tails and unstable-source summarization. They
are AUTHORED_NOT_EXECUTED under current operator policy; there is
no new watcher, permanent log index, file store or Minecraft hook.

## Phase 15: bounded visual-worker startup and evidence isolation

The optional WGC source previously checked active workers only BEFORE its
asynchronous executable-file inspection. Concurrent starts for a slot, Stop
during this wait, or MCP connection shutdown could allow an untracked
late worker. `gpu-frame-source.mjs` now reserves each slot before the
await. Stop/shutdown revoke pending reservations; cancelled starts refuse
to spawn a process. Failure to supervise a spawned worker triggers a
kill attempt; failures while attaching worker handlers/timers also drop
the worker. These actions affect **capture children only**, not Minecraft.

MCP `stop_visual_observation` now cancels an overlapping, unfinished
multi-slot WGC startup batch rather than reporting an unqualified Stop.
The response explicitly reports
`STARTUP_CANCELLATION_REQUESTED` with an unconfirmed outcome until
the original Start call resolves. The existing observer immediately
terminates on a confirmed GPU/native window identity-change error,
clears the old frame, and runs its existing worker cleanup callback.

Skills Probe record IDs are not globally unique across all Minecraft
sessions. `evidence-summary.mjs` now scopes deduplication to the
reported `session_id` as well as slot and source; events without a
session ID are kept distinct rather than silently merged across
unknown sessions. File-to-player/world attribution remains
`OPERATOR_NAMED_UNVERIFIED`.

All changes reuse existing GPU, MCP, visual sampler and evidence owners;
only existing regression files were extended. Tests are AUTHORED
but NOT_EXECUTED. Windows capture, real game session identity, release
of a hung native process, account login and multi-client gameplay remain
NOT_VERIFIED. No extra service, installer, input backend or CI added.

## Phase 14: pin GPU frames to their original native client (source only)

The optional WGC worker is created for an exact, native-verified
Minecraft PID, HWND, process birth FILETIME and Windows session. The
previous `gpuSource.latest(slot)` checked packet age, while MCP checked
the *current* native target before/after the call, but did not verify the
worker was originally started for that exact target. A restarted MC02
could therefore be incorrectly attributed a stale frame captured by
an older worker when the old worker remained active.

`gpuSource.latest(slot,expectedNativeTarget)` now requires a fresh exact
binding and compares all four identity fields against the worker's
original pinned target. Any mismatch immediately discards the cached
PNG and stops that capture worker. The MCP passes its already-verified
native target into every WGC read, then still revalidates the target
after the frame read. Future-dated frame timestamps are refused until
the capture time has actually arrived.

The existing tests now cover PID, HWND, birth and Windows session
replacement, required binding, and future frames; MCP tests assert the
correct native identity is passed to the GPU cache. They are
**AUTHORED_NOT_EXECUTED**. No new tool, continuously running daemon,
input backend, driver, game control or profile lifecycle component
was added. A source-level identity guard is NOT proof that WGC can
capture Minecraft Education gameplay on Windows.

## Phase 13: Windows Agent Workspace + Codex interoperability verdict

**Outcome: not a production input provider, no new implementation.**

Microsoft's Windows On-Device Agent Registry (ODR) can run an
MSIX/package-identified binary MCP server under a separate agent user and
Windows session. The contained MCP server may launch executables **within
that same agent session**, but cannot directly inspect or control the
human's window/process/session. This is a useful official *process
boundary*, not a documented game-oriented keyboard/mouse delivery API.

There are three distinct integration paths; do not conflate them:

| Integration | Actual published capability | Lazy Tester decision |
| --- | --- | --- |
| Current Codex + local Node stdio MCP | Invokes existing same-session controller, plans/observes clients; no native game input | **Keep**, no new tool |
| Windows ODR + contained MCP (.exe/MSIX identity) | ODR registers packaged tools and launches them under agent-account session; may run in-session executables | **Not wired to Codex**; generic game input/capture and five-client session lifecycle unverified |
| Windows 365 for Agents computer-use MCP | Desktop click/type/screenshot/session lifecycle against a remote Cloud PC | **Not eligible** for local/no-cloud design |

ODR `odr.exe` registration is **not** evidence of a per-process keyboard
or relative-mouse transport, and package identity alone does not turn
Minecraft Education into a remotely controllable game. Likewise,
Windows support for a separate agent account does not prove that four
independent game sessions can remain concurrently playable.

The following independently documented or observed conditions would be
necessary before reopening this route: a supported third-party local
agent-session lifecycle with a verified Codex host path; keyboard,
relative mouse and visual capture within one agent session; the
Minecraft Education client rendering and accepting gameplay in that
session; preserved human MC01 input and independent account/world
ownership; then four AI sessions and P2P gameplay. These are currently
**UNVERIFIED/NOT_EXECUTED**. Do not build a new MSIX, ODR adapter,
session broker, input driver, or second Minecraft controller solely
to satisfy a hypothetical workflow. Do not turn on "Reduce protections
for agent connectors".

Windows 365 desktop computer-use documentation is a **different
cloud product**, not a substitute for missing on-device ODR input APIs.
The source-only decision closes Agent Workspace as an immediate coding
task; it does not reject future supported APIs.

Verified official references:
- https://learn.microsoft.com/en-us/windows/ai/mcp/servers/mcp-containment
- https://learn.microsoft.com/en-us/windows/ai/mcp/servers/mcp-server-overview
- https://learn.microsoft.com/en-us/windows/ai/mcp/odr-tool
- https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features
- https://developers.openai.com/codex/mcp/
- https://learn.microsoft.com/en-us/microsoft-copilot-studio/mcp-windows-365-agents

**Proof ceiling:** REMOTE_GITHUB source/API review. No tests, builds,
ODR actions, Windows installation, Codex sessions, Minecraft gameplay,
or input were executed. Production dispatch remains disabled.

## Phase 12: native Windows multiseat feasibility decision (source only)

**Decision: No multiseat runtime implementation is admitted yet.**
We examined native controller ownership, official Windows/Minecraft
platform support and a community RDP-based multiseat implementation.
No available path currently satisfies the product's simultaneous
MC01 human + MC02–MC05 independently controlled AI acceptance.

| Windows design | Source-supported fact | Decision for Lazy Tester |
| --- | --- | --- |
| Extra Win32 desktops in the same session | Only one input desktop is active at a time | Does not provide four independently focused games |
| Windows Server RDS | Multiple interactive RDS sessions, but Minecraft Education **client** explicitly does not support Windows Server | Exclude |
| Windows Enterprise multi-session | Microsoft's licensing does not allow production deployment outside Azure Virtual Desktop | Exclude from local no-cloud/no-VM design |
| MultiSeat/RDPWrap on Windows 10/11 | Uses RDP loopback, RDPWrap, streaming components, and privileged system modifications; its optional K/M InputHook is explicitly documented as a no-op | Research reference only, no integration/installation |
| Windows Agent Workspace / ODR | Preview agent identity runs in separate Windows session; contained MCP must be a packaged .exe with MSIX identity and cannot directly control a human session | Most relevant first-party research candidate, NOT a validated game controller |

A session-local input desktop is a **plausible** way to separate AI
controls from the human console session, but Microsoft Agent Workspace
does not establish either four independent agent sessions or acceptance
of Minecraft Education game input/graphics. Microsoft's contained MCP
server model also differs materially from Lazy Tester's unpackaged
Node stdio MCP: no automatic packaging, ODR discovery from Codex, game
startup or cross-session bridge exists. Do not use the existing
`--lazy-tester-agent-cli target` process lookup to cross sessions.

**Required proof order before any implementation:** (1) documented,
license-compliant and approved way to run ONE isolated Windows agent
interactive session under the user's target Windows edition; (2) one
independently licensed Minecraft Education client launches/signs in,
renders, and accepts keyboard/relative mouse entirely inside that session;
(3) human MC01 continues to receive uninterrupted input; (4) world
host/join, game-state evidence and native identity are attributable to
the two respective accounts; (5) prove four independently addressable
AI sessions and resources before promising a four-AI or five-AI mode.
Any failure remains `BLOCKED`, not a "source test PASS".

Neither package creation, new service, RDPWrap, driver, agent workspace
activation, input injection, Minecraft nor source tests were run.
MCP still exposes read-only route planning but no gameplay actuator.

Official/maintainer references:
- https://learn.microsoft.com/en-us/windows/win32/winstation/desktops
- https://edusupport.minecraft.net/hc/en-us/articles/13106858087956-Windows-Installation-Guide
- https://learn.microsoft.com/en-us/azure/virtual-desktop/windows-multisession-faq
- https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features
- https://learn.microsoft.com/en-us/windows/ai/mcp/servers/mcp-containment
- https://github.com/vibesoftwarecoder/MultiSeat/blob/master/README.md

## Phase 10: Windows per-client game input feasibility (source-only)

Read-only native PID/HWND identification is NOT a channel for delivering
independent Minecraft movement to that window. The existing session
controller and Skills Probe evidence owners remain unchanged.

| Candidate | Observed limitation | Decision |
| --- | --- | --- |
| `SendInput` / Windows `InputInjector` | Shared system input, no external exclusive game PID target | Not an independent input backend |
| Raw Input `RIDEV_INPUTSINK` | Receiving game must register itself for background input | Not an external sender |
| GameInput `SetFocusPolicy` | Focus policy belongs to the game API consumer | Not an external process router |
| ProtoInput | Can reroute inputs using hooks inside game processes | Not authorized in no-injection design |
| Windows Agent Workspace | Preview separate Windows accounts/sessions | Research only: native controller has no cross-session route |
| ViGEmBus / XInput | Retired driver, globally visible device and four standard indices | Historical source retired from Local in Phase 11 |

**Phase 10 source correction:** `assess_input_route` rejects any
cross-Windows-session target combination in BOTH hybrid and five-AI
modes. Windows Agent Workspace cannot be used as a drop-in input provider
for current same-session Lazy Tester. It would require a separate
architectural decision and target-session ownership proof. Neither
support for four independent Minecraft game sessions nor GPU rendering,
Education sign-in or P2P connectivity has been demonstrated.

Minecraft Education documents keyboard/mouse controls and a limited
controller settings screen, while a later Education support response
states Windows computer gameplay is keyboard/mouse only. Therefore
controller acceptance remains **UNVERIFIED**; keyboard and relative
mouse input are the required capabilities.

**Future acceptance (not performed):** MC01 remains human-controlled
and uninterrupted; independent MC02 moves, looks and attacks while
unfocused, with game-frame/Skills Probe evidence attributed to MC02.
No external driver, client injection, local tests or gameplay ran.

Sources:
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
- https://learn.microsoft.com/en-us/windows/uwp/ui-input/input-injection
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputdevice
- https://learn.microsoft.com/en-us/xbox/gdk/docs/reference/input/gameinput/interfaces/igameinput/methods/igameinput_setfocuspolicy
- https://www.splitscreen.me/docs/proto/
- https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features
- https://edusupport.minecraft.net/hc/en-us/articles/360047116832-Minecraft-keyboard-and-mouse-controls
- https://edusupport.minecraft.net/hc/en-us/community/posts/53960255250324-cant-connect-ps5-controller-to-play

## Phase 8–9: retired virtual-gamepad experiment

The optional ViGEm/XInput worker and offline mapper were **removed from
the active `Local` source during Phase 11**. They were never production
input providers, were never built or installed, and were never connected
to the MCP. Their exact source remains recoverable at immutable commit
[`fc4d87d9defe`](https://github.com/OpenITStudios/M-Minecraft-Emulator/tree/fc4d87d9defe01eafd87a0abc6904f2f9d7d5807/Development/tooling/agent).

Retirement reason: the candidate emitted a globally visible virtual
gamepad rather than keyboard/relative-mouse input exclusively bound
to MC02–MC05. Standard XInput has four device indices; ViGEmBus is
retired. Minecraft Education's 28 September 2026 support response says
computer play currently supports keyboard and mouse. Older Education
documentation mentioning controller settings is contradictory, so an
XInput claim cannot be promoted without actual game acceptance.
There is no installed component to remove from user machines.

Removing this lab also removes its experimental test from the existing
`test:localtest` command and its irrelevant diagnostic field from
`assess_input_route`. The tested semantic gameplay planner, read-only
PID/HWND route, safety gate, optional MCP observation, native controller,
profile/account/world data and all live acceptance requirements remain
unchanged.

## Phase 7: bounded independent input transaction runner (source only)

The source-only `input-delivery.mjs` is a narrow **per-client native input
provider port**, not an installed keyboard/mouse driver and not a second
Minecraft session controller. It is **NOT registered** as an executable MCP
tool. Only read-only `assess_input_route` and existing planning/observation
tools are exposed to Codex.

The source harness accepts internal callbacks that a future validated native
provider must implement: `verify(slot)`, `admit(target)`, `send(target,event)`,
`releaseAll(target)`, and `protectHuman()`. It cannot be enabled by passing
an MCP tool parameter. The production MCP always reports
`provider:NONE`, `execution:DISABLED`, and
`BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`.

Safety properties implemented in **source**, not demonstrated on Windows:

- One bounded transaction; no overlapping runs, at most 64 semantic events
  within 35 seconds. A native route older than 1.5 seconds is refused.
- Recheck the exact PID/HWND/birth/session identity before each timestamp
  group. In hybrid mode a separate human-desktop protection callback is
  mandatory. The MC01 slot is never an AI input target.
- Parallel **between different MC slots**, ordered **within one slot**:
  release a previous button before pressing its replacement; relative
  camera deltas are separate events.
- Cancellation and transport errors enter a `finally` block that calls
  `releaseAll` for every target. Incomplete release is a hard error.
  The provider must additionally implement its own crash watchdog:
  JavaScript alone cannot guarantee hardware release if the process hangs.
- Even acknowledged provider callbacks are reported as **transport only**;
  they never establish gameplay movement, camera control or human-input
  safety in Minecraft.

**Critical remaining blocker:** An independent, non-foreground native input
transport that works with Minecraft Education has not been found or
accepted. Windows `SendInput` is a shared system stream; Cua Driver's
Win32 background action does not guarantee game controls; ProtoInput's
multigame routing uses game-process hooks. Microsoft's isolated agent
workspaces exist but have no demonstrated five-Minecraft integration.

No DLL injection, driver installation, desktop-wide `SendInput`,
foreground-swap, MakeCode, gameplay or executable MCP input action was
added in this phase. The test file uses internal callback fixtures to
exercise sequencing and safety only; it is **NOT_EXECUTED**. A passing
fixture in the future would not count as Windows or Minecraft acceptance.

Relevant research:
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-sendinput
- https://www.splitscreen.me/docs/proto/
- https://cua.ai/docs/cua-driver/concepts/how-cua-driver-works
- https://support.microsoft.com/en-us/windows/ai/ai-features/experimental-agentic-features

## Phase 6: native-bound independent input admission (source only)

**New read-only tool:** `assess_input_route({actions})` connects the
existing concurrent input timeline to freshly observed native Minecraft
window/process identities. In `human-plus-four-ai` it also inspects
MC01's window **only as a protected human reference**, never as an
AI input destination. In `five-ai`, MC01 may be an AI target.

The MCP reuses its current, retained authenticated controller and
`--lazy-tester-agent-cli target <slot>`; each target must pass the
same PID, process creation FILETIME, session, root HWND, visible and
responsive checks as visual observation. A route is rejected if two
slots resolve to the same PID/HWND, any target is unavailable, or the
overall native observation spans more than five seconds. This is an
ephemeral diagnostic mapping, **not an input authorization token**.

Every response explicitly reports:
- `NOT_DISPATCHED` and
  `BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`.
- The AI-only PID/HWND/window identity map, plus an observational
  (not guaranteed) human MC01 foreground indicator.
- The independent capabilities that a future native provider must
  demonstrate: genuine per-client keyboard, relative mouse, concurrent
  clients, release-all, preserved human input, and actual Minecraft
  game-state evidence.

No actual keyboard, mouse or game action is delivered by this tool.
Windows `SendInput` targets the shared system input stream, not a
specified Minecraft process. `RIDEV_INPUTSINK` does not allow an
external MCP to force another application to accept raw input; the
receiving application must register. Cua's background UI input does
not establish raw game control, while ProtoInput requires in-process
hooks whose use with Minecraft Education has not been approved.

Relevant sources:
- https://learn.microsoft.com/en-us/windows/win32/api/winuser/ns-winuser-rawinputdevice
- https://cua.ai/docs/cua-driver/concepts/how-cua-driver-works
- https://www.splitscreen.me/docs/proto/

Source regression fixtures were authored for hybrid/5-AI slots,
aliasing/refusal and truthful NO-INPUT reporting. **No tests, local
builds, game sessions, CI, or external input processes were run.**

## Phase 5: concurrent independent-input contract (source only)

The previous planner refused **any** simultaneous controls for one Minecraft,
which would prevent normal AI gameplay. `plan_player_actions` now allows
overlapping *independent channels*: forward/back movement, lateral movement,
camera, jump, sneak, attack, and item use. Conflicting directions on the
same axis and exclusive waits still cannot overlap. Different MC slots may
use different controls concurrently.

A new `look_relative` action accepts integer `deltaX/deltaY` within
±512 and a 50–250 ms scheduling window. It is **not calibrated** to Minecraft
mouse sensitivity or an actual input device. Request
`plan_player_actions({actions,includeTimeline:true})` to compile ordered
PRESS/RELEASE/POINTER_DELTA events grouped by timestamp. The default
returns only the existing concise plan. `input-timeline.mjs` can also
prepare a per-slot emergency-release intent.

**Nothing is executed.** All outputs remain `NOT_DISPATCHED`, with
`BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`. The future Windows
provider must independently route actual controls to each game, map game
keybinds, revalidate PID/birth/session/HWND, avoid the human foreground,
and implement physical per-client release-all on errors/disconnect.
A sent-input ACK would not prove a Minecraft player moved.

ProtoInput and Nucleus use in-process game hooks; no Minecraft Education
compatibility proof exists. No hooks, DLL injection, device driver, global
SendInput, MakeCode, UI, second game controller or executable MCP input
tool was added. Tests are authored but NOT_EXECUTED by operator instruction.

## Phase 4: opt-in persistent GPU frame stream (source only)

An isolated Rust worker source using Windows Graphics Capture (`wgc 3.1.0`)
and a bounded Node frame reader now exist under:
- [Native WGC worker](./native-gpu-capture/README.md)
- [GPU frame protocol and reader](./GPU_CAPTURE.md)

Existing `start_visual_observation`, `read_visual_observation`,
`stop_visual_observation` and `observe_client` tools work with an
**optional** persistent WGC provider once its separately reviewed binary
exists. Opt-in configuration: `--gpu-capture-exe` pointing to an absolute
`lazy-tester-gpu-capture.exe`. If specified, this provider takes precedence
over Cua and requires explicit start before a frame is available. The default
remains Cua snapshots when configured, with unchanged legacy behavior.

The MCP verifies each slot via the retained native controller before
starting one bounded worker per selected Minecraft and revalidates PID,
process birth, session and HWND before/after reading each cached GPU frame.
Each worker independently re-verifies the Windows process identity before
emitting. One in-memory frame per window, rate capped at five source
frames/second by MCP, local sampler reads approximately twice per second;
no images sent to Codex unless it explicitly asks for a changed sequence.

**This is neither full-motion video to Codex nor verified Minecraft GPU
performance.** The WGC crate has not been dependency-resolved/built and
the standalone worker is not packaged or installed. Native graphics consent,
Windows capture border, frame availability when Minecraft is backgrounded,
memory/FPS cost and multi-client behavior remain UNKNOWN. No fallback to
a desktop-wide screenshot, global SendInput, native injection, or gameplay
script exists. The worker terminates on stop, expiry, changed target, repeated observer
errors and MCP shutdown; it is not kept alive after an observer finishes.

All local tests, compiler runs, packaging and game execution remain prohibited
until separate operator approval.

## Phase 3: bounded continuous local visual observation (source only)

The existing MCP process can now run a **time-limited rolling frame sampler**
without Codex requesting a fresh screenshot for every observation.
`start_visual_observation` starts in-process sampling for 1–5 AI-owned
windows (MC01 prohibited in hybrid mode). `read_visual_observation`
returns the latest *changed* cached PNG only when Codex requests it, and
`stop_visual_observation` immediately clears the in-memory frame cache.

- Default: one read-only Cua window capture attempt every 1250 ms per slot,
  with slower sampling while the PNG bytes remain identical. Allowed interval
  750–5000 ms, session duration 5–90 seconds. No overlapping captures;
  native controller operations share the existing gate.
- One cached frame at most per client, each at most 4 MiB of encoded data.
  Maximum five independently observed windows. Single frame copy is never
  interpreted as verified gameplay progress, semantic motion, or a real
  30/60-FPS video stream. PNG byte hash change only signals changed **bytes**,
  not a detected enemy, movement or actual pixels.
- Cua preview capture requests `display_only:true`, preserving any
  interactive Cua action snapshot (a visual frame is not an action target).
  A cached frame older than 3500 ms is never returned as a new image.
- Old or replaced PID/birth/session/HWND terminates that slot's sampler.
  Three consecutive capture failures also stop it. Busy native operations
  skip sampling instead of interrupting game lifecycle.
- Timers stop on explicit stop, expiry, and MCP stdio connection closure.
  Source images are ephemeral in memory; not written to persistent
  account/world files or automatically forwarded to an AI model.
- `read_visual_observation({slot,afterSequence,includeImage})` returns
  status only if the newest sequence is unchanged, avoiding repeated model
  image ingestion. The model can combine changes with existing analytics
  summaries through `get_test_context` when needed.
- No background-control keyboard, mouse, camera-rotation, inference model,
  camera stream provider, second daemon, auto-join, or UI is created.
  Cua CLI remains a conservative **bounded frame sampler**, not continuous
  GPU video capture. For genuinely low-latency PvP, a later single persistent
  Windows Graphics Capture frame source would be needed, after proof.

**No local tests or Minecraft live verification are authorized yet.**
Both Cua Windows performance and real multi-window frame freshness remain
UNKNOWN. This code is an opt-in read-only local observation tool.

## One-call AI test context

The first MCP choice for Codex/Claude analysis should be
`get_test_context` (optional 1–5 slots). It obtains exactly one current
native `Inspect` through the existing authenticated session owner, then
combines the conservative host/join plan and compact Skills Probe evidence.
The handoff includes five native profile statuses, selected capture summaries,
and explicit uncertainty markers. It does not infer signed-in accounts,
in-world players, Minecraft host/join, input acceptance, or success.

This is an on-demand local tool response, not a background reporter, prompt
injection path, event daemon or autonomous AI task scheduler. The user may
continue to call `inspect_profiles` and `read_game_evidence` separately
for finer investigation.

## Source-only five-player coordination contracts

Two additional MCP tools prepare the Windows peer-to-peer workflow without
launching a world, taking keyboard focus, or inventing player movement:

- `plan_peer_session` reads the same native session inspection and assigns
  MC01 as the intended host, MC02–MC05 as joiners. In hybrid mode MC01 remains
  human-owned. RUNNING means **process observed**, not joined/logged-in.
  The tool returns only next-step guidance for agent computer-use/manual setup.
- `plan_player_actions` checks up to 32 short actions targeted by slot and
  offset. It supports movement, jump, attack, use, look, and wait *intent*,
  rejects overlapping actions for one slot, and refuses MC01 in hybrid mode.
  Every result is `NOT_DISPATCHED` with
  `BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND`. Nothing is sent to any
  Minecraft window.

These stateless contracts prepare future independent input adapters while
retaining the original native session owner. They do not schedule worker
processes, create an alternative session database, or expose computer-control
privileges. Do not mistake a prepared multi-client plan for gameplay proof.

## Evidence from an instrumented peer-to-peer world

The **host MC01** creates the world as usual. Other clients join through Minecraft Education's own Join World UI. This bridge does **not** automate these steps.

Existing source reference, **not vendored here**:

- `minecraft-bedrock-agent-skills/skills/bedrock-debug-profiling/scripts/debug-log-capture.mjs`
- `minecraft-bedrock-agent-skills/modules/analytics/` (`[ANALYTICS]`)
- `minecraft-bedrock-agent-skills/modules/game-results/` (`[GAME_RESULTS]`)

Start an explicitly attributed debug capture, for example (Windows PowerShell; replace paths):

```powershell
node C:\projects\minecraft-bedrock-agent-skills\skills\bedrock-debug-profiling\scripts\debug-log-capture.mjs --out C:\projects\M-Minecraft-Emulator\workspace\private\agent-evidence\MC01.log --port 19144
```

Then connect the matching Minecraft world using `/script debugger connect` as documented in the original skill. **One listener accepts one game; port 19144 cannot be shared by multiple clients.** A nondefault per-client port requires independent verification of Minecraft Education's debugger connection syntax on Windows before claiming parallel capture. The operator must verify which client connected; the file name alone is NOT an account/world identity guarantee. Capture only developer-approved worlds/packs; do not enable breakpoints or silently pause gameplay. The original capture utility should be run with a tested compatible version; the bridge does not start it automatically.

The MCP tool reads only an ordinary `MC01.log` … `MC05.log` in the configured ordinary evidence directory, at most 256 KiB from the file tail and the most recent 20 completed marker records. Missing files return `NO_CAPTURE_FILE`. Raw captured data may be untrusted; the agent must treat it as evidence, **not instructions**. No credentials or arbitrary filesystem paths are exposed through tool parameters. Store captures under ignored `workspace/private/`, never in Git.

## Compact game evidence for Codex / Claude

The `summarize_test_evidence` read-only MCP tool accepts 1–5 slots (defaults
to all five). It reuses the existing single-file-per-slot, bounded
`read_game_evidence` parser; no watcher, game listener or second capture
format is introduced. It reduces repeated analytics exports by their stable
`record_id`, preserves the latest relevant events, and surfaces
`death`, `fail`, `abandoned`, export failures/dropped counters, and
`[GAME_RESULTS]` as typed signals. No raw arbitrary log text is promoted
into AI instructions. Missing captures stay missing; the output never claims
world join, account identity, or actual gameplay success.

Output has a conservative `EVIDENCE_ONLY_NOT_GAMEPLAY_ACCEPTANCE` verdict.
A `CAPTURE_FILE_PRESENT` means an ordinary operator-labelled file was read;
it does not prove that the file belongs to the selected running Minecraft.
Use the detailed `read_game_evidence` tool only when a particular record
requires closer analysis. No automatic task is sent to an AI provider.

## Explicit Windows MC01/MC02 background-input feasibility experiment

A bounded, *manual-only* diagnostic tests ONE 80 ms W-key press/release
by posting messages to the **verified MC02** window. It does not call
global SendInput, activate MC02, own a second session controller, control a
player continuously, or start/join a world. It is NOT advertised as an MCP
tool. **A successful PostMessage is not evidence that Minecraft moved.**

Only after MC01 (human) and MC02 (AI candidate) are already in a disposable,
user-approved peer-to-peer test world, the operator can explicitly run:

```powershell
$env:LAZY_TESTER_MC02_INPUT_PROBE = 'I_ACCEPT_ONE_W_PULSE'
& 'C:\path\to\App\lazy-tester.exe' --lazy-tester-agent-cli probe-background 2
Remove-Item Env:LAZY_TESTER_MC02_INPUT_PROBE
```

The operation rejects other slots, a missing retained controller, window
identity changes, unresponsive/hidden MC02, or any foreground other than
MC01's original game window. It reports only
`WINDOW_MESSAGES_POSTED_ONLY` / `gameplayObserved:false`. Validate
**actual MC02 movement and MC01 uninterrupted keyboard/mouse** with live game
evidence. If Minecraft ignores the messages, mark this transport BLOCKED; do
not silently fall back to global input or claim parallel AI play.

## Source tests and acceptance boundary

```powershell
node --test Development\tooling\agent\mcp.test.mjs
```

`mcp.test.mjs` covers protocol discovery, hybrid-mode slot protection, serialization, input validation, and bounded evidence parsing using synthetic fixtures. These tests do **not** execute Minecraft.

Remaining real-Windows gates on one exact `Local` revision:

1. Verify the packaged executable handles `--lazy-tester-agent-cli inspect` and the existing authenticated session pipe without breaking manager startup, and that `launch` opens only the requested licensed client.
2. Confirm MC01 user can create/host a world and MC02 can join; retain exact game/account/world evidence. The native snapshot is not join/host evidence.
3. Establish one independent input backend for MC02 while MC01 retains keyboard/mouse without disruption, **before** attempting four AI clients. If unsupported, report `BLOCKED` rather than simulate a pass.
4. Verify real Script Debugger capture and `[ANALYTICS]` / `[GAME_RESULTS]` on the target Windows Minecraft Education version, including per-client source attribution.

No GitHub Actions, VMware, Sandboxie, new launcher, or developer task runner was introduced.
