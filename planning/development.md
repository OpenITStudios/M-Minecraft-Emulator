# Current Development Handoff — Lazy Tester

## Current status

Branch `Local`; Unreleased. Work: `intermodule-communication-efficiency` (ICE-01/02). Baseline [c7bc043](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/c7bc04300882e10274951fe0adbcdfc0ec4a2c12).

- ICE-01: existing one-shot Rust agent CLI consumes owned OS arguments rather than explicitly copying valid UTF-8 strings. Invalid arguments still refuse; operation/receipt/exit contracts unchanged. No extra CLI, process, wrapper or runtime owner.
- ICE-02: existing authenticated byte-mode session pipe writes a complete newline-delimited JSON request buffer with one `write_all` invocation; reject frames above the existing 32 KiB inclusive limit before writing. Existing request validation, peer verification, reply/ACK, timeout, uncertainty/no-replay, ingress and original actor unchanged. Partial OS writes remain possible. One extra bounded buffer trades memory for fewer small writes; actual latency/CPU benefit unmeasured. Authored pure frame fixture covers exact bytes, one full writer call, inclusive limit and oversize refusal; NOT_EXECUTED.
- Earlier PLA-01–08 source review remains closed; [exact prior handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/c7bc04300882e10274951fe0adbcdfc0ec4a2c12/planning/development.md) preserves previous detailed scope and proof. No reopening of retired providers, data changes or native/ABI gates.
- ICE-03/04 remain proposed only where current source reveals real redundant observation or event reuse; no duplicate state or watcher. ICE-05/OPT-07 persistent IPC, CLI process lifecycle and worker replacement remain measurement-dependent and held. No new dependencies or protocol changes.

**Next — STOP at scoped ICE-01/02 source proof ceiling.** No claim of zero overhead or measured improvement. Standard native ABI v4 digest approval missing; standard dev/package/new launch remains blocked. All tests, builds, benchmarks and native acceptance NOT_EXECUTED. Existing verified PC-local authorization and CI/test holds unchanged.

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

Applies only to completed scoped source work (PLA, ZW, MCP/Code Mode, earlier scopes), not whole product readiness or permission to execute. Re-pin exact HEAD before separately authorized verified LOCAL_PC_AGENT phase.

| Gate | Existing evidence owner |
| --- | --- |
| VAL-01/02 | [Windows first-run](../docs/validation/local-pc-first-run.md), [native CMake controls](../Development/tooling/acceptance/windows/bootstrap-ordering/README.md): compile/link/fixtures and independent ABI v4 payload review. |
| VAL-03 | [Progressive smoke](../docs/validation/efficient-local-testing.md): one then five licensed sessions, identity/layout/close/reconnect/preservation, MC02 exit regression. |
| VAL-04 | [Data layout](../docs/system/data-layout.md): disposable data, exact producer/root, reference stability and exclusive disposition before new cleanup admission. |
| VAL-05 | [Native collectors](../Development/tooling/acceptance/windows/README.md): matched game/workload, FPS, CPU/RAM/GPU, handles and timings. Compare MCP surfaces and actual local host/SDK; OPT-07 measured IPC remains held. |
| VAL-06 | [Distribution](../Development/engine/distribution/README.md): extracted installer, signatures, legal notices, layout and active-session/data lifecycle. |

All native and source test execution remains NOT_EXECUTED under the hold. No CI, cloud substitution, release or version bump.
