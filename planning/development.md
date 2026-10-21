# Current Development Handoff — M-Minecraft-Emulator

## Rust-first hybrid decision — 21 October 2026

**APPROVED, architecture-only**: Rust is the primary language for the
independent engine domain, Win32 user-mode adapters and any later justified
user-mode broker. Tauri/Svelte remains the UI; the existing independent C++
AppContainer fixture stays test-only. An independently developed, minimal
C/C++ WDK driver may be considered **only after** proving a concrete kernel
capability gap, with security, signing and rollback review. Do not translate
the retired Sandboxie fork, initiate a mass C++ rewrite or add another
lifecycle authority. Canonical policy:
[Independent Engine Architecture](../docs/system/independent-engine-architecture.md#implementation-language-and-ownership-policy---21-october-2026).

**Implementation effect:** no provider activation, Windows service/driver,
packaging, user account, world or existing runtime data change. This
decision records future source standards, not test evidence. The prior
source-first task remains next: bind future synthetic process-local
path observations to independently verified PID/FILETIME and keep
auth/roaming unknown until target acceptance. Windows/Minecraft
acceptance remains NOT_EXECUTED.

## Independent engine — source-first A/B path collector, 21 October 2026

The existing native AppContainer synthetic fixture reports separate
per-child `processPaths` for Local AppData Known Folder, APPDATA environment,
and Roaming Known Folder. The offline evaluator checks schema-2 child
observations against aggregate flags and does not infer real process identity
or successful data isolation. Negative/source tests authored, NOT_EXECUTED.
No Minecraft, Windows driver, installation, old profile, `main`, or CI
changes. The historical negative-control failure remains schema 1.
**Next:** bind future native path collection to independently pinned
PID/FILETIME without treating synthetic A/B labels as game process identity.

## Independent engine — source-first architecture, 21 October 2026

**Operator decision:** finish modular architecture and source integration on
GitHub branch `Local` before executing Windows/Minecraft local acceptance.
New source-only domain contracts in `engine/runtime-core/src/independent_engine/`:
`identity`, `storage`, `operations`, `ports`. They define exactly four
canonical desktop slot IDs, full-width process identities, nine separate
storage/broker boundaries, presentation versus lifecycle side effects, and
separate native observer/provider interfaces. The default provider explicitly
returns NotImplemented; existing Tauri/admission paths remain unchanged.
The canonical design is [Independent Engine Architecture](../docs/system/independent-engine-architecture.md).
Tests are authored, with Rust compilation and Windows acceptance NOT_EXECUTED.

**Read-only orchestration closure:** `independent_engine/inspection.rs`
takes typed observer ports, rejects stale PID/creation-time identity before
storage probing, and exposes unresolved roaming/authentication findings.
Three negative/source-only Rust tests use fake adapters. This does not
connect to the UI, grant a profile, authenticate, or choose a provider;
Rust/Windows execution remains NOT_EXECUTED.

**Source adapter milestone:** The existing `instance_identity.rs` now
implements the typed `WindowsProcessObserver` using the canonical pin
and FILETIME check. `inspect_independent_source` composes that native
process observation with an explicit all-`Unknown` storage observer.
Signed Minecraft image, HWND, roaming, registry and broker are never
claimed on this path. Windows and non-Windows negative/source tests
are authored; compilation/execution remains NOT_EXECUTED here.
No Tauri command or new provider was registered.

**Current source contract:** Storage inspection now requires the same
full-width process identity as the process observer, preventing a future
adapter from silently relying on slot/PID alone. The existing Windows
pinned-handle observer returns only a sanitized AppContainer token
category, with all filesystem, registry, known-folder and authentication
surfaces still Unknown. Negative tests assert that a stale/missing
process cannot trigger storage inspection. Windows/Rust execution remains
NOT_EXECUTED; no Tauri launch/provider mutation or new service.

**Bound storage evidence source:** `independent_engine/evidence.rs` now
implements typed, exact-process validation for nine storage/broker
surfaces, rejecting wrong slot, changed FILETIME, duplicated surfaces,
synthetic positive claims, and path-as-file-access claims. The domain
storage observation setter is crate-private, so future desktop/native
adapters must use the reviewed conversion function rather than
arbitrarily assign verified fields. Negative Rust tests are authored but
NOT_EXECUTED; no Windows or Minecraft acceptance is claimed.

**Process-local path contract:** `independent_engine/paths.rs` now
maps three sanitized process-local LocalAppData/roaming observations
to explicit `PathOnly`, `PathOutsideProfile` or `Unknown`.
It binds slot and full-width process identity and never upgrades a
path string to private file access. New negative unit tests are
authored, but not executed. The existing native desktop storage
adapter still reports all Unknown and launch remains blocked.

**Next source-only milestone:** specify a native storage evidence adapter
with exact-handle/lifetime binding and explicit known-folder/registry/broker
unknowns; keep account-bearing paths untouched pending local acceptance.

## Independent sandbox engine — ACTIVE source foundation, 21 October 2026

**Operator explicitly resumed implementation in ChatGPT on 21 October 2026.**
Scope: independently authored, modular Minecraft-focused isolation engine with
OpenITStudios-owned source; Sandboxie runtime/fork remains retired. The initial
implementation is the pure evidence classification policy in
`engine/runtime-core/src/independent_isolation.rs`, covering synthetic Windows,
native filesystem/namespace/roaming and two-/four-game acceptance. It does not
activate a provider or create accounts, drivers, processes, profiles or new
stores. Its authored negative/unit tests are source only; source/Windows execution
is NOT_EXECUTED in remote GitHub. No imported Sandboxie source is used by this
module. Prior source/fork trees remain historical, not implementation inputs.

The existing desktop `instance_identity.rs` also adds a read-only comparison
of two pinned process AppContainer token identities (sanitized booleans only).
No login, game launch, Windows service/driver or profile operation was performed.
Windows tests are authored but NOT_EXECUTED in remote GitHub.

**Current bounded implementation:** extended the independent native
AppContainer synthetic fixture to create private roaming-like canaries,
stop both children, relaunch two new children under the same distinct
AppContainer identities, verify persistent own data and denied peer reads,
then clean only the fresh fixture profiles. The PowerShell runner asserts
synthetic-only result fields. Neither Windows Known Folder roaming
redirection nor Education broker behavior is inferred from these checks.
Updated MSVC/Windows execution is **NOT_EXECUTED** in ChatGPT/GitHub.

**Current source review:** `tooling/acceptance/evaluate-appcontainer-isolation.mjs`
is a strict-schema, offline, non-authoritative reviewer for the synthetic
JSON output. It separates privately persistent fixture canaries from
`APPDATA` and `FOLDERID_RoamingAppData` path observations. Even when
both paths look private, the reviewer never claims actual known-folder
file-access virtualization, Microsoft authentication or Minecraft readiness;
`isolatedLaunchAllowed` is always false. Negative/malformed evidence is
rejected; no raw account, path or SID fields are accepted. Node unit tests
are included in `test:native-feasibility`. These are source changes, not
new Windows or Minecraft acceptance.

**Next:** execute this opt-in Windows synthetic fixture on an isolated
Windows target and investigate supported private `RoamingAppData` routing;
if the Win32 path remains host-wide, record NO-GO for the unmodified game
unless a supported independent isolation mechanism is proven. Retain current isolated Launch/
Provision blockade; preserve old accounts/worlds/services. No GitHub Actions,
release, main, VM or certificate changes.

### Historical paused handoff (superseded by explicit resumption)

## Independent engine - PAUSED for operator discussion, 21 October 2026

**PAUSED at operator request.** Last implementation/proof commit:
`d823f68456e64298aedea38d879daabe3be4aa55` (`Local`). No further experiments,
game launches, service changes or implementation until explicitly resumed.
Start the ChatGPT discussion from the
[discussion brief](../docs/system/independent-isolation.md#bahan-diskusi-chatgpt---21-oktober-2026).
Architecture and retirement decisions remain in the existing linked owners there.
[Local source archive](../experiments/history/local-investigation-2026-10-21/README.md)
adds 29 historical snapshots plus sanitized fixture results for discussion only;
no runtime/implementation resumed.
Sandboxie and its fork remain retired for new product sessions; installed
services, old profiles/accounts/worlds and legacy VM data remain preserved.
Two synthetic native AppContainers passed token/namespace/canary/lifetime checks;
negative reused-token control failed as expected; fixture profiles removed.
Local AppData redirected, roaming environment/known-folder paths did NOT.
No independent Minecraft launch, auth or four-client acceptance exists.
**Candidate after discussion, not authorization:** prove private persistent
roaming storage without host access or game modification before game smoke tests.

## Windows-first efficiency — 20 October 2026

**State: DONE for bounded desktop process inspection.** Source:
`bb588902f472ba6413325c12ab522e0a9908a339`. The existing desktop instance owner
now requests only its target PID and basic identity fields, replacing a full
system/process refresh plus redundant refresh. PID/start-time identity and Windows
exclusive handle semantics are preserved; no dependency or release format changes.
Two real-process regressions were added. Full `npm run verify:source`: **657 passed,
0 failed, 4 existing file-symlink skips**; desktop 64 tests. TS6 compatibility,
formatting, Windows compilation and dependency audit also pass with the existing
two cross-platform GTK informational warnings. Those crates are absent from the
explicit Windows Cargo graph. The canonical [report](../docs/validation/dependency-modernization-2026-10-20.md)
records exact evidence and unchanged exceptions. Actual startup/RAM/FPS gains,
installer, gameplay and four-client acceptance remain NOT_EXECUTED. No CI, main,
driver or account operations were performed.

## Dependency modernization and structure cleanup — 20 October 2026

**State: DONE for supported Windows dependency/source maintenance**, with explicit
upstream/runtime exceptions. Source: `972b0e40440452d16ff5ee35678559afa66d84e8`.
The complete `npm run verify:source` passes: **655 passed, 0 failed, 4 existing
file-symlink tests skipped**. Native TS7 and TS6 checks have 0 errors/0 warnings;
both Rust packages pass formatting, checks and tests. The canonical
[report](../docs/validation/dependency-modernization-2026-10-20.md) owns versions,
change inventory, source commits and exact evidence.

Continuation fixed Windows lock-contention error mapping and journal retries
while removing fs2; actual cross-process regressions pass. Real Windows junctions
replace directory fixtures that previously skipped. Installs enforce supported
Node/npm engines. Deprecated atomic calls were migrated without an MSRV increase.
Source batches were pushed and remotely verified.

Npm/Cargo audit: zero vulnerability-class findings; two upstream GTK Linux
informational warnings remain. Four file-symlink execution cases, native installer
release approval and gameplay/four-client proof remain unexecuted/held. No CI,
main, driver, account or gameplay mutation occurred. Maintain dependencies through
existing audit/check commands; do not reopen features or legacy operations merely
to erase these separately scoped exceptions.


**Repository:** `OpenITStudios/M-Minecraft-Emulator`; all product changes on
branch `Local`. Current operator decision: **GitHub Actions CI OFF**.

## Current work: retire CI and preserve safe foreground observation

The operator explicitly revoked approval for GitHub Actions on
20 October 2026. Remove **both** workflow definitions from `Local`:
`.github/workflows/verify.yml` (automatic Full Source / manual Quick /
Full Source / Windows Build) and `.github/workflows/package-smoke.yml`
(manual installer/package testing). The separate registration-only
`main` workflow was removed in
[main commit cb106f9](https://github.com/OpenITStudios/M-Minecraft-Emulator/commit/cb106f9013206a73f6172c12fb62ee590a1c6b15).
The canonical `toolchain.json`, root rules, repository verifier and
validation router now reflect the CI-disabled policy. Never restore an
Actions workflow, third-party hosted runner, or scheduled CI without
fresh explicit user approval. Existing `npm run verify:source`, Rust, Svelte
and Windows scripts remain *available* for optional manual execution
when supported and requested. They do not run automatically.

This repository write is not the same as changing the repository-wide
GitHub **Settings → Actions** administrative switch: the connector exposes
source editing but not that settings mutation. To disable GitHub Actions
even against future workflow re-additions or other branches, an
administrator must turn Actions off in the repository settings. No new
Actions workflow definitions remain on current Local/main after this
batch if both deletion commits succeed. Previously started workflow
runs may finish; historical logs remain historical evidence only.

## Continued engineering: fail-closed observer recovery

The existing opt-in Windows foreground event hook is still ONE Rust worker
in `apps/desktop/src-tauri/src/engine/instance_focus_events.rs`, with
unbuffered startup handshake, same-thread unhook and explicit Stop. The
operator-facing Start/Read/Stop state previously had a blind spot: a
failed IPC could cause the UI to offer Start even when hook cleanup was
unverified, and a worker holding the lock could be misreported as stopped.

Changes in this batch:
- A Rust status read returns an error for uncertain worker ownership
  (installing, exiting, unreaped or busy), not a fabricated `enabled=false`.
- The Svelte UI blocks Start after any failed transition or status read,
  offers **Retry Stop/cleanup** and a separate explicit **Read event count**.
- A successful Stop or valid status read restores the appropriate controls.
- Transport checks and negative frontend/source contracts updated.
- No extra service, timers, process mutation, Sandboxie driver action,
  or Minecraft graphics/simulation/account setting change.

## Evidence / remaining blockers

The preceding `Local` commit `b5db7f6c4a026ab17a29edc012427b0d04cb05d9`
[failed desktop Vitest on GitHub](https://github.com/OpenITStudios/M-Minecraft-Emulator/actions/runs/39292712346):
one assertion sought a cleanup log string without its `MIVUBI` prefix.
The actual backend log was correct. Repository, native/isolation stages
and Rust checks passed on that prior SHA. The assertion has been fixed.

**No CI will run for this new commit**. Static review supports source
coherence only; the new Rust/UI behavior is **NOT_EXECUTED** in Windows.
Source compilation, hook lifecycle on a real desktop, active Client 03/04,
four independent licensed Minecraft Education sessions and 15–20 FPS
external limiting are still unverified or not implemented. Editable owned
v1.18.6 source exists but is not an installed, supported runtime.

**Next source task:** establish positive, provider-owned Client 01 and
03/04 process attribution; do not infer it from process names or generic
Sandboxie query failures. Keep the entire performance system external
to Minecraft and leave FPS mutation disabled until a supported vendor/API
route has been measured on Windows.

Historical work: [planning/history](./history/development-through-2026-10-19.md).
