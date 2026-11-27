# Current Development Handoff — Lazy Tester

## Current status

Branch `Local`; source-only work, Unreleased. Baseline `9b285c2110b7cdf91eda7e26c315a0b850eab82c`.
The operator authorized further zero-waste improvements **excluding MCP and AI integration**, including MCP visual observation and shared experimental input interfaces. No UI redesign, new runtime controller, data migration, CI, local test or release. The next source work is limited to independent non-MCP consumers and concrete redundancy; do not reopen closed work without evidence.

Work: `rust-execution-efficiency` (RUST-03) — web-cache observation and exclusive cleanup preflight now transfer the already validated final ancestor/root pin into the existing pinned traversal instead of reopening that directory. One fewer directory open, repeated directory identity/path validation and retained handle per existing cache root; same access/share flags, ancestor and file pins, bounded traversal and disposition gates. Directory limit still counts actual retained handles. Missing roots and held download/log/pack/world routes unchanged. Fixture covers root count/path, rename refusal and leaf preservation in both plans; not executed. Independent metadata/reference checks retained; no persistent cache, new worker, dependency or AI/MCP changes. No measured I/O or latency savings.

Prior work: `rust-execution-efficiency` (RUST-02) — selective inventory and confirmation fingerprints now write lowercase hex directly into one exactly reserved String instead of allocating a String per byte, collecting a Vec and joining. SHA-256 input, domain separators, ordering and 64-character wire output unchanged. Pack dependency version validation deserializes the borrowed JSON Value instead of cloning its tree; same [u32; 3] validation and 65535 bound. Fixtures cover all byte values, leading zero, borrowed/owned valid and invalid version parity, and existing fingerprint/selection contracts. No dependencies, IPC, data disposition or AI/MCP changes. Source-reviewed; fixtures not executed; no measured RAM/latency claim. Retain lifecycle receipt ownership and other clones without measured or concrete redundancy.

Prior work: `rust-execution-efficiency` (RUST-01) — controller preparation resolves the official registered image for its initial native process observation, then pins that exact path and repeats the existing RegisteredOnly registration/signature and file-identity check. Successful preparation uses two resolver subprocesses instead of three; existing-game rejection still uses one. Each launch retains its independent two-pass admission, and the legacy selector remains in the initial observation. No persistent cache, PowerShell trust-policy rewrite, new dependency, IPC or input route change. Synthetic regression covers post-pin validation, changed path, failure and pin release; not executed. Complete native replacement of the resolver is deferred without equivalent Start-menu, Store-package and signature/catalog evidence; no measured savings.

Prior work: `storage-interface-efficiency` (EFF-06/08) — active desktop web-cache clear, settings reset/restore and full profile clear explicitly request one exact-slot post-mutation storage row; the default wire contract still returns five storage rows, and all lifecycle rows and fresh mutation admissions remain. Held pack/world/download-cache and experimental settings operations retain their full receipts. No second request, cache, command, or new controller. EFF-08: dormant Saved Locations native store/command deliberately retained by prior retirement decision, with no mounted UI consumer or periodic work; no data or compatibility deletion for an unmeasured benefit. Source/receipt fixtures authored, not executed. Broader source review scoped, native/build/performance NOT_EXECUTED.

Prior work: `background-efficiency` (EFF-04/05) — EFF-04 reviewed: retain the existing fixed-buffer caption check and exact HWND validation; the game can reset its title, and neither a cached label nor a second observer is warranted. EFF-05: with live games and no pending storage/audio/retry, the healthy native wait now uses the maximum finite Win32 budget; retained game handles, request event and pipe-worker exit remain wake sources. Manager-only/startup cadence, audio deadlines, storage maintenance and degraded channel fallback are unchanged. No new timer, thread, cache, command, or AI/MCP route change. Fixture authored, not executed; resource savings not measured.

Prior work: `session-efficiency` (EFF-02/03) — retained-controller read-only InspectSettings now derives its report after the existing full lifecycle snapshot, with one reap rather than two; fresh settings mutation admission, post-mutation snapshots, five profile rows, pinned-image read-only inspection and uncertainty gates are unchanged. If an external game exits during the report, the previous live observation remains until the next normal observation; no cached mutation permission. EFF-03 reviewed: retain the one-shot IPC worker/receiver because its join and pending-result provenance bound uncertain transactions; a persistent worker adds lifecycle/idle state and changes a shared AI/MCP transport. No rewrite without measured need and separate scope. Fixture authored, not executed; no benchmark or runtime proof.

Prior work: `workspace-efficiency` (EFF-01/07) — bounded source change: preview overlap admission projects the main viewport and up to four tiles in one manager geometry sample; actual native placement keeps fresh observations. UI skips redundant geometry reads when no selected-game update is awaited and passes readonly tiles without a second array. Existing full session Inspect for retained/requested sources remains required by the current presentation contract; no lease-only bypass, new cache, new command, wire change, or MCP/AI edits. Source fixtures authored, not executed. No measured gain. Broader EFF-01 lease-only admission deferred absent a complete compatible lifecycle proof. EFF-02/03 subsequently reviewed in the current work above.

Prior work: `zero-waste-followup` — scoped source review closed. ZW-01: selected Storage entry/Refresh/launcher selection use one exact-slot report and a single UI publication fence; five lifecycle rows, legacy full report and full mutation receipts preserved. No persistent cache, new controller or MCP/AI route edits. Picker focus regression and Windows fixture guard authored, not executed. ZW-07/08: semantic inspection names and shared ownership, no cosmetic file splitting or package/lock/path rename; current build identities documented in canonical naming. Full mutation report cost remains; visiting all five profiles can require five requests. No measured gain or native/UI execution claim.

