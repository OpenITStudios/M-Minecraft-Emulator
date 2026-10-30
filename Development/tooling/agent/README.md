# Windows AI test bridge — source implementation

The bridge exists for **Minecraft Education peer-to-peer client testing** using the five native profiles owned by Lazy Tester. It does **not** start a dedicated server, create a second game controller, inject input, move players in background windows, or modify account/world data.

## Exactly what works in source

- A local **stdio MCP** server in `mcp.mjs` exposes four tools: `agent_capabilities`, `inspect_profiles`, `launch_profile`, and `read_game_evidence`.
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
