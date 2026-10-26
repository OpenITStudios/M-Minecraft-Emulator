---
id: document.product.unified-workspace
class: DOCUMENT
domain: product
role: CONTRACT
authority: WORKING
lifecycle: ACTIVE
---

# Lazy Tester workspace contract — 26 October 2026

The approved product is one central application controlling five stable Minecraft
profiles. Users explicitly open each desired profile; app startup launches no game.
One large viewport contains the selected playable game. Four other sessions belong
in a lower preview rail; next/previous navigation skips sessions without a usable
window. The profile number is fixed by its storage identity, never enumeration
order, a title, or a claim that an account is signed in.

Session ownership belongs to the existing native bootstrap; the UI is a consumer.
The bootstrap keeps the reviewed game/DLL hashes, original process handles, private
Roaming/Local/TEMP/HKCU stores, scoped singleton compatibility, and 144/10 FPS policy.
Presets are written only to the selected inactive profile before its launch, with
existing backup/atomic-write behavior and simulation settings preserved.

An inactive profile retains its validated storage handles until startup transfers
ownership to the started transaction. The child receives noninheritable duplicates
of those same directory/local-registry objects, not paths reopened as fresh data.
This follows Windows [DuplicateHandle semantics](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle).
A failed startup permits a later explicit retry only if no child was created or
the retained never-resumed child was confirmed exited during cancellation. The
per-attempt in-memory cleanup receipt is not a readiness flag or another lifecycle
owner. Unconfirmed cancellation discards retry admission and reports uncertainty;
no controller restart, game kill or automatic relaunch is attempted. After any
resume attempt, the existing started owner retains the game and its uncertain
outcome. Disposable Windows fixtures cover the cleanup boundary; actual Minecraft
failure/retry with live peers remains unexecuted.

The automatic App/Data setup and preserved old-profile adoption are defined by
[data layout](../system/data-layout.md). Fresh profiles use the product catalog;
existing pilot stores remain in place. The legacy recovery fallback may ask the
user to select the old launcher folder;
the backend reads its fixed independent-bootstrap/replay-profile-five.bin only.
Import is unavailable while a retained controller
exists, checks the reviewed capability hash, original folder IDs and existing HKCU keys, and
publishes only 288 bytes of identity metadata beneath the existing private
MinecraftInstanceManager/NativePreview directory. No login/world contents are
copied and no game is launched. Selecting the same bytes is idempotent; different
or malformed preexisting metadata is preserved/refused instead of overwritten.
For this legacy recovery route the installed product reads the private binding, with no fallback to capability
files beside its executable. The private developer preview keeps its original
adjacent-capability route. Existing profile inspection now opens TEMP only if it
exists; it never creates missing profile directories during validation.

The typed binding report says only whether the original profile identities can
be opened; it is not account/sign-in/gameplay readiness. Launch controls wait for
that report, and the bootstrap revalidates identities on every launch. A cancelled
file picker leaves metadata unchanged. Publication uses a create-new temporary,
flush and same-directory [MoveFileExW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw)
without replacement. An incomplete/different existing binding requires deliberate
repair; no automatic reset is provided. Fresh product stores use the separate
catalog and location owner described above. Actual new-machine login, first-run
dialog/import and installed gameplay still need acceptance.

Closing the manager preserves games and restores presented windows. Helpers retain
policy/storage handles after UI exit until the games exit normally. Reopening
the same application binary reconnects through its local named pipe,
scoped by executable file identity and Windows logon/session. Both peers verify
user, logon, integrity, executable and a retained live process identity before any
command. No persistent readiness file or account store is introduced. A missing
endpoint is not authorization to adopt unknown games. Busy/denied/changed peers
never trigger a replacement controller. A timed-out request retains its original
worker; an inspection must resolve uncertainty before further mutations.
This is an application channel for the same Windows user, without elevation; it
is not a security boundary against malicious code already controlling that user.
Cross-version recovery and actual GUI/gameplay reconnection remain unproven.

