---
id: document.system.source-dependency-graph
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Source Dependency Graph — selective impact analysis

This is an **opt-in, read-only, source-only DERIVED projection** for the
Emulator's existing repository knowledge navigation. It follows the
source-dependency/provenance ideas reviewed in M-Lazy-Developer
`Local` at `0346a09603a9d5e8fc141b6f6be972e8b77e0117` but adapts the
actual Rust, TypeScript, JavaScript and Svelte codebase. It does not add
an agent, API, controller, daemon, cache, state store or native game hook.

## Responsibility and existing source owners

- **Documentation Router/Catalog/Graph:** [repository knowledge graph](repository-knowledge-graph.md)
  and `Development/tooling/repository/knowledge-graph.mjs`. Its Markdown
  relationships are intentionally **not** source dependencies.
- **Source declaration index:** [source-dependencies.mjs](../../Development/tooling/repository/source-dependencies.mjs)
  computes file-level literal declaration edges from current owned source.
- **Affected-file reasoning:** [source-reasoning.mjs](../../Development/tooling/repository/source-reasoning.mjs)
  traces forward paths and reverse imports with source witnesses.
- **Read-only command entry:** existing `Development/tooling/repository/knowledge-query.mjs`.
- **Canonical implementation owners:** [implementation map](implementation-map.md)
  and the live source itself. A derived file ID is **not** a replacement
  owner registry or an authorization to change native code.
- **Current work state:** `planning/development.md` (one existing handoff).

Source graph IDs such as `source.file.development-apps-desktop-src-main-ts`
identify **exact currently indexed source files**. A rename intentionally
changes a file's derived ID. Unlike stable document IDs, source-file paths
are identity for this narrow graph. No persistent snapshot is maintained.

## Languages and evidence ceiling

| Language | Extracted source declarations | Boundary |
| --- | --- | --- |
| TypeScript / JavaScript | AST static imports, re-exports, type imports, literal `import()`, literal `require()` | Uses the already-declared desktop `typescript` devDependency, **only when actually installed** |
| Svelte | Script block AST + TypeScript AST imports within scripts | Uses existing `svelte/compiler` and `typescript` dependencies, not markup/DOM usage analysis |
| Rust | Conservative, comment/string-aware lexical `mod foo;`, literal `#[path]`, `use crate::...` / `use super::...`, and literal `include!` | **Not** a complete Rust AST, macro expansion, feature-gate evaluation or rust-analyzer |
| Local Cargo path dependencies | Explicit first-party `Cargo.toml` inline-table `path` declarations | Separate **crate-level** potential consumer projection, not function/file call edges or Cargo feature resolution |
| C++, PowerShell, other Cargo package resolution, assets | Not traversed | No false claim of full repository coverage |

The input is limited to tracked-style code under `Development/apps/desktop/src/`,
`src-tauri/src/`, `apps/desktop/scripts/`, `engine/runtime-core/src/`,
and `Development/tooling/`, with an explicit source path for the native
Tauri `build.rs`. The reader does not follow symlinks or inspect ignored
`App/`, `Data/`, accounts, worlds, credentials, logs, or binaries.
No additional npm package is introduced, installed or executed.

Each admitted edge is `DEPENDS_ON` with `EXTRACTED_SOURCE_DECLARATION`
and **path, line, column, literal specifier and syntax** witnesses. Distinguish:
- observed declaration ≠ function call, used symbol, compiled target, executed code;
- Rust `#[cfg]` and feature-specific declarations ≠ universally active build path;
- source fixture ≠ test executed;
- runtime input ACK ≠ Minecraft game consumption;
- reverse import reachability ≠ certain regression.

Resolution is conservative. A relative `.js` specifier may correspond to a
real `.ts` source in this repository. When both valid candidates exist,
the resolver returns `AMBIGUOUS_SOURCE_TARGET` rather than guessing.
Calculated dynamic imports, unresolved relative modules, missing parsers and
unsupported source syntax remain explicit unresolved/gap evidence.
External package imports and non-code resources are skipped and counted.
Known first-party Cargo `path` dependencies are separately attached to
potential package-level impact reports, without manufacturing importer
relationships to every Rust file. Large source files beyond the read-only
2 MiB bound remain explicitly unparsed.
A path with no observed importers is **not** guaranteed safe to change.

## Queries for architecture-first development

The existing CLI can now be used as follows. These are optional,
**read-only local developer commands**; the GitHub ChatGPT source workflow
remains independent of a local checkout or tool installation. Do not run
these as proof unless execution is separately authorized.

~~~text
node Development/tooling/repository/knowledge-query.mjs --source-summary
node Development/tooling/repository/knowledge-query.mjs --source-id Development/apps/desktop/src/main.ts
node Development/tooling/repository/knowledge-query.mjs --impact Development/apps/desktop/src/app/bridge/lazyTester.ts
node Development/tooling/repository/knowledge-query.mjs --impact Development/tooling/agent --max-hops 4 --format markdown
node Development/tooling/repository/knowledge-query.mjs --source-trace Development/apps/desktop/src/main.ts Development/apps/desktop/src/LazyTesterApp.svelte --format markdown
node Development/tooling/repository/knowledge-query.mjs --source-graph --format mermaid
~~~

`--source-summary` reports parsed file/edge counts, parser availability,
unresolved specifiers, omitted languages, and unparsed source diagnostics.
`--source-id` provides incoming and outgoing declaration witnesses.
`--impact` starts only from an **existing indexed source file / directory
or exact source ID**, and reports potentially affected importers and separate local Cargo dependents at bounded
depth (1–6 hops) including test fixtures as **AUTHORED_NOT_EXECUTED**.
`--source-trace` reports an observed dependency path and its source
locations or `NO_PATH`. A diagram is a Mermaid/Pele-compatible projection
and Markdown includes optional Obsidian vault-root paths. Nothing writes
a vault, GitHub file, diagram cache or persistent architecture database.

The TypeScript/Svelte parser dependencies are declared in
`Development/apps/desktop/package.json`. If a developer environment does
not have them installed, the source query must show
`INCOMPLETE_PARSER_COVERAGE` instead of silently pretending the graph
is complete. The code cannot certify import correctness against all TS
path mappings, module resolution conditions or compiler versions.

## Default agent development decision sequence

When evaluating a requested change:

1. Pin `Local` HEAD; read [root Agent Rules](../../AGENTS.md) and
   [GitHub Rules](../../GITHUB_RULES.md) if not already read for this SHA.
2. Route task through the documentation owner and
   [implementation map](implementation-map.md). **Current source wins**.
3. Select the exact changed file(s) and perform a conservative potential
   impact analysis when possible; inspect source witnesses, unresolved
   inputs and compilation/feature conditionality.
4. Review direct consumers and relevant authored fixtures; propose a
   minimum complete correction in the actual owner. Do not create a
   second controller, importer manager, persistent graph or checklist.
5. Carry out only approved source edits, optional tests only if newly
   authorized, report actual evidence ceiling, then STOP.

When ChatGPT has only GitHub access, perform the same checks by direct
bounded source reads and imports; do not imply that the optional Node
query ran. No GitHub Actions, local tests, Windows/game acceptance,
new subagents, provider switch or installed app state is introduced.
