---
id: document.validation.efficient-local-testing
class: DOCUMENT
domain: validation
role: PROCEDURE
authority: WORKING
lifecycle: ACTIVE
---

# Efficient progressive local testing — Lazy Tester

**Status: PLAN ONLY / NOT_EXECUTED.** Scope: native Windows x64, branch `Local`,
MC01–MC05 Minecraft Education profiles. This runbook is a practical subset,
not a replacement for the existing 198 standard + 108 experimental case inventory.
No GitHub Actions, auto sign-in, world mutations, AI input or installs are authorized
by a source plan. Use an authorized dummy world and licensed accounts.

## Selection model

| Profile | Intent | Source work proposed | Manual acceptance |
| --- | --- | --- | --- |
| `fast` (default) | Subsequent code iteration | Existing path-based source check from working tree or most recent commit; use `--base-sha` for a wider batch | None inferred |
| `local-smoke` | First baseline regardless of last commit | `source.quick` after one-time bootstrap and locked dependency setup | LT-01–LT-13 below (MC01 then MC02) |
| `five-client` | Expand proven baseline | Reuse same-SHA source evidence; new source work only if changed | FC-01–FC-04; distinct five licensed accounts |
| `release` | Installer candidate | Explicitly gated `source.preflight`, then `build.candidate` | RL-01–RL-03, disposable install/repair/uninstall |
| `experimental` | AI/MCP/Skills Probe | No standard action proposed: approved separate ABI, feature and source evidence needed | EX-01–EX-04, only if explicitly authorized |

For `local-smoke`, the JSON plan includes **all 13 actions, objective
pass conditions, expected evidence families, original case cross-references
and per-case fail-fast flags**. It is self-contained for an operator or
local AI reviewer, but it never evaluates, executes or auto-records a manual
game step. No screenshot is taken automatically.

These are **read-only plans**. The controller's `plan` never runs source
checks, starts the manager, launches Minecraft, stages installers or enables AI.
Do not use `DEV.cmd run auto` for first-run acceptance: on a clean checkout
its changed-source basis can be a documentation-only latest commit.

## Day-one instructions (operator-controlled)

1. Preserve `App/`, `Data/`, profiles/worlds and installed game; use a
   disposable Windows user where practical, with a tested backup plan before
   any game-account mutation. Do not run `git clean`, reset, force-kill or
   destructive Storage operations. Record Windows build, local `Local` HEAD
   SHA and a clean checkout.
2. Read-only: `DEV.cmd localcheck`. On a fresh clone it may be `BLOCKED`
   until toolchain and reviewed native DLLs are present. Do not synthesize
   native DLLs, bypass SHA-256 admission, or reuse experimental AI DLLs.
3. **With separate operator approval**, run `DEV.cmd bootstrap` once
   if portable tools are missing, `DEV.cmd doctor`, then `DEV.cmd setup`
   once if locked npm dependencies are not prepared. These download/write
   ignored build inputs; they are not read-only inspections.
4. After portable Node exists, run `DEV.cmd plan --profile local-smoke --json`
   to inspect the recommended source screen and the LT-01–LT-13 IDs.
5. **With approval for source execution**, run `DEV.cmd check:quick` from a
   clean checkout; it performs repository layout/lock checks, repository
   fixtures, frontend typecheck/Vitest and Rust compile checks. It does
   **not** run all Rust tests or full `preflight`. A nonzero status is a
   first-failure blocker, not a reason to weaken tests. The command writes
   only a private source-quick receipt under `workspace/runs`, with
   `quickSourceScreen`, elapsed milliseconds and first failing stage;
   `sourceVerification`, installer and gameplay remain `NOT_EXECUTED`.
6. Configure `LAZY_TESTER_NATIVE_PAYLOAD_DIR` to the reviewed standard
   two-DLL directory. `DEV.cmd localcheck` must recognize the pinned
   hashes. Only then, **with approval**, `DEV.cmd dev` starts the manager.
   It is not a login/gameplay acceptance result.
7. Optionally create an **operator-only checkpoint report** (this does
   not launch Minecraft or mark cases PASS):
   
   ```powershell
   $sha = (git rev-parse HEAD).Trim()
   .\DEV.cmd smoke start --expect-sha $sha --json
   ```
   
   Copy the returned `runId`. After personally completing and confirming
   each case, record one result, such as:
   
   ```powershell
   .\DEV.cmd smoke record --run <runId> --case LT-01 --result PASS --observed --expect-sha $sha --json
   .\DEV.cmd smoke status --run <runId> --json
   ```
   
   Use `FAIL`, `BLOCKED` or `INCONCLUSIVE` without `--observed`
   when the checkpoint did not pass. The first such result terminates
   further PASS reporting for that run; create a new run after addressing
   the blocker. The report includes next scenario details, outcome counts
   and a strong self-report trust label. No account/world text or screenshots
   are stored by these commands, and 13 reported PASS cases are **not**
   automatically certified Minecraft acceptance.
