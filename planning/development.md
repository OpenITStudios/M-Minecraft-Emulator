# Current Development Handoff — Lazy Tester

## Current status

**Active source work — Education update compatibility (UPD).** Baseline
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
authored NOT_EXECUTED. Pending: native updater/task behavior and costs,
safe official check action, capability-specific admission, future recipe
review and end-to-end update acceptance.
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

CAP-01–05 source integrated: lossless client WebP screenshot, selected-slot AV1 NVENC/MKV recording, hardware-only FFmpeg candidate, audio off, no fallback, bounded ownership/no-clobber publication. Binary approval gates remain closed. Native capture/resize, GPU negotiation, disk failure and measured quality unverified. [Recording contract](../docs/system/recording.md); [prior details](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md#capture-and-recording--separately-authorized-source-workstream). Shared MCP observation remains separate.

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
6. **Feedback/STOP:** use existing private `workspace/runs/` receipts, exact SHA, first failing stage, sanitized observed vs expected and canonical source owner. Do not dump logs or commit private Data. No automatic fix, replay or repeat of successful unaffected checks; stop at the selected proof ceiling. OPT-07/ICE-05 and other holds below remain unchanged.

All commands above are deferred; this is a source handoff, not a test or release authorization.

| Gate | Existing evidence owner |
| --- | --- |
| VAL-01/02 | [Windows first-run](../docs/validation/local-pc-first-run.md), [native CMake controls](../Development/tooling/acceptance/windows/bootstrap-ordering/README.md): compile/link/fixtures and independent ABI v4 payload review. |
| VAL-03 | [Progressive smoke](../docs/validation/efficient-local-testing.md): one then five licensed sessions, identity/layout/close/reconnect/preservation, MC02 exit regression. |
| VAL-04 | [Data layout](../docs/system/data-layout.md): disposable data, exact producer/root, reference stability and exclusive disposition before new cleanup admission. |
| VAL-05 | [Native collectors](../Development/tooling/acceptance/windows/README.md): matched game/workload, FPS, CPU/RAM/GPU, handles and timings. Compare MCP surfaces and actual local host/SDK; OPT-07 measured IPC remains held. |
| VAL-06 | [Distribution](../Development/engine/distribution/README.md): extracted installer, signatures, legal notices, layout and active-session/data lifecycle. |

All native and source test execution remains NOT_EXECUTED under the hold. No CI, cloud substitution, release or version bump.
