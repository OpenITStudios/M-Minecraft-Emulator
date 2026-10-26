---
id: document.system.native-efficiency-implementation-playbook
class: DOCUMENT
domain: system
role: REPORT
authority: WORKING
lifecycle: ACTIVE
---

# Minecraft Education native efficiency — implementation playbook for Codex

**Status (2026-10-25):** REVIEW / DESIGN ONLY. Concrete optimization approaches, not instructions to deploy or change running Minecraft. This playbook is a supporting engineering reference; `planning/development.md` alone determines the current active goal, blocker, and next action. Inspect a fresh `Local` HEAD and owner before implementing anything.

**Reviewed source baseline:** `Local` at `99fee8e4ebe859958cf0517f7dfb65a89a4db6d6`. Later commits override source descriptions here. Original live two-client measurements and their limits are in `planning/development.md` and [the research report](native-memory-efficiency-research.md); neither proves any proposed optimization.

## 1. Product target and what "saving resources" actually means

- Five separately licensed Minecraft Education Windows desktop clients, one independently assigned Host World and four joining it; retain normal account/world persistence and multiplayer.
- Foreground render ceiling 144 FPS; background ceiling 10 FPS; foreground assignment MUST NOT change the Education host identity. A ceiling is not an achieved-rate guarantee.
- Hardware target recorded in planning: i7-14700 (no integrated GPU), 32 GiB-class system RAM, RTX 3070 with 8 GiB dedicated VRAM, 512 GB SSD.
- The operator's **A/B/C priority** at the reviewed revision: **A** durable approved Simple preset, 12-chunk-minimum render distance and no simulation-distance reduction; **B** real, supported internal-game efficiency; **C** reduce *inactive* client work without host/network/saves regression. The later operator decisions recorded in planning supersede older suggestions for "settings unchanged" across *all* tracks. **A is an approved visual-quality tradeoff; B/C gains must not be misrepresented as setting changes.**
- Objective is **less real work or fewer retained allocations**, not cosmetically lower Task Manager Working Set. Do not treat paging, freezing a host, stopping helpers, or swapping textures as equivalent savings.
- Current two-client private-commit levels are roughly 2.49/2.68 GiB and CLI manager roughly 2.32 MiB (first 15-minute baseline). The manager is not the obvious multi-GiB cause. The later minimized-client observation did not establish RAM saving. No benefit can be promised numerically or extrapolated to five clients.

### Implementation separation (do not conflate)

| Surface | We can actually change | Candidate technique | Scope boundary |
| --- | --- | --- | --- |
| Our Windows native compatibility DLL | Yes, C++ | cheaper path routing, avoid repeated registry reopens, disable expired diagnostic work, bounded native lifetime | preserve original process isolation and fail-closed handling |
| Our Rust bootstrap/manager | Yes, Rust | event-driven lifetime waiting, no duplicate observers, reuse existing capability ownership | no new launcher, daemon, auth manager or additional public admission |
| Our DXGI presentation adapter | Yes, C++ at the already-owned Present boundary | bounded pacing waits, avoid unnecessary HUD composition and redraws | cannot silently alter game draw loop, D3D allocation lifecycle or tick ownership |
| Official Education settings | Only with operator-approved normal settings | supported Simple profile, supported world options | these are A, not an internal optimization |
| Authored world/add-ons | When the actual world has authorized editable content and compatible API | event-driven scripts, filtered entity queries, geometry/texture/pack hygiene | content/world-specific; not automatic optimization of stock Education |
| Minecraft's renderer/mesher/heap | No verified owned integration seam | Sodium/FerriteCore-style chunk meshing, culling, batching, vertex compression, mesh deduplication | **B: NOT IMPLEMENTABLE TODAY from Present alone**; requires a proven Education-native owner, lifetime, rebuild/invalidation and safe integration contract |

## 2. Tool and API choice: choose a technique, not a second product

