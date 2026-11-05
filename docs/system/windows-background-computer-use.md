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
