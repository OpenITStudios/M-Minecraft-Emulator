# Emulator Agent Routing

This file defines repository-wide routing and non-negotiable boundaries only. The detailed development procedure belongs to `.agents/skills/emulator-development/SKILL.md`, and implementation semantics belong to their actual source owner.

## Priority and branch

Current user intent guides bounded work, subject to source authority,
safety and available proof. **`Local` is the only active working branch for
product development.** `main` remains release-reserved. The operator has
explicitly superseded previous CI approval: **GitHub Actions CI is disabled
entirely**; there must be no workflow definitions on `Local` or `main`,
and no push, manual, schedule, or package job. The workflow registration-only
exception on `main` is retired. This CI shutdown is not permission to
publish product changes, installers, releases or gameplay to `main`.
Source checks remain available only when explicitly requested outside
GitHub Actions. Never silently fall back to `main`.

For material writes read `GITHUB_RULES.md`, pin exact `Local` HEAD, and consult `docs/system/implementation-map.md` before broad searching. `CONTEXT.md` holds stable facts; `planning/development.md` holds current continuation. Current source wins if either document is stale.

## Current product goal and handoff

The active product is **Lazy Tester**, with five independently managed
Minecraft Education Windows profiles. The retired Sandboxie/VMware runtime
has already been removed from Local. Current source work prioritizes one canonical development handoff,
source/validation efficiency and final native Windows acceptance while
preserving session, game, account data and normal-close contracts. RAM/FPS
exploration is deferred until real resource evidence justifies it. MC02 exit
source is not an ongoing P0 blocker; runtime acceptance remains deferred.
`planning/development.md` owns the current blocker and next source action.
Target Windows/macOS platform-neutral adapters remain future work; a build
alone cannot claim multi-account Mac compatibility.

## Current product mode

The native-first five-profile controller, workspace recovery, FPS policy and
reviewed native DLL are the only game/runtime owners. Sandboxie forks and
VMware/Guest Agent source have been retired from the current branch; historical
source is recoverable from Git. Do not revive old providers or assume external
running processes were stopped by removing their source. The legacy installer
is explicitly blocked.

## Execution and evidence

Use the strongest **actually available** evidence context:

```text
REMOTE_GITHUB → exact-source / static contract review
SOURCE_EXECUTION → actually run Node, Svelte, Rust and Tauri checks
PACKAGE_EXECUTION → actual Windows installer construction/inspection
TARGET_WINDOWS → VMware, guest identity, interactive Windows session
MINECRAFT_RUNTIME → live Education sign-in, gameplay, peer-to-peer network
FIVE_CLIENT_ACCEPTANCE → five distinct licensed playable Minecraft Education sessions accepted concurrently on the chosen native approach; former four-client VM evidence is historical
```

Never promote one rung into another. Absence of runtime or source execution evidence is `NOT_EXECUTED`, not PASS.

GitHub/ChatGPT source work is remote-first. Do not demand the user's PC, a checkout, installs, or running Minecraft just to complete source-verifiable repository work. If an observed machine capability is genuinely needed, record the unresolved evidence instead of guessing.

## ChatGPT-GitHub Work Continuity

**Short-prompt repository startup.** A repository/`Local` URL with "amati", "cek repo", or "ikuti aturan" is a read-only inspection, not permission to change code or start VMware. Fetch the latest `Local` HEAD, read `AGENTS.md` and `GITHUB_RULES.md` fully, and route through `docs/system/implementation-map.md` to the minimum current owner. "Lanjutkan" retains only the identifiable user-approved topic.

**Cross-chat recovery.** Recover by topic and source changes: compare `planning/development.md` with the current exact GitHub HEAD and relevant commit `Work` / `State` / `Proof` / `Unresolved` / `Next`. Historical `Next`, unrelated latest commits, chat titles, memory, and Project settings are not implementation truth or new authorization. Respect `PAUSED`/`BLOCKED` until explicitly resumed; if GitHub cannot be inspected, say the state is unverified. Project Instructions, if used, merely point to current repository rules, not duplicate them.

Development uses ChatGPT + GitHub only. Do not assume or propose another execution environment, ask for a user computer, or make incremental machine tests a prerequisite for source work. Unavailable Windows, VMware, Minecraft sign-in/network and five-client proof stays `NOT_EXECUTED`; continue bounded source work when it has independently justified value.

## Operator routes

| Request | Procedure | Owner / artifact |
| --- | --- | --- |
| Improve or repair product, UI, source, packaging code, repo organization | `SYSTEM DEVELOPMENT` → `.agents/skills/emulator-development/SKILL.md` | `docs/system/implementation-map.md`, existing code |
| Review environment, VM provisioning or guest issues | `DIAGNOSE` | Rust runtime/VMware source + observed evidence |
| Measure or validate Windows/Minecraft runtime | `RUNTIME ACCEPTANCE` | `docs/validation/acceptance.md`, `Development/tooling/acceptance/windows/` |
| Windows package / signing / release | `PACKAGE VERIFICATION` | `Development/engine/distribution/`, Tauri package config; manual gate |

A diagnosis does not automatically authorize code changes or a destructive VM action. No operator mode automatically publishes to `main`.

## Canonical owners

- Game startup/session lifecycle and process identity: `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`, `independent_bootstrap/session_controller.rs`, and `process_identity.rs`.
- Platform-neutral domain types and bounded subprocess helpers: `Development/engine/runtime-core/`.
- Native installer/preservation contract: `Development/engine/distribution/lazy-tester-package.json` plus desktop Tauri configuration.
- User-facing UI/typed Tauri bridge/window arrangement: `Development/apps/desktop/`.
- Source verification, optional developer commands and acceptance collectors: `Development/tooling/`.
- Stable architecture and how-to: `docs/`.
- Current work and actual evidence handoff: `planning/`.

No duplicate lifecycle authority, agent/network gateway, credentials store, or parallel setup-complete flag. Do not alter installed app identity, Windows data paths, snapshots, signed-release format, or public wire schemas as an incidental folder rename.

## Completion rule

Identify one owner, trace consumers, make the smallest complete change, evaluate relevant static/regression evidence, record exact SHA and remaining `UNKNOWN`, then STOP. Avoid cosmetic renames and speculative restructuring.
