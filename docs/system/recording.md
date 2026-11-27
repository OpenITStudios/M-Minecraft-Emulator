# Product recording — source candidate

CAP-03/04/05 initial source checkpoint; both binary admission gates remain closed. The retained game controller owns process
and profile lifetimes. DWM thumbnails and LTG1 AI observation stay unchanged.

## Backend decision
FFmpeg 8.1 introduced the WGC gfxcapture source with explicit HWND, client-only
capture and D3D11 output. NVENC accepts D3D11 hardware frames. The Matroska
muxer handles AV1 codec private, timestamps and trailer without implementing
these formats again in Rust. Review references:
- https://www.ffmpeg.org/doxygen/8.1/vsrc__gfxcapture_8c_source.html
- https://ffmpeg.org/ffmpeg-codecs.html (NVENC, hardware inputs, CQ)
- https://ffmpeg.org/ffmpeg-protocols.html (fd seekable regular file)
- https://www.matroska.org/technical/codec_specs.html (V_AV1)
- https://ffmpeg.org/legal.html (distribution obligations)

The fixed hardware candidates are attempted in this order:

| Candidate | GPU frame path | Quality default |
|---|---|---|
| AV1 NVENC | NVIDIA D3D11 → AV1 | P4 / CQ 23 |
| H.264 NVENC | NVIDIA D3D11 → H.264 | P4 / CQ 23 |
| H.264 AMF | AMD D3D11 → H.264 | Balanced / CQP 23 |
| H.264 QSV | Intel D3D11 → GPU NV12 → direct QSV mapping → H.264 | Medium / global quality 23 |

Source references: [AMF D3D11 input](https://ffmpeg.org/doxygen/8.1/amfenc_8c_source.html),
[AMF quality options](https://ffmpeg.org/doxygen/8.1/amfenc__h264_8c_source.html),
[scale_d3d11](https://ffmpeg.org/doxygen/8.1/vf__scale__d3d11_8c_source.html),
[QSV mapping](https://ffmpeg.org/doxygen/8.1/hwcontext__qsv_8c_source.html),
[H.264 QSV](https://ffmpeg.org/doxygen/8.1/qsvenc__h264_8c_source.html).
This is a source candidate matrix, not a certified GPU compatibility list.
A nonzero child exit is eligible for the next candidate only if progress
drainage completed without a terminal marker, no frames or output bytes were
produced, original target is still valid and Stop was not requested. All
attempts share a 15-second startup deadline (plus bounded cleanup); no
retries after output, timeout, malformed progress, target change or
ambiguous cleanup. The exit alone does not diagnose unsupported hardware.
Actual codec is exposed in the typed receipt only after a positive frame
count. No separate probe process or idle hardware polling.

All candidates preserve the VFR cap of 30, even original client dimensions,
no B-frames, no audio or cursor. No hwdownload, raw frame IPC, automatic
conversion or software fallback. GPU-to-GPU conversion is explicit for
QSV; zero-copy operation and matched quality across encoders are not
proven. An incompatible direct map fails closed.

One adjacent self-contained ffmpeg.exe must be reviewed and pinned by
compiled SHA-256 (currently None); no PATH search, user executable choice,
download, packaging, or dependency resolution. A later review must pin
upstream release/source, build configuration (including gfxcapture, d3d11va,
av1_nvenc, h264_nvenc, h264_amf, h264_qsv, scale_d3d11, hwmap,
format, fd, pipe and matroska), dependency and driver compatibility,
corresponding source/license notices, redistribution eligibility, actual
binary size, and absence of unreviewed DLL search/dependencies. The default
installer and executable identity are unchanged.

## Lifecycle and limits
One explicit start per selected controller-verified slot, with native save
dialog and exclusive temporary file. The recorder has no output path in its
command arguments: stdout is the manager-owned file handle, fd protocol
allows muxer seeks. FFmpeg progress is read from a bounded pipe and only
frame counts/state and observed codec are exposed. Only success exit, final progress=end,
positive frame count, EBML header and bounded size permit no-clobber publish.
These are structural/producer receipts, not independent decode validation.

Up to five logical slots; initial aggregate pixel throughput is capped at
4096×2160×30 pixels/s. This is a policy candidate, not measured hardware
capacity. One hour and 4 GiB nominal file ceiling per recording (allow up
to 1 MiB muxer overshoot). No auto-start, no idle recording poll. The
independent per-recording thread checks original process creation time,
Windows session, root HWND owner/thread, visibility, responsiveness and
unchanged client dimensions every 200 ms. Loss/resize fails closed and
discards the temporary output instead of publishing ambiguous footage;
no attempt to reacquire a replacement HWND. This periodic check is not
per-frame identity proof; exact WGC capture-item lifetime must be verified.
Stop sends q to the child stdin only, then waits for graceful finalization
up to 10 s. Child/job cleanup on failure; process crash can leave a .part
file, which is not marked Saved or auto-deleted by storage maintenance.
Immediate job assignment has a small spawn-to-assignment race; binary
approval requires review of this boundary. No game process is in the job.
Desktop close commands refuse active recordings before game/workspace
mutation; external game exit invalidates the worker. The gated topbar
slot selector projects only bounded native status receipts and Record/Stop
commands. Stop is independent of workspace selection and available from
Settings; start is limited to Play with a responding profile. The existing
2 s observer reads recording status only while active or uncertain. Unknown
status never authorizes another Start; explicit per-slot Stop remains
available while a recording may be active. Close
controls for a known active slot are disabled; native CloseOne/CloseAll
still enforce admission. A switch or layout that changes target client
dimensions can invalidate recording; this is not proof of uninterrupted
recording across selection changes. No change to account,
world, cache or diagnostic data.

## Acceptance — NOT_EXECUTED
Review exact FFmpeg version/options and native filter negotiation; verify
D3D11 frames reach each available hardware encoder without CPU readback,
verify each fallback boundary and codec receipt, decode the completed MKV,
test stop/start, immediate stop, target exit/resize, no-clobber, disk full,
GPU exhaustion, failure cleanup, crash recovery, job containment, mixed DPI,
background/occluded games, and one-to-five workloads. Measure CPU/GPU/RAM,
game FPS, actual video FPS, bitrate, small text and texture detail. Review CAP-05 typed UI controls at supported widths/DPI, independently
stop any slot, verify close refusal and failure/terminal status.
All native/build/fixture execution and release remain held.
