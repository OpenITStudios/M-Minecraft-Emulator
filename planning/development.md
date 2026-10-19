# Current Development Handoff — M-Minecraft-Emulator

**Repository:** `OpenITStudios/M-Minecraft-Emulator`; active branch `Local`.

## Current source batch — on-demand external focus diagnostic

A single read-only Client 02 foreground snapshot reuses the existing
official Sandboxie and Microsoft-signed game PID/FILETIME/HWND/window-owner
verification. The new `instance_focus.rs` calls Windows
`GetForegroundWindow()` **only after the operator clicks**
Settings → Diagnostics → Check focused Client 02; no event hook, thread
or polling loop is introduced. Foreground `NULL` is `UNKNOWN`, never a
reason to throttle Minecraft. An available result is momentary only.
Client 01 remains unattributed; 03/04 retain no verified provider.

The mounted command uses the same Rust admission lock and a strict
TypeScript response. It does not write Minecraft settings, alter
process priority, install Sandboxie drivers, change accounts or enable
an FPS limiter. Backend file/image/box checks are repeated despite a
prior UI candidate snapshot.

## Established implementation boundaries

- Four stable slot identities and operation admission owner:
  `apps/desktop/src-tauri/src/engine/instance_catalog.rs`.
- One manual four-slot discovery route, independent Client 01/02 read-only
  evidence, shared PID/FILETIME/HWND parser, official Client 02 Focus/Place.
- One editable Sandboxie-derived v1.18.6 source
  (`engine/isolation-owned/`), immutable v1.18.6 reference
  (`engine/isolation-candidates/sandboxie-v1.18.6/`).
  Owned driver/source remains NOT_INSTALLABLE and cannot coexist with
  installed official Sandboxie in the same namespace.
- Local developer artifacts and toolchain controlled under `workspace/`,
  subject to actual OS-level Git, MSVC/WDK, Windows and Minecraft requirements.

## Verification and next action

Exact preceding commit `ddb26ae2bbff03f22cb07c5549bd9edcc91464a1`
[Full Source CI 39268545301](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39268545301)
**PASS** all 12 source stages and Windows debug executable upload.
The new focus diagnostic requires its own exact-SHA CI result; no actual
Windows focus/gameplay measurement has been run.

**Next:** verify Full Source and Windows debug build for the focus diagnostic,
fix only proven regression, then evaluate how to attribute Client 01 and
future owned 03/04 windows without guessing sandbox membership.
The proposed 15–20 FPS external-only background cap is still
`UNSUPPORTED/NOT_EXECUTED` pending vendor/OS capability verification and
frame-time measurement. Never change render/simulation distance,
Minecraft graphics settings, user accounts or running Sandboxie drivers.

Historical research remains under [planning/history](./history/development-through-2026-10-19.md).
