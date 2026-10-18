# Current Development Handoff — M-Minecraft-Emulator

## Full Source #9–10: three root causes patched together (18 October 2026)

**Observed:** manual Windows GitHub [#9](https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/39142688943) and [#10](https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/39144237781) at `51029b4b57a268424cf45ce43cbf5661521a3c10` both reported 3 failing stages of 12: repository, isolation (164/168 PASS, 4 FAIL), and desktop (148/151 PASS, 3 FAIL). All canonical Rust check/test/format stages passed.

**First-owner fixes:**
- Repository policy reader normalized Windows CRLF to LF **in memory** before comparing `verify.yml` stage contracts. Strict canonical 12-stage coverage and fail-final-gate checks remain.
- Reviewed `mivubi_core_identity.h` is normalized to LF **in memory** when loaded for private C/C++ patch construction. Tests now assert generated private C/C++ source has no noncanonical CR; kernel ABI, source hash and Git `--index --whitespace=error` remain untouched.
- The disposable *LF* Git test fixture sets `core.autocrlf=false` in its isolated seed/app repositories so Windows/global Git policy cannot convert the nested inline test-source script unexpectedly. Separate explicit Windows CRLF fixture tests are retained.
- Desktop test fixes: define app source in the Create Virtual test's lexical scope (previously ReferenceError), verify the actual engine-package checksum notice rather than obsolete Folder snapshot text, and include `test:isolation-source` plus `test:presentmon` in the canonical verification routing expectation.

**Proof ceiling:** exact source and two Windows logs analyzed, SHA consistency and changed file readback available. Patched Vitest/Node and full CI are NOT_EXECUTED until a new manual dispatch. No UI runtime, installer, signed driver, sandbox account, deployed game or `main` changes.

**Next:** run the manual Full Source scope once on the new Local HEAD and examine all 12 outcomes.

## Full Source diagnostic coverage: expose all stage failures per manual run — 18 October 2026

**Reason:** Windows Full Source #8 stopped at the isolation suite (140/149 PASS, 9 FAIL), leaving Svelte/Vitest desktop, canonical Rust and formatting untouched. The previously sequential `npm run verify:source` runner therefore requires repeated manual GitHub runs to discover independent errors. A new `Local` commit `9b195475f2e430dec6c17dd25b031dd58cfd95b3` has already corrected all nine observed fixture/import categories from #8 but has no executed CI verdict.

**Implementation:** retain root `verify:source` as the canonical list of 12 verification commands. The manual Windows `full-source` GitHub job runs each exact canonical command in its own diagnostic step with `continue-on-error: true`, then an always-run aggregate gate fails the job if **any** command failed or was skipped. This does NOT relax any validator, suppress evidence or treat a partial run as PASS. The repository verifier now asserts full coverage of all canonical commands, twelve diagnostic steps and the final failure gate so workflow drift cannot silently omit checks. Rust toolchain setup also honors each command exit code.

**Boundaries:** strictly `workflow_dispatch` and `refs/heads/Local`; `main` unchanged. No installer/package, owned driver activation, Sandboxie mutation, credentials or gameplay. More tests will run in one manually requested Full Source job; this trades a longer single run for a complete failure inventory and fewer manual CI iterations.

**Proof:** workflow/static contracts reviewed at exact commit source; no automated runner was triggered by this change. The new Full Source diagnostic execution and patched nine isolation cases remain NOT_EXECUTED until a user explicitly dispatches `full-source`.

**Next:** manually run Full Source once against this new Local SHA, then fix the complete reported failing-stage inventory in one batch.

## Full Source #8 — remaining 9 isolation failures, bundled corrections (18 October 2026)

**Observed:** [Windows Full Source #8](https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/39139293068) at `16a1e15e8e84e386d2ea39f3dacb7205c09735db` reached isolation suite: 140/149 PASS, nine FAIL. Setup and repository, recorder, feasibility tests passed, but Svelte/Vitest/Tauri/Rust compilation had NOT_EXECUTED due to the isolation suite gate.

**First-owning faults fixed together:** (1) Two private Git patch fixture tests assumed LF files, though Windows worktrees legitimately have CRLF. Compare LF-normalized output against the exact reviewed private patch and retain the explicit CRLF assertion; Git indexed/hunk/identity checks remain. (2) Disposable cloned application fixtures omitted the newly imported `git-directory-identity.mjs`; copy its real production bytes together with dependent source. (3) Exact Git subtree identity was already checked, while one test compared a Windows worktree file directly to LF; normalize EOL for only that assertion. (4) Baseline build preflight checks were correct but the source-order assertion searched a literal LF sequence on Windows CRLF; normalize only the test's read string. (5) Tampering with ledger used an unrecognized `imageSignatureStatus` field and was rejected as malformed before reaching hash verification; mutate valid `signatureStatus` to prove the chain is actually checked. (6) `windows/transport-commands.mjs` incorrectly imported `../attestation-gate.mjs`; the existing trusted empty production key owner is `./attestation-gate.mjs` alongside it. Keep rollback gated, with no new trusted key.

**Safety:** no weakening of upstream pins, package manifest closure, Git `--index --whitespace=error`, cryptographic source checks, service ownership rules, provider selection, driver load or installed account data. This batch changes only test fixtures and one module-relative import, not the engine activation policy. No workflow triggers or `main` edits.

**Proof ceiling:** actual #8 errors mapped to corresponding source, static cross-file invariants and GitHub exact-SHA readback. Patched tests are NOT_EXECUTED until the operator runs manual Full Source. Subsequent Svelte/Rust failures may remain; do not assert Localtest Ready.

**Next:** manually run Full Source once on new Local SHA, then fix any newly exposed Svelte/Rust errors in a complete batch.

## Full Source #7 — update private Git patch command contract (18 October 2026)

**Observed:** [manual Full Source #7](https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/39138030002), exact `Local` `e9df2de39146fe2ee4c59157949526f03b54a1c5`, failed after successful Windows checkout, Node 24/npm 11, Rust 1.98.1 and locked npm setup. `npm run verify:repository -- --g0` failed at `tooling/repository/verify.mjs:258`, with `Private fork source/build must be explicit, reviewed and non-installing`. No isolation-source, desktop Svelte/Vitest, Tauri or canonical Rust build/test suite executed for #7.

**Confirmed owner:** the static verifier still demanded the historical prefix `git(root, ["apply", "--check", "--index"` despite the previous Windows CRLF patch adding `git -c core.whitespace=cr-at-eol apply ...`. The production patch routine still uses two strict commands: one indexed `--check` then indexed `apply`, both with `--whitespace=error`. This is a stale verifier predicate, not evidence to weaken patch validation.

**Bounded change:** require exact source statements for both CRLF-aware strict commands (including `-c core.whitespace=cr-at-eol`, `--index`, `--whitespace=error`, and the preparatory `--check`). Keep all other private provenance, noninstallable driver and provider checks untouched. No workflow, `main`, desktop app, accounts or kernel runtime changes.

**Proof ceiling:** the exact predicate/producer mismatch and replacement checked against pinned source. Full Source on the corrected commit is NOT_EXECUTED. The 28 isolation failures observed on #6 remain unverified after the Windows fixture changes in `e9df2de`; do not claim they were resolved.

**Next:** manually dispatch Full Source for the new `Local` SHA and inspect the first real failing check or test category.

## Full Source #6 — isolation fixture and Windows path repairs (18 October 2026)

GitHub Actions Full Source #6 at `4039c5b` completed environment setup but failed in `test:isolation-source`: 121 of 149 isolation tests passed; 28 failed. Svelte/Vitest/Rust builds were not reached. This patch addresses identified Windows Git-directory short-name aliases without accepting different physical directories, Git patch CRLF trailing-carriage-return false positives without disabling whitespace/index enforcement, obsolete subset baseline hash fixtures using the immutable canonical manifest, and 10-versus-11 owned source file provenance drift. Four Git source consumers use the same constrained directory identity helper; installation, runtime data, signed driver, Sandboxie, manual CI and default branch remain unchanged. Other failures are unresolved; no new Windows run executed yet.

Next: one manually requested Full Source run at the new Local SHA and diagnosis of remaining failed cases.

## Quick #4: repair obsolete blanket MIVUBI name prohibition — 18 October 2026

**Actual CI evidence:** [manual Windows Quick run #4](https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/39134013426) on `84d481434ffaf9ddc1462f8fbc52da8f2f14c22b` failed properly at the first fail-fast `Verify repository contracts` step (exit 1). The remaining five Quick source suites were SKIPPED, not FAIL or PASS in this run. The exact verifier error at `tooling/repository/verify.mjs:420` was `Active generic naming must remain consistent without renaming installed VMware identity`.

**Confirmed first wrong owner:** the verifier required `!isolatedScreen.includes("MIVUBI")` even though the active UI must explain the status of the MIVUBI-owned engine, explicitly stating it is not active and that no owned virtuals have been created. All ten other individual checks in the naming block matched the exact `Local` source, including the unchanged historical installed product name and identifier.

**Bounded fix:** separate installed Windows identity checks from active runtime/user-facing naming checks; preserve the fixed package identity and existing app/collector/data-path contracts. Replace the blanket MIVUBI ban with two affirmative, scoped UI requirements: existing Sandboxie is the active pilot, while MIVUBI-owned engine/virtuals remain unavailable and unverified. This is a verifier correction, not product renaming. No changes to driver, installer identity, runtime, account data, workflow triggers or providers.

**Proof:** original failing run logs and exact source predicates inspected; source-based positive and negative cases verified for legacy identity and engine non-activation wording. Full repository verification on the new SHA still awaits a manually requested Quick CI run; Full Source/Windows Build/Minecraft acceptance are NOT_EXECUTED.

**Next:** manually start Quick on the new `Local` commit and examine the first remaining failing check, if any.

## Quick #3 false green diagnosed: restore fail-fast CI and current UI contract — 18 October 2026

**Actual evidence:** [manual Windows Quick run #3](https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/39128459740) at `9bb52ac5d344e89f2269db0e4af2175e625bbdd5` showed overall SUCCESS and 62 source tests PASS (6 localtest + 15 feasibility + 18 native feasibility + 15 Sandboxie feasibility + 8 PresentMon). However, its first command `npm run verify:repository` threw `Active native/isolated pilot may not call legacy VM lifecycle commands` at `tooling/repository/verify.mjs:194`. Windows PowerShell executed every subsequent command, whose final exit status masked the failure. The run is **PARTIAL / FAILED CONTRACT VERIFICATION**, not a complete Quick PASS.

**Root cause and bounded correction:** the first wrong owner is a stale static verifier requirement `<h1>Instances</h1>`; mounted `CompactManagerApp.svelte` now uses `<h1>Virtual Manager</h1>` and separately `<h2>Minecraft sessions</h2>`. Update the exact verifier predicates, preserving all legacy VM prohibitions and Tauri command guards. Convert each Quick `npm run` to its own GitHub Actions step, and separate Windows Build icon/frontend commands, so no prior failure can be masked. Preserve manual-only `workflow_dispatch`, `Local` ref guards, Windows runners and no installer or driver action.

**Proof boundary:** actual run #3 job logs retrieved; current source checked for every predicate in the previously failing active UI block (13 unaffected checks and corrected headings). New workflow source reviewed for single-command fail-fast steps. New Quick execution on committed SHA remains **NOT_EXECUTED** until manually dispatched; Full Source, Tauri compile, Minecraft gameplay and driver acceptance remain unexecuted.

**Next:** run manual Quick on `Local` at the new commit, verify all six steps PASS, then run Full Source.

## Windows Quick manual CI first-run fix — 18 October 2026

**Observed:** [GitHub Actions #2](https://github.com/MIVUBI-STD/M-Minecraft-Emulator/actions/runs/39123683409) on `c8bcbc4126aa5f38461aa156ca219bdd3d3b45ba` passed checkout and installed Node 24.21.0/npm 11.19.0, then failed in `actions/setup-node@v7`: its implicit npm cache search expected root `package-lock.json` although the only lockfile is `apps/desktop/package-lock.json`. All Quick test commands were skipped.

**Change:** move Quick from Ubuntu to `windows-latest`, disable implicit npm caching (`package-manager-cache: false`) only for Quick's no-install tier, and use Windows PowerShell for job summaries. Full Source and Windows Build keep their explicit lockfile-specific cache. Existing Local-only guards, manual workflow trigger, main registration, and no driver/install restrictions remain.

**Evidence:** first failed run's job logs and exact source configuration reviewed. Corrected Quick workflow remains NOT_EXECUTED until the operator manually runs it. Windows runner source tests cannot prove Minecraft gameplay or the target PC environment.

**Next:** manually run only Quick against the new Local SHA and review the resulting tests.

## Local-only manual Actions registered on main — 18 October 2026

**Decision:** the user approved a single discovery-only workflow file on `main` so manual Actions can target the source on `Local`. CI remains fully opt-in. No `push`, PR, schedule, release, merge, package installer or driver install triggers are introduced.

**Implementation:** the `Local` verification workflow now checks `github.ref == 'refs/heads/Local'` for Quick, Full Source and Windows Build jobs. The default-branch workflow holds only the same dispatch inputs and an always-skipped placeholder job. An accidental `main` selection consumes no build/test job and cannot count as PASS.

**Proof ceiling:** GitHub exact-head source/branch registration will be checked. Workflow execution, Svelte/Rust compilation, native Windows and Minecraft acceptance remain NOT_EXECUTED until actually run.

**Next:** run manual Quick verification for the selected `Local` SHA, then examine its logs before requesting expensive tiers.

## Manual-only CI tiers prepared without default-branch changes — 18 October 2026

**User decision:** keep CI strictly manually triggered. Automated `push`, pull-request, schedule and auto-release triggers are disallowed to avoid unwanted expensive runs. Preserve `Local` as the development authority and do not change `main`.

**Implementation:** the existing `.github/workflows/verify.yml` now offers one `workflow_dispatch` `scope` choice with default `quick` (Linux source contracts), optional `full-source` (complete Node/Svelte/Vitest/Rust verification on Windows), and `windows-build` (frontend plus Tauri MSVC compile; never package or install). The three jobs use mutually exclusive conditions and bounded timeouts; no automatic triggering, artifacts, login flows or driver operations. The existing VMware NSIS package smoke remains historical and separate.

**Important operational blocker:** GitHub requires the workflow definition on the default branch for `workflow_dispatch`. As `main` must remain README-only by current repository policy, none of these manual tiers can yet be dispatched from the Actions UI. This is a deliberate unresolved activation dependency, not a PASS or proof that CI is currently runnable. Do not bootstrap `main` without explicit separate authorization.

**Proof:** source/YAML structural contract and exact `Local` readback can verify trigger and scope separation. No GitHub Actions run, full compilation, Windows, Minecraft, driver or four-account acceptance is claimed.

**Next:** obtain explicit branch-policy permission for a minimal default-branch workflow registration if actual GitHub manual dispatch is desired; otherwise preserve these definitions as source-only.

## Compact Manager regression assertions aligned to active UI and operator guide — 18 October 2026

**Verified test blocker:** literal-source contract review at pinned `Local` discovered five deterministic failures in the existing `apps/desktop/test/compact-manager.test.ts` suite: two checks assumed an obsolete `Check status` toolbar label, one checked old owned-driver explanatory copy, and two asserted phrases that are not present in the current canonical two-client localtest guide. The mounted UI correctly labels the toolbar `Check status · 02`, distinguishes Client 01 from Client 02 inspection, and states that the owned isolation engine is not available. These were stale test expectations, not reasons to change user-facing behavior.

**Bounded fix:** update only those five exact assertions in the existing test owner to match the actual rendered UI and canonical operator-guide wording; preserve checks for explicit Check status, no Host 01 authentication inference, and disabled Create Virtual. No runtime, browser UI, IPC, Sandboxie, driver, account or product policy was changed. No new test runner or duplicate source of truth was introduced.

**Proof:** source-specific scripted audit inspected 128 literal `app` expectations and other bound file expectations, first identifying the five mismatches, then re-evaluating those expectations against the exact GitHub source texts with no remaining known mismatch. This is targeted static contract verification, **not** Node 24 Vitest/Svelte compiler execution; full pinned source verification, Rust, Windows/WebView2 and Minecraft runtime remain **NOT_EXECUTED**.

**Next:** run complete pinned same-SHA source preflight when execution is actually available.

## Mounted Host 01 Svelte click binding repaired — 18 October 2026

**Source-confirmed blocker:** the active `apps/desktop/src/CompactManagerApp.svelte` primary Host 01 button had a missing `}` after its `choosePrimary("01", event.currentTarget)` click expression. The next `aria-expanded` attribute was consequently merged into an invalid Svelte event expression. This is a source syntax error, not a Minecraft runtime issue.

**Bounded correction:** close the `on:click` expression; keep the Host 01 `aria-expanded` and `aria-controls` values as independent attributes. Add regression coverage using the existing Vitest test owner and `svelte/compiler` `parse` on the exact mounted Svelte file, plus Host 01/02 binding assertions. No backend command, Sandboxie config, driver, account data or launch policy changes.

**Proof ceiling:** original malformed binding confirmed from pinned GitHub `Local`; corrected expression extracted and evaluated with an actual JavaScript callback and exact-file static checks. The Svelte compiler regression was authored but not run: pinned Node 24/npm 11 and the project's installed Svelte dependencies are unavailable here. Complete Vitest/Svelte/Rust, Windows/WebView2 and gameplay acceptance remain **NOT_EXECUTED**.

**Next:** run full same-SHA source verification in the supported toolchain, then proceed to a controlled Windows pilot.

## Client 02 failed Open request retains launch safeguard — 17 October 2026

**Reproduced source fault:** an accepted Client 02 Open request retained a pending launch notice, but a rejected/timed-out `requestSandboxClient02()` returned to `reportIssue()` only. The next `checkStatus()` cleared that issue, and a quick empty candidate snapshot could project `Ready to open` even though an earlier Start.exe request might already have spawned Minecraft. The Rust retry gap is not an atomic game-process status proof.

**Bounded correction:** the active `openMinecraft02()` catch now records both the actionable launch error and the existing `Open requested` safeguard. The canonical recheck behavior retains the notice through empty/UNKNOWN or failed inspection, releases it for an observed boxed candidate, or permits operator review after a fresh safe-empty check. No retry, extra manager, new state store, driver, box mutation or account access was added. Existing source test owner and active operator instructions were updated.

**Proof ceiling:** the precise catch/recheck fault is source-confirmed; affected exact-source control-flow/static regression checks performed and a canonical Vitest regression authored. No successful Node24/npm11, Rust/Cargo, Svelte or Windows/WebView2 build or Minecraft playback is claimed. The available local CLI is Node22/npm10 with no cargo or supported Git checkout network.

**Next:** perform a complete exact-SHA source preflight once the pinned toolchain is available.

## Client 02 unresolved launch survives early and failed checks — 17 October 2026

**Source defect:** active `checkStatus()` previously cleared Client 02's `Open requested` notice before inspection. A quick snapshot with no name-matched candidate then projected `Ready to open` even though an earlier request might still be starting. A failed inspection also dropped the launch notice. Rust's 30-second retry gap alone cannot prove a child cannot appear later.

**Bounded fix:** preserve pending Client 02 Open feedback through empty/UNKNOWN status and errors, clear it when an observed boxed candidate exists, expose a reviewable Details control and allow explicit operator acknowledgement only after a fresh safely observed empty box. Unknown status remains an actionable issue, not a successful running claim. The acknowledgement is UI-only; the Rust preflight, Sandboxie box, signed executable policy, account storage, driver and unavailable Create Virtual stay unchanged.

**Proof ceiling:** exact pinned source inspected and source-derived V8 projection/guard cases evaluated; source regression added to existing Vitest owner and localtest guide updated. Full Node24/Vitest/Svelte/Rust/Windows/WebView2 and real Minecraft startup/crash proof remain NOT_EXECUTED. An empty snapshot is never an atomic guarantee.

**Next:** run the pinned full source preflight when the appropriate toolchain becomes available.

## Client 02 move notice no longer hides safe Switch — 17 October 2026

**Verified source defect:** after `moveMinecraft02()` accepted a window placement request, the UI stored `Move requested` in instance 02 notices. The canonical `rowPresentation()` treated *every* notice as an unavailable primary action, even when the previously observed unique client window still had `action: "switch"`. This forced another potentially expensive status inspection merely to switch back, despite the existing backend revalidating PID/HWND, sandbox membership and signed image for every focus/move. This is a source-reproducible UI dead end, not evidence of successful window placement on Windows.

**Bounded correction:** `apps/desktop/src/app/presentation/compactManager.ts` keeps `Move requested` visible without hiding Switch **only** for Client 02 with a base action of `switch`. Busy, issue, no eligible candidate, Client 01 and all `Open requested` notices remain restricted; no automatic launch, poll, provider change, new helper or box mutation. The existing `apps/desktop/test/compact-manager.test.ts` checks positive and negative cases, and the active localtest guide describes the new distinction.

**Proof ceiling:** exact pinned `Local` source and affected callers inspected; pure canonical presentation function exercised against source-derived V8 scenarios before and after patch, with expected blocked pre-fix case and passing post-fix cases. Added Vitest regression is source-authored, not an executed Node24 Vitest suite. Full Svelte, Rust, Windows/WebView2, actual focus/window placement and gameplay remain **NOT_EXECUTED**.

**Next:** perform the complete exact-SHA source preflight on a supported toolchain when genuinely available. Preserve official `MinecraftClient02`, account data and the disabled owned-engine Create Virtual route.

## Host 01 manual feedback recovery and focus — 16 October 2026

**Reproduced UX blocker:** the mounted `apps/desktop/src/CompactManagerApp.svelte` sets a Host 01 `Open requested` notice on an accepted native Windows launch request and sets a sticky issue on an error. The only user-facing native row action comes from `rowPresentation`, which intentionally suppresses Open while any notice/issue exists. There is NO Host 01 inspection route wired into the mounted UI: the global `Check status` inspects **Virtual 02 only**, and previously the Host 01 Details button appeared only for an issue, not for `Open requested`. Hence a successful initial launch request left the native row with no way to review/dismiss its feedback and request another launch in the same app session; a failed launch also offered no explicit feedback dismissal.

**Bounded fix:** preserved the source-safe `Open requested` and issue statuses, but exposed a read-only Host 01 `Details` affordance for either outcome. Details now clearly warns that MIVUBI cannot verify whether the native Minecraft window is still running and advises inspecting Windows first. A manual `I checked Windows · Clear message` button clears **only Host 01 local UI issue/notice state**, never calls native/Tauri, never retries launching, never terminates Minecraft, never claims game closed, and cannot operate while a manager request is in flight, on Settings, or when another client's panel is expanded. It deliberately restores Host 01 Open only after that user acknowledgement. Keyboard focus is restored to the new Host 01 primary button when the note is cleared, or to the new Details button after completion of a native Open request if the user did not navigate elsewhere; missing/disabled focus targets are never focused. Kept existing Virtual 02 / four-slot readiness and the disabled Create Virtual owner untouched.

**Actually checked:** confirmed actual Node22 and TypeScript CLI exist in this runtime, but **Node24, Svelte/Vitest packages, Rust/Cargo and PowerShell are absent**, so full pinned frontend and Rust builds remain `NOT_EXECUTED`. Exact GitHub `Local` source readback; executed the new targeted source test body in V8 (PASS), evaluated live `dismissHost01Feedback` logic in V8 with eight in-memory outcome/focus/guard scenarios (8/8 PASS), and checked balanced selected markup and if/each controls with no new timers, provider routes or auto-launch. These tests are targeted source execution **not canonical Vitest, Svelte compilation, browser accessibility, WebView2 or Windows gameplay proof**.

**Next evidence:** when pinned dependency/toolchain execution is actually available, run canonical full frontend/Svelte and Rust source checks; later observe native Host 01 and official Client 02 in the single final Windows campaign. The official sandbox `MinecraftClient02` and account-bearing storage have not been modified.

## Host 01 status must not inherit Virtual 02 inspection — 16 October 2026

**Confirmed mounted-UI defect:** `apps/desktop/src/CompactManagerApp.svelte` passed `observation !== undefined` (ONLY a Virtual 02 Sandboxie inspection) as the `inspected` argument to `rowPresentation("01", ...)`. The `apps/desktop/src/app/presentation/compactManager.ts` projection used that shared flag to change Host 01 from `Not checked` to `Status unconfirmed`, although the active check only called `desktop_inspect_sandbox_client_02` and never inspected Host 01. This created a misleading cross-instance status transition after users checked Virtual 02.

**Bounded correction:** removed the semantically invalid `inspected` parameter from the single canonical `rowPresentation` helper, updated the mounted Svelte consumers and existing regression fixtures, and always keep Host 01's BASE status `Not checked`. Host 01 own feedback remains independent: `Open requested` after a successful native Windows open *request*, or a visible issue when its request fails. Virtual 02 still displays `Checking…` and its last observed/read-only statuses, without ever converting uncertain membership to launch permission. No new IPC calls, background polling, status cache, driver, Sandboxie setting, account mutations or Create Virtual activation.

**Proof on exact `Local` source:** executed eleven affected `compact-manager.test.ts` bodies in V8 with current TypeScript projection bodies extracted from GitHub, bounded dependency stubs and a limited assertion adapter; **11/11 PASS** including a new regression that explicitly checks no Host 01 status drift after Virtual 02 inspection. Another eleven direct checks of the current `rowPresentation` function and caller contracts passed. These are targeted SOURCE-derived checks, NOT a canonical Node24 Vitest, `svelte-check`, WebView2, Rust/Cargo, official Sandboxie, Windows gameplay or independent Host 01 runtime test. No User PC testing is requested before the user's consolidated final acceptance campaign.

**Next:** continue only with a source-reproducible cross-instance UI/backend fault; otherwise defer target-Windows behavior to the final campaign. Protect `MinecraftClient02` and the disabled MIVUBI-owned Create Virtual route.

## Pilot EXE picker and launch evidence consistency — 16 October 2026

**Confirmed UI/backend mismatch:** the official Client 02 file picker in `apps/desktop/src-tauri/src/engine/native_clients.rs::select_education_exe` previously accepted any user-selected `*.exe` and returned it unvalidated. The mounted Svelte app then said "Minecraft selected" and, given preflight+licence, could show Open; only `launch_with_official_sandboxie` invoked `validate_education_path` and rejected unrelated filenames. This created a misleading setup and delayed failure, not evidence that arbitrary binaries were actually launched: the real launch retained Windows Authenticode/ProductName validation and signed-publisher checks.

**Exact owner correction:** the Windows file picker now reuses the exact canonical `validate_education_path` used at launch, rejects paths not representable as UTF-8 instead of silently lossy-converting them, verifies a still-existing selected file, and returns `None` unchanged on cancellation using `Option::map(...).transpose()`. This is early path/filename sanity only, NOT a Microsoft signature/authenticity proof; the launch path still rechecks filename, signature and ProductName. In `CompactManagerApp.svelte`, a rejected/failed selection clears the previously selected executable and Sandboxie licence checkbox before showing a picker-specific error, whereas a canceled dialog leaves both intact. The UI now labels any accepted path "File selected · Publisher checked when opening" instead of claiming Minecraft identity already verified. None of these changes enables owned Create Virtual or modifies the official box, engine, token or Minecraft content.

**Proof this change:** actual GitHub `Local` source readback plus the existing desktop UI test owner extended with a cross-file regression ensuring picker and launcher use the same validator, signature remains launch-time, stale licence/path clear only on picker error, and the new honest label. The regression's actual test body PASSED when executed with source-backed fixtures inside the available V8 isolate. This is a selected source-level regression, NOT a Windows file-dialog invocation, `cargo test`, Node24/Vitest, Svelte build, real Authenticode call, Windows acceptance or game login. The target runtime remains `NOT_EXECUTED`.

**Next evidence:** test real Windows picker selection, cancellation, wrong EXE rejection, Microsoft signature validation and UI focus in the final local pilot campaign; separately confirm official Client 02 idle stability. Preserve private driver/service isolation and the disabled Create Virtual 03/04 controls.

## Virtual 02 setup and failure recovery UX safety — 16 October 2026

**Concrete source mismatch fixed:** Previously, `Settings → Set up Minecraft 02` used `isSandboxieBoxPrepared(observation)` alone to decide whether to show Browse and Sandboxie licence consent. That predicate only proves the observed persistent box settings, **not** observed membership or that zero sandbox process candidates were found. As a result the setup panel could display launch-preparation controls when membership was `UNKNOWN` or an isolated candidate process was already present—even though the row-level `client02Presentation` correctly refused Open. This was confirmed by comparing the mounted markup against canonical `canRequestClient02` and its independent membership/occupancy checks.

**Owner / bounded implementation:** `apps/desktop/src/CompactManagerApp.svelte` now displays the executable picker and licence confirmation only when `canRequestClient02(observation)` is true. Unverified, unprepared or occupied states display the canonical setup instructions and a manual **Check status · 02** action instead. Detailed Sandboxie box setup instructions appear only when settings are actually unprepared, not when an observed game/process may already be running; the official `MinecraftClient02` is never changed by the UI. `apps/desktop/src/app/presentation/compactManager.ts::setupInstruction` now labels occupied-box observations `Minecraft may already be open`, warns against opening another copy, and keeps UNKNOWN evidence as uncertain. The error panel also offers an explicit read-only recheck. Setup's recheck button is kept mounted across a successful transition to the picker to avoid losing keyboard focus; resolution of an issue calls the existing `closeDetails()` focus-restoration route rather than removing the panel abruptly. These are UI-only additions/restrictions, not a new launch, engine executor, implicit retry or service configuration.

**Proof obtained:** Exact `Local` GitHub source readback and newly added regression in the existing `apps/desktop/test/compact-manager.test.ts`. Five selected current UI regression bodies (including the new unsafe-setup negative cases and navigation expectations) were actually executed in V8 against extracted live TypeScript presentation function bodies and source-backed UI fixtures; **5/5 passed**, NOT a Node24/Vitest run. Eleven independent cross-file source invariants passed, including setup/issue recheck, no issue auto-launch, no owned Create Virtual command and balanced selected HTML/Svelte blocks. Updated `docs/validation/native-pilot-localtest.md` to show recheck and blocked unsafe picker. No actual Svelte compilation, WebView2 keyboard focus testing, PowerShell, driver activation, Microsoft sign-in or four-client acceptance was performed.

**Unresolved:** Windows UX and native engine validation remain `NOT_EXECUTED`; source cannot guarantee service readiness, account isolation or actual game stability. Create Virtual 03/04 stays blocked, official Sandboxie persists unchanged. Next: review a genuine remaining UI/backend defect if source-reproducible; otherwise defer runtime-only questions to one final local Windows acceptance session.

## Virtual 02 first-use UX: explicit row-level Check status — 16 October 2026

**Issue reproduced in active UI source:** after previous Minecraft-first / Settings Diagnostics refactor, first-time Virtual 02 appeared as `Not checked` but `client02Presentation(undefined,...)` returned `action:null`, meaning **there was no primary action on the virtual's own row**. The operator had to discover the separate top-toolbar `Check status · 02` command before setup/open/switch became available. This is a first-run discoverability issue, not a claim of failed Windows execution.

**Bounded UX correction:** the canonical presentation owner `apps/desktop/src/app/presentation/compactManager.ts` now supplies `action:"check"` ONLY while the Client 02 observation is absent. The mounted `CompactManagerApp.svelte` maps the new primary-label `Check status` and routes it through its EXISTING `checkStatus()` handler/typed read-only Tauri command, with `aria-label="Check status for Minecraft 02"` for accessibility. The old global toolbar check remains as an alternate explicit refresh. `rowPresentation` continues to suppress primary actions while busy, after a launch request notice or on an issue; it does not turn unknown/invalid membership into permission to open. Setup guidance points to the new in-row control. The existing pilot validation guide `docs/validation/native-pilot-localtest.md` now documents both row and toolbar access. No new IPC, native provider, creation flow, account handling, timer or Sandboxie engine modification.

**Proof:** exact current GitHub source re-read after edits, and five selected existing `compact-manager.test.ts` test bodies executed in the available V8 isolate with source-backed TypeScript presentation functions, dependency stubs and a limited assertion adapter; **5/5 PASS**. Six independent V8 source-projection checks (initial Check, busy suppression, error override, launch-notice suppression, native-slot separation and no instant Open) also **PASS**. This is NOT a full pinned Node24 Vitest run, Svelte/TypeScript compilation, Windows/WebView2 rendering, accessibility device audit, real running Sandboxie or four-account Minecraft proof. The existing official Client 02 runtime can still crash or fail a probe; no source test resolves that runtime issue.

**Next meaningful proof:** check the fully mounted frontend with the prescribed source toolchain when available, then perform the final real Windows/WebView2 pilot flow and separate signed-owned-provider acceptance. Preserve disabled Create Virtual and account-bearing `MinecraftClient02`.

## Main UX simplifies Minecraft sessions; Sandboxie inspection moves to Settings Diagnostics — 16 October 2026

**Decision implemented on mounted UI:** MIVUBI's primary view focuses on supported Minecraft session actions and visible slot readiness, not Sandboxie engineering controls. In `apps/desktop/src/CompactManagerApp.svelte`, the `Inspect Sandboxie names` and `Check engine readiness` buttons (and their respective inventory, host and error output) moved OFF the primary sessions screen into a collapsed `Settings → Diagnostics` disclosure below ordinary Minecraft setup, Performance, window preference and Help sections. Existing read-only `Inspect engine candidate` and its status/errors moved into the SAME diagnostic area. The canonical action functions, typed native bridge and registered Rust/Tauri endpoints remain unchanged. There is no extra manager, IPC command, background polling, provider selector, copy/delete operation or engine activation.

**Honest user-facing boundaries:** Main screen shows the 01/02 pilot and separately displays unavailable 03/04, retaining a disabled Create Virtual control with a visible, short reason. Detailed service/collision evidence is displayed only in Diagnostics using the existing `ownedVirtualCreationReason` projection. Main readiness copy no longer presumes any Minecraft is running: Host 01 *can request* a Windows launch, and Virtual 02 may be checked and opened through the still-installed official Sandboxie Plus box after setup. Planned 03/04 explicitly say no **MIVUBI-managed** session (not an unsupported assertion that no user account or external box exists). The pre-existing signed private-driver, independent-provider and four-client limitations are NOT silently resolved by the UI refactor. Official Sandboxie pilot settings still require explicit existing-box setup; engine-private full integration remains blocked.

**Accessibility and visual hierarchy:** The Diagnostics disclosure is keyboard-operable native `<details><summary>`, closed initially, with existing manual-only inspection controls disabled while manager operations are busy. Associated inventory and diagnostic errors stay inside the disclosure, including while an IPC operation is in flight. The main disabled Create Virtual control keeps `aria-describedby` pointing to an always-visible blocker; no user has to trigger a failed action to learn why creation is disabled. Added compact, lower-prominence diagnostics styling to `apps/desktop/src/app.css`, reusing the same mounted app design language; did NOT rename installed Windows product identifier or change the old official Client 02 box.

**Proof and tests (current source only):** Extended existing `apps/desktop/test/compact-manager.test.ts` with a source-level check that technical controls/results exist exactly once and ONLY under Settings Diagnostics; updated prior copy expectations to avoid false availability/account claims. Nine selected source test bodies were ACTUALLY EXECUTED with a small V8 assertion adapter over exact GitHub `Local` blobs; all nine PASS. Ten independent cross-file source checks PASS, and a limited markup scan found balanced selected HTML elements and Svelte if/each blocks. This is **not** execution of the canonical Node 24 Vitest suite, `svelte-check`, Svelte compilation, actual WebView2 visual rendering, keyboard/screen-reader interaction, Windows service calls, signed driver startup or four-account Minecraft acceptance. All those stronger proofs remain **NOT_EXECUTED**; no user-PC interim testing requested.

**Next real evidence:** when the source/build blockers have been resolved, compile the Windows desktop and inspect visual layout, disclosure focus/keyboard behavior, disabled Create control and actual 01/02 official-pilot workflows at final local acceptance; independently verify any future private driver/service compatibility before enabling 03/04. Do not present "Sandboxie internal integration complete" as a product fact merely because technical controls have moved into Diagnostics.

## Pre-local-PC desktop UI/UX audit — 16 October 2026

**Intent:** audit the real mounted Svelte desktop, CSS, typed bridge and active backend route before the user tests on their local Windows PC. Do not falsely declare full readiness, alter the Windows bundle identity, install services or activate owned Sandboxie.

**Verified active UI limits:** `apps/desktop/src/main.ts` mounts only `CompactManagerApp.svelte`; the only usable slots are Host 01 (Windows app launch request) and Virtual 02 (existing official Sandboxie Plus pilot; inspected/focused/moved via guarded backend). A planned private-owned profile 02/03/04 is NOT an installed driver or a runnable virtual. `desktop_create_virtual` remains absent from the registered Tauri commands and Create Virtual is disabled. There has been NO completed four-client UI/runtime acceptance.

**UI source fixes this batch:** changed mixed native/virtual heading from "Your virtuals" to "Minecraft sessions"; top copy now states the actual available two slots and blocked Virtual 03/04 in plain language. The mounted UI now explicitly lists planned Virtual 03 and 04 as unavailable **without buttons** or fabricated runtime/account identity. Create Virtual's permanently disabled control points to a visible, live readiness explanation through `aria-describedby`. Repeated "Details" links have distinct accessible names for Client 01 error and Client 02. The Check status button specifies that only Client 02 is inspected. Both Left/Right choices in window placement become disabled during any ongoing manager operation to avoid displaying a different choice during a pending IPC request. Reused existing `app.css` classes with a compact low-emphasis Planned virtuals section. Added focused source regression checks to existing `compact-manager.test.ts` without introducing a new UI framework or any new IPC.

**Actual source proof:** exact GitHub `Local` readback, limited static checks for balanced HTML element tags and Svelte `if/each` blocks, the absence of any action control in planned 03/04 slots, disabled Create Virtual/unchanged backend command registration, CSS class presence and test-owner expectations. This is **not** a Svelte compiler result, real Chromium/WebView2 rendering, assistive-technology test, Node 24 Vitest/typecheck, Tauri Windows bundle, Windows sign-in or Minecraft gameplay acceptance. Design aesthetics, fonts, density, screen reader focus and real window sizing still require real rendered inspection.

**Open decisions and blockers:** three user-visible names still differ by design history (`Virtual Manager` UI, `Minecraft Instance Manager` window, `M-Bedrock Virtual Clients` package). Installed package identity is NOT renamed here because it requires migration review. Private engine source remains not compiled/signed/installed, service identity cannot safely coexist with official Sandboxie, and Client 03/04 cannot be claimed functional. The existing official Client 02 idle-crash root cause is still unknown. These are genuine acceptance blockers; do not tell users the four-client product is ready for a full acceptance run.

**Next evidence:** finish exact-source checks possible in this environment, then one final Windows acceptance campaign with real UI/WebView2 inspection and actual Minecraft Education sessions. Keep the user PC untouched until they elect to run that campaign.

## Native v1.18.6 builder report safety — 16 October 2026

**Confirmed source defect / owner:** `tooling/isolation-source/build-owned-v1186.ps1` formerly guarded `-ReportDirectory` only against overlapping `engine/isolation-owned`, while `-PackageDirectory` was forbidden inside the entire Git checkout. A new report directory inside `planning/`, `.git/`, another tracked repo subtree, or an external junction pointing into the checkout could therefore be created AFTER the clean-tree Git preflight and BEFORE/WHILE native compilation. That breaks source/build reproducibility and can poison later invocations or evidence. This risk comes from the exact preflight ordering and is not a proven Windows exploit.

**Bounded fix:** before checking Git or invoking MSBuild, `-ReportDirectory` now refuses drive roots, paths contained by or containing the *whole repository checkout*, existing output paths, missing parent folders and ANY reparse-point ancestor of the parent. The existing later checks of 3 build stages, 16 outputs and SHA-256 report remain unchanged. `-PackageDirectory` retains its separate non-overwriting preflight. No service/driver install, signer, provider selection, privileged transport, automation or game process was modified. The active README now requires the report directory outside the entire checkout.

**Proof obtained at exact source revision:** GitHub `Local` readback of the edited PowerShell source, existing `build-profile-v1186.json`, three actual `.sln` files and both output `.props`. Added a test in the existing `build-owned-v1186.test.mjs`. Seven of eight current static/source regression bodies executed and passed in the available V8 isolate with a limited strict-assert fixture adapter (NOT the canonical Node 24 test runner); the eighth disposable-Git fixture was independently executed in the available container's `git` CLI and produced its expected ignored-`.obj` observation. This does not execute Windows PowerShell, MSBuild v143, WDK, native driver signing/loading, Tauri or Minecraft. **All Windows native and gameplay tests remain NOT_EXECUTED.**

**Next real dependency:** make a full actual Windows MSVC/WDK build and examine authentic output/logs at final acceptance; if no matching Windows execution context exists, keep the builder and private Create Virtual blocked. No request for intermediate testing on the user's PC.

## Owned launcher path admission parity — 16 October 2026

**Reproduced cross-language defect:** `engine/isolation-native/owned-runtime/minecraft-profiles.mjs::ownedMinecraftLaunch` accepted absolute Minecraft Education executable paths with an alternate data stream colon inside a component (`C:\\folder:stream\\Minecraft.Windows.exe`) or a forward slash inside a component (`C:\\folder/other\\Minecraft.Windows.exe`). The exact native `mivubi_owned_start_policy.h::MivubiParseOwnedStart` rejects both. A valid JS plan therefore produced a command that native `Start.exe` would refuse. This was demonstrated by running the old JS function in V8 and checking the native rejection branches before mutation.

**Fixed in canonical JS planner:** reject `:` and `/` after the `X:\\` drive prefix, preserving the permitted drive colon and all prior traversal, filename, client-ID and shell-character checks. The JS planner is intentionally conservative (it may reject other paths allowed by C++; it must not admit paths native code rejects). Added explicit negative cases to the existing `minecraft-profiles.test.mjs` and pinned its checks to the two concrete native rejection branches. Extended `mivubi_owned_start_policy.test.cpp` with the same two path cases; no C++ production parser modification, new runtime process, account access, driver install or Tauri registration.

**ACTUALLY EXECUTED proof this revision:** all **15 existing JS owned-profile test bodies** (including added path cases) passed in the available V8 isolate using a limited equivalent basic assertion adapter and current GitHub-backed source fixtures; this is **not** Node 24 `node:test`. More strongly, the exact native C++ parser header and test were materialized in an isolated build environment: `git hash-object` confirmed exact Git blob IDs `e4c7e0022b1c24a0a98eb821a9aa274e00e0a35d` and `7550ab8db69804b98dc3ba66524121fbaab920bb` against `Local`. Actual `g++ -std=c++17 -O2 -Wall -Wextra -Werror -pedantic` build and 28-case native parser test PASSED; a second `-fsanitize=undefined -fno-sanitize-recover=all` build/test also PASSED. These are genuine **portable C++ source execution** for two exact files, not the Windows-native Sandboxie solution, driver ABI, shell/PowerShell integration, Node24 full verification, installed runtime, login or Minecraft gameplay.

**Decision / next dependency:** source-side Create Virtual planning/launch admission advances, but `desktop_create_virtual` remains unregistered/disabled. The v1.18.6 private `SbieDrv/SbieSvc` namespace still conflicts with the official Sandboxie pilot. Do not install/activate it or mutate persistent `MinecraftClient02` until a real signed, exclusive Windows driver build, host ownership verification and user acceptance are obtained. Continue only with a new reproducible source defect; no reason to add a second readiness/monitoring layer.

## Create Virtual source safety: disable before configuration — 16 October 2026

**Scope completed this batch (source-only):** `engine/isolation-native/owned-runtime/minecraft-profiles.mjs` now puts `SbieIni set MivubiEdu0[234] Enabled n` and its exact `query` readback **before** the existing canonical Minecraft Education compatibility settings. Only after their ordered set/readback pairs does it attempt the final `Enabled=y` write followed by its own readback. Each target thus has 18 guarded actions (9 writes + 9 readbacks); the historical eight-command write-only list remains for source compatibility but is explicitly not an approved executor. Final-state `matchesOwnedMinecraftReadback` still expects the eight FINAL setting values (including `Enabled=y`); it must not conflate the earlier `Enabled=n` proof with the final state.

**Failure semantics:** no executor is enabled or exposed over Tauri, and no signed/installed private provider has been verified. The returned source-only plan explicitly requires manual reconciliation of partial configuration, rejects deletion/cleanup and has no automatic retry or resume. A failure after the last `Enabled=y` write but before its successful readback is **ambiguous**: the section may already be enabled. This is not an atomic Windows transaction, and cannot be claimed to be crash-safe without a future exclusive driver/service, true same-host inventory and real Windows acceptance. It must never touch the official account-bearing `MinecraftClient02` box or silently copy its credentials. Compatibility mode `NoSecurityIsolation=y` / `Template=OpenCOM` remains reduced isolation and requires explicit operator approval.

**Proof obtained:** exact GitHub source readback plus V8 execution of the latest planner: three valid end-state profiles, 15 rejected invalid/stale/occupied inventories, 51 interruption-prefix invariants across Clients 02/03/04, and the complete 54-step bulk source plan. Existing `minecraft-profiles.test.mjs` was updated for 18-step plans, final-readback semantics and partial-failure guard checks. These V8 checks are NOT a full Node24 test run or a Windows implementation test. Native compile, signing, driver activation, Microsoft authentication, actual four Minecraft clients, reboot persistence, and idle crash diagnosis remain **NOT_EXECUTED**.

**Next actual blocker:** a private driver/service cannot be safely activated or used for a live Create Virtual command until Windows MSVC/WDK compilation, signed exclusive ownership, read-only host proof, and the operator's final Windows acceptance are available. Continue other evidence-based source work as useful; do not add a fake production Create button, second provider, new dashboard or speculative Windows mutation.

## Source contract correction: Sandboxie section-name candidates — 16 October 2026

**Confirmed owner mismatch:** `tooling/acceptance/windows/collect-virtual-boxes.ps1` runs Sandboxie `SbieIni query *`, which returns configuration **sections** (including unproven custom sections) rather than authenticated box identities. The UI was corrected earlier but the wire contract still labeled this data `BOX_NAMES_ONLY`, which could let consumers mistake it for verified boxes. The collector and existing active Rust `virtual_inventory.rs` parser, TypeScript `nativeFeasibilityApi.ts` parser/contract, and existing Vitest/Rust fixtures now agree on `SECTION_NAME_CANDIDATES`. Old `BOX_NAMES_ONLY` evidence is explicitly rejected, rather than silently upgraded, on the same read-only IPC. `boxes` remains the existing wire array key for compatibility with the mounted UI; its contents are **candidates only**. Source ownership mapping updated in `docs/system/implementation-map.md`. No new IPC command, schema version, data mutation, app provider, installer or account operation was introduced.

**Actually checked:** exact `Local` GitHub readback of all five edited owner/consumer files, six cross-file contract markers, and actual V8 execution of the extracted current TypeScript parser after removing type-only annotations: 3 valid and 6 invalid examples, including old evidence, incorrect source/readiness, duplicate names and extra private fields. This is a targeted source-parser check; full Node 24 Vitest/Svelte/Tauri, Rust/Cargo and Windows/PowerShell executions are **NOT_EXECUTED**. A live box identity, signed provider, game login and four-client performance remain unverified. No user-PC test is requested before the final campaign.

**Next:** continue only with source defects that can be reproduced or coherently demonstrated against the active owner. Preserve the disabled Create Virtual command, existing account-bearing Sandboxie `MinecraftClient02` and legacy VMware boundaries.

## Code-only blocker closure: repaired Sandboxie inventory PowerShell — 16 October 2026

**Confirmed defect at previous `Local` HEAD:** `tooling/acceptance/windows/collect-virtual-boxes.ps1` had a literal PowerShell regex truncated at `{1,32}` and a duplicated trailing `ConvertTo-Json` output block. This was an actual malformed committed source file, not an inferred runtime issue. The proximate source-edit mechanism was reproduced in V8: JavaScript `String.replace(search, replacementString)` interprets `$'` as the suffix of the original string when a replacement contains a regex ending in `$'`. The canonical `GITHUB_RULES.md` now requires callback/splice literal editing plus committed-byte readback.

**Fixed in the existing collector:** one complete `if ($name -cnotmatch '^[A-Za-z0-9_]{1,32}$' -or -not $seen.Add($name))` validation; one JSON output block; explicit `GlobalSettings` header; `query *` remains read-only to include disabled box names but only yields **candidate sections**, not verified boxes or Create Virtual authority. The raw 512-section limit is distinct from the 128 candidate-name limit because documented template/user sections also consume input rows. Padded raw names are rejected rather than normalized into a different identity. The current `apps/desktop/test/virtual-inventory.test.ts` now rejects a truncated regex, duplicate collector output, missing bounds, name trimming or removed reserved-namespace filtering. All active IPC/result fields remain unchanged.

**Actual validation:** read back the exact GitHub commits and evaluated 11 selected source files with targeted static/structure checks, including V8 parsing of four edited pure JavaScript modules/tests (after stripping imports solely for syntax inspection); all the **selected** checks passed. These are not a complete PowerShell AST/runtime parse, Svelte/Vitest test run, Rust/Cargo checks, signing, Windows driver/service activation, or Minecraft gameplay. No local-PC testing requested; all stronger proofs remain **NOT_EXECUTED**.

**Next:** focus further code-only work on reproducible source blockers and existing test consumers; never interpret helper timeout, box-section candidate names or unsigned built artifacts as authenticated/working Minecraft clients. Preserve the currently mounted official Client 02 provider, account-bearing data, and disabled private Create Virtual.

## Code-only launcher and Sandboxie inventory contract review — 16 October 2026

**Goal and scope:** continue source-only work on official Client 02 launch safety, duplicate request admission, read-only official Sandboxie inventory, and user-facing readiness truth. No local PC request, no private driver activation, no account migration or automatic launch retry.

**Launcher findings (review, no speculative mutation):** active Tauri command `desktop_request_sandbox_client_02` is serialized with other expensive manager operations; `native_clients.rs` has a separate Client 02 launch lock, fresh Sandboxie PID membership inspection, a 30-second helper-completion-based retry gap, explicit-only `Start.exe` request, and bounded PowerShell subprocess timeout. A timed-out launcher may have already spawned Minecraft, so source rightly refuses automatic retry and describes the result as UNKNOWN. No source evidence demonstrates that the manager kills Minecraft when idle. A retry race involving an external launcher/app restart and later-starting game processes remains unproven; do not invent a fix or claim real idle-crash resolution.

**Defect found during all-section inventory review:** `SbieIni query *` reaches upstream `DoListSections` and `SbieApi_EnumBoxesEx(..., TRUE)`, which enumerates configuration sections, **not just proven boxes**. The earlier disabled-inclusive change could present documented `Template_*` and `UserSettings_*` sections as if they were virtuals. `tooling/acceptance/windows/collect-virtual-boxes.ps1` now excludes those namespaces (and `TemplateSettings`) before candidate-name validation; names are still candidate-only and unknown custom sections cannot be authenticated by this call. The collector remains on-demand, read-only and bounded and cannot authorize Create Virtual.

**UI and regression alignment:** `CompactManagerApp.svelte` now calls the result `Sandboxie name candidates (read-only)`, uses `Inspect Sandboxie names` for the action, warns that disabled and possibly non-box sections can be present, and no longer claims the inventory is enabled-only or authenticated. Updated existing `compact-manager.test.ts` and `virtual-inventory.test.ts` to bind the terminology, source-owned auxiliary exclusion and the permanently disabled Create Virtual control. No new IPC fields/schema, runtime permissions, launch operations or workers.

**Actually verified:** exact GitHub source readback; 13 source-level checks PASS covering all-section command, header, auxiliary exclusions, non-mutating collector, honest UI, disabled Create Virtual, tests and no automatic polling. A generic V8 parse attempt on a TypeScript Vitest file was not applicable (TypeScript annotations), and must **not** be reported as a failed or passed compile test. Pinned Node24 Vitest/Tauri/Rust, Windows PowerShell, user-account Minecraft gameplay and native driver build remain **NOT_EXECUTED**.

**Next code-first priorities:** correct any further reproducible backend-only failure handling defect, then run available exact source suites in a valid toolchain. Reserve final Minecraft/Windows acceptance as one campaign; preserve official `MinecraftClient02` box and accounts.

## Code-only batch: readback identity and disabled-box enumeration — 16 October 2026

**Exact source defects confirmed and fixed on `Local`:**

- `engine/isolation-native/owned-runtime/minecraft-profiles.mjs` accepted a readback object with four unrelated own keys and all required identity/value properties inherited from its prototype. Verified by actual V8 execution before the change. `matchesOwnedMinecraftReadback` now requires own `client`, `box`, `key`, `value` properties in addition to exact shape, order, count and expected values. The existing `minecraft-profiles.test.mjs` includes both full-inheritance and partial-inheritance negative cases for all three clients.
- `tooling/acceptance/windows/collect-virtual-boxes.ps1` previously ran official `SbieIni query /boxes *`. Its pinned upstream `engine/isolation-owned/Sandboxie/apps/ini/query.c::DoListSections` shows this filters out boxes when `SbieApi_IsBoxEnabled` reports disabled. Name inventory could therefore omit a real disabled/colliding box. The read-only collector now invokes `query *`, requires the exact `GlobalSettings` header, removes that header, bounds the remainder to 128 boxes and preserves duplicate/name rejection. The existing desktop `virtual-inventory.test.ts` pins this behavior to the inspected upstream source. This **does not** convert official Sandboxie inventory into owned-engine proof or authorize Create Virtual.

**Evidence ceiling:** GitHub exact source readback, actual V8 function execution of 3 good per-client readback cases and 6 forged-prototype negative cases (all expected), and source-level confirmation of upstream `query.c` filtering with collector/test routes. Full pinned Node24/Vitest/Rust, Windows PowerShell execution, actual box inventory on Windows, engine build/install/signing and four-account Minecraft acceptance are **NOT_EXECUTED**. The old missing `node:test` import impression was from a truncated output excerpt; exact file preamble has the correct import and no test-import change was necessary.

**Next code-first priority:** examine remaining genuine backend/launcher safety faults or exact build blockers before introducing new abstractions. Preserve official account-bearing `MinecraftClient02`, keep Create Virtual disabled and do not equate an empty or disabled-inclusive official inventory with owned provider readiness. User PC testing remains deferred to one final Windows campaign.

## Code-only owned virtual readback and package ancestor safety — 16 October 2026

**User directive unchanged:** perform source-verifiable implementation work in GitHub `Local` now, defer target Windows/Minecraft testing to the final campaign. A new code path or authored test is not evidence of operating four playable accounts.

**Builder source correction:** the existing `tooling/isolation-source/build-owned-v1186.ps1` package-target preflight checked only the immediate parent for `ReparsePoint`. An ancestor junction could redirect a lexical outside-repository output into another location, postponing rejection until after MSBuild. The preflight now walks the existing parent chain and rejects any junction/reparse-point ancestor **before creating a report directory or compiling**. The existing assembler remains the final realpath and no-overwrite authority. Extended its existing `build-owned-v1186.test.mjs` regression, without modifying driver installation or game state.

**Owned box configuration contract correction:** `engine/isolation-native/owned-runtime/minecraft-profiles.mjs` previously returned only eight write commands per Client 02/03/04 and flagged `settingsReadbackVerified=false`, while its comments required each setting to be verified. It now also returns `guardedSteps` generated from the same eight canonical settings: for each setting, `SET` followed by `REQUIRE_READBACK` with an exact expected value. The final write remains `Enabled=y`, followed by its readback. Source-only policy flags demand stopping on mismatch and prohibit automatic deletion. A strict pure comparator `matchesOwnedMinecraftReadback(client, readings)` rejects missing, duplicate, reordered, foreign-box, foreign-client, extra/private-field and mismatched records. Neither the plan nor comparator authenticates the host; only a future independently verified owned provider may execute operations. The historical write-only `commands` output remains for existing source consumers and is NOT an approved executable transaction.

**Proof actually obtained:** read back exact edited GitHub source; checked placement of ancestor-path validation before native work and confirmed no install/activation calls added. In V8, executed the exact owned-profile module with three valid client readbacks and 42 negative readback cases; all conditions matched expectations. Updated the existing `minecraft-profiles.test.mjs` with ordered readback and malformed data cases. This is **NOT** a complete Node 24 test-suite execution, Windows PowerShell parser/runtime verification, actual native compilation, signed driver/service acceptance, box modification, Microsoft login or Minecraft crash resolution.

**Next source-only task:** inspect adapter integration constraints and account-preserving failure behavior for owned box create/launch without enabling the currently blocked Create Virtual UI, preparing one consolidated final Windows test plan. Preserve official `MinecraftClient02` and all licensed accounts; do not auto-delete, auto-migrate, auto-retry or silently change Sandboxie profile isolation settings.

## Code-only native build stage and output preflight — 16 October 2026

**User directive preserved:** no incremental Windows/PC testing. Keep exact source proof distinct from successful Windows compilation, signed kernel/service activation, account safety, idle-crash diagnosis and four-client Minecraft acceptance.

**Source audit:** inspected the real v1.18.6 `SandboxDll.sln`, `Sandbox.sln`, `SandboxDrv.sln` and shared output `.props` files. The pinned SbieRelease solution schedules 2 required Win32 support outputs, 13 x64 user-core outputs and the single WDK driver output in its separate final solution. `SboxDrv` is present in the main solution but **not scheduled** for `SbieRelease|x64`; it is scheduled in `SandboxDrv.sln`. The original builder checked missing native outputs only after all three MSBuild stages, wasting later compilation if an earlier stage omitted an expected artifact.

**Implemented:** `tooling/isolation-source/build-owned-v1186.ps1` now derives each stage's required outputs from the existing profile, requires exactly 2/13/1 ordinary nonempty files immediately after successful MSBuild exit for that stage, and refuses the next stage if any is absent. The existing final sixteen-file SHA-256 capture remains untouched. The same builder now rejects optional package output paths that are drive roots, under/above the repository, already occupied, missing a parent or directly under a reparse-point parent **before** any native build or report directory is created. The existing assembler remains the final path and payload authority; no install/sign/load code has been enabled.

**Tests authored:** extended the existing `tooling/isolation-source/build-owned-v1186.test.mjs` to require all 13 scheduled x64 native projects, to assert complete 2/13/1 stage partition and early failure guards, and to assert optional package path preflight precedes compilation. No additional test framework was created.

**Actual proof ceiling:** GitHub source inspection of the three solution project GUID/configuration mappings, plus direct V8 checks showing every selected x64 project is scheduled, profile output counts are 2/13/1, and every new static guard exists before the build loop. The edited Node test file passed V8 syntax parsing in isolation. **Windows PowerShell AST/runtime, Node 24 full tests, MSVC/WDK compilation, native binary output, signing/loading, Minecraft idle stability and four account playability remain NOT_EXECUTED.** Do not infer that either an installer or engine is ready to use.

**Next meaningful source-only owner:** verify the remaining private-driver/service/tool ABI assumptions, and preserve source-only, fail-closed Create Virtual behavior until an actual signed exclusive runtime can be tested in the final Windows campaign. Avoid creating new dashboards or duplicative readiness systems.

## Code-only batch: owned payload integrity and explicit box inventory — 16 October 2026

**Scope and source owners:** (1) `tooling/isolation-source/assemble-owned-v1186.mjs` plus its existing `.test.mjs`; (2) `engine/isolation-native/owned-runtime/minecraft-profiles.mjs` plus its existing `.test.mjs`. All changes are source-only on `Local`; the active official Sandboxie backend, MIVUBI native driver, Minecraft game, credentials and account-bearing boxes are untouched.

**Closed source defect A — time-of-check/source provenance:** tracked Git status alone could not prove that config/license bytes stayed unchanged between the check and the final copy. The assembler now reads the nine required committed blob identities from **one** `git ls-tree -r --full-tree HEAD` call and checks the actual copied files against canonical Git blob SHA-1 (including the `blob <length>\\0` prefix). Compiled binaries continue to use the existing SHA-256 build-report checks. A dirty-worktree refusal remains in place. Existing regression fixtures now bind the nine committed resources and include two late-change rejection cases for `Templates.ini` and `LICENSE.Plus`; no new checker, monitor or daemon was added.

**Closed source defect B — missing box inventory:** `planOwnedMinecraftVirtual` and the historical bulk planner used to default an **omitted** `existingBoxes` parameter to `[]`, silently treating an unavailable inventory as an empty host. Both require an explicitly supplied, bounded list now; null/undefined/malformed inventories fail closed. Existing explicit empty-list planning remains unchanged and still is **not** proof that any Windows host is actually empty. No box configuration executor or driver activation was enabled.

**Actual proof obtained this batch:** GitHub exact-source readback and clean latest-head commit checks; Git tree API confirmed all nine resource/license blobs present in `Local`; V8 ran the actual committed-resource tree parser for 1 valid and 5 invalid cases; V8 ran the exact updated owned-profile module functions for 5 allowed and 11 disallowed cases. Native Node 22 and Git `hash-object` produced identical Git blob SHA-1 on a disposable test fixture. V8 parsed the edited assembler/test source after removing imports solely for syntax-checking. These are limited, explicit checks — **the complete Node 24 test suite, Tauri/Svelte/Rust checks, Windows PowerShell, MSVC/WDK build, signing/loading, and real Minecraft gameplay are still NOT_EXECUTED**.

**Next concrete source-only sequence:** review remaining engine build-stage output/ABI assumptions against pinned native solution files; audit the owned-profile configuration transaction boundaries and negative tests without creating or modifying live boxes; then prepare a single final Windows validation campaign. Do not lift the Create Virtual or package/install guards on source proof alone.

## Code-first completion policy — 16 October 2026

**User directive:** finish every source-verifiable implementation task first; do **not** require or request incremental local-PC Minecraft testing. Reserve Windows/WDK driver loading, account sign-in, idle-crash reproduction, four-game soak, GPU/FPS measurement and installer/install lifecycle acceptance as one final operator test campaign, clearly marked **NOT_EXECUTED** until it happens. GitHub source edits do NOT convert these outcomes to PASS.

**Completed source correction on `Local`:** `tooling/isolation-source/assemble-owned-v1186.mjs` now refuses tracked working-tree modifications anywhere under `engine/isolation-owned/` or the v1.18.6 build profile before copying artifacts/resources/licenses. The verifier previously attested committed HEAD bytes, whereas the assembler copied some uncommitted on-disk inputs. Ignored compiled binaries remain allowed because the build report hashes those independently. The existing assembler regression suite was extended for dirty tracked inputs and a real Git staged/unstaged/ignored binary fixture; no engine, game, account, Windows driver, provider or box configuration was changed.

**Proof actually executed:** source read-back from GitHub; V8 syntax parsing of the edited assembler/test source with module imports removed for parsing only; independent real Git command fixture under local Linux Git, confirming ignored binary output is permitted and tracked staged/unstaged source changes are detected. This is **NOT** the full Node 24 test runner, a Windows native build, a compiled/signable driver or a Minecraft result. The two new regression test bodies were authored, not executed by the pinned Node 24 suite.

**Next code-only work queue, in order:**
1. Inspect the exact current v1.18.6 owned native build and assembler source for real build-stage errors or missing artifacts. Patch only reproducible source contradictions; retain the standalone build-only/no-install policy and existing source tests.
2. Close owned engine's account-safe provider configuration/launch integration in source only where driver/service ownership is independently enforceable. Keep Create Virtual blocked and official Client 02 selected until an exclusively signed tested owned driver exists; never add an unconditional activation path.
3. Reconcile the four-client UI/API contracts against actual supported box identities (02/03/04), preserve already running official accounts, and add negative tests for duplicate/unknown sessions and rollback. Do not mark four-client support available.
4. Run every available portable source test in a compatible environment where possible and record exact command/runtime/exit code; no fake Node24/Rust/MSVC PASS. Prepare one final consolidated Windows acceptance checklist rather than repeated per-commit local requests.

**Current unresolved runtime risk:** the 14 October official Client 02 idle crash has no proven root cause. A bounded A/B/C reproduction protocol is recorded in `docs/validation/native-pilot-localtest.md`; execution is deferred under this directive. The source change above does **not** fix or explain that crash.


## Product direction: direct, minimal Sandboxie/Minecraft optimization — 15 October 2026

**User clarification (highest current priority):** do not overdevelop. Optimize the original Sandboxie-derived engine and Minecraft Education only where there is a real simple safe improvement. No new performance dashboards, monitoring agents, extra modes, collectors or abstractions. Prefer keeping proven functionality and stable licensed Minecraft accounts over speculative resource savings.

**One bounded source fix committed:** `tooling/acceptance/windows/collect-sandboxie-feasibility.ps1` no longer starts a second `Start.exe /listpids` helper when the first query failed. A valid first snapshot STILL triggers the second query, whose exact PID inventory must match; UNKNOWN and the existing trusted DLL fallback remain unchanged. No original Sandboxie C/C++, driver, config, Minecraft graphics or game processes were modified. Updated an existing regression instead of introducing a new test infrastructure.

**Verified:** seven representative first/second snapshot decision cases agree with previous admission behavior; source static checks confirm second helper guarded, dual snapshot check and DLL fallback preserved. Windows PowerShell/real client runtime and measured CPU/RAM improvement are **NOT_EXECUTED**. The effect is elimination of one unnecessary helper invocation on failed first queries, not a demonstrated percentage improvement.

**Next:** stop adding performance tools. Only change another Sandboxie core/launch hot path after finding actual safe repeated work or runtime bottleneck; Minecraft savings should use supported in-game visual settings chosen deliberately without modifying account/world files. Prioritize two-client idle crash and real owned-engine build/stability evidence.

## PresentMon offline frame pacing diagnostics, runtime unchanged — 15 October 2026

**Continuing user priority:** make CPU/RAM/GPU optimization evidence-driven while **preserving current Minecraft/Sandboxie stability**. No driver install, service/control mutation, game process priority, graphics/FPS cap, DLL hook or continuous sampler was introduced.

**Implemented:** one `tooling/acceptance/analyze-presentmon.mjs` offline CLI and `analyze-presentmon.test.mjs` under the existing acceptance tools. `npm run frametime:review -- --csv "<existing-PresentMon.csv>" --process-id <operator-selected-PID>` reads an already-recorded CSV ONLY. Parser recognizes canonical `Application`, `ProcessID`, `SwapChainAddress` and exactly `MsBetweenPresents` (or historical `msBetweenPresents`), refuses silently treating PresentMon v2 `FrameTime` as an equivalent; official PresentMon README/CsvOutput.cpp and the 2025 schema-change discussion confirm versions differ. Exactly one allowed Minecraft Education executable and one swapchain per selected PID; other PIDs ignored, a reused PID with a foreign image or multiple swapchains blocks review. Input bounded to 32 MiB / 250,000 rows / 128 columns / 8,192 characters per CSV field, RFC4180-style quoting handled, at least 30 valid selected frame intervals and at most 5% unavailable accepted. Output contains only statistical present-interval summaries: mean, p50, p95, p99, intervals exceeding 33.333 ms/50 ms, sample count and coverage. It does NOT expose raw PID, application, swapchain, file path, frame-by-frame contents, account or world; no JSON output is written to disk by the reviewer. Existing CPU/GPU report schema-1/2, native performance collector and active UI are UNCHANGED.

**Compatibility:** reviewed known PresentMon columns against https://github.com/GameTechDev/PresentMon/blob/main/README-ConsoleApplication.md and upstream https://github.com/GameTechDev/PresentMon/blob/main/PresentMon/CsvOutput.cpp. One-frame `FrameTime` or dropped/repeated frames do not automatically mean the same quantity as `MsBetweenPresents`. The new report expressly states `fpsVerified=false`, `frameDisplayTimingVerified=false`, `optimizationVerified=false`, `gameplayVerified=false`, and `accountIsolationVerified=false`. A user-supplied CSV alone is NOT authenticated, and these statistics cannot prove that four licensed game sessions are playable or establish true displayed FPS; they are diagnostic clues to potential stuttering.

**Actually executed:** developed and ran the matching analyzer and its 8 complete CLI/test fixtures under real local **Node v22.16.0**, `node --test`: **8/8 PASS**. Test cases include selection of one PID, old-header/BOM compatibility, v2 FrameTime refusal, invalid/malformed CSV, PID reuse/multiple swapchains, missing frame coverage and actual CLI plus symlink/path/PII output refusal. Source and tests were committed on `Local`, `test:presentmon` is included in the existing `verify:source` sequence. Note repo explicitly pins Node24, so canonical Node24 suite is NOT_EXECUTED. Source data was entirely synthetic, not a real PresentMon trace; target Windows/ETW/Minecraft graphics and frame playback/stability are NOT_EXECUTED.

**Next:** acquire a real authorized Minecraft Education PresentMon CSV on the target machine with one controlled world and settings, analyze one/then two isolated sessions, and independently confirm dropped frames/display presentation and game idle stability. Only after comparing the same workload with observed FPS/frame pacing may an opt-in Eco mode be developed. The unbuilt MIVUBI-owned Sandboxie C/C++ driver/service must not be treated as operational. Preserve official `MinecraftClient02` account data.

## Stability correction — Windows GPU/CPU capture cannot destroy or misread reports — 15 October 2026

**User priority:** keep the Sandboxie/Minecraft app and game sessions stable while continuing performance work. Current batch fixes actual diagnostic-file/compatibility hazards, without changing the active Minecraft provider, owned C/C++ core, box profiles, accounts or games.

**Discovered and fixed in the original source owner:** on **Windows PowerShell 5.1**, `Set-Content -Encoding UTF8` creates a UTF-8 **BOM** (confirmed by Microsoft PowerShell encoding docs), whereas the existing `evaluate-native-performance.mjs` CLI parsed raw UTF-8 JSON without stripping a BOM. A real report could be rejected despite correct metrics. `tooling/acceptance/windows/collect-native-performance.ps1` now serializes to UTF-8 **without BOM**, writes exclusively to a new GUID-named temporary file in the final output folder, closes/flashes it and uses the two-argument `File.Move` to publish the completed report **without overwriting any previous final report**. A normal failure cleans only its own temporary file; an abrupt process kill might still leave an incomplete temporary file but cannot publish it as a final capture. The old `Test-Path` + `Set-Content` check/write race is gone.

**Backwards compatibility:** `tooling/acceptance/evaluate-native-performance.mjs` CLI strips one leading U+FEFF marker when reading old Windows PowerShell 5.1 report bytes. The evaluator's strict schema-1/schema-2 validation, UNKNOWN/null semantics, PID/account privacy, GPU unverified status and refusal to claim FPS/frametime/GPU performance are unchanged. A cross-platform Node CLI fixture test for old BOM and current no-BOM reports was **authored**, but not run under the canonical Node runner here.

**Stable GPU behavior:** if either operator-selected game process identity became missing/inaccessible during the timed CPU/RAM capture, the optional `-GpuSnapshot` operation now skips `Get-Counter` entirely and reports both GPU hints as UNKNOWN/null. It cannot accidentally read a replacement PID or add GPU-counter overhead after an already inconclusive measurement. GPU probing remains explicitly optional, one-shot and after the timed CPU capture, never a background daemon or gameplay-setting mutation.

**Proof level:** **17/17** executable V8 source/evaluator and collector-contract test bodies PASS from the current GitHub sources. The eighteenth test invokes a real Node CLI process and remains **NOT_EXECUTED** in the available V8 harness. Canonical Node24 tests, Windows PowerShell execution, GPU PDH performance counters, actual GPU/FPS/frametime measurements, MSVC/WDK native build, Minecraft sign-in and multi-virtual stability all remain **NOT_EXECUTED**. No measured efficiency gains or stable four-game acceptance are claimed.

**Next meaningful execution:** run the existing canonical `npm run test:native-performance` and real PowerShell capture on a compatible Windows host, verify complete BOM-free JSON and no overwrite even on repeated/concurrent requests, and establish actual FPS/frametime data before enabling Eco/Performance mode. Continue preserving official Sandboxie Client 02 account-bearing data and the untouched owned v1.18.6 kernel/service/DLL source while evidence is missing.

## Optional Windows GPU snapshot in existing CPU/RAM collector — 15 October 2026

**Actual feature source committed:** `tooling/acceptance/windows/collect-native-performance.ps1` has an **opt-in** `-GpuSnapshot` switch to collect a single diagnostic GPU performance-counter snapshot AFTER the original CPU/working-set sampling window. Normal invocation still emits exactly **schema-1** CPU/RAM evidence and does NOT invoke `Get-Counter`; no daemon/auto-sampling, game priority changes, new desktop provider or Sandboxie configuration changes were introduced. Opt-in emits **schema-2** with the original measurements and a strictly bounded `gpuDiagnostic` field. Windows PDH supplies at most a largest measured 3D GPU-engine instance percentage and largest GPU dedicated-memory counter instance for each manually selected, identity-pinned process. The probe requests two rate-counter samples once at one-second spacing, filters counter-instance PIDs without serializing them, and returns `UNKNOWN` with null values if counters are absent, paths are localized, values are invalid, or the process identity changed. The timed CPU duration is frozen before this optional diagnostic so it cannot be mistaken for game performance.

**Reviewer contract:** `tooling/acceptance/evaluate-native-performance.mjs` accepts strict schema 1 and strict schema 2, preserves the exact historical schema-1 result, explicitly returns `gpuPerformanceVerified=false`, `frameTimingVerified=false`, `gpuCounterMemoryMayBeStale=true` for schema 2, and retains `GPU_VRAM`, `FRAME_TIMES`, authentication and multiplayer as **missing acceptance evidence**. It rejects invented FPS fields, source PID/account identity leaks, impossible utilization/memory numbers and malformed UNKNOWN readings. Before/after numeric comparisons are withheld across mixed evidence schemas; any allowed numeric difference is CPU/RAM only, not a GPU gain. `tooling/acceptance/windows/README.md` includes the opt-in command and warning.

**Technical accuracy:** GPU performance counters are administrative diagnostic data, not high-frequency graphics profiling (Microsoft: https://learn.microsoft.com/en-us/windows/win32/perfctrs/about-performance-counters). Microsoft documents cases where GPU Process Memory counters overreport/stale memory (https://learn.microsoft.com/en-us/troubleshoot/windows-client/performance/gpu-process-memory-counters-report-wrong-value). The captured values are isolated per-counter **hints** and are not total GPU usage, physical VRAM, FPS, frametimes, game-running proof or meaningful four-client readiness. Windows version/GPU driver/localization determine availability. Separate frame-present measurement must be independently collected before implementing a real Eco rendering cap.

**Verification:** all **16** deterministic evaluator and collector-source checks were executed on the exact GitHub source through a V8 JavaScript harness, PASS. This is **NOT** execution of the canonical Node24 `node --test` command, nor PowerShell/PDH/Windows graphics tests. No CPU/RAM/GPU/FPS savings can be claimed from the new diagnostic alone. The original driver/DLL, service, installed official Sandboxie and account-bearing boxes are unchanged.

**Next:** complete a real Windows opt-in capture on the target configuration, verify GPU counter availability/units and independent frametime collection in Minecraft Education; diagnose real frame pacing/graphics settings per version. Only then introduce an explicitly tested Eco mode. Prioritize owned native MSVC/WDK build and authenticated service lifecycle separately; no provider cutover is authorized.

## MIVUBI performance — native read-only Windows service probe — 15 October 2026

**Concrete optimization in active desktop source:** `apps/desktop/src-tauri/src/engine/owned_isolation.rs::inspect_host_service_namespace` now queries Windows SCM directly using `windows-sys` `OpenSCManagerW` / `OpenServiceW` for `SbieSvc` and `SbieDrv`, and `ProcessIdToSessionId` for interactive-session evidence. This replaces the previous active invocation of embedded `probe-host-service-names.ps1` with two CIM/WMI lookups, a separately launched PowerShell process, redirected pipes and helper threads. Only `SC_MANAGER_CONNECT` / `SERVICE_QUERY_STATUS` access is requested; service handles and the SCM handle are closed. The former PowerShell source is retained as a reference only and is no longer invoked by the application.

**Exact safety parity:** any successful `OpenServiceW` means only a PRESENT service name (not engine ownership); only Win32 `ERROR_SERVICE_DOES_NOT_EXIST (1060)` is classified ABSENT. Access denied, unexpected Win32 errors or an unavailable SCM remain UNKNOWN. Unknown/noninteractive/occupied/empty service-namespace cases all retain the preexisting `OwnedHostInspection` result shape with `providerSelected=false`, `trustedDriverLoaded=false`, `activationAllowed=false`, and `gameplayVerified=false`. The single existing Tauri `desktop_inspect_owned_engine_host` command, Balanced exclusive request admission and UI explanation are unchanged. No signed driver is assumed, installed, loaded, started or stopped; no Sandboxie/account/VM/Minecraft state is mutated.

**Source/dependency edits:** `apps/desktop/src-tauri/Cargo.toml` adds only `Win32_System_RemoteDesktop` and `Win32_System_Services` features to the already-used `windows-sys` 0.61 dependency; no new crate dependency. The old JSON host parser is `cfg(test)` only, while production native lookup feeds the same pure `interpret_host_probe` policy. Two Rust regression cases and an existing Vitest source-route regression verify strict error mapping, state parity, and no accidental PowerShell/privileged service calls. Canonical Cargo/Vitest/Windows compilation and live resource benchmarking remain **NOT_EXECUTED**.

**Expected effect vs proof:** removal of a PowerShell/WMI child from each manual engine-readiness inspection reduces the work that MIVUBI requests, but the actual CPU percentage, RAM peak, helper startup time and GPU/FPS changes are **not measured**. This does not speed up or throttle Minecraft rendering, and does not affect the still-unbuilt v1.18.6 isolation engine. Next collect same-Windows-host before/after readiness timing/CPU and game-frame/GPU evidence, fix any real compiler errors, and only then calibrate optional Eco/game graphics controls.

## Active Balanced resource policy — serialize heavyweight app commands — 15 October 2026

**Implemented, not merely specified:** the current Svelte Virtual Manager and Rust/Tauri controller now admit **one user-requested heavyweight operation at a time**. This is the implemented default **Balanced** manager policy; it does not attempt GPU or game-thread throttling.

- `apps/desktop/src/app/presentation/compactManager.ts`: `isResourceOperationBusy` combines ongoing normal UI actions, owned service-name inspection and full candidate-package SHA-256 inspection. `CompactManagerApp.svelte` gates all nine applicable user action handlers and their buttons against this one predicate; it retains the existing explicit, on-demand status/diagnostics, fresh launch/focus validation and no `setInterval` / `onMount` monitoring. Settings states Balanced is active and says Eco/Performance are not operational.
- `apps/desktop/src-tauri/src/commands/native_clients.rs`: all **eleven** existing active Tauri command closures now use ONE nonblocking `MANAGER_RESOURCE_LOCK` (Rust `Mutex<()>/try_lock`) during their actual subprocess, file inspection, picker, window or launch operation, so overlapping direct Tauri requests also fail before starting an additional heavyweight task. The lock is automatically released after success or error, no hidden queue, retry or extra daemon. `native_clients.rs` retains its preexisting separate launch admission and fresh process/box verification. Original Sandboxie core, box profiles and accounts were not changed.

**Actual verification:** nine JavaScript executions of the exact TypeScript busy-predicate logic PASS, and the UI has the expected guard for each of nine user operation handlers. A complete source review also confirmed eleven Tauri commands registered and wrapped, unchanged explicit-only diagnostic UI, no accidental new provider, and no fake Minecraft CPU/GPU caps. Three Vitest and two Rust unit-test cases were authored but have **NOT** been executed by their canonical toolchains; native Windows builds, graphics/frame pacing, process-tree resource measurements and game performance remain NOT_EXECUTED.

**Boundaries/next:** reducing *possible concurrent manager-created helper load* is not proof of measured CPU/RAM reduction. This policy changes no game processes, frame limits, memory quotas or OS scheduling. Existing Sandboxie compatibility uses `NoSecurityIsolation=y`, where Sandboxie Job Object limits may not apply. Next collect actual target-host baseline (MIVUBI idle vs active command, Sandboxie service/DLL overhead and each game process/descendants), plus GPU/VRAM and frametime evidence for one/two/four simultaneous Minecraft Education sessions. Only then implement measured Eco/game-rendering behavior, preserving account isolation and multi-instance playability. The separately required first successful owned v1.18.6 MSVC/WDK build remains NOT_EXECUTED.

## Per-virtual Create planning + engine readiness in the single MIVUBI app — 15 October 2026

**User direction continues:** actual Create Virtual must be managed inside MIVUBI, using its Sandboxie-derived kernel/service/DLL and single Rust/Tauri operational owner, not a second SandMan UI. The original current kernel policy `mivubi_kernel_box_policy.h` admits only `MivubiEdu02/03/04`; general user-chosen sandbox names require a separately validated kernel-policy change.

**Source implemented:** `engine/isolation-native/owned-runtime/minecraft-profiles.mjs` now exports `planOwnedMinecraftVirtual(client, existingBoxes)` for exactly ONE selected client 02, 03 or 04. It shares the same eight persistent compatibility settings and `Enabled=y`-last order as the legacy batch planner; duplicate/unsafe/stale names or overlong inventories are rejected. `planOwnedMinecraftProfiles` remains backward compatible but no longer defines the single-create experience. The planned output is explicitly `PRIVATE_BOX_CONFIGURATION_PLAN_NOT_EXECUTED` and never invokes SbieIni or touches host boxes.

**Mounted app integration:** the existing `CompactManagerApp.svelte` includes `Check engine readiness` beside the disabled Create Virtual control, reusing `inspectOwnedServiceNames()` and the existing Rust/PowerShell read-only SCM namespace probe. `ownedVirtualCreationReason` in `app/presentation/compactManager.ts` translates all four blocked host outcomes to plain-language explanations. A disclosure shows the exactly supported internal Minecraft sandbox identities and states they have NOT been created. No extra Tauri command, new service, selector, background polling, account migration, install or active provider fallback was added. Responsive UI uses the existing `app.css`.

**Verification boundary:** actual JavaScript from GitHub's current planner was exercised in V8 for 15 one-target/bulk/no-overwrite scenarios, all PASS. Readiness projection was independently exercised for five probe states, all PASS, and current source/UI/namespace correspondence was checked. Three Node unit tests and two Vitest tests were authored but not executed by their canonical Node24/Vitest runners; Svelte/Cargo/Windows/Wdk/Minecraft runtime execution remains NOT_EXECUTED. Do not claim a virtual was created, service ownership verified or driver installed.

**Decisive next step:** successful Windows MSVC/WDK native compilation, signed/attested exclusive service+driver acceptance on an account-clean environment, then real per-box SbieIni `set` and `query` execution with atomic workflow accounting, duplicate prevention, safe stopped-state and confirmed readback. Only after independently observed results may the current Create Virtual control be enabled. Never issue a mutation against the current official SbieSvc/SbieDrv registration, delete boxes after ambiguous errors, or treat a staged package/SCM name as authorization.

## Virtual Manager existing-box discovery implemented — 15 October 2026

**Current bounded result:** actual read-only discovery of names of boxes available to the current Windows user in the **installed official Sandboxie pilot**, now invoked from the ONE MIVUBI desktop UI. This is a functional replacement for opening SandMan merely to inspect existing box names; it is **not** successful Create Virtual, owned service activation, account authentication or game playability.

**Code path:** `tooling/acceptance/windows/collect-virtual-boxes.ps1` queries the signed installed `SbieIni.exe query /boxes *` once on user request, limits output to 128 canonical names and reports exactly `EXISTING_OFFICIAL_SANDBOXIE_PILOT/BOX_NAMES_ONLY`. `apps/desktop/src-tauri/src/engine/virtual_inventory.rs` parses and strictly rejects ambiguous names/provenance/false-ready flags. The existing Tauri command module and bootstrap register `desktop_discover_existing_virtuals`; `nativeFeasibilityApi.ts` validates the same contract, and the mounted `CompactManagerApp.svelte` displays observed names on request without background polling. A failed query shows an error, not a fabricated zero-box state. `Create virtual` still stays disabled, because the MIVUBI-owned v1.18.6 driver/service have not been built, signed or proven active.

**Native create capability constraint from *actual current C/C++ source*:** `engine/isolation-native/owned-runtime/mivubi_kernel_box_policy.h` only admits `MivubiEdu02/03/04`. Arbitrarily named virtuals are **not yet supported by that kernel policy**. The current `SbieIni.exe` C source accepts `set <section> <key> <value>` and read-only `query /boxes *`, but enabling writes before independently verified exclusive owned-driver/session proof would risk modifying the official account-bearing provider. A proposed executable configuration transaction was **not saved**; do not claim in-app Create Virtual is implemented.

**Evidence:** 11 direct V8 checks of the exact TypeScript inventory parser PASS (valid/empty inventory, duplicated/unsafe names and readiness/account overclaims rejected), and 11 source routing/guard checks PASS. Rust unit-test fixtures and Vitest tests were authored but not executed with canonical runners. **No Windows PowerShell execution, real box enumeration, owned-driver activation or sandbox creation** is claimed. Current revision's source can be inspected in `apps/desktop/src-tauri/src/engine/virtual_inventory.rs` and `apps/desktop/test/virtual-inventory.test.ts`.

**Next concrete gate:** produce and verify the owned Windows service/driver runtime in an isolated account-clean environment, then bind its *actual* SbieIni configuration/query transport to a single Rust Create Virtual operation with duplicate refusal, persistence and readback before enabling the existing UI control. Keep the original source, upstream license notices, official pilot data and historical VM state intact.

## Integrated Virtual Manager product correction and first app implementation — 15 October 2026

**Explicit user decision:** MIVUBI is the APP containing the original Sandboxie-derived isolation system, not a Minecraft-only launcher nor a visual wrapper delegating virtual setup to the external Sandboxie Plus GUI. The main user object is a **Virtual** that MIVUBI will create/configure/run/inspect/manage from one desktop UI. Minecraft Education is the first supported usage/profile, not the abstraction's hard limit. Retain original C/C++ kernel/service/DLL as one private core; Rust/Tauri stays the sole authoritative operational controller and Svelte owns UI. No parallel backend, duplicate manager, second SandMan-style UI, or silent fallback to historical v1.18.5.

**Implemented in the existing mounted desktop during this batch:**
- Found and corrected a concrete disconnected IPC contract: `apps/desktop/src-tauri/src/engine/owned_isolation.rs` accepted current v1.18.6 schema-2 file packages, but `apps/desktop/src/app/bridge/nativeFeasibilityApi.ts` rejected the Rust inspection output because it only recognized v1.18.5. Both versions now use the same strict IPC shape: legacy v1.18.5 requires 24 artifacts, current owned v1.18.6 requires 25 including `LICENSES/LICENSE.Classic`. Neither can report provider selected/driver loaded/installation permitted/gameplay verified. Added `apps/desktop/test/owned-candidate-bridge.test.ts` to the existing Vitest directory.
- The single `CompactManagerApp.svelte` now uses **Virtual Manager** as the user-facing main view; differentiates Windows Host Minecraft 01 from current existing isolated Virtual 02. Settings identifies M-Isolation-Engine as the internal runtime and distinguishes staged v1.18.6/v1.18.5 package inspection. Virtual creation is visible as a **disabled** action with the genuine missing-driver prerequisite. An unbuilt package does NOT become a usable virtual, and no forged create command/virtual data record is added. Existing Client 02 official-pilot Open/Status/Focus/Move commands and their safety verification remain intact.
- The responsive style is consolidated in the existing `app.css`, and existing UI regression assertions were updated rather than creating another UI framework.

**Actually checked:** ten direct V8 executions of the exact current TypeScript parser logic (with TS type-only syntax stripped and actual validation unchanged) PASS for version pairing, missing/extra fields, false readiness and counts. Source/DOM contract consistency with the current mounted Svelte app and CSS was also inspected. The authored Vitest/Rust tests are **NOT_EXECUTED** by their canonical runners, nor is Svelte/Cargo compilation, native Windows driver/service installation or Minecraft gameplay.

**Next true product functionality—not another synthetic validator:** implement the owned engine's safe Windows service/driver lifecycle on a signed, exclusive non-account-bearing test host; then connect the existing Tauri Rust controller to native Sandboxie configuration/query/start operations for transactional **Create Virtual** with readback, duplicate-name refusal and safe failure recovery. UI must enable Create only on actual runtime capability evidence, never because native source or a package directory exists. Until then, the current official Sandboxie Client 02 remains a protected compatibility pilot and not the final MIVUBI-owned virtual provider. No account/world reset or VBox/VMware resurrection.

## v1.18.6 native C/C++ engine -> built payload -> existing desktop inspector — 15 October 2026

**Response to source/application integration blocker:** previous development left authentic owned Sandboxie C/C++ v1.18.6 source, a build-only PowerShell script, historical v1.18.5 packaging tools, and a Rust desktop candidate inspector disconnected. The exact editable native source remains `engine/isolation-owned/` (1,627 files/11 MIVUBI changes); **no original kernel/service/DLL files were rewritten or removed** in this batch.

**Implemented now:** the single `engine/isolation-native/owned-runtime/build-profile-v1186.json` owns the full current three-solution MSBuild stage sequence, sixteen expected binary outputs, five source resources, and four original licenses. `build-owned-v1186.ps1` consumes this profile rather than maintaining its own duplicate paths; it accepts optional `-PackageDirectory` for an explicit build+package request after the successful compiler report is saved. `tooling/isolation-source/assemble-owned-v1186.mjs` is the current native package owner and binds actual build-result JSON, reviewed `Local` HEAD/owned source tree, 16 binary SHA256 digests and packaged source/resource/license bytes to one **schema-2 MIVUBI_V1186_CORE_STAGED_NOT_INSTALLABLE** payload. Missing/tampered input, reused target, unsafe source/result or incomplete build fails closed; assembly never signs, installs or loads the kernel driver. The existing Rust inspector `apps/desktop/src-tauri/src/engine/owned_isolation.rs` now recognizes exactly that schema and provenance in Settings while preserving v1.18.5 legacy behavior. No second app backend or automatic provider selector was introduced.

**Active command ownership fixed:** `npm run isolation-source:assemble` now invokes the **current** v1.18.6 assembler; the earlier `assemble-engine.mjs` / `assemble-private-engine.mjs` are history, not the selected route. The new `assemble-owned-v1186.test.mjs` is registered in `test:isolation-source`, and existing build-test expectations now use the current shared profile. Source routing was updated in README, `tooling/isolation-source/README.md` and `docs/system/implementation-map.md`.

**Evidence ceiling:** 11 cross-language metadata/safety consistency checks and 20 real V8 executions of the current assembler's pure build-profile/report/path contract checks PASS. The new file-copy fixture tests and Rust schema-2 fixture test are **authored but not independently executed** under Node24/Cargo; the complete native MSVC/WDK build, signed isolated driver/service registration, private Sandboxie process isolation, four account logins and sustained Minecraft gameplay are **NOT_EXECUTED**. A packaged source contract is not a functioning Sandboxie installation. Existing official account-bearing Client 02 and old VMware data are untouched.

**Next first owned-engine deliverable:** obtain actual MSVC/WDK compilation/linker output for `engine/isolation-owned` and correct the first genuine C/C++ error; then run the current v1.18.6 assembler and Rust read-only inspector on real produced artifacts. In a separately safe, exclusive disposable Windows acceptance stage, complete properly signed service/driver lifecycle, account-safe box configuration and real Minecraft sign-in/graphics testing before enabling an owned active provider. Do **not** add more validators, legacy wrappers, parallel manager APIs, or aggressive source pruning without a measured root cause.

## Minecraft isolation optimization — process-only measurement implemented — 15 October 2026

**This batch actually changes tooling source, not the unbuilt Sandboxie native core.** The canonical offline evaluator `tooling/acceptance/evaluate-native-performance.mjs` now derives peak selected-process host CPU percent, mean selected-process working set, observation coverage percent and first missing sample time alongside the original mean CPU / peak working set. It retains **schema-1** Windows report input, exact privacy/field validation, and missing-as-UNKNOWN handling; a missing/inaccessible PID is NOT diagnosed as a Minecraft crash, and a sample means a single selected process, not an entire sandbox process tree.

**Before/after baseline comparison is now implemented in the SAME evaluator, without an extra daemon, polling service, provider or second evidence state machine.** Pass two existing reports to `npm run native-performance:review -- <baseline.json> <candidate.json>`; a single report remains supported. `compareNativePerformance` requires matching requested duration, interval, sample count, logical CPU count, close enough recorded duration and complete uninterrupted process readings before emitting descriptive candidate-minus-baseline CPU percentage-point / RAM MB differences. Missing or mismatched evidence yields `differences:null`. Even a numeric comparison returns `performanceVerdict:NOT_PROVIDED`, `improvementVerified:false`, `sameHostVerified:false`, `sameWorkloadVerified:false`, and `fullProcessTreeMeasured:false`; same CPU core count does not prove the same machine.

**Verification performed:** all ten deterministic evaluator regressions evaluated successfully against the exact GitHub `Local` JavaScript source in a V8 harness with equivalent core assertions. This is an *executed JavaScript logic check*, NOT execution of the canonical Node24 `node --test` command and NOT Windows/PowerShell/WDK/Minecraft acceptance. The operating instructions were updated in `tooling/acceptance/windows/README.md`. No changes were made to the original kernel, DLL, Windows service, selected official Sandboxie provider, profile settings or account-bearing data. Runtime efficiency gains have NOT been measured or claimed.

**Next:** inspect a true on-demand two-process Windows capture from an authorized real Minecraft session; confirm comparable same-host/same-workload conditions with independent operator evidence and capture GPU/FPS/child-process resource use separately where possible. Identify a measured startup/idle bottleneck or first actual crash, then fix its exact owner rather than pruning native C/C++ modules or imposing unproven resource caps. Keep native v1.18.6 build-only compilation as the independent required engine deliverable.

## MIVUBI-specific Sandboxie optimization decision — 15 October 2026

**User direction:** retain the original Sandboxie-derived C/C++ kernel, service, DLL and launcher as the isolation core. Optimize its *use, integration, packaging and measured behavior* for Minecraft Education instead of rewriting it in Rust or creating multiple per-client drivers. Rust/Tauri is the one desktop command authority; Svelte is UI. A single exclusive owned Windows driver/service eventually manages persistent `MivubiEdu02/03/04` boxes, while Client 01 remains native.

**Current proof ceiling:** full editable native v1.18.6 source and exactly 11 reviewed C/C++ edits are present, but the owned MSVC/WDK build, private signed driver/service, Minecraft login and four-client gameplay are NOT_EXECUTED. Do not add arbitrary extra native changes that invalidate the exact 11-file source verifier before the first genuine Windows compiler/linker error is known.

### Ordered optimization work and evidence

1. **Establish an honest baseline (P0).** On the exact authorized, already operating official two-client pilot, compare idle/start/login/world load, CPU and working set, responsiveness, graphics/frame times, crash/event evidence, and account persistence. Existing `tooling/acceptance/windows/collect-native-performance.ps1` and `evaluate-native-performance.mjs` provide process CPU/working-set samples only; they cannot prove FPS, GPU use, account independence or even full process-tree cost. They must not be relabelled as four-client metrics. Never collect tokens, passwords or world contents. The previously observed boxed-client idle crash remains unresolved, so do not benchmark only a successful startup.
2. **Keep one controller, on-demand inspection (P0).** Current `apps/desktop/src-tauri/src/engine/native_clients.rs` is the active official Client 02 boundary. Retain explicit per-client launch, fresh PID/window verification, bounded timeouts, no repeated automatic launch, no extra polling daemon or competing authority. Source-first improvements may reduce duplicate invocations only when a measured operation proves expensive. Never auto-switch the official pilot to the unbuilt private provider.
3. **Minecraft-specific box settings (P0, separate compatibility/security review).** The authored private profile planner `engine/isolation-native/owned-runtime/minecraft-profiles.mjs` preserves three boxes with `AutoDelete=n`, `AutoRemove=n`, `OpenCredentials=n`, `OpenProtectedStorage=n`, `DropAdminRights=y`, and the operator-confirmed login compatibility combination `NoSecurityIsolation=y` plus `Template=OpenCOM`. These settings are *not* stronger security and do not prove separate WAM/SSO sessions. Future reduced-permission alternatives require disposable test evidence and may NOT be silently substituted for this explicit compatibility profile. Never mutate account-bearing `MinecraftClient02` settings.
4. **Resource policy must respect App Compartment semantics (P0).** Upstream Sandboxie documentation says `NoSecurityIsolation=y` skips Sandboxie's root Job Object. Thus `CpuRateLimit`, `ProcessMemoryLimit`, `TotalMemoryLimit`, and `ProcessNumberLimit` implemented through that Job Object do **not** normally enforce limits in this mode. Do not implement a UI slider that falsely claims these controls work. Do not apply arbitrary low memory caps or priority throttling to game, graphics, WAM/COM or helper processes. Any alternative Windows resource-control design requires compatibility, process-tree and gameplay testing rather than a config-only guess.
5. **Package only proven dependencies (P1, AFTER successful native build).** Keep full `engine/isolation-owned/` source and licenses. Map complete x64 + Win32 runtime closure first, including core driver, service, DLL, launcher, SbieIni, injection/COM helpers and resource files. `SandMan` and upstream Classic UI are not intended as separate MIVUBI application windows, but Classic compilation in the upstream solution does NOT authorize early removal of runtime dependencies. Build output may remain larger than final payload. Use the one future v1.18.6 provenance/packaging authority; historical v1.18.5 assembly tooling must not be reused unmodified.
6. **Reproduce and correct stability defects before speed tuning (P1).** Attribute cold-start, login/WAM, credential persistence, COM, graphics and idle-crash failures by the first owning layer (Minecraft/Windows, Sandboxie core, wrapper/controller, box profile); document first error and verify absence of account crossing. Avoid speculative kernel hooks, DLL hook disables, manual copying of account data or broad OpenFile/OpenIpc/OpenCOM exceptions. Changes to native patches require revised exact source verification.
7. **Scale 2 -> 4 only after owned two-client acceptance (P2).** Reuse the SAME owned driver/service for three persistent boxes with independent licensed accounts. Measure per-client and aggregate CPU, memory, GPU/frame pacing, startup latency, time to interactive world, idle stability, window/input focus, restart and multiplayer. Stagger starts only if measured concurrent startup causes failures; do not force arbitrary CPU/GPU/priority caps. Evaluate at comparable app/game versions and host load; do not claim performance wins without same-machine before/after measurements.

**Optimization acceptance:** (a) source test/ABI and account-data safety unchanged; (b) a compiled owned driver/service only promoted after safe exclusive disposable Windows evidence; (c) observed two-client Minecraft startup/login/render/input/restart, no known idle crash during reviewed soak; (d) independently measured latency/CPU/RAM/graphics improvements relative to baseline, explicitly identifying regressions; (e) only then separate real four-account simultaneous, sustained and recovery evidence. Performance targets are to be derived from measured baseline, not invented.

**Immediate next source work:** keep the original 11-file native modification closure intact and finish the current v1.18.6 build-only evidence gate first; separately audit helper/monitor call frequency and final packaging dependency inventory read-only. Do not implement aggressive resource caps, source pruning, provider cutover or `main` release from this optimization decision.

Official technical references: https://sandboxie-plus.github.io/sandboxie-docs/Content/NoSecurityIsolation/ and https://sandboxie-plus.github.io/sandboxie-docs/Content/JobObjects/ and https://sandboxie-plus.github.io/sandboxie-docs/Content/CpuRateLimit/.

## Build-only preflight source correction — 15 October 2026

**Fix applied to the actual v1.18.6 owned native build owner** `tooling/isolation-source/build-owned-v1186.ps1`: ordinary `git status --porcelain` does **not** show Git-ignored compiler intermediate files. The build used to reject only sixteen named executable artifacts, potentially accepting an otherwise clean-looking checkout with stale .obj/.lib/.pdb files. The preflight now also rejects ANY Git-ignored untracked entry beneath `engine/isolation-owned/` via read-only `git ls-files --others --ignored --exclude-standard -- engine/isolation-owned`. It does not clean/delete any local files, restart a build, install a driver, or change the active official Sandboxie pilot. Failed MSBuild/manifest stages now write `failureReason` into the already existing `build-result.json` alongside stage exit codes and logs; exceptions still fail closed.

**Verified in this continuation:** real disposable Git repository with tracked C source and ignored `stale.obj`: `git status --porcelain=v1 --untracked-files=all` reported clean, while the new `git ls-files` query identified the stale object (PASS under Linux Git/Node 22). Four pre-existing/new source contract test bodies for stage paths, flags, compiler logging and project mappings also passed in an explicitly adapted JavaScript V8 harness reading exact `Local` GitHub source; the additional Node-Git fixture test body was separately reproduced in real Node 22. **This does NOT claim the canonical complete Node24 test suite ran** or that Windows PowerShell, MSBuild, WDK, signing, loading, Minecraft sign-in, graphics or multiplayer ran. The Windows path separator used by the prior builder was already a correct single literal backslash; no path-containment change was necessary.

**State:** native source remains 1,627 files / exactly 11 approved modifications; build scripts now have stronger evidence preflight only. NEXT: genuine first Windows build-only run against a fresh clean `Local` checkout, examine the first actual stage error/log, and patch the canonical source owner. Private engine installation, official Sandboxie mutation and runtime provider cutover remain blocked.

## Native v1.18.6 compiler/evidence corrections — 15 October 2026

**Implemented in `Local` source:** `build-owned-v1186.ps1` now resolves VS2022 MSBuild explicitly through installed `vswhere.exe` (17.x), instead of requiring a developer-shell PATH entry. Each Win32 / x64 / WDK stage runs as an explicitly waited compiler child process with separate immutable stdout/stderr files and the actual `Process.ExitCode` captured in `build-result.json`; a launch exception records `launchFailure` and fails the build. MSBuild node reuse is disabled to avoid leaving compiler workers after exit. The report now includes the selected compiler file version and SHA-256 beside exact Git head/source trees and all sixteen required artifact hashes.

**Source contract audit:** inspected actual v1.18.6 `SandboxDll.sln` / `Sandbox.sln` / `SandboxDrv.sln` project GUID-to-`Build.0` mappings: required service, DLL, Start, Ini, KmdUtil, classic control (build-time reference only) and kernel targets are scheduled in their expected stages. Both `Sandbox32.props` and `Sandbox64.props` use the binary paths expected by the builder. Added an in-place regression against these real solution/property-sheet files to `build-owned-v1186.test.mjs`, which is now included in the existing canonical `test:isolation-source` command. Classic control being a compilation target does **not** authorize packaging a second Sandboxie UI.

**Evidence:** exact GitHub source review, actual solution/props comparisons and 30 builder regular-expression assertions evaluated in the available JavaScript context; these are **not** a successful Node test suite or PowerShell/Windows build. Four Node test bodies are authored; full `node --test`, VS2022/WDK compilation/linking, driver signing/loading, profile read-back and Minecraft gameplay are **NOT_EXECUTED**. Do not claim Windows pass or promote to package/installation on this evidence.

**Next single engineering gate:** obtain a real build-only `build-result.json` and stage stdout/stderr logs from the current clean `Local` checkout, then fix the **first observed** compiler/linker/WDK error in its canonical source owner. No official Sandboxie mutation, driver activation or provider cutover.

## Active implementation sequence — MIVUBI owned isolation engine (15 October 2026)

**Product end state:** one ordinary Windows Minecraft Education Client 01 plus three independently licensed isolated Clients 02–04, controlled by one lightweight Minecraft Instance Manager. This is an owned **Sandboxie-derived** Windows isolation core, not a clean-room Sandboxie replacement, a new full guest-VM platform, or a second app lifecycle manager. Preserve upstream license notices and source attribution. Only `Local` is the development authority.

**Current exact source context:** complete editable `engine/isolation-owned/` v1.18.6 native tree (1,627 files), exactly 11 MIVUBI C/C++ source changes; untouched candidate at `engine/isolation-candidates/sandboxie-v1.18.6/`. Build identity `5.73.6-MIVU`, ABI `0x57230`, strict verifier `tooling/isolation-source/verify-owned-v1186.mjs`. The official Sandboxie `MinecraftClient02` two-client pilot remains the ONLY active Windows launch route. Its two-account playability has been operator-observed, but an idle crash and lack of prolonged stability prevent acceptance. The owned core has NEVER been demonstrated running on Windows.

### Ordered delivery: one canonical owner per layer

| Order | Deliverable and implementation authority | Exit evidence — never infer from a source commit |
| --- | --- | --- |
| **0 — Source ownership (source-complete)** | Keep the full upstream candidate unchanged; modify only canonical `engine/isolation-owned/`; verify exact 11-file native patch and v1.18.6 profile. Old v1.18.5 builder/overlays are historical. | 1,627-file tree parity, 11 permitted native deltas, original license notices; `npm run isolation-source:verify` on exact clean `Local` HEAD. |
| **1 — Real native compilation (CURRENT BLOCKER)** | `tooling/isolation-source/build-owned-v1186.ps1`: Win32 support, x64 service/DLL/Start and x64 WDK driver, only from owned tree; record separate immutable logs, exit codes and hashes of sixteen required outputs. First diagnose actual C/C++/linker/WDK errors in their real first-failing owner, not by recreating wrappers. | Authenticated clean exact-head source, successful three-stage MSBuild on Windows with compatible VS C++/SDK/WDK, actual sixteen outputs hashed. **NOT_EXECUTED** as of this handoff. |
| **2 — Coherent owned engine payload** | After Stage 1 passes, adapt the existing singular package/inspector authority for **v1.18.6** (never silently reuse the v1.18.5 provenance schema); close actual binary/template/dependency/license inventory; preserve source and artifact provenance. | Independently verified manifest + complete executable/DLL/helper dependency closure. Bundle stays **NOT_INSTALLABLE**. |
| **3 — Exclusive disposable Windows acceptance** | Driver signing and ABI; isolated `SbieDrv`/`SbieSvc` registration, safe native launcher, box creation/configuration and rigorously owned rollback in a separately authorized disposable Windows environment. Existing same-name installed official provider means **no coexistence**; never switch or stop it. | Signed binary evidence, observed service/driver handshake, `MivubiEdu02` profile read-back, process containment and tested failure/rollback; never from a self-declared lab or static simulator. |
| **4 — Single owned-provider integration** | Add the independently accepted core as the **one** selected Tauri/Rust runtime authority and thin desktop UI path. No parallel provider manager, stale legacy installer, implicit provider fallback or automatic box mutation. Version/ABI/signature/SCM mismatch fails closed. | Actual Windows app launch/status/stop/restart flows with provenance, controlled error UI and separate non-destructive account state. |
| **5 — Minecraft Edition + two-client stability** | Run native Client 01 plus **owned** isolated Client 02 using separately licensed Education accounts. Diagnose the previously reported idle crash, login/WAM, graphics, focus/input, filesystem persistence and CPU/RAM use. | Operator-verified real sign-in, independently controllable rendered worlds, repeated idle/play cycles and relaunch/recovery without crossing or losing identities. Original official pilot observations do **not** pass this stage. |
| **6 — Four-client acceptance** | Expand the same proven engine to owned `MivubiEdu02/03/04`, not three newly invented engines. Gate multi-client process attribution, account isolation, resource contention, focus/window controls and cleanup. | Four simultaneous distinct licensed sign-ins, four responsive worlds, sustained run, restart/recovery and required multiplayer paths observed and documented. |

**Safety/architecture invariants:** never sign/load/install the unsigned private driver, mutate `MinecraftClient02`, disturb official Sandboxie, copy tokens/accounts/worlds into Git, enable old VMware flows, or claim four-client functionality based on PID counts. Source-derived engine is independent to distribute only after upstream license compliance; it is initially **exclusive**, not installed side-by-side with official `SbieDrv`/`SbieSvc`.

**Work possible entirely in ChatGPT + GitHub:** native source reviews, solution/project output-path audit, deterministic tooling corrections, static regression checks, licensing/dependency closure analysis, manifest design and code commits. These are distinct from Windows build, Authenticode, SCM/driver and actual Minecraft proof. Do not let unavailability of Windows stop GitHub-only source work or relabel it as executed.

**Current immediate next task:** perform a v1.18.6 solution/project dependency and output-path audit against `build-owned-v1186.ps1`; correct any verifiable compiler-invocation/output manifest mismatch inside the existing build owner, then preserve source tests and exact-head commit evidence. The subsequent decisive external gate remains **one real build-only execution** and triage of its *first actual* MSBuild/WDK failure. No provider install/cutover is authorized.

## v1.18.6 native build-only entrypoint authored — 15 October 2026

**Implemented:** `tooling/isolation-source/build-owned-v1186.ps1` compiles the **actual current** `engine/isolation-owned/` v1.18.6 native source, not a separate old v1.18.5 fork or official-installed Sandboxie. It first runs `verify-owned-v1186.mjs` against a clean exact `Local` HEAD and requires 1,627 source files, exactly 11 approved native edits, complete upstream source tree parity, the `5.73.6-MIVU` version/ABI profile, and unused report path. It checks for stale required binary outputs before running Win32 `SandboxDll.sln`, x64 `Sandbox.sln`, and x64 WDK `SandboxDrv.sln` in the documented order. It records separate MSBuild logs, bounded stage exit codes and SHA-256 of the sixteen required native artifacts. It does not sign/load/install any driver or activate a provider; emitted evidence remains `NATIVE_BUILD_ARTIFACTS_CAPTURED_NOT_INSTALLABLE` even after compilation.

**Execution ceiling:** source authored and bounded static regression tests added; this ChatGPT session does NOT have Windows MSBuild/WDK, therefore **Windows native compilation/linking, staging into a runtime manifest, code signing, driver loading, Minecraft login and gameplay remain NOT_EXECUTED**. Do not interpret a committed script as a passed build. Existing v1.18.5 `build-private.ps1` and old manifests remain historical and cannot validate the new version.

**Next:** execute `tooling/isolation-source/build-owned-v1186.ps1 -ReportDirectory <new-empty-report-directory>` on an authorized Windows build environment, inspect the first genuine compiler/linker failures, and apply any corrections to the current owned native files only. Do not touch the official account-bearing Sandboxie pilot.

## MIVUBI v1.18.6 full owned source and native migration — 15 October 2026

**Current canonical source is real editable C/C++, not an external
Sandboxie wrapper:** `engine/isolation-owned/` has **1,627/1,627**
v1.18.6 source files. The independent untouched upstream reference
`engine/isolation-candidates/sandboxie-v1.18.6/`
still exactly matches official tree
`f690477c07a40810f479f53973334ceae7e0ab13`.
The approved v1.18.5 private transformation was checked against authentic
v1.18.6 source, generated **24 source hunks in 11 actual C/C++ files**,
then its full edited bytes were staged directly into the owned Git source.
Git tree inspection confirms exactly 11 modified blobs, 1,616 unchanged
original blobs, **zero missing/extra files and zero mode mismatches**.
Original licensing headers and notices remain.

**Source routing:** `engine/isolation-native/owned-runtime/build-profile-v1186.json`
defines the selected source groups for kernel, service, DLL, launcher,
configuration, COM helpers and package exclusions for upstream UIs.
`tooling/isolation-source/verify-owned-v1186.mjs`
is the new read-only exact-source verifier. Old v1.18.5 tools
must not silently become the current build path. The historical Git
commits preserve the older partial source; no second active source
authority or provider has been introduced.

**Only source-evidence claimed here:** V8 execution of the canonical
private source transformation against all 11 original v1.18.6 files
PASS (24 hunks); recursive Git tree compare PASS; new verifier tests
authored but not independently executed in this ChatGPT/GitHub source
context. **Still NOT_EXECUTED:** real full-native MSVC/WDK compile,
kernel signing/loading, driver/service coexistence/ownership,
Windows profile and login validation, Minecraft client stability
and four simultaneous independent accounts. Official Client 02
sandbox, original installed Sandboxie, identities and world data
remain untouched.

**Next single task:** run the strict v1.18.6 owned-source verifier in
a complete checkout and prepare a *Windows native build-only* entry
that compiles Win32 support, x64 service/DLL/Start, then WDK kernel
without signing, installing, selecting a provider or invoking the old
v1.18.5 build route. Only real compiler/linker observations can close
that next gate.


## Full v1.18.6 source import completed — 15 October 2026

**Current user request:** import every source file from the supplied
Sandboxie-1.18.6.zip into Local to begin the owned-engine rebuild. The checkout
was fast-forwarded from the prior Windows handoff to GitHub
`036b87be2077f88216c1bc96cd2d58ea06b3ebf9` before continuing its existing
827-file candidate. The remaining **800 files** have now been added at
`engine/isolation-candidates/sandboxie-v1.18.6/`, for **1,627/1,627 upstream
files** as ordinary editable Git blobs. No source groups, translations,
installer source, tools, resources or original licenses were omitted.

**Decisive proof:** the complete staged candidate Git tree equals official
v1.18.6 tree `f690477c07a40810f479f53973334ceae7e0ab13`, pinned upstream
commit `cdba7096240b93d79c5373aa81a75a1c5c71b883`. ZIP SHA-256 remains
`22d8b004d4fe8dcf05654c4704a8b121c730352c1b406d2f1dad564dea451b33`.
The candidate's upstream files and license headers are unchanged.
Historical partial-import counts below are superseded by this full tree.

**Importer corrections and executed checks:** Windows Git's forward-slash
root path initially failed the raw-string root comparison; resolved Path
comparison fixes that. Inherited parent `text=auto` also normalized one new
upstream header's original CRLF bytes. The root attributes now defer this
snapshot to upstream's own explicit text rules, preserving its original
blobs. The importer stages only missing paths via a NUL-delimited pathspec,
avoiding Windows argument-length limits and re-adding existing blobs.
An actual Windows partial-import fixture with the parent attributes reached
the official 1,627-file tree, left a clean checkout and passed an idempotent
second invocation. No upstream source was edited to bypass tree validation.

**Installer reference:** the supplied EXE remains outside tracked source.
Its SHA-256 is
`3ad29e1ec18618ee9aa4ef4e21ab0548cf2188b94c05c41cb7a07b5d2d0270e2`,
size 24,914,432 bytes, Authenticode Valid / NOVASYNC LABS PTE. LTD.
It was not executed, installed or uploaded. Upstream workflow files are
preserved inside the candidate only, not activated as repository workflows.

**Next:** port/review the existing v1.18.5 11-file MIVUBI native overlay
against the complete v1.18.6 candidate before selecting a build/runtime
cutover. The older owned tree and official account-bearing installation are
preserved. Full source import is PASS; private v1.18.6 build, signing,
driver/service load, crash repair, resource savings and gameplay remain
NOT_EXECUTED. Importing source alone does not fix the reported slow process.

## Source and artifact verification — 15 October 2026 (latest Local audit)

**GitHub-owned v1.18.6 candidate: 827 / 1,627 files (50.83%), 800 absent.** This import adds **104 genuine original files** to the prior 723-file commit, including SandMan launcher/process/box-monitor source, MiscHelpers archive/common code, repository metadata and original assets. Every one of the 827 staged candidate file paths matched the official v1.18.6 blob SHA and Git file mode in a recursive tree audit. Upstream source tree: `f690477c07a40810f479f53973334ceae7e0ab13` (tag commit `cdba7096240b93d79c5373aa81a75a1c5c71b883`). The candidate remains **incomplete**; do not promote or build it as the owned private engine yet.

**Original ZIP, independently inspected:** the user-supplied archive contains 1,627 files, SHA-256 `22d8b004d4fe8dcf05654c4704a8b121c730352c1b406d2f1dad564dea451b33`. Independent extraction and `git write-tree` yielded the exact official tree `f690477c07a40810f479f53973334ceae7e0ab13`, PASS. This verifies local archive completeness, not remote import completion.

**Original installer, inspect-only:** `Sandboxie-Plus-x64-v1.18.6.exe` (24,914,432 bytes) SHA-256 `3ad29e1ec18618ee9aa4ef4e21ab0548cf2188b94c05c41cb7a07b5d2d0270e2`. These bytes exactly match GitHub's published v1.18.6 release asset digest. Installer was **not executed, installed or committed** and is not MIVUBI's private compiled engine.

**Transfer constraint:** GitHub connector text transport is not byte-faithful for some legacy non-UTF-8 translation/report files. Never silently replace a source blob with a re-encoded copy. Use the already committed `tooling/isolation-source/import-v1186-archive.py` with the exact ZIP in a clean `Local` checkout for the remaining byte-exact import, then verify the complete candidate subtree hash. The older `engine/isolation-owned/` v1.18.5 copy and installed official Sandboxie are untouched.

**Runtime proof remains NOT_EXECUTED:** MIVUBI v1.18.6 overlay port/review, MSVC+WDK build, signing, driver/service loading, Windows lifecycle, two-client stability and four-client Minecraft Education acceptance. The locally observed official two-client pilot is not an owned-engine acceptance.

**Next single task:** complete the remaining 800 upstream files as byte-exact writable Git objects and prove the candidate subtree equals the pinned official tree; only afterward port and test the 11-source-file MIVUBI private overlay.

## Uploaded v1.18.6 full source and actual candidate migration — 15 October 2026

Source ZIP supplied by user was verified locally: **1,627 authentic files**,
Git tree **f690477c07a40810f479f53973334ceae7e0ab13**, original tag
commit **cdba7096240b93d79c5373aa81a75a1c5c71b883**.
Uploaded installer EXE is not the source, not executed or installed.

**Direct GitHub proof:** 526 authentic v1.18.6 source files now exist as
ordinary Git files at `engine/isolation-candidates/sandboxie-v1.18.6/`.
All 526 SHA/mode entries match v1.18.6 upstream (5 revised kernel/service/
common blobs were copied byte-identically; unchanged owned blobs reused).
This is **partial** (1,101 missing); not a completed full GitHub source transfer.
The parallel older `engine/isolation-owned/` remains at 526/1,624 v1.18.5.

**Executed source-only proof:** a Python standard-library importer was
successfully exercised on the user's ZIP in a disposable local Git fixture.
It copied the missing 1,625 files after a 2-file partial seed, reached the
exact **1,627-file upstream tree**, left a clean working tree, and passed an
idempotent second invocation. The exact tested importer source Git blob
`b8e3c500597103ea9d0a5f60b1b44fb7199a41c6` is committed under
`tooling/isolation-source/import-v1186-archive.py`.

**Next:** complete the remaining 1,101 v1.18.6 candidate source files to
GitHub (do not mislabel a locally tested copy as uploaded). After full tree
parity, port/review the 1.18.5 MIVUBI native patches against 1.18.6 and run
native build tests in an isolated Windows test environment. No driver install,
official Sandboxie modification, gameplay or multi-account acceptance here.

## Direct GitHub transfer of core runtime sources — 15 October 2026

**Committed migration checkpoint:** 314 / 1,624 pinned upstream v1.18.5
files are copied as normal MIVUBI-owned Git blobs. The complete
`Sandboxie/core/svc` (74), `Sandboxie/core/low` (7),
`Sandboxie/apps/start` (13), and `Sandboxie/apps/ini` (7) groups now
join the preexisting driver (89) and DLL (117).
All 314 actual blob SHAs and executable modes were compared against the
pinned upstream Git tree and matched exactly; 1,310 upstream files remain.
Source transfer only: private native overlay, Windows binary build, driver
installation and Minecraft runtime tests are NOT_EXECUTED.
The complete upstream target tree SHA is
`9b3d37901e0b6cfb0cfa77cfa465b23c758deefe`.

**Next:** import pinned common/shared, messages, build solution and other
remaining upstream paths without touching the official Windows installation.

## Resumable full-source import — 15 October 2026

**Current verified GitHub base**: `1752e98eeba3c5294889580b26537e6d4fa9a679` (Local).
The original checkpoint held 213 / 1,624 genuine pinned v1.18.5 tracked blobs:
89 core/drv, 117 core/dll and seven additional source/license files.
Existing SHA/mode parity against upstream is exact. The source migration
is NOT complete; private native 11-file patch has NOT been applied to owned
files. The external installed Sandboxie pilot is still active.

**This batch** corrects the partial-import blind spot in
`tooling/isolation-source/import-full-sandboxie.mjs`. The importer can
continue from a clean, committed partial `engine/isolation-owned/`,
refuses any changed/foreign blob or path, copies missing pinned binary/text
bytes, and requires exact final upstream Git tree parity before committing.
Negative and completion fixture tests were added. Native Windows build,
driver signing/loading, Minecraft gameplay and crash analysis remain
NOT_EXECUTED. The importer does NOT mutate existing account data.

**Next**: transfer all remaining pinned upstream blobs and verify that
`engine/isolation-owned/` resolves to source tree
`9b3d37901e0b6cfb0cfa77cfa465b23c758deefe`, then apply the
existing exact private MIVUBI native overlay directly to the owned files.
Avoid repeating source-plan work before achieving full source completeness.

## Complete upstream source ownership — 15 October 2026

**User decision:** stop developing only a small external Sandboxie overlay;
copy the *entire* authentic Sandboxie Plus / Classic v1.18.5 tracked source
tree into the existing `Local` repository, then modify native code directly.
The new `tooling/isolation-source/import-full-sandboxie.mjs` uses a
full-history pinned upstream Git seed and `git subtree add --squash` to
create a normal writable `engine/isolation-owned/` directory in `Local`.
It compares the imported complete Git tree SHA to the upstream tag's
tree SHA. A second command `apply-owned-sandboxie.mjs` applies and
commits the existing 11-file MIVUBI native changes directly inside the
imported COPY; the pinned upstream seed and account-bearing host are not
mutated. This is the future native source owner after actual import.

**Executed:** four positive/negative real local Git subtree/patch integration
tests passed on disposable fixture repositories in the ChatGPT Linux
container, including exact imported Git tree parity, independent source
editability, a clean Local boundary and a second committed engine patch.
The Git blobs of the four committed JS scripts/tests matched their
container-executed `git hash-object` source blobs.

**Not done:** the 1,624 authentic upstream tracked files have **not yet been
transferred into the remote MIVUBI GitHub repository**. The available
GitHub connector cannot create a fork or reference unrelated upstream
tree objects (GitHub returned 422), while the ChatGPT container has
no GitHub network/DNS. The user-approved one-time Git subtree import
is implemented, not executed against the real upstream checkout. No
MSVC/WDK build, signed driver, service activation or Minecraft private
engine runtime is claimed. See `docs/system/owned-source-import.md`.

**Next:** perform the single full-source Git subtree import from the
initialized pinned source checkout, then apply the native code as a regular
owned Git commit; only then build and debug real MSVC/WDK outputs.

## Private Minecraft Education login configuration parity — 15 October 2026

**Observed mismatch corrected in source:** the operator-confirmed
two-account Sandboxie 1.18.5 pilot signed in only after applying
`NoSecurityIsolation=y`, `DropAdminRights=y`,
`Template=OpenCOM`, and `OpenProtectedStorage=n`
alongside persistent storage and closed credential settings. The
private Client 02–04 `SbieIni.exe` profile planner previously
omitted those settings, risking recurrence of HRESULT `0x80070424`
even if the MIVUBI Windows driver compiled successfully.

The canonical `minecraft-profiles.mjs` now reproduces the proven
localtest configuration on three **new** isolated boxes, with
`Enabled=y` last; tests cover all expected values and ordering.
The plan explicitly marks reduced security isolation, requires
operator approval, and does not claim settings read-back or login.
Existing occupied boxes and accounts remain untouched.

**Executed proof:** portable JS plan was evaluated in ChatGPT against
expected three-box/24-command contract; this does not prove WAM/SSO
behavior, actual SbieIni calls or Windows source compilation.

**Next:** compile full private native runtime and verify authorized
box settings plus login in an isolated Windows environment.

## Native SbieIni configuration error signaling — 15 October 2026

**Functional defect:** pinned Sandboxie v1.18.5 `SbieIni.exe` always
returned exit code 0 for most `SbieDll_UpdateConf` or
`SbieApi_UpdateConf` failures, so a caller could falsely consider
a persistent Minecraft client sandbox configured.

**Change:** the private native `Sandboxie/apps/ini/update.c` now
returns 3 when the update API fails, retaining exit code 0 for success,
1 for invalid syntax and 2 for wrong password. The canonical overlay
extends to 11 upstream source files and 24 nonoverlapping hunks;
read-only Rust payload inspection and source tests reflect the same closure.

**Evidence boundary:** private source transformation and negative error-code
assertions can be executed against genuine upstream bytes without affecting
official Sandboxie. Native Windows MSVC/WDK binary build, service/driver
loading, persisted profile read-back and Minecraft gameplay remain
NOT_EXECUTED.

**Next:** compile and execute the full private Windows core and verify
SbieIni exit codes and read-back in an exclusive lab.

## Private Start.exe cold-boot launcher fix — 15 October 2026

**Functional integration issue:** the opt-in MIVUBI `Start.exe` mode
performed `SbieApi_GetVersionEx` before `Validate_Box_Name`, the first
normal location that waits for/starts `SbieSvc`. On a clean first run
without an already running service/driver, the private handshake could
reject the valid new installation instead of starting it.

**Source fix:** start and await the existing Sandboxie service broker via
`CallSbieSvcGetUser` **only for admitted private client modes**, then
perform the mandatory private version/ABI check before setting the box
name or launching a child. A failed broker request or foreign driver
remains rejected. This modifies actual private Start.exe source overlay,
not an app wrapper or an installer.

**Evidence:** source patch and regression test cover startup order,
private ABI admission, and tamper refusal. Full Windows MSVC/WDK,
SbieSvc service startup, driver loading, REA binary inspection and
Minecraft runtime remain NOT_EXECUTED; no official host mutation.

**Next:** compile the complete owned Windows fork to validate the changed
broker/launcher interplay against real service initialization.

## Unblocked iterative private native builds — 15 October 2026

The previous private fork verifier required one single direct-child commit
of Sandboxie v1.18.5. That forced rewriting Git history to record each new
compiler or linker fix, blocking normal native build/debug iterations.
The verifier now accepts a linear chain of source-only commits descending
from the pinned upstream revision; it rejects merges and unauthorized paths
in **every intermediate commit**. The final committed source still must
match the canonical exact 10-file overlay with a clean checkout. Existing
build scripts, candidate ABI and package staging retain this one verifier.

**Executed:** a four-case real disposable-Git reproduction on ChatGPT Linux
demonstrated controlled multi-commit acceptance and failure on merges,
unapproved intermediate files and non-descendant ancestry. Targeted Node
regression source is committed. This is a source workflow blocker removed,
not a signed Windows driver or engine runtime acceptance.

**Next:** run the existing private MSBuild pipeline, save first real Windows
compile/link failures, and commit the concrete kernel/service/DLL fixes
linearly without squashing the previously verified source history.

## Kernel query-box-path namespace admission — 15 October 2026

**Implementation:** the existing private Sandboxie kernel source now enforces
the MIVUBI box namespace in `Process_Api_QueryBoxPath` before calling
`Box_Create` for a non-sandboxed caller. Foreign names return
`STATUS_ACCESS_DENIED` directly; they no longer allocate and free a
transient BOX before surfacing an unrelated `STATUS_UNSUCCESSFUL` result.
The earlier named-process start, model-PID clone and Box_Alloc checks remain.

**Source scope:** canonical fork overlay patches ten pinned v1.18.5
files with 22 nonoverlapping hunks. Expected-source verifier and fixture
regression cover the extra query path. Service namespace, kernel ABI,
driver signing and active official Sandboxie installation are unchanged.

**Evidence:** authentic ten-file source transformation and byte-exact
comparison passed in ChatGPT V8. Node test runner, full Windows WDK/MSVC
build, signed driver loading, REA binary analysis, Minecraft gameplay and
four-account acceptance remain NOT_EXECUTED.

**Next:** integrated private Windows build and observed error-driven debugging
in an isolated environment; no migration from the official pilot.

## Model-PID kernel clone admission — 15 October 2026

**Work:** the private Sandboxie-derived `Process_Api_Start` kernel path
now rejects an unowned model PID's sandbox before calling `Box_Clone`.
The gate checks the existing model process's box name, and releases
`Process_ListLock` plus restores IRQL on the rejection path.
It returns `STATUS_ACCESS_DENIED`, rather than misclassifying the
denial from lower-level `Box_Alloc` as insufficient resources.
The prior explicit named-box admission and allocation-level denial remain.

**Source scope:** existing pinned 10-file private fork closure;
the process API gains one nonoverlapping patch hunk (21 total).
The canonical source assembler, strict expected-byte verifier, and
fixture-based regression tests all share this source owner.
No new driver ABI, service namespace, installer action, or provider switch.

**Proof boundary:** static source review of actual pinned upstream
`Process_Api_Start` and repository regression assertions. Node test suite,
full Windows WDK/MSVC compile, signed driver loading, REA binary inspection,
and Minecraft runtime acceptance have not been run in this change.
Existing official Sandboxie, account data, and world files are untouched.

**Next:** full private Windows WDK/MSVC build in an authorized isolated
environment; fix actual compiler/linker/runtime errors before any cutover.


## Kernel Process_Api_Start policy fix — 15 October 2026

**Implementation:** the MIVUBI Sandboxie-derived kernel source now checks
`MivubiBoxNameAllowed(boxname)` in upstream
`Sandboxie/core/drv/process_api.c` immediately after
`Api_CopyBoxNameFromUser`, before looking up an impersonation token or
allocating a BOX. Forbidden requested box names return
`STATUS_ACCESS_DENIED` (not misleading `STATUS_INSUFFICIENT_RESOURCES`).
Valid box names retain the original token/session/Box_CreateEx path.
The existing lower-level `Box_Alloc` denial remains active for
create/clone paths. This change adds a tenth exact pinned upstream
source file, producing 20 nonoverlapping Git hunks; the Rust inspector,
private assembler and source regression tests share the updated list.

**Executed evidence:** actual upstream v1.18.5 file bytes were used for
the exact-source transformation and strict byte check: 10 paths, 20
hunks, PASS. Ten authored pure JavaScript source regression tests ran
in ChatGPT V8 and passed, including altered status/gate detection.
The canonical `mivubi_kernel_box_policy.h` has a matching Git hash
`cdd280a784ccb67e9f6d4c56b0f20cde0fa86921` in the ChatGPT
container. A C smoke test using that exact header passed GCC/Clang and
compiled with clang-cl to a Windows x64 COFF object.

**Limits:** this does not compile the complete Sandboxie driver,
service, DLL or Start.exe. Model-PID clone failures still rely on the
lower-level Box_Alloc gate and existing upstream error mapping; the
new explicit STATUS_ACCESS_DENIED targets the named-box start path.
Actual Windows WDK/MSVC, signing, service load, process isolation,
REA binary inspection and Minecraft account tests are NOT_EXECUTED.
Keep the official Sandboxie installation, accounts and worlds intact.

**Next:** build and debug the real integrated Windows private kernel
and service in a disposable exclusive environment. Do not return to
metadata-only work unless needed to correct a blocking implementation.


## New native kernel function: whitelist BOX creation — 15 October 2026

**Work:** substantive MIVUBI-owned Sandboxie engine implementation,
not a launcher/UI/report-only milestone. The private pinned v1.18.5
fork now changes `Sandboxie/core/drv/box.c`, placing one canonical
`mivubi_kernel_box_policy.h` C gate immediately before `Box_Alloc`
allocates a kernel sandbox object. Both new boxes and cloned child
contexts go through this function. Only the three exact MIVUBI names
(02/03/04; ASCII case-insensitive) are permitted; unowned box names
are denied with `STATUS_ACCESS_DENIED`, before allocation. No new
Windows driver API, ioctl, or service namespace is introduced.
The approved canonical source closure becomes **nine source files**,
**18 source hunks**; Rust inspector and private assembler follow it.

**Executed:** 35 native allow/deny test cases, each run in GCC Release,
Clang Release, GCC ASan/UBSan and Clang ASan/UBSan (all PASS).
Candidate source/test Git SHA matched bytes in the ChatGPT container.
Nine eligible authored JavaScript source regression bodies PASS.
Actual nine pinned Sandboxie source blobs produce 18 nonoverlapping
hunks and pass the byte-exact private fork transform. Whole-driver
MSVC/WDK compilation/signing, REA binary analysis, service/kernel
loading and Minecraft account proof are **NOT_EXECUTED**.

**Next:** actual private full-core compilation with Windows toolchain
in an approved disposable lab, then test kernel box enforcement,
parent/child process isolation and service startup. No active
official Sandboxie installation, accounts, or world files changed.


## Private fork Windows Git CRLF compatibility — 15 October 2026

**Goal:** actual native MIVUBI-owned Sandboxie core binaries that compile
and run. Do not confuse source-work milestones with an operational
replacement for official Sandboxie.

**Implemented:** upstream .gitattributes marks .c/.cpp as text but not
`eol=lf`. A clean Windows fork with `core.autocrlf=true` can have CRLF
working-tree source. The previous private patch generator rejected CRLF
before invoking Git apply, preventing actual private fork construction
on a valid Windows checkout. The owner now normalizes each authorized
file *in memory* to LF for patch creation, rejects mixed/lone-CR source,
does not change `core.autocrlf` or rewrite source files, and preserves
expected source bytes for either clean newline convention.

**Evidence:** all eight actual pinned v1.18.5 upstream file contents
produced exactly the same 16-hunk patch from LF and CRLF representations;
mixed/lone-CR variants failed closed; eight authored pure JS test bodies
executed and passed in ChatGPT V8. A disposable Git checkout with
`core.autocrlf=true` demonstrated that `git apply --check --index`
and `git apply --index` accept the LF patch and preserve CRLF on disk.
The checked-in Git integration regression covers all eight source paths.
**NOT executed:** full pinned Node24 test suite, whole-Sandboxie Git
checkout/build in a Windows MSVC/WDK environment, REA native binaries,
kernel load/service, and Minecraft four-client acceptance.

**Next single step:** test and build the private fork using an eligible
isolated Windows toolchain, then fix actual native compile/load
incompatibilities and inspect artifacts with REA. Do not touch the active
account-bearing official Sandboxie host.


## Private DLL atomic handle-verification cache — 15 October 2026

**Primary goal:** MIVUBI's own functioning Sandboxie-derived native
engine. **State:** source implemented; portable pointer type probe
compiled; full native Windows core, signing, driver load and gameplay
still NOT_EXECUTED.

**Actual implementation:** in the private `SbieDll` source, replace
unsynchronized reads/writes of `MivubiVerifiedDeviceHandle` with
`InterlockedCompareExchangePointer` and
`InterlockedExchangePointer`. Verification remains per opened handle
with explicit invalidation on close/reopen. All modifications stay in
the canonical eight-file fork overlay, not a separate runtime wrapper.
The changed code was checked against Microsoft Win32 pointer-atomic
signatures. The relevant small native C type probe compiled through
`clang-cl` to Windows COFF. Eight authored source regression
test bodies passed in ChatGPT V8, including tampering with cache
admission, reset and atomic operations. Exact eight-file upstream
overlay generated 16 nonoverlapping hunks and matched expected bytes.

**Not established:** thread-safe lifetime of upstream
`SbieApi_DeviceHandle` itself. Atomically storing an approved handle
does not serialize concurrent handle close/reopen or the kernel ioctl
that may already be in progress. No claim of full multi-threaded
correctness, co-installability, authenticated kernel signing or
four-account acceptance is made.

**Next:** prioritize a full native MSVC/WDK build and runtime tests on
an approved disposable Windows environment; fix observed driver,
service and DLL issues before replacing the official backend.


## First valid native Release policy regression — 15 October 2026

**Primary objective remains:** an actual functioning Sandboxie-derived
MIVUBI native isolation engine. Desktop/client-manager additions are not
a substitute for kernel/service/DLL integration.

**Verified defect (fixed):** `owned-runtime/
mivubi_owned_start_policy.test.cpp` used `assert` for all native checks.
CMake Release builds normally define `NDEBUG`, which removes those
checks. The old test passed despite a deliberately wrong expected
result; the new always-on `require` helper fails on the same planted
error, and 26 correct cases pass. Six truncated-input cases were added.
The source policy's combined conditional was made sequential for
auditability; ASAN did *not* show a memory error in the old expression
because C++ short-circuit semantics already protected the missing
character positions. Do not cite it as a demonstrated memory-safety bug.

**Exact candidate proof in ChatGPT:** the two GitHub candidate C++/header
blobs were reproduced byte-for-byte and their `git hash-object`
matched. Native Release `-DNDEBUG` GCC and Clang: PASS, 26 cases
each. AddressSanitizer + UBSan GCC and Clang: PASS. Two CMake Release
builds and CTest PASS. The updated Start.cpp source patch still
matches all eight pinned upstream v1.18.5 source files, produces
16 hunks and passes exact edited-byte comparison in ChatGPT V8.
No claim is made about Windows/MSVC/WDK source compilation.

**Next substantive task:** the private runtime must be compiled as an
actual Windows kernel minifilter/service/DLL/Start set on an approved
disposable lab, with REA inspection of genuine native outputs and
driven bug fixes. Merely running more static/checker tests or modifying
docs must not be treated as product completion. Existing official
Sandboxie and account-bearing profiles are unchanged.


## Core identity memory-safety correction — 15 October 2026

**Priority:** deliver a functioning Sandboxie-derived native MIVUBI core,
not an external management wrapper or new readiness bureaucracy.

**Actual defect repaired:** the shared C identity helper previously ran
unbounded `wcscmp` on the `WCHAR[16]` response of upstream
`API_GET_VERSION`. A foreign driver may return a malformed response without
a NUL byte, risking an out-of-range read before the caller can reject it.
The implementation now takes the caller's explicit WCHAR capacity,
compares the exact marker including NUL only if it fits, and never scans
past that capacity. All private Start and DLL callsites pass 16 WCHAR;
the same C helper remains inlined into kernel and usermode source.

**Evidence actually executed:** C11 test binaries for the capacity-bounded
identity were compiled with GCC and Clang under
`-Wall -Wextra -Werror -pedantic -fsanitize=address,undefined` and
both exited successfully. Cases included 2-element nonterminated version,
16-WCHAR nonterminated version, truncated declared capacity, NULL,
wrong ABI, tampered prefix, correct identity. Exact pinned upstream
eight-file patch/expected-byte comparison in ChatGPT V8 PASS, with 16
hunks and all three caller capacities verified; seven authored pure JS
regression test bodies PASS. Pinned Node24/Rust and Windows MSVC/WDK,
native service/kernel, driver signing and Minecraft gameplay remain
NOT_EXECUTED.

**Next ONE task:** investigate actual native build/link failures with the
full Windows toolchain in a disposable isolated lab; correct the private
core driver/DLL/service source based on those results before attempting
private provider cutover. Existing official Sandboxie and account-bearing
world/profile data remain untouched.


## Private DLL ordinary-ioctl ownership and service shutdown — 15 October 2026

**Goal remains:** working MIVUBI-owned Sandboxie-derived core, not a
wrapper, demo dashboard or documentation project. **State: SOURCE_CHANGED;
Windows signed driver/service/DLL and Minecraft gameplay NOT_EXECUTED.**

**Functional implementation:** `core/dll/sbieapi.c` now checks the
MIVUBI driver version/ABI with an ordinary `API_GET_VERSION` on each
newly opened device handle, before permitting non-version ioctls.
It caches the verification per handle and resets when closing or
reopening, including reused Windows handle values. This avoids
requesting version on every normal game call. It continues to refuse
an official driver with `STATUS_REVISION_MISMATCH`. Private
`core/svc/DriverAssistStart.cpp` invokes the existing `AbortServer()`
shutdown path when it sees the same foreign-driver status, instead of
returning prematurely from the startup thread.

**Patch correctness:** the GetVersionEx source lines changed in three
adjacent places. Separate context hunks caused a reproducible Git
`patch does not apply` failure. The source owner now emits one combined
nonoverlapping 7-line hunk, rather than three overlapping hunks.
A disposable local Git fixture verified the combined hunk using
`git apply --check --index` and `git apply --index`. The exact eight
pinned upstream v1.18.5 source blobs produced **16** nonoverlapping
hunks and passed expected-byte verification in ChatGPT V8. Seven
authored JS source test bodies passed with service/DLL tamper rejection.

**Limits:** This is a source implementation, NOT a compiled Windows
kernel minifilter, production signing, installed isolated service,
authenticated kernel measurement, account separation, crash fix or
four-client acceptance. The normal private DLL ioctl path is guarded;
other external DLLs or kernel callers must not be assumed covered.
The unchanged `SbieDrv`/`SbieSvc` system namespace still prevents
coexistence with official Sandboxie.

**Next SINGLE substantive task:** inspect actual ABI and source build
failures in the complete Windows toolchain when available, then
correct concrete driver/service/DLL incompatibilities and test on a
separate disposable Windows host. Do not execute privileged engine
cutover on the user's official account box.


## Native DLL/service/driver core identity — 15 October 2026

**Primary goal:** MIVUBI's own *functioning Sandboxie-derived isolation
runtime*, not merely a launcher wrapper or audit machinery.
**This batch:** the pinned private fork now modifies eight actual upstream
source files. Coded and statically reviewed, but **not compiled or loaded
using Windows MSVC/WDK**.

**Concrete native changes:** kernel `api.c` continues responding with its
private `5.73.5-MIVU` version marker and pinned ABI `0x57230`. Private
`SbieApi_GetVersionEx` in `core/dll/sbieapi.c` always queries both the
driver version and ABI—even when its caller asks for `NULL` output—and
returns `STATUS_REVISION_MISMATCH` when the connected kernel is not the
MIVUBI fork. Original API signatures, IOCTL values and argument structures
remain intact. In the correct asynchronous service owner,
`core/svc/DriverAssistStart.cpp`, an early mismatch result aborts the
startup path **before** it tries `NtLoadDriver` against an already present
official Sandboxie driver. If no driver is present, the original driver
load/retry flow is preserved. The opt-in MIVUBI Start.exe path still
requires the private identity/ABI.

**Proof:** eight real pinned upstream files at
`abd4c240031cfb46666daa74ca47ed4e48b561e4` were passed through
the updated patch and strict expected-byte verifier in ChatGPT's JS runtime:
8 source paths, 14 patch hunks, exact bytes PASS. Six authored pure
JavaScript source tests were executed in ChatGPT V8 and passed, including
tamper rejection on service/DLL version admission. Portable C/C++ identity
tests from previous batch passed under GCC and Clang. Exact Windows
service startup and SbieDll binary behavior remain **NOT_EXECUTED**.

**Source ownership changes:** eight path names must be kept aligned between
the patch producer's SOURCE_FILES, Rust inspector's PRIVATE_SOURCE_FILES,
and build assembly's source closure. The existing service name `SbieSvc`,
driver name `SbieDrv` and minifilter registration still conflict with
official Sandboxie. This is a fail-closed *version-path* integration, not
total ioctl-channel attestation, cryptographic code-signing proof or
service/kernel installation success.

**Next ONE task:** when a disposable Windows environment with the pinned
MSVC/WDK is available, compile full upstream and private core and verify
SbieApi_GetVersionEx/StartDriverAsync behavior with real driver artifacts,
using REA to guide any actual binary mismatch correction. Before that,
continue only implementation improvements whose source failure can be
demonstrated. Do not install private drivers into the active account host.


## Core driver version identity — source batch 2026-10-14

**Goal:** build an independently functional MIVUBI Sandboxie-derived
driver/service/DLL/Start core, not a wrapper around installed Sandboxie.
**Status:** DRIVER-AND-START SOURCE MODIFIED, portable native tests PASS;
private Windows core NOT_COMPILED, NOT_INSTALLED, NOT_GAMEPLAY_VERIFIED.

**Concrete engine change:** reuse pinned upstream kernel
`API_GET_VERSION`, returning `5.73.5-MIVU` on MIVUBI private builds
while retaining `MY_ABI_VERSION=0x57230`, the original ioctl and struct
ABI. The private Start.exe opt-in three-client path now calls
`SbieApi_GetVersionEx` and refuses owned-box launches unless both
MIVUBI marker and ABI are returned. An upstream official driver cannot
be mistaken for the private engine on this path. The canonical C11
`owned-runtime/mivubi_core_identity.h` source is inlined into the
private driver's API and Start.cpp, alongside the earlier C++17
Start policy. Existing upstream service/driver names still collide,
so **no coexistence with the installed official runtime**.

**Actual execution:** CMake/CTest compiled and ran 2 portable native
tests (Start C++17 policy + C11 driver identity) on each of GCC/Clang,
2/2 PASS on both. Exact patched GitHub seven-file Sandboxie source
produced 9 hunks; strict expected-blob comparison and modified-driver
negative case passed using ChatGPT V8. Five authored pure-JS fork
regression tests executed and passed. This is not an MSVC/WDK/kernel
runtime test and does not establish a signed or functioning service.

**Main next task:** verify private SbieSvc and SbieDll lifecycle accepts
only this core identity, compile real private binaries, and investigate
inter-component compatibility with REA on a disposable Windows host.
Avoid ad-hoc driver installation or modification of account-bearing
Client 02. Functional code takes priority over further proof-only layers.

## Primary milestone: actual MIVUBI Sandboxie fork (2026-10-14)

**Priority contract:** The product objective is a **working, independently
built, Sandboxie-derived native isolation system owned by MIVUBI**. No more
source-checker/proof bureaucracy as a substitute for implementing the native
engine. Three Minecraft boxes, REA and desktop UI are consumers/developer
tools, not separate competing products.

**Current implementation batch:** the private upstream `Start.exe` parser
now consumes one shared, independently compilable C++17 admission policy
(`engine/isolation-native/owned-runtime/mivubi_owned_start_policy.h`).
The fork overlay INLINES those exact bytes into the pinned seven-file
upstream patch, with one opt-in dispatch site; non-owned upstream launch
behavior remains intact. The duplicated hand-authored JS array of C++
parser rules is eliminated. The final approved overlay has **seven source
files and eight Git hunks**. The standalone native test contains **20
positive/negative cases** and is registered under the canonical repo's
`isolation-native:test-owned-start` CMake/CTest command.

**Executed native evidence:** C++17 policy compiled with **GCC** and
**Clang** in this ChatGPT execution container under strict warnings, and
both CMake/CTest executable runs PASSED. The earlier handwritten harness
for the identical 3228-byte policy had 17+ edge cases, while the committed
native test source expands this to 20. Real full-upstream patch generation
and strict expected-byte verification using exact seven GitHub upstream
v1.18.5 source files passed in ChatGPT V8, with one additional modified
Start source rejected. The private native Start.exe, service, DLL and
driver remain **NOT_COMPILED / NOT_EXECUTED** in Windows/MSVC/WDK.

**Proof boundary:** Native C++ parser acceptance is not a full Sandboxie
core build, kernel minifilter signing, safe coexistence with official
Sandboxie or four-account Minecraft stability. Service `SbieSvc`, driver
`SbieDrv`, IPC and minifilter altitude are unchanged in the current
exclusive upstream-derived private fork. Do not attempt to install alongside
the user's active official environment.

**Priority after this batch:** compile the actual pinned upstream core
with Windows MSVC/WDK on an authorized disposable Windows host, then build
the same source with the reviewed MIVUBI patch, compare native binaries via
REA, validate DLL/service/driver ABI, and implement the necessary core
isolation changes revealed by those results. Only after a loadable,
self-contained MIVUBI core is proven should it replace the official
Sandboxie backend, and only after real two-client stability should Client
03/04 be enabled. Do not misclassify this as finished product.


## Historical initial JS-only launcher customization — superseded by native policy

**Work:** move from ABI-marker-only overlay to real pinned upstream C++
`Start.exe` command-line behavior plus deterministic MIVUBI three-box
profile/launch control. **State:** SOURCE_IMPLEMENTED with bounded tests;
native private binaries and Windows gameplay NOT_EXECUTED.

**New real behavior in the seven-file direct-child private fork:** upstream
`Sandboxie/apps/start/start.cpp` receives an opt-in
`/mivubi_client:02|03|04` parser branch and validation that routes those
three client IDs to separate `MivubiEdu02/03/04` box names. The managed
switch rejects nonexistent IDs, second switches, mismatched box selection,
non-absolute/UNC/URL targets, unexpected parameters and non-Education EXE
names. Other upstream Start CLI paths remain unchanged when not opting
into the new mode. The patch/verifier still approves EXACTLY seven source
files; Start now contains three unified hunks (ABI, CLI dispatch and target
restriction), making nine hunks total. The producer and verifier use one
canonical expected-byte transformation.

**New functional host contract:** `engine/isolation-native/owned-runtime/
minecraft-profiles.mjs` generates a three-box persistent `SbieIni set`
configuration command plan and strict per-client private `Start.exe`
launch command line. Existing box names cannot be overwritten. Its six
authored test bodies were executed in ChatGPT's JavaScript evaluator and
passed (not full pinned Node24). A disposable-Git three-hunk fixture
successfully accepted `git apply --check --index` and `git apply --index`;
the exact updated private-source patch was also checked in V8 against the
actual upstream seven-file corpus, yielding seven changed paths and nine
hunks, with extra edits rejected.

**Not claimed:** private fork compiled to a Windows Start.exe; actual
SbieIni.exe execution; signed SbieDrv/SbieSvc; safe coexistence with official
Sandboxie (NOT possible with unchanged service names); Microsoft WAM
credential isolation; or independent Minecraft gameplay. No profile,
account, official box, kernel driver or Windows installation was changed.

**Next SINGLE implementation task:** integrate an owned runtime adapter
behind a verified dedicated private-driver lab admission, *not* the active
official Sandboxie backend, then test native Start/SbieIni binaries. Do
not consider source implementation sufficient for four-client acceptance.


## Actual private-fork patch application defect fixed — 2026-10-14

**Work:** make the seven-file private source overlay actually applicable by
Git, rather than just syntactically plausible text. **State:** DONE for
source correction and disposable-Git reproduction; the genuine upstream
private checkout and native Windows build remain NOT_EXECUTED.

**Defect:** `buildPrivateSourcePatch()` emitted Git unified-diff hunks
containing the insertion anchor but no original following context line
(`@@ -line,1 +line,added+1 @@`). In a disposable seven-file Git fixture,
`git apply --check --index` rejected that form as `patch does not apply`
even though every anchor matched. This would block `--fork-root` before
MSBuild or REA could make progress.

**Fix:** each hunk retains one following original source line and updates
the header to `@@ -line,2 +line,added+2 @@`. Unexpected CRLF source is
rejected rather than producing misleading provenance. Existing disposable
Git patch-application test plus new format/negative-case assertions
cover the correction. Seven-file Git fixture application now succeeds
(`git apply --check --index`, then `git apply --index`) and the actual
candidate JS hunk output passed five focused assertions in ChatGPT V8.

**Boundaries:** this does NOT prove patch application to the *full pinned
upstream checkout* in this environment, successful MSBuild/WDK compilation,
driver load, independent Minecraft gameplay, license clearance or REA
native provider Evidence. No Windows PC, account data, kernel service or
real game process was touched.

**Next ONE task:** run the exact seven-file overlay test and subsequent
source/build stages against a verified full-history pinned upstream
checkout when eligible; continue ChatGPT-only development only for
demonstrable defects, not repeated unexecuted readiness audits.


## Status decision — 14 October 2026: ChatGPT source stage NOT completely closed

**Exact GitHub HEAD before this handoff update:** `0e486f151cef`.
The product is NOT a finished Sandboxie replacement. No private signed
driver/service, installed source-compiled runtime, REA native binary Evidence,
four-profile launch backend or four-client gameplay proof exists.

**Confirmed done in GitHub source:** pinned v1.18.5 upstream inventory,
source-fork/patch bootstrap, exact committed seven-file patch checker, Windows
build scripts, noninstallable artifact stager, offline source/ABI probe,
bounded read-only Tauri inspector, unchanged official Sandboxie Client 02
backend and regression fixtures. A source patch marker is NOT a new
isolation implementation.

**Remaining GitHub/ChatGPT-only work:** the full `npm run
test:isolation-source` and other exact-revision Node/Rust/desktop tests
remain unexecuted at the pinned toolchain. Static V8 test-body runs do
not count as those test suites. Any genuine test failures or demonstrable
source contradictions should be fixed on `Local`; do not add speculative
readiness layers. The upstream build guide was corrected to use **different
report paths** for collection-only and the subsequent build, while build
and build-plus-staging are **alternatives** on a fresh checkout.

**Cannot be completed by GitHub source review alone:** native MSVC/WDK
compilation, signing/cert trust, exclusive service/driver load and rollback,
REA disassembly/decompilation of compiled Windows PE outputs, Windows
WAM sign-in and idle-crash diagnosis, multi-hour soak, three/four-client
resource profiling and independent account acceptance. These must remain
NOT_EXECUTED until proper evidence exists.

**Acceptance to close source handoff:** source fixes are in a committed
exact `Local` HEAD; all documented commands are consistent with the current
nonclobber behavior; no critical source-only known blocker remains; and the
specific unexecuted toolchain/runtime tests are carried as an explicit
disposable-Windows lab handoff. This is **source handoff readiness**, not
product implementation completion.

**Next ONE task:** do not perform a private-driver cutover. Where an eligible
toolchain becomes available, execute pinned source suites at exact HEAD;
then build the unmodified baseline in disposable Windows, followed by the
separate committed private fork and REA comparisons. Never change
account-bearing Client 02 to force progress.


## Private build preflight syntax and path-collision repair — 2026-10-14

**Work:** remove an actual PowerShell parse blocker in the private native
builder and prevent report/stage/log path collisions before MSBuild.
**State:** DONE for source changes; Windows/WDK execution NOT_EXECUTED.

**Defect:** `engine/isolation-native/fork-overlay/build-private.ps1` had
one unmatched opening parenthesis in the `-StageOutput` source-overlap
`if` condition. The script could not enter native compilation as written.
The conditional was corrected, preserving its same three overlap checks.
Both private and upstream baseline builders now also reject report files
inside the staged payload directory and report paths matching reserved
MSBuild stage logs. This avoids expensive compilation before a predictable
staging failure. No installation, signing, service, box, account or world
mutation was added.

**Evidence:** 2/2 existing authored source regression test bodies executed
against the modified exact GitHub PowerShell candidate text and PASSED.
Both modified PowerShell files additionally passed a quote/comment-aware
balanced-delimiter scan, including parentheses, brackets and braces.
This custom lexical scan is *not* a PowerShell AST parser; actual PowerShell,
MSVC/WDK, complete Node24/Rust and Minecraft runtime remain NOT_EXECUTED.
The regression explicitly refuses the old missing-parenthesis condition.

**Next ONE task:** only address further demonstrable source blockers;
full parser, build and REA native-binary proof belongs in an authorized
disposable Windows lab after ChatGPT-first source work is complete.


## Latest source batch — protect Windows build evidence (2026-10-14)

**Done in GitHub source:** `build-baseline.ps1` and `build-private.ps1`
now reject pre-existing report/log/stage destinations *before MSBuild*;
the private build additionally rejects stale optional PDB symbols, matching
the upstream baseline's required+optional artifact preflight. Both keep
existing noninstalling behavior, use no-clobber writes, and refuse
source/output overlap. The repo's existing Node source tests contain
new build safety assertions.

**Executed evidence:** two authored JavaScript build-script contract tests
were run against the updated exact candidate files in ChatGPT V8 and
passed. Full pinned Node24 test runner, PowerShell parser, Rust, CMake,
MSBuild, WDK, REA native and Minecraft tests remain NOT_EXECUTED.
No private binaries were compiled and no active service or accounts
changed. No claim is made about cross-process atomic filesystem locks.

**Next:** only substantive remaining source blockers should prompt a new
source patch. Otherwise stop expanding static gates and proceed to
separately authorized disposable Windows native build/proof.


**Authority:** GitHub `MIVUBI-STD/M-Minecraft-Emulator`, branch `Local`. Never
change `main` or the active Windows user environment to satisfy source tests.
This is the current work queue, not a second implementation database.
Full older handoff (~130 KB) remains permanently available at
[the previous Local revision](https://github.com/MIVUBI-STD/M-Minecraft-Emulator/blob/dc695bf9670cd07a8d5b7a9b51a8dc6b80fb7999/planning/development.md);
commit history contains each bounded intervention and actual evidence status.

## Goal and current behavior

- Target: **four simultaneously playable Minecraft Education Windows desktop
  clients, four distinct licensed accounts, one host**. Preferred eventual
  layout is Client 01 native plus Client 02–04 in separately isolated profiles.
  This target has **NOT** passed four-client acceptance.
- The only **active** isolation backend is an independently installed,
  official Sandboxie Plus 1.18.5 using persistent `MinecraftClient02`.
  VMware/guest paths are historical and inactive; user accounts, worlds,
  saved VMs and Sandboxie service/driver are not to be migrated, deleted or
  modified without a separate approved Windows acceptance.
- Real operator evidence on 14 October 2026: two distinct logged-in accounts,
  concurrent gameplay and multiplayer initially worked, then Client 02 crashed
  while idle. Crash root cause UNKNOWN; stable two-client soak and reboot
  persistence NOT_EXECUTED. Refer to
  [first-success/crash observation](../docs/validation/two-client-first-success-2026-10-14.md).
- The app-owned Sandboxie-derived engine is **source/build candidate only**.
  No private production driver, signed service, installed native core,
  proven ABI, REA binary analysis or owned runtime cutover exists. Its initial
  fork source merely inserts seven ABI/build guards; it does **not** implement
  new isolation semantics, improve performance, or allow co-installation.
  The existing Sandboxie names `SbieSvc`/`SbieDrv` remain exclusive.

## Current source ownership

| Area | Canonical source | Guaranteed scope |
| --- | --- | --- |
| Exact pinned upstream and all native binary/resource/license paths | `tooling/isolation-source/upstream-baseline.json` and `validate-baseline.mjs` | v1.18.5 complete closure (16 binaries, 9 optional PDBs, 5 resources, 3 notices) |
| Upstream checkout and Windows baseline | `verify-checkout.mjs`, `build-baseline.ps1` | provenance/read-only build, no install |
| Private fork source | `engine/isolation-native/fork-overlay/apply-private-source.mjs` | exact seven-file patch, clean direct-child commit, SHA/tree/patch digest |
| Private Windows build and staging | `engine/isolation-native/fork-overlay/build-private.ps1`, `tooling/isolation-source/assemble-private-engine.mjs`, `assemble-engine.mjs` | noninstallable payload, no driver or account changes |
| Native ABI probe | `engine/isolation-native/CMakeLists.txt`, `src/abi_probe.cpp` | source ABI, lab-only probe, no service install |
| Candidate read-only inspection | `apps/desktop/src-tauri/src/engine/owned_isolation.rs` | exact source/path/digest and folder snapshot, ALWAYS not installable |
| Active Client 02 launch | `apps/desktop/src-tauri/src/engine/native_clients.rs` | official Sandboxie only, fresh box admission, mutex, post-helper retry gap |
| UI/IPC | `apps/desktop/src/app/bridge/nativeFeasibilityApi.ts`, `CompactManagerApp.svelte` | candidate diagnostics, never engine activation |
| REA workflow | `docs/validation/rea-source-audit-2026-10-13.md` | developer-side methodology, NOT_EXECUTED on native binaries |

The root [GitHub rules](../GITHUB_RULES.md), architecture and tests own their
specific behavior. Avoid second managers, generic CI, duplicate names,
unnecessary abstraction layers and parallel documents containing the same
authoritative state.

## Last substantive source work

1. Compiled-source provenance closes the bug of reporting the original
   upstream HEAD while private edits remained staged. The exact clean
   private Git commit/tree and patch SHA-256 propagate through build report,
   payload and the Rust read-only inspector. CMake also calls the same
   committed-source verifier.
2. Windows private build report no longer references the removed PowerShell
   `$expected`; it checks outputs, license/resource existence, logs, source
   provenance before/after build, and refuses overwriting previous evidence.
3. Candidate scanner enforces 24 mandatory payload files, optional pinned PDB
   mapping, manifest provenance, strict safety marker, bounded metadata read,
   closed folder inventory, and Windows reparse-point rejection. These are
   snapshot integrity checks, NOT authenticity, signing or installation.
4. Active Client 02 launch guard uses a 30-second in-process anti-duplicate
   interval starting **after** the Start.exe helper finishes or errors. It
   still requires a fresh sandbox membership check. External process races
   remain unproven.
5. **Most recent completed source closure:** `validate-baseline.mjs` is being changed to reject
   reduced/reordered/injected artifact, optional-symbol, resource and license
   lists; the private assembler and private MSBuild script now share that
   complete inventory validation. New negative-case tests use the actual
   pinned complete manifest, not a misleading five-file fixture.

## Evidence ledger — do not inflate results

- **Remote GitHub:** source reviewed, versioned commits on `Local`, original
  upstream source files and solutions checked through GitHub.
- **Source-level V8:** specific pure/structural scenarios ran in ChatGPT's
  JS execution environment, including the canonical manifest mutations.
  This does **not** establish full Node test-runner success.
- **Full pinned Node 24.21.0 + npm/Vitest, Rust 1.98.1, CMake/VS2022, WDK,
  Authenticode signing, kernel load, REA native provider, WAM/crash tracing,
  Minecraft four-account acceptance:** **NOT_EXECUTED** here.
- Terminal toolchain in ChatGPT currently has Node **22.16.0**, lacks Rust/Cargo
  and cannot DNS-resolve github.com; GitHub connector remains available to
  inspect and commit text source. Do not weaken toolchain pins to manufacture
  a success status.
- REA is **not a shipped runtime dependency**. Later inspect signed/native
  binaries in a disposable lab, record hashes/provider Evidence, and compare
  to pinned source. See [REA campaign](../docs/validation/rea-source-audit-2026-10-13.md).
- Modified upstream source may be subject to GPL and special Sandboxie Plus
  distribution conditions. Public commercial/rebranded redistribution needs
  an actual licensing review; source checkout is not distribution approval.

## Next ONE executable task

**At the first genuinely eligible execution context**, run the complete
exact-HEAD Node/Rust source tests and pinned *unmodified* Sandboxie MSBuild
baseline in a separate disposable Windows lab. Then compile the reviewed,
committed private fork and run REA offline binary comparisons and ABI checks.
Do NOT install any private `SbieDrv`/`SbieSvc` beside the account-bearing
official Sandboxie installation.

Until that execution context exists, focus ChatGPT work only on demonstrable
source defects with direct implementations and negative tests. Do not
repeatedly grow this handoff or call source verification real gameplay.
