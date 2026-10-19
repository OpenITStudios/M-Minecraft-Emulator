# Current Development Handoff — M-Minecraft-Emulator

## Shared four-slot identity/admission — 19 October 2026

**Scope:** one fail-closed slot operation matrix in the existing Rust
`instance_catalog.rs`, enforced by the current Tauri commands before
spawning expensive helpers; a small reusable PID/FILETIME/HWND parser
in `instance_identity.rs` replaces duplicate parsing inside the existing
Client 02 fresh-window verification. The signed image, box membership and
live HWND ownership checks are unchanged.

**Supported operations:** Client 01 candidate inspection/official launch;
Client 02 official Sandboxie inspection/selection/launch/focus/place.
Client 03/04 launch, provisioning, process mutation, and external FPS
control all remain explicitly blocked. No Minecraft setting, account or
driver/service change; no new monitoring loop, IPC alias or provider.

**Evidence:** prior `87fc898ced54f1be8a0a8d75a8d30a2f26b2bf42`
passed all 12 Full Source Windows stages and debug compilation/upload,
[run 39258261515](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39258261515).
This source revision requires its own exact-SHA CI. Next task after
verification: build read-only, provider-specific discovery across slots
without inventing or enabling a v1.18.6 driver.

---


## Four-client foundation — 19 October 2026

**Scope:** one read-only catalog of four slot identities in the existing
desktop Rust engine, exposed via a single lightweight Tauri IPC and strictly
validated in the mounted Svelte UI. Client 01/02 retain their existing
official Windows/Sandboxie pilot launch implementations. Client 03/04
are rendered from the same backend catalog as `NOT_AVAILABLE`, not started
or provisioned, and cannot be misinterpreted as ready.

**Constraint:** this is the first bounded source change toward a four-slot
architecture, **not** completed four-client lifecycle or four-account
acceptance. No private v1.18.6 driver, FPS limiter or outside-game
resource governor is enabled; no Minecraft settings or account data change.
Future external focus/FPS work needs a real supported vendor/API capability
and Windows measurement before applying policy.

**Proof:** prior Local `260ef64a3175a043b43503c18666395c817c4a95`
passed automatic Full CI run
[39250836579](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39250836579).
Current change requires its own Full CI verification and subsequent
Windows/UI observation. Next source unit: generalize the existing process
identity and lifecycle admission owner from Client 02 to slots 02–04
without enabling an unproven provider.

---


## Correct documented authority regression — 19 October 2026

**CI evidence:** Full Source [run 39247152326](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39247152326) on `480891c27419a4715c45d8f1e9117b32b7673a72` FAILED two source-verification stages (repository and isolation), although desktop, runtime and other source stages passed. The repository contract expected the new `engine/README.md` but that file had not been updated in the prior commit. The v1.18.5 baseline test still sought its historical command examples in the now-active v1.18.6 README instead of the archive.

**Bounded correction:** route `engine/README.md` to the sole editable v1.18.6 owner and preserve its immutable reference/legacy distinctions. Give each router a specific static invariant and rebind the v1.18.5 historical example regression to `planning/history/isolation-v1185-source-notes.md`; additionally refuse old baseline commands in the active README. No service, installer, provider, code of the active UI or native engine was touched. CI must pass on the newer SHA before this cleanup is considered verified.

**Next:** inspect automatic Full Source CI on the corrected exact SHA, then attempt only a checksum-verified UI smoke on the user's Windows PC.

---


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
