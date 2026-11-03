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

## P11 — Fresh input/mode selection for manual XYZ output (3 November 2026)

The existing component's reactive block already uses
`formatUserPastedCoordinates(raw, mode)` to render the text for exact
XYZ, floored Block XYZ or position-only `/tp @s X Y Z`. P11 addresses
the narrower UI race: the old Select text button trusted the last
rendered `output` without checking that it still represented the
latest input and selected format when the selection occurred.

Select text now captures the current raw input and output format,
reformats synchronously through the same strict parser and refuses
empty, malformed or command-bearing input. It then waits for one
Svelte `tick()` and selects only if the disclosure is still open,
input and mode are unchanged, the validation is valid, the
output field is connected, and its DOM value exactly equals the
fresh formatted text. A mode change, edit, close or unmount during
the pending update causes a quiet no-op rather than selecting stale
content. No polling, background work or global shortcut is added.

To prevent retaining an earlier selected string, typing new XYZ or
changing the dropdown clears the readonly output's selection range.
The output still requires explicit Select text then Ctrl+C; none of
these actions read or write clipboard contents or execute a command.

Authored source fixtures now cover all three modes across several
inputs, including negative floor and tiny decimal precision, plus
incomplete/invalid input. Separate UI source checks require
fresh input/mode comparison, the tick boundary and DOM equality.
These tests are AUTHORED_NOT_EXECUTED; actual Svelte rendering,
keyboard selection, typecheck, Windows and game runtime remain
NOT_EXECUTED. Manual text remains USER_PASTED_UNVERIFIED and
does not provide an authenticated MC01–MC05 player/world coordinate.

References:
- Svelte tick: https://svelte.dev/docs/svelte/svelte
- MDN input.select: https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement/select
- MDN selection range: https://developer.mozilla.org/en-US/docs/Web/API/HTMLInputElement/setSelectionRange

## P12 — Lossless manual XYZ and mathematically correct block flooring (3 November 2026)

Concrete source defect: the P3 manual parser converts each decimal
token to a JavaScript Number. For input such as
`0.99999999999999999`, Number can become 1, and subsequently
Math.floor produces block 1 even though the original decimal is
strictly below 1. Likewise exact user digits could disappear from
XYZ and position-only teleport text.

The existing pure `coordinates.ts` owner now parses each manual
decimal as a bounded, canonicalized **digit string**. Exact XYZ and
position-only `/tp @s X Y Z` use that value without intermediate
floating-point rounding. Block XYZ is computed from the exact
integer and nonzero fractional parts, including negative fractional
values just below an integer. Leading plus signs, leading zeroes,
insignificant fractional trailing zeroes and negative zero are
normalized without changing the value. Large magnitudes are rejected
using exact integer digits, so `1000000000.00000000001` cannot
slip through via Number rounding.

`parseUserPastedCoordinates` retains its existing public numeric
`position` shape for compatibility, but this Number projection can
round its decimals; it is explicitly **not** the source for precise
formatted output or block math. No provenance/identity/verified field
was added. The unbound Script API-shaped numeric coordinate contract,
live proof gate and existing Svelte panel are unchanged.

Authored Vitest scenarios cover exact significant digits, near-integer
rounding, tiny negatives, canonical decimals, +/-1e9 bounds,
position-only teleport text and no floating-point block drift.
They are AUTHORED_NOT_EXECUTED. No typecheck, desktop render,
Windows/Minecraft runtime, native IPC, clipboard API, game command
or CI was run.

## P13 — F3 Inspector read-only selected-client UI (3 November 2026)

**Product choice:** Minecraft Education does not offer Java's native F3
debug screen. Official Education support instead documents Show Coordinates
in world Settings. Lazy Tester therefore introduces a separate custom F3
Inspector for *native process diagnostics*, not a fabricated Minecraft
player/world data feed.

New source owners:
- Desktop src/app/presentation/f3Inspector.ts is a PURE, stateless
  projection of already parsed native ProfileSession and SettingsReport;
  its statuses are NO_SELECTION, NOT_CURRENT, NATIVE_PROCESS_OBSERVED.
