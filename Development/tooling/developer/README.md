# AI Developer Controller (source-only)

`DEV.cmd` is the only user/AI entrypoint. This owner provides one reviewed
Task Registry (`tasks.json`) and one Node.js controller (`controller.mjs`).
It **does not** replace the existing `windows-toolchain/dev.ps1`, Cargo,
npm, Tauri, the native DLL, or the Lazy Tester runtime.

## Contract

```text
DEV.cmd inspect --json
DEV.cmd catalog --json
DEV.cmd plan --json
DEV.cmd review --json
DEV.cmd review --base-sha <40-char-ancestor-SHA> --json
DEV.cmd review --json --paths Development/tooling/agent/check-education-native-hud-source.mjs
DEV.cmd plan --profile local-smoke --json
DEV.cmd smoke start --expect-sha <40-char-Local-SHA> --json
DEV.cmd smoke record --run <smoke-run-id> --case LT-01 --result PASS --observed --expect-sha <40-char-Local-SHA> --json
DEV.cmd smoke status --run <smoke-run-id> --json
DEV.cmd plan --profile five-client --json
DEV.cmd plan --profile release --json
DEV.cmd plan --profile experimental --json
DEV.cmd plan --json --paths Development/apps/desktop/src/LazyTesterApp.svelte
DEV.cmd plan --base-sha <40-char-ancestor-SHA> --json
DEV.cmd run frontend.validate --expect-sha <40-char-Local-SHA> --json
DEV.cmd run auto --expect-sha <40-char-Local-SHA> --json
DEV.cmd run auto --expect-sha <40-char-Local-SHA> --base-sha <40-char-ancestor-SHA> --json
```


The read-only `review` operation is an evidence-grounded improvement plan,
not a second agent harness or test runner. It uses the same exact Local source
identity and changed-file selection as `plan`: pending edits, the latest
commit on clean checkout, a verified ancestor `--base-sha`, or explicitly
declared `--paths`. Caller-declared paths are marked **not Git-verified**.
It refuses protected, repeated, absolute and traversal paths and cannot run commands,
start agents, install REA, modify Minecraft, or mark any test PASS.

`review` returns a bounded list of canonical owner areas and concrete risk
checks. The HUD source checker, its authored regression fixture, the optional
Resource Pack files and the canonical HUD guide deliberately map to the same
`education-hud` review area, even when only the test or guide changed.
A docs-only HUD review still recommends the existing documentation validation
task; a fixture/source change still recommends the existing source validation
task. Neither recommendation means the task ran.

For critical MCP/native/UI source boundaries, the same read-only result also
includes `crossModuleReviews`. Each source-selected seam lists the affected
participants, **other participants that were not selected**, exact canonical
files to inspect, and risk checks. The reviewed interfaces are:
`agent-input-lease` (MCP ↔ original Rust controller),
`native-desktop-session` (Rust ↔ typed Svelte/Tauri UI), and
`agent-activity-mirror` (MCP → Rust → UI). A single changed file can
require reviewing other owners even when those files are absent from the
selected diff; this absence never proves compatibility. Every participant
remains `NOT_REVIEWED` and each seam reports
`SOURCE_PATH_SELECTION_ONLY`/`NOT_EXECUTED`. There is **no automatic
caller traversal**, runtime control, new source test executor, or PASS.

For the **compile-OFF native input ABI v4** bridge, the same review
also emits `native-input-v4` whenever a participating MCP scheduler/
CLI file, original Rust session/writer, or optional C++ Raw Input
consumer changes. It lists the exact unselected counterpart owners
for 96-byte shared memory, 15 byte offsets, eight rising-edge
counters, 500-ms TTL, 350-ms pulse, 1-ms neutral, and the
P39–P43 expiry/deferred key-up/cursor paths. The C++ source and
header are mapped to the `native-input-v4` area rather than
generic developer tooling. The result is a **source review
obligation**, not a dynamic ABI checker, compiler, executed
negative test, native process or gameplay verification.

The review emits three **NOT_EXECUTED** stages: Propose (alternatives), Probe
(counterexamples/negative cases), Compose (select only source-supported change).
It also exposes the existing recommended validation tasks as **recommendations
only**. Harness² supplies the review pattern; DeepSeek Harness suggests
delegated reviewer roles; REA remains optional developer-side investigation
of authorized artifact copies. No external library is installed or executed.
See [development discipline](../../../docs/system/development-discipline.md).

