---
id: document.system.development-operations
class: DOCUMENT
domain: system
role: GUIDE
authority: CANONICAL
lifecycle: ACTIVE
---

# Development Operations

## GitHub-first work

The regular ChatGPT developer path is **REMOTE_GITHUB**:

```text
pin Local HEAD
→ AGENTS.md / GITHUB_RULES.md
→ docs/system/implementation-map.md
→ canonical owner / consumers
→ smallest coherent repository change
→ exact-head static verification / evidence classification
→ commit to Local / STOP
```

A user-PC checkout, npm installation, `DEV.cmd`, VMware or Minecraft execution is not required for source-verifiable tasks. Actual compilations, package builds, Windows guest behavior and gameplay must not be claimed without executed evidence.

## AI-ready single developer entry

The GitHub-first source workflow above remains authoritative for remote
ChatGPT edits. For an actual compatible Windows checkout, `DEV.cmd` now
supports a **single AI-facing contract**:

```text
DEV.cmd inspect --json
DEV.cmd catalog --json
DEV.cmd plan --json
DEV.cmd plan --base-sha <exact-ancestor-SHA> --json
DEV.cmd run frontend.validate --expect-sha <exact-Local-HEAD> --json
DEV.cmd run auto --expect-sha <exact-Local-HEAD> --json
DEV.cmd run auto --expect-sha <exact-Local-HEAD> --base-sha <exact-ancestor-SHA> --json
```

The registry and bounded Node controller live at
[Developer Controller](../../Development/tooling/developer/README.md).
The default planning action reads changed tracked/untracked *source path
metadata*, or the latest committed delta on a clean checkout, and proposes
one documentation, frontend, Rust, or full-source check. `run auto` executes
that **single** safe validation; `plan` itself never starts a task.
For a batch spanning several commits, use an explicitly supplied ancestor
`--base-sha` to include all changed committed paths since that baseline and
any pending working-tree paths. The current exact `Local` HEAD remains required
for execution. This uses the same controller, task registry and source checks,
with no stored last-success marker. An affected-path run is not full candidate
verification; complete the existing full-source, Windows and game acceptance
at the final validation boundary.
Documentation accompanying only frontend or Rust code does not unnecessarily
escalate it to the full suite. Cargo manifests, lockfiles, build scripts,
mixed/native and unknown changes take the full-source route; native DLL
acceptance and Windows/Minecraft tests remain separately unverified.
No path or arbitrary command from user/AI input becomes a shell invocation.

`run` requires an unchanged, exact `Local` commit and an allowlisted
task. A separate explicit operator approval is required before specifying
`--approve <task-id>` for setup, lockfile mutation, full preflight or
candidate packaging. Command success reports **exit status only**, not
gameplay, installer, or signing acceptance. Exclusive local developer-output
tasks are serialized by a fail-closed lock under ignored `workspace/tmp`.
A stale lock must be inspected before any manual recovery; nothing
automatically deletes protected data or forces a game process to close.

The existing `DEV.cmd bootstrap`, underlying `dev.ps1`, pinned toolchain,
package/retention guards and manual commands are retained. The AI `run`
dispatcher uses asynchronous, streamed child execution instead of a blocking
process call. A lock for write-producing tasks stays held through the entire
child lifetime; a second guarded task fails closed. In JSON mode child logs
go to stderr and the final status is the only stdout envelope. `INTERRUPTED`
never qualifies as `COMMAND_SUCCEEDED`. On Windows a Ctrl+C does not trigger
programmatic child.kill of the PowerShell/Tauri tree; a stale lock requires
manual provenance review, never forced cleanup. This is a bounded
orchestration layer, **not** a second build engine, UI, daemon, CI workflow,
or new compiler cache.

## Toolchain authority

Read [toolchain.json](../../Development/toolchain.json) for canonical Node/npm/Rust/PowerShell pins. Desktop owns `Development/apps/desktop/package-lock.json`; desktop Tauri and runtime-core own independent Cargo locks. The root intentionally has **no separate Node dependency graph**.

## Optional source execution

```bash
npm --prefix Development run setup
npm --prefix Development run verify:repository
npm --prefix Development run test
npm --prefix Development run check:desktop
npm --prefix Development run check:frontend
npm --prefix Development run check:rust
npm --prefix Development run check:runtime
npm --prefix Development run test:runtime
npm --prefix Development run format:runtime
npm --prefix Development run verify:source
npm --prefix Development run dev
```

`verify:repository` is a structural policy/consumer check. Desktop `verify:source` runs Svelte typecheck, Vitest source tests, frontend build, Tauri Rust check and Rust desktop tests; `verify:source` at **root** also checks canonical runtime Rust and formatting. Neither automatically starts VMware or Minecraft.

