# Current Development Handoff — M-Minecraft-Emulator

## Current correction — verified WinEvent unhook outcome

Audit of the current opt-in focus observer found that `UnhookWinEvent`
and abnormal `GetMessageW` errors were ignored, allowing Stop to report
success without having verified hook removal. The sole worker now owns a
`JoinHandle<io::Result<()>>`, returns failure from unhook or message-loop
errors, and both normal Stop and prior-worker reaping propagate the result.
The already-established zero-capacity startup handshake, single-owner
guard and same-thread cleanup remain unchanged. Shutdown logs any unverified
cleanup after the Tauri event loop exits. The public `OBSERVE_ONLY`
IPC/schema, Start/Read/Stop controls, and four-slot registry are unchanged.

The previous `Local` commit
`6b2bd798e9a9ba1236d9cde7417b0ed8405822d8` had passed
all 12 source stages and started Windows debug compilation on
[GitHub run 39290629066](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39290629066)
at last inspection. This correction needs its own exact-SHA Full Source
and Windows debug-build outcome.

**Next:** verify the new exact-SHA Full CI and resolve concrete errors,
if any. The observer has not been exercised in an interactive user
Windows session; Client 01 attribution, owned engine v1.18.6 installation,
Client 03/04 playability, four licensed sessions and safe external
FPS limiting remain `NOT_EXECUTED` / `UNSUPPORTED`.

---


**Repository:** `OpenITStudios/M-Minecraft-Emulator`; development branch `Local`.

## Current bounded correction — no orphaned foreground hooks

The first hook batch remains source-only and opt-in. This change hardens
lifecycle admission: the startup handshake is unbuffered, so a late
worker can never acknowledge startup after the waiting caller times out.
A timed-out worker retains its JoinHandle under the single owner until
it has fully stopped; a second Start cannot install another hook during
cleanup. Stop reports WouldBlock while this cleanup is pending instead
of posting WM_QUIT to an unknown thread ID. A live worker still receives
WM_QUIT and unhooks on its own message-loop thread. The public
OBSERVE_ONLY counter/IPC remains unchanged; no PID, HWND or FPS is stored.

Previous exact `Local` commit
`5a465e8a86206c621cfb2fc636b7184d2400aa24`
[CI 39288641331](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39288641331)
had passed all 12 source stages and entered Windows debug compilation at
last review; its terminal CI status must be checked independently.
The current bounded correction requires its own exact-SHA Full Source CI.

---

## Current bounded batch — opt-in event-driven focus observation

The desktop now contains one **operator-controlled** foreground-event
counter in `apps/desktop/src-tauri/src/engine/instance_focus_events.rs`.
It registers exactly one `EVENT_SYSTEM_FOREGROUND` hook from a dedicated
Windows message-loop thread using `WINEVENT_OUTOFCONTEXT` and
`WINEVENT_SKIPOWNPROCESS`. The thread waits inside `GetMessageW`,
does not poll and unhooks on its installing thread. Start/Stop/Read
controls are under Settings → Diagnostics; `app_bootstrap.rs` requests
shutdown after the Tauri runtime exits. Nothing starts automatically.

The callback counts foreground events and discards all HWND/PID/window
titles, accounts and process data; the existing four-slot catalog and
separate Client 02 signed box/window validation remain the only source
owners for instance identity. The event counter is **not** a client-specific
foreground classification, Minecraft FPS measurement, proof of gameplay
or signal authorizing CPU/GPU changes. Neither a driver nor a frame limiter
has been added. No Minecraft render/simulation/world/login or Windows
priority/sandbox configuration is modified.

## Evidence and proof ceiling

The source hook commit `510945632b827c88eebef13b89b67e4cce449689`
[failed Full Source CI 39286306975](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39286306975)
at its Windows desktop compile stage: three `windows-sys` 0.61
WinEvent constants were incorrectly imported from `Win32::UI::Accessibility`.
They belong to `Win32::UI::WindowsAndMessaging`, while
`SetWinEventHook` and `UnhookWinEvent` remain under Accessibility.
Other eleven source stages passed; no debug desktop artifact was produced.

This bounded fix changes only the Rust imports and adds a source contract
test for the actual `windows-sys` module owners. It does not weaken hooks,
admission, cleanup or Minecraft isolation.
The preceding `b71384e98bcf3414b72b18a2863667e8a6f59d8e`
[passed Full Source CI 39283268843](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39283268843).

**The new exact-SHA Full Source / Windows compile result is pending.**
The Windows hook has not been observed operating on the user's PC.
Client 01 native attribution, owned v1.18.6 driver/service activation,
Client 03/04 provisioning, four independent licensed Minecraft Education
sessions and safe external-only background FPS limiter remain
`NOT_EXECUTED` or `UNSUPPORTED`. No 15–20 FPS or RAM savings claim.

## Next action

Check the new exact-SHA Windows Full Source CI and repair proven failures,
if any. Then review provider-specific attribution for Client 01 and
v1.18.6 sandbox slots before connecting focus events to *any*
reversible performance action.

History: [planning/history](./history/development-through-2026-10-19.md).
