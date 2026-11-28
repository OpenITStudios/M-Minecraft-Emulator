# Current Development Handoff — Lazy Tester

## Current status

Branch `Local`; Unreleased. Work: `prelocal-modularity-efficiency`.
MOD-01–09 closed at [64d3922](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/64d392206c51922d5fccc191eafe99acc6fc95a0). SIM-MOD-01–04: [exact prior handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/d8a49bcd7dc144da2ea67a4758a7256ce4e1d9af/planning/development.md). SIM-MOD-05: session protocol owns shared request validation and frame size; exact JSON size counted without discarded frame; transport fixtures separate, original test namespace retained. Both ingress checks and worker uncertainty preserved. Source only; fixtures NOT_EXECUTED.
Final review baseline: `bf2588dc527d137fe75e4cdf6559130259b1dcf3`. Original work baseline: `ed320acd98065300e48dce9f574af0bfd7e7f18b`.
No source test, compiler, formatter, build, CI, PC or runtime execution.

| Scope | Source result / exact commit |
| --- | --- |
| MOD-01 | Agent activity contract, fixed slot flags — [2efb1c8](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/2efb1c84c1e11a5ecb0ced9279977bc09a1749f8) |
| MOD-02 | Session protocol, Testing Run, AI input leases — [45c3674](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/45c3674b6fcf813a8b60376d0df6f9fe258b3fe2) |
| MOD-03 | Reviewed payload, child configuration/environment, fixtures — [0078a2c](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/0078a2c1e17707563a23776f4cf6ccaf16ee4a24) |
| MOD-04 | Owned windows, audio, reply scope, fixtures — [f04a61c](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/f04a61ce540c12d03c59fe58597a933dbf83ee6f) |
| MOD-05 | Windows identity, package, signature, private-root observers — [476e05d](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/476e05d5452a53259da7b7710b0868b893dc3307) |
| MOD-06 | Agent configuration, Probe QA and evidence operations — [14ffcac](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/14ffcac87f66325685cb37be430de51c4ece1538) |
| MOD-07 | Report-local process ownership reuse and source routing — [f7fc983](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/f7fc983ead90964b40d99ec68c06213ab19e8215) |
| MOD-08 | Scoped named-owner declarations in existing verifier; review responsibility, state, naming, forwarding and dependencies without a line limit. Handoff compacted to existing bound. Fixtures authored NOT_EXECUTED. |

| MOD-09 | Correct staging, packaging and readiness digest-owner paths; repair stale startup fixture boundaries and classify extracted Rust test files correctly. Source fixtures authored NOT_EXECUTED. |

Next: review agent operation core redundancy, then focused integration. Keep native holds. Re-pin actual HEAD for any separately
approved PC-local stage. The existing native DLL and release holds remain.
No compilation, runtime improvement or whole-product readiness claimed.

## Prior source work and continuity

UPD, RPT-01–07, CAP-01–05, HW-01, FIN-01–08, REV-01–08, BS-01–07,
FLOW-01–09, MCP-01–08, OPT (except OPT-07), E2E-01–06, BS-01–10,
SIM/EFF/DDR/NGE/FMS/DST/Rust-first and ICE-01–04/PLA scopes remain
source-closed, not executed acceptance. OPT-07, ICE-05 and VAL-01–06 held.
Preserve existing decisions and stricter safety constraints; do not reopen
closed work or treat historical Next fields as authority.
[Complete exact pre-MOD-08 handoff and decision history](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/f7fc983ead90964b40d99ec68c06213ab19e8215/planning/development.md).
Use [implementation map](../docs/system/implementation-map.md),
[architecture](../docs/system/architecture.md) and
[development discipline](../docs/system/development-discipline.md).

## Proof ceiling and unresolved gates

- Native presentation ABI v4 digest pair not admitted; standard dev/package/new launch held. Exact DLLs require independent review; never substitute arbitrary bytes.
- No current source test, Windows build, five-client/gameplay, UI visual, capture/recording or performance evidence.
- World/pack disposition, download cache root and seven-day log producer/root held. Existing web-cache admission unchanged; preserve accounts, worlds, private hives, snapshots and legacy roots.
- Client-native F3 `BLOCKED_ENGINE_ABI`: exact Education 26.33 image and input/player/dimension/camera/render interface required. No guessed offsets or fake toggle. [Design](../docs/system/minecraft-education-native-f3-client.md).
- Experimental input/AI/MCP holds unchanged: default OFF, exact writer/neutral/quarantine semantics, no unverified gameplay or Probe dispatch. [Prior decision](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/9b285c2110b7cdf91eda7e26c315a0b850eab82c/planning/development.md#separate-original-native-input-hold--unchanged).
- Candidate export not publication: extracted payload/legal notices, signed bytes, Authenticode chain/timestamp, clean/upgrade/uninstall and active/concurrent native session evidence held. Installer idle check is point-in-time. Repository visibility operator-managed.

## Deferred PC-local validation — not an authorization

Only a separately selected verified `LOCAL_PC_AGENT` with explicit
operator approval may execute the existing sequence. Re-pin actual Local
HEAD after the focused review; the old source-ready baseline is not current proof.
Use [Windows first-run guide](../docs/validation/local-pc-first-run.md),
[progressive smoke](../docs/validation/efficient-local-testing.md) and
the [full prior acceptance sequence](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/f7fc983ead90964b40d99ec68c06213ab19e8215/planning/development.md#source-to-local-handoff--source_ready_for_local).

1. Admission: actual Windows x64 PC/agent, clean checkout and exact SHA;
   read-only DEV.cmd localcheck. Preserve App/Data; bootstrap pinned tools
   only if missing and approved.
2. Source: approved DEV.cmd preflight once, default Cargo features;
   no redundant setup/quick/check or agent suite. Stop at first failure,
   correct canonical owner and re-pin; retain private receipts.
3. Native prerequisite: independently review exact ABI v4 DLL pair and
   admit approved digests in reviewed_bootstrap_payload.rs. Ordinary
   dev/package/new launch held until then; no old or placeholder hashes.
4. Product smoke only after matching native admission: manager then
   MC01/MC02, progressive escalation, preserve slots/data, normal close
   and reconnect. Packaging, capture/recording, AI/input separate gates.
5. RPT and UPD acceptance during approved smoke: follow the exact prior
   acceptance sequence above for disposable roots, checkpoint/receipt
   preservation, ordinary official update observation, generation pins,
   version/ABI refusal, no unreviewed task invocation or forced update.
6. Feedback: exact SHA, first failing stage, sanitized observed versus
   expected, canonical owner. No private logs/data committed, automatic
   repair/replay or repeat of unaffected successful checks.

| Gate | Evidence owner |
| --- | --- |
| VAL-01/02 | [Windows first-run](../docs/validation/local-pc-first-run.md), [native CMake controls](../Development/tooling/acceptance/windows/bootstrap-ordering/README.md) |
| VAL-03 | [Progressive smoke](../docs/validation/efficient-local-testing.md): one then five sessions, identity/layout/close/reconnect/preservation |
| VAL-04 | [Data layout](../docs/system/data-layout.md): exact producer/root, reference stability and exclusive disposition |
| VAL-05 | [Native collectors](../Development/tooling/acceptance/windows/README.md): matched workload, FPS, CPU/RAM/GPU, handles and timings; OPT-07 held |
| VAL-06 | [Distribution](../Development/engine/distribution/README.md): installer bytes, signatures/legal notices, lifecycle |

All execution deferred. No CI/cloud substitution, release or version bump.
