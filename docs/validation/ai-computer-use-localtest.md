---
id: document.validation.ai-computer-use-localtest
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Computer Use — controlled Windows local test

**Scope:** Lazy Tester on Windows 10/11 x64, `Local` branch. The source
for independent Minecraft Education AI input across 4–5 clients exists but
is compile-OFF/default-OFF and **not included in the normal installer**.
No current exact-HEAD experimental C++/Rust build, native input/neutral
acceptance or Minecraft gameplay has been observed. These are future
**approval gates**, not tasks authorized by this document.

The ONE active status/next action belongs to
[planning/development.md](../../planning/development.md).
The current source owners are [MCP](../../Development/tooling/agent/README.md),
[controller](../../Development/apps/desktop/src-tauri/src/engine/native_clients_session.rs)
and [native C++ input](../../Development/tooling/acceptance/windows/bootstrap-ordering/mc02-virtual-input.cpp).
The VMware/Sandboxie G0 and earlier MC02 pilots are historical, not
alternative local-test procedures.

## Non-negotiable safety gates

- Each build/game execution needs separate operator authorization. This
  cleanup does not authorize game launch, installing hooks/drivers,
  forced focus, machine-wide keyboard/mouse injection or Microsoft
  authentication automation.
- Use a suitable approved Windows x64 machine, official licensed Minecraft
  Education clients/accounts and only reviewed original process owners.
  Preserve all account/world/private-profile data; never copy tokens.
- Never use normal `DEV.cmd package` or an old installed `App/` candidate
  as proof of AI input: the regular package **excludes** that experiment.
- Stop on uncertain process identity, mismatched binary, lost MC01
  foreground/control, interrupted native IPC, stuck key release, crash,
  or partial/ambiguous observation. No force-kill, data deletion or
  automatic retry after uncertain delivery. GitHub Actions stays disabled.

## Gate A — exact source, read-only checkout

From a Windows checkout of the repository root, use:

```powershell
git switch Local
git fetch origin Local
git pull --ff-only origin Local
git status --short
git rev-parse HEAD
```

Require `Local`, a **clean** worktree and an exact 40-character HEAD
before interpreting any subsequent command. A new commit invalidates
earlier validation evidence; never silently switch to `main`.

If reviewed portable Node has already been installed in the ignored
`workspace/toolchains/`:

```powershell
.\DEV.cmd inspect --json
.\DEV.cmd catalog --json
.\DEV.cmd plan --json
```

These are inspection/planning only. If pinned portable Node is missing,
do NOT use global Node; `DEV.cmd bootstrap` downloads local toolchains
and requires separate permission. An inspected checkout is not proof of
a source test, build, or game.

## Gate B — standard-source preflight (NOT_EXECUTED)

With explicit authorization for the actual source checks and potential
locked dependency installation, use the existing `DEV.cmd doctor` then
`DEV.cmd preflight` on an unchanged, clean `Local` SHA. Bootstrap first
only if separately authorized and needed. Preflight checks pinned tools
and lockfiles, may run locked `npm ci`, runs existing `verify:source`
and writes metadata only under ignored `workspace/runs/`. Review the
actual result and exit code, not merely command presence.

**A successful standard `preflight`, `check`, `dev` or normal installer
does NOT compile or enable the experimental C++ input hooks.** Do not
upgrade this source-only result to a game-ready claim.

## Gate C — opt-in native build and ABI acceptance (NOT_EXECUTED)

**Separate DLL identity gate:** the standard Lazy Tester hash authorizes a
compatibility DLL compiled WITHOUT the experimental Win32 input hooks.
The feature-enabled Rust controller now selects a distinct
`REVIEWED_EDUCATION_AI_BOOTSTRAP` digest, deliberately set to `None`.
Therefore even a successful standalone experimental C++/Rust compilation
cannot launch an AI game until its exact binary pair and SHA-256 digest
are separately reviewed and pinned in a future source commit. Do not use
the normal compatibility DLL as a substitute, overwrite an installed
account-bearing `App/`, or add a runtime hash/path override. All five
clients under one experimental controller must use the same reviewed
compatibility image; unconfigured human MC01 receives no AI input section.

