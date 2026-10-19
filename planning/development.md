# Current Development Handoff — M-Minecraft-Emulator

**Repository:** `OpenITStudios/M-Minecraft-Emulator`; only GitHub branch
`Local` is authorized for active development. The exact branch HEAD
and current source always outrank historical notes.

## Current product route

- **Active runtime pilot:** official Minecraft Education Client 01 on
  Windows, plus Client 02 through officially installed Sandboxie Plus
  box `MinecraftClient02`. Create Virtual 03/04 and owned-engine
  driver/service activation are blocked.
- **Owned future isolation engine:** one editable Sandboxie-derived
  v1.18.6 tree in `engine/isolation-owned/`, compared against immutable
  `engine/isolation-candidates/sandboxie-v1.18.6/`. Its build owner
  and outputs are documented only in
  [Isolation Source Tooling](../tooling/isolation-source/README.md).
  Older v1.18.5 scripts are reference/test fixtures, **not active fallback**.
- **Single Windows local checkout:** `DEV.cmd bootstrap` → `doctor`
  → `preflight` → `dev`; developer toolchains/downloads, native reports,
  staging and test evidence stay under `workspace/`. Windows system
  services, Minecraft profiles and MSVC/WDK remain OS-managed and are not
  silently relocated.

## Latest independently observed CI evidence

Automatic [Full Source CI run 39239232750](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39239232750)
for exact `Local` commit
`48222b8f3fae0fd19790bbdc003c3ab8351e715b` **PASSED**
all 12 source stages, Windows desktop debug compilation and artifact upload.
Artifact `localtest-debug-48222b8f3fae0fd19790bbdc003c3ab8351e715b`
was present and unexpired at review (scheduled expiry 18 November 2026).
This is **not** proof of Windows UI startup, portable bootstrap execution,
a signed installer, installed driver, Microsoft login or game stability.
Any newer source commit needs its own exact-SHA CI verdict.

## Next single action

**Operator-controlled UI-only smoke on Windows 10/11 x64.** Use the
[CI Debug Candidate](../docs/validation/ci-debug-candidate.md) with a
verified exact-SHA manifest, save and extract under `workspace/`, and
observe window open/render/close. Only afterward, when full source execution
is necessary, use the
[native pilot localtest procedure](../docs/validation/native-pilot-localtest.md)
for the optional bootstrap/doctor/preflight/dev route and two-client
gameplay checks. **Do not install a private driver or migrate accounts.**

## Unresolved (actual target-machine evidence)

Portable developer setup on the user PC, WebView2 UI smoke, Client 02
idle-crash root cause, durable separate-account persistence, multiplayer,
four concurrently playable accounts, and unsigned owned-driver
MSVC/WDK build remain **NOT EXECUTED / UNKNOWN** as appropriate.
Prior operator-confirmed two-account gameplay on 14 October 2026
was followed by an unproven idle crash, not stable acceptance.

Historical CI failures, alternate routes, and former progress notes
are preserved separately under [planning/history](./history/development-through-2026-10-19.md).
Do not carry an old `Next` or `PASS` into the current head.
