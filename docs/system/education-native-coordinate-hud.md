---
id: document.system.education-native-coordinate-hud
class: DOCUMENT
domain: system
role: GUIDE
authority: WORKING
lifecycle: SOURCE_ONLY
---

# Education native coordinate HUD — opt-in JSON UI prototype

**Project:** Lazy Tester / M-Minecraft-Emulator, `Local` branch.
**Status:** SOURCE_ONLY, compatibility NOT_VERIFIED, not installed.
**Intent:** extend the vanilla `hud.player_position` component *without*
recreating Minecraft's location logic or installing a second gameplay runtime.

## What can be genuinely reused

The Mojang Bedrock sample `resource_pack/ui/hud_screen.json` defines:

- `hud.player_position`: an image whose child label uses
  `#player_position_text` to display native player and possible Agent Pos.
- `#player_position_visible`: existing HUD visibility control.
- `hud.root_panel`: the general HUD controls owner; changing the complete
  screen or replacing `root_panel` is unnecessary for a minimal test.

The experimental Resource Pack v0.2.0 lives in
`Development/tooling/agent/education-native-hud-prototype/` and
**does not modify the `player_position` definition**. Instead it
declares one static `hud.lt_native_coordinates_hint` and inserts
the label immediately *after* the existing `player_position`
instance in `root_panel/chat_stack` via `insert_after`.
The label observes the native `#player_position_visible` binding.
Minecraft still owns the original content-sized position image and
`#player_position_text` data. Its nested-path insertion behavior
and chat-stack layout are **not yet runtime verified**. The hint is
not a button or player/world data source. No touch, hotbar, actionbar,
scoreboard, F3/FPS, Script API or telemetry owner is added.
Nothing in the standard Lazy Tester build installs the pack.

**Compatibility warning:** Microsoft's cooperative Add-Ons guidance says
JSON UI overrides cannot be combined reliably as cooperative Add-Ons.
Minecraft Education 26.33-specific JSON UI merge behavior, agent-specific
position rows, GUI scale and RP priority are NOT verified. This is an
explicit operator-only disposable-world prototype, not a promised supported
product path.

## Rendering and capability boundary

| Feature | Owner | Current status | Hard limitation |
| --- | --- | --- | --- |
| Vanilla XYZ and Agent Pos | Minecraft `hud.player_position` | Existing native binding | Do not re-render, parse, overwrite or suppress |
| Static sibling hint after XYZ | This optional RP | Authored, NOT_TESTED | Nested path may fail; possible chat-stack reflow |
| Facing / pitch / yaw | Existing team gameplay `Player` | NOT_CONNECTED | No native JSON UI binding established |
| Dimension and chunk | Existing team gameplay `Player` | NOT_CONNECTED | A separate, supported UI data path is needed |
| Saved location count | Existing team world/player storage | NOT_CONNECTED | Desktop data is not accessible by in-game JSON UI |
| In-game Save / List / Delete | Original team forms/coords owner | NOT_IMPLEMENTED | JSON UI controls do not automatically call Script API |
| Teleport | Original team teleport owner | NOT_IMPLEMENTED | Require explicit player request and authorization |
| Copy XYZ | Native Bedrock Copy UI, if on Education | NOT_VERIFIED | Script API/JSON UI has no proven desktop clipboard bridge |
| Desktop Saved Locations | Existing Lazy Tester Rust notebook | SOURCE_ONLY implementation | MCxx/world labels are operator-declared, not game identity |

## Rules for future real in-game features

The team-managed `minecraft-bedrock-agent-skills` modules are the *sole*
gameplay and storage authority. Its pinned, approved ZIP source is referenced
by `Development/tooling/agent/team-source-baseline.json`, but the live
team repository is not available through the connected GitHub account at
this review. Therefore **DO NOT vendor/fork/modify those team modules** from
a guessed interface. No second Script API subscription, HUD writer,
filesystem bridge, command processor, remote listener or telemetry engine.

When the upstream module owner can be reviewed and Education API support
verified, propose one narrow extension via the team's actual, existing
`coords`, `display`, `forms`, `teleport` owners:

1. **Player-origin event only:** the same original player object must
   initiate the request. Never use `world.getPlayers()[0]`, a supplied
   display name, Windows PID or Content Log row to pick a player. Refuse
   unbound/server-origin, spectator-only, revoked or ambiguous context.
2. **World-scoped notes:** Save requires a user-entered location name and
   stores validated finite XYZ, actual dimension, world-scoped epoch and
   original player identity via the team's approved persistence owner
   (if its API supports it). No automatic export or overwrite of current
   `Data/Coordinates/bookmarks.json`; desktop and world scopes differ.
3. **Bounded/listed actions:** List, Rename/Delete with explicit confirmation,
   optional chunk/facing/dimension projection, max record count and no
   background per-tick polling. Check world settings, gameplay permissions,
   dimension, spawn safety and user approval before any teleport. Teleport
   failure does not delete/overwrite saved position. No automatic cross-world
   or cross-client teleport.
