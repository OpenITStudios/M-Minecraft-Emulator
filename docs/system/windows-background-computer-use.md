---
id: document.system.windows-background-computer-use
class: DOCUMENT
domain: system
role: GUIDE
authority: WORKING
lifecycle: ACTIVE
---

# Codex Windows background Computer Use — evidence-grounded direction

**Source-only research decision, 5 November 2026.** This document is a
technical design input, **not evidence of an implemented or functioning
Windows background Computer Use system.** The active product goal is to
let Codex control selected Windows applications without stealing the
operator's real cursor, foreground focus, keyboard, or unrelated windows.
Minecraft Education (up to five independently licensed clients) is a
separate, difficult target to serve via the same Codex workflow.

## Non-negotiable behavior

1. Background means **no** global cursor warp, `SendInput`, focus
   activation/restoration, window raise/restack, silent HID, or automatic
   fallback to a briefly foreground action. If no eligible route exists,
   return `BACKGROUND_UNAVAILABLE` with no action.
2. Bind every observation/action to the intended exact `(pid, HWND)`
   and original native PID/birth/session/window ownership. Recheck
   before/after a potentially delayed action, refuse replaced targets.
   Never let an address or reused HWND authorize a different process.
3. Prefer semantic UI Automation/typed browser actions over pixel-only
   inference. Screenshots are on demand. Never treat a posted message,
   input transport ACK, UIA method return, or hash change as proof of
   application/gameplay effect without independent readback.
4. User-owned MC01 stays protected in hybrid mode. A per-client AI writer
   uses the original native controller and exact lease/epoch/neutral
   recovery; do not create a second Minecraft lifecycle/input authority.
5. Source-only GitHub work until explicit further permission. No
   local/Windows/Codex/Minecraft tests, game hooks, CI, packages,
   installations, credentials or private world access.

## Mac evidence and boundary

- Official OpenAI descriptions show background Codex Computer Use with
  a separate agent cursor on macOS. They do **not** publicly specify
  its exact native event-routing implementation:
  https://openai.com/index/codex-for-almost-everything/
- Public, independently authored macOS implementations show the
  useful *pattern*: process-scoped CoreGraphics input
  (`CGEventPostToPid`), AX actions/value-setting, scoped window
  capture, and virtual-cursor visualization. Cua's research describes
  private SkyLight `SLEventPostToPid` and no-raise focus delivery:
  https://github.com/trycua/cua/blob/main/blog/inside-macos-window-internals.md
  https://github.com/OpenCodexLabs/open-codex-computer-use
  https://github.com/huytieu/computer-harness
- Third-party reverse-engineering reports describe a Codex
  `SkyComputerUseService` binary with cursor/capture structures
  and a separate host add-on. That is version-specific independent
  evidence, not verification that current official Codex uses
  the exact Cua/SkyLight action implementation:
  https://apache.googlesource.com/maka/+/refs/heads/agent/1341-graph-read-model/docs/codex-cursor-reverse-engineering.md
  https://apache.googlesource.com/maka/+show/851712a51464266512e078883f464cbcdc8365c9/docs/codex-pip-reverse-engineering.md
- REA is a candidate to inspect **authorized, supplied binary or
  application artifacts**, link imports/string references/call graphs,
  and preserve exported evidence. No `SkyComputerUseService` or
  macOS host binary was supplied in this ChatGPT/GitHub source
  session; **REA was not executed**. REA's own skill says skip binary
  analysis for complete open-source repositories:
  https://github.com/morluto/rea/blob/main/.agents/skills/reverse-engineer-anything/SKILL.md
- macOS private SkyLight functions are **not portable Win32 APIs**;
  an agent-cursor overlay is visual feedback, not an input mechanism.

## Windows evidence — closest existing implementation

Primary source: Cua Driver (MIT core):
https://github.com/trycua/cua
https://cua.ai/docs/cua-driver/concepts/platform-support
https://cua.ai/docs/cua-driver/concepts/how-cua-driver-works
https://github.com/trycua/cua/blob/main/libs/cua-driver/rust/Skills/cua-driver/WINDOWS.md

