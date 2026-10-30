# Optional persistent GPU stream transport — source only

`gpu-frame-source.mjs` consumes bounded, binary `LTG1` PNG frames produced
by the isolated `native-gpu-capture` Rust worker. It is the **same existing
Lazy Tester MCP process** that owns at most one worker per profile and at
most one recent complete frame in RAM per worker.

Each worker must be explicitly configured and started only after native
controller verification. It cannot launch/close Minecraft, receive
keyboard/mouse actions, or change game focus. The decoder refuses corrupt,
oversized, out-of-order, mismatched PNG and stale timestamp packets. Frames
older than 3.5 seconds are never presented as current. A stalled stream,
early exit, or deadline kills only the capture worker, never Minecraft.

The capture exe must have the exact name
`lazy-tester-gpu-capture.exe` and an absolute trusted local path. It is
**not currently part of the product or installer**; there is no approved
binary build yet. Cua Driver remains the existing optional snapshot
provider. This transport becomes selectable through the established MCP
observation tools only after the separate source integration is completed.

See `native-gpu-capture/README.md` for the upstream WGC crate,
frame protocol, licensing and unverified platform limitations.

**All executable/source tests and Windows Minecraft acceptance are
NOT_EXECUTED** by explicit operator request. The Node fixture file is for
the eventual approved test phase only.
