# Developer and Acceptance Tooling

Tooling provides optional, bounded checks and evidence collection. **It does
not implement a second Minecraft runtime or background service.**

| Owner | Current use |
| --- | --- |
| [`isolation-source/README.md`](./isolation-source/README.md) | **Sole current v1.18.6** source verifier, optional compile-only build and NOT-INSTALLABLE assembler; v1.18.5 scripts are historical |
| [`repository/verify.mjs`](./repository/verify.mjs) | Static repository/source ownership invariants |
| [`local-test/session.mjs`](./local-test/session.mjs) | Metadata-only exact-SHA source-preflight records; never captures raw logs or Minecraft profiles |
| [`windows-toolchain/dev.ps1`](./windows-toolchain/dev.ps1) | Portable `DEV.cmd bootstrap` / `doctor` / `preflight` / `dev` using one checkout |
| [`acceptance/windows/collect-sandboxie-feasibility.ps1`](./acceptance/windows/collect-sandboxie-feasibility.ps1) | **Active** fixed `MinecraftClient02` candidate and box inspection; desktop calls `-SummaryOnly` without writing a report |
| [`acceptance/evaluate-sandboxie-feasibility.mjs`](./acceptance/evaluate-sandboxie-feasibility.mjs) | Interprets collected account/gameplay claims **only as operator-attributed evidence**, not automatic login proof |
| [`acceptance/windows/README.md`](./acceptance/windows/README.md) | Active Sandboxie entrypoint plus preserved historical VMware G0–G4 stage documentation |
| [`acceptance/evaluate-client-feasibility.mjs`](./acceptance/evaluate-client-feasibility.mjs) | **Legacy guest-VM** feasibility evidence; inactive for selected Client 02 path |
| [`acceptance/evaluate-native-feasibility.mjs`](./acceptance/evaluate-native-feasibility.mjs) | Earlier native-only feasibility evidence; not the selected isolation engine |

Sandboxie inspector output is candidate-only. Source tests are not measured
CPU/RAM savings, authenticated Minecraft Education accounts or Windows
gameplay acceptance. Canonical architecture:
[Lightweight Windows Architecture](../docs/system/architecture.md).