- Already supports Codex MCP registration and strict
  `delivery_mode:"background"` for supported Windows applications.
  Semantic UIA `Invoke` / value patterns, snapshot-bound
  `element_token`, and supported window-targeted actions are
  preferred. Its Windows source uses PostMessage and guarded
  coordinate injection only where eligible. Occluded/unsupported
  cases refuse with `background_unavailable`; its docs classify
  games/canvas controls as generally requiring foreground.
- The source also has an **optional** brief-foreground fallback
  (`CUA_DRIVER_ALLOW_BRIEF_FOREGROUND`), which MUST remain
  disabled for our strict-background goal. No caller may silently
  escalate from `background` to `foreground`.
- `run_steps` can batch bounded decisions with one observation;
  `get_window_state` can omit screenshots. App-owned state
  verification and refusals are more important than successful
  event injection.
- Another cross-platform reference, `munim-computer-use`, has
  Windows UIA and a virtual cursor but its typing pathway temporarily
  borrows the foreground and sends input globally. This violates our
  zero-interruption requirement even if focus is restored:
  https://github.com/munimtechnologies/munim-computer-use
- Windows-MCP / WinApp CLI are further UIA/agent references.
  PostMessage-only game wrappers, HID driver injection, virtual
  cursor overlays and Windows virtual desktops do **not**
  establish independent per-game input:
  https://github.com/microsoft/winappCli
  https://github.com/CursorTouch/Windows-MCP
  https://github.com/alstonmendonca/openMouse

## Integration with current M-Minecraft-Emulator source

| Existing owner | Reuse, do not duplicate | Missing proof / next work |
| --- | --- | --- |
| `Development/tooling/agent/mcp.mjs` | The ONE local stdio MCP handler, admission/identity and target checks | Define scoped `STRICT_BACKGROUND` capability and refusal contract before any write-action API |
| `Development/tooling/agent/visual-capture.mjs` | Optional Cua Driver CLI already called for **read-only** `get_window_state`/PNG with `display_only:true` | Extend carefully to semantic UIA observation, not a recurring PNG-control loop; a display-only snapshot must not be used as a mutable action snapshot |
| `src-tauri/src/engine/agent_cli.rs` + `native_clients_session.rs` | Original authenticated process/session authority | Never replace native PID/HWND/birth/session, user-mode control or writer lease with third-party window names |
| `mc02-virtual-input.cpp` and original per-client writer | Existing compile-OFF experimental Minecraft keyboard/cursor/Raw-buffer polling paths | Minecraft game input consumption remains NOT_VERIFIED; UIA may control menus but cannot be asserted to drive gameplay |

**P58 upstream source pin:** Cua Driver `trycua/cua` `main` at
`044f528265234d255f0792d4929b434b5e9adf8e` (source reviewed
on 5 November 2026). Exact Windows
`get_window_state` and CLI `run_call` source were inspected:
https://github.com/trycua/cua/blob/044f528265234d255f0792d4929b434b5e9adf8e/libs/cua-driver/rust/crates/platform-windows/src/tools/get_window_state.rs
https://github.com/trycua/cua/blob/044f528265234d255f0792d4929b434b5e9adf8e/libs/cua-driver/rust/crates/cua-driver/src/cli.rs
The source indicates `structuredContent` is printed as JSON and a
tree-only call avoids PNG capture; **installed Cua binary compatibility
remains NOT_VERIFIED**.

**P59 source-only action admission:** Cua element token indices can be
zero (source: `element_token.rs`), so the original UIA projection now
accepts `element_index:0`. The existing P58 MCP
`inspect_client_ui` remains the one UIA observation owner. The new
`assess_client_background_action` tool performs a fresh version of
that *same* native-fenced read, then refuses missing, ambiguous,
degraded, disabled or non-Button/non-Invoke targets. A complete enabled
Button with an `invoke` pattern was originally reported as
`SEMANTIC_CANDIDATE_NOT_EXECUTED` (now `BACKGROUND_UNAVAILABLE`
under P61); there is no action token, input
dispatch, focus or foreground escalation, UIA invocation, or proof
of background behavior. Source-only negative fixtures authored,
NOT_EXECUTED. The Cua `InvokePattern` result alone cannot guarantee
the application will not activate itself.

