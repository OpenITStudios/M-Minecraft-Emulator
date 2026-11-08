---
id: document.system.minecraft-education-native-f3-client
class: DOCUMENT
domain: system
role: GUIDE
authority: WORKING
lifecycle: SOURCE_ONLY
---

# Minecraft Education 26.33 — F3 native CLIENT feature

**Operator requirement (8 November 2026):** Modify only the Minecraft
Education Windows **client** so a single physical F3 press toggles a
Java-style debug HUD **within the game's own rendering**, in any world.
Do **not** build an Add-On installed per world, a Script API gameplay
listener, a menu/modal, a LazyTester UI panel, or an independent
Windows-layered overlay. The same client feature must be independent per
MC01–MC05; no changes to world/game rules, accounts, profiles or saves.

**Status: ENGINE ABI UNKNOWN; IMPLEMENTATION BLOCKED, NOT COMPLETE.**
This document is an exact-owner architecture decision, NOT a feature
implementation or proof of operating Minecraft. The pinned GitHub
`Local` source at `f770ad43f2e5c3dcf54ddc6ab96ba5e44003a5f0`
contains no tracked `.exe` or `.dll` game binary and no verified
Minecraft Education native Player/HUD ABI. Do not guess offsets, use
Bedrock vtable values as Education proof, or silently enable hooks.

## Why native Show Coordinates is NOT a ready-made F3 API

- Mojang's Bedrock `hud.player_position` consumes the native binding
  `#player_position_text` and `#player_position_visible`. These are
  **consumer labels**, not an externally callable API for toggling an
  arbitrary debug panel or obtaining a native Player pointer.
- Bedrock/Education "Show Coordinates" is controlled by a world option/
  `showcoordinates` game rule. Reusing that rule would **modify world
  state**, violate this request, and still display only the native
  coordinate line rather than Java's F3 content.
- The standard Bedrock Script API `InputButton` exposes `Jump` and
  `Sneak`, not raw `VK_F3`; world Behavior Packs and `/scriptevent`
  cannot establish a physical F3 press.
- Bedrock's global Resource Pack facility does not supply an input
  callback or an Education-compatible arbitrary native HUD data binding.
  JSON UI is version-sensitive and increasingly supplanted by Ore UI.
  The old world-scoped RP remains archived/source-only, not a route for
  this client feature.
- Bedrock client loaders (such as LeviLamina) illustrate a plausible
  **in-process** input/render integration model, but their supported
  Bedrock binaries are NOT evidence of compatibility with Education's
  distinct signed client and original launcher. Do not install a
  third-party loader or patch the official executable based on analogy.

## Existing, verified-as-source owners

| Responsibility | Actual current source | What it proves / does not |
| --- | --- | --- |
| Exact owned game process and private profile start | `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs` | One owned Education game process identity; not a Player-world pointer |
| Reviewed native presentation candidate | `Development/tooling/acceptance/windows/bootstrap-ordering/presentation-enrollment.cpp` | Existing OGLES-owned DXGI interception path; not a native debug screen |
| Wrapped Present/Present1, FPS pacing | `.../dxgi-pacing.cpp` | Native frame acceptance measurements; no world coordinates, biome or player info |
| Measured FPS panel | `.../fps-hud.cpp` | Uses `WS_EX_LAYERED`/`UpdateLayeredWindow` in a separate native window. **Explicitly NOT the requested in-game F3 renderer** |
| Legacy F3/coordinate research | `Development/tooling/agent/education-debug-prototype/` | Pure ScriptEvent/Player data plans only. World/Script API path superseded for this feature |
| Old JSON UI RP | `Development/tooling/agent/education-native-hud-prototype/` | Source-only world-scoped additive hint; do not install, package or extend as F3 |

No Python/OCR, game Content Log, screen pixel parser, desktop Saved
Locations file, player nickname, Windows PID, controller ACK or DWM
preview supplies the same live game-local Player state.

