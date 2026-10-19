# Current Development Handoff — M-Minecraft-Emulator

**Repository:** `OpenITStudios/M-Minecraft-Emulator`. Active GitHub branch: `Local`.

## Current change: avoid manager-focus contamination

The existing four-slot focus diagnostic stays one-shot and read-only.
Client 02 still requires a fresh signed Education executable check and
official Sandboxie PID/creation FILETIME/HWND/root-owner verification.
The Windows foreground HWND and its owning PID are read, then foreground
is read again. The result is `UNKNOWN` when the foreground belongs to
MIVUBI's own process, Windows returns NULL, the PID lookup fails, or the
foreground changes across the observations. An unexpected PID owning a
reused matching HWND also becomes `UNKNOWN`.

This corrects a real usability/safety ambiguity: clicking the Diagnostics
button usually foregrounds the *manager*, so an earlier `NOT_FOREGROUND`
snapshot could be mistaken for a safe background-FPS signal. Current
`NOT_FOREGROUND` now requires another **non-MIVUBI** window with a stable
identifiable owner; it remains an instant-only observation and cannot
authorize any FPS cap. The canonical slot 01 remains `UNATTRIBUTED`;
03/04 remain `PROVIDER_NOT_AVAILABLE`. Schema-2 OBSERVE_ONLY IPC is
unchanged, and no extra watcher, service, provider or thread is added.

## Verification

Preceding `Local` commit
`6b09f52e18300742b09946eea2a276441ee2a484` has already passed
all twelve source stages, Windows debug compilation and artifact upload
on [GitHub run 39278677898](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39278677898).
The workflow had not yet reported its terminal state when inspected.
This source change must have its own exact-SHA CI result; it has not
been executed on the user's Windows PC.

## Unresolved and next action

Client 01 origin attribution cannot be derived from name matches or a
nonzero result of the official Sandboxie DLL query. That API's error
values also cover exit/permission/unsandboxed cases, so they are not
proof of native ownership. The owned v1.18.6 Windows service/driver,
Client 03/04 provisioning, four independent licensed active sessions,
gameplay, actual Windows focus performance and 15–20 FPS external-only
limitation remain `NOT_EXECUTED` / `UNSUPPORTED`.

**Next:** verify Full Source and debug Windows compilation at this exact
SHA, then design an opt-in event-driven Windows foreground observer
(`SetWinEventHook` requires a message loop) only after reliable
per-slot process attribution; continue to block any actual FPS mutation.

No Minecraft settings, render/simulation distance, game executable,
accounts, official Sandboxie driver or external system services changed.

History: [planning/history](./history/development-through-2026-10-19.md).