| Task class | Preferred existing tool or API | Why / what not to add |
| --- | --- | --- |
| Native interception already required for isolation | Existing Microsoft Detours integration; C++20/MSVC in `Development/tooling/acceptance/windows/bootstrap-ordering/compat.cpp` | Modify one actual owning hook, not a second injected DLL or global file-system filter |
| Native file identity and safe traversal | `GetFileInformationByHandleEx(FileIdInfo, FileAttributeTagInfo, FileCaseSensitiveInfo)`, retained directory `HANDLE`, `NtOpenFile` relative `RootDirectory` and `OBJ_DONT_REPARSE` | avoid name-only or stale-HANDLE caches; do not bypass reparse/access checks |
| Native registry | `NtQueryKey(KeyNameInformation)`, `NtQueryObject(ObjectBasicInformation)`, `NtOpenKey`, `RegOpenCurrentUser` within existing `compat.cpp` | keep caller access rights, impersonation detection, backing-hive protection and close semantics |
| Focus | Existing Rust `SetWinEventHook(EVENT_SYSTEM_FOREGROUND)` + PID/birth-bound policy leases | no second polling governor or process priority auto-tuner |
| FPS cap | Existing `frame-pacing.h` waitable-timer/event pacer; `dxgi-pacing.cpp` wrapper | preserve native `Present/Present1` behavior; do not fake occlusion, drop arbitrary draw calls or throttle game-wide threads |
| Background timer candidate | `SetWaitableTimerEx` with documented `TolerableDelay`; use existing focus-change/stop/controller events | conditional experimental mode, not a global `timeBeginPeriod` / priority change |
| Compositor / HUD | Existing `fps-hud.cpp` (GDI `CreateDIBSection` / `UpdateLayeredWindow`); examine Win32 layered-window behavior and DXGI flip-mode docs | avoid replacing the game's swap chain or injecting game-owned shader state blindly |
| Immutable data reuse | Windows image-backed memory sharing and read-only `CreateFileMapping` only for OUR proven immutable data | Windows may already share identical executable/DLL image pages; never share mutable Roaming/HKCU/world/token/cache |
| Mod-like internal renderer research | Original Sodium/FerriteCore/Lithium source as *algorithm references*, not binaries or Java mixins | no Java mod loader, Fabric/Forge injection, or "drop-in Sodium for Education" claim |
| Authored-world optimization | Minecraft Creator resource-pack structure, Creator performance practices, official Education feature support | verify Education build support first; never edit an account-bearing live world from this playbook |
| Verification after a genuine change | Already-existing MSVC Release CMake controls, narrow Rust/Node tests, then normal two-client functional acceptance when allowed | no CI (explicitly disabled), no additional telemetry service, no recurring tests for a documentation-only change |

**SDK reality:** the installed Minecraft Education build is not a Java runtime and does not expose a documented Sodium/Fabric-style renderer plugin ABI. A Windows API export or the existence of `ID3D11Device`/`IDXGISwapChain` does not grant ownership of Minecraft mesh allocation and reclamation.

## 3. Prioritized implementation options (not a second active task queue)

**Status codes:** `CANDIDATE` = source seam exists but benefit unproved; `RESEARCH_GATE` = prerequisite absent; `ALREADY_PARTIAL` = specific code improvement exists; `REJECT` = do not implement as proposed. Risk is *implementation/regression risk*, NOT likelihood of an asserted saving.

| ID | Candidate mechanism | Target | Owner | Feasibility / risk | Proposed decision |
| --- | --- | --- | --- | --- | --- |
| N01 | Registry-handle provenance fast path: avoid repeated `NtQueryKey → route → NtQueryObject → NtOpenKey → close` on a known-safe handle | CPU, kernel transitions | `compat.cpp` value hooks | CANDIDATE / high correctness risk | Design a proof-preserving eligibility fast path; do NOT cache bare handle integers |
| N02 | Reduce redundant filesystem traversal only where one operation already pins/validates ancestors | CPU, I/O | `compat.cpp` `fileRoute`/`privateParent` | CANDIDATE / medium-high risk | Preserve per-operation file identity and reparse safety |
| N03 | Stop bounded startup tracing clock and record work after the 30-second trace actually expires | CPU | `startup-observation.h` plus `compat.cpp` callers | CANDIDATE / low-medium risk | One atomic expired state; keep functional hooks and lifetime storage counters |
| N04 | Reuse existing Rust process handles and switch manager exit waiting from periodic polling to event-led waits | CPU, wakeups | `independent_bootstrap.rs` | CANDIDATE / medium risk | Use bounded Win32 process wait; keep window-caption refresh and journal save behavior |
| R01 | Background timer tolerance / avoid unnecessary precision for 10-FPS mode | wakeups/CPU/power | `frame-pacing.h` | CANDIDATE / medium risk | Retain fast foreground promotion; don't change renderer's simulation ownership |
| R02 | HUD redraw avoidance / temporary graphics-resource allocation lifetime | GDI CPU, compositor | `fps-hud.cpp` | ALREADY_PARTIAL / low-medium | Direct-to-upload DIB already done; consider event-paced redraw without a new service |
| R03 | Inline GPU HUD as alternative to separate layered window | DWM/GPU compositing | `dxgi-pacing.cpp`/`fps-hud.cpp` | RESEARCH_GATE / high | Requires safe D3D11 & D3D12 state/lifetime seam, otherwise keep existing HUD |
| C01 | Inactive client no-work pathways that maintain host simulation | GPU, CPU | game's own supported behavior + existing focus/pacing | RESEARCH_GATE / high | Minimize experiment did not establish savings; do not auto-minimize or fake occlusion |
| M01 | Reuse immutable image/data, avoid duplicate proprietary asset copies | physical RAM/cache/disk | `independent_bootstrap.rs`/existing storage owners | ALREADY_PARTIAL / medium | Both official clients already launch same pinned image; add sharing only for proven duplicate *owned* data |
| M02 | "FerriteCore for our own data": value-interning, compact metadata, deterministic buffer ownership | memory/alloc churn | existing owned native/Rust structs | CANDIDATE / low-to-medium | Apply only to repeated large owned structures; do not introduce a global cache |
| B01 | Compact chunk vertices, meshing/culling, batching, GPU arena reuse, mesh invalidation | large renderer CPU/RAM/VRAM | internal Education renderer | RESEARCH_GATE / highest | Need real game-compatible internal API/source/lifetime; cannot implement from Present wrapper |
| B02 | Canonical model/quad/block-state deduplication | game RAM | internal Education block/model owner | RESEARCH_GATE / highest | Need immutable-data guarantee and ownership; unverified for Education |
| W01 | Optimize permitted authored resource packs/scripts instead of the engine | world CPU, texture VRAM | actual pack/script project, NOT this launcher | CONDITIONAL / low-medium | Useful as separate content intervention after checking build support; doesn't satisfy B |
| X01 | Periodic `EmptyWorkingSet`, `Trim`, `OfferResources`/`ReclaimResources`, arbitrary page/CPU caps, whole-process suspend | "RAM number" only or unstable | none | REJECT | Risk of paging, resource loss, host stalls; no valid owner/rebuild contract |