This is the **next required engineering proof** and needs its own
approval plus real Windows MSVC x64. Keep all binaries, DLLs and test
results tied to the same exact `Local` revision:

1. Check desktop Rust `Development/apps/desktop/src-tauri/Cargo.toml`.
   The explicitly selected experimental
   `multi-client-native-input` Cargo feature is an alias of historical
   `mc02-native-input`; it is NOT in default product builds.
2. Check the existing CMake owner
   `Development/tooling/acceptance/windows/bootstrap-ordering/CMakeLists.txt`:
   it requires `BOOTSTRAP_ORDERING_TEST_ONLY=ON`, reviewed clean Detours
   and an explicitly gated `BOOTSTRAP_MULTI_CLIENT_VIRTUAL_INPUT=ON`.
   `BOOTSTRAP_MC02_RAW_BUFFER_POLL` is separately OFF and must not be
   enabled based on assumptions about Minecraft's input API; the virtual
   input candidate and API-only observer are mutually exclusive.
3. Build and check the exact matched C++/Rust experimental pair, original
   process/channel enrollment, private IPC requests, 96-byte ABI v4,
   per-slot binding, mode admission, lease epoch and ReleaseAll/deadline
   behavior. The default release bundle is not this experimental pair.
4. Execute available own-process compiler/synthetic fixtures only with
   approval; a fixture's presence or authored assertion is NOT a test
   PASS. Inspect actual build identity and ABI receipts before opening
   Minecraft. A test DLL must never silently replace the installed one.

**Stop if exact-source opt-in compilation, matching binary identity or
native admission cannot be demonstrated.** Do not invent another input
backend or injection/packaging route to skip this gate. In addition, the
Tauri/Cargo `build.rs` now explicitly refuses any `release` profile with
`mc02-native-input` or `multi-client-native-input`: this experiment may
be compiled for developer verification only, not included in an ordinary
installer or released as a product.

## Gate D — one human MC01 + one AI MC02 game (NOT_EXECUTED)

Requires Gate C evidence and a **separate** scoped authorization for real
Minecraft game input hooks/runtime. Test only two licensed clients first:

| Area | Required independently observed outcome |
| --- | --- |
| Process identity | Original MC01 and MC02 PID/birth/HWND/session remain distinct and verified |
| Human control | MC01 is genuinely foreground and retains physical keyboard/mouse |
| AI keyboard | MC02 moves without switching Windows foreground |
| Relative mouse | MC02 visibly rotates its own camera |
| Attack/use | MC02's action is observed; animation is not automatically a confirmed hit |
| Neutral safety | Held keys/buttons stop; no continued walking/look/attack after release |
| Failure paths | Wrong identity, process replacement, stale lease, focus loss and uncertain IPC do not retarget games |
| Frames | MC02 image observations actually work and are correctly attributed; PNG change alone is not movement proof |

An IPC ACK, synthetic hook return or process state is **not** proof of
Minecraft movement, key-up, human isolation or multiplayer join. If MC02
does not consume the selected Win32 input API, mark **BLOCKED**, not PASS.

## Gate E — 4–5 clients (DEFERRED)

Only after D passes **movement, relative camera, attack/use, neutral and
uninterrupted human input** in actual game conditions, consider hybrid
MC02–MC05 or separately opted-in five-AI MC01–MC05. Observe all actual
clients in one licensed test world, concurrent input timing, partial
release, process exit/relaunch and image→reasoning→action→image loops.
Normal installer, signing, data preservation and public release are
separate final gates, never implied by this pilot.

## Evidence and clean handoff

For each actually executed gate, retain privately the complete source
SHA, approved target Windows/game version, exact feature flags, binary
hashes, real commands/exits, selected slot and objective observation
(PASS/FAIL/NOT_EXECUTED/UNKNOWN). Do not commit credentials, account
identifiers, world contents, private paths or raw screenshots publicly.

Update only [the single current development handoff](../../planning/development.md)
with resulting status and ONE next action. Detailed older activity is
in [planning history](../../planning/README.md); the
[validation router](README.md) distinguishes retired acceptance reports.
