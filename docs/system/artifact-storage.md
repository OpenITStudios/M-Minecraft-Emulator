---
id: document.system.artifact-storage
class: DOCUMENT
domain: system
role: POLICY
authority: CANONICAL
lifecycle: ACTIVE
---

# Artifact and Local-Test Storage Policy

## Scope

Keep the source tree reproducible and readable while retaining meaningful
local-test provenance. This document governs **storage**, not installer behavior,
runtime state, or additional product acceptance gates. Source owners remain
`Development/apps/`, `Development/engine/`, and `Development/tooling/`; implementation authority remains in
[Development Operations](development-operations.md) and
[Distribution](../../Development/engine/distribution/README.md).

REA is a methodological reference for origin, immutable evidence, and explicit
unknowns. We have not run REA against a built emulator or Windows installer
in this change. Do not describe source inspection as REA runtime proof.

## Single-location ownership

| Classification | Canonical path | Tracked? | Rule |
| --- | --- | --- | --- |
| Product code and runtime contracts | `Development/apps/`, `Development/engine/`, `Development/tooling/` | Yes | Keep source and deterministic tests under their actual owners |
| Toolchain declarations and lockfiles | `Development/toolchain.json`, `Development/apps/desktop/package-lock.json` and Rust `Cargo.lock` | Yes | These are immutable inputs, not generated binary caches |
| Tool-native frontend and Tauri staging | `Development/apps/desktop/node_modules/`, `Development/apps/desktop/dist/`, Tauri staging | No | Preserve required tool-native paths; historical source-local Cargo targets are not active roots |
| Active Cargo output and intermediates | `workspace/builds/desktop/target/` | No | One target for DEV.cmd and supported package entrypoint |
| Prepared application and private product data | `App/`, `Data/` | No | See [product data layout](data-layout.md); separate installed root, never a developer cache |
| Portable developer tool installations | `workspace/toolchains/` | No | Opt-in `DEV.cmd bootstrap`: pinned and checksum-verified Node/PowerShell ZIPs, Rustup checksum sidecar and local Rustup home |
| Developer caches and temporary files | `workspace/cache/`, `workspace/tmp/` | No | DEV.cmd and supported packaging use process-scoped paths; unrelated direct tools are not managed |
| External installer and prerequisite downloads | `workspace/downloads/` | No | Record original URL, vendor, version and SHA-256 |
| Native-engine build reports | `workspace/builds/<fresh-run>/` | No | Only new, non-reparse destinations; reports remain inside this checkout |
| Engine staged payloads and future approved packages | `workspace/packages/<fresh-run>/` | No | Fresh path only; staged native engine is NOT-INSTALLABLE, never released or signed by copying |
| Per-attempt local tests | `workspace/runs/` | No | Separate logs, raw records, redacted extracts and provenance |
| Temporary investigation | `workspace/scratch/` | No | Never promote hypotheses into observed results |
| Retained local history | `workspace/archive/` | No | Not active output; see preservation boundaries below |
| Restricted machine data | Protected user/application paths; `workspace/private/` only when authorized | Never | Git ignore is not a secrecy control |
| Durable, sanitized decisions | `docs/` and `planning/development.md` | Yes | Record only reviewable facts with exact revision |
| Published release | Separately approved distribution channel | No | Requires release gate, signatures and hashes first |

Only `workspace/README.md` may be Git-tracked under `workspace/`.

## Read-only developer disk inventory

Use `DEV.cmd storage report` (or `DEV.cmd storage explain`) after the pinned
portable toolchain is present. The command is owned by
`Development/tooling/storage/inventory.mjs` and runs in the existing developer
launcher. It scans **metadata only** under a fixed developer-output allowlist,
with a finite entry/time budget, bounded directory traversal and no junction/
symlink following. Separate categories partition the current Cargo target,
historical compiler archives, registry/download caches, packages, toolchains,
Node output, and the locally prepared `App/`. Source-local legacy Cargo targets
are measured separately, not merged with the active target.

`Data/`, `workspace/private/`, `workspace/runs/`, runtime/VM/profile
material, Git history and user/system stores are outside this inventory.
`ABSENT` means a category directory does not exist; `PARTIAL` means that a
link, inaccessible entry or traversal limit prevented a complete count.
Partial output is a **lower bound**, never a measured full-disk total.
Even complete totals are logical file bytes; compression, sparse files and
hardlinks mean they are **not physical used space or reclaimable savings**.
The inventory does not save raw filenames, delete, move, clear, open file
contents, alter the Cargo target, or modify app/profile data.

Both Cargo roots use a lean `dev`/`test` profile: line-table debug metadata
for workspace code, zero debug information for dependencies, and incremental
compilation retained. `release`, installer identity, native DLL hashes and
runtime settings remain unchanged. The first profile change can cause a
one-time recompilation; old artifacts remain until separately reviewed.

