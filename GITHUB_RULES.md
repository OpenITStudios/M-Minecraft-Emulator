# GitHub Rules — M-Minecraft-Emulator

Canonical GitHub delivery and proof policy. **Development authority:
`Local` only.** Following the operator's 20 October 2026 explicit decision,
**GitHub Actions CI is fully disabled** (not merely changed from automatic
to manual). Workflow definitions are removed from `Local` and the previously
registration-only `main` workflow is retired. No push, manual, PR, schedule,
package-smoke, release or deployment workflow should be reintroduced without
new explicit user approval. This does not authorize product writes on
`main`, releases, driver operations or account/gameplay access.

## Operating sequence

```text
PIN → EXECUTION CONTEXT → READ MINIMUM → DIAGNOSE OWNER
→ BOUNDED CHANGE → VERIFY → COMMIT/HANDOFF → STOP
```

## 1. PIN

Before mutation, establish repository, branch `Local`, HEAD SHA, first semantic owner, current file/blob authority and the write permission. Recheck HEAD before moving a ref. Never silently target `main`, overwrite a concurrent change or trust search/history over current source.

## 2. Execution context and proof ceiling

- `REMOTE_GITHUB` — code review, ownership analysis, Git trees, static policy/consumer checks and source commits. Default ChatGPT context.
- `SOURCE_EXECUTION` — actually executed typecheck, tests, Rust build/format and desktop bundle checks.
- `PACKAGE_EXECUTION` — actual NSIS build, installed payload integrity, install/repair/uninstall test.
- `TARGET_WINDOWS` — actually observed native Windows host, game sessions, private profiles and DWM window behavior; historical VM evidence is separate.
- `MINECRAFT_RUNTIME` — actual Education login, graphics and multiplayer.
- `FIVE_CLIENT_ACCEPTANCE` — five independently licensed playable Minecraft Education clients meet the current accepted target simultaneously on the native approach; earlier four-client VM evidence is historical only.

A static test **authored** is not a test **executed**. A VM process is not game-ready, signed-in, or network-ready proof. Multiple SHAs cannot be combined as exact-head acceptance.

## Retired VM and Sandboxie routes

The legacy VMware, App Sandbox pilot and Sandboxie fork/box implementation
is absent from the active source/IPC graph. It is not a supported recovery
or launch fallback. The old legacy packager remains a hard refusal; source
cleanup must not remove the Native Lazy Tester installer admission safeguards.
Historical Git revisions are provenance, not instructions to reactivate VM
or sandbox operations. Never touch an installed service, VM disk, snapshot,
game account, stored world or running Minecraft process during source cleanup.

## 3. Read minimum

Default work starts with `AGENTS.md` → [Implementation Map](docs/system/implementation-map.md) → one source owner and consumers. Read more only to answer a concrete unresolved question. Relevant source authority outranks stale planning/prose.

## 4. Diagnose the first wrong owner

```text
Game launch/lifecycle/profile issue   → desktop engine independent_bootstrap / session_controller
Identity and native window issue      → process_identity / native_clients / workspace_surface
Session transport / IPC issue         → native_clients_session and its pipe
Bundle/trust/preservation issue       → distribution/lazy-tester-package.json + Tauri source
Portable domain/finite runner issue   → Development/engine/runtime-core/
Source verifier or developer CLI      → Development/tooling/
Historical VM/box behavior            → Git archive only, no executable owner
```

Do not introduce second lifecycle managers, mutable frontend truth or a
desktop IPC route back to retired sandbox/guest code.

## 5. GitHub-first and no-user-PC boundary

Complete all source/static work directly in GitHub when possible. `Local` names a **GitHub branch**, not a physical PC. `npm`, `DEV.cmd`, source tests and native Windows/Minecraft execution are stronger-proof contexts, not gates placed on user workstations to complete ChatGPT repository edits. Keep truly unresolved runtime requirements `NOT_EXECUTED`.

### Short-prompt repository startup and topic recovery

