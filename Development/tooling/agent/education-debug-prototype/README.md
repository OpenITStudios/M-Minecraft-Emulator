# Minecraft Education F3-like Debug Data — source-only adapter

**Status:** `SOURCE_PREPARED / NOT_EXECUTED`. This folder does **not** contain an installable Add-On, listener, key hook, UI overlay, clipboard bridge or LazyTester desktop feature.

## Intended in-game behavior

- F3 key release **while the correct Minecraft Education game is actually receiving keyboard input**: toggle that player's debug panel.
- F3 again: hide it; do not leave information visible after exit/world change.
- The panel uses a snapshot from the same original `Player` (location, getRotation, dimension). Never select a player using `world.getPlayers()[0]`, MCxx labels, Windows PID, or Content Log.
- Copy Coordinates is an **in-game request** and must use a genuinely verified Minecraft clipboard facility. `copyCoordinatesText` is prepared text, **not evidence of having copied anything**.
- Use the original team's `coords`/`display`/`forms` owners; do not install another HUD writer or compete with actionbar, scoreboard, or any map's UI.

## Pure data formatter prepared

`format-player-debug.mjs` has one exported `formatPlayerDebugSnapshot` function. It accepts plain values that an approved team owner can derive from **the same** `Player.location`, `Player.getRotation()` (`x` pitch, `y` yaw), and `Player.dimension.id`. It returns immutable debug text lines and integer block-coordinate text.

A direct Java-style F3 toggle cannot currently be wired through the stable documented Bedrock Script API `InputButton` enum (only Jump/Sneak are exposed). Minecraft Education does not ship Java's F3 debugger. **Do not claim pressing F3 will show the panel on a real game.** A future client-side input mechanism, if available and authorized, must demonstrate F3 key-up provenance for exactly one foreground game and a supported in-game rendering bridge. Do not silently substitute a LazyTester popup, OS overlay, global hotkey, chat automation, or CUA input injection.

The team source `achmadawdi/minecraft-bedrock-agent-skills` is not available through the connected GitHub scope; do not fabricate or fork its subscriptions/interfaces. The existing optional RP at `../education-native-hud-prototype/` is a static note, not a dynamic F3 renderer.

## Candidate for the original team's in-game event owner

`in-game-toggle-intent.mjs` adds **one pure adapter**, not a new runtime:
`planPlayerSourcedDebugToggle(event, {eventId, currentlyVisible})` reads
only the exact `sourceEntity` when the existing team's Script API event
already says `sourceType: 'Entity'`, the entity is
`minecraft:player`, the **team-chosen** namespaced `eventId` matches,
and the message equals `toggle`. It produces a `show` intent with
player-derived debug text, or a `hide` intent without touching a possibly
stale player. The **team** retains the per-player visibility state and
must render/clear the panel in Minecraft. This adapter never subscribes to
`scriptEventReceive`, and no specific event ID is registered or shipped.

**Potential Bedrock-only command-macro experiment:** Bedrock 26.x offers up
to ten in-game command macros using `Alt + key`; the existing
`/scriptevent` command can reach a team's *already registered* listener.
This needs Game Directors/cheats for `/scriptevent`. Minecraft Education
26.33 support and physical key origin are UNKNOWN. Even if a macro is
bound to **Alt+F3**, it does not satisfy the required single-key **F3**
Java-style toggle. An `Entity` source alone also does not authenticate
a physical F3 keypress: chat, NPC/executed commands and scripts have
separate origin and permission concerns. No macro, command, handler or
HUD has been installed.

Do not create a second telemetry module, event subscription, state
database, `@minecraft/server-ui` form or actionbar/scoreboard writer in
LazyTester. The team repository is not visible through this GitHub
connection, so this source is only an integration candidate.

## Acceptance proof needed, not yet executed

1. Prove keyboard F3 event delivery **inside the Minecraft client** with world loaded, chat closed, and game foreground; a second F3 hides it.
2. Prove the player-specific dynamic data channel without replacing native `hud.player_position`, scoreboard/actionbar, or existing team owner.
3. Test MC01-MC05 identities independently; F3 for one game never toggles a peer or desktop UI.
4. Verify negative XYZ/chunk boundaries, four cardinal yaws, Nether/End, resize/GUI scale and game reconnect.
5. Verify clipboard paste result from the **Minecraft client**, or explicitly label Copy unavailable. Never infer clipboard success from `copyCoordinatesText`.
6. Preserve disabled/off-by-default policy and copied QA world. Windows, Bedrock/Education version and F3 input remain NOT_VERIFIED.

Official reference: https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/inputbutton
Education F3 limitation: https://edusupport.minecraft.net/hc/en-us/community/posts/27889945431572-How-to-display-the-debug-screen-in-minecraft-education-edition