4. **Menu is not a JSON label:** any Save/Load controls need a real reviewed
   event route to a version-compatible `@minecraft/server-ui` form or an
   existing team form. A static JSON UI element cannot call JavaScript.
   Do not invent a new global shortcut, chat trigger or clickable fake button.
5. **HUD data channel:** if the team has an existing compatible display API,
   assess whether it can show a separate nonconflicting companion panel
   without taking ownership of vanilla `#player_position_text`, map
   actionbar, scoreboard or chat. If no supported binding/channel exists,
   display dynamic extras only in the approved in-game form, not as fake
   live text in vanilla HUD.
6. **Isolation:** a shared multiplayer world may have several MCxx clients.
   World `Player` provenance does not identify the Windows client, and an
   MCxx bookmark label is not in-game identity. Never sync across those
   boundaries without independent proof and review.
7. **Reversibility:** no global RP installation, no automatic world imports,
   no game binary editing, no modifying pre-existing world assets, no
   unreviewed upgrade of team lockfiles. Uninstall/disable the optional
   prototype RP to restore vanilla UI.

## Specific, evidence-based runtime acceptance plan — NOT_EXECUTED

| ID | Condition | Accept | Refuse / stop |
| --- | --- | --- | --- |
| HUD-01 | Education 26.33 copied world with Show Coordinates ON and **no** prototype | Native XYZ (and Agent Pos when applicable) visible | Native setting blocked or coordinate field missing |
| HUD-02 | Activate prototype RP *only* on copied QA world | Native XYZ still reads and updates; one small sibling hint below the position row | Client refuses nested patch, coordinate missing, or text/day/chat overlap |
| HUD-03 | Flip Show Coordinates OFF | Both native XYZ and new hint disappear (same visible signal) | Hint remains visible when native field is disabled |
| HUD-04 | Small/large GUI scales, windowed/fullscreen, touchscreen if available | No overlap with agent, chat, scoreboard, inventory/hotbar, safe zones | Clipping, blocked controls or unreadable position |
| HUD-05 | Competing RP/education pack | Clear incompatibility evidence and safe disable | Never force pack priority or patch vanilla files |
| HUD-06 | Disable prototype and reload copied world | Exact vanilla HUD restored without loss of world/profile data | Any persistent UI defect |
| HUD-07 | Existing desktop manual notebook | Save/List/Delete work *separately* without automatic game data claims | Cross-slot/player/world attribution is fabricated |
| HUD-08 | Team-owned future dynamic UI | Version-verified player-origin menu with correct permission/ownership tests | Untrusted `@s`, unmatched client, unsupported API |

**Evidence labels:** REMOTE_GITHUB authoring is NOT source execution. An
authored source check is NOT a pass. No Node, TypeScript, Rust, Windows,
resource pack packaging, Minecraft, multiplayer, five-client or Education
HUD screenshot acceptance has been executed or observed for this prototype.

## v0.2 source correction and pinned Mojang witness (5 November 2026)

The previous v0.1 prototype added a second label **inside**
`hud.player_position.controls`, whose native image size is
`100%c + 6px` by `100%c + 2px`. The old child declared
its own `100%c` size, risking unexpected content-size growth
and Agent Pos overlap. The v0.2 source deliberately inserts a
short sibling in the existing chat stack and uses the vanilla
visible binding without altering the native position definition.

Pinned Mojang Bedrock revision:
`Mojang/bedrock-samples@15701fa9c13923f12662984c32c124327e40e3d3`,
`resource_pack/ui/hud_screen.json` blob
`e2037b2d0d789a1431610a8f639ccd16e63686c3`.
This is a **Bedrock reference**, not a verified Minecraft
Education 26.33 JSON UI schema or a guarantee of merge behavior.

An `insert_after` on the nested
`root_panel/chat_stack` path may fail if the Education HUD tree
is different. Any UI rearrangement or orphan hint must cause a
failed runtime test and RP deactivation, not a second override.
The source test was authored, not executed. No pack was built,
installed or used in gameplay.

## References

- Mojang vanilla Bedrock HUD sample:
  https://github.com/Mojang/bedrock-samples/blob/main/resource_pack/ui/hud_screen.json
- Bedrock Wiki JSON UI modifications:
  https://wiki.bedrock.dev/json-ui/json-ui-intro
- Microsoft's official cooperative Add-Ons warning:
  https://learn.microsoft.com/en-us/minecraft/creator/documents/practices/guidelinesforbuildingcooperativeaddons?view=minecraft-bedrock-stable
- Minecraft Education Add-Ons:
  https://education.minecraft.net/en-us/resources/minecraft_education_add_ons_collection
- Minecraft Education release notes:
  https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log
- `@minecraft/server-ui` API:
  https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server-ui/minecraft-server-ui
