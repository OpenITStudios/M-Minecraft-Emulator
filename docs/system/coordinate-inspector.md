---
id: document.system.coordinate-inspector
class: DOCUMENT
domain: system
role: GUIDE
authority: WORKING
lifecycle: SOURCE_ONLY
---

# F3-style Coordinate Inspector — bounded source-only foundation

## Purpose and scope

Minecraft Education does not supply the identical Minecraft Java F3 debug
screen. Lazy Tester may eventually show position, dimension, pitch/yaw and
the already-owned native FPS measurement for the selected MC01–MC05 client.
This is an inspector, not a new Minecraft emulator, Script API analytics
module, background collector, game controller or map test runner.

P0 implements only two pure, non-I/O functions in the existing desktop app:
an unbound coordinate parser and copy-text formatting. No on-screen F3 panel,
clipboard integration, global shortcut, native IPC, game command, content-log
reader or new Script API event emitter is introduced in this step.

## Current source owner

- Desktop pure data and text: Development/apps/desktop/src/app/coordinates.ts
- Source-only regression fixtures: Development/apps/desktop/test/coordinates.test.ts
- Future desktop presentation (not implemented): existing LazyTesterApp.svelte
  and its selected-profile state; do not rearrange the fixed launcher rail
- Existing process/session authority: independent_bootstrap/session_controller.rs
  and process_identity.rs; preserve original native session ownership
- Game telemetry source authority: the original minecraft-bedrock-agent-skills
  team repository; never recreate the retired Lazy Tester diagnostics emitter
- Existing native FPS HUD: bootstrap-ordering/fps-hud.cpp; Present/Present1
  accepted-rate measurement is not proof of displayed FPS or game coordinates

## Internal P0 sample (illustrative, NOT a verified wire schema)

    {
      "schema": 1,
      "position": {"x": 123.456, "y": 64, "z": -45.125},
      "rotation": {"x": -12.5, "y": 90},
      "dimensionId": "minecraft:overworld",
      "gameTick": 100
    }

The parser enforces exact fields, finite bounded numeric values, namespaced
dimension identifiers and a nonnegative safe-integer game tick. It returns
a detached copy. It deliberately refuses fields that claim slot, player,
world, process, verified status, capture time or source provenance. Rotation
follows the Script API convention: x = pitch, y = yaw. Text for /tp uses
y (yaw) first, then x (pitch).

A parsed sample is only well-formed. Its dimension and tick are NOT
authenticated game/world identity or wall-clock freshness.

## Copy behavior

- XYZ: full numeric decimal text, invariant locale, no exponent or rounding
- BLOCK: x/y/z floored; -45.125 maps to -46
- TELEPORT: /tp @s x y z yaw pitch, text only, no command dispatch or
  cross-dimension promise
- Presentation display: x / y / z rounded to three decimals, separate
  from the unrounded clipboard text
- Chunk helper: floor(x/16), floor(z/16), including negative axes

Formatting never reads or writes the system clipboard and never mutates a
world. Even valid text cannot be presented as a current client coordinate
unless a future independent native binding and evidence source prove it.

## Required P1 integration gates (not implemented)

1. Review the ORIGINAL team Script API source for player.location,
   player.getRotation() and player.dimension.id compatibility with the
   exact Minecraft Education version. No new Lazy Tester telemetry pack.
2. Define a bounded opt-in transport in the canonical team module and
   consume it read-only through a reviewed existing evidence/native route.
   Content Log file names or a supplied slot number alone do not establish
   the game process, player, or world origin.
3. Bind the observation to the original native client's verified process
   creation identity and slot, plus an independently attributable world
   and player context. Reject ambiguous or unverifiable cross-client cases.
4. Treat world exit, client replacement, selection switch, dimension
   change and lost/stale observations as invalidation events. A gameTick
   alone is not freshness evidence; the native receiving owner must set
   observation time and bound validity.
5. Only after those gates permit valid copy should a selected-client
   inspector call a narrowly scoped write-only clipboard integration.
   Never infer F3 position from an FPS overlay, DWM screenshot or process
   metadata. Do not install a global F3 hotkey without an explicit input
   and focus conflict review.
6. Keep desktop/Windows tests and actual Minecraft runtime acceptance
   separate from authored static fixtures. SOURCE_ONLY is not tested.