**Ordering rule:** user asks for actual savings, not a new observer. Prefer meaningful in-place N01/N02 if their safety invariant can be preserved; N03/R01/R02/N04 are bounded CPU-overhead repairs; B01/B02 are **blocked until an Education-native owner exists**, not TODOs to solve with fake abstractions. C01 is still a primary outcome, but its game-internal leverage is not proven. No claimed percentage or memory reduction here.

## 4. N01 — Registry fast path: exact currently redundant work and a safe shape

**Code:** `Development/tooling/acceptance/windows/bootstrap-ordering/compat.cpp`, `keyName`, `registryRoute`, `openPrivateValue`, `translatedQueryValue`, `translatedSetValue` and `enumeratePrivate`.

The current path for each value read/write/enumeration may:
1. Query original key name via `NtQueryKey(KeyNameInformation)`.
2. Reconstruct/canonicalize private registry route.
3. Check thread impersonation and original handle's granted rights via `NtQueryObject`.
4. Reopen an equivalent private key from the retained `privateHive` using `NtOpenKey`.
5. Invoke actual query/set/enumeration then close that temporary mapped key.

The expensive operations are *plausible* overhead; they are not yet proven high-volume in this Education build. Rewriting them without guarding inherited/pre-opened handles risks exposing another profile.

**Preferred redesign, if sound:** establish native handle *provenance at creation*, use the existing translated `NtOpenKey/NtOpenKeyEx/NtCreateKey` to return an actual private-backed handle. For a provenance-qualified, already-private handle, issue the original query/set/enumeration directly; for an inherited, pre-hook, impersonating, unknown, reparse-equivalent or security-ambiguous handle, **retain the current validation and reopen path**. This is analogous to Sodium's avoidance of repeated derivation on stable owned data, but it applies only to Windows handle routing.

**Critical caveat:** a numeric HANDLE may be closed and reused for a different kernel object. A map keyed solely by HANDLE value, path string or PID is forbidden. A practical implementation would require an exact handle lifetime protocol and invalidation on `NtClose`/`NtDuplicateObject` and all other relevant acquisition/transfer paths, with ownership and concurrency verified. That is substantial scope; **do not implement a cache unless that completeness can be justified**. Start with local-invocation elimination of redundant name/rights work or an exact known-owned handle special case (e.g. retained private root when identity and rights are invariant), then stop if deeper hooks are required.

**No-go:** skip `noImpersonation`, bypass `GrantedAccess`, use handle-only cache, incorrectly send host-HKCU values to a slot, expose tokens/key names in logs, or broaden registry writes to foreign hives.

**Minimal proving cases:** existing ordinary/private keys, root/subkey, KEY_QUERY_VALUE vs KEY_SET_VALUE and enumeration rights, inherited and impersonated handles, close/reuse, concurrent calls, registry replay across normal restart, and arbitrary public keys remaining unaffected. One bounded fixture suite; no new generic harness.

## 5. N02 — Filesystem path/traversal: work inside one owned operation only

**Code:** `compat.cpp` `fileRoute`, `privateParent`, `translatedFileCreate/Open`, `translatedSetInformation`, `translatedQueryAttributes/FullAttributes`.

**Observation:** `fileRoute` may query file ID and canonical path of relative-parent HANDLEs. `privateParent` then walks every child directory using `NtOpenFile`, retains ancestor handles and checks directory identity/reparse/case properties, and the final native open uses that pinned private ancestry. These checks defend host/profile separation.

**Safe optimization outline:**
- First classify a direct absolute name by a cheap bounded prefix check. Leave non-private names to the original NT API before allocating/converting full translated path. Respect `OBJECT_ATTRIBUTES` validation.
- For a RootDirectory that is the *currently pinned* logical or private root, use already-established root `FILE_ID_INFO`. This path already exists in source; avoid "optimizing" it again under another name.
- Within ONE existing translated call, avoid copying/reformatting normalized path data twice; carry one stack-backed relative span + translated leaf from `fileRoute` to `privateParent`. Keep buffers bounded.
- Preserve the existing per-call ancestor handles and close them on *all* successes and failures. Do not replace the proven per-call traversal with a process-global directory-handle cache based on path or integer handle: a directory can be renamed, reparse targets can change, and Windows reuses handle values.
- If an additional gain truly requires cross-call reuse, define a precise retained-handle/identity, rename/invalidation and privilege model **before** proposing code; otherwise reject this candidate.

