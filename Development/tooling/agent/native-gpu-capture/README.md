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
monitor capture, full-desktop capture or file recording.

Frames are fitted with letterboxing to fixed 960×540 RGBA, encoded into
PNG and emitted as a binary stdout protocol:

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

No tests or builds may be run until the operator explicitly authorizes them.
