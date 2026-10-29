# Current Development Handoff — LazyTester

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