**Tooling:** existing Detours/C++ source, Win32 `GetFileInformationByHandleEx`, NT relative-root open semantics, repository's MSVC Release compatibility control. No filter driver, virtual disk, new service, or write-through cache.

**Must keep:** atomically private destination rename, root/peer denial, FILE_EXECUTE/traversal distinction, no reparse or device-path bypass, distinct Temp/Roaming/Local, protected world paths, same ERROR/status/IO_STATUS_BLOCK and LastError semantics. Reject apparent savings achieved by lowering checks.

## 6. N03 — End expired startup tracing without touching required hooks

**Code:** `startup-observation.h` `active`, `within`, `begin` and `finish`; `compat.cpp` installs optional CreateProcess/mutex observation when `active` is present.

Current startup record scope is 30 seconds; after expiry, the hooks can still enter `begin` and call QPC. The shared storage-failure counters are intentionally process-lifetime, not the same scope.

**Proposed state machine (source sketch, not a tested patch):**

~~~cpp
enum class TracePhase : unsigned { Active, Expired };
std::atomic<TracePhase> phase{TracePhase::Active};

bool begin(Record& record, Kind kind, void* caller) noexcept {
    if (!active || phase.load(std::memory_order_relaxed) == TracePhase::Expired)
        return false; // No QPC or record initialization after expiry.
    // Existing recursion guard and QPC handling...
    if (!within(*active, now)) {
        phase.store(TracePhase::Expired, std::memory_order_relaxed);
        return false;
    }
    // Existing record semantics unchanged.
}
~~~

The model must account for (a) end-of-trace record finalization that began before expiry, (b) concurrent hooks, (c) real clock failures which must NOT incorrectly be treated as elapsed, (d) memory ordering and error preservation, (e) optional diagnostic capability absent. `storageResult` must stay functional for its designed lifetime and singleton binding must still work.

**Optimization:** bounded avoidance of repeated expired checks; no hook detachment under concurrent callers. Do not claim whole-process RAM savings or significant CPU improvement without evidence.

## 7. N04 — Avoid repeated manager wait/poll scheduling

**Code:** `Development/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs`, `observe_until_exit`. The current manager updates window captions about once per second and polls per-slot `save_after_exit` with 250-ms sleeps.

Candidate: use the already-owned child PROCESS handles with Win32 `WaitForMultipleObjects` or a narrow existing Rust wait helper, with a bounded timeout only for caption refresh. The same thread must process both signaled exits, save each journal once, preserve normal exit codes and keep profile capability alive. If the existing exit/save helper cannot safely expose those handles, do **not** build a second process keeper to force the rewrite.

Sketch:
~~~text
while any child still running:
    wait for only unfinished original child handles OR next caption-refresh deadline
    on PROCESS signaled: perform original journal-saving/exit classification exactly once
    on caption deadline: re-run the existing pinned-window labeling
    on error/unexpected wait: preserve current explicit failure handling
~~~

Keep command output ordering/protocol, normal close behavior, Rust ownership, no new daemon, no TerminateProcess or account read. Potential savings are reduced wakeups, not GiB.

## 8. R01 — Idle CPU/power reduction within existing 144/10 timer

**Code:** `frame-pacing.h` `Pacer::wait_for_frame` + `dxgi-pacing.cpp` `Chain::present`; focus changes come from `instance_focus_events.rs`.

Current `Pacer` uses a high-resolution waitable timer and the already-owned event handles. The direct optimization lever is to choose precision appropriate to the *mode*, while retaining 144-FPS foreground responsiveness and 10-FPS background ceiling. Microsoft documents `SetWaitableTimerEx`'s `TolerableDelay` but does not promise a specific CPU-saving percentage.

Implementation choice:
- Keep one policy owner, one pacing worker on the rendering consumer, and the existing events (focus level, changed, stop, controller death).
- Experimental background mode MAY use non-high-resolution/coalescible timer semantics where available, or a nonzero documented tolerable delay; foreground mode retains existing precise timing. **Do not** silently swap timer handles under a waiter or lose stop/focus events.
- All timer choices must compute deadlines from monotonic QPC; missed frames are not repaid with bursts; promotion interrupts background waits. Do not use busy-spin or global `timeBeginPeriod`.
- Preserve `DXGI_PRESENT_TEST` and `DXGI_PRESENT_DO_NOT_WAIT` bypass behavior, Present/Present1 result and `GetLastError`, COM chain destruction/resize behavior.
- The pacer currently waits **immediately before native Present**. This throttles submission, not necessarily expensive earlier draw work. Reducing CPU wakeups here is NOT the same as implementing Sodium's chunk renderer.

Reject any approach that blocks the Education simulation/network thread when a background window is host, or claims 10 displayed FPS / 144 foreground FPS just because the policy contains those numbers.

