---
id: document.product.visual-identity-and-ui-ux
class: DOCUMENT
domain: product
role: GUIDE
authority: REFERENCE
lifecycle: ACTIVE
---

# Lazy Tester — Minimal Professional visual identity and UX consistency

**Scope:** standalone product name **Lazy Tester** only in product-facing
branding. No parent-brand lockup, "powered by", decorative gaming motif or
unverified gameplay implication. This reference is subordinate to the active
[unified workspace](unified-workspace.md), actual desktop source, and native
session ownership. It does not change installed identifiers, release metadata,
private profiles, session authority, or packaged installer compatibility.

## Professional UX closure criteria — 7 November 2026

**Scope and evidence rule:** this is the canonical **product UX quality bar**,
not a second project tracker, approval register or status database. The
current handoff remains in `planning/development.md` and the Windows
verification owners remain under `docs/validation/`. A source change is
**SOURCE_PREPARED**, never a Windows PASS until actual execution is
authorized, observed, and attributed to one exact commit. Each known
source risk and its necessary proof must remain explicit; avoid claiming
that ten rounds of source commits constitute finished UI acceptance.

### Non-negotiable user journeys

1. **Starting:** once Windows native capability is confirmed, present
   exactly five stable, disabled MC01–MC05 controls while session data
   is unavailable. Unsupported or still-unverified host capability
   does not advertise a native game inventory. Native `profileSlots` and the
   original five-slot parser remain authoritative; UI placeholders are not
   observed STOPPED processes and must never enable Launch.
2. **Playing:** one selected native 16:9 window and five stable keyed DWM
   targets in unchanged slots. A DWM failure does not mean the process
   stopped, is signed in, or is playable.
3. **Recovering:** every error or uncertain status has truthful wording
   and only the original read-only Refresh / verified Select admission.
   Never offer a parallel game controller or fake success.
4. **Changing settings:** per-slot unsaved preferences are reversible;
   saving, failure and unconfirmed receipts are visibly different.
   Re-entering Settings must not silently discard a user's work.
5. **Managing storage:** measurements distinguish unavailable from zero;
   show the full date and time of the last UI-received report. Clear,
   restore and full deletion use the original native admission.
6. **Confirming risky actions:** switching pages does not silently
   dismiss a pending CloseOne or Data Storage review, nor the in-flight
   full-data-clear action. Cancel is explicit; Escape only applies while
   keyboard focus is inside the active review.
7. **Using secondary tools:** F3, Coordinates and optional AI & Testing
   are passive, truthful disclosures, not new game owners. Manual
   coordinate drafts stay isolated per launcher and are never attributed
   to an actual world or player.
8. **Navigating:** all interactive controls have visible keyboard focus,
   logical Tab order and discernible state. Focus remains visible and
   returns predictably when short-lived UI is removed.

### Prioritized closure matrix

`SOURCE_PREPARED` means source exists and has been inspected, **not**
compiled or visually accepted. `RUNTIME_GATE` means the next claim
requires separately authorized execution. `OPEN_SOURCE_RISK` is a
known candidate for another bounded source correction if evidence
supports it. Do not treat these as release approval.

| Priority | Area and blind spot | Current source position | Closure evidence |
| --- | --- | --- | --- |
| P0 | Confirmed Windows host shows fewer than five cards during pending/failed binding | SOURCE_PREPARED: five fixed slot placeholders, disabled and unavailable | Delayed binding/failed binding/connected recovery, five stable DWM nodes |
| P0 | Clicking navigation silently cancels open CloseOne/Storage review | SOURCE_PREPARED: route guard, disabled nav and explicit Cancel guidance | Escape, brand click, top/side tab, and restored keyboard focus |
| P0 | Navigate while full-data-clear request is in flight | SOURCE_PREPARED: navigation remains unavailable until result | No page change before native confirmation/uncertainty handling |
| P0 | Unsafe Launch / Close / Clear with stale native state | Existing native/controller gates retained | Actual original-process and token admission; no second client launched |
| P0 | Five native viewport and preview identities | Existing owner retained | 920×620 and 1280×900, mixed five-slot HWND/DWM snapshots and stable coordinates |
| P1 | Incomplete loading, offline and stale profile observations | Existing status projection; source-only confidence | Startup loss/retry with visible Disabled/Unavailable states, zero false readiness |
| P1 | Long messages, varied card states and error overflow | SOURCE_PREPARED: keyboard-readable 13px bounded notices, 28px recovery actions and per-slot accessible guidance | Tab/scroll/Narrator access, no focus clipped or text permanently unreachable |
| P1 | F3 / Coordinates popup placement and exclusivity | Source geometry + HTML details group | WebView2 named-details compatibility, overlap/scroll/resize at minimum window |
| P1 | Optional AI & Testing header crowding on narrow/high-DPI view | SOURCE_PREPARED: flex summary within fixed-height topbar; only expanded details overlay | Show/hide at 920×620 and 125–200% DPI without covering Play or nav |
| P1 | Keyboard Escape, Cancel and ephemeral focus restoration | SOURCE_PREPARED: same-slot notices, transient header actions and scoped Settings/Storage save/review focus anchors, no native-focus takeover | Focus trace, game keyboard unaffected, nested, disabled-control and offline-refresh cases |
| P1 | Dirty forms, saved versus unconfirmed receipts | Source guards/drafts present | Preset success/failure/retry; FPS invalid; discard and return flows |
| P1 | Storage measurements, cache, recovery and full deletion | Source guards and dated measurement label | Last-measured clarity; reversible cancel; disposable data-only destructive test with explicit approval |
| P1 | Accessible contrast, non-color state and hit target sizing | SOURCE_PREPARED: semantic tokens, Settings controls, 28px launcher recovery actions and 32px Close game review targets | WCAG 2.2 AA contrast, 24px target/spacing, forced-colors and Narrator screenshots |
| P1 | Narrow Settings/Storage and zoom/reflow | Responsive source scopes present | No hidden controls/dual-axis text scrolling at supported Windows scales |
| P1 | Browser regression previously sent reversed profile order rejected by parser | SOURCE_PREPARED: valid inventory journey plus separate rejection fixture | Authorized synthetic browser suite and original parser tests |
| P2 | App icon quality and independent visual identity | One SVG source and app title retained | Native Taskbar 16/24/32/48px, Win10/11 dark/light/forced-color review |
| P2 | Repeated raw colors, selector specificity and spacing density | SOURCE_PREPARED: specialized surfaces and action colors tokenized; redundant global button focus rule removed | Token audit + rendered component inventory, consistent states/spacing |
| P2 | Consistent naming, help and disclosure discoverability | Source wording partially unified | English copy review with representative first-time and expert users |

The highest-priority source risks must be resolved before calling this
phase source-complete. `RUNTIME_GATE` rows cannot be closed by writing
more unexecuted fixtures or generating mock UI images.

