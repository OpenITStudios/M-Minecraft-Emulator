# Current Development Handoff — M-Minecraft-Emulator

**Repository:** `OpenITStudios/M-Minecraft-Emulator`; development branch `Local`.

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
