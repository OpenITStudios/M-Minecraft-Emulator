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
