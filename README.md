# M-Minecraft-Emulator

**Product goal:** four concurrent, independently playable Minecraft Education **Windows desktop** sessions with four separately licensed accounts on one Windows PC. The **single active engineering route** is **Minecraft Instance Manager**: native Client 01 and isolated Client 02 in the persistent Sandboxie Plus box `MinecraftClient02`. Four accounts remain the long-term goal. Two-account playability was operator-confirmed on 14 October 2026, followed by a boxed-client idle crash; stable two-client and four-client acceptance remain unproven. The previous VMware implementation remains as **inactive legacy source**: VM mutation commands are paused and its UI is unmounted. All existing account-bearing guest data is preserved. The official Sandboxie two-client pilot has limited success evidence, not stable completion or independent owned-engine proof.

## Product definition — integrated Virtual Manager

**MIVUBI is the application that manages virtual sandboxes**. The original
Sandboxie-derived C/C++ driver, service and DLL belong to its internal
isolation engine; Rust/Tauri is the single virtual lifecycle authority and
Svelte provides one simplified user interface. Users should eventually
create, configure, launch and manage virtuals entirely in MIVUBI rather than
opening SandMan/Sandboxie Plus. Minecraft Education is the first supported
profile, not the definition of a virtual.

**Current implementation limit:** the Virtual Manager UI and current
version-aware owned package inspection are integrated, but the app still
uses the official Sandboxie Client 02 compatibility pilot. The internal
owned driver/service have not been built, signed, installed or validated.
The **Create virtual** action correctly remains unavailable: no visible
virtual may be called created or ready until its actual sandbox configuration
is written to and confirmed by a trusted running engine. Existing account
data is never migrated/deleted as part of UI cleanup.

## Branch authority

```text
Local → active development / source authority
main  → reviewed stable/release authority
```

## Native source ownership — Sandboxie v1.18.6

**Source migration and private native source patch are complete (source-only).**
All 1,627 authentic v1.18.6 files remain untouched at
`engine/isolation-candidates/sandboxie-v1.18.6/`, with pinned Git tree
`f690477c07a40810f479f53973334ceae7e0ab13`.
The sole editable MIVUBI native owner is now `engine/isolation-owned/`:
1,627 files, with **exactly 11 MIVUBI-modified native source files** and
all original licenses and other upstream files preserved. The old partial
v1.18.5 owned tree was superseded; its history is retained in Git, not used
as the current source. Read-only inspection:
`npm run isolation-source:verify`.

**Not complete:** no v1.18.6 MSVC/WDK build, driver signing/load,
service lifecycle, Microsoft Education login, four-client stability or
performance results are claimed. The existing official Sandboxie Client 02
pilot is still the active runtime and is never switched merely because
source code exists. The v1.18.5-specific builder/patch commands remain
historical source and must not be used for the current engine. Follow
`engine/isolation-native/owned-runtime/build-profile-v1186.json`
for required build components and excluded app UIs; this does not approve
runtime pruning without a native build closure.

**New build-only entrypoint:** `tooling/isolation-source/build-owned-v1186.ps1` is a version-pinned Windows MSBuild/WDK compiler path for the actual MIVUBI-owned v1.18.6 source. It verifies exact source before the build, rejects stale native outputs, and saves Win32/x64/WDK logs and hashes in a new report directory. It **never signs, loads, installs, launches or selects** the private driver. The script is authored but **has not run on Windows**.

**Owned v1.18.6 implementation path:** the source, build and read-only
desktop inspection are now joined through one chain:
`engine/isolation-owned/` → `tooling/isolation-source/build-owned-v1186.ps1`
→ `tooling/isolation-source/assemble-owned-v1186.mjs`
→ `apps/desktop/src-tauri/src/engine/owned_isolation.rs`.
The exact build/payload inventory belongs to
`engine/isolation-native/owned-runtime/build-profile-v1186.json`.
`npm run isolation-source:assemble` selects this v1.18.6 pipeline, not the
older v1.18.5 assembler. See [native build-to-desktop instructions](tooling/isolation-source/README.md).
No successfully built, signed, installed or playable private Windows engine is
claimed by this source integration. The installed official Client 02 remains
the only active provider.

## Start here

For ChatGPT + GitHub development, the authoritative route is:

```text
AGENTS.md → GITHUB_RULES.md → docs/system/implementation-map.md
→ exact canonical source owner → bounded source change → evidence review → STOP
```

An operator does **not** need a local checkout to complete repository/source-verifiable tasks. Windows or Minecraft acceptance is a separate, higher proof context. `DEV.cmd` is an optional local developer convenience, never the remote GitHub completion requirement.