## 9. R02/R03 — HUD and presentation: use real ownership and avoid extra work

**Code:** `fps-hud.cpp`, `dxgi-pacing.cpp`, `presentation-enrollment.cpp`.

**Already implemented:** the newest HUD implementation paints directly into the layered-window upload DIB, removing one intermediate vector, one second DIB, and copies per redraw (existing `perf(hud)` commit). Do not propose that same patch again.

**Additional low-risk options:**
- Keep redraw on changed displayed one-decimal FPS/DPI/size, not every frame. Current code already checks rate and size; first find an actually redundant branch rather than duplicating it.
- Existing 250-ms bound-window wake and `EVENT_OBJECT_LOCATIONCHANGE` hook coexist; assess event-led relocation only if the wake branch demonstrably duplicates work. Preserve DPI, minimize/restore, overlay visibility and hit transparency.
- Pool only a bounded **owned** temporary graphics object when retained footprint is justified; caching an entire bitmap/font per game can increase RAM. RAII cleanup and GDI handle accounting are mandatory.

**High-risk option:** replace the `WS_EX_LAYERED` FPS window with in-swap-chain draw to reduce potential DWM compositor work. There is no currently proven D3D11/D3D12 device/context ownership or render-command seam. A correct implementation must identify native graphics API, obtain the correct command context/queue, preserve render targets/pipeline state and command ordering, survive ResizeBuffers/device removed/fullscreen, and respect the game's frame lifetime. **Do not** attempt intrusive generic D3D hook injection as a presumed fix.

**Important:** Microsoft describes flip-model/Independent Flip/MPO as efficient *when the application's swap chain and HWND are eligible*. Our adapter forwards the game's present path; it must NOT arbitrarily rewrite `SwapEffect` or `CreateSwapChain` flags. A layered HUD might affect composition in some layouts, not always. No promised VRAM saving.

## 10. C01 — Background client efficiency: available vs unavailable levers

At the reviewed revision, a confirmed minimized guest returned to a two-person paused multiplayer surface, and host progress was briefly observed. The comparison was not equivalent gameplay and working set did not decline; automatic minimization is explicitly not approved.

**What our original engine can actually do without a new game API:** maintain the 144/10 presentation schedule, avoid manager/HUD scheduling overhead when inactive, and use existing normal Windows window operations only on explicit operator request. The OS/game can independently decide how occlusion affects drawing. **Do not** forge `DXGI_STATUS_OCCLUDED`, skip drawing arbitrarily, suspend background client threads, lower background host's CPU priority globally, trim its graphics device, or reduce the host simulation distance.

**What would save substantially more GPU work if supported:** avoid producing terrain/lighting/shadow meshes/commands for a frame that will never be shown, before submission. This requires a renderer-owned scheduling/culling hook or supported Education option. `IDXGISwapChain::Present` sees an already built frame: it is insufficient to cancel earlier CPU draw preparation while preserving simulation. Without such a verified seam this candidate is **blocked**, not a request to invent an FPS controller.

**Fallback separate A experiment, only with operator approval:** set supported graphics/resolution constraints via ordinary Education UI and persistent per-profile options, retaining at least 12 chunks and unchanged simulation distance as presently authorized. That changes quality/workload and is **not** equal-output B/C efficiency. Client-area resizing can reallocate the game's swap chain and visually change viewport; do not auto-toggle sizes on focus without a separately supported contract.

## 11. M01/M02 — Genuine memory reuse, not a RAM-number trick

### Image-backed sharing already available from Windows

The bootstrap already uses the *same pinned official game executable/image* for both slots. The OS can share image-backed executable/DLL pages automatically. Don't make a "binary deduplication service", don't hard-link mutable profile directories, and don't copy tokens. Separate physical asset copies with the same hash are not automatically identical mapped file objects; avoid claiming shared pages without the actual mapping contract.

### Where source-owned deduplication makes sense

FerriteCore's transferable principle is *one immutable representation for many logical references*, not a generic cache that grows forever. Candidate only for OUR repeated data:
- immutable per-bootstrap configuration and presentation constants,
- truly duplicated repeated short static strings/typed metadata,
- scratch buffers with known lifetime and bounded reuse,
- generated immutable design-time pack data (outside signed Minecraft binaries).

For a structure used by N clients, compare a single read-only backing `CreateFileMapping`/`MapViewOfFile` view with N copies **only if the bytes are actually substantial and immutable**. Validate identity and exact mapping access; no profile/auth/world content. A 31,968-byte startup journal and small HUD pixel planes cannot explain multi-GiB game allocations. A new cache may increase commit rather than reduce it.

### What cannot be externally deduplicated safely today

Minecraft's internal chunk/block palette, baked model quads, mesh vertex arrays, terrain cache, texture residency, or D3D11/D3D12 resources created on different devices. Windows offers D3D shared resource mechanisms in other contexts, but **the game must cooperate with resource creation, ownership, synchronization, updates and destroy/rebuild**. Never rewrite or silently alias its heaps by intercepting allocations.

