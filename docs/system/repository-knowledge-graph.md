---
id: document.system.repository-knowledge-graph
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Repository knowledge navigation: Router, Catalog, Graph, Retrieval, Context

This document owns **documentation/architecture navigation policy**, not Minecraft process
authority, test execution, a new agent runtime, persistent status, or source-code ownership.
It adapts the existing M-Lazy-Developer Local architecture at commit
0346a09603a9d5e8fc141b6f6be972e8b77e0117. Its source of truth is this
repository's exact current Local ref, never a live cross-repository dependency.

## Deterministic path

~~~text
Task/intent
  -> docs/README.md (one domain)
  -> domain README (one canonical concern)
  -> Resource Catalog (stable document ID, location and authority)
  -> Graph (explicit authored navigation + witness)
  -> scoped lexical Retrieval / section selection
  -> source owner via implementation-map.md
  -> Context (source vs history vs unknown distinguished)
  -> decision / optional change / proof ceiling / STOP
~~~

Stable `id` frontmatter is identity; path is location. A path relocation is
not a new authority. One document remains the physical source for its derived
H2/H3 section resources. Derived sections are retrieval addresses, not
authoritative copies. Avoid broad repository scans when the relevant owner
is already known. This tool does not replace the
[implementation map](implementation-map.md) or
[Agent Rules](../../AGENTS.md).

## Resource, evidence and authority contract

| Resource | Authority | Meaning |
| --- | --- | --- |
| `DOCUMENT` with `CANONICAL` / `ACTIVE` | Stable human contract | Current architecture/routing rule |
| `SOURCE` with `CANONICAL` / `ACTIVE` | Actual file | Source location; correctness still requires code examination |
| `DOCUMENT` with `REFERENCE` | Reference only | Legacy `WORKING` metadata is **not** silently promoted |
| `DOCUMENT` with `HISTORICAL` | Historical only | No current launch or implementation authority |
| `DOCUMENT` with `SOURCE_ONLY` lifecycle | Active source ceiling | May guide source work, never proves game/runtime |
| `DOCUMENT` with `DRAFT` lifecycle | Draft/reference only | Cannot authorize production behavior |
| `DOCUMENT` with `DERIVED` | Derived section | Address for retrieval, never independent policy |

Existing Emulator documents also use role labels such as `REPORT` and
`PROCEDURE`, legacy `WORKING` authority, and `SOURCE_ONLY`, `DRAFT`
or `HISTORICAL` lifecycle. The tool retains declared values. `WORKING`
normalizes to `REFERENCE`; `SOURCE_ONLY` remains navigable as active
source-level guidance **without runtime proof**; `DRAFT` is
non-authoritative; `HISTORICAL` / `RETIRED` remain historical.
This does **not** mass-rewrite old documents or reactivate old CI, VMware
or Sandboxie designs.

The repository graph is a derived read-only projection of **authored Markdown
and exact-path Obsidian wikilinks** plus source-proven H2/H3 hierarchy:
`ROUTES_TO`, `RELATES_TO` and `DERIVED_FROM`. Each edge retains
`path`, `line` and literal `target`. Fences, inline code, comments, images,
external URLs, ambiguous Obsidian aliases and missing physical targets
cannot create invented edges. No code symbols, Rust calls, TypeScript
imports, UI bindings, runtime sessions or gameplay are inferred from document
links. Missing relationships stay unresolved.

### Keep three graphs separate

1. **Documentation/navigation graph** — built here from authored route links.
2. **Repository source-dependency / affected-work graph** — requires separate
   actual source analysis; the current CLI makes *no* import/impact claim.
3. **Minecraft/runtime/process graph** — must use original Rust session,
   identity, IPC and verified Windows/game observations; never derived from
   this repository documentation graph.

Do not treat navigation reachability as functional dependency or a PASS
result. Caching and persistent graph exports are deliberately absent:
the graph is computed from the current read-only checkout; it cannot become
a second state database. CI remains disabled.

## Read-only query surface

The single optional existing-tooling owner is
[knowledge-graph.mjs](../../Development/tooling/repository/knowledge-graph.mjs),
exposed by [knowledge-query.mjs](../../Development/tooling/repository/knowledge-query.mjs).
These commands are **optional for local developers**; ChatGPT GitHub-only work
can follow the same exact links and file metadata without a local checkout.
The operator's prohibition on source-test execution remains binding.

~~~text
node Development/tooling/repository/knowledge-query.mjs --summary
node Development/tooling/repository/knowledge-query.mjs --id document.system.architecture
node Development/tooling/repository/knowledge-query.mjs --query "session" --domain system
node Development/tooling/repository/knowledge-query.mjs --trace document.docs.router document.system.repository-knowledge-graph
node Development/tooling/repository/knowledge-query.mjs --trace document.docs.router document.system.repository-knowledge-graph --format markdown
node Development/tooling/repository/knowledge-query.mjs --graph --format mermaid
node Development/tooling/repository/knowledge-query.mjs --audit
~~~

The Markdown trace emits root-relative `[[path|stable-id]]` links for an optional
Obsidian vault (repository root is the vault root) and a Mermaid diagram that
can be inspected with compatible diagram tooling, including Pele workflows.
It **does not install or invoke Obsidian, Pele or Graphify**. Graphify-like
query-first navigation inspires the design, but this implementation does not
claim full Graphify AST parsing, clustering, embeddings, HTML visualization,
MCP, watchers, automatic exports, or change-impact analysis. Those require
separate justification and explicit permission, particularly if they introduce
new dependencies or mutation.

For CI-free review, `--audit` reports metadata/route integrity and unresolved
paths without writing outputs or changing tracked source. A static audit is
not an executed test, Windows acceptance or Minecraft proof.

## Method selection and constraints

The seven reference methods from M-Lazy-Developer are selected by task:
Karpathy + Ponytail for minimal justified implementation, Pstack conditional
playbooks, Harness² for isolated measured experiments only when separately
authorized, DeepSeek Harness as lifecycle/tool-scoping guidance, REA only for
rights-verified investigation, and UI UX Pro Max only for genuine UI work.
These are reference methods; no third-party code, agent, tool, model,
background worker, CI, or plugin is installed or activated.

The exact upstream provenance is linked in
[the root agent policy](../../AGENTS.md). For native launch/controller
authority consult [architecture](architecture.md); for source routing
consult [implementation map](implementation-map.md); for permissions,
commit proof and no-CI policy consult [GitHub Rules](../../GITHUB_RULES.md).

## Completeness gate

For a change to documentation or this query system, verify by exact-source
review (and optional tests only if separately authorized) that: identifiers
are unique, active routers point to current owners, traces include real source
witnesses, historical and derived material cannot override canonical policy,
unknown links do not turn into edges, source/runtime evidence is never
upgraded, and no executable runtime or account/world data is touched.
Do not create a `.context/` status fork, hidden Obsidian symlinks, generated
`graphify-out/` commits, an automatic Graphify watch process, or a
parallel task list; `planning/development.md` remains the sole active handoff.
