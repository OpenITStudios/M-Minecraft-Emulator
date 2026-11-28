# Current Development Handoff — Lazy Tester

## Current status

Branch `Local`; Unreleased. Work: `selective-issue-reporting` (RPT-01–05). Baseline [35e6524](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/35e652489d1ba297aff3845ec51964f5018678b8).

- RPT-01–04: selective desktop and Rust request-client report, 48+16 rows, bounded counts and receipt-grounded recovery/recurrence; no routine logs, polling, raw data or inferred mutation outcomes. [Exact prior handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/35e652489d1ba297aff3845ec51964f5018678b8/planning/development.md).
- RPT-05: one ordinary product desktop writer after exclusive instance lease; `Data/Reports/issues.json` latest issue-bearing lifetime, 16 KiB hard cap. Native checkpoint after request gate release; UI issue-only coalesced submission on new/recovered/recurrent/overflow or power-of-two counts. Explicit Copy retry and exit flush; unchanged digest skip; exclusive fixed staging, pinned directory chain, same-directory publication. Healthy runs preserve previous checkpoint. Interrupted/unrecognized files preserved, no automatic deletion or historical recovery-state import. No new dependency, timer or service.
- Standalone process failures, crash before checkpoint, unsubmitted UI repeats, historical UI read, exact build identity, correlation and measured overhead remain unresolved. [Checkpoint contract](../Development/apps/desktop/README.md#bounded-local-checkpoint). Native/TS fixtures authored NOT_EXECUTED. ICE-01–04 and prior PLA/source scopes closed; ICE-05/OPT-07 held.

**Next — STOP at scoped RPT-01–05 source ceiling.** Native ABI v4 digest approval missing; standard dev/package/new launch blocked. All tests/builds/benchmarks/native acceptance NOT_EXECUTED. No CI or unapproved local validation.

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
