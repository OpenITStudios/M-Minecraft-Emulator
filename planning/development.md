# Current Development Handoff — Lazy Tester

## Current status

Branch `Local`; Unreleased. Work: `prelocal-efficiency-and-correctness-audit` (PLA-01–08). Scoped source corrections and integration review closed at available source evidence; **not full product certification**. Baseline `f7bcf68309c92c68536dd32a931b2c6b7b5900cb`; reviewed implementation through `4ddb3c34311ccbe81b4f831cd743426d6cd90ee5`. Six commits, 15 changed files before closure; no manifest, dependency, lockfile, ABI, installer identity, protected data, or CI changes. No new runtime owner. Closure adds docs and authored fixture guards only.

- PLA-01/02: reconcile uncertain Settings/Storage via full Inspect under one 45s budget, no replay; pure agent metadata receipts omit unrelated five-profile roster. Desktop and held input ACK distinctions preserved.
- PLA-03/04: informational activity uses existing settled-live 5s cadence; fast recovery/attention/stale/input cadence and idle rules remain. Skip needed caption writes when hung; post-operation observation retained. Wait once on original helper child after successful End; fallible transport worker spawn avoids mutex poisoning.
- PLA-05/06: shared trusted desktop exact-slot target for workspace/capture, full Inspect when uncertain; DirEntry metadata only for ordinary cache reports; retain fresh mutation/fingerprint probes and pins. Reuse one storage category type, no speculative module splits.
- PLA-07/08: compare exact baseline, manifest/locks, package overlay, guards and selected legacy paths; scan 27 desktop fixtures for changed routing/receipt names. Handoff exceeded strict 8,000-character guard; condensed without weakening checker. Default vs product/experimental feature coverage clarified in existing runbook; fixture authored NOT_EXECUTED.

No executable VMware/Sandboxie provider found in inspected active paths, manifest/lock metadata, packaging/toolchain routes or retired-provider tree names. Retain historical provenance, refusal and absence guards, not a second runtime. No data deletion or local uninstall. Bounded review, not exhaustive function-level census, compiler/feature proof or vulnerability assessment. No blanket dead_code removal, dependency upgrade or persistent IPC without evidence. OPT-07 measured IPC held.

**Next — STOP at scoped source evidence ceiling.** Standard native ABI v4 digest approval missing; standard dev/package/new launch remains blocked. Default source verification is not product-feature compilation. All tests, builds, native acceptance and measurements NOT_EXECUTED; separate verified PC-local authorization and existing holds remain. [PLA acceptance and feature coverage](../docs/validation/efficient-local-testing.md#prelocal-efficiency-and-correctness-pla-0108).

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