The user asks for a browser-like experience: one navigation surface and controlled
taskbar presentation. Minecraft remains multiple OS processes. Task Manager grouping
is Windows behavior requiring separate proof; process hiding or renaming the game
binary is not part of the design. Native presentation retains independent top-level
windows and recovery; destructive parent reassignment is not an accepted shortcut.

Storage inspection is on demand and reads metadata only, with traversal/count/time
bounds and no reparse traversal. Size estimates may be partial while playing. A cache
cleanup action preserves auth, worlds, and settings. No blanket deletion of Local,
Roaming, or TEMP is allowed.

The storage page now exposes per-profile logical file bytes, quota-aware available
and total disk space, and the separately classified HTTP cache. Disk free space is
shared volume capacity, not a quota or allocation for each of the five profiles.
Unavailable disk metadata stays unknown; cache bytes are part of profile bytes,
not additional usage. Measurements remain on demand, without reading file contents.

The sole session owner delegates filesystem details to
independent_bootstrap/profile_storage.rs. The only disposable subtree is the
observed private Local/Temp/minecraftpe/codeBuilderV1/Default/Cache HTTP cache.
Default itself, Cookies, Local Storage, GPU/Code Cache, other TEMP contents,
authentication helpers, worlds and presets are excluded. This follows Chromium's
[HTTP disk-cache design](https://www.chromium.org/developers/design-documents/network-stack/disk-cache/);
CEF's [cache_path documentation](https://chromiumembedded.github.io/cef/general_usage.html#cefsettings)
also shows why its broader profile/cache root can contain persistent storage.
The exact Education path is observed metadata, not a general Minecraft cache API.

Cleanup requires explicit per-profile selection/confirmation, all managed games
stopped and no externally discovered Minecraft game. Backend admission is fresh
and serialized with launches. A new bounded plan pins every ancestor and opens
every candidate exclusively before marking any file for deletion by its retained
handle. Locked files, reparses, hard links and incomplete scans refuse the plan;
there is no path-based recursive delete, directory removal or background cleaner.
Failure during disposition can remove part of this disposable cache and is
reported explicitly. This does not prove every unowned helper has exited or
prevent an external same-user process starting later; currently open cache files
are protected by exclusive admission. Subsequent cache recreation is normal.

Space uses Windows [GetDiskFreeSpaceExW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw)
available-to-caller and total-to-caller values, preserving user quotas. Disposable
fixture cleanup passed; cleanup/login persistence on actual profiles and the
complete storage page remain unexecuted. Cache deletion is disk maintenance,
not a claimed RAM/gameplay optimization.

Four native DWM preview destinations are implemented as nonactivating top-level
windows owned by Lazy Tester. They render only controller-attributed game surfaces;
clicks enter the existing selection path, and input goes directly to the selected
game. No screenshot/video encoder or game input forwarding is introduced.

Background presentation uses the existing checkpoint/helper owner. Live-preview
mode parks ordinary game windows beyond the virtual desktop, keeping them visible
to Windows and retaining the main viewport size across role swaps. Stable parked
geometry is checked before sending another placement command. This avoids forced
SW_HIDE in that mode but does not prove how Minecraft handles offscreen occlusion.
Dialog-blocked/unresponsive windows are not newly parked; affected tiles report
unavailability and retain normal game access/recovery.

A native recovery gate is jointly retained by the GUI and its recovery helper
until the original checkpoint has been restored. A reopened GUI refuses capture
while that old handoff is still in progress. No new persistent readiness flag or
account state is created. Preview requests/events are coalesced/bounded; the
worker stays idle without redraw/capture loops. Thumbnails hide when the manager
is minimized or another application is foreground. DPI/client bounds and tile
non-overlap are checked before moving/adopting windows.

The mounted interface now has the large selected view/four lower cards, manual
launch with pending-window state, and queued explicit selection. A new session
opens only through a user launch action. The old app.css is no longer bundled.

Native DWM registration on hidden fixtures is not live Minecraft proof. Pixel
freshness, preview click/input, foreground/background FPS, offscreen Education host
simulation, DPI/minimize/dialog recovery and taskbar/Task Manager behavior still
need integrated runtime acceptance. Final installed branding and cache cleanup
also require acceptance. The native installer candidate now has an explicit
separate Lazy Tester identity and only the two reviewed engine DLLs; existing
settings/profile paths are retained. It excludes private profile bindings, whose
first-run import is implemented for the reviewed existing profiles. Packaging ownership and migration decisions
are in engine/distribution/README.md. Their state and the next bounded action
are recorded in the sole handoff, not inferred from this design contract.

## Historical workspace decisions below (superseded where inconsistent)


> Current direction (21 October 2026): Sandboxie and its derived fork are retired
> for new sessions. Isolated launch/provision is blocked; independent provider is
> not implemented. Existing-session controls preserve recovery and data. Earlier
> fork-specific behavior below is historical. See
> [retirement register](../system/legacy-retirement.md).



# One Minecraft workspace

Operator-approved design, 20 October 2026, amended by explicit operator approval
of independent native composition after the crash experiments. A developer
integration now provides the play area and session rail; real game acceptance
and live previews remain incomplete.

**20 October acceptance correction:** the operator reports unstable app switching
and now requests genuine in-app control/containment. Independent composition has
failed user acceptance and remains a developer experiment, not a completed substitute
for hosting. No single taskbar entry or safe child-window hosting is proven. The
existing destructive SetParent failure must not be bypassed to hide an icon.
Current corrections preserve recovery controls and handle minimized updates;
they do not resolve this architectural requirement. See the canonical test report.

### Latest operator clarification: browser-like shell

One manager taskbar entry and one navigation surface are required. Minecraft
01–04 behave like stable tabs: select one to play, keep other sessions alive,
close only the chosen session after normal save prompts. Separate game processes
in Task Manager are expected and must not be hidden. Multiple application buttons
and independent game-window navigation are not the intended accepted UX.

The existing provider/catalog remains the launch, profile and ownership authority.
The desktop presentation module now retains up to four verified presentation
leases, with one active viewport. Background sessions are hidden and retain their
original recovery checkpoints; Settings suspends rather than detaches them.
Startup adopts uniquely attributed responsive sessions without launching duplicates.
These are developer mechanics; the operator's blank/unresponsive-game report
still prevents product acceptance. Tabs alone do not prove usable gameplay.

The 21 October recovery increment validates engine/profile ownership on admission
and binds that authority to each retained lease. A running-tab swap validates the
live process/window identity without repeating package hashing or full profile
inventory. The manager's own region excludes the validated viewport while a game
is presented; it becomes opaque again for Settings, minimize and recovery. A
bounded raise restores the game's original topmost eligibility immediately.
This remains native composition and can use a classic Windows title frame.
In-game exclusive fullscreen is not accepted by the manager maximize/restore test.
The current App Compartment engine feature is inactive: its five-minute evaluation
limit prevents durable four-session acceptance. New owned launches fail preflight
until the engine route is resolved; four Education licenses alone do not establish
this separate engine capability. See the current Windows report for evidence.

Required implementation checks before enabling a single-shell mode:

- Retain per-tab recovery checkpoints and commit selection only after a successful
  switch; reject PID reuse, foreign profiles, stale HWNDs and partial adoption.
- Obtain fresh native/provider identity even for hidden background windows;
  `MainWindowHandle` being zero is not proof that a hidden game exited.
- Keep background games alive without stealing focus; handle minimize/restore,
  dialogs, resize, fullscreen, DPI changes and normal close in one sequence.
- Change shell/taskbar eligibility only after recovery is established, restore
  it on detach and host/helper loss, and test Explorer restart. Never use owner
  or parent reassignment merely to suppress an icon. Taskbar suppression alone
  is not interactive hosting or proof of one-shell behavior.
- Prove the native baseline versus composed game behavior, then one/two/four tab
  switching; no multi-client escalation while the observed game hang is unresolved.

Windows offers [taskbar item removal](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-itaskbarlist-deletetab)
and [shell eligibility styles](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles).
The developer implementation hides a window before clearing `WS_EX_APPWINDOW`
and adding `WS_EX_TOOLWINDOW`, then shows the selected session. Original shell
eligibility is restored on detach/recovery. This follows the documented
[dynamic taskbar style procedure](https://learn.microsoft.com/en-us/windows/win32/shell/taskbar).
Visual taskbar count, Explorer restart and game-dialog behavior are not accepted yet.
No Task Manager process concealment, game injection or new isolation provider is
authorized or required by this UX. Windows Sandbox remains outside the design.

## User experience

The operator's 21 October correction selects one row of Minecraft 01–04 tabs
above one large play area. Duplicate lower Open/Play cards are removed. Selecting
an unopened tab opens its session; selecting a running tab shows its existing
process. Other sessions stay alive. Close game is a separate explicit action.
Each tab includes a short state label. A failed switch retains the previous
selection and reports the destination failure below the play area. Stable client
identities come from the existing canonical target catalog. Live previews are
outside this bounded correction. Never present a screenshot as an interactive
game or describe native composition as true embedded hosting.

The normal flow is open the manager, open a client, sign in inside Minecraft,
and play. Startup discovers and verifies the registered Education installation
and active owned engine automatically. No executable or engine folder picker
belongs in normal Settings. Missing installation or engine produces a short
reason and one Check again action; diagnostics holds paths and technical detail.
Detection cannot silently adopt official Sandboxie or activate a driver.
Automatic setup does not remove required Windows UAC or manual licensed login.

The ordinary workspace has a compact header, an active-client label, session
controls and Help/Diagnostics. Gameplay requires no page scrolling. The preview
rail becomes compact client tabs on a narrow window. Preserve the game's aspect
ratio; use letterboxing rather than stretching. Keyboard focus remains visible,
status has text as well as color, and loading/error views retain usable actions.

## Interaction and recovery contract

| Situation | Required behavior |
| --- | --- |
| Open an absent client | Prepare its canonical persistent profile through the existing provider; obtain explicit compatibility consent once before reduced-containment creation; manual login only |
| Open clicked twice | One launch request, bounded observation, no automatic launch retry |
| New session loading | Current playable session remains available until the new surface is ready |
| Select a preview | Validate provider/profile/PID/start/session/HWND; commit active identity only after presentation succeeds |
| Presentation fails | Keep the old active client; explain the failed action; never kill or restart either game |
| Mouse captured by game | A tested workspace escape action releases capture; Ctrl+Alt+Space is only a candidate until tested, with conflict handling |
| Login, save or UAC prompt | Keep the prompt visible and reachable above the game surface; never auto-answer it |
| Preview capture stalls | Mark preview unavailable; do not infer game crash or close the game |
| One client fails | Other clients remain usable; no global restart |
| Focus belongs to another application | Do not steal it because a background observation/capture completed |
| Audio | Follow active session only after per-session audio ownership is proven; never mute an ambiguous process tree |
| Settings opened | Native surface cannot cover settings, dialogs or keyboard navigation; restore it safely on return |
| Close manager with games open | Confirm intent, request normal closes, allow save prompts; no forced termination |
| Exit manager only | Offer only after detach/restoration is proven; do not hide or orphan games |
| Manager failure | Restore usable standalone game windows without closing games or losing unsaved worlds; test with synthetic windows before game use |
| Resize, minimize, restore, monitor or DPI change | Preserve correct input coordinates and usable surfaces; do not alter game DPI awareness silently |

No token copying, game patching, injected input/capture DLL, new credentials
store, account migration, unsupported FPS cap, or background auto-retry.
Preview refresh has a bounded budget and no overlapping capture work; displayed
game FPS and preview refresh are different measurements. Background clients
continue running. Target active displayed FPS remains 60 with separate evidence.

## Implementation seam and decision gate

The first owner is `apps/desktop/`: one presentation module for surface hosting,
layout, capture and recovery. It consumes freshly verified process/window
identity from the existing typed bridge and owned/native provider. It does not
create a second lifecycle manager, readiness database or profile catalog.
Frontend transient selection is not evidence that a game is ready or playable.

Cross-process SetParent is a candidate, not an approved shipping implementation.
Microsoft documents explicit style handling and cross-process DPI-reset risks.
Capture can supply previews but cannot replace interactive rendering/input.
Native surfaces can also cover webview dialogs. A successful synthetic parent
change therefore does not establish game input, graphics, popups or recovery.

The synthetic crash experiment rejects putting a game directly below the
manager/UI process: terminating that parent destroys the synthetic client's
HWND, even though its process can remain alive. A separate native presentation
owner with the UI and game as sibling children survives a synthetic UI-process
failure and can restore the game window. This is only a candidate presentation
boundary, not an implemented product helper or a second lifecycle authority.
Failure of that presentation owner still has no proven recovery. Do not attach
Minecraft merely because the narrower UI-crash scenario passes.

### Approved presentation mode: independent native composition

An opt-in synthetic comparison leaves the client as an unowned top-level native
window, temporarily removes its caption/frame, and aligns it to the workspace's
main rectangle. It does not call SetParent on that client or synthesize input.
Terminating the host leaves the same client HWND alive; an independent test
coordinator restores its saved style and screen bounds and requests normal close.

This is native window composition, **not embedded hosting**. The operator
explicitly selected this approach after the synthetic comparison. This replaces
the earlier strict embedding requirement, without weakening provider attribution,
normal-close, no-injection or account/data-preservation boundaries.

`NativeWorkspace.svelte` and `workspace_surface.rs` implement the first developer
increment. The existing provider authenticates the selected process before the
same-executable recovery helper prepares its original style/rectangle checkpoint.
The independent game is positioned over a 16:9 native play area, with direct
Windows input. Manager connection loss restores through the helper; helper loss
restores through the manager's reader worker. No new lifecycle owner, credentials
store, engine selector or persistent readiness flag is added. Settings suspends
the game before displaying its page; manager close offers to restore and leave
the game running. The side rail currently contains session controls, not live
previews. Combined manager/helper failure has no persistent recovery checkpoint.

The accepted user flow remains one workspace, an active native game region and
three previews, automatic setup and manual login. Before acceptance it must
prove game input/capture release, app switching without focus theft, host resize,
mixed DPI, minimize/restore, z-order, unobscured login/save dialogs, usable
taskbar/navigation, actual preview capture, and recovery when the coordinator
also fails. The current synthetic test does not prove any of those behaviors or
promise a single taskbar entry. No installation, accounts or production gate is
changed by this proposal.

Before accepting the developer integration as complete, prove:

1. Synthetic independent composition, normal release and rejected replacement.
2. Manager/helper failure and restoration without destroying the client window;
   mismatched DPI, minimized/maximized windows and identity changes fail safely.
3. One actual game: rendering, relative mouse/capture/release, keyboard, login and
   save dialogs, resize, minimize/restore, close and reopen inside the manager.
4. Two games: repeated swaps, correct input destination and independent sessions.
5. Four games: live previews, sequential restart, multiplayer and 60-minute soak.
   Measure displayed FPS separately; unavailable measurement is NOT_EXECUTED.

The failed SetParent route remains rejected. Do not call a capture image an
interactive session or substitute synthesized input for the native game.

Evidence belongs in the [canonical Windows test report](../validation/native-pilot-localtest.md).
The [synthetic probe](../../tooling/acceptance/windows/test-workspace-host.ps1)
does not accept game PIDs or external windows and is not a product host module.

Platform references:
[SetParent](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent),
[SetWindowLongPtrW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw),
[Windows Graphics Capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture).
