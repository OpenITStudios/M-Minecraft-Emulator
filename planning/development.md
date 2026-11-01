# Current Development Handoff — Lazy Tester

## Decision — 1 November 2026

**State: PREPARED FOR CONTROLLED LOCAL SOURCE PREFLIGHT; MINECRAFT AI INPUT BLOCKED / NOT_EXECUTED.**

Lazy Tester has one native controller for five Minecraft Education Windows
profiles. The separate optional Computer Use candidate includes source for
4/5-client AI control, but remains **compile-OFF, default-OFF and absent from
the normal product installer**. Last native source change before this
documentation cleanup: `22057376b8e60760abaeab968cb69f6d3a9bef45` (Raw Input malformed-argument isolation).
Re-pin the latest `Local` HEAD before any future test; older commits are not
acceptance for the current revision.

**Local test eligibility is not local test success.** No current-exact-revision
Node/Rust/MSVC build, actual Windows game input, release of held keys, visual
capture or multi-AI gameplay acceptance has been executed or verified. This
request authorizes source cleanup, not background tests, game hooks, packages
or use of local account/world data.

## One source graph

```text
External Codex/Claude model (vision/reasoning)
  -> Development/tooling/agent/mcp.mjs (local stdio)
  -> same-image agent_cli.rs
  -> original authenticated native_clients_session.rs
  -> independent_bootstrap/session_controller.rs (one game owner)
  -> independent per-client 96-byte input mappings (opt-in only)
  -> bootstrap-compat C++ Win32 polling/Raw Input candidate (opt-in only)
  -> Minecraft Education's actual input consumer (NOT_VERIFIED)
```

Read-only visual observation is optional via Cua Driver or an unbuilt WGC
worker; frames, IPC acknowledgements and native state writes are not
Minecraft movement or game-level key-up proof.

| Area | What exists | Evidence ceiling |
| --- | --- | --- |
| Five native profiles | Existing manager, prior operator pilot | Not current AI acceptance |
| Hybrid MC01-human + MC02–MC05 AI; five-AI mode | MCP planning and opt-in source | REMOTE_GITHUB |
| Per-client writer lease, transaction epoch, neutral TTL | Native feature-gated source and authored fixtures | NOT_EXECUTED |
| Keyboard, relative mouse, attack/use hooks | Test-only Win32 API candidates | Consumer and gameplay UNKNOWN |
| Per-window capture | Optional Cua/WGC source adapters | Windows capture NOT_VERIFIED |
| Four/five concurrent AI gameplay | Source schedules, not observed control | NOT_EXECUTED |
| Production AI input package/release | Explicitly excluded | BLOCKED |

## Current proof boundary and safety

The single procedure for the next stage is
[Computer Use Windows local test](../docs/validation/ai-computer-use-localtest.md).
Once separately authorized, `DEV.cmd inspect --json`, `catalog`, and
`plan` inspect a clean pinned checkout. `DEV.cmd preflight` may install
locked dependencies and verifies **standard** source; it does not enable
experimental native input. The C++/Rust experiment must be compiled and
reviewed together at the same source revision before any Minecraft test.
A normal `DEV.cmd dev` or installer is not an AI input candidate.

No global `SendInput`, forced desktop focus, undocumented alternative
provider, CI activation, unapproved hooks, credential automation, data
migration, force-close, or source-to-production promotion. Preserve
persistent Minecraft accounts/worlds, original process identity and
normal game exit/recovery.

## One next action

**Only after separate operator approval on a compatible Windows x64 host:**
perform the exact-`Local`-SHA native build and ABI preflight for the matching
opted-in C++/Rust/controller pair. Stop on the first compiler, binary,
identity or IPC failure. Game-level MC01-human / MC02-AI testing is a
separate later gate, not permission implicit in a successful build.

**Proof recorded now:** REMOTE_GITHUB source/tree/doc and consumer inspection
only; executable test suites, Windows/MSVC build, installer and Minecraft
remain NOT_EXECUTED.

All previous detailed development notes were preserved in
[the historical handoff](history/ai-input-source-handoff-through-2026-11-01.md);
the former large MCP implementation README is also preserved in
[historical MCP notes](history/mcp-agent-implementation-notes-through-2026-11-01.md).
These records have no active authorization or alternative `Next`.
