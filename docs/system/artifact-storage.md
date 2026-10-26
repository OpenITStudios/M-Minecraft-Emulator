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
`apps/`, `engine/`, and `tooling/`; implementation authority remains in
[Development Operations](./development-operations.md) and
[Distribution](../../engine/distribution/README.md).

REA is a methodological reference for origin, immutable evidence, and explicit
unknowns. We have not run REA against a built emulator or Windows installer
in this change. Do not describe source inspection as REA runtime proof.

## Single-location ownership

| Classification | Canonical path | Tracked? | Rule |
| --- | --- | --- | --- |
| Product code and runtime contracts | `apps/`, `engine/`, `tooling/` | Yes | Keep source and deterministic tests under their actual owners |
| Toolchain declarations and lockfiles | `toolchain.json`, `apps/desktop/package-lock.json` and Rust `Cargo.lock` | Yes | These are immutable inputs, not generated binary caches |
| Node, Vite, Rust, Tauri output | `apps/desktop/node_modules/`, `apps/desktop/dist/`, Rust/Tauri `target/`, Tauri staging | No | Already contained by the one checkout; preserve tool-native paths |
| Portable developer tool installations | `workspace/toolchains/` | No | Opt-in `DEV.cmd bootstrap`: pinned and checksum-verified Node/PowerShell ZIPs, Rustup checksum sidecar and local Rustup home |
| Developer caches and temporary files | `workspace/cache/`, `workspace/tmp/` | No | `DEV.cmd` applies process-scoped npm/NuGet caches and temporary directories; direct shells are not managed |
| External installer and prerequisite downloads | `workspace/downloads/` | No | Record original URL, vendor, version and SHA-256 |
| Native-engine build reports | `workspace/builds/<fresh-run>/` | No | Only new, non-reparse destinations; reports remain inside this checkout |
| Engine staged payloads and future approved packages | `workspace/packages/<fresh-run>/` | No | Fresh path only; staged native engine is NOT-INSTALLABLE, never released or signed by copying |
| Per-attempt local tests | `workspace/runs/` | No | Separate logs, raw records, redacted extracts and provenance |
| Temporary investigation | `workspace/scratch/` | No | Never promote hypotheses into observed results |
| Restricted machine data | Protected user/application paths; `workspace/private/` only when authorized | Never | Git ignore is not a secrecy control |
| Durable, sanitized decisions | `docs/` and `planning/development.md` | Yes | Record only reviewable facts with exact revision |
| Published release | Separately approved distribution channel | No | Requires release gate, signatures and hashes first |

Only `workspace/README.md` may be Git-tracked under `workspace/`.

## Lazy Tester development, candidate and installed data

There is one source tree. `Local` owns product development; `main` is reserved for
separately authorized release work. Neither branch stores built installers or
private runtime data. Do not create root developing/release copies of source.

Root `npm run package:windows` is owned by
`apps/desktop/scripts/package-lazy-tester.mjs`. It requires clean committed Local
source (a detached worktree must match Local exactly), declared Node/npm versions,
and the reviewed native payload. It runs the existing desktop build:app, then
rechecks the source revision/cleanliness before exporting. A build failure or
source change produces no completed export. Relative/external compiler-output
overrides and linked output directories are refused.

Each export is a new ignored
`workspace/packages/lazy-tester/<version>-<12-character-SHA>-<UTC>/` directory.
It contains exactly the renamed installer, SHA256SUMS.txt and package-manifest.json.
The manifest is written last, records the full source SHA and installer digest,
and always marks publication disabled, signing unverified and runtime acceptance
not executed. It is a build receipt, not a second runtime-readiness store.
Existing exports are never overwritten or recursively copied into newer ones.
This command does not sign, upload, install or run the application.

After exporting, the same command stages only the four required runtime payload
files into the ignored root `App/`, with a source/hash receipt. Root `RUN.cmd`
opens this app without a folder picker. It creates an empty `Data/` only when
absent, preserves an existing data locator, and archives the previous `App/`
under `workspace/private/local-app-backups/` before replacement. Running local
apps prevent refresh. No profile capabilities, accounts or worlds are copied.
An empty unconfigured default Data folder can be initialized; nonempty partial
data or an unavailable saved location still blocks fresh replacement.

The installer payload remains governed by
`engine/distribution/lazy-tester-package.json` and the Tauri native configuration;
the export does not add source, tools, profile bindings, accounts or worlds.
Installed copies are independent of this optional checkout-local candidate.
Reorganizing development output never relocates their user data. Product data
locations are owned by product_profiles.rs; cross-volume migration and fresh-PC
Minecraft acceptance remain unfinished.

**Repository-controlled storage contract:** one checkout root is the parent
of every project download, source tree, locally staged payload and
project-generated report. The current owned v1.18.6 builder must accept
`workspace/builds/<new-name>/` as its report directory and
`workspace/packages/<new-name>/` as the optional payload directory;
the assembler enforces those same boundaries. Both require preexisting,
non-junction parent directories and refuse reuse. Tool-owned compiler
outputs inside `engine/isolation-owned/` are ignored and independently
checked for preexisting binaries. The opt-in `DEV.cmd bootstrap` installs pinned portable developer tools
under `workspace/toolchains/` from checksum-verified downloads saved in
`workspace/downloads/`. `DEV.cmd` refuses global runtime fallback and
scopes developer caches/temp to this root without altering OS/user PATH.
Git and VS2022/MSVC/WDK remain system prerequisites; neither game runtime
nor installed Windows services is relocatable by this developer workflow.
These source contracts do not prove that bootstrap executed on target Windows.
Do not create root `artifacts/`, `downloads/`, `installers/`,
or `test-results/` directories. The root is an entry/config boundary,
not a second build or evidence owner.

Existing destinations `workspace/evidence/`, `captures/`, `images/`,
`runtime/` and `diagnostics/` are historical compatibility paths.
**Do not redirect or delete them** without tracing their actual callers
and agreeing on a reversible migration.

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

The **active** experiment is official Native Client 01 plus Sandboxie Plus
Client 02. Packaging with the older VMware NSIS resources remains blocked by
`PACKAGE_BLOCKED_NATIVE_FEASIBILITY`. `DEV.cmd preflight` and `DEV.cmd dev`
are not installer validation. This policy does not authorize lifting that
guard, modifying Windows installed identity, touching `M-Bedrock/VirtualClients`,
or deleting account-bearing guest/sandbox data.

Minecraft Education software, licensed game content, vendor installer binaries,
Microsoft credentials and profiles must never become distributable repository
contents. Third-party installers should originate from approved official
sources; preserve publisher identity and file digest prior to later review.

No automatic cleanup of failed evidence, VM/sandbox contents, rollback
snapshots, user application data or old runtime data. `scratch/` is only
disposable after verifying that the operator does not need it for recovery or
investigation. Any future cleanup command must first dry-run a strictly
allowlisted target, avoid symlink escapes, and require explicit approval.
Keep sensitive raw records in appropriately secured local storage. Publish
only reviewed sanitized excerpts.

## Enforcement and proof ceiling

The existing `npm run verify:repository` checks effective ignore behavior
and rejects unexpected Git-tracked paths under machine-output roots.
The `npm run test:localtest` suite exercises manifest lifecycle using
disposable directories. Windows/PowerShell integration still requires
actual execution. These are **source-layout and unit guards**, not proof
of actual installer behavior,
Windows execution, Minecraft sign-in, GPU performance or REA analysis.
Use [native pilot local-test procedure](../validation/native-pilot-localtest.md)
for actual operator testing when authorized.
