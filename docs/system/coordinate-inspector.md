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
