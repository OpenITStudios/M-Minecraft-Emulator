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
DEV.cmd run auto --expect-sha <40-char-Local-SHA> --json
```

`inspect` reports the Git revision, changed file counts/scopes and *presence*
of portable tools, not version or gameplay proof. `catalog` lists task IDs,
risk and approval policy without running a task. `plan` recommends the
smallest safe check from changed paths; on a clean checkout, it examines
the newest commit versus its parent. Unknown, native and mixed sources
escalate to full verification. Planning never executes a recommendation.
`run auto` runs **only one** recommended validation task and skips execution
when no changes exist. It does not install dependencies, generate lockfiles,
package releases, delete files directly, or bypass consent-gated operations.
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
Read-only tasks and `inspect/catalog/plan` do not acquire that lock.

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

- `controller.mjs`: read-only inspection/planning and guarded task execution.
- `tasks.json`: one canonical task catalog; command vocabulary is statically
  allowlisted in the controller, not dynamically executable strings.
- `windows-toolchain/dev.ps1`: unchanged backend for installed toolchain,
  checks, benchmark, Tauri development, and package staging.
- `storage/`: existing read-only inventory, Cargo inspection and retention
  planning, plus strictly bounded **post-success installer export turnover**
  invoked by the existing package owner. No generic cleanup task exists.
- `controller.test.mjs`: registry/plan/argument/safety contracts.

Only `Local` is authorized for task execution. GitHub Actions remain fully
disabled. No database, daemon, UI, second Cargo target, new package format,
new test runner, account access, or generic developer cleanup command is added.
The package owner's authenticated export cleanup is described in the
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
