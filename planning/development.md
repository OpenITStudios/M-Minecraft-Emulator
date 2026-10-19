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

Prior commit `b71384e98bcf3414b72b18a2863667e8a6f59d8e` entered
[Full Source CI 39283268843](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39283268843),
which was **queued** at the last inspection. Do not promote that run
or the new batch into PASS before observing the exact outcome.
The preceding `6b09f52e18300742b09946eea2a276441ee2a484`
passed all 12 source stages and Windows debug build/upload
([CI 39278677898](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39278677898)).

**New commit requires fresh exact-SHA full Windows source/test/compile.**
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