8. Perform LT-01–LT-13 manually, recording each checkpoint once. Stop
   escalation on P0 data/identity/ownership failures. Do not launch MC03–05
   before both MC01 and MC02 are genuinely playable together.

## First-run acceptance — thirteen meaningful checks

| ID | Scenario and source-case cross-reference | Objective pass condition |
| --- | --- | --- |
| LT-01 | Checkout, Windows, reviewed DLLs (`STD-SRC-01..05`, `STD-SRC-13`) | Exact SHA, x64, safe private data baseline and approved DLL admission |
| LT-02 | Fast source screening (`source.quick`; not a full `STD-SRC-11` preflight) | Actual exit 0 on this SHA; no game claim |
| LT-03 | Manager opens (`STD-UI-01`) | One real rendered Tauri window, not just an executable process |
| LT-04 | Five fixed slots, no automatic game/login (`STD-UI-02..04`) | MC01–MC05 once, status grounded and all games remain stopped |
| LT-05 | Refresh and manager reopen without game (`STD-UI-05`, `STD-UI-11`) | No duplicate controller, phantom running status or profile rewrite |
| LT-06 | Manual MC01 game (`STD-GAME-01..03`) | Licensed sign-in; dummy world rendered; movement/camera/interact work |
| LT-07 | Save, close, relaunch MC01 (`STD-GAME-04..05`) | Saved world/profile intact, no forced kill |
| LT-08 | MC02 added while MC01 alive (`STD-GAME-06..08`, `STD-GAME-12`) | Distinct licensed user and PID/birth/HWND/slot; both worlds playable |
| LT-09 | Switch/focus/selected window (`STD-WIN-01`, `STD-WIN-03`, `STD-WIN-05`) | Correct original process selected; no cross-client input or unrequested relaunch |
| LT-10 | CloseOne MC02 (`STD-LIFE-01..04`) | Normal exit of only MC02; MC01 remains interactive |
| LT-11 | Reopen manager with owned game active (`STD-LIFE-12..13`) | Correct original controller/process retained or safely reported UNKNOWN, no fake stopped state |
| LT-12 | Read-only storage and persistence (`STD-DATA-01..03`) | Five private profile inventories are not mixed/reset; inspect only metadata |
| LT-13 | Manual coordinates and honest F3 (`STD-SET-12..13`, `STD-SET-15..16`) | XYZ negatives/fractions format correctly; unsupported live copy/rotation remains unavailable |

**Priority distinction:** an existing P0 case denotes serious failure if
triggered; it does not imply that all 213 P0 catalog cases should run in
every smoke. No smoke result proves absence of untested P0 failures.

## Escalation only after the previous gate passes

| Gate | Targeted next checks | Stop condition |
| --- | --- | --- |
| FC-01 | Launch MC03–MC05 one by one (`STD-GAME-09..12`) | Any slot cannot become independently playable |
| FC-02 | Five selected windows, DWM previews and identity (`STD-WIN-01..03`) | Wrong HWND/PID, accidental input to another slot |
| FC-03 | Normal CloseOne while other four remain (`STD-LIFE-05`) | Peer closes, unclear ownership or data mutation |
| FC-04 | Same-world multiplayer and measured resource sample (`STD-PERF-01..03`, `STD-PERF-07..09`) | No real overlap/gameplay, crash or unstable workload; do not invent FPS/RAM |
| RL-01 | `source.preflight` separately approved | Full exact-SHA source checks not actually passed |
| RL-02 | `build.candidate` with separate package approval | Missing reviewed DLL, corrupt/stale manifest/hash |
| RL-03 | Real installer/repair/uninstall on disposable environment | Any private profile/world loss, foreign process effect |
| EX-01 | Synthetic MCP/visual/observed-action fixtures (`MCP`, `VIS`, `OBS`) | Incorrect trust, identity, source PNG admission |
| EX-02 | Reviewed opt-in AI DLL/feature/ABI separate from standard | Missing reviewed experimental pair = BLOCKED |
| EX-03 | Human MC01 + actual AI MC02 input, camera, neutral (`AI`) | An IPC ACK or PNG without game action is INCONCLUSIVE |
| EX-04 | Four/five AI and original Skills Probe/map (`AI`, `QA`) | Requires EX-03 live PASS and separate original Probe evidence |

Installer mutations, Full Clear, cache reset, fault injection, multiplayer
and experimental AI are **not** part of the initial 13-case smoke.
Full regression still uses
[the existing comprehensive master](lazy-tester-test-master.md).

