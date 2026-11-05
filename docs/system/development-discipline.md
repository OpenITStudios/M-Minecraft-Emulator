---
id: document.system.development-discipline
class: DOCUMENT
domain: system
role: CONTRACT
authority: CANONICAL
lifecycle: ACTIVE
---

# Development Discipline

Mandatory for repository/system development, adapted from the owner-first, minimum-complete approach of M-Lazy-Developer/Local.

## Operating standard

1. **Normalize user intent:** define the concrete goal, constraints, success checks and non-goals. Product requests are not automatic authorizations for changes to installed Minecraft/profile data or releases.
2. **Pin authority:** read exact `Local` HEAD, use `AGENTS.md`, `GITHUB_RULES.md` and [Implementation Map](implementation-map.md), then inspect one owning implementation and its consumers.
3. **Ground every claim:** distinguish current source fact, inferred risk, executed test, and real VMware/Minecraft runtime observation. Historical notes and test definitions cannot substitute for a passing run.
4. **Choose the smallest complete solution:** evaluate no change → fix existing owner → remove dead code → reuse → minimal extension → new component only if proven necessary. Do not add a new manager, alias, registry or helper merely to make the folder tree seem symmetrical.
5. **Maintain one name per function:** follow [Canonical Naming](canonical-naming.md). Preserve installed app, account/world data, persisted schemas and release compatibility unless their change is explicitly approved and proven.
6. **Work in coherent batches:** update the owner, imports, consumers, tests, docs and verification routes in one logically reviewable commit. Do not leave half-migrated paths or add compatibility mirrors for retired source names.
7. **Verify proportionally:** use exact-tree/consumer checks and positive/negative regression cases first. Executed Node/Rust/Tauri tests require actual execution evidence. Package/Windows/game claims require corresponding environments. GitHub Actions remain disabled.
8. **Stop with truth:** record the commit, changed owner, verified claim, proof ceiling and next necessary task in `planning/development.md` at a meaningful handoff. Do not continue cosmetic or speculative work after the requirement is met.

## Evidence-grounded source improvement (Harness² / DeepSeek Harness / REA)

Use the existing `DEV.cmd review --json` source-only route when a reviewed
local developer toolchain is available; GitHub-only analysis follows the same
policy directly without needing a local command. Review is a **plan**, not a
grading result, live agent, verified compiler result, second test runner, or
Minecraft controller.

1. **Propose (Harness² pattern):** start with the exact Local SHA, first
   canonical owner and its direct consumers. Compare *no change*, correction
   of existing code, reuse, and only then a minimal new seam. Record at most
   three materially different proposals; avoid speculative parallel work.
2. **Probe:** challenge each proposal against real source invariants,
   negative/rollback cases, permission/privacy bounds, changed dependencies,
   and documented test coverage. Separate `OBSERVED_SOURCE` from
   `INFERRED_RISK`; an authored test is `AUTHORED_NOT_EXECUTED`.
   Failed or missing checks are never scored as PASS, and the reviewer must
   not call a source-only plan a benchmark improvement.
3. **Compose:** adopt the smallest supported solution *only after operator
   authorization for changes*. A parallel agent may review another candidate,
   but one original canonical owner integrates the final change. Preserve
   unresolved/blocked evidence and exact SHA; commit one coherent batch.
4. **Optional REA:** use only on developer-owned or explicitly authorized
   artifacts when ordinary source review cannot answer a concrete question.
   Pin target SHA/digest, tool/provider versions, evidence IDs, coverage,
   limitations, and distinguish static observations from native runtime.
   REA never gains Minecraft credentials, live game/process control, or
   a place in the production launcher.
5. **DeepSeek role model:** delegate *source analysis*, *architecture review*,
   and *negative QA review* as bounded optional **developer-side** roles,
   with tools least-privileged and no implicit execute/write powers. This
   borrows its scoped capability design; no Cordis/plugin runtime is
   bundled in Lazy Tester. MCP `mcp.mjs` and Rust session controller
   retain their established owners.

The `review` envelope contains area/risk prompts, selected validation task
IDs, input provenance, and explicit `NOT_REVIEWED`/`NOT_EXECUTED` states.
It deliberately cannot produce a winner, execute tests, install upstream
projects, adopt REA output, or elevate QA. Existing `probe_qa` and
`DEV.cmd plan` remain separate, canonical owners.

Upstream *design references* (not dependencies, endorsements, or installed
versions): [Google Harness²](https://github.com/google-research/harness2/tree/6868817518876de8fd170c6a8354daca273b6191)
(Apache-2.0), [DeepSeek Harness](https://github.com/deepseek-ai/deepseek-harness/tree/fcc0ad462172522aaf5c93a7b2e01f6b9f0b0159)
(MIT, developer preview), [REA](https://github.com/morluto/rea/tree/71f680d7c1aa7a45ba0325fe87b104744b0c105f)
(MIT, authorized analysis only). Importing code or enabling an external
runtime requires a separate compatibility, dependency, security and
approval review. No CI/workflow is reintroduced.

## Repository completion preflight

Before committing, answer:

- Does this path or responsibility already exist? If so, why isn't an edit to it sufficient?
- Did an existing import, Rust path dependency, Tauri resource, release manifest, test, workflow or documentation link break?
- Were source paths confused with installed paths (native payloads, binary names, Windows profile storage)?
- Is the strongest reported evidence actually available, or only described in code/tests?
- Does the commit update the single continuation owner without creating another backlog?

## Proof and security

No unattended manipulation of account/world data or historical guest disks, no Microsoft credential automation, no fictitious GPU/FPS knobs, no blind retry after ambiguous native session or storage mutation, no automatic publish, and no user-PC test requirement for ChatGPT-owned source work.

A source-only commit may be **complete as source work** while `SOURCE_EXECUTION`, `TARGET_WINDOWS` or `FIVE_CLIENT_ACCEPTANCE` remains `NOT_EXECUTED`. Do not blur that boundary.
