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

## P5: per-client live coordinate source feasibility (3 November 2026)

### Technical decision: keep game player, client process and world origin separate

This research compares source capabilities; it has **no executed Windows
or Minecraft Education 26.33 result**. The current original Rust session
controller still owns all MC01-MC05 process/window lifetimes. No new game
controller, in-game telemetry module, desktop status store or listener
is necessary for this documentation milestone.

| Candidate | Potential value | Evidence still missing | Priority |
| --- | --- | --- | --- |
| Official Bedrock Copy Coordinate UI | User-triggered XYZ text in clipboard | Education 26.33 parity, clipboard source | Existing manual P4 fallback |
| Education Code Builder / MakeCode player.position() | Position of the current player in that Code Builder context | Local client-to-desktop output, precision, orientation, version behavior | First client-local research candidate |
| Education /connect (/wsserver) | Client-originated connection to a companion websocket | Actual 26.33 capability, cheats/permissions, safe read-only message protocol | Opt-in research only |
| Windows GetExtendedTcpTable | TCP connection 4-tuple and OS-reported owning PID | Process created FILETIME, socket possibly owned by helper process, player/world provenance | Possible original native read-only witness |
| Original-team Script API | Precise in-world position, rotation and dimension for explicitly selected Player | Server scripts see all players; host world does not identify the specific local Windows client | Only after independent client/player binding |
| Existing verified-window GPU capture | Pixel frame tied to a game PID/HWND observation | Onscreen coordinates may be disabled, stale or misread | Manual visual evidence, not trusted automatic XYZ |

**Do not combine two unverified mechanisms into an assumed working API.**
The documented Code Builder player.position() feature and the /connect
websocket protocol are distinct. No source has proven that Code Builder
on Education 26.33 can emit client-scoped position events to a Lazy
Tester-owned localhost endpoint. The MakeCode reference does not
guarantee sub-block precision, dimension or yaw/pitch. Full F3 will
need additional evidence.

The official Education changelog lists a historical Code Connection
1.5 client compatibility ceiling of 1.18.xx, **and** a newer 1.X
entry compatible with clients 1.19.xx and higher. Neither entry is
specific acceptance evidence for 26.33. Do not use legacy version 1.5
as a 26.33 dependency; do not assume newer 1.X is verified either.

The official Minecraft Creator multiplayer guide states server Script
API runs in shared server-side context. Never choose
world.getPlayers()[0] as a local-client guess, and never promote
ScriptEventSource.Server to an authenticated player-origin request.

### Candidate native process-bound path — designed, NOT implemented

If and only if a separately approved Education runtime experiment
confirms per-client /connect (or an equivalent client-local transport),
the following is a possible one-controller integration:

1. Operator intentionally selects one slot and requests an opt-in
   *read-only* capability check. Launch, Refresh and window selection
   must not silently start any bridge or send game input.
2. The existing original native controller verifies its exact managed
   process handle, PID, creation FILETIME, Windows session and game
   window for that slot. A PID or title match alone is not a lease.
3. For a deliberate loopback connection, any future listener must bind
   only 127.0.0.1 or ::1, never a remote/wildcard interface, and
   must have separately approved, narrowly scoped protocol authority.
   No listener was added here. Cheat/world permissions cannot be
   changed as a workaround.
4. A future native owner could query GetExtendedTcpTable using
   TCP_TABLE_OWNER_PID_CONNECTIONS, separately for AF_INET and
   AF_INET6, and correlate an ESTABLISHED 4-tuple and network-order
   ports with the actual connected endpoint. Reject no match,
   multiple matches, changed states, and port reuse.
5. Match the OS-reported socket PID with the retained process PID,
   then recheck its exact creation FILETIME. A socket owned by
   WebView, Electron or another helper does **not** automatically
   prove the game process: verified helper parentage and lifetime
   would require additional independent native evidence.
6. A connection PID only identifies an OS process that opened a
   socket; it does NOT prove that the messages contain local-player
   Minecraft coordinates, correct dimensions, current world context,
   authorized gameplay or unmodified data. Require separate
   local actor/world epoch and response-correlation evidence.
7. Treat exit, reconnect, window/process swap, world change,
   dimension change, stale or reordered observations, disconnect
   and ambiguous actor identity as immediate invalidations. Old XYZ
   must never be presented as current.
8. Only after independently verified native read-only proof and field
   compatibility could the app expose true live XYZ. Do NOT admit
   copy based on a user-provided verified flag, PID, nonce or log name.

**Security boundary:** A /connect websocket channel can also expose
game commands. This project does not authorize issuing commands,
installing a remote gateway, overriding protected Minecraft worlds,
automating a player login, injecting a DLL or starting a background
network service. This plan is not authorization for those operations.