`DEV.cmd bootstrap` is the single opt-in Windows x64 setup entrypoint: it downloads verified portable Node/PowerShell and checksum-verified official Rustup only to `workspace/`. Other `DEV.cmd` commands run the local PowerShell runtime, source `Development/tooling/windows-toolchain/local-env.ps1`, refuse global Node/npm/Rust fallbacks, and use process-scoped caches, Cargo/Rustup homes and temp within `workspace/`. Git, Visual Studio C++/MSVC and WDK still require a system installation; game/driver/service locations are not rewritten. `DEV.cmd` delegates to `Development/tooling/windows-toolchain/dev.ps1` for optional Windows developer tasks. The selected native pilot uses one fail-fast preflight before manual gameplay; the old `g0` packaging route remains separate:

```text
DEV.cmd bootstrap
DEV.cmd setup
DEV.cmd doctor
DEV.cmd preflight
DEV.cmd check
DEV.cmd check:frontend
DEV.cmd check:rust
DEV.cmd bench:frontend
DEV.cmd bench:rust
DEV.cmd test
DEV.cmd dev
DEV.cmd package
```

`doctor` checks developer toolchain only. `preflight` requires a clean
committed `Local` SHA, pinned tool versions and committed lockfiles. Before
full `verify:source`, it checks `package.json` against the committed npm
lockfile with the existing metadata verifier, checks the actual installed
dependency graph using pinned `npm ls --all --offline`, and confirms the
required CLI launchers are present. **Only when the installed dependency
tree is missing or inconsistent** does it run the existing locked
`npm ci` setup. This avoids deleting and reinstalling `node_modules`
on every preflight. A linked or unexpected dependency root is refused,
not overwritten. `DEV.cmd setup` remains the explicit unconditional
clean-install route. Dependency reuse is not a cryptographic content audit:
the full source checks still run, and no test is presumed successful.
`preflight` confirms source cleanliness again afterwards; it does
**not** test Minecraft sign-in, Windows gameplay, VMware/Sandboxie,
installed application acceptance or installer preservation.

## Scoped developer checks and timing

Daily UI/TypeScript changes can use `DEV.cmd check:frontend`, which delegates
to the existing Svelte typecheck and Vitest suites. It **does not** run Rust
checks, Tauri, Vite release build, installer packaging or Minecraft. The full `DEV.cmd check` verification scope remains intact, as does the
`DEV.cmd preflight` source acceptance scope; only redundant npm dependency
reinstallation is avoided when the existing installation passes its
readiness checks.

Rust changes can use `DEV.cmd check:rust`; it delegates to the
desktop `check:rust:target` and standalone `runtime-core` Cargo check
using the explicit Windows MSVC target `x86_64-pc-windows-msvc`, pinned
lockfiles and the existing `CARGO_TARGET_DIR`. This aligns the
**target output layout** with the existing `tauri dev --target` command.
It does not guarantee artifact reuse across different feature sets,
Cargo profiles or build-script configurations and can require a one-time
new compilation. The quick route deliberately skips `--all-targets` and
Rust tests. The full `verify:source` composition is unchanged, while the desktop and
runtime-core Cargo checks and tests now use the explicit Windows MSVC target
consistently. This avoids a second host-default output layout for the supported
Windows developer workflow; different Cargo features/profiles can still
produce separate artifacts.
This check is **not** a Windows game test, DLL build or packaging approval.
No native Tauri feature/config policy is bypassed.

Manual opt-in `DEV.cmd bench:frontend` or `DEV.cmd bench:rust` runs the
corresponding scoped check twice on the **same cache**, after verifying the
pinned portable toolchain and clean `Local` SHA. Stopwatch durations for
pass 1 and pass 2 are printed to the console only; a failed pass aborts,
and source is checked again after each pass. Neither pass is a proven
**cold** or **warm** build: there is no clean, new target directory,
cache deletion, log collection, runner, automation, installer or Minecraft
launch. The second pass may have greater reuse but a faster time is not
guaranteed. Actual cold-build and disk comparisons remain separate
opt-in experiments requiring sufficient disk and new evidence.

Existing `DEV.cmd dev` retains Tauri's frontend hot reload / Rust rebuild
behavior, the reviewed compatibility DLL staging and native identity. The
new scoped routes do not modify game/session/process authority.

## Cargo storage attribution