## Required in-process architecture (NOT YET IMPLEMENTED)

The **one existing original owned-game bootstrap** retains process,
library and window authority. A future independent, **default-OFF**
version-pinned client extension may be admitted only when its target
Minecraft Education image/version/signature and relevant native
interfaces are verified. It must not replace the already reviewed
session controller, alter the game executable on disk or change
authentication/profile storage.

1. **F3 input owner:** receive the actual `VK_F3` key transition from
   the verified **OGLES game window's own thread**, while that exact
   window is foreground and gameplay receives keyboard input. One
   key-down edge toggles once; autorepeat and long holds never flicker.
   Ignore chats, menus, other apps, stale/recycled HWND, wrong
   process/thread and synthetic controller-injected input. Re-verify
   both before action and on teardown. How Education handles real
   WM_KEY*/Raw Input needs direct target evidence.
2. **Game-local debug data:** resolve the original active client Player,
   camera, dimension, position and look direction by a **version
   verified engine interface**, read on an engine-safe thread, without
   modifying world state. Prefer existing native coordinate-provider
   data if its ABI is actually established. Never copy Java/Bedrock
   offsets by version resemblance. All unknown fields stay
   `Unavailable`; absent Player hides F3 immediately.
3. **Renderer:** compose text directly through the original client's
   in-game HUD/render path (or a proven safe in-swap-chain stage), on the
   correct game/render thread, preserving D3D11/D3D12 state and
   resource lifetime. No `WS_EX_LAYERED`, separate top-level
   window, WebView/Svelte, DWM preview, modal form, JSON UI
   world pack, actionbar, chat or scoreboard owner. An independent
   DLL can be a *client extension* only when the engine render interface
   is verified and failure is reversible.
4. **Copy Coordinates:** only after the same live Player identity and
   client-owned XYZ text are verified, an explicit in-game shortcut
   may write `CF_UNICODETEXT` to the Windows clipboard from the
   authorized user process. Do not copy implicitly when F3 opens;
   report failure rather than inventing a successful paste. Clipboard
   is a user action, not world data, telemetry or a network message.
5. **Lifecycle:** toggle OFF on lost foreground, world leave,
   game reconnect, device/swapchain reset, renderer invalidation,
   shutdown or uncertain input. Never carry old data across worlds.
   The other four Education processes keep isolated state.
6. **Packaging:** preserve Microsoft game image exactly; do not
   distribute or patch a proprietary game executable. Do not
   opt the normal LazyTester product into unreviewed extensions.
   Any candidate must be operator-opt-in, source-reviewed and
   separately accepted on Windows first.

## Required debug content for Java-style parity

The user requested a usable **Java-style F3 debug screen**, not merely
a small coordinate label. The minimum accepted information profile is:

| Section | Required fields and source |
| --- | --- |
| Game | Minecraft Education exact client version/build and session status from this process |
| Player | Precise XYZ (including decimals), floored Block XYZ, within-chunk XYZ, chunk XYZ, yaw/pitch, cardinal facing, world dimension |
| Environment | Biome, light values (when available), targeted block/fluid and distance, day/time; only if the exact Education engine supplies verified live values |
| Performance | Actual FPS and frame time from owned Present/Present1; GPU/renderer identity and memory metrics only from established trustworthy native APIs |
| Camera/interaction | Camera target and crosshair details only after an exact same-client gameplay accessor is proven |
| UX | Readable left/right debug layout, F3 toggle, focus-safe behavior, no occlusion of chat/crosshair/inventory, and copy coordinates to clipboard from live values |

Java Edition's exact F3 contents are version-dependent. A final product
must define and accept the fields above against real Education captures;
it must not display zeros, random values, PID-derived pseudo-coordinates,
world-log guesses, fabricated biome/light data, or stale panel text. Fields
not obtainable from Education must be explicitly flagged as a
**feature-parity blocker** rather than silently dropped and declared DONE.

