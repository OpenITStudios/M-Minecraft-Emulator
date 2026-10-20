---
id: document.product.unified-workspace
class: DOCUMENT
domain: product
role: CONTRACT
authority: WORKING
lifecycle: ACTIVE
---

# One Minecraft workspace

Operator-approved design, 20 October 2026. This is the target experience;
the current card dashboard does not implement native game hosting yet.

## User experience

One maximized English-language application contains one large playable game
on the left and three live session previews on the right. Minecraft 01–04
remain stable identities from the existing canonical target catalog. Selecting
a preview promotes that session without restarting either game. The previous
active session takes its place in the preview rail. Never present a screenshot
as an interactive game or external window arrangement as completed hosting.

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

### Proposed deviation: independent native surface (decision pending)

An opt-in synthetic comparison leaves the client as an unowned top-level native
window, temporarily removes its caption/frame, and aligns it to the workspace's
main rectangle. It does not call SetParent on that client or synthesize input.
Terminating the host leaves the same client HWND alive; an independent test
coordinator restores its saved style and screen bounds and requests normal close.

This is native window composition, **not embedded hosting**. It is not an
approved fallback, and the mounted product remains unchanged. Operator approval
of this explicit deviation is required before product implementation because
the existing contract rejects substituting window placement for embedding.

The proposed user flow remains one workspace, an active native game region and
three previews, automatic setup and manual login. Before acceptance it must
prove game input/capture release, app switching without focus theft, host resize,
mixed DPI, minimize/restore, z-order, unobscured login/save dialogs, usable
taskbar/navigation, actual preview capture, and recovery when the coordinator
also fails. The current synthetic test does not prove any of those behaviors or
promise a single taskbar entry. No installation, accounts or production gate is
changed by this proposal.

Before exposing a hosted game in the normal UI, prove:

1. Synthetic cross-process attach, normal detach and partial-failure restoration.
2. Synthetic host failure and restoration without destroying the client window;
   mismatched DPI, minimized/maximized windows and identity changes fail safely.
3. One actual game: rendering, relative mouse/capture/release, keyboard, login and
   save dialogs, resize, minimize/restore, close and reopen inside the manager.
4. Two games: repeated swaps, correct input destination and independent sessions.
5. Four games: live previews, sequential restart, multiplayer and 60-minute soak.
   Measure displayed FPS separately; unavailable measurement is NOT_EXECUTED.

Do not bypass a failed native-host gate by presenting window placement or an
image with synthesized input as the requested workspace.

Evidence belongs in the [canonical Windows test report](../validation/native-pilot-localtest.md).
The [synthetic probe](../../tooling/acceptance/windows/test-workspace-host.ps1)
does not accept game PIDs or external windows and is not a product host module.

Platform references:
[SetParent](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setparent),
[SetWindowLongPtrW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw),
[Windows Graphics Capture](https://learn.microsoft.com/en-us/windows/apps/develop/media-authoring-processing/screen-capture).