**Choose a profile explicitly for the first Windows session.** The default
`plan` (equivalent to `--profile fast`) is only a changed-source recommendation,
which on a clean checkout looks at the last commit. It is not an initial
end-to-end baseline: a docs-only latest commit may legitimately select only
repository structure. `--profile local-smoke` instead always recommends the
new `source.quick` task plus thirteen human-operated checks regardless of the
commit diff. `five-client` requires prior same-SHA two-client acceptance;
`release` separately proposes fully approved preflight and package tasks;
`experimental` stays manual and compile-OFF. Non-fast profiles refuse
`--paths` and `--base-sha`. `plan` does not execute any task or game.
Use the [efficient local-testing runbook](../../../docs/validation/efficient-local-testing.md)
for the actual LT/FC/RL/EX cases.

`source.quick` runs structure/locked-dependency checks, repository fixtures,
Svelte typecheck and frontend tests, plus desktop/runtime Rust compile checks
on a clean pinned Local revision. After separately approved toolchain setup:
`DEV.cmd check:quick` or `DEV.cmd run source.quick --expect-sha <SHA>`.
This also writes a **metadata-only exact-SHA quick receipt** via the
existing `Development/tooling/local-test/session.mjs` owner under ignored
`workspace/runs`: command outcome, first stage, elapsed wall time,
`quickSourceScreen` and unchanged `sourceVerification=NOT_EXECUTED`,
`windowsGameplay=NOT_EXECUTED`. It stores no console logs or identities.
A successful quick receipt is not full preflight or Windows evidence.

`--profile local-smoke` now returns each ordered checkpoint with action,
objective pass condition, requested evidence family, original STD case
references, and fail-fast policy. No step is marked executed by a plan
response. This is intended to avoid looking up each of 13 IDs individually.

The `smoke` operations are optional, explicit **operator-reported evidence**,
not a game tester: `start` creates one small private manifest under ignored
`workspace/runs`, `record` accepts only the next LT-01..LT-13 result,
and `status` shows the first blocker/next checkpoint. Record `PASS`
only with the explicit `--observed` flag after personally observing
the scenario. Result values are `PASS`, `FAIL`, `BLOCKED`, and
`INCONCLUSIVE`; there is no free-text account/world/token field.
A FAIL/BLOCKED/INCONCLUSIVE stops this run; reopen a new run after
a fix. Recorded PASS is **never** independently authenticated gameplay
or full source proof, even for thirteen reported PASS cases. Exact Local
SHA and clean tracked/untracked source are required for start/record;
a run from another HEAD is stale/read-only. Case receipt writes are
exclusive and scoped only to that private directory.

This does **not** run all Rust tests, packaging, Minecraft or a full
`source.preflight`; do not promote its exit code into release readiness.
For a fresh clone, locked npm dependency setup is a separately authorized
`DEV.cmd setup` action. The normal `run auto` stays unchanged and cannot
run manual profile plans or request approval on your behalf.

`inspect` reports the Git revision, changed file counts/scopes and *presence*
of portable tools, not version or gameplay proof. `catalog` lists task IDs,
risk and approval policy without running a task. `plan` recommends the
smallest safe check from changed paths; on a clean checkout, it examines
the newest commit versus its parent. For a multi-commit batch, the optional
`--base-sha <exact-40-character-ancestor>` includes all net source changes
from that ancestor to `Local` HEAD **plus current tracked/untracked edits**.
A missing/non-ancestor base refuses the plan; the SHA cannot select another
branch, process, command or release. `--base-sha` is allowed only for `plan`
and `run auto`, never an explicit named task or `plan --paths`.
Markdown under `docs/`, `planning/`, and `Archive/`, along with agent
`SKILL.md`, nested `README.md` and `AGENTS.md`, takes the existing
repository-structure check even when stored near native source. Executable or
unknown extensions under those folders still escalate to full verification.
Build manifests/locks, native source and mixed/unknown changes remain guarded.
Planning never executes a recommendation. Scoped checks are iteration evidence only;
final candidate verification must still use the existing full source/Windows
gates.
`run auto` runs **only one** recommended validation task and skips execution
when no changes exist. It does not install dependencies, generate lockfiles,
package releases, delete files directly, or bypass consent-gated operations.
`run` executes **one** allowlisted task using an existing executor and
requires a current exact Local SHA. Existing `DEV.cmd bootstrap` and legacy
manual routes are retained unchanged.

