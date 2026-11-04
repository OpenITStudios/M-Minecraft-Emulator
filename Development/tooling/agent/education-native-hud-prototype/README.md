# Experimental Native Coordinate HUD resource pack

Status: **SOURCE_ONLY / NOT_INSTALLED / NOT_TESTED on Minecraft Education 26.33**.

This opt-in `resources` pack prototypes an extension of the existing
`hud.player_position` UI component rather than replacing the vanilla
`hud_screen.json`, editing Minecraft binaries or recreating the coordinates
renderer. It uses one JSON UI `modifications` insertion into
`player_position.controls` and adds exactly one **static information label**.

The vanilla `#player_position_text` and `#player_position_visible` bindings are
not redefined, so **if this partial override is accepted by the client's
JSON UI loader**, native XYZ remains owned by Minecraft. This is an intended
contract, NOT confirmed Education 26.33 acceptance.

## Present behavior

- Intended visual: native `Position: X, Y, Z` plus one modest companion line:
  `LT: Save XYZ in Lazy Tester`.
- No clickable game UI, player- or world-aware Save/Load, dynamic direction,
  dimension, chunk, saved-location count or player identification.
- No Script API, custom event, scheduler, content log, animation, font, texture,
  input, API dependency, clipboard access, network bridge or behavior pack.
- Nothing auto-installs or enables this resource pack. It is excluded from the
  standard desktop installer and existing Education team help package.
- The desktop Saved Locations notebook remains the **only implemented**
  explicit Save/List/Delete workflow and requires manual input.

## Why this is an experiment

The Mojang Bedrock sample defines `hud.player_position` as an image with
`#player_position_text` and `#player_position_visible`. Community JSON UI
documentation describes `modifications.controls.insert_back`. But Minecraft
Education 26.33 has NOT demonstrated parity or guaranteed stable merge
behavior. Microsoft's cooperative Add-Ons guidance explicitly warns that
JSON UI overrides can conflict with other packs.

The vanilla coordinate label may include an Education Agent Pos line. A
fixed static accessory could overlap a second line, another add-on, the
chat HUD or touch-scale layouts. Treat **any** overlap/vanilla-coordinate loss
as FAIL; disable this pack on the copied QA world, do not patch installed game
assets or increase global pack scope.

## Opt-in QA procedure (NOT EXECUTED)

Only after separate runtime authorization:
1. Use an isolated COPY of a QA world and save existing pack/config state.
2. Confirm the base game's Show Coordinates works without this resource pack.
3. Stage the pack as a standard Bedrock-compatible resource pack from its
   manifest and `ui/` directory; activate it ONLY on the copied world,
   never in global resources or the five profile defaults.
4. Verify vanilla XYZ still displays and updates, including negative
   coordinates and the Agent Pos case; verify one static LT companion line.
5. Repeat with GUI scales, Education UI variations, chat, scoreboard,
   multiplayer, and competing UI packs. A JSON loader refusal or overlay
   conflict is NOT a reason to patch game binaries.
6. Disable the pack and confirm exact vanilla HUD restoration.
7. Record Education version, resource pack order, original and altered UI
   screenshots, failure modes and status. Test only with approval.

## Next dynamic phase: team-owned functionality (NOT WIRED)

See `docs/system/education-native-coordinate-hud.md` for the interface
contract with the original team `minecraft-bedrock-agent-skills` owners.
A JSON UI label cannot read desktop `Data/Coordinates/bookmarks.json`,
synthesize new global binding names or dispatch a Script API callback merely
by declaring a control. A real Save/Load menu would be authored by the
original team's world-scoped `forms`/`coords`/`teleport` owners using an
approved player-origin interaction and version-compatible UI API. Do not
install a second JavaScript behavior pack from Lazy Tester.

## References

- https://github.com/Mojang/bedrock-samples/blob/main/resource_pack/ui/hud_screen.json
- https://wiki.bedrock.dev/json-ui/json-ui-intro
- https://learn.microsoft.com/en-us/minecraft/creator/documents/practices/guidelinesforbuildingcooperativeaddons?view=minecraft-bedrock-stable
- https://education.minecraft.net/en-us/resources/minecraft_education_add_ons_collection