`DEV.cmd storage cargo` is a no-build, read-only report of the canonical
Cargo `target` and historical source-local targets. It groups logical
file bytes by target layout and profile and will flag coexisting
host-default/explicit-Windows debug output without claiming either is
safe to remove. The portable `DEV.cmd` local environment refuses
noncanonical `CARGO_TARGET_DIR` overrides (and existing links in the
compiler-target ancestors). This **does not** affect direct Cargo shell
commands or any installed product/runtime data.

The scan is bounded, treats errors as `PARTIAL`, counts hardlinked files
as logical entries and makes **no** physical disk savings claim.
Read it alongside `DEV.cmd storage report` and
`DEV.cmd storage plan` before considering any separate,
explicitly authorized cleanup proposal. No automatic pruning is enabled.

## Build-check cache growth evidence

The optional `DEV.cmd bench:rust` samples only the active Cargo target
before the first pass and after each pass, using a compact, bounded,
read-only `--active-json` report from the existing Cargo footprint
scanner. Snapshot time is excluded from the compilation stopwatch.
An incomplete result is `UNKNOWN`; any displayed change is in
**logical file bytes**, not observed physical disk allocation, guaranteed
causal compiler growth or space safe to reclaim. This does not replace
the existing read-only `storage report`, `storage cargo`, and
`storage plan` analyses. No deletion command is enabled.

The portable developer shell now refuses conflicting inherited
`CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR` and
`CARGO_BUILD_BUILD_DIR` values, pinning all three to the approved
`workspace/builds/desktop/target`. Direct Cargo commands outside
`DEV.cmd` do not inherit this enforcement unless the operator sets
up the same environment.

## Package and release boundary

The current product is **Lazy Tester**, with Windows-native independently
managed Minecraft Education profiles. VMware/Sandboxie runtime providers and
the legacy VMware package path are retired and remain blocked. The
`npm --prefix Development run package:windows` command is the dedicated
native Lazy Tester candidate packager; it requires clean committed `Local`
source, reviewed DLL payload, pinned toolchain and its original safety
gates. Its export is **unsigned, unaccepted and not published**. The
`--export-only` option leaves the local `App/` candidate unchanged;
the normal local candidate refresh retains protected `Data/` and
archives the previous `App/` under the protected private backup owner.
After a **successfully completed candidate** (including any local staging
and startup verification), the packager may delete **only older verified
three-file installer exports** beyond the latest two, bounded to eight
candidates per build. Actual installer SHA-256, receipt agreement, ordinary
directories and singly linked files are revalidated before removal.
Unknown/modified files block retention. Partial cleanup is reported
separately and never includes the active Cargo target, historical archive,
toolchains, App/Data or private backups. `storage plan` stays read-only.

Separately, `storage.retire-archive` is an **approval-gated** task which
invokes pinned Cargo `clean --target-dir` only for
`workspace/archive/compiler-cache-20261026/target`. It requires a complete
read-only scan, recognized Cargo layout without links or hardlinks, and a
populated canonical Windows MSVC replacement target. Unknown or partial
archives are preserved. This never runs against the active target and does
not establish physically reclaimed NTFS bytes. No generic cleaner is exposed.

GitHub Actions is **disabled entirely** on `Local` and `main`.
No push, manual, scheduled, PR, package or deployment workflows are allowed
without new explicit operator approval. The existing checks are only run
when manually invoked in an actually available environment. Do not infer
Windows/Minecraft runtime acceptance or release readiness from source
inspection, authored tests, compiler success, a package manifest or an
unsigned candidate. Historic VMware/CI reports remain provenance only.

## Real-machine evidence

Read [Acceptance](../validation/acceptance.md) and [Failure Matrix](../validation/four-client-failure-matrix.md). The privacy-bounded Windows evidence collector lives in `../../tooling/acceptance/windows/collect-acceptance.ps1`. Running the collector does not prove Microsoft sign-in, 3D graphics or multiplayer unless separately observed.

No source edit should be left unfinished merely because real-machine verification is unavailable; mark the exact corresponding claim `NOT_EXECUTED`.

## Before localtest: one-root command protection

Use `DEV.cmd bootstrap` once, then `DEV.cmd doctor`, then
`DEV.cmd preflight`, then `DEV.cmd dev`. The `setup`, `check`,
`test`, and `dev` routes each revalidate pinned local tool versions and
lockfile presence before proceeding; they do not assume that the operator
remembered to run `doctor`. `bootstrap` uses a process-only local
TEMP/TMP before network downloads, and the Rustup sidecar has bounded plain-file
admission. This is a source contract, not completed Windows installed-PC or Minecraft
acceptance. No GitHub Actions runners are active and no package may be
published without separate review.
