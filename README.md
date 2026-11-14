# LazyTester

**LazyTester** is a Windows desktop manager for up to five independent,
licensed Minecraft Education profiles (MC01–MC05). It uses one official
game installation and persistent, private profile data. The operator
explicitly launches games; opening LazyTester never starts Minecraft,
signs in to Microsoft, or enters a world.

**Development branch:** `Local`. `main` is release-reserved and GitHub
Actions CI is disabled. This is the **source checkout**, not a distribution
folder or an approved installer. Read [current status](planning/development.md)
for the proof ceiling at the latest revision.

## Choose the correct workflow

| Goal | Entry point | Canonical details |
| --- | --- | --- |
| First efficient Windows test | `DEV.cmd localcheck`, then `DEV.cmd plan --profile local-smoke --json` after approved toolchain setup | [13-step local smoke](docs/validation/efficient-local-testing.md), [Windows first-run](docs/validation/local-pc-first-run.md) |
| Read-only evidence-grounded source review | `DEV.cmd review --json` or `review --base-sha <ancestor-SHA> --json` when the pinned toolchain is approved | [Developer controller](Development/tooling/developer/README.md), [development discipline](docs/system/development-discipline.md); no agent/test execution |
| Plan **all** tests and required evidence | 306 uniquely identified planned test cases, not yet executed | [Complete master](docs/validation/lazy-tester-test-master.md), [198 standard cases](docs/validation/lazy-tester-test-cases-standard.md), [108 MCP/AI/QA cases](docs/validation/lazy-tester-test-cases-experimental.md) |
| Open an already prepared app | Root `Lazy Tester.exe` or compatibility `RUN.cmd` | [Desktop workspace](docs/product/unified-workspace.md) |
| Review current desktop navigation safely | Open Play → Settings → General / Data Storage; inspect read-only status, without changing game data | [Current workspace](docs/product/unified-workspace.md) |
| Examine uninstalled experimental native coordinate HUD extension | `Development/tooling/agent/education-native-hud-prototype/` | [Compatibility and integration contract](docs/system/education-native-coordinate-hud.md) |
| Develop or package locally **when explicitly authorized** | `DEV.cmd` | [Developer controller](Development/tooling/developer/README.md), [distribution](Development/engine/distribution/README.md) |
| Inspect or launch profiles through an external model | Optional `Development/tooling/agent/mcp.mjs` | [MCP bridge](Development/tooling/agent/README.md) |
| Review the original team's Skills Probe results | Existing MCP tool `probe_qa` | [Map-Aware Probe QA](docs/system/map-aware-probe-qa.md) |
| Check map-specific requirements **without executing them** | Optional approved map contract + `probe_qa` | [Contract and scope](docs/system/map-aware-probe-qa.md) |
| Compare new team/Probe ZIP snapshots | Explicit offline `team:update:inspect` | [Team source review](docs/system/education-team-source-updates.md) |
| Inspect imported diagnostic evidence | Existing read-only diagnostic MCP tools | [Local evidence](docs/system/map-diagnostics.md) |

The complete current [flow and ownership diagram](docs/system/architecture.md)
separates desktop, native controller, MCP, source intake, Probe QA and
reporting. For individual files use the [implementation map](docs/system/implementation-map.md).

## What runs — and what does not

**Desktop:** Svelte/Tauri sends validated requests to the original Rust
session controller, which owns process lifetime, private storage, and
normal exit. Window presentation/DWM previews are a separate concern.
Minecraft itself owns accounts, rendering, worlds, gameplay and networking.
A process or preview is not evidence of a signed-in playable world.

**MCP:** the optional local stdio bridge reuses that same controller.
Normal tools inspect profiles, explicitly request launches, plan actions,
read bounded evidence, and optionally capture windows. Independent
keyboard/mouse AI input is an **OFF-by-default, not runtime-verified**
experiment and is excluded from standard release builds.

**Skills Probe:** `probe_qa` verifies a pinned original Skills Probe ZIP,
records a stable local capture checkpoint, optionally pins a read-only
native window target and prepares the original admin command. It **does
not** send that command, install a Probe pack or certify a Minecraft world.
On later calls it reads appended Probe log markers; completed results are
frozen until `restart=true`. Even Probe-reported all-PASS means only
that those original probes reported PASS, **not** map gameplay acceptance.

**Map-Aware QA:** an optional local `map-test-contract.json` selects
`full`, `smoke` or `affected` requirements. Without independently
verified map source diffs, `affected` conservatively selects `full`.
The `mapPlan` contains bounded check summaries; the optional
`qaReport` combines the original Probe's reported result with
map test coverage and missing evidence. It is available even when
the original Probe is not configured. Map checks remain NOT_TESTED,
and no second in-game scoring/test engine, game action or automatic
command dispatch was introduced.

