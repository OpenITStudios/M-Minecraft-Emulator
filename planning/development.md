## Current product work — minimal launcher guidance and private window feedback (29 October 2026)

**State:** DONE in source; complete pinned Svelte/Vitest/Playwright and Windows/Minecraft acceptance NOT_EXECUTED.
**Decision:** Reuse the five existing launcher cards; nonintrusive hover/accessibility hints for STARTING, NOT_RESPONDING and ATTENTION use only typed, observed status. Normal cards stay unchanged. WindowTitlebar now uses existing sessionFeedback for authored window failures, preserving the old event messages and showing collapsed Support references only for real alerts. No extra buttons, page, timer, IPC, credentials/world actions, auto-retry, settings, monitor, installer or CI.
**Proof:** REMOTE_GITHUB source/consumer and content-contract checks. Focused Vitest pure-guidance/source tests and synthetic Playwright hostile-window-error and launcher accessibility regressions AUTHORED, NOT_EXECUTED.
**Unresolved:** Pinned frontend/Svelte/Vitest/Playwright; Windows window controls, installer and Minecraft Education multi-client acceptance NOT_EXECUTED.
**Next:** Execute pinned frontend compilation and synthetic browser checks in a compatible environment; fix observed failures before another user-facing feature.

---

# Current Development Handoff — LazyTester

## Current product work — typed Diagnostic & Error Contract v1 (29 October 2026)

**State:** DONE in source; pinned Svelte/Vitest/Playwright and actual Windows/Minecraft NOT_EXECUTED.
**Decision:** Extend the existing `sessionFeedback.ts` owner with operation-scoped authored category, stable support reference and message, without interpreting native exception text. General/connection alert state stores only typed keys, not raw strings; original user-facing messages remain unchanged. Move two previously inline Graphics/FPS refusals to the catalog. Display a collapsed **Support reference** only when a current alert exists; expanding shows category and reference, never file paths, process IDs, credentials or native errors. No diagnostics page, clipboard export, IPC, polling, persistent error store, new lifecycle, account/world operation or CI.
**Proof:** REMOTE_GITHUB exact-parent error-producer audit, unique catalog/code and caller/IPCs source checks. Vitest and browser hostile-error fixtures AUTHORED, NOT_EXECUTED.
**Next:** Run pinned frontend/Svelte and browser checks when compatible dependencies exist before adding any further diagnostics/reporting workflow.

---

## Current product work — Data Storage presentation extraction (29 October 2026)

**State:** DONE in source; Svelte/Vitest/Playwright and Windows/Minecraft acceptance NOT_EXECUTED.
**Decision:** Extract the original Data Storage view into DataStorage.svelte. Metadata, cache review, game settings recovery review, full-clear confirmation and location controls become passive markup. All review records, typed-name/checkbox confirmation values, selection and native mutation admissions stay in LazyTesterApp.svelte, with advanced/fullName/fullAck bound to that original state. Preserve action guards, CSS classes, copy and original IPC. No new store, monitor, migration or automatic deletion.
**Proof:** REMOTE_GITHUB pinned source and consumer inspection; exact markup/callback/IPC parity guards. Focused source test and browser storage review/cancel fixture AUTHORED, NOT_EXECUTED.
**Next:** Verify compiler and browser with genuinely compatible pinned dependencies before claiming interactive acceptance; then continue source work only where clearly justified.

---

## Current product work — General Settings presentation extraction (29 October 2026)

**State:** DONE in source; pinned Svelte/Vitest/Playwright and live Windows/Minecraft NOT_EXECUTED.
**Decision:** Extract all four existing General Settings sections to one passive GeneralSettings.svelte view. Three input bindings (graphicsPreset, activeFps, backgroundFps) propagate to single parent state. Graphics, FPS, audio, data location and navigation actions still invoke only original LazyTesterApp callbacks; preserve DOM classes, control IDs and user feedback. No state ownership duplication, native IPC, timer, credential or world mutation.
**Proof:** REMOTE_GITHUB exact-parent markup/action audit with structural and native-call parity checks; focused source ownership and browser 120/5→Discard regression authored, NOT_EXECUTED.
**Next:** Review compiler acceptance when compatible dependencies are present; then consider a single bounded Data Storage presentation extraction, retaining destructive confirmation/admission with the original owner.

