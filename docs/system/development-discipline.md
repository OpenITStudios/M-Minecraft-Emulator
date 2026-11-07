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

For native controller, MCP and desktop presentation changes, the existing
`review` response includes explicit `crossModuleReviews` source-path
seams. Each requires examining the other established owners even if they are
absent from the selected diff: authenticated MCP/native input admission,
native/Tauri/typed-UI session presentation, and the bounded MCP activity
mirror. A fourth narrowly scoped `native-input-v4` review follows the
same MCP/CLI input author, the original Rust shared-memory writer and
the optional C++ Raw Input reader. It requires manual source comparison
of the complete 96-byte v4 ABI, eight 8-bit counters, input TTL/neutral
timing, and P39–P43 pending-release guards whenever any one participating
file changes. No C++ compiler, native runtime or automatic dependency
analyser is invoked. The entries are **review requests only**, not completed dependency
analysis or an assertion that unselected files are unchanged. A single changed
critical source may trigger more than one seam; no second controller, automatic
agent dispatch or new quality runner is introduced.

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

## Matt Pocock engineering method (adapted)

**Reference and authority.** Reviewed [mattpocock/skills at `4ae2317ddb22b046e26455e4f9c9b3aaaf62212d`](https://github.com/mattpocock/skills/tree/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d) (MIT). Its engineering *methods* are adapted, not installed or copied as a separate plugin. Root `AGENTS.md`, `GITHUB_RULES.md`, the sole `emulator-development` Skill, this document, current source ownership and the operator's test/CI prohibition prevail. Never use upstream setup, issue-tracker, subagent, shell, worktree, TDD or release instructions as implicit permission. Choose the one method whose trigger actually matches the problem; do not chain every method on every task.

### A. Domain vocabulary and deep module interfaces (DESIGN)

When two domain terms produce different requirements, resolve them using
[Canonical Naming](canonical-naming.md), [Implementation Map](implementation-map.md)
and the real caller. A `profile` is not a Windows process, a `session` is
not a playable Minecraft client, an `IPC ACK` is not a game action receipt,
and source evidence is not Windows/Minecraft acceptance. Edit the existing
terminology owner only if the definition truly changes. Do not create a new
global glossary or parallel architecture ledger.

Trace **caller → interface → implementation → observable result**. An
interface includes the error model, ownership, ordering, cancellation,
preservation guarantees and timing, not merely a function signature.
Prefer a small stable interface with complex behavior hidden behind
the current Rust/Tauri/MCP or UI owner. Use the *deletion test* before
adding a wrapper: if deleting it simply removes forwarding, it is probably
unnecessary; if real behavior must be duplicated across callers, the
module may earn its place. A new adapter requires an actual distinct
consumer/implementation or a real trust/protocol seam, never hypothetical
future portability. Source dependency graph results are bounded static
reachability, not semantic call or runtime proof.

Upstream: [domain-modeling](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/domain-modeling/SKILL.md), [codebase-design](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/codebase-design/SKILL.md).

### B. Clarify only material decisions and implement vertical slices (SCOPE)

Normalize a material request to: **outcome; actual symptom/acceptance;
first owner and consumers; must preserve; forbidden expansion; available
proof; STOP**. Use the source and operator decision when intent is already
clear, rather than an unneeded questionnaire. A larger change is one
coherent vertical slice from the authoritative owner through its actual
callers, test seam (when authorized) and documentation. Do not create
mandatory new tickets, status databases, task managers or architectural
phases for a bounded source change. The single current continuation
remains in `planning/development.md`.

Upstream: [to-spec](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/to-spec/SKILL.md), [implement](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/implement/SKILL.md).

### C. Bug diagnosis: distinguish the failure before patching (DIAGNOSE)

1. **Exact symptom first:** write Expected vs Actual, affected client
   slot/mode, source revision and observed evidence origin. Determine
   whether the issue is source-proven, a captured external observation,
   or an unverified runtime report.
2. **Find one discriminating feedback signal** at the *correct interface*:
   an observed source contradiction, an already available immutable
   diagnostic record, or, only when separately authorized, a small
   deterministic red-capable replay/fixture. It must detect the reported
   symptom, not merely `no crash`. Optimize signal specificity before
   searching unrelated owners. If an executable red loop cannot be run,
   explicitly label it **PROPOSED_NOT_EXECUTED** and do not claim a
   reproduced failure or regression PASS.
3. **Minimize** by removing irrelevant files/clients/actions *on paper
   or in approved reproducible evidence only*. For 4–5-client failures,
   explicitly preserve slot identity, release/quarantine, bounded
   scheduler and MC01 human-mode protections. Never touch live
   profiles, world data, input hooks or Windows processes as part of
   source-only diagnosis.
4. **Competing causes:** rank up to three falsifiable hypotheses with
   one source/observation that would disprove each. Do not change
   unrelated modules while trying guesses. When source proof confirms
   a mismatch, correct the one canonical owner and affected callers.
   If the failure fundamentally requires Windows/Minecraft observation,
   report that root cause as **UNVERIFIED** instead of guessing.
5. **Regression:** when an approved executor and agreed seam exist,
   use red → minimal green and rerun the original repro. During the
   current source-execution hold, do only bounded source/consumer review;
   any authored fixture is **AUTHORED_NOT_EXECUTED**. Never weaken
   a test merely to conceal an unsolved bug.

Upstream: [diagnosing-bugs](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/diagnosing-bugs/SKILL.md), [tdd](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/tdd/SKILL.md).

### D. Two-axis review before the source commit (REVIEW)

Review the **actual pinned change**, not general impressions. Both axes
may be inspected by the same agent: separate subagents are not required.

- **Standards:** authority/branch/permissions, source owner and callers,
  contract compatibility, privacy and native account/world preservation,
  interface locality, security, no retired runtime/CI and honest proof
  tiers. Treat code smells (duplicated logic, pass-through owners,
  speculative abstraction, scattered fixes) as source-grounded
  heuristics, not automatic violations.
- **Spec:** actual user outcome, expected behavior, missing/incorrect
  callers, relevant negative and recovery paths, changed behavior outside
  the request, and what evidence would falsify success.

Each substantive finding should cite an exact path/contract and state
`OBSERVED_SOURCE`, `INFERRED_RISK`, `AUTHORED_NOT_EXECUTED` or
`NOT_VERIFIED`. A static source review does **not** certify test success,
Windows behavior or five Minecraft clients. Fix only in-scope blockers;
do not spawn a separate review bureaucracy.

Upstream: [code-review](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/code-review/SKILL.md).

### E. Source research and agent-documentation economy (RESEARCH/HANDOFF)

Prefer pinned official documentation and the actual current repository
source over an unsourced summary. Preserve the exact reference and
separate an external claim from a verified local contract. In always-loaded
`AGENTS.md`, write only a *conditional pointer*; put specialist details
in the existing Skill and this discipline. Read a branch-specific reference
only when its trigger fires, avoiding wholesale external skill loading and
duplicated instructions. A next-session handoff uses the canonical
`planning/development.md` and commit trailers, never new memory/state files.

Upstream: [research](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/engineering/research/SKILL.md), [writing-for-agents](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/productivity/writing-for-agents/SKILL.md), [handoff](https://github.com/mattpocock/skills/blob/4ae2317ddb22b046e26455e4f9c9b3aaaf62212d/skills/productivity/handoff/SKILL.md).

### F. Evaluation and explicit proof ceiling

The [manual routing/evidence scenarios](../../.agents/evals/matt-pocock-method-scenarios.md)
exercise bug, design, review, research, prompt-injection and native
acceptance decisions. The optional repository test checks link/routing
**source consistency only**, not real agent behavior or execution quality.
An asserted improvement requires an independently observed
baseline-versus-candidate task on the same inputs, with evidence of
fewer wrong-owner probes, fewer ungrounded guesses and no degraded
safety/acceptance. Until that happens, the status is **SOURCE_POLICY_ADAPTED;
AGENT_EFFECTIVENESS_NOT_MEASURED**. Return to the existing completion
preflight and STOP; do not create a third workflow lane.

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
