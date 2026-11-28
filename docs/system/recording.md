# Product recording — source candidate

CAP-03/04/05 initial source checkpoint; both binary admission gates remain closed. The retained game controller owns process
and profile lifetimes. DWM thumbnails and LTG1 AI observation stay unchanged.

## Backend decision
FFmpeg 8.1 introduced the WGC gfxcapture source with explicit HWND, client-only
capture and D3D11 output. NVENC accepts D3D11 hardware frames. The Matroska
muxer handles HEVC codec private, timestamps and trailer without implementing
these formats again in Rust. Review references:
- https://www.ffmpeg.org/doxygen/8.1/vsrc__gfxcapture_8c_source.html
- https://ffmpeg.org/ffmpeg-codecs.html (NVENC, hardware inputs, CQ)
- https://ffmpeg.org/ffmpeg-protocols.html (fd seekable regular file)
- https://www.matroska.org/technical/codec_specs.html (V_MPEGH/ISO/HEVC)
- https://ffmpeg.org/legal.html (distribution obligations)
- https://developer.nvidia.com/video-encode-decode-support-matrix (RTX 3070: HEVC encode supported, AV1 encode unsupported)
- https://ffmpeg.org/doxygen/8.1/nvenc__hevc_8c_source.html (HEVC profile and RGB input options)
- https://ffmpeg.org/doxygen/8.1/nvenc_8c_source.html (Direct3D input registration)
- https://docs.nvidia.com/cuda/cuda-c-best-practices-guide/index.html (minimize host/device transfers)

The candidate explicitly selects NVIDIA D3D11 and hevc_nvenc, Main profile, packed RGB to YUV 4:2:0,
P4, CQ 23,
no B-frame or lookahead queue, VFR capped at 30 FPS, original even client
size, no audio or cursor. No hwdownload, raw frame IPC, automatic conversion,
or encoder fallback. This is not a claim of zero GPU copies, 30 CFR,
pixel-identical video or measured quality. The operator's RTX 3070 compatibility direction supersedes the previous
AV1-only codec decision; retain exactly one encoder, HEVC NVENC. AV1,
H.264, AMF, QSV, encoder selection and startup retry remain excluded.
No application-owned CUDA context, toolkit, kernels or extra GPU scheduler:
D3D11 hardware frames can feed NVENC directly, while screenshot encoding
and small controller work stay on CPU. NVENC is distinct from CUDA cores.
CUDA compute remains a measured-bottleneck option, not a dependency. One child per recording request;
if unsupported or initialization fails, report failure without retrying.
Recording requires a compatible NVIDIA HEVC encoder; screenshot remains
independent.

One adjacent self-contained ffmpeg.exe must be reviewed and pinned by
compiled SHA-256 (currently None); no PATH search, user executable choice,
download, packaging, or dependency resolution. A later review must pin
upstream release/source, build configuration (including gfxcapture, d3d11va,
hevc_nvenc, fd, pipe and matroska), dependency and driver compatibility,
corresponding source/license notices, redistribution eligibility (including applicable HEVC patent/licensing obligations), actual
binary size, and absence of unreviewed DLL search/dependencies. The default
installer and executable identity are unchanged.

## Lifecycle and limits
One explicit start per selected controller-verified slot, with native save
dialog and exclusive temporary file. The recorder has no output path in its
command arguments: stdout is the manager-owned file handle, fd protocol
allows muxer seeks. FFmpeg progress is read from a bounded pipe and only
frame counts/state are exposed. Only success exit, final progress=end,
positive frame count, EBML header and bounded size permit no-clobber publish.
If final publication fails after these checks and sync (for example, a file
appears at the destination during recording), the completed temporary file
is retained with its existing unique .part name, when the filesystem permits.
The terminal Failed receipt reports only that generated filename in the chosen
save folder; it never reports Saved, overwrites, retries, or auto-renames.
A failed keep operation is unconfirmed; automatic cleanup is disabled for
that entry, but recovery is not promised. Earlier capture/identity/encoder
failures do not use this retention path. These are structural/producer
receipts, not independent decode validation.

Up to five logical slots; initial aggregate pixel throughput is capped at
4096×2160×30 pixels/s. This is a policy candidate, not measured hardware
capacity. One hour and 4 GiB nominal file ceiling per recording (allow up
to 1 MiB muxer overshoot). No auto-start, no idle recording poll. The
independent per-recording supervisor verifies creation time and Windows
session once against its retained process handle, then checks handle
liveness before and after root HWND owner/thread, visibility, responsiveness
and unchanged client dimensions every 200 ms. Screenshot and recording
share the original controller slot admission; neither depends on the other.
The separate bounded pipe reader avoids blocking the supervisor's deadlines.
Malformed, disconnected or prematurely ended progress now stops the child
instead of waiting for the normal duration limit. Loss/resize fails closed and
discards the temporary output instead of publishing ambiguous footage;
no attempt to reacquire a replacement HWND. This periodic check is not
per-frame identity proof; exact WGC capture-item lifetime must be verified.
Stop sends q to the child stdin only, then waits for graceful finalization
up to 10 s. A BrokenPipe during concurrent natural exit still requires
successful exit and complete progress; it is not an automatic success or
retry. A final progress report without child exit also has a 10 s deadline.
The same 15 s frame-progress deadline covers both startup and mid-stream
stalls: only a completed, monotonic frame-count report resets it. A game
that produces no new frames for 15 s is conservatively failed, including
a static/stalled capture source; this is not proof of game failure. The
progress parser rejects missing/duplicate frame counts, unknown terminal
markers and data after progress=end. Child/job cleanup on failure; process crash can leave a .part
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
unsupported hardware fails with one attempt and no alternate encoder;
D3D11 frames reach NVENC without CPU readback, decode the completed MKV and independently verify HEVC Main video,
4:2:0 output, dimensions and playback compatibility,
test stop/start, immediate stop, first/later frame stalls, malformed progress,
terminal report without child exit, natural exit racing Stop, target
exit/resize, no-clobber, disk full,
GPU exhaustion, failure cleanup, crash recovery, job containment, mixed DPI,
background/occluded games, and one-to-five workloads. Measure CPU/GPU/RAM,
game FPS, actual video FPS, bitrate, small text and texture detail. Review CAP-05 typed UI controls at supported widths/DPI, independently
stop any slot, verify close refusal and failure/terminal status.
All native/build/fixture execution and release remain held.