### Future validation matrix (NOT_EXECUTED)

| Gate | Evidence required | Without it |
| --- | --- | --- |
| EDU-1 | Education 26.33 supports an approved opt-in client-local position/connection mechanism | UNSUPPORTED |
| EDU-2 | Source genuinely addresses current local game player, not arbitrary server-side world participant | PLAYER_UNATTRIBUTED |
| WIN-1 | Native connection 4-tuple maps to exact pinned MCxx process PID and birth (or verified helper chain) | SOCKET_OWNER_UNKNOWN |
| MULTI-1 | Five clients simultaneously bind to five distinct local-player/process contexts | CROSS_CLIENT_UNVERIFIED |
| WORLD-1 | Actual session/world actor and its replacement/rejoin boundary independently attributable | WORLD_UNVERIFIED |
| FRESH-1 | Ticks/time, dimensions, reconnect and PID/port reuse handled without displaying stale coordinates | STALE |
| F3-1 | Precision, rotation, dimension and FPS have source-specific verified meanings | FIELD_UNAVAILABLE |

The only released-like feature in this workstream remains the
user-initiated **Paste XYZ** UI, clearly labeled UNVERIFIED. Existing
source fixtures were not executed; neither Code Builder nor /connect
was operated from ChatGPT or the user's machine.

### Recommended engineering sequence after P5

First, an optional **read-only Windows socket owner witness** could be
designed alongside existing instance_identity.rs, with pure synthetic
edge cases for IPv4/IPv6, 4-tuple ambiguity, port reuse and PID birth
changes. This must not create a listener, socket owner, process manager
or Copy permission. After that, only real separately authorized
Education 26.33 compatibility evidence could justify any live
transport implementation. If EDU-1 is false, stop this approach
instead of forcing the obsolete Code Connection or adding hooks.

Sources:
- https://edusupport.minecraft.net/hc/en-us/articles/360047116992-Code-Builder-in-Minecraft-Education
- https://minecraft.makecode.com/reference/player/position
- https://learn.microsoft.com/en-us/minecraft/creator/documents/scripting/multiplayer-scripts?view=minecraft-bedrock-stable
- https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/scripteventsource?view=minecraft-bedrock-stable
- https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getextendedtcptable
- https://learn.microsoft.com/en-us/windows/win32/api/tcpmib/ns-tcpmib-mib_tcprow_owner_pid
- https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log
- https://minecraft.wiki/w/Commands/wsserver

## P6 — Bounded synthetic TCP PID witness (3 November 2026)

The existing native process owner, Development/apps/desktop/src-tauri/src/engine/instance_identity.rs,
now includes a PRIVATE test-only Rust module, guarded by #[cfg(test)].
It is not mounted into the normal Lazy Tester runtime, and no Windows TCP
table query, network listener, WebSocket endpoint, new game controller or IPC
operation is introduced.

The fixture models host-order IPv4 and IPv6 loopback four-tuples, exact
local/remote endpoints, ESTABLISHED state, duplicate and missing TCP rows,
non-game helper PIDs, process creation FILETIME changes, and port reuse.
Every unique matching OS PID classification is named
PidCandidateOnlyUnverified; two stable synthetic process/connection snapshots
only produce PidOnlyStillUnverified. Neither status means the source player,
world, Minecraft client or Copy Coordinate is verified.

Microsoft IP Helper uses MIB_TCPTABLE_OWNER_PID for IPv4 and
MIB_TCP6TABLE_OWNER_PID for IPv6. In a future OS adapter, convert the
network-order dwLocalPort and dwRemotePort fields to host-order u16,
normalize IPv6 scope IDs separately and require an exact established
connection four-tuple. The owning PID must be checked against the retained
original native process HANDLE and creation FILETIME before and after any
observation. OS rows do not themselves provide process birth or game
player/world attribution. A Code Builder/WebView helper PID must never be
silently assigned to the owning Minecraft client.

The new fixtures do NOT call GetExtendedTcpTable; no wire protocol, real
connection, Education Code Builder runtime, source player identity or
automatic F3 output was added. Synthetic results cannot grant any
clipboard or game-control authority. Authored Rust tests are
AUTHORED_NOT_EXECUTED; Windows and Minecraft 26.33 remain unverified.

Next source question: without an authorized Education 26.33 client-originated
connection and an independently established local player/world source,
what real endpoint would a native TCP table lookup legitimately target?
Do not add speculative listeners or self-issued provenance. Preserve
the manual Paste XYZ fallback.

