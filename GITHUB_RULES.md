# GitHub Rules — M-Minecraft-Emulator

Canonical GitHub delivery and proof policy. **Development and source-verification authority: `Local` only.** `main` is reserved for a future reviewed release, with one **explicitly user-approved** exception (18 October 2026): a registration-only `.github/workflows/verify.yml` to expose manual Actions for testing `Local`. No other `main` changes, automated triggers, default-branch switch or source promotion are authorized.

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
- `TARGET_WINDOWS` — actual host and VMware/Windows guest session observations.
- `MINECRAFT_RUNTIME` — actual Education login, graphics and multiplayer.
- `FOUR_CLIENT_ACCEPTANCE` — four independently licensed playable Minecraft Education clients meet the accepted target simultaneously, whether native or isolated; previous one-Native/three-VM evidence is historical only.

A static test **authored** is not a test **executed**. A VM process is not game-ready, signed-in, or network-ready proof. Multiple SHAs cannot be combined as exact-head acceptance.

## Paused legacy VM routes

The old VMware App, App Sandbox power controls and Guest Agent operations are preserved as source but paused by engine/runtime-core/src/command.rs and not registered in Tauri. Consult docs/system/legacy-retirement.md before any future changes; do not resume VMware onboarding, remove saved guest data or re-enable guest mutations without explicit authorization and real Windows/gameplay evidence. Read-only diagnostics are not permission to mutate VM state.

## 3. Read minimum

Default work starts with `AGENTS.md` → [Implementation Map](docs/system/implementation-map.md) → one source owner and consumers. Read more only to answer a concrete unresolved question. Relevant source authority outranks stale planning/prose.

## 4. Diagnose the first wrong owner

```text
VM lifecycle / version / identity wrong    → engine/runtime-core
Guest agent/Windows setup wrong            → engine/guest or engine/runtime-core guest protocol
Desktop flow/window wrong                  → apps/desktop
Installer/update trust wrong               → engine/distribution + Tauri package consumer
Source verifier or optional CLI wrong      → tooling
Docs navigation/wording wrong              → docs or root router
Missing provider/Windows/Minecraft proof   → actual acceptance evidence, not imaginary source code
```

Avoid second managers, inconsistent API aliases, undocumented state caches, or redundant root files.

## 5. GitHub-first and no-user-PC boundary

Complete all source/static work directly in GitHub when possible. `Local` names a **GitHub branch**, not a physical PC. `npm`, `DEV.cmd`, source tests and local VMware are optional stronger-proof contexts, not gates placed on user workstations to complete ChatGPT repository edits. Keep truly unresolved runtime requirements `NOT_EXECUTED`.

### Short-prompt repository startup and topic recovery

For a GitHub URL + "amati"/inspect request, pin `Local` HEAD, read this file and `AGENTS.md`, resolve `docs/system/implementation-map.md`, inspect the relevant current owner, then report confirmed facts and specific `NOT_EXECUTED` proof without writing. Never infer an authorization from a historical `Next`, an unrelated recent commit, or a chat title.

To continue work in another chat, recover the named topic from `planning/development.md` and relevant GitHub commits. Compare their decisions and source claims against newer changes and exact current files. Keep `PAUSED`/`BLOCKED` work inactive until the user resumes it; do not fabricate absent chat context, automatic chat naming, extra memory files or duplicate state owners.

## 6. Mutations, compatibility and commit discipline

Update canonical owner plus necessary callers/tests/docs in a coherent logical commit. Prefer correcting an existing owner over creating another abstraction. Preserve each Virtual's account-bearing disks and snapshots; reprovision is explicitly destructive and must not be a side effect of routine setup. Release/update identity, Guest Agent wire contract, installed product identifier and disk paths require an explicit migration review, not a source-folder tidy-up.

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

## 7. Verification and CI

Use the cheapest decisive evidence: exact-ref static path/contract review → targeted source test → full source check only when needed → actual package inspection → Windows/VMware acceptance → real Minecraft Education acceptance. **Only G0–G4 in `docs/validation/acceptance.md` are product-test progress gates.** Unit/regression tests, static checks and stage-recorder JSON are supporting evidence, not milestones, percentages or extra gate states. Finish a coherent engineering batch before evaluating affected gates; do not run repeated micro-tests to manufacture progress.

GitHub Actions must remain strictly **manual-only** (`workflow_dispatch`, `permissions: contents: read`). No `push`, `pull_request`, cron, auto-merge, auto-release or deployment. The user approved exactly one **registration-only** `.github/workflows/verify.yml` on default branch `main`, with a permanently skipped job and no checkout. The `Local` workflow owns `quick`, `full-source`, and `windows-build` jobs; **each job must require `github.ref == 'refs/heads/Local'`** in addition to its scope. Users must select `Local` in Run workflow. Selecting `main` must never run builds/tests. Preserve `package-smoke.yml` as historical VMware packaging and do not run it for the native pilot. No additional edits to `main` are permitted. Only executed checks at the exact selected SHA provide test evidence; skipped jobs are not PASS.

## 8. Security

No VM images, Microsoft credentials, OAuth tokens, machine-identity raw values, Guest Agent secrets, account-bearing snapshots, private worlds or signing material in Git history. Privileged Guest Agent operations stay fixed and authenticated; ambiguous provider mutations fail closed. The installer must not silently delete runtime data.

## 9. Failure policy

On an unresolved change or failing proof: preserve last good source, identify first wrong owner, report exact failing claim and environment. Do not bypass checks, weaken security gates, fabricate successful runs or modify tests solely to make an incorrect implementation appear valid.

## 10. STOP

After the requested bounded repository outcome: confirm the exact `Local` commit, source change scope, applicable static checks, genuinely executed checks (if any), and remaining higher-context evidence. Stop rather than expanding into speculative feature work.