**P60 source-only gate (6 November 2026):** Cua's
`get_window_state` publishes per-row tokens and snapshot identifiers;
a newer snapshot can retire prior tokens. The current read-only
projector checks exact `sXXXXXXXX:index` correspondence and uniqueness
for *all* returned rows. It exports only `snapshotProvenance`, never
the snapshot or token. P59 candidacy refuses missing/mismatched/duplicate
provenance. This is a check **at observation time**, not a lifetime
reservation: other daemon callers may immediately invalidate it.

**Confirmed source blocker:** pinned Windows `click` is not
Invoke-only: Chromium/Electron element clicks can use
`PostMessage`; other cases try Toggle/Selection/Expand or coordinate
injection when Invoke is unavailable or fails. The upstream WPF caveat
in `fg_bypass.rs` explicitly documents self-activation despite its
XAML/Chromium shield. Strict no-focus behavior remains unproven.
Evidence (exact upstream revision):
https://github.com/trycua/cua/blob/044f528265234d255f0792d4929b434b5e9adf8e/libs/cua-driver/rust/crates/cua-driver-core/src/element_token.rs
https://github.com/trycua/cua/blob/044f528265234d255f0792d4929b434b5e9adf8e/libs/cua-driver/rust/crates/platform-windows/src/tools/get_window_state.rs
https://github.com/trycua/cua/blob/044f528265234d255f0792d4929b434b5e9adf8e/libs/cua-driver/rust/crates/platform-windows/src/tools/click.rs
https://github.com/trycua/cua/blob/044f528265234d255f0792d4929b434b5e9adf8e/libs/cua-driver/rust/crates/platform-windows/src/uia/fg_bypass.rs

**No execution route admitted in P60.** Future mutation requires
a no-fallback Invoke-only primitive that refuses alternate dispatch
*before* acting, pre/post native PID/HWND/birth/session validation,
app-specific target-owned postcondition, and verified user focus/z-order
isolation. Neither generic UIA churn, screenshot hash nor tool ACK is
sufficient. Uncertain timeouts must not cause automatic replay.
Minecraft gameplay remains separately unverified. All tests authored
NOT_EXECUTED; no Windows/Codex/Minecraft activity.

**P61 source-only contract correction (6 November 2026):** The
upstream Cua Driver `main` revision inspected at this stage,
`290552c6374da181314e26d69156198f7d48c0df`, retains the
relevant Windows `click`, `invoke_menu`, `set_value` and
`fg_bypass` behavior from pinned P58/P60 source:
- `click` is **not** an Invoke-only API: in element-token mode
  it can select Chromium `PostMessage`, alternate UIA patterns
  or targeted coordinate injection. `action:"invoke"` is not
  a guarantee to bypass these fallbacks.
- `invoke_menu` calls `force_foreground_assisted` and reports
  foreground delivery. It must not be used under strict background.
- `set_value` has separate Value/Range/Combo paths and UIA
  handlers that may activate the window; it does not solve Invoke.
- `fg_bypass` documents an unresolved WPF self-activation
  class. Even a direct UIA Invoke transport would still require
  per-application focus isolation evidence.

Upstream source links for this exact review:
https://github.com/trycua/cua/blob/290552c6374da181314e26d69156198f7d48c0df/libs/cua-driver/rust/crates/platform-windows/src/tools/click.rs
https://github.com/trycua/cua/blob/290552c6374da181314e26d69156198f7d48c0df/libs/cua-driver/rust/crates/platform-windows/src/tools/invoke_menu.rs
https://github.com/trycua/cua/blob/290552c6374da181314e26d69156198f7d48c0df/libs/cua-driver/rust/crates/platform-windows/src/tools/set_value.rs
https://github.com/trycua/cua/blob/290552c6374da181314e26d69156198f7d48c0df/libs/cua-driver/rust/crates/platform-windows/src/uia/fg_bypass.rs

**Implementable minimal transport contract (not implemented):**
1. A future action must be **UIA Invoke-only or refused before
   any other kind of dispatch**. Do not route this through Cua
   generic `click` or `invoke_menu`; do not use desktop or
   foreground scopes, `PostMessage`, `SendInput`, pointer,
   fallback UIA patterns or silent retries. Require a semantically
   supported, enabled target and explicit no-fallback behavior.