Official references:
- https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getextendedtcptable
- https://learn.microsoft.com/en-us/windows/win32/api/tcpmib/ns-tcpmib-mib_tcptable_owner_pid
- https://learn.microsoft.com/en-us/windows/win32/api/tcpmib/ns-tcpmib-mib_tcp6table_owner_pid

## P7 — 26.33 client-local coordinate transport admission (3 November 2026)

### Evidence outcome: no supported automatic F3 source has been verified

This is an explicit STOP on speculative connection code, not a finding
that Minecraft Education can never expose client-local position. The
official Education changelog confirms version 26.33 released 28 October
2026, but documents only fixes for this patch; it does not describe
a supported desktop-read-only position streaming API.

**Current supported product behavior** remains P4's user-pasted,
USER_PASTED_UNVERIFIED XYZ -> block/teleport text. Its data cannot be
attributed automatically to MC01-MC05, world or player. The native FPS
overlay measures DXGI Present/Present1 acceptance, not player position.

### Candidate source audit and corrected assumptions

| Source | Confirmed | Not established / consequence |
| --- | --- | --- |
| Official Education Code Builder | Accessible through the assigned C key, MakeCode player.position() returns the current Code Builder player's world position | Does not expose a documented, authenticated read-only desktop export, yaw/pitch or full F3; the official Code Builder help says /code no longer works |
| Microsoft /wsserver, alias /connect | Bedrock creator command docs specify connecting to a WebSocket URI, admin permission and cheats | These are Bedrock command requirements, not a verified Education 26.33 permission profile; connection can carry commands, so not a read-only telemetry contract |
| Community Education/Bedrock WebSocket bridges | A maintained third-party source repository describes /connect and a Script API behavior pack | This is a claim about its own setup, not official Education 26.33 acceptance. It requires installing/enabling an extra world behavior pack and a separate server, conflicting with the existing team's exclusive telemetry ownership |
| Packaged-app localhost connectivity | Official Windows UWP guidance explains loopback restrictions and CheckNetIsolation exemptions for packaged apps | A third-party guide reports needing such an exemption on Education Windows; do not modify AppContainer exemptions, elevated system policy or network settings to make this feature work |
| Standalone Code Connection | Microsoft MakeCode announced end-of-support and removal from the Store on 1 December 2023 for the old Bedrock Windows companion | Education's historical changelog still lists Code Connection 1.X as compatible with 1.19.xx and higher; that older compatibility statement is NOT 26.33 runtime verification or current maintenance support. Do not conflate these facts |
| Windows GetExtendedTcpTable and P6 fixture | Can map an observed IPv4/IPv6 TCP four-tuple to the OS-reported owner PID | Without an actual approved endpoint this is a hypothetical query. Even a matching PID/creation FILETIME cannot authenticate Minecraft local player, world or returned coordinate payload |

### Decision and security invariants

1. Keep automatic F3/Copy and any GetExtendedTcpTable runtime adapter
   **DISCONNECTED** until an authorized Education 26.33 client-origin
   endpoint, data format, and real actor/world provenance are observed.
   Absence of protocol proof is a blocker, not something a type or
   synthetic fixture can overcome.
2. Never introduce another Script API collection module, MCWSS bridge,
   background listener, Tauri/MCP input service, WebSocket server or
   fallback game control; the original team modules own analytics.
3. Never elevate permissions, change cheats/world policies, install
   third-party behavior packs, disable WebSocket encryption, or add
   Windows loopback exemptions as an implicit side effect.
4. Do not reuse unrelated Education network traffic (multiplayer,
   authentication, MakeCode HTTPS/WSS or relay connections) as a
   provenance witness. Those sockets are not a position telemetry
   protocol and may be owned by an unrelated helper process.
5. No field from an unauthenticated stream or user paste can set
   copyAllowed=true in coordinateEvidence.ts. Retain P6 PID/tuple
   experiments as private cfg(test) fixtures with no OS calls.

### Practical path not requiring an automatic connection

For player-initiated use only, Minecraft Education's Code Builder
can display player.position() within its own current-player context;
the existing P4 panel then accepts explicitly entered XYZ text.
This is *manual, unverified* and is not automatic Copy Coordinate.
Code Builder's availability does not imply the old /code command
or an external WebSocket connection is usable.

### Next implementation work that is independent of the blocker

Source-inspect the EXISTING native DXGI FPS HUD owner
(Development/tooling/acceptance/windows/bootstrap-ordering/fps-hud.cpp)
for its render-rate semantics, lifetime, client association and
actual reuse constraints. Only consider an F3 read-only performance
presentation if the current native source safely exposes a per-client
sample without a second collector, added hook, new transport, or
claim of displayed FPS. Do not repurpose the blocked position bridge.