**Primary development priority (14 October 2026): build MIVUBI's own
Sandboxie-derived Windows isolation core to completion**, not an independent
wrapper or endless source validator. The first native C++ behavioral component
is the MIVUBI-owned `Start.exe` profile admission policy in
[engine/isolation-native/owned-runtime](engine/isolation-native/owned-runtime/),
compiled and tested with GCC/Clang in ChatGPT. It is also inlined by the
exact pinned Sandboxie fork overlay, but **the complete private Windows
driver/service/DLL/Start binaries have not been built or activated**.
The already installed official Sandboxie Client 02 remains the safe active
pilot until signed disposable-Windows core evidence exists.

**Definition of DONE:** [Four simultaneous Minecraft Education accounts](docs/product/four-concurrent-accounts.md). **Technical status:** candidate PID/window observations and manually reported two-account playability are separate intermediate evidence; neither proves four-client acceptance. **Current experiment:** [One native + one Sandboxie Plus Windows Client 02](docs/validation/sandboxie-two-client-feasibility.md); explicit isolated launch and separate read-only verification. Two-account simultaneous playability has been operator-reported in the later 14 October test, but sustained stability failed. Documentation: [docs/README.md](docs/README.md). **Current handoff:** [planning/development.md](planning/development.md) (top section). **Removal map:** [Legacy Retirement](docs/system/legacy-retirement.md).

## Current operating mode

The desktop now mounts one focused **two-client Windows feasibility pilot**: official native Client 01 plus one explicitly requested Sandboxie `MinecraftClient02` Client 02, with read-only candidate process/window counts. Later operator evidence reports two authenticated playable sessions followed by a Client 02 idle crash; it does not prove stable two-client or four-client acceptance. No VM refresh, guest setup or old updater runs on startup. The existing
VMware-based installer build is explicitly blocked while this experimental
native launcher is active; there is no approved native release installer yet. Legacy VMware/App Sandbox mutation commands return UNSUPPORTED; the VM-oriented routes below remain documentation and read-only reference, not active workflows.

## Choose one work route

| Intended work | Entry | Actual authority |
| --- | --- | --- |
| Develop, refactor, repair, or improve repository | [AGENTS.md](AGENTS.md) → [.agents development procedure](.agents/skills/emulator-development/SKILL.md) | Existing source owner in `apps/` or `engine/` |
| Examine VM behavior or diagnose setup | [Implementation Map](docs/system/implementation-map.md) → [Runtime Core](docs/system/runtime-core.md) | `engine/runtime-core/`, observed provider/guest evidence |
| Follow the selected Windows two-account experiment | [Native pilot local test](docs/validation/native-pilot-localtest.md) | `DEV.cmd bootstrap` → `doctor` → `preflight` → `dev`; no installer, login/gameplay still unproven |
| Review the legacy VMware NSIS release integrity gate | [Historical G0](docs/validation/pre-local-test-readiness.md) | Old NSIS packaging intentionally blocked for native pilot |
| Consult historical one-guest VMware feasibility (inactive) | [Legacy Client Feasibility](docs/validation/client-feasibility.md) | Scratch guest procedure is retained as reference; not an active pilot instruction |
| Verify Windows/VMware/Minecraft usability | [Validation](docs/validation/README.md) | Real machine and game observations; `tooling/acceptance/` collects bounded evidence |
| Prepare package or validate update trust | [Distribution](engine/distribution/README.md) | Machine-readable contracts in `engine/distribution/` and Tauri package config |
| Resume work without the previous chat | [Development Planning](planning/development.md) | Current `Local` source + explicit proof records |

Repository development does not implicitly start VM operations or change a user's guest disks.

## Root file ownership and navigation

The root is a thin entry/configuration boundary, not a second home for VM lifecycle policy or active task state.

| Root file | One responsibility | Detailed owner |
| --- | --- | --- |
| `README.md` | Human overview and navigation | Domain READMEs |
| `AGENTS.md` | Agent mode/task routing and invariants | Selected procedure, source owner |
| `GITHUB_RULES.md` | Exact-ref GitHub write, proof and STOP policy | GitHub source/commit |
| `CONTEXT.md` | Stable product/architecture facts | Source, not progress |
| `CONTRIBUTING.md` | Contribution expectations | Development discipline |
| `SECURITY.md` | Repository-wide trust and secret boundaries | Backend/Guest Agent and distribution owner |
| `package.json` | One set of repository command entrypoints | `tooling/`, `apps/desktop/`, `engine/runtime-core/` |
| `toolchain.json` | Canonical developer tool and proof policy | `tooling/windows-toolchain/` |
| `.node-version` | Node version selector | `toolchain.json` |
| `VERSION` | Product release version text | Desktop/Tauri/Rust versions must agree |
| `DEV.cmd` | One Windows developer entrypoint with portable bootstrap | `tooling/windows-toolchain/bootstrap.ps1`, `dev.ps1` |
| `.editorconfig`, `.gitattributes`, `.gitignore` | Editing, Git and generated-data boundaries | No executable VM semantics |