- Desktop src/F3InspectorPanel.svelte is a compact, collapsed, read-only
  disclosure in Play's existing header alongside Paste XYZ. It uses
  native details/summary and Escape scoped to focus inside the panel.
- Existing LazyTesterApp.svelte is the ONLY source of truth for active
  selection. It admits a native observation only while WindowsNative,
  binding configured, session available, in Play, not detached, not
  closing, not marked for uncertain window recovery, and the original
  cached selected PID/FILETIME/Windows session/HWND matches the current
  ProfileSession. Otherwise F3 suppresses PID and settings fields.

Visible data for a current selected game: MC01–MC05 slot, native
process state, PID, Windows session, window responding observation
and the manager's last configured active/background FPS limits
(explicitly not measured). The panel ALWAYS reports measured
FPS, player XYZ, rotation and dimension as not available/not
connected because no trusted game-local transport exists. Native
process binding is not a player/world or actual gameplay assertion.

Selecting another game updates the projection; missing/changed
selection, connection loss, or uncertain recovery suppresses
the process facts instead of showing a stale former client.
The summary is native observation only; nothing polls or sends
new IPC requests. No extra lifecycle owner or telemetry collector
is created. The original P4 Paste XYZ remains a separate user
entered, unverified text tool; the native DXGI HUD remains
in-process and is NOT forwarded to F3.

Source regression fixtures are AUTHORED_NOT_EXECUTED. Actual Svelte
compilation, Windows viewport/popover rendering, Minecraft Education
client readiness and five-profile correctness are NOT_EXECUTED.

Official education sources:
- https://edusupport.minecraft.net/hc/en-us/community/posts/27889945431572-How-to-display-the-debug-screen-in-minecraft-education-edition
- https://edusupport.minecraft.net/hc/en-us/articles/360047117692-FAQ-Game-Features

## P14 — Pending action exclusion and accurate F3 FPS target provenance (3 November 2026)

A native F3 inspector can otherwise display a formerly selected
PID/window while a close review, replacement selection, queued
launch or navigation is already pending. P14 derives
f3ActionPending from existing busy/closeReviewPending/queued
selection/queued page/queued launch/requested launch/manual refresh
fields. This is a read-only Svelte derivation, not another lifecycle
manager or mutable process ownership state.

During such an action the pure F3 projection emits ACTION_PENDING
before accepting any old selected PID, Windows session, window
or settings values. It withholds all native details until the
original operation settles and an existing selected identity
matches its last validated native snapshot again.

The F3 view now explicitly identifies active/background FPS numbers
as **last reported GLOBAL targets** (which may not yet have been
applied to the currently running game). They are not measured
frame rates or actual per-client limits. A normal periodic native
Inspect continues to display the last observation rather than
flickering while waiting for an observation request.

Existing measured FPS, XYZ, rotation and dimension remain
Not exposed to desktop / Not connected. No new native IPC, HUD,
Script API module, network listener, logging or clipboard access.
Regression fixtures cover pending action precedence over an
otherwise matching selected process and absence of measured FPS.
Source tests were AUTHORED_NOT_EXECUTED. Windows, Svelte/TS,
gameplay and CI remain NOT_EXECUTED.

## P15 — Existing MCP visual capture as an F3 screen-review candidate (3 November 2026)

Source audit of the exact team archive SHA-256
26c0a322cabed172ca74970413d5bb7a20d6f67181446df5f688e36057c54e21
confirmed that team analytics position_samples are opt-in, historical
block-floored server-world records without yaw/pitch or a native
MC01-MC05 process binding. The original analytics export is a paced
Content Log, NOT a live local-player signal; the original team owns it.

The existing optional observe_client and read_visual_observation MCP
routes already obtain one selected window using the original native
controller's PID, process creation FILETIME, session and HWND checks.
CUA and WGC observations carry PNG metadata and source-age screening
but do not prove that the pixels depict the local Minecraft player
or that Show Coordinates is actually visible.