**Proof boundary:** ZIP provenance, stable log bytes and PID/HWND cannot
independently establish the actual QA world, Player/admin authority or
Content Log's process/world origin. The source-only `proofGate` marks
those unknowns; command dispatch stays disabled. No exact-revision
Windows/gameplay acceptance or current Node test execution is claimed.


## Related Lazy-family handoff (source coordination only)

[Lazy-Developer](https://github.com/OpenITStudios/M-Lazy-Developer/tree/Local) owns the selected Bedrock/Education `.mcworld` source audit, finding IDs, obligations, and evidence classification. LazyTester owns **independent Minecraft Education runtime/profile observation**, not the source audit or its defect verdicts. This MCP does **not** accept `MapAuditOutputV2` or an AI `--handoff` packet as an executable map-test contract.

For an explicitly approved native investigation, the map owner must separately review a [`map-test-contract.json`](docs/system/map-aware-probe-qa.md) against the exact selected map, including the artifact SHA-256, edition/version, expected outcome, required evidence and relevant source revision. Configure it explicitly with `LAZY_TESTER_MAP_CONTRACT` or `--map-contract` only on an authorized local host. The existing `probe_qa` remains the sole probe interface; `mapPlan` and `qaReport` describe source-planned checks, and **map requirements stay NOT_TESTED** without independent native/world/Player/log evidence. Probe-reported success must not be forwarded as Lazy-Developer gameplay PASS.

[LazyDesigner](https://github.com/OpenITStudios/M-LazyDesigner/tree/Local) separately owns Bedrock Entity `.bbmodel` visuals. LazyTester does not package those assets into worlds or validate them from an export alone. No automatic map/model transfer, startup, test dispatch, credential access, world mutation, or cross-repository write is established. Shared AgentSkill governance and existing source-test/CI holds stay effective.

## Source and private data layout

```text
Lazy Tester.exe    ignored prepared root launcher (when built)
RUN.cmd            compatibility wrapper for that launcher
DEV.cmd            one guarded developer entrypoint
App/               ignored prepared application and reviewed DLL payload
Data/              ignored settings, private profiles and optional diagnostics
Development/
  apps/desktop/    one active Svelte/Tauri desktop
  engine/          portable contracts and distribution
  tooling/         source checks, MCP, controlled development tools
docs/              active guides and historical reference documents
planning/          ONE concise current handoff + archived handoffs
Archive/           old investigations; not a runtime fallback
workspace/         ignored local builds, packages, tools and private material
```

**Use a single checkout root for pre-localtest development.** Approved
`DEV.cmd dev` builds from `workspace/builds/desktop/target/` but
resolves manager/profile state to the sibling `Data/` under this checkout
even before `App/` is staged. Do not create a second Data folder inside
the build target or attempt to merge it into the repo. If a legacy
build-local Data folder is found, startup blocks and preserves it for
manual review. Keep installed NSIS application data separate until
an explicitly approved disposable install/upgrade test.

`App/`, `Data/`, worlds, accounts, private registry hives, local
compiler artifacts and installer candidates are **not Git source**.
Do not move, reset or delete them as part of repository cleanup.
See [data layout](docs/system/data-layout.md),
[artifact retention](docs/system/artifact-storage.md) and
[security boundaries](SECURITY.md). Sandboxie, VMware and Guest Agent
are retired from the active runtime; historical evidence is retained.

## Developer entrypoints (reference only; not executed by repository review)

```text
DEV.cmd localcheck
DEV.cmd inspect --json
DEV.cmd catalog --json
DEV.cmd plan --json
DEV.cmd review --json
DEV.cmd plan --profile local-smoke --json
DEV.cmd smoke start --expect-sha <exact-Local-HEAD-SHA>
DEV.cmd check:quick
DEV.cmd run auto --expect-sha <exact-Local-HEAD-SHA> --json
DEV.cmd dev
DEV.cmd package
```

Developer commands require the reviewed local toolchain, a matching
revision, and explicit authorization for effects. `DEV.cmd dev` and
package preparation require the reviewed native payload configured
through `LAZY_TESTER_NATIVE_PAYLOAD_DIR`. A package candidate is not a
signed or gameplay-accepted release. No local tests, builds, package
operations, game launches or CI are started by a GitHub source review.

## Navigation

- [Product and workspace](docs/product/README.md)
- [System architecture and file ownership](docs/system/README.md)
- [MCP and Probe QA](Development/tooling/agent/README.md)
- [Validation and evidence boundaries](docs/validation/README.md)
- [Single current handoff](planning/development.md)
- [Repository development rules](AGENTS.md) · [GitHub policy](GITHUB_RULES.md)

Historic four-client designs, prior installer experiments and resource
preview research are not live instructions; consult their linked owners
only when specifically needed.
