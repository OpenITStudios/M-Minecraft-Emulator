---
name: emulator-development
description: GitHub-first native Lazy Tester development; preserve account/world data and exact proof boundaries.
---

# Emulator Product Development

Applies to system development, refactoring, bug fixes and documentation that change this repository itself.

## Route

1. Read root AGENTS.md and GITHUB_RULES.md; pin M-Minecraft-Emulator/Local HEAD.
2. Identify the requested product outcome, non-goals, first affected owner and consumers. Use docs/system/implementation-map.md, then fetch only the relevant source.
3. Separate facts observable in source from unexecuted tests and Windows/VMware/Minecraft acceptance. Existing successful checks at a different SHA are not evidence for the current change.
4. Consider no change, correction of the existing owner, reuse, and only then a new component. Never add a second lifecycle/readiness manager, provider state store, account automation or speculative compatibility alias.
5. Implement one coherent change; keep guest disks, snapshot semantics, installer identity and release policy stable unless explicitly in scope with compatibility proof.
6. Verify at the lowest sufficient level: exact-head structural and consumer review first; executed tests/builds only when an appropriate environment is actually available. Do not ask for a user PC to finish GitHub-verifiable source work.
7. Commit to Local, inspect exact new HEAD, update planning/development.md for a meaningful continuation, report proof ceiling and STOP. Do not invoke or reintroduce GitHub Actions: CI was explicitly disabled by the operator. Manual scripts are optional, not a blocking prerequisite.

## Canonical owners

- Native game lifecycle and identity: Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/ and process_identity.rs.
- Platform-neutral contracts and bounded process helpers: Development/engine/runtime-core/.
- Machine-readable installer/release contracts: Development/engine/distribution/.
- UI/typed Tauri bridge/windows: Development/apps/desktop/.
- Runtime test collection: Development/tooling/acceptance/windows/ plus docs/validation/.
- Build and source checks: Development/tooling/repository/; GitHub Actions remain absent.
- Stable policies: docs/system/; current progress: planning/development.md.

An automatic workflow, new release, destructive reprovision or promotion to main is never implied by this procedure.
