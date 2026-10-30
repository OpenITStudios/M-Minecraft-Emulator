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