## Efficiency evidence baseline — deferred, no new collector

**Source-only specification; NOT_EXECUTED.** Record one exact `Local` commit,
tool versions, Windows/Education versions, selected game slots, capture
provider, sample duration and identical gameplay workload. These are
measurement fields, **not claimed results or success thresholds**.
Use the current `DEV.cmd`, MCP read-only observation, private run receipts
and existing acceptance/PresentMon tools when a compatible Windows/Codex
environment and separate operator permission are available. No new agent,
telemetry daemon, CI, screenshot recorder or installer is needed.

| Dimension | Existing observation/measurement route | What to compare |
| --- | --- | --- |
| Source iteration | Existing `source.quick` receipt and registered `benchmark.frontend`/`benchmark.rust` tasks | Measured wall time and first failed stage for the exact same toolchain and SHA, not inferred compile gains |
| Visual payload | One requested CUA/WGC PNG and existing `inspectPng` size, dimensions and SHA-256; MCP `read_visual_observation` only upon demand | Source PNG bytes, accepted/refused oversize, frame deliveries and their total bounded base64 bytes; never claim token counts from bytes |
| Frame freshness and contention | Existing observer `frameAgeMs`, `successfulSamples`, `repeatedPngBytes`, `duplicateGpuReads`, `skippedBusy` and `captureErrors` | Same 30-second window: sample age, useful frames delivered, busy skips and stale/invalid frames per slot |
| Native actions | Existing bounded MCP/native transport receipts and per-client neutral/release/reconciliation statuses | Aborted/uncertain/rejected transactions and actual acknowledged neutral release; an ACK alone does not prove a Minecraft action |
| Windows CPU/RAM/GPU/FPS | Existing manual native performance acceptance, sanitized per-process Windows resource samples, and PresentMon report where available | Real per-process CPU, private bytes/working set, GPU load and frame-time percentile; use `UNKNOWN` for any unmeasured dimension |

**Comparison method (future, only after explicit approval):** keep lossless
capture settings and target workload identical, record a baseline and a
candidate by their separate exact SHAs, sample the existing default
30-second visual observation period, and use independent runs instead of
reusing stale screenshots. Start with the smallest proven client scenario;
do not escalate to five concurrent licensed clients until the existing
two-client, ownership and experimental authorization gates pass.
The existing 400–5000 ms visual interval limits and 3.5-second freshness
refusal remain safety boundaries, not performance targets to relax.

**Evidence and stopping rules:** store only sanitized numeric aggregates
and existing bounded receipts under ignored private `workspace/` when
authorized. Never commit raw PNG, image base64, Microsoft credentials,
world files, account names, process-identity raw values or private logs.
`PASS` requires actual execution in the matching context. A source-only
change can eliminate a known redundant hash or align size contracts, but
cannot claim lower CPU, latency, memory usage, token cost or higher FPS.
A candidate that weakens native identity/lease/neutral guarantees is
rejected regardless of performance. If no measurable problem is shown,
**make no new optimization or abstraction**.

## Evidence policy — short, honest, reusable

Record one session report privately under ignored `workspace/`, using the
existing run outputs when available. Do not publish raw screenshots,
email/account identifiers, world files, tokens or registry hives.

| Field | Evidence |
| --- | --- |
| Revision | Full 40-character `Local` SHA and Windows / Education versions |
| Source receipt | Private `workspace/runs/*-quick-*/manifest.json`; only `quickSourceScreen` can be PASS |
| Operator smoke record | Private `workspace/runs/*-smoke-*/manifest.json`; case statuses are **self-reported**, never independent gameplay proof |
| Stage | Source screen / manager / MC01 / MC02 / five-client / package / experimental |
| Case | LT/FC/RL/EX ID, actual action, expected/observed and PASS/FAIL/BLOCKED/INCONCLUSIVE/NOT_EXECUTED |
| Failure | First wrong source owner, reproducible steps and safe sanitized log excerpt |
| Identity | Alias MC01–MC05 and sanitized original PID/birth/HWND/session bindings where relevant |
| Measurement | Optional real timing/FPS/RAM with method/window; UNKNOWN when unmeasured |
| Visual | One sanitized screenshot per significant checkpoint or failure, always tied to observed slot/process |

Do not attach a screenshot to every click. A single verified observation
can cover several catalog IDs; synthetic fixtures cannot certify game
pixels or input. **Fail fast** on wrong slot/foreign process, unverified
DLL, corrupted private data or false stopped status. If only a minor UI
cosmetic issue occurs, mark it and continue unaffected independent cases.
Quick screening is deliberately narrower than release verification and
must never be reported as `SOURCE_EXECUTION` full regression PASS.