The root **does not** need a `tsconfig.json` or `package-lock.json`: the only Node dependency installation belongs to `apps/desktop/`. Rust dependencies are locked separately in `engine/runtime-core/Cargo.lock` and `apps/desktop/src-tauri/Cargo.lock`. Avoid duplicate manifests just for visual symmetry with another repository.

## Canonical repository domains

```text
.agents/                 bounded developer-agent procedure
.github/                 automatic Local Full Source CI; manual legacy package smoke
apps/desktop/            Svelte UI, Tauri command adapters, native window layout
engine/runtime-core/     authoritative Rust VM lifecycle, identity, provider and guest protocol
engine/guest/windows/    Windows Base and Guest Agent install/uninstall/finalize scripts
engine/distribution/     installer/update contracts, release manifest and lifecycle rules
docs/product/            Base preparation and user-facing setup expectations
docs/system/             architecture, runtime semantics, ownership and developer procedure
docs/validation/         proof policy, failure matrix and platform constraints
planning/                current development intent and continuation
tooling/repository/      static repository/source contract verification
tooling/windows-toolchain/ portable Windows developer setup and launcher
tooling/acceptance/      bounded Windows host/guest evidence collection
workspace/               one local root for toolchains, downloads, caches, builds, packages and test runs
experiments/             bounded non-authoritative research
```

All human-facing documentation routes start at `docs/README.md`. Product policies are implemented by their source owner; documentation must not create another runtime state machine.

## Development verification

```bash
npm run setup                    # optional source-toolchain command; not required for ChatGPT + GitHub edits
npm run verify:repository        # static repository and cross-file contract verification
npm run test:feasibility          # pure Node evidence classification regressions
npm run test                     # desktop contract tests
npm run check:desktop            # Svelte, frontend, desktop/Tauri source checks
npm run check:runtime            # Rust canonical runtime checks
npm run test:runtime             # Rust canonical runtime tests
npm run format:runtime           # Rust format check
npm run verify:source            # complete source checkpoint
npm run dev                      # Tauri desktop development
npm run package:windows          # BLOCKED for native pilot: old VMware bundle still attached
```

For the native Windows pilot, use `DEV.cmd bootstrap` to provision pinned developer tools beneath `workspace/`, then `DEV.cmd doctor` to confirm local executable paths, `DEV.cmd preflight` to run exact-SHA locked source checks, and finally `DEV.cmd dev` for the developer UI. For UI-only startup, use the exact-SHA [CI debug candidate](docs/validation/ci-debug-candidate.md) without installing a developer toolchain. `DEV.cmd g0` and legacy packaging stay blocked. GitHub `Local` pushes run automatic Full Source CI and conditional Windows debug compilation; manual scopes remain available. CI compilation is **not** proof of GUI startup, portable bootstrap execution, independent Minecraft sign-in or four-client gameplay.

## Product and compatibility invariants

- Svelte is presentation; thin Tauri commands route to the Rust runtime authority.
- **Active pilot:** one official host-native Windows client plus one Sandboxie Plus Client 02; neither account independence nor render/playability is proven. VMware Workstation/Fusion implementations remain **inactive legacy backend source**, not the active native pilot provider.
- Native/Base/Virtual Minecraft version parity and finalized Base lineage gate VM start/provisioning.
- Each Virtual retains its own guest disk and account state; destructive reprovision requires explicit confirmation.
- VM running, Guest Agent reachable, game process running, sign-in usable and multiplayer-ready are different observations.
- Legacy installed product name **M-Bedrock Virtual Clients**, Windows application identifier, guest/protocol identities, persisted runtime paths and release-asset naming remain compatibility contracts. **Repository/source-path cleanup does not rename installed artifacts.**
- The current four-client target is **not yet proven on a real machine**, and Client 03–04 have no approved implementation approach. Consult [Validation](docs/validation/README.md).

## Extraction history

This dedicated repository originated from Virtual Clients in `MIVUBI-STD/M-Lazy-Developer` `Experimental`, exact source SHA `89cb8691f6757d287707bb8cae2d0d810b751404` (2026-10-10). Repository operating discipline is adapted from `M-Lazy-Developer/Local`. This history is provenance, **not** a dependency on the old monorepo.

No account tokens, licensed game binaries, VM images or client worlds are included.
