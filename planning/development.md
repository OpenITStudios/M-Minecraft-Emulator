# Current Development Handoff — M-Minecraft-Emulator

**Repository:** `OpenITStudios/M-Minecraft-Emulator`. Active development branch: `Local`.

## Current batch: one four-slot focus snapshot

`apps/desktop/src-tauri/src/engine/instance_focus.rs` provides one
operator-requested Windows foreground snapshot over the four IDs from the
canonical `instance_catalog.rs` source. Client 02 remains the **only**
attributable slot: the existing fresh official Sandboxie membership,
Microsoft-signed Education image, PID/FILETIME/HWND and live window-owner
checks run before the snapshot. A NULL foreground produces `UNKNOWN`.

Client 01 is `UNATTRIBUTED`; Client 03/04 are `PROVIDER_NOT_AVAILABLE`.
A different foreground window yields a momentary `NOT_FOREGROUND` for
Client 02, **not** authorization to throttle that process.

The old one-client focus IPC is replaced with one strict schema-2
`OBSERVE_ONLY` four-slot IPC, exposed through the existing manual
Diagnostics control. No parallel observer, background timer, event hook,
FPS limiter, priority adjustment, installed driver change, Minecraft
settings change or user-account access is added.

## Proof boundary / next step

Previous exact `Local` commit
`9d5c46c0e8fbea815df17aba127e8954cb8bda6a` had
[all 12 Full Source CI stages and Windows debug upload PASS](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39276042784).
**The current change needs its own exact-SHA CI verdict.**

Native Client 01 identity attribution, MIVUBI-owned v1.18.6 driver
activation, Client 03/04 provisioning, four simultaneously licensed
Minecraft Education sessions, real Windows focus measurements and external
background FPS limiting remain `NOT_EXECUTED` / `UNSUPPORTED`.
Never modify render/simulation distance, internal Minecraft settings,
credentials or official Sandboxie services.

**Next:** verify new Full Source Windows and debug-compile evidence; fix
only demonstrated regressions. Then investigate independent native Client 01
identity attribution without guessing process origin or box membership.

Historical notes: [planning/history](./history/development-through-2026-10-19.md).
