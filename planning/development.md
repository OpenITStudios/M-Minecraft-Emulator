# Current Development Handoff — Lazy Tester

## Current status

Branch `Local`; Unreleased. Work: `zero-legacy-runtime`.
Approved goal: one retained native runtime, canonical profile storage, one agent
contract (MCP and SDK adapters), no extra worker/dependency or redundant owner.
Previous MOD/SIM-MOD source closure and exact history:
[15cd0072 handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/15cd0072fcb0e78d74ae661a6092eb4a5eac8851/planning/development.md).

ZLR-01: retired executable pair/five startup and five-profile preparation CLI.
Retired flags fail before dispatch, old runner and legacy profile creation
removed; shared controller journal and test-only historical observer preserved.
Reviewed existing data/replay retained until safe storage cutover. Pair-only
historical data preserved, no automatic expansion. Source fixture NOT_EXECUTED.

Next: CONTINUE ZLR-02 canonical storage cutover: trace schema 1, reviewed
replay/binding, migration and fresh-setup refusal; do not disable recovery
before a safe explicit migration path exists. Then ZLR-03 one agent contract
with retained safety/capability gates, ZLR-04 consumer cleanup/integration.
No compiler, test, CI, runtime, data migration or performance execution.
No release or version bump.

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