**F3+C** is the intended optional explicit Copy Coordinates gesture while
F3 is open, subject to confirming it does not conflict with an existing
Education input mapping. No automatic clipboard writes.

## Native client source slice — prepared, default OFF, NOT_EXECUTED

The former plan-only route now has a **real C++ source slice** inside
`Development/tooling/acceptance/windows/bootstrap-ordering/`:

- `education-client-f3.h/.cpp` define a process-local OGLES-window
  session, single physical F3 key-down/up toggle intent with autorepeat
  suppression, optional explicit F3+C clipboard write, bounded live
  Player-frame validation with world-generation and 250-ms freshness
  invalidation, and teardown on focus/device/world change. The source
  now requires the original engine callback's `capturedTickMs`
  timestamp and strictly increasing world-generation number, so late
  snapshots cannot become fresh at delivery or revert to a retired world.
- The source-only D3D11 compositor writes directly into the existing
  game DXGI swapchain **backbuffer**, via Direct2D/DirectWrite and without
  creating a `WS_EX_LAYERED` or other separate Windows overlay.
  Native key admission now ignores WM_SYSKEYDOWN/Alt+F3 and only
  treats an ordinary F3 press/release as the requested toggle.
  Hidden F3 returns without its former per-Present COM/D2D/FPS
  workload; FPS is sampled from the existing DXGI owner only when
  debug text is drawn. Rendering requires D3D11.1 context-state
  save/restore to preserve the game's immediate pipeline across D2D
  drawing, and refuses devices without that interface. This is
  source-only protection, NOT proven Education render safety.
  Admission now requires the actual existing D3D11 device to expose
  `D3D11_CREATE_DEVICE_BGRA_SUPPORT`; an unsupported device is
  refused once for its current chain, not probed every Present. It
  releases backbuffer references before both `ResizeBuffers` variants
  and on final native chain teardown. D3D12/HDR/unsupported devices
  fail closed; render or F3 intents are never called a drawn receipt.
- F3+C no longer holds the F3 session's exclusive SRW lock while
  calling OpenClipboard/EmptyClipboard/SetClipboardData/CloseClipboard.
  The key handler snapshots the single original Player frame under
  the lock, releases it, validates the unchanged world-generation,
  original snapshot timestamp, HWND and foreground, and then attempts
  one copy. Clipboard bytes are prepared before opening the system
  clipboard. Windows may send clipboard-owner notifications; it must
  not be able to reenter a lock held by the game render path.
  A Copied receipt reflects OS SetClipboardData success, not
  a verified Minecraft input route or pasted result.
- Backbuffer acquisition errors (GetCurrentBackBufferIndex bounds,
  GetBuffer, CreateBitmapFromDxgiSurface, EndDraw) now invalidate
  Direct2D-held targets and refuse further drawing on that swapchain
  until genuine ResizeBuffers or retirement. Unsupported frame paths
  cannot repeatedly recreate expensive GPU resource bindings.
- `dxgi-pacing.cpp` calls the compositor only with
  `BOOTSTRAP_EDUCATION_CLIENT_F3_SOURCE` set. The standalone CMake
  option is **OFF**; normal DLL and product packaging remain unchanged.
  The initial input/Player source is intentionally NOT registered by
  `compat.cpp`, `presentation-enrollment.cpp` or the manager.
- The renderer's current text layout includes native XYZ, block/chunk,
  facing, rotation, dimension and the already measured FPS field.
  This is **not full Java debug parity**; biome/light/targeting and
  other game-specific fields still need actual Education engine access.
  No hardcoded Player pointer, made-up coordinate value, screenshot
  extraction or world gamerule is used.

**Source boundary:** `claim_game_window`, `publish_player`, and
`game_key` are unbound input functions for a future source-verified
Education engine adapter. A HWND/class name alone does not authenticate
gameplay, raw key provenance or a Player object. Until an exact native
source is established they are never called, the compositor is inactive,
and **F3 cannot work in Minecraft**. Direct2D/D3D11 compatibility, state
restoration, main/render-thread scheduling, copy-to-clipboard and
MC01–MC05 isolation are all NOT_EXECUTED; source-only code is not
Windows runtime acceptance. No test, build, native or game execution was
authorized by this change.

