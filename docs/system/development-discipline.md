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

1. **Normalize user intent:** define the concrete goal, constraints, success checks and non-goals. Product requests are not automatic authorizations for destructive VM state changes or releases.
2. **Pin authority:** read exact `Local` HEAD, use `AGENTS.md`, `GITHUB_RULES.md` and [Implementation Map](implementation-map.md), then inspect one owning implementation and its consumers.
3. **Ground every claim:** distinguish current source fact, inferred risk, executed test, and real VMware/Minecraft runtime observation. Historical notes and test definitions cannot substitute for a passing run.
4. **Choose the smallest complete solution:** evaluate no change → fix existing owner → remove dead code → reuse → minimal extension → new component only if proven necessary. Do not add a new manager, alias, registry or helper merely to make the folder tree seem symmetrical.
5. **Maintain one name per function:** follow [Canonical Naming](canonical-naming.md). Preserve installed app, guest data, persisted schemas and release compatibility unless their change is explicitly approved and proven.
6. **Work in coherent batches:** update the owner, imports, consumers, tests, docs and verification routes in one logically reviewable commit. Do not leave half-migrated paths or add compatibility mirrors for retired source names.
7. **Verify proportionally:** use exact-tree/consumer checks and positive/negative regression cases first. Executed Node/Rust/Tauri tests require actual execution evidence. Package/VM/game claims require corresponding environments. GitHub Actions are manual-only.
8. **Stop with truth:** record the commit, changed owner, verified claim, proof ceiling and next necessary task in `planning/development.md` at a meaningful handoff. Do not continue cosmetic or speculative work after the requirement is met.

## Repository completion preflight

Before committing, answer:

- Does this path or responsibility already exist? If so, why isn't an edit to it sufficient?
- Did an existing import, Rust path dependency, Tauri resource, release manifest, test, workflow or documentation link break?
- Were source paths confused with installed paths (guest scripts, binary names, Windows disk state)?
- Is the strongest reported evidence actually available, or only described in code/tests?
- Does the commit update the single continuation owner without creating another backlog?

## Proof and security

No unattended manipulation of account-bearing guest disks, no Microsoft credential automation, no fictitious GPU/FPS knobs, no blind retry after ambiguous VMware mutation, no automatic publish, and no user-PC test requirement for ChatGPT-owned source work.

A source-only commit may be **complete as source work** while `SOURCE_EXECUTION`, `TARGET_WINDOWS` or `FOUR_CLIENT_ACCEPTANCE` remains `NOT_EXECUTED`. Do not blur that boundary.
