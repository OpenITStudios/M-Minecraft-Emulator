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
DEV.cmd plan --json --paths Development/apps/desktop/src/LazyTesterApp.svelte
DEV.cmd run frontend.validate --expect-sha <40-char-Local-SHA> --json
```

`inspect` reports the Git revision, changed file counts/scopes and *presence*
of portable tools, not version or gameplay proof. `catalog` lists task IDs,
risk and approval policy without running a task. `plan` recommends the
smallest safe check from changed paths; unknown, native and mixed sources
escalate to the full source check. It never runs that recommendation.
`run` executes **one** allowlisted task using an existing executor and
requires a current exact Local SHA. Existing `DEV.cmd bootstrap` and legacy
manual routes are retained unchanged.

High-impact tasks (`environment.setup`, `source.preflight`, `cargo.lock`,
`build.candidate`) additionally require `--approve <exact-task-id>` **after
explicit operator approval**. AI-generated approval arguments are not a
substitute for that consent. No arbitrary command, shell expansion, plugin,
cleanup, release, workflow, or Minecraft launch feature is available.
`build.candidate` still invokes the existing unsigned native-candidate
packager and its own clean-revision and data-preservation gates.

One exclusive file under ignored `workspace/tmp/` serializes task invocations
that can write developer output. A stale/unsafe lock **blocks** the next job:
never silently remove it or kill an unrelated process. Read-only tasks and
`inspect/catalog/plan` do not acquire that lock. The controller does not
capture command output to files. In `run --json`, child output goes to stderr
and stdout contains the final exit-status envelope. A successful command is
`COMMAND_SUCCEEDED`, **not** installed-app/Minecraft acceptance.

This interface requires the pinned portable Node 24.22.0 **executable
path**, not only a matching version number. Direct execution via a global
`node` binary is refused; use `DEV.cmd`. `run` tasks that
use the legacy script additionally require the local PowerShell executable.
Git remains an external developer prerequisite. On a GitHub-only ChatGPT
session, source changes can be committed remotely, but no local `DEV.cmd`
execution is implied.

## Ownership & safety

- `controller.mjs`: read-only inspection/planning and guarded task execution.
- `tasks.json`: one canonical task catalog; command vocabulary is statically
  allowlisted in the controller, not dynamically executable strings.
- `windows-toolchain/dev.ps1`: unchanged backend for installed toolchain,
  checks, benchmark, Tauri development, and package staging.
- `storage/`: existing read-only inventory, Cargo inspection and retention
  planning; no deletion or cache-clearing task is registered.
- `controller.test.mjs`: registry/plan/argument/safety contracts.

Only `Local` is authorized for task execution. GitHub Actions remain fully
disabled. No database, daemon, UI, second Cargo target, new package format,
new test runner, account access, or developer cleanup procedure is added.

## Error model

`READY` and `BLOCKED` apply to read-only inspection/planning; `run`
returns `COMMAND_SUCCEEDED` only after **both** a successful executor exit and
a post-execution `Local` HEAD verification. A changed HEAD yields
`SOURCE_CHANGED`; unavailable final Git evidence yields
`SOURCE_UNVERIFIED`. Exit success is labeled `COMMAND_EXIT_AND_HEAD_ONLY`:
uncommitted source edits and Windows/Minecraft acceptance are not proven.
If a task might have run but its completion or lock release cannot be
verified, the controller returns **one** `FAILED_UNVERIFIED` envelope,
not a contradictory `BLOCKED` result claiming nothing started. Exit codes: 0 successful, 1 invoked task
failed, 2 invalid CLI, 3 blocked/unsafe state. Source tests do not establish
Windows PowerShell, Tauri, packaged-app or Minecraft Education acceptance.