For a GitHub URL + "amati"/inspect request, pin `Local` HEAD, read this file and `AGENTS.md`, resolve `docs/system/implementation-map.md`, inspect the relevant current owner, then report confirmed facts and specific `NOT_EXECUTED` proof without writing. Never infer an authorization from a historical `Next`, an unrelated recent commit, or a chat title.

To continue work in another chat, recover the named topic from `planning/development.md` and relevant GitHub commits. Compare their decisions and source claims against newer changes and exact current files. Keep `PAUSED`/`BLOCKED` work inactive until the user resumes it; do not fabricate absent chat context, automatic chat naming, extra memory files or duplicate state owners.

## 6. Mutations, compatibility and commit discipline

Update canonical owner plus necessary callers/tests/docs in a coherent logical commit. Prefer correcting an existing owner over creating another abstraction. Preserve all native profile/world/registry data and retained historical account-bearing guest disks or snapshots; none may be deleted during routine source work. Native release/update identity, installed product identifiers, stored profile layouts and historical Guest Agent formats require explicit compatibility review before any migration.

When composing GitHub source-file replacements in JavaScript, preserve literal replacement bytes: `String.replace(search, replacementString)` interprets `$'`, `$&` and related sequences, which can silently corrupt PowerShell regex or source text. Use a callback replacer (`replace(search, () => replacement)`) or exact splice, then fetch the committed blob and verify the complete affected expression and file ending. A successful GitHub write is not syntactic proof.

Use categorized commits (`refactor(repo): ...`, `fix(runtime): ...`, `test(desktop): ...`, `docs(system): ...`). Record continuity at significant milestones in `planning/development.md`. Git history preserves old source naming; do not leave obsolete code mirrors just for traceability.

### Commit-based continuity

For each material bounded commit, use one conventional subject and truthful revision-scoped trailers:

```text
Work: <stable feature or task topic>
State: ACTIVE | BLOCKED | PAUSED | DONE
Decision: <canonical owner / reason, when material>
Proof: <actual evidence and execution ceiling>
Unresolved: <specific unknown, when applicable>
Next: <one action only when incomplete>
```

`Work`, `State` and `Proof` are required going forward. `DONE` closes the specific commit outcome, not the entire runtime/product. Git history records past decisions; `planning/development.md` owns current development intent and handoff, not a second implementation-status database. No checkpoint commits, automatic chat naming or duplicated TODO files.

## 7. Verification without CI

Use the lowest sufficient available evidence: exact-HEAD static source /
contracts → optional tests in a genuinely available execution environment →
real Windows/gameplay acceptance only when observed. **No GitHub Actions
workflows or CI jobs are enabled**, including manual or package-smoke.
Do not create a new workflow, hidden schedule or alternative hosted runner
to replace CI. The repository's pinned Node/Rust/Powershell scripts may be
run manually when the user explicitly asks and a compatible execution
environment is available. Do not block GitHub source commits waiting for
a runner, or claim authored tests as executed. The user's choice to turn off
CI does not permit deleting safety checks, weakening code review, or
pretending five Minecraft instances or FPS limiting have been validated.

**G0–G4 in `docs/validation/acceptance.md` are historical VMware acceptance gates, NOT current native-engine progress.** Track live native product outcomes and a single current next action only in `planning/development.md`; do not create another TODO/status system. Static checks are supporting evidence, never gameplay acceptance; old CI proves only its original SHAs.

## 8. Security

No VM images, Microsoft credentials, OAuth tokens, machine-identity raw values, Guest Agent secrets, account-bearing snapshots, private worlds or signing material in Git history. Retired Guest Agent and VM providers remain inactive; ambiguous native session, profile or installer mutations fail closed. The installer must not silently delete runtime data.

## 9. Failure policy

On an unresolved change or failing proof: preserve last good source, identify first wrong owner, report exact failing claim and environment. Do not bypass checks, weaken security gates, fabricate successful runs or modify tests solely to make an incorrect implementation appear valid.

## 10. STOP

After the requested bounded repository outcome: confirm the exact `Local` commit, source change scope, applicable static checks, genuinely executed checks (if any), and remaining higher-context evidence. Stop rather than expanding into speculative feature work.
