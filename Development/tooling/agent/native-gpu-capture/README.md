# Windows Graphics Capture worker — source-only candidate

This is a **standalone, optional frame producer** using
[Atliac/wgc 3.1.0](https://github.com/Atliac/wgc) (MIT/Apache-2.0),
which wraps Microsoft's `Windows.Graphics.Capture` and persistent
`Direct3D11CaptureFramePool`.

**Status: authored, NOT BUILT, NOT TESTED, NOT INSTALLED.** This independent
Cargo manifest is intentionally NOT included in Tauri, DEV.cmd, the product
installer, or the original five-session controller. It has no generated
Cargo.lock yet because the operator explicitly deferred all builds/toolchain
execution; a dependency-locked build is required before runtime acceptance.
Do not distribute this executable as part of Lazy Tester until that gate passes.

## Exact scope

One process captures **one pre-authorized native Minecraft window**.
The existing MCP chooses and verifies the target through the retained
controller. The producer independently pins the process and checks:
PID, process creation FILETIME, Windows session, visible responsive
top-level HWND, before every emitted frame. It never touches Minecraft
memory, DLLs, gameplay, login, network, keyboard, mouse or foreground focus.

On a verified Windows host, after source/binary review and explicit operator
approval, this candidate would be invoked as:

```text
lazy-tester-gpu-capture.exe
  --pid <verified-pid> --window-id <verified-HWND-as-number>
  --birth <verified-FILETIME> --session <verified-session>
  --duration-ms <5000..90000> --fps <1..10>
```

The capture session owns one WGC window item. No interactive picker,
monitor capture, full-desktop capture or file recording. The legacy LTG1 observation mode remains unchanged.

Frames are fitted with letterboxing to fixed 960×540 RGBA, encoded into
PNG and emitted as a binary stdout protocol. The worker reuses one PNG output
buffer for its lifetime and writes the 28-byte header from a fixed array,
rather than allocating both on every frame (P51, source-only):

```text
LTG1 | pngLength:u32be | width:u32be | height:u32be
     | sequence:u32be | UnixCaptureMs:u64be | bounded PNG bytes
```

Maximum PNG is 3 MiB, each worker has a duration limit of 90 seconds,
and emitted rate is capped at 10 frames per second. The *parent* must also
enforce a wall-clock kill: a WGC iterator may block indefinitely when
Minecraft renders no new frame. A captured frame is **not proof of world
membership, player movement, freshness at AI-consumption time, or PvP
response latency**.

The Node MCP process is responsible for explicit admission, per-slot worker
ownership and termination, discarding stale/replaced frames and forwarding
only selected frames to Codex. No callback into a game process is required.
Windows may show a native capture border. WGC may return a black/stale
surface or refuse certain protected/minimized windows. Runtime behavior
for Minecraft Education 26.32/26.33 remains UNKNOWN.

## Fitted AI WebP preview / observation (source-only)

The same WGC producer and existing image-webp encoder expose two explicit
modes, with bounded fitted width/height in 160–1536 (default selected by MCP
640x360). No PNG intermediate, no second capture library or worker owner.

```text
lazy-tester-gpu-capture.exe --preview --pid <pid> --window-id <hwnd> --birth <birth> --session <session> --width 640 --height 360
lazy-tester-gpu-capture.exe --webp --pid <pid> --window-id <hwnd> --birth <birth> --session <session> --duration-ms 5000 --fps 2 --width 320 --height 180
```

Both use the same 28-byte packet header with magic LTG2, actual fitted
canvas dimensions, sequence, and producer readback timestamp; payload is
one RIFF VP8L WebP up to 3 MiB. The parent selects the exact expected format
and size; the one-shot parent requires exactly one complete packet and a
successful bounded exit, and refuses an already-active sampler. Fit preserves
aspect via the existing WGC fitting path; no promise of native pixel fidelity
or coordinate mapping. WebP source readiness is not binary approval. No
automatic fallback to LTG1 or CUA. Legacy CLI flags/PNG remain unchanged.

## Explicit one-shot product screenshot candidate (CAP-01)

A separate strict mode preserves the existing observation arguments and LTG1 protocol:

```text
lazy-tester-gpu-capture.exe --screenshot --pid <verified-pid> --window-id <verified-HWND-as-number> --birth <verified-FILETIME> --session <verified-session>
```

Output is exactly one bounded binary WebP (VP8L), not LTG1 or Base64, with
success only after stdout completes. The worker accepts no file paths.
The future product parent must admit a controller-verified slot, supervise
the worker with an independent wall-clock timeout, validate the entire output
and exit status, and publish only a complete file; it must not accept an
arbitrary executable or PID from UI. The source-only product command, UI, bounded parent supervisor and no-overwrite publisher are now connected (CAP-02). The product admission digest remains None; UI stays unavailable until exact binary review. The worker is not built or included in the installer.

The one-shot path disables cursor capture, reads the original RGBA8 frame,
checks DWM physical bounds against WGC size, crops the verified client
rectangle without resizing (in-place row compaction), and uses pure-Rust
`image-webp = 0.2.4` directly, without intermediate PNG. It refuses mismatched
or changed geometry, nonopaque premultiplied pixels, invalid length, and
dimensions above 4096 on either side. Raw and encoded limits are 64 and
80 MiB respectively; these bound the payload, not total codec/GPU memory.
See [WGC 3.1.0 Frame](https://docs.rs/wgc/3.1.0/wgc/frame/struct.Frame.html)
and [image-webp 0.2.4](https://docs.rs/image-webp/0.2.4/image_webp/struct.WebPEncoder.html).
Original process handle liveness is rechecked after window identity observation. Native DWM/WGC mapping, DPI, opacity, deadline and actual file size are
unverified. No GPU or CPU recording encoder has been selected or added.

No tests or builds may be run until the operator explicitly authorizes them.