Official references (review actual Education parity before use):

- https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/entity?view=minecraft-bedrock-stable
- https://learn.microsoft.com/en-us/minecraft/creator/reference/content/commandsreference/examples/commands/teleport?view=minecraft-bedrock-stable
- https://v2.tauri.app/plugin/clipboard/


## Original-team source audit and P1 candidate gate (3 November 2026)

Original user-uploaded archives (3)(1) and (3)(2) are byte-identical:
SHA-256 26c0a322cabed172ca74970413d5bb7a20d6f67181446df5f688e36057c54e21.
This is a pinned upload, not evidence about current private team HEAD.

- modules/coords/coords.ts already owns spawn and block conversion. Its
  integer X/Z +0.5 centring belongs to spawn/reference parsing; DO NOT
  silently centre copied exact player XYZ.
- modules/analytics/analytics.ts exposes opt-in tier-3 position capture.
  modules/analytics/capture.ts stores [tick, floor(x), floor(y), floor(z)]
  as position_samples; no rotation, sub-block precision or live-player
  world binding. modules/analytics/analytics-export.ts emits historical
  [ANALYTICS] output through the original scriptevent pull, not a live HUD.
- modules/display/hud-line.ts already owns multiplexed game actionbar
  display. Avoid adding an independent actionbar sender.
- Existing Lazy Tester evidence-summary.mjs keeps world/player provenance
  NOT_VERIFIED. MC01-MC05 Content Log file names alone do not authenticate
  the game process or the actor whose coordinates were captured.

The original modules cannot supply an authenticated F3 live coordinate
source in their current form. Do not reinterpret historic analytics rows
as realtime XYZ, do not add a duplicate diagnostics/telemetry pack, and do
not silently change the team module from this repository.

### Read-only candidate screen

Existing desktop src/app/coordinates.ts now exports
assessUnverifiedCoordinateCandidate and typed status reasons. This is a
pure candidate comparator, not a network reader or a proof issuer.

- Candidate shape is exactly {schema,slot,process,sample}. The slot is an
  untrusted hint, never proof of Minecraft player/profile identity.
- It screens the already selected ProfileSession and original observed
  PID, createdFileTime, Windows session ID and HWND. Matching fields
  only prove value equality, not source authenticity.
- Independently supplied receiver monotonic time has a two-second age
  bound. A time in source log cannot substitute; a recent receipt does
  not exclude replay without a native non-replay proof.
- Unknown fields claiming verification, raw player/world identity or
  timestamp are rejected. No candidate can issue its own authority.
- Every outcome has copyAllowed=false. Even matching fresh hints remain
  PLAYER_WORLD_ORIGIN_UNVERIFIED. Raw coordinates are not projected
  into the assessment result.

No new Tauri IPC, MCP tool, native DLL, HUD, global shortcut, clipboard,
game command, telemetry emitter, runtime observer or status storage was
added. Source-only regression scenarios are AUTHORED_NOT_EXECUTED.

### Unresolved independent trust boundary

The ORIGINAL team owner must explicitly approve an opt-in, bounded
live-position producer that reads local player.location, getRotation(),
dimension.id and game tick under compatible Education APIs. The original
native controller must independently attribute the delivered observation
to the exact live client process, local player and world session. It
must reject replay, changed process/world/actor, dimension transitions,
out-of-order delivery and stale samples. A matching PID, local screenshot,
selected slot, log filename, world label, self-issued nonce or actor ID
cannot independently establish that proof.

Only after an audited native proof issuer exists may selected-client
presentation and a narrow write-only clipboard action be considered.
Original testing recovery CAS remains a separate blocked feature.

Official API references:
- https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/entity?view=minecraft-bedrock-stable
- https://learn.microsoft.com/en-us/minecraft/creator/reference/content/commandsreference/examples/commands/teleport?view=minecraft-bedrock-stable

## P2 source ownership and continuity (3 November 2026)

The original uploaded team modules are canonical for all in-world
analytics and HUD display. Their analytics position_samples are opt-in
history only, with floored block XYZ but no rotation or authenticated
local-player-to-Windows-client association.