## Read-only PE evidence collector — source prepared, NOT_EXECUTED

`Development/tooling/acceptance/windows/inspect-education-f3-pe.ps1`
is a developer-only, explicit-call **metadata** collector. It uses
the existing `Resolve-ManagerEducationTarget -RegisteredOnly` to
locate exactly one registered signed Education Desktop executable.
It reads bounded AMD64 PE header/sections, optionally decodes
a *reference* to CodeView/RSDS symbol metadata, hashes the entire
existing image using SHA-256 and emits JSON without local paths,
binary bytes, PDB source paths, or account/world data.

Neither an RSDS reference nor a PE import proves a callable game
`Player` interface, raw F3 key event, or in-game render hook. The
report explicitly marks those ABI fields `NOT_ESTABLISHED` and
does not launch or attach to Minecraft. It is not an F3 mod, is not
connected to LazyTester startup, does not install a DLL or enable
CI. The exact Education 26.33 build and native interfaces remain
unverified until separate authorized evidence and Windows execution.

## Real integration evidence that is MISSING

| Gate | Evidence required | Now |
| --- | --- | --- |
| CLIENT-01 | Actual authorized Windows Education **26.33** game executable identity, compatible symbol/API evidence, exact build hash | NOT_AVAILABLE |
| CLIENT-02 | Valid game-thread F3 edge input without stealing focus or another MCxx input | NOT_VERIFIED |
| CLIENT-03 | Native Player/coordinate accessor that works in multiple worlds without toggling gamerules | NOT_VERIFIED |
| CLIENT-04 | In-client compositing on actual Education D3D path (not an OS layered window) | NOT_VERIFIED |
| CLIENT-05 | Positive runtime screenshot + state readback for F3 on/off and movement/facing/dimension | NOT_EXECUTED |
| CLIENT-06 | Native clipboard Ctrl+V matches selected client's real XYZ | NOT_EXECUTED |
| CLIENT-07 | MC01–MC05 isolated launches/login/multiplayer, focus changes, crash-safe removal and clean rollback | NOT_EXECUTED |

**Necessary next engineering step:** obtain independently authorized
*read-only* evidence from the exact installed Education 26.33 client
for its keyboard routing, native coordinate provider and render ABI.
Then implement all three connected owners (input + live Player data +
in-game renderer) in one version-gated feature slice, followed by
actual target-Windows/Minecraft acceptance before calling this
feature DONE. Metadata or source-only mock tests are not a substitute
for these three runtime dependencies. Never ship a F3 key handler
with a blank/stale panel.

## Do not confuse three different surfaces

- **Native Minecraft game HUD**: target, must be inside client rendering.
- **A native owned layered window on top of Minecraft**: technically a
  Windows overlay, even if injected from the same process; **rejected**.
- **Resource Pack/Script API panel inside a particular world**:
  world/gameplay extension; **rejected** for the F3 requirement.

## Source and compatibility evidence

- Mojang sample HUD fields: https://github.com/Mojang/bedrock-samples/blob/main/resource_pack/ui/hud_screen.json
- Education F3 support answer: https://edusupport.minecraft.net/hc/en-us/community/posts/360074495351-F3-Doesn-t-Work
- Education version 26.33: https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log
- Community loader client mechanics, NOT Education compatibility: https://docs.levimc.org/LeviLamina/developer_guides/architecture/
- Minecraft modification distribution: https://www.minecraft.net/en-us/usage-guidelines

**Proof ceiling:** REMOTE_GITHUB (source/architecture review) only.
No Windows/game execution, target image reverse engineering, DXGI
hook installation, EULA compatibility/legal opinion or gameplay QA
was performed. No tests were executed or CI jobs enabled.