P7 changed only this design and the single handoff owner. No TCP query,
client connection, game command, native source, behavior pack, clipboard
permission, Windows test, compiler/test suite or CI was run.

Public references:
- Minecraft Education releases and Code Connection historical table:
  https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log
- Minecraft Education Code Builder help:
  https://edusupport.minecraft.net/hc/en-us/articles/360047116992-Code-Builder-in-Minecraft-Education
- Microsoft MakeCode current player position:
  https://minecraft.makecode.com/reference/player/position
- Microsoft Bedrock /wsserver documentation:
  https://learn.microsoft.com/en-us/minecraft/creator/commands/commands/wsserver?view=minecraft-bedrock-stable
- Microsoft PXT 2023 old Code Connection end-of-support:
  https://github.com/microsoft/pxt/blob/master/docs/blog/minecraft/2023-release.md
- Microsoft UWP loopback exception guidance:
  https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/troubleshooting-uwp-firewall
- Community implementation (untrusted as acceptance; do NOT install):
  https://github.com/Mming-Lab/minecraft-bedrock-education-mcp
- Windows TCP owner API:
  https://learn.microsoft.com/en-us/windows/win32/api/iphlpapi/nf-iphlpapi-getextendedtcptable

## P8 — Existing FPS HUD source and lifecycle audit (3 November 2026)

### Confirmed source ownership and measurement semantics

The active C++ owners already exist under
Development/tooling/acceptance/windows/bootstrap-ordering:

- dxgi-pacing.cpp owns exactly one process-global Policy, currently
  at most one wrapped live IDXGISwapChain4, one paced Present route,
  and the existing fps_hud::Hud instance. No extra collector is needed.
- The wrapper checks the bound game's own HWND/thread/PID before
  recording outer Present/Present1 native calls, then records only
  S_OK returns excluding DXGI_PRESENT_TEST. It does not claim
  scanout/displayed frames, GPU simulation FPS or the configured
  foreground/background FPS ceiling. Microsoft explicitly states
  that the number of Present calls differs from displayed frames.
- fps-hud.cpp owns Meter's QPC-based interval window (500 ms),
  two-second inactivity invalidation, a 128×32 DIP owned layered
  overlay, and its one worker whose window/DPI/geometry changes
  are bounded to the original game process. Unbind resets the
  meter; a short grace period permits swap-chain recreation.
- dxgi_pacing::hud_snapshot() returns only an **in-process**
  fps_hud::Snapshot. compat.cpp exposes BootstrapHudSnapshot for
  existing C++/native diagnostic consumers. This is a private DLL
  export, **not** a Rust SessionSnapshot field or a Tauri IPC
  route, and does not automatically associate an observed FPS value
  with MC01–MC05's verified Windows process identity.

Do not interpret Microsoft GetFrameStatistics as an automatic
substitute: that API is swap-chain/model dependent, has documented
multi-monitor limitations, and does not by itself supply local
player/world position or an authorized cross-process transport.

### One bounded source correction

HUD::snapshot() now marks its reading invalid when the retained
workerActive flag is false, even if the meter's accepted Present
samples have not yet expired by QPC age. It retains the accepted
count and does not change the native HUD overlay painting, pacing,
Present dispatch, swap-chain ownership or worker lifetime.

The existing fps-hud-control.cpp **authored, NOT_EXECUTED**
hidden-window fixture now covers an owner HWND destroyed while the
Policy/Hud object remains alive. After retirement it deliberately
submits recent synthetic Present samples through the public
diagnostic entry. A snapshot must preserve the accepted counter
but refuse a live-rate claim, and the existing replacement-HWND
rebinding check remains intact.

This is a diagnostic truthfulness fix, NOT proof of game runtime,
FPS cap, live F3 player/world source, desktop status integration,
multi-client performance or gameplay acceptance. No new native
IPC, user-facing F3 panel, bitmap allocation, extra frame sampler,
DXGI hook, overlay worker, global shortcut or clipboard API was
added. MSVC /W4 /WX, Windows HUD self-tests, Minecraft Education
and five-client acceptance remain NOT_EXECUTED at this HEAD.

### P9 candidate boundary (not implemented)

An F3 desktop performance view would require the **original**
native controller to receive a bounded process-bound reading
without using a second sampler or a new unauthenticated channel.
The only current API is an in-process compatibility DLL
diagnostic export, so no direct read from Svelte or a random
Windows PID is authorized. Before designating a field as
"FPS", specify "accepted DXGI Present rate" and require
freshness plus exact original process identity. If no safe
existing transport is available, do not expand the HUD; leave
its own-process overlay as-is.