One bounded implementation now adds f3VisualReview to these EXISTING
visual-read metadata responses. The new pure
projectF3VisualReview() in visual-observer.mjs rejects missing, stale,
invalid or contradictory frame metadata. When a usable captured
window frame exists, it only reports
WINDOW_FRAME_FOR_EXPLICIT_VISUAL_REVIEW_ONLY; otherwise
NO_CURRENT_WINDOW_FRAME. Both outcomes ALWAYS state:
coordinateExtraction=NOT_PERFORMED,
playerWorldAttribution=NOT_VERIFIED, livePositionAvailable=false,
and automaticCopyAllowed=false.

This can support a future AI/operator manually examining Show
Coordinates text in an actual captured MCxx window. Even that
observation would be an unverified *visual candidate*, not a
server-authenticated player/world position or automatic Copy.
F3 Inspector desktop continues to show Player XYZ: Not connected.

The change reused the original one-shot and bounded cached frame
read tools. It added NO new MCP tools, capture workers, game commands,
Script API modules, telemetry emitters, native IPC paths, clipboard
writes, UI polling, or profile/world mutations. The human MC01
capture restriction in hybrid mode remains unchanged.

Node source fixtures covering CUA/WGC metadata, changed PID/HWND,
unknown providers, stale cached captures and no Copy authorization
are AUTHORED_NOT_EXECUTED. Windows capture, actual screen contents,
official Education 26.33 client source and Minecraft gameplay
remain NOT_VERIFIED/NOT_EXECUTED.

## P16 — Native-window ↔ player pairing feasibility, original team HUD only (3 November 2026)

### Evidence reviewed

Original team archive `minecraft-bedrock-agent-skills-main (3)(2).zip`
was inspected read-only; SHA-256:
`26c0a322cabed172ca74970413d5bb7a20d6f67181446df5f688e36057c54e21`.
These findings describe THAT pinned archive, not an independently
verified up-to-date team repository:

- `modules/display/display-service.ts` owns the single Script API
  title/subtitle/actionbar writer and uses `player.onScreenDisplay`
  for an **explicit Player**. It provides no inbound Lazy Tester
  transport, native process identity, response receipt or coordinate
  export. Its `showActionBar` is a one-way render call.
- `modules/display/hud-line.ts` owns per-player actionbar slots
  (`setHudPart`, `clearHudPart`, `flashHud`, `renderHud`) and refreshes
  unchanged text every 10 ticks. The map must wire `renderHud()`
  to its own tick loop. There is no existing `f3:pair`, nonce exchange
  or native PID/world binding API. External `/title` or `/titleraw`
  actionbar writes would compete with the existing per-player owner.
- `modules/coords/coords.ts` is a PURE transform of spawn/block
  references, not a reader of the currently selected client.
- `modules/analytics/analytics.ts` retains opt-in, stored historical
  floored position samples; analytics-export prints the stored
  records to a server Content Log. A stored player_id and world
  data are **not** evidence of this Windows client's local actor.
- The team's `bedrock-onscreen-display` skill explicitly prefers
  Script API over commands for Education compatibility and says
  classroom restrictions can affect commands and scriptevents.
  Existing source tests on macOS 26.3x are NOT acceptance on
  Education Windows 26.33.

### Feasibility decision: distinguish two different outcomes

| Path | Required input | Achievable claim today | Missing proof |
| --- | --- | --- | --- |
| A. Built-in Show Coordinates + existing exact-window PNG | Operator explicitly enables the world setting where permitted and requests the already existing `observe_client` or cached visual observation for exactly one MCxx | **Screen-visible coordinate candidate**, if a human visually confirms XYZ actually appears in the captured window | Screenshot readability, exact field semantics, player/world and live frame age still require runtime checks; no rotation/dimension or automatic Copy |
| B. One-time actionbar challenge through original team HUD | Original team source must add its OWN reviewed per-player handshake entrypoint and the native owner must provide an authenticated challenge/response transport | **Pairing candidate**, not a verified actor or world until independent provenance/epoch gates pass | Neither a challenge endpoint nor a trusted game-to-native return channel exists in the pinned archive or active manager |

