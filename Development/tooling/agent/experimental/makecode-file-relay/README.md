# EXPERIMENTAL: Minecraft Education MakeCode File Relay (Windows)

**Source only, not run, not built, not verified in Minecraft.**

Purpose: bypass **foreground keyboard/mouse contention** for *supported
in-world test actions* by having each AI-controlled Minecraft Education
client read an ordinary local command file through the official
MakeCode **File Read & Write** extension. This is NOT an implementation
of genuine WASD, mouse look, block mining, item use or PvP. It does not
replace the five native processes or the existing multiplayer host/join.

Official basis:
- https://edusupport.minecraft.net/hc/en-us/articles/38490275582100-File-Read-Write-with-MakeCode
- https://minecraft.makecode.com/reference/loops/forever
- https://minecraft.makecode.com/reference/player/teleport
- https://minecraft.makecode.com/reference/positions/add

## Composition

```text
Codex / Claude -> existing MCP/AI coordinator (no new LLM)
  -> EXPERIMENTAL file-relay.mjs (NOT wired into MCP by default)
    -> C:/.../AgentData/MC02.in.txt  -> MC02 MakeCode File extension
    -> C:/.../AgentData/MC03.in.txt  -> MC03 MakeCode File extension
    -> C:/.../AgentData/MC04.in.txt  -> MC04 MakeCode File extension
    -> C:/.../AgentData/MC05.in.txt  -> MC05 MakeCode File extension
  <- per-client .ack.txt (file receipt, not gameplay proof)
  <- existing [ANALYTICS] / [GAME_RESULTS] for actual test evidence
```

MC01 remains owned by the human in `human-plus-four-ai`. In
`five-ai`, MC01 can also have the same MakeCode client project. Minecraft
Education's **normal peer-to-peer world host/join UI** remains the way
players connect. No dedicated server, new VM, privileged injector, HTTP
port, OS keyboard takeover or new native runtime owner is introduced.

## Protocol

Per AI client, a trusted existing local Windows folder contains:

- `MC02.in.txt`: `LT1|0|IDLE`, later `LT1|seq|ACTION`
- `MC02.ack.txt`: written by MakeCode, `LT1|seq|RESULT`

`file-relay.mjs` exports `prepareMailbox`, `stageMakeCodeAction`,
`readMailboxStatus`, and `renderMakeCodeTemplate`. It has no CLI
entry point and **is not an exposed MCP tool**. Each command is one
bounded action: `PING`, `STEP_NORTH`, `STEP_SOUTH`, `STEP_EAST`,
`STEP_WEST`. A step uses MakeCode `player.teleport` by exactly one
world-coordinate block. It is **NOT natural movement**; collisions,
physics, attack triggers and animation differ from WASD. The MakeCode
template does not execute arbitrary strings as game commands.

The human operator must first choose a safe disposable world and an
ordinary writable folder, add the File extension to each client project,
prepare the inbox, and start the corresponding MakeCode program.
A command can be staged only after the previous sequence is acknowledged.
A lost/mismatched ACK is an uncertain outcome, **not a retry instruction**.
Use an existing directory under user-managed non-synced private storage,
outside Minecraft account/world paths and outside Git. File paths must
remain fixed to this experiment and ordinary (no symlinks).

## Required eventual acceptance (do NOT run until explicitly authorized)

1. Pinned Windows source/compiler and installed native Lazy Tester accepted.
2. MC01 hosts, MC02 joins, MakeCode File extension genuinely reads its
   dedicated inbox while its game is unfocused and MC01 receives input.
3. `PING` round-trip, then a *single* one-block teleport succeeds on MC02;
   MC01 remains foreground and is not moved. Confirm movement from real
   in-game coordinates/telemetry, **not merely a STEP_OK file receipt**.
4. Verify four independently running MakeCode projects can poll their own
   inboxes without cross-control, and reconnect/restart never replays stale
   commands. Resource/CPU and MakeCode scope must also be measured.
5. If client MakeCode file polling is suspended or disallowed in background,
   mark this candidate **BLOCKED**. Do not automatically fall back to
   global SendInput, game DLL injection, virtual gamepads, or VM.

A file receipt does NOT authenticate a signed-in account, prove a world join,
or prove a gameplay action. The editor/extension's behavior under five
isolated Windows profiles is still UNKNOWN.