## Cargo build-cache attribution (read-only)

`DEV.cmd storage cargo` uses
`Development/tooling/storage/cargo-footprint.mjs` to inspect **only** the
canonical `workspace/builds/desktop/target`, two historical
source-local Cargo `target` directories and the fixed retired compiler archive target. It groups regular-file logical
bytes by Windows MSVC versus host-default output, debug/release profile,
and Cargo subdirectories including `deps`, `incremental`, `build`,
`.fingerprint`, `examples`, and `bundle`. Unknown Cargo layouts remain
counted under `other` rather than being dropped.

The scan reads directory metadata only, has finite time/entry/depth
budgets, refuses linked ancestors and does not follow symlinks or
junctions. Hardlinks are counted as logical file entries and explicitly
reported; totals **cannot** be treated as NTFS allocated bytes,
distinct physical data, duplicates, or recoverable savings. A partial
scan is a lower bound and cannot authorize deletion. The tool never
touches `App/`, `Data/`, backups, game data or installed services.

The supported build and package routes use the [canonical output contract](#lazy-tester-development-candidate-and-installed-data).
This optional report does not launch Cargo, compile Rust, relocate caches
or produce additional log files. It does not override package release gates.

Cargo's [build-cache documentation](https://doc.rust-lang.org/cargo/reference/build-cache.html)
notes that internal output layout can change and that
`incremental` is reused for compilation. A nonempty `target/debug`
alongside `target/x86_64-pc-windows-msvc/debug` only indicates two
output layouts, **not** proof that either tree is obsolete. Keep the
existing `storage plan` command read-only; no mutation is approved.

## Build evidence with bounded per-pass Cargo snapshots

The optional `DEV.cmd bench:rust` path now samples **only the active Cargo
target** before the first check and after each of the existing two check passes,
using `cargo-footprint.mjs --active-json`. Snapshot scanning happens **outside**
the compilation stopwatch and has a 15-second/250,000-entry limit. The compact
JSON schema reports only aggregate logical file bytes, file/hardlink counts,
entry count and completion status. It never stores names, arbitrary paths,
account material, telemetry or a persistent log. Missing or incomplete scans
yield `UNKNOWN`, never a claimed zero or positive saving.

A valid pair of snapshots prints the observed *logical byte change* around
each pass. That delta is **not** attributable compiler cache saving,
allocated/free NTFS bytes, verified duplicate content, or proof that a cache
is removable: hardlinks, external writes and Cargo's build reuse can affect
the result. A failed check still fails the benchmark; the extra sampling
neither builds a new target nor removes old artifacts. The frontend benchmark
does not scan Cargo.

The standalone scanner is `node Development/tooling/storage/cargo-footprint.mjs --active-json`.
See the [canonical output contract](#lazy-tester-development-candidate-and-installed-data)
for Cargo target/intermediate path admission.

## Retention review and verified post-build export turnover

The `DEV.cmd storage plan` command remains a **read-only** report based on
`Development/tooling/storage/retention.mjs`. Its historical review default
protects the newest **three** valid candidate exports and all candidates
younger than **30 days**; older exports receive `REVIEW_ONLY`, not automatic
deletion permission. The archived
`workspace/archive/compiler-cache-20261026` remains `REVIEW_ONLY`
regardless of size. No direct CLI `--apply` or generic deletion mode exists.

The now-authorized **post-success package turnover** is a separate, bounded
internal operation of the existing native packager:
`Development/tooling/storage/package-recycle.mjs`. Only after a completed
new candidate export and successful local App/startup checks (when selected),
it requests the latest-two/no-age **candidate-only** retention plan. It may
remove at most **eight** superseded, complete three-file installer export
directories per build beneath `workspace/packages/lazy-tester/`. Before
deleting anything, it validates the current export and every selected old
candidate: exact name, version/SHA/UTC receipt identity, the three expected
ordinary files, length-bounded manifest and checksums, actual streamed
installer SHA-256, no extra contents, no links/hardlinks, and stable filesystem
identities. Each deletion rechecks the directory and exact file metadata;
only these three files and their empty parent candidate directory are removed.

Any unknown candidate, corrupt receipt, mismatched installer bytes, linked
path, incomplete directory scan, invalid current package or future-dated
current export **blocks** turnover. Incomplete deletion reports `PARTIAL`;
it never claims all older artifacts were removed. A blocked cleanup does not
turn the already successful package into a false build failure. The process
does **not** delete `workspace/builds/desktop/target`, Cargo incremental
artifacts, toolchains/registry, old compiler archives, `App/`, `Data/`,
`workspace/private/`, evidence, backup folders, worlds or installed data.
The active Cargo target is reused to preserve the incremental-build benefit.
This is intentional: stable Cargo's global dependency GC does not
garbage-collect Rust target artifacts by project version. See
[Cargo build cache](https://doc.rust-lang.org/cargo/reference/build-cache.html),
[Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)
and [Cargo clean](https://doc.rust-lang.org/cargo/commands/cargo-clean.html).

These source policies do not establish real Windows NTFS allocation savings.
Installer export byte counts and Cargo inventory are logical sizes, and a
hardlinked or compressed file may use different physical disk allocation.
Historical caches and App backups require a separately approved, evidence-led
review before any broader cleanup.

### Historical compiler target retirement (separate approval)

`DEV.cmd run storage.retire-archive --expect-sha <Local-SHA> --approve
storage.retire-archive` is a separately approved one-time operation, **not**
automatic maintenance during everyday build/check. Its only possible
compiler-clean destination is the fixed **old archived** Cargo target:
`workspace/archive/compiler-cache-20261026/target`.

The `archive-retire.mjs` owner first requires an ordinary archive parent
containing only `target`, a recognized Cargo output layout, and a
complete metadata scan without symlinks, junctions, hardlinks or unknown
entries. The canonical replacement target
`workspace/builds/desktop/target` must separately have a complete scan
and nonempty Windows MSVC dependency output. If any admission fails,
no cargo clean is invoked. An approved run calls only the pinned Cargo binary
with the explicit `clean --locked --manifest-path ... --target-dir
<archived target>` arguments and sets all Cargo build/output paths to that
same retired archive. This does **not** invoke unscoped `cargo clean` on
the live developer target, and it does not clean other archive folders,
toolchains, registry, `App/`, `Data/`, private backups, or Minecraft worlds.

If Cargo reports failure or the archived target remains nonempty, the outcome
is `FAILED` or `PARTIAL`, never false success. A completed scoped clean is
not proof of actual physical NTFS bytes reclaimed. The real archive layout,
local Windows toolchain state and saved data must be observed before assuming
this approved task will be eligible on the user's machine. Normal source
builds retain incremental compilation and do not call this operation.

## Lazy Tester development, candidate and installed data

There is one source tree. `Local` owns product development; `main` is reserved for
separately authorized release work. Neither branch stores built installers or
private runtime data. Do not create root developing/release copies of source.

Root `npm --prefix Development run package:windows` is owned by
`Development/apps/desktop/scripts/package-lazy-tester.mjs`. It requires clean committed Local
source (a detached worktree must match Local exactly), declared Node/npm versions,
and the reviewed native payload. It runs the existing desktop build:app, then
rechecks the source revision/cleanliness before exporting. A build failure or
source change produces no completed export. Both DEV.cmd and direct
`package:windows` use the same `workspace/builds/desktop/target` for
`CARGO_TARGET_DIR`, `CARGO_BUILD_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR`;
any conflicting inherited value (including another in-checkout target)
is refused before cache initialization. The packaging process and its children
use `workspace/cache/npm`, `workspace/cache/nuget` and `workspace/tmp`.
Existing directories and ancestors must be ordinary, not links or junctions.
This does not relocate toolchains, user/app data or old outputs; the npm
process that starts packaging and unrelated direct tool invocations are
outside this process-local scope. Use DEV.cmd for the full portable environment.

Each export is a new ignored
`workspace/packages/lazy-tester/<version>-<12-character-SHA>-<UTC>/` directory.
It contains exactly the renamed installer, SHA256SUMS.txt and package-manifest.json.
The manifest is written last, records the full source SHA and installer digest,
and always marks publication disabled, signing unverified and runtime acceptance
not executed. It is a build receipt, not a second runtime-readiness store.
Existing exports are never overwritten or recursively copied into newer ones.
This command does not sign, upload or install the exported installer.

With `--export-only`, export stops before changing App or Data or running
the app. Otherwise, the same command stages the allowlisted runtime payload
and role aliases into the ignored root `App/`, with a source/hash receipt,
and performs the existing separately gated local desktop startup check. Root `RUN.cmd`
opens this app without a folder picker. It creates an empty `Data/` only when
absent, preserves an existing data locator, and archives the previous `App/`
under `workspace/private/local-app-backups/` before replacement. Running local
apps prevent refresh. No profile capabilities, accounts or worlds are copied.
An empty unconfigured default Data folder can be initialized; nonempty partial
data or an unavailable saved location still blocks fresh replacement.

The installer payload remains governed by
`Development/engine/distribution/lazy-tester-package.json` and the Tauri native configuration;
the export does not add source, tools, profile bindings, accounts or worlds.
Installed copies are independent of this optional checkout-local candidate.
Reorganizing development output never relocates their user data. Product data
locations are owned by product_profiles.rs; cross-volume migration and fresh-PC
Minecraft acceptance remain unfinished.

## Preserved local history

One checkout root owns project-generated files; OS installations and explicit
user exports remain separate. The historical owned v1.18.6 builder and assembler
are retired, not active output producers. See [legacy retirement](legacy-retirement.md).
The opt-in DEV.cmd bootstrap stores verified downloads in workspace/downloads
and pinned portable tools in workspace/toolchains. See [development operations](development-operations.md)
for setup and host prerequisites. No system toolchain or game data relocation.
Do not create root artifacts/, downloads/, installers/ or test-results/ as second output roots.

Existing workspace/evidence/, captures/, images/, runtime/ and diagnostics/
are historical compatibility paths. The archived compiler cache and
archive/layout-20261026 checker tree, loose logs, scripts, screenshots and
opaque probe directories remain preserved; their current contents and
consumers are not established by source review. No blanket deletion,
migration or reclaimable-space claim is authorized.

## Session storage contract (for future real tests)

Each attempt should use a distinct UTC timestamp and the tested Git revision,
for example `workspace/runs/20261013T050000Z-9c309670-preflight/`.
Do not overwrite failed evidence with newer output.

```text
workspace/runs/<UTC>-<shortSHA>-<scenario>/
  manifest.json
  logs/
  raw/
  sanitized/
```

The current `DEV.cmd preflight` creates a **metadata-only** `manifest.json`
after Windows, branch, pinned toolchain and lockfile checks succeed. It
records exact commit, UTC timestamps, source-only result and failing phase.
No console output, account information or host fingerprint is captured
automatically. Earlier failures may leave no report. An interrupted
`RUNNING` record is **not a PASS**. Finalizing a run now requires an exclusive
local lock and a matching manifest/folder source identity. A stale lock is
reported as a blocked finalization, never automatically overwritten. This manifest is not an additional
product acceptance gate or remote telemetry store.

Future manually curated evidence may use these additional fields: schema version, complete 40-character
source commit, branch, UTC start/end, scenario, executed command/manual steps,
relevant environment/tool versions, evidence filenames and SHA-256 hashes,
package SHA-256 if applicable, actual exit status, explicit operator-only
observations, limitations, unknowns, redaction status.

The short SHA in the folder name is **not** sufficient provenance.
Use `NOT_EXECUTED` when a test did not run. Never infer Minecraft login,
playability, installed-package integrity, or four concurrent clients from
successful compilation, process counts or authored test cases.

REA-aligned interpretation:
- **Observed:** source or runtime facts actually inspected, tied to exact input.
- **Inferred:** an explanation or hypothesis, clearly distinguished from fact.
- **Unknown:** missing, conflicting, partial or unexecuted evidence.

Only an actual REA invocation can produce REA-native evidence identifiers.

## Installer, protected runtime and cleanup boundaries

The active product is Lazy Tester with independently managed Windows-native
Minecraft Education profiles; Sandboxie and VMware runtime paths are retired.
The old VMware/legacy installer remains blocked. `DEV.cmd preflight` and
`DEV.cmd dev` are not installer validation. This policy does not authorize lifting that
guard, modifying Windows installed identity, touching `M-Bedrock/VirtualClients`,
or deleting account-bearing guest/sandbox data.

Minecraft Education software, licensed game content, vendor installer binaries,
Microsoft credentials and profiles must never become distributable repository
contents. Third-party installers should originate from approved official
sources; preserve publisher identity and file digest prior to later review.

No automatic cleanup of failed evidence, VM/sandbox contents, rollback
snapshots, user application data or old runtime data. The tightly scoped
post-success installer export turnover above is the only authorized exception
for generated candidate copies. `scratch/` is only
disposable after verifying that the operator does not need it for recovery or
investigation. Any future cleanup command must first dry-run a strictly
allowlisted target, avoid symlink escapes, and require explicit approval.
Keep sensitive raw records in appropriately secured local storage. Publish
only reviewed sanitized excerpts.

## Enforcement and proof ceiling

The existing `npm --prefix Development run verify:repository` checks effective ignore behavior
and rejects unexpected Git-tracked paths under machine-output roots.
The `npm --prefix Development run test:localtest` suite exercises manifest lifecycle using
disposable directories. Windows/PowerShell integration still requires
actual execution. These are **source-layout and unit guards**, not proof
of actual installer behavior,
Windows execution, Minecraft sign-in, GPU performance or REA analysis.
Use [native pilot local-test procedure](../validation/native-pilot-localtest.md)
for actual operator testing when authorized.