2. Resolve the provider-owned UIA element and its snapshot **inside
   one bounded Cua operation**, with exact PID/root HWND/birth/session
   from the original native controller before dispatch and again
   afterward. A token/index received by Codex in a prior tool
   result is not valid authority. Serialize relevant reads/actions;
   stale/replaced target must refuse rather than retarget.
3. Verify focus/cursor/z-order unchanged across the complete
   operation for the intended class of target; a pre/post check
   alone can miss brief focus theft, so actual Windows continuous
   disturbance observation is required for acceptance. Treat
   WPF/Chromium/unknown app handlers as unsafe until evidence
   supports them; no undocumented focus-restore workaround.
4. Require a **target-owned typed postcondition** selected for the
   exact application action before dispatch: a known state
   transition exposed by the application or its independently
   trusted status interface, not generic UIA label disappearance,
   any screenshot/hash change, a tool ACK or click route field.
   The current Minecraft UIA tree has **no verified app-owned
   postcondition**. The future verifier must bind before/after
   to the same controller identity and fail without a known
   expected change.
5. If an attempt may have been dispatched and then times out or
   loses identity, report `OUTCOME_UNKNOWN_NO_RETRY`; never
   claim no effect, replay automatically or target a new HWND.
   If a gate fails before any dispatch, report
   `BACKGROUND_UNAVAILABLE` with no attempted action.

The existing MCP P59/P60 assessor is **not** an action route.
To avoid suggesting otherwise, P61 changes its positive semantic
result to `BACKGROUND_UNAVAILABLE` /
`CUA_INVOKE_ONLY_ROUTE_NOT_AVAILABLE`; it retains
`semanticCandidate:true` only as non-executable diagnostic
information. No new action tool, token export, native controller
change, Cua binary install, test execution or Minecraft input.
Source fixtures were authored; not executed.

**P62 architecture decision — reuse Cua upstream, no local driver fork
(6 November 2026; REMOTE_GITHUB only).** The minimal owner of a
potential strict Invoke action is Cua Driver's Windows tool registry
(`platform-windows/src/tools/impl_.rs`) and its existing
`ToolState.snapshots` / `cua-driver-core/src/snapshot_store.rs`.
These already handle daemon-owned snapshots, a retained UIA element
and PID/HWND window-ownership admission. The native session controller
in Lazy Tester continues to own MC slot/birth/session identity; its
MCP bridge remains the ONE stdio entrypoint. The existing Cua
`get_window_state` read is not an action reservation.

**Selected upstream design, not present in any installed binary:**
a narrowly-scoped `invoke_element_strict` *candidate* in Cua's
original registry (NOT a new Windows controller and NOT a new
`click` alias). Its eventual API must accept one exact PID,
root HWND, expected process birth FILETIME/session, and a
provider-internal snapshot-bound element selector. No name-only
target, bare row number, arbitrary client-supplied token, desktop
scope, numeric coordinate, modifier, repeated click, or model-supplied
fallback. The Cua operation itself must:
1. Refuse when its tool capability, app-class focus isolation or
   app-specific outcome contract is unavailable. In this revision
   **all app classes remain unverified and non-executable**.
2. Check expected process birth/session and original top-level
   HWND ownership using OS-observed evidence at the native boundary
   (not merely PID/HWND from a caller). Reconcile against the
   original controller both before and after the bounded call.
   Retain/resnapshot a uniquely identified, enabled UIA Button in
   that same operation; refuse stale/replaced/ambiguous UIA rows.
   A retained COM pointer alone does NOT prove live window identity
   and pre/post checks cannot eliminate every race.
3. On the actual execution path allow *only*
   `IUIAutomationInvokePattern::Invoke` once; refuse unavailable
   pattern/unsupported host without running `PostMessage`,
   `SendInput`, pen/touch, Toggle/Selection/Expand, menu
   activation, cursor overlay animation or focus/z-order
   shims. No `fg_bypass` window-disable guard may be presumed
   zero-disturbance without separate evidence.
