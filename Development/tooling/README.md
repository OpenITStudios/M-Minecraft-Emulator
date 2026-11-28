# Developer tooling

The only active product is the Lazy Tester native Minecraft Education manager.
This directory contains source/structure checks, package preparation, Windows
native compatibility source, and retained historical evidence. Nothing here is
a second live game process/session owner.

| Path | Status |
| --- | --- |
| `repository/` | Active Git-tree, dependency, route, CI-disabled and documentation-link checks |
| `windows-toolchain/` | Source/build/package helpers, targeted frontend/Rust checks and opt-in two-pass timing; no automatic local test |
| [`developer/`](developer/README.md) | AI-facing `DEV.cmd` registry/inspection/planning and guarded dispatch; existing scripts remain execution owners |
| [`agent/`](agent/README.md) | Local stdio MCP, read-only visual planning, and OFF-by-default per-client native input candidate; not verified Minecraft gameplay |
| `storage/` | Read-only artifact inventory plus fail-closed retention planning for reviewed developer outputs; no deletion |
| `dependencies/` | Dependency consistency review and maintenance |
| `acceptance/windows/bootstrap-ordering/` | **Active native DLL source** used by the five-session game bootstrap |
| `acceptance/` | Native and historical acceptance collectors; older Sandboxie/VM results are not current gameplay proof |
| [`lazytester-particles/`](lazytester-particles/README.md) | Optional Particle Tuning v0.3 offline .mcpack builder; no automatic installation |
| [`lazytester-textures/`](lazytester-textures/README.md) | Optional block-only Texture Lite v0.1 offline .mcpack builder; entity/PBR extension retired |
| `acceptance/macos/lifecycle-fixture.swift` | Explicit synthetic AppKit launch/identity/normal-exit fixture; never launches Minecraft |
| `acceptance/macos/education-target.mjs` | Explicit offline Mac bundle metadata collector; no native launch or runtime admission |
| `acceptance/windows/education-target.ps1` | **Active** registered Minecraft Education executable resolver, moved from the retired isolation-source tree |
| `local-test/` | Optional source acceptance bookkeeping, not runtime evidence until executed |

Retired Sandboxie/VMware build and runtime source has left the active tree. The native process and per-profile controller remain the only active Minecraft launch authority. Historical notes remain in Git revisions.

## Developer disk inventory (read-only)

Use `DEV.cmd storage report` or `DEV.cmd storage explain` after bootstrap.
The Node CLI is `node tooling/storage/inventory.mjs report` from inside
`Development/`. It reports disjoint logical-byte categories with
`ABSENT`/`PARTIAL` markers and scans no profile/world/runtime data.
It does not delete or compact caches; first-time Rust profile changes may
temporarily increase target size before any separate retention review.
Synthetic tests: `node --test tooling/storage/inventory.test.mjs`.

`DEV.cmd storage plan` runs `storage/retention.mjs`, a second **read-only**
metadata review. It checks only the historical compiler-cache archive and
versioned Lazy Tester export candidates, preserving the newest three and
anything under 30 days. `REVIEW_ONLY` is never deletion permission;
malformed receipts and incomplete scans are `BLOCKED`. The planner does
not verify installer file hashes, perform cleanup or support `--apply`.
It is also available without PowerShell as
`node tooling/storage/retention.mjs` from `Development/`.
Synthetic tests: `node --test tooling/storage/retention.test.mjs`.

`storage/package-recycle.mjs` is the **only narrowly destructive developer
artifact owner**. It is called only by the successful native candidate
packager: preserve the newest two exports and, at most, delete eight older
verified three-file installer candidates after SHA-256 and path validation.
It never runs `cargo clean`, removes Cargo incremental artifacts, touches
private App backups, or exposes a generic delete CLI. A blocked scan is
reported, not bypassed. Synthetic tests:
`node --test tooling/storage/package-recycle.test.mjs`.

## Historical compiler cache retirement (explicit approval)

`storage/archive-retire.mjs` is an allowlisted backend for the AI task
`storage.retire-archive`. It uses the existing Cargo footprint inspector and
only permits a scoped, pinned `cargo clean --target-dir` against the retired
`workspace/archive/compiler-cache-20261026/target`, **never** the live
`workspace/builds/desktop/target`. The archive must have only recognized
Cargo layout, no links/hardlinks, a complete scan and an already populated
Windows-MSVC active target. Unknown or partial contents are blocked;
physical reclaimed bytes require Windows evidence. This task needs
`--approve storage.retire-archive` after explicit operator consent.
Synthetic tests: `node --test tooling/storage/archive-retire.test.mjs`.

## Cargo target attribution

`DEV.cmd storage cargo` uses the bounded read-only
`storage/cargo-footprint.mjs` scanner. It separates canonical versus
historical Cargo targets and provides debug/release and
deps/incremental/build size groups. It does **not** calculate NTFS
allocated bytes, delete data, run Cargo or authorize cache pruning.
Synthetic tests:
`node --test tooling/storage/cargo-footprint.test.mjs`.
The portable `DEV.cmd` environment rejects an ambient
`CARGO_TARGET_DIR` pointing outside the canonical target so new
compiler output is not silently split across extra directories.

