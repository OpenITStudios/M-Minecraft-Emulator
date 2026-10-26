---
id: document.validation.dependency-modernization-2026-10-20
class: DOCUMENT
domain: validation
role: REPORT
authority: CANONICAL
lifecycle: ACTIVE
---

# Dependency modernization and repository cleanup — 20 October 2026

Branch: **Local**. Repository: OpenITStudios/M-Minecraft-Emulator.
Baseline: `5226e30a9bb37a3b007af0020a787e81ed10a3d0`.
Continuation baseline: `b8d7f7d1efe7c62c3be8d76eb4fa29995fcdd871`.
Executed source revision: `bb588902f472ba6413325c12ab522e0a9908a339`.
This report supersedes the earlier partial-verification batches in this same file;
their original text remains in Git history. It is the one outcome report;
[dependency tooling](../../Development/tooling/dependencies/README.md) owns command usage.

## Outcome and evidence ceiling

**Supported Windows dependency/source maintenance is complete with documented
upstream and execution exceptions.** The full `npm --prefix Development run verify:source` pipeline
PASSES at the exact source revision above: **657 passed, 0 failed, 4 skipped**.
The four existing skipped cases require file-symlink creation, which this Windows
token cannot perform. They remain NOT_EXECUTED, not security PASS. Real Windows
directory junctions now execute without skips for storage, source checkout,
dangling destination and PresentMon input rejection.

Both compiler routes report **0 errors and 0 warnings**. Core and desktop Rust
formatting, tests and Windows compilation pass. Deprecated atomic calls were
migrated without changing memory orderings or raising MSRV. Npm and Cargo audits
report zero vulnerability-class findings; two upstream GTK informational warnings
remain explicit below. This is not an all-platform security clearance. Preserved
paused/legacy Rust code still produces unused/dead-code warnings; no blanket
warning suppressions or legacy deletion were used.

The native installer release hold, interactive Windows/gameplay and four-client
acceptance are unchanged and NOT RUN. No CI, main, driver, account or provider
cutover changes were made. Evidence ceiling is **SOURCE_EXECUTION**, not package
or gameplay acceptance.

## Continuation repairs

### Windows-first process inspection efficiency

At `bb588902f472ba6413325c12ab522e0a9908a339`, the desktop instance authority
uses one targeted PID refresh with `ProcessRefreshKind::nothing().without_tasks()`
instead of `System::new_all()` followed by another refresh. Both current-process
identity and stale-owner inspection retain PID/start-time matching. Windows still
uses its exclusive file handle for instance authority. No extra dependency,
manifest/lockfile, IPC, data path, installer or legacy-route change is needed.
The existing bootstrap is the consumer; no second process manager was added.