Official Microsoft DXGI source:
- https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiswapchain-present
- https://learn.microsoft.com/en-us/windows/win32/api/dxgi/ns-dxgi-dxgi_frame_statistics
- https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiswapchain-getframestatistics

## P9 — BootstrapHudSnapshot ownership and binding-safe FPS readings (3 November 2026)

The actual active source graph is closed:

- compat.def exports BootstrapHudSnapshot, and compat.cpp implements a
  strictly sizeof(fps_hud::Snapshot)-checked C ABI entry in the *child's*
  bootstrap-compat.dll. It calls dxgi_pacing::hud_snapshot() in that
  process. It is NOT an external desktop FPS-reading service.
- The concrete GetProcAddress consumer is bootstrap-ordering/probe.cpp
  in its own test process. frame-pacing-control.cpp also reads
  dxgi_pacing::hud_snapshot() directly for its C++ control fixture.
  The inspected Rust session_controller.rs, native_clients_session.rs,
  agent_cli.rs, command allowlist and Svelte bridge have NO current
  BootstrapHudSnapshot read route.
- The original Rust controller holds the separate native Minecraft
  process identities and windows, but cannot call another process's
  DLL export by GetProcAddress in the manager and recover its per-process
  HUD state. No shared memory, private pipe, user-approved snapshot
  transport or native source attestation exists for this metric.
- The accepted-rate sample covers outer native successful DXGI
  Present/Present1 only. It does not certify display scanout, simulation,
  configured frame ceiling, game readiness or local player/world origin.

### P9 bounded correction (source-only)

Hud::snapshot() now reads state, QPC meter and worker flag under the
EXISTING State::gate shared lock, following the State::bind/unbind
exclusive gate -> meter reset lock order. This prevents an unbind
from changing the owner generation between retrieving display state
and rate. It invalidates reading.valid when owner or ownerThread is
absent, even if the single HUD worker is still alive in its normal
one-second unbind grace, or when workerActive is false. It preserves
the historical reading.accepted counter for diagnostic purposes.

The existing fps-hud-control.cpp hidden-window control now has an
authored source scenario that calls unbind(), records eight accepted
synthetic presentations while the owner window intentionally stays
alive, and asserts the snapshot reports no valid current rate.
This supplements P8's destroyed-HWND/retired-worker fixture.

NO new sampler thread, hook, BPP/GDI surface, overlay, FPS label,
Tauri command, native game transport, event subscription, clipboard
API or file/network stream has been added. This is a conservative
C++ source correction and it is **AUTHORED_NOT_EXECUTED**; MSVC
build, Windows native HUD self-test and Minecraft validation remain
NOT_EXECUTED at this exact HEAD.

### P10 direction — no unjustified cross-process export

Treat desktop F3 performance and live XYZ as distinct unresolved
capabilities. The native FPS overlay already works in its own
process/source design, but obtaining a process-identified rate inside
the manager would require a newly reviewed transport and genuine
Windows evidence. Under the current no-new-IPC/no-local-test boundary,
do not build that transport merely to show a number. The Play
Paste XYZ fallback stays manual and UNVERIFIED. The next useful source
pass is a bounded review of the P4 panel's existing accessibility,
focus/closing, and safe output disclosure, not new telemetry.

## P10 — Manual Paste XYZ keyboard and assistive-technology corrections (3 November 2026)

Audit found that the P4 disclosure relied on native details toggle only:
it lacked Escape-to-close and did not associate invalid-coordinate
feedback with the editable input. Both are corrected in the same owner.

The existing CoordinatePastePanel.svelte now handles Escape only while
the disclosure is open AND the key event came from inside its own
DOM subtree. It closes the native details element and restores focus
to the summary without scrolling. Escape in Minecraft/launcher or
the separate selected-game close review is not intercepted.

The XYZ textarea exposes aria-invalid and connects its guidance
and any active visible validation error through aria-describedby.
The readonly formatted text connects to its own Ctrl+C instruction.
Select text still only highlights the prepared string for manual Ctrl+C.

This is a nonmodal native details disclosure, not a focus-trapping
dialog. On close, existing resetOnClose still clears user input,
chosen output mode and text; Play unmount discards the component.

No native clipboard read/write, game input, cross-process FPS route,
timer, local storage, CSS layout change or fifth-card regression was
introduced. All new Vitest checks are AUTHORED_NOT_EXECUTED;
TypeScript/Svelte compile, actual keyboard/screen-reader use, Windows
and Minecraft gameplay remain NOT_EXECUTED. This does not authorize
live XYZ/F3 or copyAllowed for an unverified player or world.