High-impact tasks (`environment.setup`, `source.preflight`, `cargo.lock`,
`build.candidate`, and `storage.retire-archive`) additionally require
`--approve <exact-task-id>` **after explicit operator approval**. AI-generated approval arguments are not a
substitute for that consent. No arbitrary command, shell expansion, plugin,
cleanup, release, workflow, or Minecraft launch feature is available.
`build.candidate` still invokes the existing unsigned native-candidate
packager and its own clean-revision and data-preservation gates.

One exclusive file under ignored `workspace/tmp/` serializes task invocations
that can write developer output. The lock is held **until the registered child
process closes**, including a long-running Tauri development session. Child
output is streamed instead of buffered; in `run --json`, stdout remains
reserved for **one final result envelope** and child output goes to stderr.
The Node controller remains responsive while the child is running.

`Ctrl+C` is observed as an interruption. On Windows, the controller does
**not** call `child.kill()`: Node's simulated POSIX signals would abruptly
terminate the child and might abandon a separate Tauri process. It waits for
the console-managed developer child to exit. On other hosts, programmatic
termination is scoped to the directly owned developer child only, never to
a game process tree. The final result can be `INTERRUPTED`, not `PASS`.

A stale/unsafe/replaced lock **blocks** the next job; it is never taken over,
silently removed, or used as permission to kill another process. A normal
task failure releases only this controller's verified lock. A sudden process
termination can leave a stale lock requiring separate manual inspection.
Read-only tasks and `inspect/catalog/plan/review` do not acquire that lock.

A successful command is `COMMAND_SUCCEEDED`, **not**
installed-app/Minecraft acceptance.

This interface requires the pinned portable Node 24.22.0 **executable
path**, not only a matching version number. Direct execution via a global
`node` binary is refused; use `DEV.cmd`. `run` tasks that
use the legacy script additionally require the local PowerShell executable.
Git remains an external developer prerequisite. On a GitHub-only ChatGPT
session, source changes can be committed remotely, but no local `DEV.cmd`
execution is implied.

## Ownership & safety

- `controller.mjs`: read-only inspection/planning/quality-review contracts and guarded task execution.
- `tasks.json`: one canonical task catalog; command vocabulary is statically
  allowlisted in the controller, not dynamically executable strings.
- `windows-toolchain/dev.ps1`: unchanged backend for installed toolchain,
  checks, benchmark, Tauri development, and package staging.
- `storage/`: existing read-only inventory, Cargo inspection and retention
  planning, plus strictly bounded **post-success installer export turnover**
  invoked by the existing package owner. No generic cleanup task exists.
- `storage.retire-archive`: separately approved, one-time cleanup of the
  exact **historical** compiler target using pinned `cargo clean --target-dir`
  after complete metadata validation and presence of a nonempty canonical
  Windows MSVC target. It never touches the active build cache, other
  archives, toolchains, or protected product data. Incomplete scans, unknown
  entries, links, and hardlinks refuse this operation.
- `controller.test.mjs`: registry/plan/argument/safety contracts.

Only `Local` is authorized for task execution. GitHub Actions remain fully
disabled. No database, daemon, UI, second Cargo target, new package format,
new test runner, account access, or generic developer cleanup command is added.
The package owner's authenticated export cleanup and the separately approved
retirement of the verified historical compiler target are described in the
[storage policy](../../../docs/system/artifact-storage.md).

## Error model

`READY` and `BLOCKED` apply to read-only inspection/planning; `run`
returns `COMMAND_SUCCEEDED` only after **both** a successful executor exit and
a post-execution `Local` HEAD verification. A changed HEAD yields
`SOURCE_CHANGED`; unavailable final Git evidence yields
`SOURCE_UNVERIFIED`. A canceled developer child is `INTERRUPTED`. Exit success is labeled `COMMAND_EXIT_AND_HEAD_ONLY`:
uncommitted source edits and Windows/Minecraft acceptance are not proven.
If a task might have run but its completion or lock release cannot be
verified, the controller returns **one** `FAILED_UNVERIFIED` envelope,
not a contradictory `BLOCKED` result claiming nothing started. Exit codes: 0 successful, 1 invoked task
failed, 2 invalid CLI, 3 blocked/unsafe state. Source tests do not establish
Windows PowerShell, Tauri, packaged-app or Minecraft Education acceptance.
