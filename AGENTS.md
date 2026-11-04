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
has already been removed from Local. **Operator scope reset — 1 November 2026:**
while the operator cannot access the Windows PC, develop the SAME
experimental native input capability for 4–5 AI clients at once rather
than scaling MC02 source later. The two modes remain MC01 human +
MC02–MC05 AI, or all five clients AI. Never route an AI action to human
MC01 in hybrid mode. Retain one authenticated controller, one separate
per-process input mapping per opted-in AI client, per-client identity
and ReleaseAll, and one bounded cross-slot scheduler. Keep all new
input source compile-OFF, manually opted-in, and excluded from normal
product builds/packages. No local-PC tests, Windows builds, installer
changes, enabling game hooks or gameplay execution without separate
approval. Whether Minecraft Education consumes these Win32 inputs is
still unverified on MC02 AND all other clients. Neither source-only
work nor IPC ACKs nor hook-return markers are game acceptance.

**Scoped diagnostic authorization — 2 November 2026:** operator
explicitly approved **source-only local-first Minecraft Education
diagnostics**: immutable Data/Diagnostics evidence import, grouped issue
summaries, and bounded read-only Codex/MCP queries. This authorizes no
automatic Education log discovery, game execution, token upload,
debugger TCP connection, CI, installs, changes to the single native
controller, account/world access, or source tests. All live Education
data paths and actor/world attribution remain NOT_VERIFIED. See
docs/system/map-diagnostics.md.

**Superseding team-source decision — 2 November 2026:** The uploaded
minecraft-bedrock-agent-skills team modules own all in-game analytics,
logging and game results. Skills Probe owns runtime validation. Retire
the parallel Lazy Tester Universal Diagnostics collector and
[LT_EDU_DIAG] schema; do not recreate them or a short lt:diag command.
Only the read-only local evidence/MCP consumer and an optional
**data-only Minecraft function help pack** may remain. In-game help
lists the ORIGINAL team commands with /function help (fallback:
 /function lazytester/help if there is a function-name conflict).
Minecraft's built-in /help remains untouched. No Script API
subscription, JavaScript help listener, or second team analytics
engine is authorized. The helper cannot install team modules. Team updates use their
skills:check / skills:sync workflow, not a copied module fork.
Actual Education Windows capability stays NOT_VERIFIED.

**Scoped operator-approved source prototype — 5 November 2026:** A new
experimental, resource-only `education-native-hud-prototype/` is authorized
on the `Local` branch solely to investigate an additive JSON UI sibling immediately after
the existing `player_position` in `hud.root_panel/chat_stack` within
a disposable QA world. Never replace or append children inside the
content-sized native `hud.player_position` image.
It MUST remain uninstalled, unbundled, untested and disabled by default
until separate runtime authorization. The manifest must explicitly declare
`pack_scope: world` (never implicit `any`) and the optional source
preflight must remain read-only, without automatic packaging. It may add only a truthful static
hint, never replace native XYZ/visibility, inject game hooks, claim
dynamic world/player data, offer nonfunctional buttons or change the
existing team modules. A resource-only prototype does not supersede
the exclusive team-owned gameplay/Script API, data-only in-game help
or five-client process authority. See
`docs/system/education-native-coordinate-hud.md`.

**Team source update admission:** When newer team skills/Skills Probe ZIP
snapshots are supplied, use the local read-only review gate described in
`docs/system/education-team-source-updates.md`. No automatic module
synchronization, source vendoring, unreviewed baseline-promotion,
GitHub Actions, Minecraft execution or data upload is authorized.
The team's `modules-lock.json` and `skills-lock.json` remain
the actual module-copy authority; Lazy Tester stores only reviewed
snapshot fingerprints and consumes their bounded evidence.

**Hard scope gate for experimental AI input:** no generic MCP/privacy/parser/observer audits,
second game controller, speculative provider or milestone churn.
Native AI input source work must directly implement the existing per-client
native input route coherently across 4–5 profiles, with only bounded
authored regression fixtures until execution is approved. The separately
authorized local-first diagnostics scope above is not an AI input feature.
No unapproved game hooking, DLL injection, driver installation,
foreground takeover, VM/RDP patching, CI or user-PC execution. Preserve
all native profile/world/account data and normal close contracts.
RAM/FPS exploration and unrelated cleanup remain deferred.
`planning/development.md` owns the single current blocker and next action.
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
TARGET_WINDOWS → actual Windows native launcher, profiles, windows and process observations
MINECRAFT_RUNTIME → live Education sign-in, gameplay, peer-to-peer network
FIVE_CLIENT_ACCEPTANCE → five distinct licensed playable Minecraft Education sessions accepted concurrently on the chosen native approach; former four-client VM evidence is historical
```

Never promote one rung into another. Absence of runtime or source execution evidence is `NOT_EXECUTED`, not PASS.

GitHub/ChatGPT source work is remote-first. Do not demand the user's PC, a checkout, installs, or running Minecraft just to complete source-verifiable repository work. If an observed machine capability is genuinely needed, record the unresolved evidence instead of guessing.

## ChatGPT-GitHub Work Continuity

**Short-prompt repository startup.** A repository/`Local` URL with "amati", "cek repo", or "ikuti aturan" is a read-only inspection, not permission to change code or start VMware. Fetch the latest `Local` HEAD, read `AGENTS.md` and `GITHUB_RULES.md` fully, and route through `docs/system/implementation-map.md` to the minimum current owner. "Lanjutkan" retains only the identifiable user-approved topic.

**Cross-chat recovery.** Recover by topic and source changes: compare `planning/development.md` with the current exact GitHub HEAD and relevant commit `Work` / `State` / `Proof` / `Unresolved` / `Next`. Historical `Next`, unrelated latest commits, chat titles, memory, and Project settings are not implementation truth or new authorization. Respect `PAUSED`/`BLOCKED` until explicitly resumed; if GitHub cannot be inspected, say the state is unverified. Project Instructions, if used, merely point to current repository rules, not duplicate them.

Development uses ChatGPT + GitHub only. **Current operator instruction: keep building source; do not run local tests, source test suites, Windows builds, packages, or Minecraft gameplay until explicit new approval.** Do not assume or propose another execution environment, ask for a user computer, or make incremental machine tests a prerequisite for source work. Unavailable Windows, VMware, Minecraft sign-in/network and five-client proof stays `NOT_EXECUTED`; continue bounded source work when it has independently justified value.

## Operator routes

| Request | Procedure | Owner / artifact |
| --- | --- | --- |
| Improve or repair product, UI, source, packaging code, repo organization | `SYSTEM DEVELOPMENT` → `.agents/skills/emulator-development/SKILL.md` | `docs/system/implementation-map.md`, existing code |
| Diagnose native launch, session, profile or window failures | `DIAGNOSE` | `docs/system/implementation-map.md` → existing Rust session, identity or workspace owner + observed evidence |
| Measure or validate Windows/Minecraft runtime | `RUNTIME ACCEPTANCE` | `docs/validation/README.md`, `Development/tooling/acceptance/windows/` |
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

No duplicate lifecycle authority, remote agent/network gateway, credentials store, or parallel setup-complete flag. The explicitly requested optional local stdio MCP adapter delegates to the same authenticated native session controller; it is not a new game owner or independent-input implementation. Do not alter installed app identity, Windows data paths, snapshots, signed-release format, or public wire schemas as an incidental folder rename.

## Completion rule

Identify one owner, trace consumers, make the smallest complete change, evaluate relevant static/regression evidence, record exact SHA and remaining `UNKNOWN`, then STOP. Avoid cosmetic renames and speculative restructuring.
