# Native Coordinate HUD — isolated JSON UI candidate v0.3.0

**Status:** SOURCE_ONLY · NOT_PACKAGED · NOT_INSTALLED · NOT_EXECUTED · Minecraft Education 26.33 compatibility UNKNOWN.

This is an **opt-in resource-only proof of concept**. Its only intended
visible change is a short, noninteractive note directly **after** the native
`player_position` in Mojang's `root_panel/chat_stack`:

    [Minecraft native Position: X, Y, Z]
    LT: XYZ notes in app
    [Minecraft's other HUD rows]

The exact display is hypothetical until Education runtime observation.

## Why v0.2 replaced the earlier v0.1 patch

The previous commit inserted a label **inside** `hud.player_position`,
whose vanilla image size is `["100%c + 6px","100%c + 2px"]`. Adding
a second child, especially one with its own `100%c` dimensions,
may disturb this native auto-sized container and Agent Pos. v0.2
**does not override `hud.player_position` at all**.

It introduces `hud.lt_native_coordinates_hint` and modifies **only**
`hud.root_panel/chat_stack` with a single `insert_after` operation
targeting the existing `player_position` child. The new sibling is a
small static label whose visibility reads the existing global
`#player_position_visible` binding. It declares no size override,
new viewport, button mappings, Script API or gameplay authority.

**Trade-off:** the chat stack may shift down by one line, and nested
control paths / `insert_after` may not load in Education 26.33.
These are unverified engine behaviors. Any overlap, orphan hint,
vanilla XYZ loss or pack conflict is a failed test — disable the RP.

## Source contents

- `manifest.json`: one `resources` module, immutable pack UUIDs,
  experimental version `0.3.0` and `header.pack_scope: world`.
- `ui/hud_screen.json`: one label and one targeted control insertion.
- `README.md`: this declaration. It is not a Minecraft pack asset.

No `_ui_defs.json` or new UI screen name is needed for the
hypothesized override of an existing vanilla HUD file; the actual
Education loader behavior is NOT verified. No other files, assets,
libraries, dependencies, executable scripts or embedded behavior pack.

The installed Minecraft binary, original profile stores, five native
session owner, original team modules and Lazy Tester
`Data/Coordinates/bookmarks.json` are unaffected. This source folder
is excluded from the official Lazy Tester distribution and existing
data-only in-game help pack. The manifest explicitly sets `pack_scope: world` so Minecraft's
normal resource-pack UI must not offer this prototype as a global pack.
This is a pack-manifest restriction, not proof that Education 26.33
respects the documented Bedrock behavior. No auto-activation or global
RP changes.

## Version-pinned research witness

Bedrock samples main, reviewed 5 Nov 2026:

- commit: `15701fa9c13923f12662984c32c124327e40e3d3`
- `resource_pack/ui/hud_screen.json` blob:
  `e2037b2d0d789a1431610a8f639ccd16e63686c3`
- Vanilla `hud.player_position` is a content-sized image with
  `player_position_text` bound to `#player_position_text`.
- Its visibility is controlled by `#player_position_visible`.
- `root_panel/chat_stack.controls` includes the instance
  `player_position@hud.player_position` immediately before
  `number_of_days_played`.

**This is a Bedrock source witness, NOT Education compatibility proof.**
Microsoft states that JSON UI overrides are not cooperative across packs.
Another world RP modifying `hud_screen.json` may silently override/conflict.

## Read-only source preflight — authored, NOT_EXECUTED

The companion `Development/tooling/agent/check-education-native-hud-source.mjs`
reads only this folder and rejects unexpected source files, symlinks,
oversized files, version drift, `pack_scope` other than `world`,
non-resource modules, UI changes outside the reviewed sibling and
fake buttons/game hooks. It does **not** build a .mcpack, download assets,
open Minecraft, install a pack, access a world or claim gameplay support.

For a separately authorized manual source check from the repo root:

```text
node Development/tooling/agent/check-education-native-hud-source.mjs
```

For an **optional** check against an operator-supplied exact Mojang
Bedrock `hud_screen.json` file, use
`--pinned-vanilla <absolute-file-path>`. The tool verifies the original
Git blob SHA, not merely a name or editable excerpt; no reference
file is downloaded. Omitting this file reports `NOT_SUPPLIED` but
does not claim compatibility.

Official Mojang source: `15701fa9c13923f12662984c32c124327e40e3d3`.
This check is source-only and must not be described as validating the
Minecraft Education renderer or multi-client behavior.

## Operator-only QA — intentionally NOT_EXECUTED

Only after separately approved Windows / Minecraft testing:

1. Use a **copy** of an Education 26.33 QA world. Record version and
   current world RP stack; do not change global resources.
2. Verify native XYZ and Agent Pos with Show Coordinates enabled and
   with this prototype **disabled**. Record a baseline screenshot.
3. Prepare an ordinary resource pack archive containing only
   `manifest.json` and `ui/hud_screen.json`. The repo does not ship,
   build or install this archive automatically.
4. Enable it in **only that copied QA world**. Verify native position,
   Agent Pos, Show Coordinates visibility, one static sibling note,
   chat position and days/scoreboard layout.
5. Repeat with small/large GUI scale, a negative XYZ, chat open,
   agents active, touch-layout where available, and a competing UI RP.
6. Disable the RP; verify vanilla HUD is restored in the copied world.
   On any regression: do not widen the patch or edit game binaries;
   retain the observed failure and leave the RP disabled.

## Dynamic in-game features: NOT implemented here

JSON UI `label` is NOT a playable Save/Load button, and this RP cannot
read the desktop bookmark JSON file or create arbitrary game bindings.
Actual in-game Save/Load, facing/dimension/chunk and permitted teleport
must be implemented by the **original team's** validated
`forms/coords/display/teleport` gameplay owners after its source and
Education API compatibility are available. Do not fork another telemetry
module or invent a new listener. See
`docs/system/education-native-coordinate-hud.md`.

## References

- https://github.com/Mojang/bedrock-samples/blob/15701fa9c13923f12662984c32c124327e40e3d3/resource_pack/ui/hud_screen.json
- https://wiki.bedrock.dev/json-ui/json-ui-intro
- https://wiki.bedrock.dev/json-ui/best-practices
- https://learn.microsoft.com/en-us/minecraft/creator/documents/practices/guidelinesforbuildingcooperativeaddons?view=minecraft-bedrock-stable
- https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log
