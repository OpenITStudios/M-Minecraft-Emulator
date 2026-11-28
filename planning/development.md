# Current Development Handoff — Lazy Tester

## Current status

**Source integration closed — Education update compatibility (UPD, scoped SOURCE_READY_FOR_LOCAL).** Baseline
[`41e7eadd`](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/41e7eadde35bd72f215f0da983136a2817750c65).
First slice: official Desktop updater state guard in the existing post-pin
registered resolver; one query per managed launch, no task changes, updater
execution, downloader, polling, new service or dependency. Preserve the
existing signature/image/ABI checks; extract reviewed image recipe data
without admitting new bytes. Compare candidate identity against all retained
active child image pins before profile mutation, and clear only stale
unsupported-image issues on verified compatible admission. Authored synthetic
generation fixture NOT_EXECUTED. Updater state is a point-in-time observation,
not an installer lock; missing/disabled tasks may be managed policy.
Second slice (baseline [ea12355](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/ea12355be1e20f04b57e4e733d219438da7d3338)):
explicit Settings entry/header Refresh reads bounded registry version,
task state and exact reviewed-image match from existing registered resolver
and post-pin checks; hashes once per explicit observation and releases pin.
Normal session wire, passive polling and startup remain unchanged. One
read-only Tauri route uses the existing manager gate; no raw path/hash in
receipt, no launch permission, no task execution. TS contract fixtures
authored NOT_EXECUTED.
Third slice (baseline [2671af6](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/2671af695d0819aba967862766b51c21f0f7e5a3)):
operator UX feedback: one version row only when healthy; one authored
notice for actionable exceptions, never technical image-review or task
state rows. Installation check on Settings entry precedes session settings
so a blocked native payload does not suppress independent observation.
No new requests, timer, backend authority or schema change. UI fixtures
authored NOT_EXECUTED. Capability review: existing default-OFF native
F3, experimental AI and heap experiments are compile-time isolated;
ordinary and AI DLL admission are separately digest-pinned, with no
verified new game ABI. Preserve these existing boundaries rather than
add a duplicate capability manager or enable unsupported optional features.
Fourth slice (baseline [8d9b262](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/8d9b262ed1e2c953cfab0f529192107f00702274)):
official Education support confirms the Desktop task checks at sign-in and
opening Minecraft can cancel an update. Keep official auto-updates; direct
task invocation held because task name/Ready alone cannot prove publisher,
permissions or installer exclusion against active/unmanaged game launches.
Do not add a button, elevated helper or updater. Existing exception notice
explains leaving games closed. Reduce managed launch hashing: exact
continuous child image pins and matching reviewed recipes permit reuse within
the active cohort; no idle cache, new pin, service or dependency. All launches
still freshly validate signed registration, updater state, image identity and
native DLLs. New or conflicting image generation refused; last exit requires
full hash again. Synthetic fixture authored NOT_EXECUTED.
Final integration slice (baseline [8c12940](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/8c12940976cada6326dc1dae0eda5282001dec5a)):
launch checks updater before/after pin using the two existing resolver
processes. Explicit Settings observes updater first and skips pin/hash
when already busy; if busy after pin it skips the hash and releases the
pin. Nullable image review is valid only during Busy, not a positive
compatibility receipt. Manual Refresh pauses competing UI/background
observations and checks installation independently before session settings.
No new polling, process count, dependency, service, updater action or
native admission. Core and direct consumers reviewed; fixtures authored
NOT_EXECUTED. Official Windows scheduler state values and Desktop task
name checked against Microsoft/Minecraft documentation. Actual scheduler
visibility, task provenance, pin/installer coordination (including the
existing temporary controller-preparation pin), measured overhead,
future exact recipe/ABI review and end-to-end update acceptance remain
unverified. Manual official task execution remains held pending verified
native evidence. Stop source work at scoped handoff; no release or
automatic future-version compatibility claim.
No new binary is admitted, and the ABI v4/local-test holds below remain.

Branch `Local`; Unreleased. Work: `selective-issue-reporting` (RPT-01–07). Baseline [35e6524](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/35e652489d1ba297aff3845ec51964f5018678b8).