## 12. B01/B02 — A precise Sodium / FerriteCore / Lithium translation table

This is the most important distinction for the requested native engine. The references below are **specific algorithms in Java projects**, not evidence that they can be compiled or installed for Minecraft Education.

| Original Java optimization | How it works | Required native system/tool for equivalent | Available to this repo? |
| --- | --- | --- | --- |
| Sodium compact chunk vertex | Encode terrain vertices in a compact layout (reviewed source includes a 20-byte implementation), reducing per-mesh memory traffic | Game-owned chunk mesh builder and vertex format, custom shader input signature, buffer upload/rebuild and resource release | **NO**; current DXGI adapter has no chunk/vertex data |
| Sodium mesh arena and builder reuse/compaction | Reuse bounded upload memory and free/compact mesh backing at proper unload/rebuild boundaries | Real terrain allocation owner, command queue/fence synchronization, lifetime hooks, device-loss recovery | **NO**; a generic `ID3D11Device` pointer does not establish it |
| Sodium visibility sorting/culling | Skip terrain sections known invisible before generating/submitting draws; batch compatible draws | Game chunk culling/visibility lists, camera data and render-stage commands | **NO**; post-render Present cannot perform it |
| FerriteCore canonical model/quad/voxel shape data | Intern identical immutable data, replace many maps/tables with indexed representation | Game's block-state/model allocator plus mutation contract and canonicalizer | **NO**; it would change Education internals |
| Lithium block/entity ticking | Event-driven entity tracking, filtered lookups, fewer allocations / faster collision searches | Game's simulation/tick/entity engine source or supported scripting hooks | **NO** for game internals; **CONDITIONAL** for *authored world scripts* |
| Sodium/FerriteCore engineering pattern for MIVUBI | Avoid redundant work, bound buffers, reuse immutable owned data, one owner per state | Existing C++ hooks, Rust handles and Windows file APIs | **YES** for N01–N04/R01–R02/M02 only, with much smaller possible effect |

**Feasibility gate for genuine B (before any patch):**
1. Identify the **exact** official Education version/build, supported native integration contract or genuinely authorized source/resource API; do not infer the renderer ABI from Java Minecraft or Bedrock community code.
2. Identify one actual allocation/mesh factory, its owning object, reference semantics, thread, GPU queue and wait/fence lifetime; locate creation, rebuild, visibility updates, unload, device loss and crash recovery.
3. Demonstrate correct invalidation and the immutable/mutable distinction under changing chunks/resource packs, multiplayer world transitions and five independent profiles.
4. Specify the transformation and the original fallback (e.g. compact vertices decode identically and match shader input); exclude undefined behavior and account/game-file mutation.
5. If the interface is absent, record **INTEGRATION_NOT_AVAILABLE** and stop. No generic Direct3D `CreateBuffer` interception, unbounded allocation pool, swap-chain hook or disassembler run should be relabeled as a solved internal optimizer.

No licensed game binaries may be patched, no authentication bypass or unsupported Java loader deployed. Native reverse engineering or process hooks require their own authorized scope and a real compatibility proof, and are **not** authorized by this documentation request.

## 13. W01 — An actual content-based optimizer that may work without game-engine hooks

**This is a separate content/world option, not internal B.** Minecraft Creator official documentation gives concrete algorithms that can reduce rendering/simulation resources in authored worlds; **Bedrock documentation does not prove that the installed Minecraft Education build supports every feature or script version**. Verify support against the target Education version before making a pack-specific proposal.

**Tools when content authoring is permitted:** Blockbench for geometry, a PNG/texture atlas workflow, validation of resource-pack manifest/geometry JSON, VS Code for supported Script API content, existing Minecraft world/pack test workflow. Do not edit installed game's signed packs or users' account data.

Concrete interventions:
- **Geometry budget:** remove invisible cube faces and unnecessary repeated cubes from authored models; prefer fewer cubes and reuse common shapes where appearance allows it. Official Creator guidance recommends targeting <=50 cubes per block model as a rule of thumb, often less if repeated densely. This is a content complexity guideline, not a vanilla renderer memory limit.
- **Texture budget:** reduce unnecessarily oversized textures, remove duplicate source images, atlas compatible authored textures, consolidate materials when supported. Memory grows with pixel dimensions; do not claim compressed PNG-on-disk bytes equal GPU allocation. Visual equality is only possible for actual duplicate/redundant data or lossless changes.
- **Texture handle budget:** avoid thousands of unique tiny texture files/variants. The official Creator guide describes recommendations for world templates/add-ons and device stability; a handle reduction may be more important than ZIP size.
- **Tick budget:** use documented event subscriptions and event filters instead of global every-tick scans, narrow `getEntities` queries, cache stable dimension handles, batch costly work with `system.runJob` where supported. Never reduce command/simulation semantics to "save CPU".
- **Entity/particle budget:** cap temporary entity and particle density, clean up appropriately, and remove redundant ticking areas in owned maps only after validating gameplay mechanics.