Do not implement B using a second Script API module, game command,
legacy Code Connection, a global world.getPlayers()[0] choice,
arbitrary username, player_id in stored analytics, or an ephemeral
nonce that has only been generated and compared locally. A code
visible on the right HWND proves at most that those pixels carried
that code; it does not independently authenticate who rendered
it, what world was active, or that the position payload belonged
to that actor.

### Source-only future pairing contract — NOT IMPLEMENTED

1. The **original native session controller** must pin the chosen
   MCxx PID, creation FILETIME, Windows session, root HWND and
   the current owner/selection epoch. The MCP screenshot source
   must revalidate this identity before and after every capture.
2. Any future pairing flow requires an **explicit operator opt-in**
   in a disposable, authorized Education QA world. Source startup,
   Refresh and routine F3 panel expansion must never trigger a
   command or mutate any live world.
3. The **original team module author** (not Lazy Tester) would have
   to design a single per-player, short-lived challenge slot inside
   `setHudPart` or the canonical display owner. It must select a
   real event-associated Player from a verifiable player-origin
   event, not pick an arbitrary server-side player.
4. A challenge is accepted only after proving an independent,
   authenticated path connecting the server-issued actor/world
   session epoch to the original controller-issued window binding.
   A log line, echoed nonce, player-entered text or `@s` by itself
   cannot satisfy this step. If event source is Server/Block or
   has no uniquely attributable Player, refuse.
5. The visual witness must be a **new** capture from that same
   pinned HWND within a bounded challenge lifetime. Reject changed
   PID/HWND/birth, repeated cached PNG, expired window frame,
   multiple matching actor candidates, cross-slot replay and
   reconnect/respawn/world/dimension uncertainty. No screenshot
   OCR or automation is implemented in this milestone.
6. Expire any candidate on user cancellation, window replacement,
   disconnection, player leave, world/session restart or failure.
   Original game HUD owner must clear any temporary challenge
   and preserve existing map HUD priorities; never overlay a
   global second actionbar writer.
7. Only separate live **Education 26.33 Windows** proof for
   both the game actor/world and native process relationship
   could justify proposing new runtime transport or live XYZ.
   Until then, `coordinateExtraction=NOT_PERFORMED`,
   `playerWorldAttribution=NOT_VERIFIED`,
   `livePositionAvailable=false`, and
   `automaticCopyAllowed=false` must remain unchanged.

### Acceptance matrix (all NOT_EXECUTED)

| Gate | Acceptance evidence required | Fail-closed outcome |
| --- | --- | --- |
| EDU-HUD | Original team's display module is installed, wired and displays a short-lived per-player HUD part without clobbering the map | NO_TEAM_DISPLAY_OWNER |
| EDU-ORIGIN | A verified player-origin event gives exactly one real Player and world/session epoch, never an unauthenticated username or Server source | PLAYER_ORIGIN_UNKNOWN |
| WIN-BIND | Native PID/birth/session/HWND remained pinned through both sides of capture; no helper or replacement adopted | WINDOW_REPLACED |
| CHALLENGE | An independent authenticated native ↔ server challenge path exists; nonce-only equality is explicitly insufficient | INDEPENDENT_PATH_MISSING |
| FRAME | Newly produced readable pixels from that exact HWND, correct challenge and no cached/stale or cross-slot replay | FRAME_NOT_TRUSTED |
| MULTI | Five independently managed clients cannot adopt one another's actor/world, even in a shared world | CROSS_CLIENT_NOT_ATTESTED |
| WORLD | Rejoin, dimension/world replacement, stage changes and player leave revoke pending identity | WORLD_EPOCH_UNKNOWN |
| F3 | Real source XYZ precision, rotation, dimensions and freshness are verified with separately reviewed field mapping | LIVE_F3_NOT_VERIFIED |

### Product/permission boundary

- **No code path in this change** sends a command, adds a logger,
  patches a behavior pack, installs a team module, enables cheats,
  changes classroom/world options, reads clipboard or obtains an
  automatic game/world connection. All current source-only F3
  statuses remain unchanged.
- Operator-initiated **Show Coordinates** within the game's existing
  world settings is the lower-risk exploratory route. It can
  support an explicitly **screen-observed, unverified** XYZ review
  once actual Windows pixels exist, NOT authoritative live data.
  Some worlds/settings have permissions; do not alter them here.