### Required visual acceptance artifacts

Use the existing validation system, not a new runner: capture sanitized
screenshots and DOM/focus measurements of **Play, General, Data Storage,
all five mixed statuses, delayed startup, connection failure, preview
unavailable, CloseOne, cache/restore/full-clear review, F3, Coordinates,
AI & Testing (when truly reported), and transient error/recovery**.
Record the exact 40-character `Local` SHA, OS/WebView2 versions, viewport
size, DPI, expected/observed result and PASS/FAIL/INCONCLUSIVE. Minimum
sizes are 920×620 and 1280×900 with 100%, 125%, 150% and 200% DPI.
Tests of profile data clearing require a disposable profile and separate
destructive-action approval. Absent evidence remains `NOT_EXECUTED`.

Quality references: [Windows navigation principles](https://learn.microsoft.com/windows/apps/design/basics/navigation-basics),
[Windows focus navigation](https://learn.microsoft.com/windows/apps/develop/input/focus-navigation),
[WCAG 2.2 Focus Not Obscured](https://www.w3.org/WAI/standards-guidelines/wcag/new-in-22/),
[WCAG Non-text Contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast),
[WCAG Target Size Minimum](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum).
The app is a native-window host; spatially necessary DWM layout still
requires its own Windows acceptance rather than a guessed web-only
reflow result.

## Identity source of truth

- Master vector asset: `Development/apps/desktop/src-tauri/app-icon.svg`.
- Motif: **Focused Workspace** — a selected foreground frame over a retained
  background frame. It must remain legible on the Windows Taskbar at 16–32 px.
- One source generates packaging PNG/ICO through existing
  `Development/apps/desktop/scripts/prepare-icons.mjs`. The WebView titlebar,
  Play header, and empty workspace use the same SVG by CSS URL; do not embed a
  second independently maintained logo.
- Name/wordmark: **Lazy Tester**, in the platform UI family. Do not add a
  tagline inside the manager.
- Accessibility: decorative inline logo is hidden from the accessibility tree
  because adjacent text already names the app. Taskbar icon has the app title.
- Do **not** change Tauri identifier, publisher metadata, install folder,
  Windows registration, profile storage layout or native installer hooks as part
  of visual branding. Legal/compatibility metadata is not a UI brand lockup.
- Other logos, game marks, copyrighted Minecraft assets, and lookalike game UI
  are out of scope.

## Palette and component rules

The current dark presentation remains primary, with reusable semantic tokens
under `.lt-app` in `src/lazy-tester.css`:

| Token | Value | Intent |
| --- | --- | --- |
| `--lt-bg` | `#14171e` | Application canvas |
| `--lt-panel` | `#1c2029` | Standard surface |
| `--lt-surface-raised` | `#242a36` | Secondary action / popup |
| `--lt-text` | `#edf0f7` | Main copy |
| `--lt-muted` | `#b5bfd0` | Secondary copy |
| `--lt-accent` | `#bac2ff` | Focus, selected outline, primary action |
| `--lt-control-outline` | `#667387` | Functional control boundaries |
| `--lt-line` | `#343b48` | Decorative separator only |
| `--lt-success` | `#95dfbd` | Positive state with text |
| `--lt-warning` | `#d3b480` | Uncertain state with text |
| `--lt-danger` | `#efacb6` | Error text and risk with text |

Never use color as the only indication of state. Functional control outlines
must have sufficient non-text contrast; decorative separators need not use the
same color. Do not copy magic hex colors across Svelte components. Use Segoe
UI/system font, 14px body/control labels where layout permits, minimum 12px
help/captions, explicit focus rings, and reduced-motion/forced-color support.

## Core UI interaction grammar

1. **Understand:** selected profile, last native observation, and operation
   uncertainty are distinct. `RUNNING` is a native state, not authenticated
   Minecraft gameplay. Display "reported running" only for native `RUNNING`
   profiles; count `STARTING` separately. The conservative existing occupied
   count may continue to block unsafe settings or launches.
2. **Act:** use existing typed bridge and original Rust controller.
   `Launch`, `Close`, `Restore`, `Clear`, `Present` remain bounded by
   native admission, never button availability alone.
3. **Feedback:** show request, confirmed receipt, failure or unconfirmed
   separately. Do not show backend exception strings in the UI. No spinner
   implies success; no preview failure implies a stopped game.
4. **Recover:** specific actions near their affected launcher or form.
   Never auto-relaunch, overwrite sessions, force-close, or auto-reset data.
5. **Drafts:** draft graphics preferences are ephemeral per launcher, retained
   while the manager is open and discarded only on explicit save/discard or
   application exit. Manual coordinate text remains while its disclosure is
   collapsed; it is still an operator-entered note with no world attribution.
6. **Focus:** cancel actions are accessible by keyboard. The original
   selected-game Close confirmation defaults focus to Cancel, accepts Escape,
   and returns focus to the invoking control.
7. **Disclosure:** Play tools may be grouped visually but may not become
   independent game owners. Optional AI/Testing reports remain the original
   transient, accurately qualified single disclosure, not a new dashboard.

## Hard workspace constraints

- Preserve the central 16:9 native viewport and its exact measured bounds.
- Preserve five stable, keyed native DWM launcher-card elements, order, and
  target identity. Do not reparent, sort, virtualize or replace preview targets.
- Keep the original session scheduler and its stale-result/close gates. Do not
  add a second observer, listener daemon, IPC channel, dashboard data owner,
  new overlay, or activation on app startup.
- Keep app-facing display text in English until an authorized full
  localization design; do not introduce half-translated strings.

## Blind spots and acceptance plan (not yet executed)

| Check | Source-level expectation | Needed runtime proof |
| --- | --- | --- |
| Taskbar and titlebar identity | One SVG and original icon-generation route | Inspect 16/24/32px on Windows dark/light Taskbar |
| Density | Five cards and selected window stay fixed | 920x620, 1280x900, larger windows, 100/125/150/200% DPI |
| Tool disclosure | F3/Coordinates present in existing Play header | Verify no popup clipping, overlap, obscured focus |
| Graphics draft | Per-profile unsaved choice survives switching/settings navigation | Verify switching, save, discard, failed save |
| Coordinate draft | Closing panel preserves raw/labels; no automatic clipboard/game action | Verify collapse/reopen and switching active launcher |
| Status summary | `RUNNING` only in reported-running label; starting separately | Mixed five-slot live session observations |
| Keyboard | Visible focus; safe Close confirmation; proper Escape | Keyboard and screen-reader walkthrough on Windows |
| Contrast | Main/secondary text and functional borders use semantic tokens | Contrast and forced-colors review at real rendered surfaces |
| Performance | No extra preview, observer or polling owner | Native DWM performance and layout acceptance |

**Evidence ceiling at authoring:** REMOTE_GITHUB source structure and
consumer review only. Executed Svelte/Node/Rust tests, Windows package,
live Minecraft sessions, screen-reader and visual acceptance are
`NOT_EXECUTED`. New production automation and GitHub Actions remain off.

**Next bounded visual work:** inspect layout/keyboard behavior across the
supported Windows sizes and failure states when separate runtime approval
is given; otherwise continue source-only fixes on the first UI owner.


## Storage review keyboard policy — source-only addition

The three existing inline confirmations (web-cache cleanup, saved-settings
recovery, and full profile deletion) share one UI-only keyboard grammar:

- **Open:** cache/settings review focuses **Cancel**; full deletion keeps the
  existing initial focus on the typed exact-profile identity field.
- **Cancel:** a visible Cancel or **Escape from within the active review** returns
  focus to the original action button if it is still connected and enabled.
  Escape outside an active review is not intercepted and cannot steal game input.
- **Admission:** reviews remain inline; they are **not** modals and do not trap
  Tab or add a new lifecycle state. All actions still use existing controller
  admission, confirmation tokens, and fail-closed outcomes.
- **Disabled guidance:** settings and Storage explain unavailable changes
  using the last observed result; explanations are not proof of permission.

This is an authored source contract, **not** an executed Windows accessibility,
Tauri, keyboard, or screen-reader acceptance result. The existing
`Development/apps/desktop/test/storage-review-focus.test.ts` source fixtures
are `AUTHORED_NOT_EXECUTED`.

## Bounded Play disclosures — source-only policy

F3 Inspector, Coordinates & Saves, and optional AI & Testing are HTML
`details` disclosures in one exclusive `name="lt-play-disclosures"` group.
Only one opens at a time without another session controller, observer, polling
loop or timer. HTML `details[name]` requires a compatible WebView2 engine
(Edge 120+); older installed runtime compatibility is **NOT_VERIFIED**.

The two Play tool panels use one pure UI geometry projection,
`app/presentation/playToolPopup.ts`, to clamp to available width and height
below or above the triggering summary at 16px viewport margins. A panel is
hidden until an open/resize/scroll calculation returns its visible location.
Popup scroll does not recompute placement. This does not move or resize the
native 16:9 aperture, five keyed DWM previews, or selected Minecraft window.

Escape from within an active disclosure returns focus to its summary. Keyboard
focus moving from an open disclosure to another app control collapses it.
A missing `relatedTarget` does not discard the panel or manually entered
coordinate drafts. No clipboard operations or unverified game input are added.

**Evidence:** authored geometry/source fixtures only. Typecheck, Svelte
runtime, WebView2 compatibility, 920×620 and DPI matrix, scroll/focus
navigation, native DWM and Minecraft acceptance remain **NOT_EXECUTED**.

## Settings / Storage navigation consistency — source-only update

- The existing `LazyTesterApp.svelte` navigation dispatcher retains route
  sequencing and native `workspaceSurface.suspend/present` ownership.
- On explicit navigation into **General** or **Data Storage**, the page heading
  receives programmatic focus after the Svelte view update. Returning to Play
  does not redirect focus into its game-facing native viewport.
- A short, live status announces that current Settings/Storage data is being
  inspected. It does not claim that current values are game-verified or saved.
- **Graphics** offers an explicit `Discard draft` operation which resets
  only the currently selected in-memory graphics choice to the last saved
  preset. Other profile drafts are unaffected; no Tauri or game IPC is called.
- **Data Storage** keeps its launcher selector disabled while a cache/settings/
  deletion review is active. An inline explanation tells users to finish or
  cancel before switching. Native confirmation tokens and all existing
  action-admission checks remain unchanged.
- Below 1050 CSS px, Settings/Storage rows may wrap secondary controls instead
  of forcing wide rows. The central Play viewport and all five keyed DWM
  preview tiles retain their original dimensions and ordering.

These are source-only improvements. Visual layout at 920×620, DPI 125–200%,
keyboard/screen reader focus, WebView2, Svelte typechecking and Minecraft
runtime remain `NOT_EXECUTED` without separate authorized execution.

## Field feedback, storage uncertainty, and bounded danger actions — source-only

General Settings shows specific, accessible feedback for the two existing
Frame Limits inputs. Each invalid number input has `aria-invalid` and
`aria-describedby` referencing a visible field-specific explanation. The
existing `validFpsLimits` and native `savePerformance` admission remain the
only decision path for saving; the new UI expressions are strictly display
projections. Values must be integers between 5 and 1000 and the background
limit cannot exceed the active limit.

The Data Storage full-profile deletion confirmation still requires the
exact profile label, explicit acknowledgement and matching original
confirmation token. Nonempty mismatched text is marked invalid, described
inline, and cannot enable deletion. These explanations do not trigger extra
deletion attempts or alter the existing native fail-closed checks.

Storage measurements distinguish three conditions: no storage report,
no measurement for the selected profile, and unavailable session status with
previously measured sizes. In none of these states is an unmeasured value
assumed to be zero or the profile assumed empty.

Only Settings/Storage receives additional narrow-window wrapping for the
danger action row, long storage values, form feedback and confirmations.
The five-card Play rail, native 16:9 viewport, DWM preview placement and
associated owners are outside this change.

**Proof ceiling:** SOURCE_REVIEWED only at the exact committed revision.
New regression expectations are `AUTHORED_NOT_EXECUTED`. Svelte typecheck,
Windows keyboard, screen reader, 920×620 and DPI layouts, installer,
five-client gameplay and native recovery remain `NOT_EXECUTED`.

## Minimal Professional — visual hierarchy and launcher status labels

The active five-card launcher rail is unchanged in number, keyed slot order,
thumbnail positioning, 16:9 aspect ratio, and selected native presentation.
The accessibility name on a card describes the **observed action**, not an
invented launch: select when the native state is `RUNNING`, launch only
when the existing `canRequestLaunch` predicate permits it, and otherwise
announce the last native state (or unavailable connection). This is
presentation-only; native admission remains authoritative.

Launcher card status text is unchanged. The selected status uses the
existing accent color, and `ATTENTION` / `NOT_RESPONDING` use the warning
color. A failed DWM preview is amber rather than a fatal-game error.
Color is supplementary to the English textual status, not a status source,
gameplay readiness indicator, or new alarm mechanism. No preview/card
height or width is altered.

General Settings receives explicit section names for Graphics, Frame
Limits, App behavior and Installation & data. Data Storage groups its
existing three measurements under **Storage overview** and exposes the
original destructive section with an accessible heading. No metrics,
permissions, profile data or automatic observations are added.

Settings/Storage action buttons have one shared 36px minimum height,
13px label size, and control radius; hover/review styling is scoped only
to Settings and Storage. The new token does not resize Play controls,
the five-card rail, or game windows. A small set of previously stale source
fixture expectations for the existing **Refresh** recovery wording were
aligned with the current source contract, without changing product behavior.

**Evidence:** exact GitHub source review only; fixtures are
`AUTHORED_NOT_EXECUTED`. Svelte/TypeScript, native Windows visual
accessibility, screen reader, 920x620 and DPI scaling, DWM preview geometry,
and five-client Minecraft acceptance remain `NOT_EXECUTED`.

## Long-copy and mixed-status launcher readability — source-only update

- Keep all five launcher cards in their current keyed storage-identity order.
  The existing native DWM preview targets and 16:9 center aperture are not
  moved, replaced, virtualized, hidden or reparented.
- Each card derives one observed `statusLabel` for its visible status,
  hover title and accessible action name. A process/window `RUNNING` state
  does not assert a playable, signed-in or authenticated Minecraft world.
  A blocked or unobserved card never announces an available Launch action.
- At narrow effective widths, the launcher name and status use a single
  visual line with an ellipsis. Full text remains in DOM, hover title and
  the accessible button name, without making one card taller than others.
- `Your games` uses one presentation-only full summary of the original
  `RUNNING`, `STARTING` and action-required observations. It is visually
  constrained to one line but the complete text remains exposed for
  screen-reader reading and mouse hover. It is never a second process
  counter, telemetry owner or gameplay certification.
- The existing per-card recovery message is bounded to 38px. Both preview
  and window/session guidance remain scrollable, now also keyboard-focusable
  with a clear focus ring and `aria-describedby` pointing to the exact
  authored guidance. The status message and any offered original recovery
  action remain distinct: native Refresh/Select admission is unchanged.
- Main Play's authored error, original binding recovery and uncertain
  window-restoration notices are keyboard-focusable when long. Alert and
  exit-recovery content has an explicitly bounded scrolling region so long
  copy does not indefinitely obscure the game viewport. No raw native
  exception text becomes visible and no modal/game-keyboard owner is added.
- The source-only fixture covers card labeling, full summary exposure,
  keyboard-readable recovery and unchanged preview boundaries.

**Evidence boundary:** verified only through exact GitHub blob, reference
and consumer review. New and updated fixtures are
`AUTHORED_NOT_EXECUTED`; WebView2/Svelte compatibility, live screen reader,
Windows 920x620 at 100/125/150/200% DPI, native DWM, five mixed licensed
sessions and Minecraft gameplay acceptance remain `NOT_EXECUTED`.


## Keyboard focus and Windows visual acceptance matrix — source-only plan

The original CloseOne review now handles **Escape only when the event
originates inside the focused confirmation section**. Opening CloseOne still
focuses **Cancel**; Escape there still cancels and restores focus to the
original Close game button. An Escape pressed over another enabled WebView control (such as a
window control) must not silently dismiss an unrelated close review;
Play/Settings navigation remains disabled until explicit Cancel. The original native CloseOne admission,
normal-exit process owner and stale-inspection gates remain unchanged.

Short-lived `AI & Testing` controller reports may vanish while keyboard
focus is inside their disclosure. Immediately before the read-only
component unmounts, it attempts to move focus to the existing currently
selected top-navigation button, **only if focus is inside that disclosure**.
It never redirects focus from a game, starts an observer, or retains an
expired report. WebView2 runtime focus restoration is NOT_VERIFIED.

The existing opt-in headless-browser journey had role queries for the
older bare `Launch Minecraft - N` / `Select Minecraft - N` names. The
present launcher names also expose observed status, so those source-only
queries use action+slot prefixes. When the native state blocks Launch,
assert against the exact keyed card and ensure its accessible name does
not promise launch. This aligns test intent with product behavior; no
fixture was run and no failure was masked.

### Additional visual/interaction acceptance checks

This is a **supplemental source-authored QA matrix**, not new test-case
registry, a runnable script or an extension of the existing 306-case
test master. All rows are `NOT_EXECUTED` until separately approved
Windows/runtime evidence exists.

| Check | View and setup | Required observed acceptance | Evidence and ceiling |
| --- | --- | --- | --- |
| UX-F01 | Play, selected running slot, close review open | Cancel receives initial focus; Escape inside review cancels, restores original Close game focus without a game exit | Keyboard trace + native CloseOne absence; TARGET_WINDOWS |
| UX-F02 | Same review; Play/Settings navigation disabled; focus moves to an enabled titlebar control | Escape outside review leaves review open and does not send CloseOne | Focus/DOM trace; TARGET_WINDOWS |
| UX-F03 | Storage, cache/settings/full-clear review | Safe Cancel or exact identity input receives focus; Escape within review returns to initiating control | Keyboard trace; TARGET_WINDOWS |
| UX-F04 | F3, Coordinates, optional AI & Testing disclosures | SOURCE_PREPARED: one open via named group, scoped Escape, Tab/Shift+Tab and deferred null-focusout/blur close without game-focus hijack | Keyboard focus trace + WebView2 version; TARGET_WINDOWS |
| UX-F05 | Optional AI & Testing report expires while focused | Report disappears on schedule, no stale gameplay claims; focus returns to active nav only when owned by disappearing disclosure | Recorded keyboard focus + native report age; TARGET_WINDOWS |
| UX-F06 | Keyboard focus on launcher Refresh/notice while native Inspect changes status | Refresh holds focus in authored notice, then returns to enabled same-slot card or named non-Tab fallback on unavailable status; never steals a newer or Minecraft focus | Browser and Windows focus trace + native Inspect receipt; TARGET_WINDOWS |
| UX-F07 | Keyboard Refresh becomes disabled, or a confirmed CloseOne/full-clear review disappears | Original focused context remains on the page heading, same-slot launcher group, or persistent Storage warning heading; Tab/Shift+Tab user choices always prevail | Synthetic DOM/focus trace and Windows WebView2 focus + native receipts; TARGET_WINDOWS |
| UX-F08 | Settings graphics/FPS/audio save becomes temporarily disabled; Storage cache/settings review is submitted | Owned WebView focus moves to its persistent section/Storage overview heading only; keyboard traversal continues and Cancel still returns to original trigger; native admission unchanged | Authored browser/source fixtures, Windows Tab/Shift+Tab, failure/success receipts, Narrator; TARGET_WINDOWS |
| UX-F09 | Popup F3/Coordinates or AI & Testing loses WebView focus; focusout has null relatedTarget | Popup closes without re-focusing WebView/native game. Escape from inside returns to its summary; Tab/Shift+Tab moves naturally to the next control | Authored synthetic focus/blur fixture and Windows native-focus/WebView2 evidence; TARGET_WINDOWS |
| UX-V01 | 920×620 at 100/125/150/200% display scaling | All five keyed MC cards remain ordered, central 16:9 viewport remains positive and fully in bounds; no clipped essential action | Sanitized screenshot + measured DOM/native DWM rectangles; TARGET_WINDOWS |
| UX-V02 | 1280×900 and expanded desktop window, 5 mixed observed states | Names/status and long errors remain accessible; selected card never moves/reparents, no stale running count | Screenshot, text/accessibility snapshot, native state receipts; TARGET_WINDOWS |
| UX-V03 | Long connection/recovery messages, narrow effective width | All support references and warnings remain reachable by Tab/scroll; no overlay covers the native game or critical button | Keyboard/scroll video or screenshot; TARGET_WINDOWS |
| UX-V04 | Popup F3/Coordinates at minimum viewport and scrolled Settings | Popups stay within viewport, do not displace native tiles, do not carry stale selected game data | Panel bounds + screenshot + native selected identity; TARGET_WINDOWS |
| UX-A01 | Windows High Contrast, 200% text, reduced motion | Focus outlines, action state and error meaning are visible without relying on color alone | Windows UI accessibility screenshots + screen reader traversal; TARGET_WINDOWS |
| UX-S01 | Storage full clear in disposable profile, operator-approved separately | Typed exact profile name + acknowledgement + native token required; data is never cleared merely by UI validation | Synthetic/disposable safe setup and original native admission evidence; TARGET_WINDOWS + explicit destructive-action approval |

For each executed row, capture one pinned 40-character commit SHA,
Windows build, WebView2 version, DPI scaling, viewport dimensions, native
window-binding identity as sanitized metadata, control focus before/after,
expected/observed result, and PASS/FAIL/INCONCLUSIVE/BLOCKED. Do not
upload credentials, private world names or raw machine identifiers.
Record outcomes only through existing validation/handoff evidence paths.
Source fixture authoring and repository review are `REMOTE_GITHUB`,
not Windows, Minecraft gameplay, or five-client acceptance.

**Current phase proof:** exact GitHub source/blob checks only.
Svelte/typecheck, browser fixture execution, Windows focus/DPI testing,
DWM runtime and Minecraft remain **NOT_EXECUTED**.

## Post-eight-stage source consistency review — fail-safe draft and CSS cascade

A static review across the exact `Local` UI source and authored fixtures
found no mismatches in the direct literal source assertions sampled. That
is **not** proof the Vitest fixtures, Svelte compiler, WebView2 or real game
sessions execute successfully. Dynamic assertions and source compiler
behavior remain unverified.

One concrete source behavior gap was corrected: if an original
`applyGraphics` request fails or returns an **unconfirmed**
`changedProfiles` receipt, the chosen preset is retained as a UI-only
per-slot draft before `settings` becomes unavailable. Users can revisit
Settings after a deliberate status refresh without silently losing that
choice. The original Rust admission, confirmation, disk writes and backup
semantics are unchanged; an unconfirmed save never shows a saved state.
Successful native confirmation still removes the draft. A visible status
sentence now uses the exact **Lazy Tester** product spelling.

One known CSS cascade conflict was removed: four Play-tool font sizes
were already effectively **12px** because of later duplicate overrides.
Those 12px values now reside at the original component selectors and the
redundant group override is removed. Effective visual values stay the same;
no launcher, DWM, native aperture, tooltip location or input-coordinate
dimensions have been altered.

**Proof ceiling:** `REMOTE_GITHUB` source/blob/consumer review and authored
fixtures only. The exact current SHA must still undergo separately
authorized Svelte/TypeScript execution, viewport visual inspection,
WebView2 keyboard/accessibility, Windows DWM and Minecraft acceptance.
No prior source-only result is promoted to executable PASS.

## Svelte binding and Play-popup lifecycle — source-only correction

The desktop uses **Svelte 5 in legacy component syntax**, with existing
`export let`, `$:`, `bind:this` and `on:` handlers. This bounded
correction stays in those existing component owners. It is not a rewrite
to runes or a new UI/session state manager. Compiler and WebView2
behavior remain unverified until separately authorized execution.

### Per-launcher Coordinates & Saves drafts

`CoordinatePastePanel.svelte` previously retained its unsaved XYZ text
and manually entered labels when the selected MC slot changed. The
underlying `saveBookmark` action already pinned the selected slot at
request time, but this reuse of input could silently associate an MC01
draft with MC02.

The original single mounted panel now keeps at most one **ephemeral
in-memory draft per MC01–MC05 launcher**, plus the optional unassigned
state. On selecting a different launcher, the panel saves the current
manual text/format/name/world/dimension to that original draft and
restores only the new selected slot's draft. New slots start blank.
Returning to an earlier slot restores its own unfinished text; collapsing
the disclosure does not discard it. This provides no world or player
identity verification and never reads the clipboard or sends a game
command. Stored Saved Locations are still read through the original
explicit notebook operation and filtered to the requested native slot.

The original save admission remains responsible for validating
current XYZ and manual labels, and for persisting only through the
existing explicit saved-location bridge. Drafts are not persisted,
uploaded, automatically saved, or merged across slots.

### Scrolled-away popup anchor

Both `F3InspectorPanel.svelte` and `CoordinatePastePanel.svelte`
retain one browser-computed fixed popup derived from the triggering
`summary` geometry. During a captured window scroll or resize, an
anchor that has moved entirely outside the visible WebView now closes
its own existing HTML `details`, rather than leaving a detached
floating panel over Play or the launcher rail. Internal panel scrolling
still does not reposition it. The original exclusive HTML disclosure
group, focus-scoped Escape, five preview targets, viewport dimensions,
and native ownership stay unchanged.

The source-only fixtures now cover draft isolation on slot switches,
the existing non-destructive on-close behavior, and both offscreen
popup guards. The old visual-foundation fixture was updated to reflect
the accurate per-launcher help copy.

**Proof ceiling:** REMOTE_GITHUB source/blob/consumer check plus
`AUTHORED_NOT_EXECUTED` fixtures only. No Svelte typecheck, browser
fixture, WebView2 support/scroll verification, Windows 920×620/DPI
rendering, DWM preview, or Minecraft session acceptance was executed.

## UX closure audit and source corrections (7 November 2026)

The source correction in this revision pins the Windows-native manager
inventory to five unobserved, non-launchable UI slot placeholders while
session binding is pending, until the original native controller supplies
a complete, validated five-slot snapshot. Unknown host capability does
not show a game inventory; the confirmed unsupported-host policy remains
unchanged: no native-game inventory or commands. A status-less placeholder cannot trigger Launch, Select,
Close, DWM recovery or data operations by itself.

Page transitions are held while a CloseOne / Storage review is open, or while
full profile deletion is in progress. The disabled navigation and review
help explicitly direct the operator to **Cancel** before switching. The
original shared UI dispatcher, native CloseOne, Storage approval/backup
and deletion token checks remain unchanged.

The synthetic browser regression now tests a delayed native binding with
five non-actionable cards. The prior reversed native snapshot fixture was
invalid because the native-schema parser explicitly requires five ordered
slots; it now verifies stable card identity through **valid** updates,
while a separate source fixture asserts reversed snapshots are rejected.
None of these tests was executed.

The Storage timestamp now includes a date and a time so an older measured
size does not read as a current-day measurement. This remains the time the
UI received the controller report; it does not inspect file changes itself.

Remaining high-risk items include optional topbar disclosure crowding,
minimum-window/high-DPI actual geometry, screen-reader focus, target sizing,
all unconfirmed native receipts, and true five-session DWM performance.
These are **NOT_EXECUTED**, not passed.

## Optional AI & Testing header containment — source-only

The controller-reported AI & Testing summary previously used
`position:absolute;right:230px` in the app topbar. This ignored the
navigation and connection text in the available width and could overlap
controls at narrow effective WebView sizes.

The same summary now participates as a **shrinkable flex item** in the
existing fixed-height header. Only the expanded read-only details remain
an overlay. The bar remains 56px high (54px in the existing short-height
media query); the game viewport and five keyed DWM targets are not moved,
resized or reparented. The status is still absent without an original
controller report, and no new event listeners, observers or polling are
added.

At <=860px CSS width, the topbar reduces gaps and horizontal padding.
At <=650px, its product wordmark yields visually to the original logo,
but its link retains the explicit accessible name `Lazy Tester — Play`.
AI status and the connection label shrink rather than overlap; the full
connection text remains in the DOM and title. On especially narrow views,
the expanded details use a bounded fixed panel inside the WebView, rather
than extending outside the left boundary.

The existing optional synthetic-browser journey now contains
**authored, unexecuted** geometry assertions for 920, 720, 620 and
480 CSS-pixel widths, the 54px header at 620px height, nonoverlapping
topbar children, and popup containment. These are planned checks, not
Windows display-scale results.

**Unresolved:** Svelte typecheck, synthetic-browser execution, WebView2,
screen reader, Windows 100/125/150/200% DPI, native DWM coordinates,
and actual Minecraft runtime are all `NOT_EXECUTED`.

## Semantic control tokens, pointer states and source contrast audit — 7 November 2026

The active Minimal Professional stylesheet continues to own **one** dark
semantic palette within `.lt-app`. Previous literal colors for the
advanced outline and expanded surface, preview placeholder, alert
surface/outline, destructive button/border/foreground, hover and the two
overlay shadows are now named custom properties within that owner.
The named surfaces keep their previous colors (including the existing
red destructive button), with two deliberate exceptions: the Advanced
options outline changes from #53618c to #6e7b9a to improve functional
boundary contrast, and the background-audio checkbox reuses the main
accent token instead of a slightly different violet.

### Interactive-state policy

- **Primary actions:** Settings/Storage primary buttons have a readable
  accent hover; disabled buttons retain the existing native admission
  and show a less-faded 0.68 opacity rather than 0.5.
- **Secondary actions:** original secondary buttons use a restrained
  shared hover surface. Existing Settings-specific hover rules retain
  their higher specificity.
- **Destructive actions:** full-profile deletion retains its warning
  color and exact native confirmation-token gate. Its hover uses the
  existing danger-hover token with readable light text; forced-colors
  mode uses Windows system colors rather than an unrecognizable dark red.
- **Keyboard:** the redundant early button-only focus rule was removed.
  The later common `:focus-visible` rule remains the canonical visible
  focus ring for buttons, anchors, summaries, fields and selectors.
  Error inputs retain their dedicated invalid border state.
- **Targets:** in Settings/Storage only, graphic/FPS selectors and the
  sidebar navigation use the already-established 36px control minimum.
  The audio and deletion acknowledgement labels have a minimum 32px
  clickable label height. Text-only configuration disclosures have a
  32px minimum target and an explicit hover indicator. These do **not**
  change the Play controls, five card previews, 16:9 aperture or DWM
  positioning.

### Contrast calculation and evidence ceiling

From the authored opaque hex tokens, static sRGB relative luminance
calculations yield representative contrasts:

| Pair | Calculated contrast | Intended check |
| --- | ---: | --- |
| Main text / canvas | 15.72:1 | Normal text >=4.5:1 |
| Muted text / panel | 8.79:1 | Normal text >=4.5:1 |
| Accent text / selected panel | 6.50:1 | Normal text >=4.5:1 |
| Warning text / standard panel | 8.24:1 | Normal text >=4.5:1 |
| Control outline / standard panel | 3.39:1 | Functional non-text >=3:1 |
| Advanced outline / standard panel | 3.85:1 | Functional non-text >=3:1 |
| Advanced outline / expanded surface | 3.23:1 | Functional non-text >=3:1 |
| White destructive text / red action | 7.33:1 | Normal text >=4.5:1 |
| White destructive text / red hover | 5.66:1 | Normal text >=4.5:1 |

The new `design-token-accessibility.test.ts` provides a **source-authored,
unexecuted** contrast calculation against actual CSS token declarations,
plus controls for hover, focus, disabled and target-source rules. The
optional existing browser regression also has authored target-height
checks (minimum 24 CSS px, Settings form controls 36 CSS px).
This is based on [WCAG 2.2 Non-text Contrast](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast)
and [WCAG 2.2 Target Size Minimum](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum);
small controls may qualify through the official spacing and other
exceptions, which have **not** been audited empirically here.

**Not yet accepted:** rendered contrast, outline clipping, actual
pointer bounds, Windows 100/125/150/200% scaling, Narrator, screen-reader
output, forced-colors, 920x620 navigation, Svelte compilation, DWM
placements and Minecraft gameplay all remain `NOT_EXECUTED`.
These rows are `SOURCE_PREPARED`, **not** accessible UI PASS.

## Launcher recovery target and focus clarity — source-only update

The canonical five-card rail presents one primary **Launch/Select** action
per native profile, plus the original **Close game** review button when
the observed session state allows it. A per-slot read-only health notice
may additionally offer **Refresh** or a verified **Select game** action.
Previously the text-like health action used only 3px of vertical padding
and no minimum height, so its hit area could be less than the WCAG 2.2
24px CSS target minimum.

The existing `SessionHealthNotice.svelte` action now has
`min-height:24px`, 6px horizontal click padding and a 4px top margin
to approximately offset the previously used 6px margin. The original
`onAction(item.slot,item.action)` handler, `disabled` admission and
`guidedRecoveryAllowed` projection are **unchanged**. No action is
started by focusing a message, no native IPC is added, and the label
does not imply the game is playable.

The action now references the **same existing per-slot authored guidance
ID** that the keyboard-scrollable status message already uses. This
provides context when assistive technology reaches the individual
Refresh/Select game button. Hover uses the existing selected surface;
High Contrast uses Windows `LinkText`, `ButtonText`, `GrayText` and
`Highlight`. Focus styling remains visible.

The separate `Close game` normal-exit review target retains its full
card width and now has an explicit 32px minimum height. Its click still
starts the original Cancel-first close confirmation rather than closing
Minecraft immediately. Changes apply only to these **secondary controls**;
launcher card width, the fixed five-slot grid, the 16:9 preview-image
CSS rules and the selected-game native aperture are not edited.

An authored synthetic-browser regression measures the actual rendered
Refresh target (at least 24x24 CSS pixels), Close game button (at least
32px tall), the nonzero preview placeholder and the recovery button's
`aria-describedby` relationship. A focused source fixture records
those controls and unchanged original safety gates. Both test sources
are **AUTHORED_NOT_EXECUTED**.

**Remaining gate:** whether these adjustments shift the actual native
DWM rectangles, keyboard focus, tooltip rendering, Narrator/High Contrast,
Windows DPI 100–200%, responsive overflow and gameplay is
`NOT_EXECUTED`. Keep this source work **SOURCE_PREPARED**, not UI
acceptance PASS.

## Disappearing launcher health notices — scoped focus recovery

The existing five-slot `SessionHealthNotice.svelte` can disappear after a
new native Inspect because the original `projectSessionHealth` projection
no longer reports the per-slot problem. Previously its scrollable message
and individual Refresh button could own keyboard focus at the moment
their component was removed. If the button was disabled as the original
inspection began, focus could be lost before a clean observation arrived.

For a **focused Refresh action only**, the existing UI first moves
focus into that slot's read-only, scrollable authored status message
before asking the original `onAction(slot,'refresh')` callback to run.
There is no additional refresh request, retry, watcher or game input.
The separate **Select game** action remains on its original native
presentation path; this focus handoff never reroutes an explicit
attempt to enter Minecraft.

If the Refresh-type notice then unmounts with focus still inside it,
the component waits for the Svelte DOM update and validates all of the
following before restoring focus:

- The original MC01–MC05 launcher entry is still connected in Play.
- The WebView still has focus; an actual native Minecraft focus
  transfer must not be stolen back.
- No other user-selected WebView control has acquired focus.

The existing parent entry is now a programmatically focusable
`role="group"` with a slot-specific accessible name and
`tabindex="-1"`. This is **not** a new Tab stop, launcher button
or native-session owner. If the original launcher card is active and
allowed by its existing native-observation gate, focus returns to
that card; otherwise it returns to the non-actionable same-slot
group, where a visible ring preserves user context. This does not
make a disabled card interactive. A removed entire Play page or an
operator's newer focus choice takes priority over restoration.

The authored, **unexecuted** browser regression now describes three
negative/positive cases: clean Inspect returning a now-enabled MC02
card, lost Inspect returning a disabled MC02 card plus safe group
focus, and an unrelated deliberate Play-navigation focus that must
not be overridden. The new source fixture checks the original
action callback, DOM lifecycle gate, group focus semantics and
unaltered five native preview targets.

**Evidence ceiling:** pinned `REMOTE_GITHUB` source/fixture review
only. The `UX-F06` acceptance case remains `NOT_EXECUTED`.
Svelte lifecycle behavior, browser focus heuristics, WebView2,
Windows game-focus isolation, Narrator/forced-colors, all DPI
settings and Minecraft gameplay remain `NOT_VERIFIED`.

## Disabled and disappearing controls — focus continuity source contract

Three already-existing user actions can disable or remove the very control
that holds WebView keyboard focus. The original session, native close and
destructive-data controllers remain the only action owners.

- **Header Refresh:** for keyboard-generated clicks (`detail=0`) with
  focus actually on the Refresh button, the UI first focuses the existing
  page's non-Tabbable `h1`. Refresh still follows the original
  `manualRefreshRequested` / `refreshPage` sequence. After the
  original request finishes and Svelte updates the button, focus returns
  to that same Refresh button **only if** the page is unchanged, the
  WebView still has focus, the heading still owns focus, and the button
  is connected and enabled. A user who pressed Tab/Shift+Tab to another
  control is never redirected. Pointer clicks do not initiate this
  keyboard-only focus handoff.
- **Confirm Close Minecraft:** the reviewed confirm action reanchors
  focus, before removing its panel, to the existing same-slot
  `lt-launcher-entry` (`tabindex=-1`). This is safe even when native
  CloseOne remains unconfirmed and its launcher card is disabled.
  The original Rust CloseOne admission, required review, and Cancel
  behavior are unchanged; the reanchor neither selects nor launches
  the game and does not pull focus back from a native game.
- **Permanent profile-data clear:** after the original exact-slot,
  typed-name, acknowledgement and controller-token UI guards pass,
  focus moves from the disappearing confirmation form to the persistent
  **Clear all data** heading (`tabindex=-1`) if the form truly owns
  WebView focus. The original native deletion is still requested only
  once through the same controller and its own safeguards. No new
  confirmation path, storage owner or automatic removal is introduced.

The two headings have explicit focus rings, including system Highlight
when forced colors are active. They are **programmatic anchors**, not
new Tab stops or modal traps. Original five DWM preview rectangles and
selected 16:9 aperture sizing remain unchanged.

The existing optional synthetic-browser journey has new
**AUTHORED_NOT_EXECUTED** scenarios for: Refresh while disabled,
Tab and Shift+Tab before Inspect completes, safe restoration only
while the heading retains focus, and same-slot focus after the
original CloseOne review is submitted. Permanent data-clear focus
is represented in source fixtures **without** triggering the
destructive operation. The explicit `UX-F07` acceptance row
remains `NOT_EXECUTED`.

**Evidence ceiling:** remote exact-source/blob checks only. Svelte
compilation, Edge/WebView2 focus and keyboard behavior, screen reader,
Windows 100–200% DPI and forced colors, native DWM/game-focus
handoff and Minecraft runtime still require separately authorized
execution; none is a current PASS.

## Settings saves and Storage review completion — transient focus continuity

A previously focused **Save preference / Apply preset / Save changes**
control can be disabled during a pending operation and remain disabled
after a successful save because there is no longer a dirty change.
The checkbox for **Mute background games** is likewise temporarily
disabled during the existing native audio request. The original
callbacks already own all admission and success/failure messages.

The existing `GeneralSettings.svelte` now gives its Graphics and
Frame Limits section headings and Background game audio heading
`tabindex="-1"`, with visible focus outlines. Each existing save
handler first checks that the WebView still has focus and the triggering
button, form member, or audio checkbox owns focus. Only then does it
focus that unchanged heading, **before** calling the same original
callback. A browser-invalid FPS form never submits and therefore does
not trigger an artificial focus change. Pointer and keyboard events
reuse the existing actions; no new hotkeys or hidden proxy buttons
are introduced. The headings do not enter ordinary Tab order.

Cache/settings confirmation actions inside `DataStorage.svelte`
previously removed their own review surfaces as soon as the existing
parent submitted the request. When the operator is focused inside
one of these forms, the child now moves focus to the persistent
**Storage overview** heading (also `tabindex="-1"`) before forwarding
the original `onClearCache(slot)` or `onRecoverSettings()` callback.
No account/world deletion call is added. The original Cancel-first
review focus, Escape scope, typed full-clear name, acknowledgement,
confirmation token and Rust admission remain unchanged.

The original stylesheet gains only Settings/Storage-scoped heading
focus outlines, including Windows system Highlight in forced-colors
mode. This does **not** change five card previews, any native DWM
target, the selected 16:9 viewport, or the Tauri action graph.

Source checks and `settings-storage-focus-continuity.test.ts` cover
native callback retention, conditional ownership checks, review
Cancel behavior, read-only focus anchors, and CSS geometry invariants.
The existing optional synthetic browser regression has **authored,
unexecuted** checks for completed FPS/Graphics saves and Cancel
returning to the cache trigger. Live background-audio transitions
and actual completed cache/settings mutations remain untested,
rather than being simulated as a destructive acceptance PASS.

**Evidence boundary:** `REMOTE_GITHUB` exact-source and blob review
only. Svelte/TypeScript compilation, Vitest, synthetic browser execution,
WebView2 keyboard/Tab order, Narrator, High Contrast, Windows DPI and
native DWM/Minecraft behavior remain `NOT_EXECUTED`. The new
`UX-F08` row is not accepted until real observations exist.

## Nonmodal Play disclosure keyboard/focus lifecycle — source-only correction

F3 Inspector and Coordinates & Saves use the existing HTML
`details name="lt-play-disclosures"` group. The optional AI & Testing
report is the third disclosure in that same group. There is **no modal
dialog**, no custom Tab key handling, no new gameplay listener, and no
additional process, DWM or clipboard authority.

The previous focusout handler only closed a disclosure when the browser
supplied another DOM `Node` as `relatedTarget`. Windows/WebView focus
can instead leave `relatedTarget=null`, including when the operator
switches to a separate native Minecraft window. The existing component
owner now handles both cases:

- For a real DOM destination outside the open disclosure, close the
  `details` immediately. The new focus target keeps keyboard control.
  Tab from the read-only F3 summary to the Coordinates summary, and
  Shift+Tab from Coordinates to an earlier control, remain native.
- For a null destination, defer **one Svelte tick**, then check the
  actual active element and `document.hasFocus()`. Keep the popup
  open when focus remains inside it; otherwise close it without calling
  `focus()`. The explicit `window.blur` handler also closes the
  disclosure when the WebView loses focus to another native window.
- For an Escape event **originating inside** the open disclosure,
  prevent only that event, close that disclosure and focus its own
  connected summary only while the WebView still owns focus.
  Escape from Minecraft, titlebar, or unrelated page controls is not
  intercepted, and a normal Tab/Shift+Tab departure never bounces
  back to the summary.

The AI & Testing report's existing `onDestroy` focus fallback now
also checks `document.hasFocus()`; an expiring short-lived report
must not steal focus from an actual native Minecraft window.
No new report source, polling or AI status owner is introduced.
Coordinate text drafts, manual labels, bookmark storage and original
read-only F3 provenance are preserved; closing a disclosure is not a
discard operation.

The optional in-memory Tauri synthetic browser journey now has
**AUTHORED_NOT_EXECUTED** cases for F3→Coordinates Tab, Coordinates
input Tab/Shift+Tab, scoped Escape restoration, focus moving to an
unrelated Refresh control, a null-target focusout, and simulated window
blur on all three Play disclosures. The existing source fixtures
have been aligned with those listeners and the guarded AI onDestroy.
This checks source intent only, not correctness of actual WebView2
native-focus/Win32 event ordering.

**Unresolved:** Edge/WebView2 `details name` support, popup hiding
when a native game takes input, Tab/Shift+Tab traversal, focus-ring
visibility, narrator, real 920×620/high-DPI rendering, native DWM
and five Minecraft client acceptance remain `NOT_EXECUTED`. The
`UX-F04` and `UX-F09` rows are `SOURCE_PREPARED`, not PASS.

## Native titlebar semantic keyboard correction — source only (7 November 2026)

The titlebar's branded pointer drag surface is now a **non-focusable**
`div`, not an unimplemented button named "Move Lazy Tester window".
`data-tauri-drag-region` remains on the original drag surface **and**
its two children, respecting Tauri's direct-target drag requirement.
The redundant custom `dblclick` handler is removed: Tauri's native
drag-region behavior retains Windows pointer drag/double-click handling.
The adjacent native Minimize, Maximize/Restore down and Close buttons
remain actual keyboard controls; original CloseRequested/session exit
gates, native window geometry and the selected 16:9 DWM viewport remain
unchanged. No additional keyboard shortcut or native command was added.

The existing visual-foundation source fixture and optional synthetic
browser journey now reject a fake "Move" button and use **Maximize** as
the unrelated focus target for a scoped CloseOne/Escape regression.
These are **AUTHORED_NOT_EXECUTED**, not a Svelte compile, browser,
WebView2 or Windows PASS. Actual native drag, double-click behavior,
focus order, titlebar at high DPI, Narrator and five Minecraft sessions
remain **NOT_EXECUTED**. The previously recorded UX-F02 and UX-A01
acceptance gates remain open.

## Bounded readability and recovery target sizing — source only (7 November 2026)

The existing Play and Settings presentation owners now prefer **13px**
for dense help copy, Storage paths/status, F3/Coordinates/AI disclosure
content and launcher health guidance. This does **not** increase the
fixed Play header summary labels, reposition native previews, replace
the five keyed cards or enlarge the native 16:9 viewport.

The per-launcher read-only Refresh/Select recovery action increases its
base pointer target from **24px** to **28px** while removing its earlier
**4px** top margin. The normal single-line base footprint is therefore
28px in both states; the health message keeps its existing **38px
scrollable maximum**. On unusually narrow cards, multi-line wrapping
may still grow the entry and must be checked rather than assumed safe.
All existing disabled/action admission, focus restoration and per-slot
guidance remain owned by the original components.

The dense Play disclosure content already scrolls within fixed,
viewport-bounded popups. Increasing the content font may change scroll
length, not the width/height of the underlying DWM game targets.
Settings and Storage are separately scrollable and keep responsive
wrapping. The authored source fixtures and optional synthetic browser
journey now check the 13px text scale and 28px recovery target; the
source assertions were inspected but **NOT_EXECUTED**.

**Acceptance still open:** verify five mixed states at 920x620 and
1280x900, 100/125/150/200% Windows DPI, effective zoom/narrow
width, text enlargement, forced colors, Narrator, popup scroll/Tab
access, exact native DWM bounds and game focus. Existing UX-V01,
UX-V02, UX-V03 and UX-A01 remain **NOT_EXECUTED**. This change
is `SOURCE_PREPARED`, not a Windows usability PASS.