**Tools are conditional:** official Education feature/API compatibility first; no assumption that a stock Education Host World uses Script API packs. Work on duplicate copies of authorized authored content and keep original world intact. Result should be described as **content optimization** with documented visual/behavior tradeoffs, not a Sodium mod or original-engine B.

## 14. GPU/VRAM boundary: choose APIs that can safely change work

- `IDXGIAdapter3::QueryVideoMemoryInfo` exposes local/nonlocal budgets **for the calling process**. This is a memory-budget API, *not* an automatic memory reducer nor cross-process global VRAM sharing function.
- `IDXGIDevice3::Trim` is intended for genuine idle situations; background 10 FPS is not proven extended idle. Do not invoke it periodically or when switching windows without the owning game's device-idle contract.
- `IDXGIDevice2::OfferResources/ReclaimResources` can discard resources that must be reinitialized. We do not own Education textures or know reconstruction semantics. Reject.
- `ID3D11Device::CreateRasterizerState` already deduplicates equivalent state under documented device behavior; building another unproven state cache is not an efficiency solution.
- Do not rewrite a game's swap chain to `FLIP_DISCARD` or force `DXGI_SWAP_CHAIN_FLAG_FRAME_LATENCY_WAITABLE_OBJECT` after creation. Microsoft's "wait before render" pattern assumes game-loop cooperation and proper swap-chain creation flags. Our current adapter only wraps Present.
- Do not mistake `nvidia-smi` per-process WDDM missing data for zero GPU usage. Optimizing CUDA allocations does not inherently optimize Education Direct3D rendering.

## 15. Explicit rejection matrix: avoid attractive but unsafe "shortcuts"

| Proposed shortcut | Why rejected for this product |
| --- | --- |
| `EmptyWorkingSet`, periodic working-set trims, memory compression/pagefile/standby-list global hacks | relocates resident pages, can increase faults and focus-return stalls; does not delete game assets |
| Process `SuspendThread` or Windows process suspension on every background client | can stop Education host simulation, connection, saving and credentials |
| Global EcoQoS, priority or affinity masks across background game processes | foreground visibility is not host role; may starve host |
| Force compositor occlusion result or drop arbitrary D3D draws | game still builds frames or receives false state; undefined gameplay/rendering assumptions |
| Regular `Trim`, `OfferResources`, `ReclaimResources` on unowned game textures | requires verified idle/rebuild semantics; device/texture corruption risk |
| Shared writable Roaming/Local/HKCU/world/cache or auth broker | crosses identity boundaries and risks profile corruption |
| Copy FerriteCore's numeric MiB wins into Education predictions | Java/modpack heap layouts and Education native allocations differ |
| Add a second polling governor, lifecycle service, cache daemon or profiler | duplicates current owners and adds steady-state overhead |
| Turn off CI ban, run Windows installer, change `main` or publish binaries | outside operator permission and repository rules |

## 16. Minimal development/check procedure for each selected optimization

This document **is not a request to run PC tests or a second project plan**. Once the operator/Codex chooses an approved candidate, use the repository's original procedure:

1. Pin current `Local` HEAD, read `AGENTS.md` + `GITHUB_RULES.md` + `planning/development.md` + `docs/system/implementation-map.md`. Keep **A/B/C current goal** intact; do not automatically promote this list into multiple active tasks.
2. Choose **ONE** mechanism ID and find the first wrong owner/actual wasted computation. Write down source entry, existing consumers and exact invariant. Check whether current source already solved it.
3. Implement smallest coherent change within that owner. Avoid parallel abstractions. For a candidate requiring a missing game API, record an explicit NO-GO with the missing prerequisite and stop—do not fabricate source integration.
4. Verify *functionality* narrowly: native MSVC Release `/W4 /WX` and existing CMake controls for C++; focused Rust source/unit checks for manager changes; only the smallest existing tests affected. Existing `fps-hud-control` and `frame-pacing-control` cover graphics control semantics; a hidden WARP control is not Minecraft acceptance.
5. Check failure paths: handle/file ID and registry provenance; last-error/NTSTATUS preservation; world/account separate after normal restart; focus state and exit; simulation/network/host when relevant. Do not clone dozens of micro-tests.
6. Only if real game operation is approved and available, stage a **new** manifest/payload through existing manager hash owner with original candidate preserved, after normal game closure. Do not replace loaded DLLs. Smoke-check two original licensed clients before any five-client claims.
7. Record genuinely executed proof and remaining `UNKNOWN/NOT_EXECUTED`. Never claim runtime savings from source arithmetic, microfixtures, reduced bitmap bytes, or a single lower working-set reading.

### Existing source tooling only

- `Development/tooling/acceptance/windows/bootstrap-ordering/CMakeLists.txt` and its current MSVC controls: compilation and semantically relevant WARP/GDI/source behavior.
- `Development/apps/desktop/src-tauri`, `Development/engine/runtime-core`: existing Rust/Cargo source/tests where already owned.
- `Development/tooling/acceptance/windows/collect-native-performance.ps1` and `Development/tooling/acceptance/evaluate-native-performance.mjs`: **verification only if needed**, not new feature scope.
- Intel PresentMon and Microsoft VMMap/WPR/WPA: optional one-off diagnostics for a specific supported regression or unknown allocation owner; NOT a justification to build another tracking product.
- No GitHub Actions CI; not even a manual workflow. Do not operate a remote PC or game merely because this playbook was committed.

