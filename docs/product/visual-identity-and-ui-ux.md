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
original Close game button. An Escape pressed over Play navigation, the
Coordinates/F3 disclosures, or another WebView control must not silently
dismiss an unrelated close review. The original native CloseOne admission,
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
| UX-F02 | Same review; keyboard focus manually moves to Play navigation | Escape outside review leaves review open and does not send CloseOne | Focus/DOM trace; TARGET_WINDOWS |
| UX-F03 | Storage, cache/settings/full-clear review | Safe Cancel or exact identity input receives focus; Escape within review returns to initiating control | Keyboard trace; TARGET_WINDOWS |
| UX-F04 | F3, Coordinates, optional AI & Testing disclosures | One open at a time, Escape returns to own summary; game focus/keys are not stolen | Keyboard focus trace + WebView2 version; TARGET_WINDOWS |
| UX-F05 | Optional AI & Testing report expires while focused | Report disappears on schedule, no stale gameplay claims; focus returns to active nav only when owned by disappearing disclosure | Recorded keyboard focus + native report age; TARGET_WINDOWS |
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
