# Dependency maintenance tooling

Root `package.json` is the command entrypoint. Application dependencies remain
in `apps/desktop`; desktop Tauri and runtime-core retain independent Cargo
manifests/locks. `toolchain.json` owns developer tool versions. Both package engine
declarations match the supported Node LTS floor and pinned npm. Setup and upgrade
installs use --engine-strict to reject unsupported runtimes.

| Command | Purpose |
| --- | --- |
| `npm run check:dependencies` | Install-free npm manifest/lock ownership, exact versions, aliases and platform integrity |
| `npm run check:structure` | Tracked output, root ownership, simple internal Markdown links and command paths |
| `npm run audit:dependencies` | npm tree/outdated/security plus both Cargo duplicate/advisory audits; requires installed dependencies and cargo-audit |
| `npm run upgrade:frontend` | Read official stable Vite/plugin/Vitest metadata without editing |
| `npm run upgrade:frontend -- --apply` | On a clean `Local` checkout with pinned tools, resolve the coupled group through npm and validate it |

`npm outdated` exit 1 means update candidates, including deliberately retained
compatibility versions. Audit failures and missing tools propagate after all
independent graphs have been checked. RustSec informational warnings still need
review even when cargo-audit exits successfully. No command enables CI.

For a new migration, inspect registry releases, peer requirements, crate MSRV and
actual source consumers first. Resolve npm and Cargo locks with their package
managers; never transplant locks or invent integrity metadata. Update coupled
frontend packages together. Tauri API, CLI and Rust crates release independently:
use supported versions, rather than requiring identical patch numbers.

TypeScript 7 is installed through the official stable alias
`@typescript/native = npm:typescript@7.0.2`. `typecheck` uses the native checker;
`typecheck:compat` retains TypeScript 6 for the Svelte JS API and compatibility
investigation. Aggregate verification runs the primary checker once.

For Rust migrations, run each manifest's `cargo check --locked --all-targets` and
`cargo test --locked --all-targets`, then affected desktop tests and builds.
The existing desktop source route also checks Rust formatting. Keep platform TLS
trust, response limits, timeout bounds and process start identity intact. File
locks use stable std::fs::File APIs with typed WouldBlock handling and bounded
journal retries; a separate locking dependency is unnecessary. Native snapshot
upgrades require their own migration; they are not routine Cargo updates.

The canonical outcomes, exceptions, versions and evidence are in the
[modernization report](../../docs/validation/dependency-modernization-2026-10-20.md).