- No original-team developer work, ZIP sync, authenticated challenge
  transport or Windows/Minecraft tests were performed in P16.

References:
- https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/screendisplay?view=minecraft-bedrock-stable
- https://learn.microsoft.com/en-us/minecraft/creator/documents/scripting/multiplayer-scripts?view=minecraft-bedrock-stable
- https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/scripteventsource?view=minecraft-bedrock-stable
- https://edusupport.minecraft.net/hc/en-us/articles/360047117692-FAQ-Game-Features
- https://edusupport.minecraft.net/hc/en-us/articles/360061369132-Classroom-Settings

## P17 — Simplified F3 MVP: AI-read screen text, no new OCR engine (3 November 2026)

User-selected MVP: choose MCxx, capture its already-owned window,
visually read built-in **Show Coordinates**, and obtain XYZ for
explicit review/copy. The existing desktop F3 Inspector does NOT
yet receive screenshot frames from MCP; this milestone does not
claim one-click in-app Copy.

### Actual source and dependency decision

- The approved MCP already has `read_visual_observation`, which
  supplies a recent pinned MCxx window PNG on explicit request.
- An AI viewing the PNG can transcribe the single top-left
  `Position: X, Y, Z` line. This is a bounded visual interpretation,
  not an externally authenticated local-player record.
- Windows.Media.Ocr is not assumed available: Microsoft documents
  package-identity requirements (MSIX) while this Tauri installer
  uses NSIS. No new Tesseract runtime/installer, model API,
  image-recognition package or DLL was added.

### P17 implemented source-only slice

Extend the EXISTING `read_visual_observation` tool with an optional
`screenReview` argument, accepted ONLY with `includeImage:false`,
containing `frameSequence`, `sourceFrameAtMs`, exact
`{pid,windowId,sessionId,createdFileTime}` and AI-transcribed
`recognizedLine`. Reuse the *same existing in-memory window-frame
cache* and the previous P15 `f3VisualReview` status. No additional
screenshot or worker starts when reporting the result.

New pure `reviewVisibleXYZFromFrame()` refuses missing/aged images,
replaced PID/birth/HWND, mismatching source timestamp or image
sequence, extra forged fields, non-labelled/free-form coordinate
text, commands, decimal/exponent values, and values outside
+/-1,000,000,000. It normalizes safe integer XYZ from a single
exact `Position: X, Y, Z` line. An accepted result is always
`SCREEN_XYZ_CANDIDATE_UNVERIFIED` with
`source=AI_TRANSCRIBED_SCREEN_TEXT_UNVERIFIED`,
`playerWorldAttribution=NOT_VERIFIED` and
`automaticCopyAllowed=false`.

**It does not run OCR automatically.** A model/operator must
explicitly inspect the PNG and submit what was visible. A model
can misread or fabricate text; this step cannot independently
verify that the pixels show those digits. The candidate may be
reviewed/copied manually outside the app but cannot be labeled
live local-player coordinates or injected into the F3 desktop
panel. There is still no backend-to-Svelte image/XYZ channel.

### Simplest next engineering step

Review whether one *existing* authenticated native desktop owner
can safely provide the same window PNG and one local image-text
recognition operation to the selected F3 panel, without exposing
raw process handles or changing the installer/CI. Only after
an actual Education 26.33 screen sample and Windows acceptance
should a user-initiated Copy button be enabled, and it must
clear immediately on game switch, source expiry or missing text.
No continuous polling or keyboard-injection fallback by default.

Source regression cases for integer XYZ, stale/replaced/forged
screens, unlabelled/ambiguous model text and denied clipboard
are AUTHORED_NOT_EXECUTED. No game or account/world command,
WebSocket, native IPC, installed module, desktop code, CI,
Windows or Minecraft execution occurred.

References:
- https://edusupport.minecraft.net/hc/en-us/community/posts/27889945431572-How-to-display-the-debug-screen-in-minecraft-education-edition
- https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr?view=winrt-26100
- https://github.com/tesseract-ocr/tessdoc/blob/main/Installation.md