4. Return separate machine-readable distinctions:
   `BACKGROUND_UNAVAILABLE` before any dispatch,
   `OUTCOME_UNKNOWN_NO_RETRY` if dispatched/possibly dispatched
   with missing ACK, timeout, lost identity or untrusted readback,
   and `TARGET_POSTCONDITION_VERIFIED` only when a previously
   declared, independent app-owned state change is confirmed.
   The UIA Invoke COM return and Cua action `effect`/route
   are **transport evidence only**, not task success.
5. Require Windows evidence of no real cursor movement, keyboard
   interception, window restacking or **transient** foreground
   change for the specific target class. A simple before/after
   foreground check misses brief focus theft. Unknown WPF, XAML,
   Electron/Chromium and Minecraft UIA providers remain refused
   until each relevant route is demonstrated safe.

**Postcondition reuse decision:** Cua already exposes
`verify_state` with bounded, stable window/element predicates via
`cua-driver-core/src/expectation.rs`; reuse that where its
readback is truly a declared application-owned outcome and matched
to the exact native target. Its `satisfied` status alone **does
not certify Minecraft gameplay**, app-internal state or
nondisturbance. The current Minecraft target has no reviewed
task-specific state oracle, so its strict action route remains
`BACKGROUND_UNAVAILABLE`. If an app has no trusted typed oracle,
do not invent one from UI label disappearance, screenshot hashes
or a generic snapshot diff.

**Compatibility / no-overdevelopment decision:** do not vendor Cua,
add a second daemon, change the current MCP tool list, introduce
an unpublished feature flag that appears to work, or invoke
`click` / `invoke_menu` as a substitute. Prefer a source-reviewed
upstream extension or maintainer-supported tool followed by an
exact version/capability pin; only then consider a disabled-by-default
MCP adapter in the existing owner. An undocumented local fork,
a not-yet-released Rust tool or an arbitrary `cua-driver.exe`
path is **not** an installed capability. Upstream feasibility,
Windows focus isolation and exact postcondition are
NOT_VERIFIED/NOT_EXECUTED.

Specific Cua source evidence at
`290552c6374da181314e26d69156198f7d48c0df`:
https://github.com/trycua/cua/blob/290552c6374da181314e26d69156198f7d48c0df/libs/cua-driver/rust/crates/platform-windows/src/tools/impl_.rs
https://github.com/trycua/cua/blob/290552c6374da181314e26d69156198f7d48c0df/libs/cua-driver/rust/crates/cua-driver-core/src/snapshot_store.rs
https://github.com/trycua/cua/blob/290552c6374da181314e26d69156198f7d48c0df/libs/cua-driver/rust/crates/cua-driver-core/src/expectation.rs
https://github.com/trycua/cua/blob/290552c6374da181314e26d69156198f7d48c0df/libs/cua-driver/docs/action-result-contract.md

**P58 source-only implementation:** `inspect_client_ui({slot})`
uses the existing configured Cua Driver `get_window_state`
CLI with `include_screenshot:false` and up to 64 structured
UIA elements/depth 10. The original native controller verifies
PID/HWND/birth/session **before and after** the read. Raw values, window
titles, screenshot bytes, action tokens and untrusted diagnostics are
not exported. Empty or incomplete trees remain unverified. This
does not drive a window or game and Windows behavior is NOT_EXECUTED.
A later, separately reviewed step may permit **only** explicit
UIA/window actions with `delivery_mode:"background"`, exact
snapshot/identity fencing and target-owned readback. Do not import the
whole Cua runtime or introduce a generic session manager into Lazy
Tester. Do not wire foreground fallback.

## Explicitly NOT established

- No Windows API equivalent to SkyLight's universal process-scoped
  game input has been verified.
- No Windows native game session, same-client gameplay movement,
  independent five-client background keyboard/mouse, or Cua Driver
  compatibility with Minecraft Education has been observed.
- No tested CPU/RAM/token savings, Mac binary decompilation by REA,
  WinApp CLI installation, Codex use or Windows session execution.
- If an app requires foreground for an essential action, strict
  no-interruption mode must refuse it. A VM/Cloud PC is a separate
  isolation route, **not** a silent fallback into a heavier product.
