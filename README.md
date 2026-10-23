# M-Minecraft-Emulator

**Product goal:** four concurrent, independently playable Minecraft Education **Windows desktop** sessions with four separately licensed accounts on one Windows PC. **Current active development:** an independently authored, Rust-first Windows session engine with bounded Win32 adapters. **Current product limit:** no independent production provider exists; isolated Launch/Provision for MC02–MC04 is deliberately blocked. Native MC01 and inspection/recovery of already-existing historical sessions remain separate operations. The Sandboxie two-client test of 14 October 2026 was limited and later encountered an idle crash; it is NOT acceptance for the replacement. Historical Sandboxie and VMware code remains preserved only for reference and data-safe legacy handling.

## Product definition — integrated Virtual Manager

**MIVUBI is the future manager for independently isolated Minecraft
Education sessions.** Rust runtime-core owns lifecycle and admission;
Windows-native code is a narrow platform adapter, Tauri connects it to
Svelte, and presentation never becomes another lifecycle manager.
Sandboxie-derived driver, service and DLL source is historical and explicitly
not a dependency of the independent engine.

**Current implementation limit:** source-only independent process observers,
storage/registry compatibility diagnostics and image signer inspection exist.
They are not an implementation of separate Minecraft launches, per-client
Microsoft sign-in, or four-session isolation. Create virtual and isolated
Launch/Provision remain unavailable. Existing account/world data and older
sessions are never reset merely to simplify development.

## Branch authority

```text
Local → active development / source authority
main  → reviewed stable/release authority
```

## Historical native source reference — Sandboxie v1.18.6

**Source migration and private native source patch are complete (source-only).**
All 1,627 authentic v1.18.6 files remain untouched at
`engine/isolation-candidates/sandboxie-v1.18.6/`, with pinned Git tree
`f690477c07a40810f479f53973334ceae7e0ab13`.
The retired historical fork is `engine/isolation-owned/`: 1,627 files
with 11 previously modified native files and the upstream license terms
preserved. It is NOT the independent runtime owner and must not be promoted,
rebranded as an independent engine, linked or packaged into the replacement.
The older v1.18.5 tree remains only in Git history. Source-only historical
inspection: `npm run isolation-source:verify`.

**Historical boundary:** no source import, MSVC/WDK build or driver signing
would validate or activate the independent engine. The old installed
Sandboxie environment can be inspected or normally closed for existing
sessions but cannot create a new production sandbox via MIVUBI. Fork
build/profiling commands are retained for provenance only.

**Retired build-only entrypoint:** `tooling/isolation-source/build-owned-v1186.ps1`
is historical source tooling. It was authored without Windows execution and
must not be mistaken for the build or launch path of the independent engine.

**Historical pipeline:** `engine/isolation-owned/` →
`tooling/isolation-source/build-owned-v1186.ps1` →
`tooling/isolation-source/assemble-owned-v1186.mjs` →
`apps/desktop/src-tauri/src/engine/owned_isolation.rs`. These
assets remain inspectable for provenance, but are NOT the replacement.
The new source authority is `engine/runtime-core/src/independent_engine/`
and the existing desktop Windows adapter
`apps/desktop/src-tauri/src/engine/instance_identity.rs`.

## Start here

For ChatGPT + GitHub development, the authoritative route is:

```text
AGENTS.md → GITHUB_RULES.md → docs/system/implementation-map.md
→ exact canonical source owner → bounded source change → evidence review → STOP
```

An operator does **not** need a local checkout to complete repository/source-verifiable tasks. Windows or Minecraft acceptance is a separate, higher proof context. `DEV.cmd` is an optional local developer convenience, never the remote GitHub completion requirement.

**Current P0:** make MC01 + a second independently licensed Minecraft Education MC02 actually playable together, with effective private Roaming/HKCU/auth and persistent worlds. Only then scale to four clients, measure against Sandboxie and distribute. PSEC/fixtures do not constitute a production isolation provider. The [single current handoff](planning/development.md) owns the active blocker and one next action; [Independent Engine Architecture](docs/system/independent-engine-architecture.md) is historical reference, not a second task list.

**Definition of DONE:** [Four simultaneous Minecraft Education accounts](docs/product/four-concurrent-accounts.md). **Technical status:** no four-client gameplay, independent provider or production security acceptance exists. Previous [two-client Sandboxie evidence](docs/validation/two-client-first-success-2026-10-14.md) is explicitly historical. **Current handoff:** [planning/development.md](planning/development.md). **Retirement map:** [Legacy Retirement](docs/system/legacy-retirement.md).

## Current operating mode

The desktop can inspect or recover previously observed legacy sessions,
but **new isolated session Launch/Provision is blocked** on official
Sandboxie and the derivative fork. MC01 native launch is a separate existing
operation. The independent provider is NOT_IMPLEMENTED. No VM refresh,
guest preparation, signed driver install, untrusted profile mutation or
automatic updater runs on startup. Legacy VMware mutations and existing
VMware-oriented packaging remain blocked.

## Choose one work route

| Intended work | Entry | Actual authority |
| --- | --- | --- |
| Develop, refactor, repair, or improve repository | [AGENTS.md](AGENTS.md) → [.agents development procedure](.agents/skills/emulator-development/SKILL.md) | Existing source owner in `apps/` or `engine/` |
| Examine VM behavior or diagnose setup | [Implementation Map](docs/system/implementation-map.md) → [Runtime Core](docs/system/runtime-core.md) | `engine/runtime-core/`, observed provider/guest evidence |
| Develop independent isolation and source-only checks | [Independent Engine Architecture](docs/system/independent-engine-architecture.md) → [Development Planning](planning/development.md) | Rust independent engine policy + existing Windows desktop process adapter; no live game test |
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
.github/                 CI disabled; no Actions workflow files
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

Optional manual source checks are available via `npm run verify:source` and
the pinned Rust/desktop commands above when an authorized execution
environment is available. **GitHub Actions workflows are disabled** by
operator decision; pushing to `Local` does NOT execute CI. Do not create
workflows, run localtest, package installers or activate a driver merely
to claim source completion. Manual source verification is not proof of
Minecraft sign-in, process isolation or four-client gameplay.

## Product and compatibility invariants

- Svelte is presentation; thin Tauri commands route to the Rust runtime authority.
- **Active development:** independent, unmounted Windows engine foundation; isolated MC02–04 Launch/Provision remains blocked pending a verified provider. Historical Sandboxie/VMware remain inactive for new sessions.
- Native/Base/Virtual Minecraft version parity and finalized Base lineage gate VM start/provisioning.
- Each Virtual retains its own guest disk and account state; destructive reprovision requires explicit confirmation.
- VM running, Guest Agent reachable, game process running, sign-in usable and multiplayer-ready are different observations.
- Legacy installed product name **M-Bedrock Virtual Clients**, Windows application identifier, guest/protocol identities, persisted runtime paths and release-asset naming remain compatibility contracts. **Repository/source-path cleanup does not rename installed artifacts.**
- The current four-client target is **not yet proven on a real machine**, and Client 03–04 have no approved implementation approach. Consult [Validation](docs/validation/README.md).

## Extraction history

This dedicated repository originated from Virtual Clients in `MIVUBI-STD/M-Lazy-Developer` `Experimental`, exact source SHA `89cb8691f6757d287707bb8cae2d0d810b751404` (2026-10-10). Repository operating discipline is adapted from `M-Lazy-Developer/Local`. This history is provenance, **not** a dependency on the old monorepo.

No account tokens, licensed game binaries, VM images or client worlds are included.