## 17. Codex handoff contract (use after operator explicitly chooses a mechanism)

~~~text
Repository: OpenITStudios/M-Minecraft-Emulator
Branch: Local only (pin fresh SHA; never main)
Goal: A/B/C as defined NOW in planning/development.md
Reference: docs/system/native-efficiency-implementation-playbook.md
Task: choose a single N01/N02/N03/N04/R01/R02/M02 mechanism only if its exact
      current source still contains an independently justified redundant operation.
Authoritative implementation: existing named owner only; preserve consumers.
No new collector/governor, no Java mod port, no speculation about Minecraft heap.
Do not weaken file, registry, auth, singleton, world, or host protections.
No CI, installer, profile mutation, game closure, public Launch or binaries.
Report: exact source diff + mechanism of fewer calls/copies/wakeups/allocations,
        relevant focused checks genuinely executed, unresolved runtime effects.
Do not claim B (internal game RAM optimization) without a real Education mesh/
resource ownership, rebuild, and invalidation seam.
Do not alter planning's single active Next unless operator authorizes that work.
~~~

## 18. Primary sources and source-to-technique map

**Original game/performance inspirations (not Education plugins):**
- [Sodium main repository](https://github.com/CaffeineMC/sodium); [pinned compact chunk vertex implementation](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/vertex/format/impl/CompactChunkVertex.java); [pinned mesh-builder lifetime](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/vertex/builder/ChunkMeshBufferBuilder.java). Ideas: compact data and explicitly bounded buffer lifetime.
- [FerriteCore technical summary at 26.1](https://github.com/malte0811/FerriteCore/blob/26.1/summary.md). Ideas: compact block-state storage, canonical immutable data, deduplicated quads. The author says memory examples are old Java modpack-specific figures: no Education transfer.
- [Lithium maintained source/config](https://github.com/CaffeineMC/lithium/blob/develop/lithium-neoforge-mixin-config.md). Ideas: fewer redundant world/entity lookups, optimized ticking/collisions, event-based work; NOT a native Education hook.
- [Minecraft Education official unsupported-mod statement](https://edusupport.minecraft.net/hc/en-us/articles/360047117692-FAQ-Game-Features) and [Education performance settings](https://edusupport.minecraft.net/hc/en-us/articles/4404976988052-Troubleshoot-Performance-Issues).

**Windows mechanisms:**
- [File Mapping](https://learn.microsoft.com/en-us/windows/win32/memory/file-mapping); [Sharing Files and Memory](https://learn.microsoft.com/en-us/windows/win32/memory/sharing-files-and-memory): process mapping semantics, NOT profile sharing.
- [GetFileInformationByHandleEx](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfileinformationbyhandleex), [GetFinalPathNameByHandleW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew), [RegOverridePredefKey](https://learn.microsoft.com/en-us/windows/win32/api/winreg/nf-winreg-regoverridepredefkey): identity and process-scoped boundaries.
- [SetWaitableTimerEx](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setwaitabletimerex), [DXGI waitable swap-chain](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_3/nf-dxgi1_3-idxgiswapchain2-getframelatencywaitableobject), [DXGI reduce latency](https://learn.microsoft.com/en-us/windows/uwp/gaming/reduce-latency-with-dxgi-1-3-swap-chains): timer and renderer-cooperative frame pacing.
- [DXGI flip-model presentation](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/for-best-performance--use-dxgi-flip-model), [DXGI video-memory budget](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgiadapter3-queryvideomemoryinfo): eligible presentation and residency semantics, not externally owned resources.
- [IDXGIDevice3::Trim](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_3/nf-dxgi1_3-idxgidevice3-trim) and [IDXGIDevice2::OfferResources](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/nf-dxgi1_2-idxgidevice2-offerresources): reasons game-owned reclamation is currently rejected.

**Official authored-content optimization (conditional on Education version support):**
- [Creator practices: improving performance and resource usage](https://learn.microsoft.com/en-us/minecraft/creator/documents/practices/improvingperformanceandresourceusage?view=minecraft-bedrock-stable): model cubes, textures, handles, pack size.
- [Creator performance in scripting](https://learn.microsoft.com/en-us/minecraft/creator/documents/scripting/script-performance?view=minecraft-bedrock-stable): filtered entity lookup, tick/event usage, bounded work.
- [Creator pack structure](https://learn.microsoft.com/en-us/minecraft/creator/documents/comprehensivepackcontents?view=minecraft-bedrock-stable): resource/behavior pack formats; no assumption they all work on Education.

**Repository evidence:** [Canonical research and current comparison](native-memory-efficiency-research.md), [Current single handoff](../../planning/development.md), [Implementation Map](implementation-map.md). All paths above are bounded source-reading recommendations, not a mandate to execute or deploy.