## Build-check space evidence

`DEV.cmd bench:rust` uses the same `storage/cargo-footprint.mjs` owner
for read-only active-target measurements before and after each of its two
checks. The `--active-json` option exposes only schema, completeness
and aggregate bytes/counts. Partial scans make the disk delta `UNKNOWN`;
scanning is outside the benchmark stopwatch and never invokes Cargo or
deletes caches. `DEV.cmd` pins both Cargo target and Cargo intermediate
build-directory environment variables to the approved output root.
Direct Cargo from an unrelated shell is not governed by these settings.

## Resource-efficiency source previews — closed decision, reusable code

These **separate, explicit build tools are retained** for possible future
operator-approved testing; they are not runtime services and neither is
installed or activated by LazyTester. Their builders and dedicated fixtures
are outside the normal product npm/build/package and DEV.cmd task routes;
Python/Pillow is not a product prerequisite. The existing full developer
verification contract and pinned toolchain gate remain unchanged. The [current closed scope, proof limits,
and reopen gates](../../planning/development.md) override historical research.

From the repository root, optional **source-only synthetic checks** are:

```sh
python -m unittest discover -s Development/tooling/lazytester-textures -p 'test_*.py'
node --test Development/tooling/lazytester-particles/tune.test.mjs
```

The block checks require Python 3.12+, Pillow 10+ and Git; the particle checks
use Node.js. Follow each linked builder README for the exact explicit
`--source`/`--output` or `--official yes` procedure and original Mojang
sample version pins. Do not commit Mojang-derived output assets, merge the
separate packs, install packs in the original hosted world, or assume
individual Education profiles can enable them independently.

For a future request, **first pin the latest GitHub `Local` HEAD, read
`AGENTS.md`/`GITHUB_RULES.md` and the single planning handoff, then review
the existing builder owner and relevant tests**. Historical Git commits and
RAM studies remain references, not authorization to restart optimization.
Actual Minecraft/Windows compatibility and savings require new separate
approval and runtime evidence. No CI or monitoring is introduced here.

## macOS native artifact review (source prepared)

After a separately authorized Mac metadata phase, the existing Node toolchain
can inspect an explicit absolute path to an authorized, unchanged offline
`.app` copy with `node Development/tooling/acceptance/macos/education-target.mjs
"/absolute/path/Minecraft Education.app"` (one command from the checkout root).
Requires macOS standard tools and available `lipo` (Command Line Tools).
No install, mount, game launch, credential access or profile data scan occurs.
The report includes bundle metadata, executable SHA-256 and architecture-specific
signature/entitlement summaries; it does not expose local paths or raw group
values. Empty entitlement output remains unknown. Codesign integrity is not a
Microsoft publisher pin, official DMG verification, loaded-image proof or
instance isolation. Use an unchanged offline copy; metadata checks are not
atomic against concurrent replacement. Refusal emits a generic JSON status.
The tool is not mounted in product, DEV.cmd, MCP or startup. See
[development handoff](../../planning/development.md). Source fixtures are
included in the existing native-feasibility test route, NOT_EXECUTED.

## macOS synthetic lifecycle fixture (source prepared)

`acceptance/macos/lifecycle-fixture.swift` is a separately compiled, opt-in
acceptance program for a future approved Mac phase. It builds no game
adapter and is not part of the desktop, DEV.cmd, startup, MCP or package.
The fixture accepts no target path and refuses root. It supports no arguments
for the original single-instance lifecycle or the explicit `--profile-probe`
option for two synthetic instances of the same bundle. It copies
its own compiled executable into a fresh private temporary application bundle,
uses AppKit to request a new application, retains the original running-app
object, verifies bundle/executable identity and requests normal termination.
It waits for observed exit; timeout/refusal preserves the application and
fixture directory, with no force close, retries, adoption or recursive deletion.
Use the existing macOS developer tools, not a new runtime dependency.
A future authorized local agent may compile with `swiftc -swift-version 5
-parse-as-library` and an explicit private output path, then run the resulting
executable without arguments or with the explicit `--profile-probe` option.
The pair uses two fresh fixture-only HOME/TMPDIR directories and reads bounded
child receipts for environment HOME, Foundation home, Application Support,
Caches and temporary path resolution. No real preference/Keychain contents are
read or written. One normal close must leave the other original app alive.
Both observed exits and path results are synthetic only; shared or unresolved
paths are reported as a non-success status. Only booleans leave the parent
report, not local paths or PIDs. Preferences/authentication remain NOT_OBSERVED.
The temporary fixture bundle is retained and
may be removed by that operator after confirming exit. The result is scoped
to a synthetic app, never game isolation or gameplay proof. Source not compiled
or executed; main-run-loop behavior, bundle launch and exit remain unverified.