---

## Current product work — UI queue scheduling policy extraction (29 October 2026)

**State:** DONE in source; exact-head Svelte compiler and Minecraft/Windows acceptance NOT_EXECUTED.
**Decision:** Extract only the pure first-eligible queue decision into the existing UI scheduling owner \`inspectionScheduling.ts\` and replace eight inline reactive action blocks with one Svelte dispatcher. Preserve the original audio → FPS → graphics → navigation → explicit Refresh → deferred Launch → selection → layout precedence, with the legacy exception that initial selection/layout may run during preparation. All queue payloads, state mutation, native IPC, preview ResizeObserver, process identity and close review stay in \`LazyTesterApp.svelte\` and the original native owners. No worker, background monitor, new scheduler timer or state database.
**Proof:** REMOTE_GITHUB exact-HEAD owner/consumer and IPC/polling parity checks PASS. **SOURCE_EXECUTION (isolated selector only):** Git blob `00db53a041c31d48c93b2fc61b56d8ac6618d4b1` was copied byte-for-byte into a Linux scratch test (verified by `git hash-object`), strictly typechecked with TypeScript 5.8.3 (`tsc --strict --noEmit`, PASS), transpiled, and exercised under Node 22.16.0 via `node --test`: **2/2 test cases PASS**, including all 2,304 original queue/admission combinations and focused startup/close-review boundaries. This does not use the pinned Node 24 toolchain or compile Svelte. Repository Vitest/source-routing cases AUTHORED, NOT_EXECUTED; Svelte/Vite/Playwright/Windows/Minecraft remain NOT_EXECUTED.
**Next:** Examine the current frontend compiler/test evidence when available; next source-only modular work can isolate General Settings presentation without moving lifecycle/state.

---

## Current product work — launcher rail presentation extraction (29 October 2026)

**State:** DONE in source; Svelte compile/browser/Windows gameplay NOT_EXECUTED.
**Decision:** Extract the five-card rail to `LauncherRail.svelte` as a presentation-only child. Maintain exactly one state, manual lifecycle owner, native preview ResizeObserver, data admission and recovery click gate in `LazyTesterApp.svelte` and existing bridges. Keep HTML classes, accessible names and global CSS selectors unchanged, with callback delegation and no new IPC/timers. Do not alter game process/window behavior.
**Proof:** REMOTE_GITHUB parent UI, CSS and consumer audit plus bounded static preservation checks; focused source ownership test authored, NOT_EXECUTED.
**Next:** Review source compiler evidence and plan a second small settings/pure scheduling extraction only where it preserves existing reactive order.

---

## Current product work — canonical versus legacy contract cleanup (29 October 2026)

**State:** DONE in source; integrated Windows/package/Minecraft acceptance NOT_EXECUTED.
**Decision:** Match the current native schema 2 data layout (`Data/Launchers/Minecraft-1..5` and `Data/Settings/data-location.json`) in manifest/test/checker, while preserving explicit legacy schema 1/flat-install paths (`Data/Profiles/` and `App/data-location.json`). Keep installer NSIS migration/refusal and data untouched. Rewrite the outdated desktop README, and correct historical FPS default documentation.
**Proof:** REMOTE_GITHUB pinned current Rust owner, existing NSIS hooks and package consumer structural review; tests authored, not executed. No installed app, private data, lifecycle or game operation changed.
**Next:** Phase B — one bounded UI presentation/scheduling extraction without adding a new state owner, worker or IPC.

---

## Current product work — Guided Recovery integration / isolated source verification (29 October 2026)

**State:** DONE for bounded source hardening and isolated module execution; full desktop, browser and Minecraft runtime acceptance NOT_EXECUTED.
**Decision:** Reuse one pure Guided Recovery gate for button state and click-time validation, including queued selection/launch checks. The frontend blocks active/uncertain process states on the card and deferred launch handler, while allowing manual ATTENTION retries for already-stopped games; the original controller alone decides actual process/storage admission. Existing Refresh, native lifecycle, IPC, polling, worlds and accounts are unchanged. No further application change was justified by this isolated verification.
**Proof — SOURCE_EXECUTION (isolated, not repo-wide):** ChatGPT Linux sandbox Node.js 22.16.0 / TypeScript 5.8.3. Exact checked-in Git blobs for `sessionHealth.ts` (`668ccead97c26eb5994711ed63ba555c5883dbe9`), `sessionFeedback.ts` (`213812be6e01b83586e049580b2e998c0b42d1bb`) and `guidedRecovery.ts` (`44ff67122e7efe2a9e902713648008ed31740b05`) were staged with a test-only typed bridge stub; strict TypeScript check/transpilation PASS and 29/29 Node built-in cross-module behavior tests PASS. The isolated stub reproduces the current `canRequestLaunch` predicate but is not the complete bridge, so neither the full `lazyTester.ts` nor Svelte was compiled. Tests cover per-slot status priority, explicit recovery action admission, no startup/close mutation, independent recovery warnings and authored-message privacy. REMOTE_GITHUB: 17/17 exact-HEAD static owner/consumer/privacy/scheduling checks PASS; existing Vitest and Playwright cases are AUTHORED, NOT_EXECUTED.
**Unresolved:** Repository pins Node 24.22.0 / npm 12.2.0 plus Svelte/Vitest/Playwright dependencies, not present in this isolated environment (network access unavailable). No exact-HEAD full Svelte check, Vite build, Vitest, Playwright, Tauri Windows compilation/package, live Minecraft Education 26.32 or five-client acceptance was executed. Passing the isolated tests does not imply these outcomes.
**Next:** Run the complete pinned-toolchain frontend and browser checks only in a genuinely compatible execution environment, then fix observed failures without adding another runtime authority or CI.

---

## Current product work — Guided Recovery v0.1 (29 October 2026)

**State:** DONE in source; compiled frontend, Windows and Minecraft runtime acceptance NOT_EXECUTED.
**Decision:** Give each contextual health warning one explicit safe action: **Check status** uses the original manual Refresh, while **Open game** uses the existing verified Play selection only for a responding background game with a failed preview. Recheck live UI status when clicked; the original native controller independently verifies game/window identity. No restart, force-close, auto-retry, new IPC, monitoring, account/world mutation or secondary lifecycle authority. Native restoration uncertainty retains its separate warning without a misleading fix button.
**Proof:** REMOTE_GITHUB exact-source review and V8 static caller/projection checks. Existing Vitest source cases and synthetic browser cases extended for Refresh/Present-only paths and disabled controls during close review; AUTHORED, NOT_EXECUTED.
**Next:** Evaluate compiled frontend and browser interactions in a compatible environment, then consider only user-evidenced remaining recovery gaps.

---

## Current product work — contextual Session Health MVP (29 October 2026)

**State:** DONE in source; manual frontend, Windows and Minecraft runtime acceptance NOT_EXECUTED.
**Decision:** Project one concise authored issue per affected Minecraft profile from the existing five-profile snapshot and DWM preview report. Show nothing extra for healthy/stopped profiles. Give priority to unavailable or unresponsive windows and uncertain startup; preview failure never means gameplay failure. The existing independent native window-recovery banner remains authoritative and is not duplicated on cards. No new IPC, polling, gameplay/launcher control, profiling or automatic recovery.
**Proof:** REMOTE_GITHUB pinned-code review and static UI/bridge/owner checks; pure projection unit fixtures and synthetic browser scenarios AUTHORED, NOT_EXECUTED. No app installer, real Minecraft window or five-client acceptance was tested.
**Next:** Review a lightweight guided-recovery experience using only already admitted manual actions, without adding new lifecycle authority or prompt-like technical detail.

---

## Current product work — session feedback and per-slot recovery (29 October 2026)

**State:** DONE in source; compiled desktop, Windows and Minecraft runtime acceptance NOT_EXECUTED.
**Decision:** Keep original five-profile native controller and DWM owners unchanged. Render only authored user-facing messages for native errors; status says "Profiles verified", not "Connected" from an idle Inspect. Track unresolved window restoration separately by slot in UI; only that slot's positive selected-close presentation receipt clears its warning. Refresh/STOPPED must not clear native recovery uncertainty. No new worker, monitor, IPC, account/world change, background retry, or CI.
**Proof:** REMOTE_GITHUB exact-source consistency and static privacy/recovery invariants reviewed. TypeScript/Vitest and browser regression scenarios authored for public copy, accurate indicator, two recovery slots, explicit per-slot resolution and raw backend error suppression; execution on Node/Windows/browser/Minecraft NOT_EXECUTED.
**Next:** Source-level Session Health projection should reuse the same typed snapshot and authored user feedback without adding a second session authority. Require real Windows/gameplay evidence for stronger status claims.

---

## Closed RAM/resource-efficiency stage (retained)

**Decision date:** 29 October 2026  
**Work:** Temporary Minecraft Education RAM/resource efficiency  
**State:** **DONE — source and documentation scope only; runtime gains NOT_VERIFIED**  
**Active next action:** **None for this scope.** The user has closed this
development stage and will focus on other work. A new explicit request is
required to reopen it.

This is the **single current decision/handoff owner**. The long historical
handoff is preserved at [the pre-close Git revision](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5b59c5d693c151428cd15424c3daee30fb63e631/planning/development.md);
[RAM research](../docs/system/native-memory-efficiency-research.md) and the
[technical playbook](../docs/system/native-efficiency-implementation-playbook.md)
are **reference evidence**, not parallel task queues. Actual current source and
`AGENTS.md`/`GITHUB_RULES.md` take precedence over old notes.

## 1. Final operational decision

Retain the **existing native Windows LazyTester controller for up to five
independently managed Minecraft Education profiles**. Preserve distinct
accounts, private Roaming/Local/HKCU storage, worlds, launcher admission,
normal close/recovery, host/network/save activity and the installed game.

| Area | Retained behavior | Current proof limit |
| --- | --- | --- |
| Foreground/background FPS | First-run defaults **144/10**. Saved `performance.json` limits win at **next launch**; e.g. saved **120/10** means 120 FPS when the game gains foreground focus and 10 FPS when backgrounded. Focus transitions select the original saved active ceiling, **not** a permanent 10 FPS clamp. | Source-wired; actual current Windows/game transitions not accepted in this closure. Caps are not guaranteed achieved FPS. |
| Graphics | Unconfigured profiles default to **Preserve Visuals**; previously saved Low/High preferences remain intact. Original 12-chunk minimum and unchanged simulation distance/target resolution are retained. | Source-implemented; no quantified memory reduction. |
| Optional game settings | Explicit and reversible stopped-profile controls for existing `gfx_texture_streaming` and `ctrl_improvedInputResponse` keys use the existing backup/admission/restore path. Never create guessed option keys or toggle automatically. | Education 26.32 feature support and RAM/VRAM effect remain unknown. |
| Particle Tuning v0.3 | Independent opt-in **offline builder** for 20 vetted ambient effects. Relative emitter distance tiers: **75% near (0–16 blocks), 50% at 32, 25% at 48, 0% at 64+**, with interpolation. Three steady effects tune spawn rate and active cap; 17 manual effects tune cap **only**. | Synthetic/source checks only; not installed; actual emission reduction for manual effects and Education compatibility not verified. |
| Texture Lite v0.1 | Independent opt-in **offline block-only** builder; pinned Mojang source, static 16×16 albedo -> 8×8 with alpha preserved; animated blocks, entities, models/UV, PBR, item/UI and gameplay stay vanilla. Ghast extension and v0.2 were removed in source commit `5b59c5d693c151428cd15424c3daee30fb63e631`. | Prior exact block source passed 5 synthetic tests; complete Mojang-derived pack, Education import and RAM benefit not executed. |
| Profile/pack inspection | Existing bounded read-only inventory of stopped-profile pack folders and saved-world declarations; reports partial/unknown rather than false zero when not complete. | This does **not** prove packs are loaded or save RAM. |
| RAM review | Existing read-only per-profile/manager/system collector and offline schema-7/8 evaluation of private commit, private working set, and synchronized selected-root comparisons. | Evidence tools only; no new background monitor, scheduler, collector, or optimizer. |

**Role policy:** two controlled clients with vanilla visuals and three mostly
idle clients are a **proposed example, not hardcoded slot identities**.
Texture Lite is **not** installed, forced on a host world, toggled by focus,
or automatically assigned to MC03–MC05. Client-local activation/isolation
in Minecraft Education 26.32 remains **NOT_VERIFIED**. A world-level pack
may affect joining clients.

## 2. Historical measurements — not savings forecasts

All figures here come from **earlier revisions/workloads**, not a current
Education 26.32 five-client acceptance. Do not combine different snapshots,
CPU private memory, GPU-local VRAM, and nominal committed capacity as
independent savings.

| Observation | Recorded result | What it establishes |
| --- | --- | --- |
| Menu snapshot — process private working set | **1,030.367 MiB** total: WriteCombined **381.488**, Heap **296.758**, image-private **286.441**, other private data **56.988 MiB**, plus remaining categories. | Classification of resident private pages, **not** proof of their exact game owner or reclaimable waste. |
| Stationary-world historical baseline | Median private working set **1,393.730 MiB** per observed guest. Target **under 1,024 MiB** remains aspirational; arithmetic difference **369.730 MiB**. | One historical workload, not five-client current usage or a safe cap. |
| UploadBufferHeap experimental retention | Retained capacity **168 -> 88 MiB**; short sequential comparison private-WS medians **1,393.730 -> 1,333.777 MiB** (**59.953 MiB** lower). | One unreplicated comparison with unequal warm-up; the **80 MiB capacity** delta is not automatically physical RAM saved. Candidate stays **OFF**. |
| Minimized guest experiment | Total working set ~**1,411 -> 1,412.87 MiB** under nonidentical observations. | No meaningful memory saving demonstrated; no auto-minimization policy. |
| Block albedo 16×16 -> 8×8 | **75% fewer source texels** per selected image; ~**0.88 MiB hypothetical RGBA8** across 1,200 images if independently loaded. | Not measured Windows RAM or VRAM. Block atlasing/mipmaps/residency prevent a savings inference. |
| Ghast entity experiment (retired) | ~**0.117 MiB hypothetical RGBA8** per client for five selected images. | Too small to justify entity/PBR risk; no active entity optimizer. |

**Largest unresolved historical class:** menu WriteCombined residency.
The **220.488 MiB mixed-owner** subset and **144 MiB synchronous-growth**
correlation have no proved live/free ownership because of a roughly 65-second
trace-to-snapshot gap. A ~67 MiB renderer-related CPU allocation likewise
has no safe resize/release contract. None are reclaimable-byte estimates.
Details and source provenance:
[Native Memory Efficiency Research](../docs/system/native-memory-efficiency-research.md).

## 3. Final choices: retained, off, deferred

| Decision | Disposition and reason |
| --- | --- |
| Keep ordinary FPS, graphics, profile guards and reversible manual settings | **RETAINED**; these are the currently owned, bounded options and do not imply real RAM savings. |
| Keep Particle Tuning v0.3 and Block Texture Lite v0.1 | **RETAINED as offline previews only**; do not bundle, activate or publish any derived Mojang pack by default. |
| Ghast/entity downscaling, model/UV/PBR rewriting, full third-party `Memory Reduce [Ram].mcpack` | **OFF / NOT ADOPTED**; infrequent target entities, very small hypothetical gain and/or unacceptable unknown visual/compatibility effects. |
| Native Upload/DefaultBufferHeap canaries, arbitrary driver trim, working-set clearing, process suspension, renderer replacement | **OFF**; real allocation ownership, GPU fences, valid release/rebuild paths and durable benefit have not been established. |
| Extra telemetry, monitoring worker, startup hooks, extra IPC, new test-service, auto resource-pack installer and CI | **NOT ADDED**; no parallel runtime authority or ongoing background work. |

**Scope guard:** Neither the host, original world, resource packs, saved
settings, Microsoft accounts nor five running Minecraft instances were changed
by this source-only finalization. Legacy VMware/Sandboxie code is retired from
the active product. `main` stays release-reserved; GitHub Actions remain
disabled by operator decision.

## 4. Future research ordering — dormant unless explicitly reopened

This is an **evidence-gated decision order, not a scheduled TODO list**.

| Rank | Question to resolve before code | Missing evidence / go-no-go |
| --- | --- | --- |
| 1 | Is there actual sustained memory growth on the exact Education build? | An observed regression/workload and relevant saved option state; historical Improved Input Response reports are not Education 26.32 proof. No automatic setting change. |
| 2 | Which GPU/CPU graphics allocations are truly unused and safely releasable? | Compatible exact game binary, synchronous ETW/VMMap/VRAM and private residency, per-resource owner, all fences/consumers and native release/rebuild. Previously observed WriteCombined and Upload candidates remain **OFF** without this. |
| 3 | Is block-heavy world memory dominated by terrain meshes/chunk/cache representations? | Actual per-component resident bytes and Education renderer/mesher ownership; Java Sodium/FerriteCore techniques are **comparators**, not supported Education plugins or a patch route. Preserve render distance at least 12 and simulation/host activity. |
| 4 | Does a specific optional pack/configuration work on one disposable Education client? | Edition-matched import, pack precedence, isolated client activation, visible quality and comparable private-RAM/VRAM/FPS results **before** any rollout. Texture packs never automatically propagate to the original hosted world. |

If a future operator explicitly reopens RAM work, **start from the top
rank that has new material evidence**, pin new `Local` HEAD and exact
Education executable revision, use the already-owned source/collectors,
then compare like-for-like workloads. No extrapolation from menu snapshots,
one short sequential run, compressed pack bytes, or fivefold multiplication
of one client result. Real acceptance would also need session, multiplayer,
world-save and normal-exit continuity. These gates are **deferred**, not
conditions for closing the current documentation stage.

## 5. Ownership, evidence ceiling and stop

- Runtime/session/profile authority:
  `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap/`
  and `Development/apps/desktop/src-tauri/src/engine/instance_focus_events.rs`.
  FPS values: `engine/manager_settings.rs`; native pacing:
  `Development/tooling/acceptance/windows/bootstrap-ordering/frame-pacing.h`.
- Offline preview owners:
  [block texture builder](../Development/tooling/lazytester-textures/README.md)
  and [particle builder](../Development/tooling/lazytester-particles/README.md).
- Existing observation-only owner:
  `Development/tooling/acceptance/windows/collect-native-performance.ps1`,
  `native-profile-baseline.ps1`, and
  `Development/tooling/acceptance/evaluate-native-performance.mjs`.
- Historical evidence:
  [research report](../docs/system/native-memory-efficiency-research.md),
  [implementation reference](../docs/system/native-efficiency-implementation-playbook.md),
  and [prior complete handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5b59c5d693c151428cd15424c3daee30fb63e631/planning/development.md).
- **Proof for this closure:** REMOTE_GITHUB source/document review and
  previously executed isolated synthetic builder checks on their exact earlier
  source blobs. **NOT_EXECUTED in this documentation-only closure:** new
  Windows/Rust/C++ build, installer, Minecraft Education gameplay/pack import,
  five-client acceptance, FPS/RAM/VRAM measurement and local-PC testing.
  P0/MC02 normal-exit acceptance stays deferred, not marked PASS.

**STOP:** The temporary resource/RAM-efficiency source scope is closed.
Do not reopen or add optimizer work, automatic schedules, monitoring, CI,
installers or local-PC testing without a new explicit user request. Other
LazyTester features are outside this closure and may be addressed separately.