- RPT-01–04: selective desktop and Rust request-client report, 48+16 rows, bounded counts and receipt-grounded recovery/recurrence; no routine logs, polling, raw data or inferred mutation outcomes. [Exact prior handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/35e652489d1ba297aff3845ec51964f5018678b8/planning/development.md).
- RPT-05: one ordinary product desktop writer after exclusive instance lease; `Data/Reports/issues.json` latest issue-bearing lifetime, 16 KiB hard cap. Native checkpoint after request gate release; UI issue-only coalesced submission on new/recovered/recurrent/overflow or power-of-two counts. Explicit Copy retry and exit flush; identical on-disk bytes skip; exclusive fixed staging, pinned directory chain, same-directory publication. Healthy runs preserve previous checkpoint. Interrupted/unrecognized files preserved, no automatic deletion or historical recovery-state import. No new dependency, timer or service.
- RPT-06: existing DEV.cmd report --json reads only the checkout's bounded checkpoint, validates current schema 2, maps all 50 known issue codes to current source investigation routes and prioritizes up to 12 observations with explicit omitted/unknown/truncated coverage. UI/native provenance retained, build and cause unknown; review SHA is not report build. No runtime instrumentation, AI calls, writes, execution or automatic remediation. Authored offline/catalog/routing fixtures NOT_EXECUTED. [Developer report guide](../Development/tooling/developer/README.md).
- RPT-07 integration audit: typed Rust operation/code vocabulary reused for full prior-checkpoint validation before replacement; explicit native-only Check/Copy with nullable UI checkpoint, no healthy report or false absence on failed native read; cross-language vocabulary and regression fixtures authored NOT_EXECUTED. Native failure checkpoint I/O is synchronous after transport gate release; outer manager gate can remain held. No new poll, service, dependency or session-wire change.
- Standalone process failures, crash before checkpoint, unsubmitted UI repeats, historical UI read, exact build identity, correlation and measured overhead remain unresolved. [Checkpoint contract](../Development/apps/desktop/README.md#bounded-local-checkpoint). Native/TS fixtures authored NOT_EXECUTED. ICE-01–04 and prior PLA/source scopes closed; ICE-05/OPT-07 held.

**Prior handoff — SOURCE_READY_FOR_LOCAL (scoped source handoff, re-pin after UPD changes).** The operator requested preparation for the PC-local phase, not execution on this surface. Use the single staged handoff below; native ABI v4 digest approval still blocks standard dev/package/new launch. All tests/builds/benchmarks/native acceptance NOT_EXECUTED. No CI or unapproved local validation.

## Unreleased — Rust-first integration

Prior FLOW-01–09, MCP-01–08, OPT (except OPT-07), E2E-01–06, BS-01–10 and SIM/EFF/DDR/NGE/FMS/DST/Rust-first scopes remain source-closed. Preserve manual/automated map playtesting and evidence-grounded issue reporting, original five-profile ownership and all active constraints. Exact prior decisions, Work IDs, relative links and proof: [complete pre-closure history](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/4ddb3c34311ccbe81b4f831cd743426d6cd90ee5/planning/development.md) and [earlier checkpoint](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md). Historical Next fields do not authorize action.

## Active implementation roadmap — Rust-first

Original Phase 01–06 source integration closed; VAL-01–06 held. Use [implementation map](../docs/system/implementation-map.md), [architecture](../docs/system/architecture.md) and [development operations](../docs/system/development-operations.md). No broader migration, new fallback or repeat of closed work.

## Capture and recording — separately authorized source workstream

CAP-01–05 source integrated: lossless client WebP screenshot, selected-slot HEVC NVENC/MKV recording, hardware-only FFmpeg candidate, audio off, no fallback, bounded ownership/no-clobber publication. Binary approval gates remain closed. Native capture/resize, GPU negotiation, disk failure and measured quality unverified. [Recording contract](../docs/system/recording.md); [prior details](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md#capture-and-recording--separately-authorized-source-workstream). Shared MCP observation remains separate.

**Hardware efficiency — HW-01 (Unreleased, source only):** Operator-selected RTX 3070 compatibility supersedes AV1-only: one HEVC Main NVENC/MKV path with D3D11 frames, unchanged bounded supervision, no software fallback, no CUDA toolkit/custom kernels, no new UI/worker/controller. [NVIDIA matrix](https://developer.nvidia.com/video-encode-decode-support-matrix) and [FFmpeg 8.1 source](https://ffmpeg.org/doxygen/8.1/nvenc__hevc_8c_source.html) support the codec decision, not native negotiation. The existing optional capture worker releases its acquired GPU frame immediately after fitted readback/timestamp, before CPU encoding or stdout backpressure; it preserves identity checks, packet formats and cadence. Existing adaptive observation, FPS pacing, DWM previews, GPU buffer reuse, resource limits and memory-canary holds remain unchanged; no speculative affinity or new CUDA path. Acceptance: existing CAP checks plus independent HEVC decode, measured game frame time/CPU/RAM/VRAM and capture latency/cadence at matched one-to-five-client loads. HEVC redistribution review, both binary gates, exact Windows driver/device negotiation and all tests/builds/benchmarks remain held / NOT_EXECUTED. Do not infer speedup, installed support or reopen prior closed optimization scopes from this change.

**Final pre-local improvement — FIN-01–08 (Unreleased, source only):**
Baseline `ac43c876bea60579164c1dc7b07cb1f7bd244fa2`. A complete recording
terminal marker now starts the existing finalization deadline before EOF;
publication still requires clean reader completion, child success, positive
frames, target and file checks. Close child/job before bounded reader result
and join on ordinary exit/error; timeout or panic containment remains native
validation, not a guarantee of thread termination. Same-slot Saved resolves
LT-CAP-05; failed/unknown/intermediate states do not imply recovery. Retain
200 ms active supervision and per-generation file identity checks (FIN-04/05):
no new worker, cache or speculative optimization. First-run guide selects
one source route; native performance guidance no longer routes to historical
VMware contract. Authored Rust/UI fixtures NOT_EXECUTED. No binary admission,
release, version bump, CI or native performance claim. Re-pin actual HEAD
for the existing staged PC-local handoff below.

**Reliability & efficiency closure — REV-01–08 (Unreleased, source only):**
Baseline `d8b03d0e9090e7e12fbaa0ec061dfc3f0536d19f`. Move existing Rust
format checks before compilation/testing, once per crate; correct preflight
next-action message to respect native payload admission. Actual formatting
and format PASS remain NOT_EXECUTED; do not claim existing source is
rustfmt-clean. Recorder uses bounded original-process wait and bounded
reader cancellation/join; uncertain cleanup refuses new recordings for
the manager lifetime, not proof of exit. Session request timeout and later
pending admission request cancellation of only the original client worker;
retain the worker and uncertainty until original result/disconnect and
fresh full Inspect; no replay or forced controller/game termination.
**REV-02 follow-through (baseline `47ac059e340491190b2635be539fa6521dcf7bf1`):**
Shared provider runner now uses one original-child guard and a 2 s
nonblocking exit-observation budget after kill; retain unconfirmed child
handles with their admission lease instead of losing ownership. The 24
I/O-worker limit is preserved; 12 direct helper leases cover active and
retained children, and later admission probes retained handles once and
refuses new launches while any remain unconfirmed. Already admitted
operations are not replayed or retroactively canceled. No idle thread,
new dependency, descendant termination or PID reopen. Ordinary early pipe
setup/worker failures and unwind run the same guard cleanup; no actual OS
termination, inherited-pipe cancellation, or speedup is claimed. Authored
fixtures NOT_EXECUTED. Actual rustfmt normalization remains held.
Keep exceptional checkpoint durability, per-request peer verification,
generation/image pins and polling until measured bottlenecks justify a
change (REV-05–07). No new worker, service, dependency or binary admission.
Authored source fixtures NOT_EXECUTED; Windows cancellation/exit, pipe
recovery and performance remain unverified.

**Cross-process and resource closure — BS-01–07 (Unreleased, source only):**
Baseline `484d9b9cb2f5b25c58adef374f6ed54c0c335212`. BS-01: reuse
named-object existence admission for per-slot recording/managed close
exclusion at the retained controller, including CLI; acquire selected
running slots before close effects, hold recording lease across final
target validation and cleanup. Release on terminal failure; unknown cleanup
still quarantines starts, does not indefinitely block normal game close.
BS-02: expose terminal cleanupUnconfirmed in existing recording receipt,
strict TS parser and disabled start across slots, no new requests.
BS-03: standalone settings saves retain original controller startup
existence gate through observation/write, refusing concurrent startup.
BS-04/06: share bounded preference reading with opened-file metadata
checks and Windows reparse-point refusal; audio stage cleanup owns only
its successfully created stage, reuse checked storage directory. Preserve
formats and independent image revalidation; ancestor replacement races
against outside writers are not proven closed. BS-07: bound WebP
destination before append and clear partial output, not codec scratch
or peak RAM. BS-05 follow-through (baseline `9b72f670d667698b434a683c593743d1ee6fa91d`):
server-only overlapped pipe I/O, one original worker and manual-reset
operation/stop events. Idle connect is event-driven; authenticated request
frame, reply write and ACK each use a 5 s phase deadline shared across
partial transfers. Actor execution and client wire unchanged; no replay.
Cancel exact pending operation and wait for actual completion before
buffer reuse/reconnect. Slow OS cancellation can retain the original
worker; existing bounded stop refuses to claim cleanup, not a hard
kernel deadline. Synthetic stalled request/missing ACK fixtures authored.
Cross-process lease behavior, native pipe recovery and all fixtures remain
NOT_EXECUTED; no binary admission, new worker or service.

## Proof ceiling and unresolved gates

- Native presentation ABI v4 digest pair not admitted; standard dev/package/new launch held. Exact DLLs require independent review; never substitute arbitrary bytes.
- No current source test, Windows build, five-client/gameplay, UI visual, capture/recording or performance evidence.
- World/pack disposition, download cache root and seven-day log producer/root held. Existing web-cache admission unchanged; preserve accounts, worlds, private hives, snapshots and legacy roots.
- Client-native F3 `BLOCKED_ENGINE_ABI`: exact Education 26.33 image and input/player/dimension/camera/render interface required. No guessed offsets or fake toggle. [Design](../docs/system/minecraft-education-native-f3-client.md).
- Experimental input/AI/MCP holds unchanged: default OFF, exact writer/neutral/quarantine semantics, no unverified gameplay or Probe dispatch. [Prior decision](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md#separate-original-native-input-hold--unchanged).
- Candidate export not publication: extracted payload/legal notices, signed bytes, Authenticode chain/timestamp, clean/upgrade/uninstall and active/concurrent native session evidence held. Installer idle check is point-in-time. Repository visibility operator-managed.

## Source-to-local handoff — SOURCE_READY_FOR_LOCAL

Source baseline reviewed: `d11a5f63b5db8db5ed63533dbf5f2b06c90724e5` (RPT-07). This handoff-only commit advances HEAD without changing executable source. Re-pin the actual Local HEAD and reconcile any later changes before validation. Applies to completed scoped source work, including RPT-01–07, PLA, ZW, MCP/Code Mode and earlier closed scopes, not whole-product readiness or execution permission.

### One PC-local entry — progressive acceptance

Use the existing [Windows first-run guide](../docs/validation/local-pc-first-run.md) and `DEV.cmd`. No new runner, package format or status store. Only a verified `LOCAL_PC_AGENT` with explicit operator approval for each applicable held stage may execute:

1. **Admission:** confirm actual Windows x64 PC/agent, clean Local checkout and full SHA; run read-only `DEV.cmd localcheck`. Preserve all existing App/Data, accounts and worlds. Missing native DLL approval is expected and blocks native stages, not independent source checks when their own prerequisites are met. Prepare pinned tools with `DEV.cmd bootstrap` only if missing and authorized; verify prerequisites per the first-run guide.
2. **Source:** run `DEV.cmd preflight` once after approval. It reuses/installs locked npm dependencies as needed and includes frontend Vitest, desktop/runtime Rust checks/tests and the developer report-review suite. Do not precede it with redundant setup, quick, check or agent-suite runs. Default Cargo features only; product/experimental feature acceptance remains separate. Stop on failure; correct the first owning source and re-pin affected evidence. Preserve the private preflight manifest.
3. **Native prerequisite:** use VAL-01/02 below for independently reviewed ABI v4 x64 DLL build/controls and separate exact digest admission; `REVIEWED_EDUCATION_BOOTSTRAP = None` at the baseline. Until admitted, do not follow preflight's generic dev suggestion, launch a new game or package. No placeholders, old DLLs, hash bypass or experimental flags.
4. **Product smoke:** after native admission and matching approval, use `DEV.cmd dev` and the existing progressive LT-01–13 plan, manager then MC01/MC02, before any five-client escalation. Preserve original slot/session ownership, normal close/reconnect and private data. Reuse already matching source receipts rather than rerunning the quick screen. Packaging, capture/recording, experimental AI/input and full regression escalation remain separate gates.
5. **Report acceptance during the same approved smoke:** on an isolated checkout/data root, healthy explicit Check yields no current issue report and creates no checkpoint; any existing historical checkpoint remains unchanged. A safely observed UI/native issue yields bounded sanitized evidence; check native-only Check/Copy, repeat/recovery semantics, exit/Copy checkpoint and retention via existing regression fixtures and naturally observed failures. Never induce faults in licensed sessions or overwrite real evidence. Use `DEV.cmd report --json` once when needed: no report is not health proof; UI/native provenance stays separate, at most 12 review candidates, root cause/build identity unknown. Malformed/oversized/interrupted evidence preservation belongs to existing disposable fixtures; do not modify real Data to simulate it. Unobserved runtime cases stay NOT_EXECUTED/INCONCLUSIVE.
6. **UPD acceptance during the same approved native phase:** observe the actual official task and registration before/after an ordinary supported update on a disposable installation/data root. Check Settings healthy/busy/disabled/missing/unknown and failed observation; no stale success or duplicate UI requests. Verify queued/running task refusal, unchanged generation reuse across five launches, last-exit pin release, full hash of new bytes, unknown hash refusal before profile mutation, and normal close/account/world preservation. Do not trigger an unreviewed scheduled task, force an update, downgrade or modify real user data. Compare actual hash count/timing separately from Authenticode and DLL checks; if no natural approved update occurs, leave transition NOT_EXECUTED. No future-image admission without exact native review.
7. **Feedback/STOP:** use existing private `workspace/runs/` receipts, exact SHA, first failing stage, sanitized observed vs expected and canonical source owner. Do not dump logs or commit private Data. No automatic fix, replay or repeat of successful unaffected checks; stop at the selected proof ceiling. OPT-07/ICE-05 and other holds below remain unchanged.

All commands above are deferred; this is a source handoff, not a test or release authorization.

| Gate | Existing evidence owner |
| --- | --- |
| VAL-01/02 | [Windows first-run](../docs/validation/local-pc-first-run.md), [native CMake controls](../Development/tooling/acceptance/windows/bootstrap-ordering/README.md): compile/link/fixtures and independent ABI v4 payload review. |
| VAL-03 | [Progressive smoke](../docs/validation/efficient-local-testing.md): one then five licensed sessions, identity/layout/close/reconnect/preservation, MC02 exit regression. |
| VAL-04 | [Data layout](../docs/system/data-layout.md): disposable data, exact producer/root, reference stability and exclusive disposition before new cleanup admission. |
| VAL-05 | [Native collectors](../Development/tooling/acceptance/windows/README.md): matched game/workload, FPS, CPU/RAM/GPU, handles and timings. Compare MCP surfaces and actual local host/SDK; OPT-07 measured IPC remains held. |
| VAL-06 | [Distribution](../Development/engine/distribution/README.md): extracted installer, signatures, legal notices, layout and active-session/data lifecycle. |

All native and source test execution remains NOT_EXECUTED under the hold. No CI, cloud substitution, release or version bump.