A future live-position snapshot must originate in the ORIGINAL team
module, by an explicit player-scoped request under compatible Script API
support. Microsoft documents ScriptEventSource.Server for commands
initiated by server console or runCommand, so a server-originated request
must NOT choose world.getPlayers()[0] as an assumed local user. An
entity-originated request can scope a player inside the world but still
cannot establish which MC01-MC05 Windows process controls that player.
Do not add another in-game logger or change installed team module copies.

The pure code is now split for maintenance:
- Desktop src/app/coordinates.ts owns validation, XYZ/block/chunk and
  teleport text. It never reads or writes clipboard.
- Desktop src/app/coordinateEvidence.ts owns only UNVERIFIED candidate
  shape comparison, profile/process hints and receiver-time screening.
- Separate authored Vitest suites correspond to the two owners.

The new assessUnverifiedCoordinateSequence pure function compares
consecutive candidate frames and returns a denial status for a changed
slot, PID/birth/session/HWND, dimension, duplicate/rewound game tick,
invalid or excessive monotonic receiver interval. It holds no state and
starts no scheduler. Continuous samples still report only
CONTINUOUS_BUT_UNVERIFIED with copyAllowed=false. This check cannot
prove source authenticity, local actor ownership or world identity.

A real user-facing F3 requires an independently attributable source
for each client and an original-controller admission binding that
rejects source mismatch and stale/replayed world sessions. Neither
an actor-provided ID nor a log filename is proof. Native transport,
clipboard, UI, global hotkeys, Script API emitter and live source are
NOT_IMPLEMENTED. Windows and Minecraft acceptance are NOT_EXECUTED.

References:
- https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/world?view=minecraft-bedrock-stable
- https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/scripteventsource?view=minecraft-bedrock-stable

## P3: Bedrock Copy Coordinate UI — manual text fallback (3 November 2026)

Microsoft's Bedrock creator documentation describes **Enable Copy Coordinate UI**
under Settings > Creator. Where supported, Ctrl+Alt+C copies player position
and Ctrl+Alt+X copies the selected block position. Bedrock clipboard text
can be pasted into another Windows text field.

**Minecraft Education 26.33 compatibility is UNVERIFIED.** The Bedrock
documentation does not establish that the same toggle/shortcut exists in
Education. Do not auto-send the shortcut to a game or claim live support.

The pure desktop coordinate owner now accepts *explicitly pasted* XYZ text
as USER_PASTED_UNVERIFIED. No source, player, world, dimension or process
identity is inferred. Accepted input is one bounded line of three signed
ASCII decimal numbers (spaces/tabs or comma separators). Unexpected extra
tokens, commands, multiline, exponent notation and out-of-range values
are rejected. Outputs are XYZ, floored BLOCK and /tp @s XYZ **without**
any fabricated yaw/pitch. The parser never reads the OS clipboard or
executes a command. No desktop UI or live game transport was wired.

This fallback is independent from the blocked native F3 live source. A
future manual paste interface may use it with explicit provenance labels;
the separate original-controller player/world proof gate remains required
for automated live coordinate capture and any automatic per-client claim.

Authoritative Bedrock feature guide:
https://github.com/MicrosoftDocs/minecraft-creator/blob/main/creator/Documents/CopyCoordinates.md
Education changelog:
https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log

## P4: optional manual coordinate preview (3 November 2026)

The existing Play header now contains one collapsed **Paste XYZ**
disclosure in `Development/apps/desktop/src/CoordinatePastePanel.svelte`.
The operator manually enters XYZ; the component uses the existing
pure `formatUserPastedCoordinates` for exact XYZ, floored BLOCK and
position-only `/tp @s ...` text. All output is **User-pasted · Unverified**.

`Select text` focuses and selects the readonly output. The operator
explicitly presses Ctrl+C; this component never reads or writes the
clipboard, sends a Minecraft command, starts polling, invokes native
IPC, installs an in-game module, or persists input. The input and
output clear on disclosure close and when Play unmounts.

The popup is absolutely positioned in the existing Play header; no
additional page, game viewport, original session manager, DWM preview
layout or five-slot launcher card is created or rearranged.

This is NOT live F3 and does NOT establish a source player, world,
dimension, process or MC01–MC05 identity. Bedrock's Copy Coordinate
UI availability in Minecraft Education 26.33 remains NOT_VERIFIED.
New Vitest fixtures are AUTHORED_NOT_EXECUTED. No TypeScript build,
Windows UI, Minecraft runtime, package, CI or gameplay was executed.