[Upstream sysinfo documentation](https://docs.rs/sysinfo/0.39.6/sysinfo/struct.ProcessRefreshKind.html)
confirms start time remains available with optional refresh fields disabled.
Two real-process regressions verify stable current identity and reject a matching
PID with a different start time; the existing exclusive-owner Windows test passes.
This reduces requested collection work; startup milliseconds, RAM savings and game
FPS were **NOT RUN / NOT_EXECUTED** and no measured speedup is claimed.

| Windows-first verification at the executed source revision | Result | Evidence |
| --- | --- | --- |
| Full source pipeline | PASS | 657 passed, 0 failed, 4 existing file-link skips; desktop now 64 tests |
| TS6 compatibility route | PASS | 0 errors, 0 warnings; TS7 also passes in full source pipeline |
| Dependency audit | PASS with existing warnings | npm clean; Cargo zero vulnerability-class findings, two GTK informational warnings retained |
| Windows dependency selection | PASS | `cargo tree --locked --target x86_64-pc-windows-msvc`; glib, gtk and proc-macro-error absent from selected graph |
| Installer/gameplay/four-client acceptance | NOT RUN | Existing package hold and acceptance boundaries retained |

Logs are reproducible local outputs under ignored
`workspace/dependency-maintenance/windows-efficiency-*`; they are not committed.
File moves, deletions, consolidation, new ignores and dependency version changes:
none in this bounded efficiency continuation. Cross-platform lockfile warnings
remain visible; Windows focus does not authorize unsafe upstream overrides.

### Earlier maintenance repairs

- Split ordinary PresentMon CLI assertions from link fixtures. If Windows refuses
  a file symlink, the latter test exercises a real junction and reports that
  substitution explicitly. Three prior directory-link skips now execute.
- Format all desktop Rust and include its check in the existing source route.
  Contract tests retain their safety conditions while accepting rustfmt layout.
- Align both package engines with the supported Node floor and exact npm;
  regenerate the npm lock with npm. Setup/upgrades enforce --engine-strict.
  Supported installation passes; a Node 24.14.1 dry-run probe fails EBADENGINE.
- Remove unused legacy CSS and use a valid focusable dialog container. No legacy
  route was enabled. Native TS7 and TS6 compatibility checks are warning-free.
- Remove fs2 after actual Windows regressions exposed its contention error as
  Uncategorized instead of WouldBlock, bypassing existing journal retries.
  Standard-library locks restore typed contention handling, shared/exclusive
  ownership, Drop release and the retry budget. Both Cargo locks were regenerated.

[File locking](https://doc.rust-lang.org/std/fs/struct.File.html#method.try_lock)
is stable since Rust 1.89, below the existing core minimum, and uses Windows
LockFileEx. Atomic [try_update](https://doc.rust-lang.org/std/sync/atomic/struct.Atomic.html#method.try_update)
is stable since Rust 1.95. Its three renamed calls retain the original closures
and orderings; deprecated-API warnings are absent without suppressions.

## Versions and compatibility

Official npm/crates.io metadata, Node distribution index, Rust stable channel,
and vendor release metadata were queried on 20 October 2026. Latest means the
published stable release at inspection, constrained by actual peer/API support.
LazyDesigner was inspected for the native compiler alias pattern only; neither
npm nor Cargo lock data was copied from it.

| Tool | Before declared | New pinned | Decision |
| --- | --- | --- | --- |
| Node | 24.21.0 | 24.22.0 | Latest LTS; 26.12.0 is Current, not LTS |
| npm | 11.19.0 | 12.2.0 | Latest stable compatible with selected Node; installed into portable Node |
| Rust | 1.98.1 | 1.99.1 | Latest stable; core MSRV 1.85 -> 1.95 for sysinfo; desktop retains 1.98 |
| PowerShell | 7.5.4 | 7.6.7 | Latest stable portable Windows release |

Node and PowerShell archives retain official SHA-256 verification; npm is
provisioned from the official registry into the checkout-local tool directory.
Bootstrap was actually executed successfully, including Rust/rustfmt. No global
PATH/profile or app installation was changed. All minimum-version builds were
executed with Rust 1.99.1; separate MSRV compiler runs were NOT RUN.

### All direct npm dependencies

| Package | Before resolved | New resolved | Compatibility / migration |
| --- | --- | --- | --- |
| @tauri-apps/api | 2.12.3 | 2.12.4 | Independent supported Tauri release |
| @sveltejs/vite-plugin-svelte | 6.2.4 | 7.3.1 | Stable; peer graph resolved and tested |
| @tauri-apps/cli | 2.12.3 | 2.12.3 | Independent supported Tauri release |
| @types/node | 24.19.2 | 24.19.2 | Matches Node 24 LTS |
| @typescript/native | not installed | 7.0.2 | Stable official npm:typescript alias; primary Svelte native checker |
| svelte | 5.57.2 | 5.57.2 | Stable; peer graph resolved and tested |
| svelte-check | 4.7.7 | 4.7.7 | Stable; peer graph resolved and tested |
| typescript | 6.0.3 | 6.0.3 | Retained for Svelte JS API and compatibility command |
| vite | 7.3.6 | 8.3.4 | Stable; peer graph resolved and tested |
| vitest | 3.2.7 | 5.0.5 | Stable; peer graph resolved and tested |

Vite 8/Rolldown, Svelte plugin 7 and Vitest 5 were resolved together. Initial
in-place resolution failed with ERESOLVE against the old inspector graph. A fresh
npm resolution, followed by npm ci and npm ls, succeeded without force or
legacy-peer-deps. All integrity metadata is now npm-generated. Removed the obsolete
esbuild-specific allowScripts entry when replacing the Vite 7 build graph.

TypeScript native is stable TypeScript 7.0.2 via an npm alias; the literal
@typescript/native registry package does not exist. Both compiler routes passed.
An illustrative run took about 2 seconds native versus 5 seconds compatibility;
this is an observed run, not a benchmark guarantee. The duplicate local *.svelte
ambient declaration was removed in favor of Svelte's own types. Vite client types
fix the CSS side-effect import without suppressing compiler checks.

### Rust direct dependencies and Tauri implementation graph

Versions below prefer runtime-core where shared; the full Cargo locks remain the
exact graph authority. Tauri's internal crates are deliberately not patch-aligned.

| Crate | Before resolved | New resolved | Compatibility / migration |
| --- | --- | --- | --- |
| tauri | 2.11.6 | 2.12.4 | Cargo-resolved supported graph |
| tauri-build | 2.6.3 | 2.7.3 | Cargo-resolved supported graph |
| tauri-runtime | 2.11.3 | 2.12.2 | Cargo-resolved supported graph |
| tauri-runtime-wry | 2.11.4 | 2.12.2 | Cargo-resolved supported graph |
| wry | 0.55.1 | 0.57.0 | Cargo-resolved supported graph |
| rfd | 0.15.4 | 0.17.2 | Cargo-resolved supported graph |
| sysinfo | 0.30.13 | 0.39.6 | API migration; system feature only; core MSRV 1.95 |
| serde | 1.0.229 | 1.0.229 | Cargo-resolved supported graph |
| serde_json | 1.0.151 | 1.0.154 | Cargo-resolved supported graph |
| sha2 | 0.10.8 | 0.11.1 | Preserve 64-character lowercase hash encoding |
| getrandom | 0.2.17, 0.4.3 | 0.4.3 | Migrate getrandom() to fill() |
| native-tls | 0.2.12 | 0.2.18 | Direct dependency removed; ureq owns platform TLS |
| ureq | 2.9.7 | 3.4.2 | v3 config/header/status/body APIs; bounded responses and TLS tests |
| url | 2.5.0 | 2.5.8 | Cargo-resolved supported graph |
| windows-sys | 0.61.2 | 0.61.2 | Cargo-resolved supported graph |
| fs2 | 0.4.3 | removed | std::fs::File locks; Windows contention regression fixed |

The desktop additionally carries sha2 0.10.9 through upstream dependencies alongside
our direct sha2 0.11.1. Independent lockfiles are retained. Both were resolved by
Cargo update and checked with --locked; no lockfile was transplanted or hand-built.
System/process consumers were migrated to current CPU refresh, process-list and
OsStr APIs. Existing PID/start-time attribution and resource thresholds remain intact.
Shared/exclusive lock ownership is retained; Windows contention mapping is corrected. Sysinfo disables unused default subsystems in both packages.
Ureq's platform TLS provider replaces our redundant direct native-tls connector;
status mapping, no proxy/no redirects for the localhost provider, timeout bounds,
HTTPS admission and bounded body reads have regression coverage.

### Remaining version exceptions

- TypeScript 6.0.3 remains required by svelte-check's JS API peer range; stable
  TypeScript 7.0.2 is also installed and used as the primary native checker.
- Node types stay 24.19.2 rather than latest 26.6.5 to match Node 24 LTS.
- fs2 was initially retained, then removed after continuation tests exposed the
  Windows contention mismatch. Stable standard-library locks need no replacement
  dependency and retain the shared/exclusive ownership contract.
- cargo-outdated reports core fully current. Desktop reports upstream feature
  branches serde_with -> indexmap 1.9.3 / schemars 0.9.0 and indexmap ->
  hashbrown 0.12.3 / autocfg. New major branches exist, but these are upstream
  compatibility edges; do not force-overwrite them or patch vendor crates.
- npm outdated --all also lists absent optional peers and non-host platform
  binaries. They are not missing mandatory Windows dependencies (npm ls succeeds).
  Installed transitive exceptions below remain within their owning package's
  selected constraints; no blanket overrides were added. npm audit found none
  vulnerable in this graph.

| Transitive package | Resolved | Latest stable | Owner constraining upgrade |
| --- | --- | --- | --- |
| @oxc-project/types | 0.154.0 | 0.155.0 | rolldown |
| aria-query | 5.3.1 | 5.3.2 | svelte |
| chokidar | 4.0.3 | 6.0.0 | svelte-check |
| devalue | 5.9.4 | 6.0.3 | svelte |
| es-module-lexer | 2.3.2 | 3.0.4 | vitest |
| magic-string | 0.30.21 | 1.4.3 | svelte |
| nanoid | 3.3.20 | 6.0.2 | postcss |
| obug | 2.2.1 | 3.0.0 | @sveltejs/vite-plugin-svelte |
| readdirp | 4.1.2 | 5.1.1 | chokidar |
| undici-types | 7.24.6 | 8.12.3 | @types/node |
| why-is-node-running | 3.2.1 | 3.2.3 | vitest |

Sandboxie owned/candidate source remains 1.18.6; the active official pilot remains
separately installed and unchanged. Vendor latest [v1.18.8](https://github.com/sandboxie-plus/Sandboxie/releases/tag/v1.18.8)
was observed. Snapshot/driver/service migration is deferred because it needs a
separate provenance, ABI, signing, service ownership and runtime acceptance scope.
No claim is made that existing native source has been audited by npm/Cargo.

## Structure audit and cleanup

Every tracked path was inventoried by ownership. Before: 3,552 tracked files,
15 root files; apps 84, docs 34, engine 3,324, tooling 79, planning 8,
experiments 5, .agents 2 and workspace 1. The root already met its role policy.
After: 3,556 tracked files in the same major ownership layout, 15 root files, with focused verification
and maintenance tooling added; no package or Cargo workspace was merged.

```text
(root policies/config/command entrypoint)
.agents/        repository agent routing
Development/apps/desktop/   frontend, Tauri package, package-scoped manifests/locks/tests
Development/engine/         runtime-core, distribution, guest, native owned/reference/candidate sources
Development/tooling/        repository/dependency checks, acceptance and native build tooling
docs/           stable navigation and validation evidence
planning/       current handoff plus historical evidence
Archive/experiments/    scoped prototypes
workspace/      README only tracked; tools/caches/reports remain ignored
```

| Requested deliverable | Actual change / finding |
| --- | --- |
| Files/directories moved | Development/tooling/dependencies/RUST-UPGRADE.md -> README.md; no application/native folder moves |
| Duplicates consolidated | Svelte ambient declaration removed in favor of upstream types; dependency workflow guide and report have distinct usage/evidence roles |
| Unnecessary files removed | Development/apps/desktop/src/svelte.d.ts; no legacy, account-bearing or vendor files removed |
| Configuration standardized | npm exact pins/alias, toolchain metadata/archive hashes, Vite client types, Rust dependency features/MSRV |
| Scripts consolidated | Existing upgrade helper now uses live stable metadata; new audit command orchestrates existing package managers, no framework |
| Documentation reorganized | One dependency guide; routers link to it and this report; report frontmatter repaired; stale toolchain prose replaced by canonical links |
| Paths/references updated | Root manual route, routing regression, documentation navigation and old Rust guide references |
| Generated output ignored | /apps/desktop/.svelte-check/; previous node_modules/dist/target/workspace ignores remain |
| Structural validation | Root/path ownership, required owners/locks, tracked generated output, simple local Markdown links and npm command paths pass |
| Remaining exceptions | Intentional upstream binaries/resources, nested vendor configs, immutable candidate copies, legacy sources and package boundaries preserved |

No cosmetic root relocation, invented directories, generic test framework, native
source deduplication or redundant root dependency graph was introduced. The fast
checker does not parse arbitrary shell languages, validate Markdown anchors or
check external URLs; package-manager/build tests validate the actual import graph.

## Verification results

All source pipeline results refer to `bb588902f472ba6413325c12ab522e0a9908a339`.
Bootstrap, strict install, unsupported-Node probe and upgrade-tooling evidence
remain the earlier maintenance runs; those toolchain/manifests are unchanged.
The final report/handoff delivery is a separate documentation-only commit.

| Check | Result | Evidence / limitation |
| --- | --- | --- |
| Full npm --prefix Development run verify:source | PASS | 657 passed, 0 failed, 4 existing file-link cases skipped |
| Portable bootstrap | PASS | Previously executed verified bootstrap; pins unchanged during continuation |
| Strict npm ci and npm ls --all | PASS | npm 12.2.0; both compiler routes installed |
| Unsupported Node install probe | PASS | Node 24.14.1 dry run rejected with EBADENGINE |
| Manifest/lock and structure checks | PASS | Engine ownership, exact pins/aliases, CLI integrity, paths/docs/output ownership |
| Tooling regressions | PASS | 10 tests including Windows PowerShell AST parsing; no downloads in parser test |
| Svelte + TypeScript 7 native | PASS | 0 errors, 0 warnings |
| TypeScript 6 compatibility | PASS | 0 errors, 0 warnings |
| Desktop Vitest | PASS | 17 files / 161 tests |
| Frontend production build | PASS | Vite 8 / Rolldown production assets |
| Core Cargo check --locked --all-targets | PASS | Windows MSVC toolchain |
| Core Rust tests | PASS | 176 tests: 163 library, 4 guest-agent, 9 integration |
| Desktop Cargo check and tests --locked --all-targets | PASS | 64 desktop unit tests |
| Desktop executable compilation | PASS | cargo build --locked; no GUI/game launched |
| Core rustfmt | PASS | Full core package --all --check |
| Whole-desktop rustfmt | PASS | Included in the existing desktop source route |
| Deprecated Rust API warnings | PASS | None; preserved unused/legacy warnings remain |
| PresentMon CLI/source tests | PASS | 9 tests; ordinary input/errors and real junction rejection |
| Isolation-source tests | PASS | 165 passed, 4 pre-existing file-symlink cases skipped |
| npm security audit | PASS | 0 vulnerabilities |
| Cargo security audit | PASS | 0 vulnerability-class findings; 2 informational GTK warnings below |
| Dependency tree/outdated audit | PASS | Both graphs inspected; retained constraints listed above |
| Native installer construction/inspection | NOT RUN | Existing PACKAGE_BLOCKED_NATIVE_FEASIBILITY release gate; no bypass |
| Interactive Windows lifecycle, sign-in, gameplay, four clients | NOT RUN | Source tests do not establish runtime acceptance |

At the continuation baseline, PresentMon failed during fixture creation and
whole-desktop rustfmt failed. Both are resolved. Three prior directory-link skips
now execute with actual Windows junctions. PresentMon reports its file-symlink to
junction substitution as a diagnostic; it does not claim that privileged file
symlink creation ran here. Four existing file-link cases remain skipped: required
native executable, external private binary, artifact hash source and dangling
optional artifact. No new skip was introduced and no safety assertion was removed.

The two new Windows locking regressions failed on fs2 with Uncategorized rather
than WouldBlock, then passed after standard-library migration. They exercise real
cross-process readers/writers, Drop release and journal retry timing. The complete
pipeline also executes all existing feasibility and performance source suites.

RustSec database commit: 999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b.
Remaining informational warnings are
[glib 0.18.5 unsound iterator, RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html)
and [proc-macro-error 1.0.4 unmaintained, RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html).
Both are carried by the upstream Tauri/GTK Linux dependency branch, not the
compiled Windows tree. They remain a real limitation for any future Linux build;
fixing them requires the upstream GTK/Tauri graph migration, not an incompatible
Cargo override. Audit exit zero does not erase these warnings.

## Git delivery

Source push: **PASS**, fast-forward to `972b0e40440452d16ff5ee35678559afa66d84e8`, verified against remote Local
on 20 October 2026. Remote main remains `cb106f9013206a73f6172c12fb62ee590a1c6b15`
and was not modified. The report, dependency guide and handoff are a separate
documentation-only delivery commit; find its exact SHA in this file's Git history.

Logical commits:

```text
a3d1649 refactor(repo): verify canonical structure and restore report metadata
d12eb90 build(desktop): migrate Vite Svelte plugin and Vitest together
add047e build(desktop): enable stable native TypeScript checking with Svelte compatibility
2fde707 build(runtime): migrate networking system and Tauri dependencies
7019769 build(tooling): centralize dependency audits and refresh portable toolchains
e6eb135 build(repo): preserve desktop MSRV and remove stale toolchain copies
5ecbf26 fix(tooling): distinguish outdated findings from registry failures
b8d7f7d docs(dependencies): report modernization results and remaining verification limits
9ae8fec test(tooling): exercise real Windows junctions without symlink privilege
3ae04c8 fix(runtime): replace fs2 with typed standard-library file locking
8bd44bd build(desktop): enforce clean source formatting and supported toolchain installs
972b0e4 refactor(runtime): use stable atomic update API without deprecation
```

No force push, workflow, driver installation or release publication was used.
Changed paths through the executed source revision:

```text
M	.gitignore
M	.node-version
M	Development/apps/desktop/package-lock.json
M	Development/apps/desktop/package.json
M	Development/apps/desktop/src-tauri/Cargo.lock
M	Development/apps/desktop/src-tauri/Cargo.toml
M	Development/apps/desktop/src-tauri/build.rs
M	Development/apps/desktop/src-tauri/src/commands/native_clients.rs
M	Development/apps/desktop/src-tauri/src/engine/app_instance.rs
M	Development/apps/desktop/src-tauri/src/engine/instance_catalog.rs
M	Development/apps/desktop/src-tauri/src/engine/instance_discovery.rs
M	Development/apps/desktop/src-tauri/src/engine/instance_focus.rs
M	Development/apps/desktop/src-tauri/src/engine/instance_focus_events.rs
M	Development/apps/desktop/src-tauri/src/engine/instance_identity.rs
M	Development/apps/desktop/src-tauri/src/engine/native_clients.rs
M	Development/apps/desktop/src-tauri/src/engine/owned_isolation.rs
M	Development/apps/desktop/src-tauri/src/engine/virtual_inventory.rs
M	Development/apps/desktop/src/NativeFeasibilityApp.svelte
M	Development/apps/desktop/src/app/components/WindowLayoutDialog.svelte
D	Development/apps/desktop/src/svelte.d.ts
M	Development/apps/desktop/test/compact-manager.test.ts
M	Development/apps/desktop/test/native-feasibility.test.ts
M	Development/apps/desktop/test/verification-routing.test.ts
M	Development/apps/desktop/tsconfig.json
M	docs/README.md
M	docs/system/development-operations.md
M	docs/validation/dependency-modernization-2026-10-20.md
M	docs/validation/native-pilot-localtest.md
M	Development/engine/runtime-core/Cargo.lock
M	Development/engine/runtime-core/Cargo.toml
M	Development/engine/runtime-core/src/bin/virtual-guest-agent.rs
M	Development/engine/runtime-core/src/capacity.rs
M	Development/engine/runtime-core/src/doctor.rs
M	Development/engine/runtime-core/src/guest.rs
M	Development/engine/runtime-core/src/journal.rs
M	Development/engine/runtime-core/src/operation_lock.rs
M	Development/engine/runtime-core/src/profile.rs
M	Development/engine/runtime-core/src/provider/app_sandbox.rs
M	Development/engine/runtime-core/src/provider/app_sandbox_pilot.rs
M	Development/engine/runtime-core/src/provider/mod.rs
M	Development/engine/runtime-core/src/provider/process.rs
M	Development/engine/runtime-core/src/provider/vmx.rs
M	Development/engine/runtime-core/src/resources.rs
M	Development/engine/runtime-core/src/update.rs
M	package.json
M	planning/development.md
M	toolchain.json
M	Development/tooling/README.md
M	Development/tooling/acceptance/analyze-presentmon.test.mjs
A	Development/tooling/dependencies/README.md
D	Development/tooling/dependencies/RUST-UPGRADE.md
A	Development/tooling/dependencies/audit.mjs
A	Development/tooling/dependencies/maintenance.test.mjs
M	Development/tooling/dependencies/upgrade-frontend.mjs
M	Development/tooling/isolation-source/bootstrap-fork.test.mjs
M	Development/tooling/isolation-source/verify-checkout.test.mjs
M	Development/tooling/local-test/session.test.mjs
M	Development/tooling/repository/check-dependency-lock.mjs
A	Development/tooling/repository/check-dependency-lock.test.mjs
A	Development/tooling/repository/check-structure.mjs
A	Development/tooling/repository/check-structure.test.mjs
M	Development/tooling/repository/verify.mjs
M	Development/tooling/windows-toolchain/bootstrap.ps1
M	Development/tooling/windows-toolchain/portable.test.mjs
M	workspace/README.md
```

The documentation-only delivery also updates this report, the dependency guide
and planning/development.md. Remaining limits are upstream-constrained versions
and GTK warnings, four file-symlink execution cases, and separately held installer
and gameplay acceptance. They do not authorize unsafe overrides or scope expansion.
