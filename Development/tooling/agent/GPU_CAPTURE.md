# Optional persistent GPU stream transport — source only

`gpu-frame-source.mjs` consumes bounded, binary `LTG1` PNG (legacy) or
explicit `LTG2` WebP previews produced
by the isolated `native-gpu-capture` Rust worker. It is the **same existing
Lazy Tester MCP process** that owns at most one worker per profile and at
most one recent complete frame in RAM per worker.

Each worker must be explicitly configured and started only after native
controller verification. It cannot launch/close Minecraft, receive
keyboard/mouse actions, or change game focus. The decoder refuses corrupt,
oversized, out-of-order, mismatched codec/dimensions and stale timestamp packets. Frames
older than 3.5 seconds are never presented as current. A stalled stream,
early exit, or deadline kills only the capture worker, never Minecraft.

The capture exe must have the exact name
`lazy-tester-gpu-capture.exe` and an absolute trusted local path. It is
**not currently part of the product or installer**; there is no approved
binary build yet. Cua Driver remains the existing optional snapshot
provider. This transport is connected to the existing MCP
`start_visual_observation` / `read_visual_observation` /
`stop_visual_observation` / `observe_client` code when explicitly configured
with `--gpu-capture-exe`. It is not packaged or runtime validated.

Playtest requests LTG2 WebP with 640x360 default or explicit fitted dimensions. One-shot preview uses the same worker without a running sampler; one worker/reservation per slot. Legacy LTG1 and the separate original-resolution product screenshot contract remain unchanged. The product parent is source-integrated with a closed reviewed-binary gate; no installer or native validation is available yet.

See `native-gpu-capture/README.md` for the upstream WGC crate,
frame protocol, licensing and unverified platform limitations.

**All executable/source tests and Windows Minecraft acceptance are
NOT_EXECUTED** by explicit operator request. The Node fixture file is for
the eventual approved test phase only.