Current source changes: [selected storage and integration checkpoint](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/b352dee93199cd8c006df8faf02cc237149907b9/planning/development.md). Prior ZW-09, narrow ZW-08 and selective identity reuse: [checkpoint at 0977e789](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/0977e7898b76b7621d9629658746eff0689cbc55/planning/development.md). Earlier ZW-02–06 and retained retry follow checkpoint links. Dependency removal remains BLOCKED_LOCK_RESOLUTION; manifests/locks preserved. SOURCE_READY_FOR_LOCAL for scoped source work / LOCAL_TEST_DEFERRED. No further speculative refactor; STOP.

## Unreleased — Rust-first integration

| Closed workstream | Current source result |
| --- | --- |
| SIM / EFF / DDR / OPT | Single existing state owners, deadline-driven observation, retained process waits, on-demand storage, bounded audio discovery, retained DWM pins, no-write equal preferences, lazy HUD canvas. |
| NGE | Handle-local native file operation fast path, expiry latch for startup diagnostic work, combined terminal pacing wait, rejected presentation fast path. |
| FMS | Canonical-first profile locator and opened-root report, selected folder action, canonical developer output paths, unchanged optional archive opt-in. |
| DST | Binary-only Tauri/NSIS candidate, strict frontend/native resource checks, registered root preservation, bounded installer digest/export and independent native staging. |
| Rust-first FPS/NAT/STG/UI/INT | Rust meter/pacer/HUD/presentation admission, paired ABI guards and existing C++ COM/DXGI/Detours adapters, held storage dispositions, native preview and desktop controls. |

All closures are source-level only, not performance, build, installed or gameplay acceptance. Keep current feature/ABI and UI contracts, data preservation, exact identity and uncertainty gates. Details and original Work IDs: [full checkpoint history](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md).

## Active implementation roadmap — Rust-first

Original scoped Phase 01–06 source integration is closed. VAL-01–06 remain held. No broader bootstrap migration, new fallback, speculative dependency removal or repeat of closed optimization work. Use [implementation map](../docs/system/implementation-map.md), [architecture](../docs/system/architecture.md) and [development operations](../docs/system/development-operations.md) to locate actual current owners.

## Capture and recording — separately authorized source workstream

CAP-01–05 source integrated: explicit lossless client WebP screenshot; independent selected-slot AV1 NVENC/MKV recording, hardware-only FFmpeg 8.1 candidate, no H.264/fallback/retry, audio off; bounded leases/job ownership and no-clobber output with retained finalized-file recovery. Both binary approval gates remain closed. Native capture/resize, GPU negotiation, disk failure and measured quality remain unverified. See [recording contract](../docs/system/recording.md) and [prior capture details](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md#capture-and-recording--separately-authorized-source-workstream). Shared MCP observation remains out of scope.

## Proof ceiling and unresolved gates

- Native presentation ABI v4 digest pair is not admitted; standard dev/package/new launch held. Exact current DLLs need independently approved hashes; do not admit arbitrary or changed bytes.
- No current source tests, Windows builds, five-client/gameplay, UI visual, capture/recording or performance evidence. Existing authored fixtures are not executed proof.
- World/pack disposition, download cache root and seven-day log producer/root remain held pending native safety/coverage evidence. Only the existing verified web-cache preference retains its current admission; preserve all accounts, worlds, private hives, snapshots and legacy roots.
- Client-native F3 remains `BLOCKED_ENGINE_ABI`: exact Education 26.33 client identity, native input, Player data/dimension/camera and in-game render interface required. No guessed offsets, alternate overlay or fake toggle. [Design](../docs/system/minecraft-education-native-f3-client.md).
- Original experimental input/AI/MCP holds unchanged, including default OFF, exact writer/neutral/quarantine semantics, no unverified gameplay or Probe dispatch. [Prior native-input decision](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md#separate-original-native-input-hold--unchanged).
- Candidate export is not publication: actual extracted payload and legal notices, final signed bytes, Authenticode publisher/chain/timestamp, clean/upgrade/uninstall and active/concurrent native session proof remain held. Installer idle admission is point-in-time, not a whole-install lock. Repository visibility is operator-managed.

## Source-to-local handoff — SOURCE_READY_FOR_LOCAL

Applies to closed source scopes including the ZW follow-up; runtime proof remains held. Re-pin exact Local HEAD before any separately authorized verified LOCAL_PC_AGENT phase. No action requested or authorized here.

| Gate | Existing evidence owner |
| --- | --- |
| VAL-01 / 02 | [Windows first-run](../docs/validation/local-pc-first-run.md), [native CMake controls](../Development/tooling/acceptance/windows/bootstrap-ordering/README.md): compile/link/fixture evidence and independent ABI v4 payload review. |
| VAL-03 | [Progressive smoke](../docs/validation/efficient-local-testing.md): one then five licensed sessions; identity, layout, close/reconnect, data preservation; historical MC02 exit regression. |
| VAL-04 | [Data layout](../docs/system/data-layout.md): disposable data, exact storage producer/root, dependency reference stability and exclusive disposition before new cleanup admission. |
| VAL-05 | [Native collectors](../Development/tooling/acceptance/windows/README.md): matched game image, workload, FPS, CPU/RAM/GPU, handles and timings. |
| VAL-06 | [Distribution](../Development/engine/distribution/README.md): exact extracted installer, signatures, legal notices, registered flat/standard layout and active-session/data lifecycle. |

All native and source test execution remains NOT_EXECUTED under the repository hold. No CI, cloud substitution, release or version bump.
