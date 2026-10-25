---
id: document.system.native-memory-efficiency-research
class: DOCUMENT
domain: system
role: REPORT
authority: WORKING
lifecycle: ACTIVE
---

# Native client memory and rendering efficiency research

Status: RESEARCH WITH BOUNDED IMPLEMENTATION AND RUNTIME OBSERVATIONS. Initial research was pinned to `5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5`; subsequent diagnostics, HUD work and the approved Education graphics intervention are identified below. A substantive Minecraft-internal optimization seam and repeatable whole-game gains remain UNKNOWN. Current operator intent is owned by planning/development.md.

## Decision context

The operator reports two original Minecraft Education clients working on an i7-14700, 32 GB RAM, RTX 3070 with 8 GB VRAM, and a 512 GB SSD, with no usable iGPU. The intended next stage is one Education client hosting a world and four guest clients joining that world. Foreground rendering should have a ceiling of 144 FPS; background rendering should have a ceiling of 10 FPS, while host simulation/networking continue and switching stays quick. These are requirements. Prior operator-confirmed two-client behavior is partial evidence, not five-client acceptance or a controlled efficiency benchmark.

The two screenshot readings, 1797.3 MB and 1724.2 MB, are observations at one moment. Their column semantics and process-group coverage are not established. They do not demonstrate a leak, private commitment, unique physical RAM, or a predictable per-client increment. Neither a savings estimate nor a capacity guarantee follows from them.

The earlier primary direction was original-system efficiency at unchanged game settings. The latest operator correction prioritizes Minecraft itself and permits reduced graphics/effects, with render distance at least 12, simulation distance unchanged, and resolution/144-10 policy preserved. Education's documented settings remain a separate optional tradeoff, not the primary engine optimization. Keep independent accounts/profiles and original game behavior. Treat external memory trimming, suspension and hard caps as poor default candidates because they can move cost into faults, stalls or failed allocations. Every resource effect remains a hypothesis until measured.

## Attribution rule: settings savings versus original-system efficiency

Operator clarification on 2026-10-25: distinguish changes to game settings
from improvements in the system we build. Engine-code gains still require equal game settings, visual output, 144/10
ceilings and host role. The later operator-approved graphics track is a separate
Minecraft workload intervention; its gains must not be credited to engine code. A lower render distance/resolution is
a separate quality/workload tradeoff and must never be reported as an
engine-code optimization.

| Layer | Example | How to credit it |
| --- | --- | --- |
| Education settings | Render distance, visual effects, supported resolution | Settings-only result, with the visual/workload change named. |
| Our original system | Owned buffer lifetime/reuse, event scheduling, duplicated observation, mediation overhead and existing pacing | System result only with equivalent settings/policy/workload and measured before/after evidence. |
| Education's internal renderer/data representations | Chunk meshing/batching, game object deduplication, replacing world/model structures | A separate compatibility research question; our current isolation/presentation wrapper does not expose these internals. No proven implementation seam or savings exists here. |

Sodium is a rendering-engine optimization mod, rather than merely a graphics
preset. Its official project emphasizes rendering performance and reduced
micro-stutter, and uses Java mod-loader integration. It can inform the
principle of doing less redundant work and managing renderer resources well;
it does not establish an Education-compatible plugin or an achievable RAM
percentage for our wrapper. [Official Sodium project](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/README.md).

Concrete Sodium examples reviewed at
`148151cf25c56f8b670986b348dcc589278c867e` show that memory engineering is
part of that inspiration:

| Mechanism in Sodium | Transferable principle | Boundary in our project |
| --- | --- | --- |
| [20-byte compact chunk vertex representation](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/vertex/format/impl/CompactChunkVertex.java) | Store required data in a smaller representation. | Applies directly only to data we own; we do not own Education's mesh format. |
| [Mesh-builder reset and destruction](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/vertex/builder/ChunkMeshBufferBuilder.java) | Bound temporary capacity and release resources at the correct lifetime. | Audit our HUD/config/diagnostic lifetimes; do not free game-owned buffers. |
| [NativeBuffer allocation accounting and explicit free](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/util/NativeBuffer.java) | Measure owned allocations and make retention visible. | Prefer aggregate own-resource counters; do not inspect game heaps or transplant Java GC behavior. |

These source mechanisms are examples, not a claim about a particular Java
modpack's savings or a native Education RAM floor. No Sodium implementation
is copied or ported into this repository.

For a specifically memory-focused Java comparison, FerriteCore documents
compact block-state maps and deduplication of model/cache data. Those changes
act on internal Java game representations, not on an external launcher's
working-set counter. Its historical modpack-specific numbers do not transfer
to native Education. This distinction does not imply Sodium has no memory
optimizations. [FerriteCore's own technical summary](https://github.com/malte0811/FerriteCore/blob/26.1/summary.md).

Transfer principles only to state our system demonstrably owns: bounded
allocation, avoiding repeated work/copies, reusable drawing resources,
event-driven observation and exact resource lifetimes. Common immutable image
mapping remains separate from private profile/account/world state. Replacing
Education's internal renderer or deduplicating its game heaps requires a
separately verified mechanism and compatibility scope; do not infer it from
the existence of our DLL.

For future measurements define E0=current engine, E1=a concrete proposed engine
change, Q0=current graphics, Q1=an agreed reduced graphics preset. Compare
E0/Q0 versus E1/Q0 for the system effect; E0/Q0 versus E0/Q1 for settings;
measure E1/Q1 separately for the combination. Do not assume the gains add.
A first bounded E1 source change now removes duplicate HUD raster/upload storage; its whole-game benefit is not measured. Keep foreground/background policy identical
when attributing a new engine change; the already-existing limiter is not a
new optimization saving. Start with composition/overhead measurements at Q0,
then investigate one source-owned bottleneck. Settings remain a separately
reported optional track.

## 1. Settings supported by Education

Minecraft Education's official performance article recommends turning off Fancy Leaves, Render Clouds, Beautiful Skies, Smooth Lighting and Fancy Graphics, and decreasing Render Distance to 6. These are the strongest documented starting controls for this edition. The article supplies no measured RAM, VRAM or CPU savings. [Minecraft Education performance guidance](https://edusupport.minecraft.net/hc/en-us/articles/4404976988052-Troubleshoot-Performance-Issues).

Render distance is a promising RAM/VRAM hypothesis because less visible world content may require fewer loaded render assets and meshes; its actual residency behavior in the installed Education version is unknown. Disabling decorative effects is primarily a rendering-work hypothesis. Lower GPU activity need not lower committed memory: an engine may retain reusable allocations. Use a stable, modest preset first; do not promise that the memory reading falls immediately or proportionally.

An Education support response distinguishes device-level render distance from world-level simulation distance, and explains that simulation distance affects how far away an agent/mob acts. That supports discussing simulation distance, but this is an older support response, not a current versioned settings contract. Changing it can change gameplay behavior and should not be part of an automatic focus policy. [Education support explanation](https://edusupport.minecraft.net/hc/en-us/community/posts/8792477830292-In-game-Builder-blocks-nerfed).

**Unverified controls:** this research does not establish current Education support for Java heap flags, Java mod settings, Bedrock-specific texture/render modes, an external live graphics API, or automatic per-focus render-distance changes. An options-file key or another edition's UI is insufficient proof. No supported Education mechanism was verified for lowering render distance when a window loses focus and restoring it when focus returns. Such a feature requires a separate compatibility investigation before implementation.

Reducing actual render resolution is another candidate for GPU work/render-target memory, but a smaller outer window does not itself prove that Education uses a smaller render target. Verify the installed edition's supported UI and actual result later; do not silently edit per-profile files. Frequent resizing or rebuilding graphics resources could work against quick switching. A single usable resolution is a safer research starting point than focus-driven resizing. DXGI requires releasing outstanding direct/indirect back-buffer references before resizing; external resizing is not a free memory-release operation. [ResizeBuffers contract](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiswapchain-resizebuffers).

## 2. Rendering limits and host continuity

144/10 are ceilings, not guaranteed rates. A frame limiter can avoid producing unnecessary frames only where its interception point and the game loop permit. It does not directly release world allocations, duplicate textures, or authentication state. A 10 FPS background guest still has simulation, network, input and helper work. A background host has an additional obligation to keep the shared world progressing.

NVIDIA documents a driver frame-rate limiter and a separate background application control. Its documented background slider range is 20–200 FPS, so that documented control does not establish a supported 10 FPS solution. Driver configuration also needs concrete executable matching and compatibility evidence; it is not an account-aware orchestration mechanism. No NVIDIA settings were changed. [NVIDIA 3D settings reference](https://www.nvidia.com/content/Control-Panel-Help/vLatest/en-us/mergedProjects/nv3d/Manage_3D_Settings_%28reference%29.htm).

The repository's limiter must be assessed at its actual source seam. Blocking presentation can be harmless to an independent simulation thread, or delay shared game-loop work; without Education thread/runtime evidence neither is guaranteed. The key later acceptance condition is that a background host continues world progression and guests remain connected while focus changes. Lower CPU/GPU usage alone cannot pass that condition.

## 3. Read the right RAM numbers

| Quantity | Meaning for this investigation |
| --- | --- |
| Working set | Pageable virtual pages currently resident in RAM; includes pages potentially shared with other processes. |
| Private working set | Resident pages private to that process; useful for avoiding shared-page double counting, but excludes nonresident private memory. |
| Private commit | Private memory the OS has committed for the process, whether currently resident or not. It is not identical to RAM in use. |
| Shared image/file-backed pages | Physical pages that multiple process mappings can reference; summing process working sets can count them repeatedly. |
| Page faults | Need classification: a soft fault can resolve from RAM; a hard fault requires backing-store I/O. |

Microsoft defines residency and the difference between soft and hard faults; removed working-set pages can remain cached in RAM until reused. Consequently, a lower process working set need not mean equally more free physical RAM or lower allocation demand. [Working Set](https://learn.microsoft.com/en-us/windows/win32/memory/working-set).

`PROCESS_MEMORY_COUNTERS_EX.PrivateUsage` reports process private commit, while `WorkingSetSize` reports the current working set. Despite its name, `PagefileUsage` is also a commit-charge field; it is not a measurement of bytes presently stored in the paging file. [Process memory counters](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex).

For later authorized measurement, compare the same workload over time using both process-level and system-level totals. Record game processes and attributable helpers separately, plus available RAM, system commit/limit, hard-fault-related reads and switch latency. Page Faults/sec alone mixes fault types and cannot establish disk paging; Microsoft recommends correlating page-read counters. [Microsoft page-fault explanation](https://techcommunity.microsoft.com/blog/askperf/the-basics-of-page-faults/373120).

A useful reduction lowers workload or retained private demand while retaining acceptable frame pacing and host behavior. A screenshot improvement immediately after trimming is insufficient. A leak hypothesis requires repeatable growth under a bounded repeated workload, accounting for startup, world loading and caches; it cannot be inferred from the two supplied numbers.

## 4. Why trims, caps and suspension rank poorly

| Mechanism | What it actually controls | Compatibility assessment |
| --- | --- | --- |
| `EmptyWorkingSet` | Removes as many resident pages as possible from a process working set. | Does not instruct Education to discard private allocations. Future accesses can fault; repeated trimming may exchange apparent RAM savings for switch stalls. Reject as routine policy. |
| Working-set maximum | Limits/targets residency, with optional hard enforcement. | Does not cap private commit. Too-small residency can force recurring faults. Reject arbitrary per-client quotas. |
| Job process/job memory limit | Limits virtual memory commitment. | Can prevent further commitment; cannot make Education render or allocate more efficiently. Reject as an optimization default. |
| Job CPU hard cap | Stops job threads for the rest of an interval after quota exhaustion. | Can delay simulation, networking, authentication and rendering together. Reject for the host; high-risk for guests. |
| Thread/process suspension | Stops execution rather than just rendering. | Incompatible with the requirement for continuing host work and guest connectivity; thread suspension also has synchronization hazards. |

These distinctions follow the API contracts: [EmptyWorkingSet](https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-emptyworkingset), [working-set limits](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-setprocessworkingsetsizeex), [job commitment limits](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_extended_limit_information), [job CPU rate control](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_cpu_rate_control_information), and [SuspendThread](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-suspendthread).

The risk assessment is an inference from these contracts and this product's continuity requirements, not a claim that a particular Education version has already failed. Do not substitute periodic standby-list purges, page-file disabling or globally altered memory policies for a retained-allocation investigation. No system changes are warranted by the supplied screenshot.

## 5. EcoQoS and memory priority

Windows EcoQoS expresses an efficiency preference for scheduling/frequency selection; it is not a RAM reclamation API or a guaranteed frame-rate limit. Windows already classifies window-owning work partly by visibility/focus. An explicit process-wide policy would cover time-sensitive game work as well as incidental background work. On this machine, assigning the host to an efficiency policy purely because its window is behind another window is a poor default. [Windows Quality of Service](https://learn.microsoft.com/en-us/windows/win32/procthread/quality-of-service).

Lower memory priority makes pages earlier candidates for working-set trimming under pressure. Microsoft positions it for background work or data unlikely to be used again soon. A playable client expected to switch back rapidly has a different access pattern; deliberately making its pages easier to evict could undermine that goal. [SetProcessInformation](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setprocessinformation).

Both are optional later experiments only if a measured bottleneck justifies them, an exact game-process identity is available, host work is protected, and original state can be restored. Neither merits implementation before a baseline. No CPU affinity masks, priority settings, EcoQoS flags or registry settings were changed for this research.

## 6. CEF and other helpers

CEF is intentionally multiprocess: browser, renderer and GPU-related processes can have distinct work and communicate through IPC. A helper is therefore not redundant merely because its name repeats or its visible window is closed. CEF's own lifecycle requires coordinated browser closure and shutdown. [CEF architecture and lifetime](https://github.com/chromiumembedded/cef/blob/master/docs/general_usage.md).

That generic architecture does **not** prove which helpers Education uses in this installation, which process owns sign-in, or whether a helper can safely terminate after login. Do not kill, suspend, merge or relaunch authentication helpers as a memory shortcut. A closed authentication window is not sufficient evidence that tokens, refresh activity or future dialogs no longer depend on its processes.

An actionable source-only opportunity is to identify processes the emulator itself creates, whether it keeps duplicate watchers/UI instances alive, and whether its own close paths release owned resources. Any future cleanup must be confined to proven emulator ownership and ordinary lifecycle handling. The private CLI route may bypass Tauri UI startup; do not budget hypothetical WebView overhead without tracing the actual entry point. CEF documentation supplies no supported external API for the emulator to shut down a third-party Education browser safely.

## 7. GPU and VRAM accounting

WDDM manages GPU scheduling and video memory. Task Manager GPU reporting draws on the OS scheduler and memory manager. Dedicated GPU memory and shared system memory are distinct categories; shared GPU memory consumes system RAM, not extra physical VRAM on the RTX 3070. Process GPU memory can include cross-process shared resources. Adding per-process values can therefore double count; use adapter-wide consumption for total pressure and process values for attribution. GPU utilization is engine-sensitive and does not express VRAM occupancy. [Microsoft GPU accounting explanation](https://devblogs.microsoft.com/directx/gpus-in-the-task-manager/).

NVIDIA explicitly documents that per-process GPU-memory usage is unavailable under Windows WDDM because the Windows kernel-mode driver owns that management. An unavailable `nvidia-smi` value is not zero, and a compute-process list is not a complete accounting of five graphics clients. [NVIDIA-SMI documentation](https://docs.nvidia.com/deploy/nvidia-smi/index.html).

Microsoft also documents a Windows 10 issue in which Details-pane/Performance Monitor per-process GPU-memory counters can appear to leak; the Performance pane and WPR/WPA are recommended there. Do not diagnose Education from a monotonically increasing process VRAM counter without checking OS/version and adapter-wide evidence. This is a counter caveat, not proof the operator's machine is affected. [Microsoft GPU counter issue](https://learn.microsoft.com/en-us/troubleshoot/windows-client/performance/gpu-process-memory-counters-report-wrong-value).

Later analysis should correlate adapter dedicated/shared memory, the relevant GPU engines, frame timing and system RAM. Lower background FPS may save GPU execution while textures stay resident. VRAM pressure may also increase system-memory pressure. No per-client VRAM allocation or driver overhead was measured here.

## 8. Sharing immutable code without sharing accounts

Windows can share physical pages backing common executable/DLL mappings while retaining each process's private virtual address space; copy-on-write separates a page when necessary. This is an OS facility, not a reason to combine profiles or make writable account directories common. Whether current instance packaging actually maps a common backing image is a source/runtime question. Separate copies with identical bytes do not establish the same file mapping or a measured sharing benefit. [Windows memory protection and copy-on-write](https://learn.microsoft.com/en-us/windows/win32/memory/memory-protection).

Windows also exposes page combining, which periodically combines identical physical-page content. Its presence is not a per-client sharing guarantee, and its current state was not inspected. Do not enable or tune global combining/compression as part of this discussion. The documented mechanism offers no promised Education savings and no way to replace independent writable state with a safe common account store. [Microsoft page-combining documentation](https://learn.microsoft.com/en-us/powershell/module/mmagent/enable-mmagent).

Keep credentials, login contexts, profile writes and world/cache state independent. Do not hardlink writable profile trees, copy another account's tokens, inspect credential contents, or invent shared game heaps/textures. A future immutable-binary reuse design must first preserve the existing proven launch/isolation contract; its potential benefit is bounded by what Windows is not already sharing.

## 9. Exact-source audit: what our engine currently owns

All source links below pin the reviewed Local SHA; statements concern these owners, not a whole-program leak audit.

| Owner / source fact | Consequence for the investigation |
| --- | --- |
| [Manager entry point](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/apps/desktop/src-tauri/src/main.rs#L7) dispatches the private experiment before the UI bootstrap. | The exercised CLI route does not call the normal Tauri UI bootstrap. Removing a hypothetical manager WebView is not a demonstrated saving for this route; measure any future GUI route separately. |
| [Registered preparation](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs#L445) creates both clients from the same pinned official image. | Common image mapping is already possible. No measured benefit follows from another binary deduplication layer; writable profiles remain separate. |
| [Pacer](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/tooling/acceptance/windows/bootstrap-ordering/frame-pacing.h#L84) waits on a timer/events for 144/10; [Present wrapper](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/tooling/acceptance/windows/bootstrap-ordering/dxgi-pacing.cpp#L40) invokes it before native Present. | This controls render submission frequency and does not evict game chunks/textures. Whether the presenting thread also gates host updates remains a runtime question. |
| [Presentation Policy and chain lifetime](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/tooling/acceptance/windows/bootstrap-ordering/dxgi-pacing.cpp#L10) retain one process-lifetime policy; final chain release unbinds HUD and releases native interfaces. ResizeBuffers forwards unchanged. | No extra world/texture cache is evident here. Do not remove lifetime pins as apparent leaks or force swap-chain eviction. This does not exclude other native/game leaks. |
| [HUD raster and Canvas cleanup](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/tooling/acceptance/windows/bootstrap-ordering/fps-hud.cpp#L61) use RAII DIB/DC/font cleanup and temporary pixels; [worker](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/tooling/acceptance/windows/bootstrap-ordering/fps-hud.cpp#L190) redraws changed labels/DPI with a 250 ms bound-window fallback. | One 128x32 RGBA raster at 96 DPI is 16 KiB by arithmetic. This is NOT the total HUD cost: thread stacks, concurrent buffers, GDI/DWM and allocator costs are additional. Reusing drawing resources could reduce churn but may increase retained memory; profile before changing it. |
| [Startup section](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/apps/desktop/src-tauri/src/engine/independent_bootstrap.rs#L501) is 31,968 bytes; [journal](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/tooling/acceptance/windows/bootstrap-ordering/startup-observation.h#L8) has 64 records. | These named buffers are bounded; their size does not explain GiB-scale growth. This is not an upper bound on all engine overhead. |
| [Current collector](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/tooling/acceptance/windows/collect-native-performance.ps1#L48) records CPU and WorkingSet64 for selected original processes, plus optional GPU hints. | It does not yet report private commit, private working set, system commitment, or attributable helper trees. This is the first measurement gap, not proof that the game needs memory reclamation. |

The existing [runtime handoff](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5/planning/development.md) records operator-confirmed two-account same-world host/join and matching HUD behavior. A 30-second trace found approximately 100.3 ms present intervals for both clients, without aligned foreground transitions. Earlier CPU/RAM readings were taken before world-ready confirmation; neither they nor the new screenshot establish steady-state overhead or savings.

Potential engine improvements are narrower than rewriting game memory management: reduce demonstrated own allocation/handle churn, duplicated observation, or file/registry mediation cost. File walkers currently close temporary parents and enforce identity/reparse guards; shortcuts or indefinite caches could weaken those guarantees. The small owned buffers make game content/helper memory a plausible larger target, but its share is **UNKNOWN** until composition is measured. No fix or engine-wide leak diagnosis is claimed.

## Mechanism boundaries for the efficiency work

Operator direction on 2026-10-25: make original-system efficiency the primary
goal and keep every mechanism's responsibility separate. The following is a
design constraint over existing owners, not a new implementation, service,
public API or runtime-readiness claim. Current development intent and the one
next action remain solely in [the handoff](../../planning/development.md).

| Mechanism | Existing owner and narrow responsibility | Boundary |
| --- | --- | --- |
| Lifecycle and identity | `independent_bootstrap.rs` composes the private startup/replay route and binds original process identities and owned resources. | Consumers use that binding; no second launcher, PID-only identity cache or readiness flag. Production admission stays with the existing catalog/runtime owners. |
| Data isolation | Existing bootstrap configuration plus `bootstrap-ordering/compat.cpp` mediate instance-specific paths/registry/IPC. | Preserve separate writable profiles, private TEMP/TMP, account/world data and path/identity guards. No FPS, graphics-quality or memory-budget policy here. |
| Foreground observation | `instance_focus_events.rs` publishes verified focus changes through existing process-bound leases. | One observer/control owner; no second polling governor. Foreground is not Education host role; it cannot authorize suspending or starving a host. |
| Frame pacing | `frame-pacing.h` owns timing waits; `dxgi-pacing.cpp` adapts native Present and swap-chain lifetime to it. | Consume existing focus policy for 144/10 ceilings. No game-cache eviction, resource trimming, credential access or second FPS limiter. Preserve native return/lifetime semantics. |
| FPS measurement and display | `fps-hud.cpp/.h` presents measured accepted Present calls through its existing window/worker. | Display actual measurement, not the policy ceiling. No independent pacing or launch authority; overlay failure must not become game termination. |
| Resource capture | `collect-native-performance.ps1` collects metadata for verified identities. | Read-only observations; do not mutate settings, trim memory, change scheduling, kill helpers or steer games. Keep game, manager, verified helpers and unattributed/shared processes distinct; never double-count. |
| Interpretation | `evaluate-native-performance.mjs` evaluates captured evidence offline. | It cannot manufacture unavailable metrics, change runtime policy or grant production/five-client readiness. Version private diagnostic artifacts through their existing owner, without changing public wire schemas. |
| Graphics/settings experiments | Separate operator-approved comparison track; no supported automatic graphics adapter is established. | Do not invent a settings service, change visual quality silently or credit settings savings to the engine. Internal Education renderer optimization remains an unverified integration question. |

These boundaries are by responsibility, not a demand for one new class,
process, trait or configuration store per row. Retain existing small interfaces
and resource ownership; split code only when an observed coupling requires it.
Adapters translate existing platform calls; they must not duplicate domain
policy. Resource/admission decisions remain with the canonical runtime/catalog
owners, not the collector or a second RAM governor.

For the next measurement change, unavailable extended metrics stay null with a
specific reason. Lost process identity invalidates that process's attribution;
shared brokers stay separate rather than being guessed into a client total.
Collector capability failures must not launch, close or alter games. Measurement
overhead itself must be reported, and telemetry/HUD allocations must not be
mislabelled as game-content memory.

Completion of an efficiency change requires one identified owner, traced
consumers, an explained change to owned work or allocation lifetime, appropriate
source checks, and a matched before/after comparison at unchanged settings,
144/10 policy and host workload. Preserve sign-in, world persistence, connection
health, host progression and focus recovery. A lower Task Manager reading alone
is insufficient. Source-contract completion and runtime benefit are separate;
the latter stays UNKNOWN / NOT_EXECUTED until actually observed.

### Implemented diagnostic increment after the source audit

The existing collector now offers opt-in two-client `-MemoryComposition`
(schema 5 over base schema 1/2). It reads private commit, supported EX2 private
working set, whole-system physical/commit counters and memory-query duration;
the existing evaluator validates/reviews them. Default schemas 1-4 remain
compatible. See the [collector usage and proof boundary](../../tooling/acceptance/windows/README.md#opt-in-two-client-memory-composition).

The first increment left manager/helper attribution NOT_COLLECTED. The
follow-up schema 6 now adds separately pinned, operator-selected manager
readings and opt-in direct-child relation counts/private-memory sums in the
same owner. Candidate births are pinned before a second relation snapshot;
root/child checks and selected-root exclusion prevent guessed or duplicated
sums. Failed or partial relation coverage remains UNKNOWN. These observations
are not exclusive helper ownership or a full tree census: shared brokers,
services and deeper descendants remain unassigned. Process totals still do not
identify engine-owned allocations. Own-data Node/Windows checks passed as recorded in
the handoff. A subsequent real two-client baseline is summarized below;
no matched optimization benefit has been demonstrated.
The earlier exact-source audit above retains its original pinned snapshot;
it is not a claim that the collector still lacks these new opt-in fields.

### First live composition result — 25 October 2026

The [canonical handoff](../../planning/development.md#first-live-memory-baseline-and-output-path-repair--25-october-2026)
records the exact source/payload hashes, local report digest, coverage and limits.
The sampler at `106117aa6b5851968fd3e7f69f5f5d585a61709a` completed a 15-minute
unchanged-settings observation; the operator reported Minecraft - 2 as Education
host. In 113 steady intervals, game private residency averaged approximately
1292/1467 MiB, private commitment 2489/2684 MiB, and the CLI candidate manager
1.46/2.32 MiB resident/commit. Direct-child residency averaged approximately
115/114 MiB; one host relation sample remained UNKNOWN. Shared brokers and
compatibility-DLL allocations inside game heaps remain unassigned.

This rules out treating the current CLI manager as the obvious GiB-scale target.
It does not establish a leak, optimal game memory, full helper ownership or
a benefit from any engine change. The operator did not observe gameplay during
recording, so background host health remains UNKNOWN. Whole-PC workload also
changed: system available/commit deltas cannot be credited to the engine.
No matching optimized run, graphics comparison or five-client acceptance exists.

An initial long-path output failure lost its report. The existing collector's
shared atomic writer now performs an owned write/move probe before the timed
capture; regression and own-process Windows checks cover early failure and
preservation of existing artifacts. This is diagnostic reliability work,
not a game memory optimization. Current composition is partially measured;
engine-owned allocation attribution and repeatable gains remain UNKNOWN.

### First bounded engine-owned change

The next audit at parent `d9e8d60d147d6d104d33255fa4a2b9e752a584f9`
identified duplicate HUD pixel storage, rather than a large launcher cache.
The existing HUD now paints into its upload DIB directly, sharing one painter
with the unchanged preview helper. It removes one bitmap creation, one vector
allocation and two copies per redraw without retaining new resources.
Explicit transient pixel storage is one plane rather than two: 16 rather than
32 KiB at 96 DPI (400 rather than 800 KiB at 480 DPI). This is small relative
to the measured game processes and does not establish lower overall residency
or commitment.

The [current handoff](../../planning/development.md#first-owned-hud-allocation-reduction--25-october-2026)
records actual MSVC DLL/control builds, DPI/lifetime checks, a red/green
API-counting probe (18 uploads each), byte-identical preview output and
D3D11/D3D12 WARP regression checks. The running Minecraft pair still uses its
old payload. Matched real-game savings and host health on the changed payload
remain NOT_EXECUTED; short own-control timings are not a CPU-speedup result.

## 10. A bounded comparison that can identify the best tradeoff

This is a proposed later protocol, not an executed test or authorization to operate games now. Keep the host role separate from foreground state: a background host remains latency-sensitive. Declare roles explicitly; do not infer them from titles.

1. **Close the measurement gap first in the existing collector.** Propose private commit, private working set when supported, system available/committed memory, and separate original/helper/manager totals with verified PID/birth attribution. Shared or unattributed brokers remain separate/UNKNOWN; never count one process twice. PROCESS_MEMORY_COUNTERS_EX provides commit; [EX2](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-process_memory_counters_ex2) adds private working set and has explicit OS prerequisites. An unavailable extended metric must remain null with a reason, not zero. Reuse the existing diagnostic artifact owner and versioning; no second governor, credentials store or product wire schema.
2. **Compare fixed candidates in one planned session.** A=current settings, B=only render distance 6, C=B plus Education's documented lightweight visual settings. Add a fixed lower-resolution comparison only if the installed UI supports it. Keep version, world/location/workload, host role, resource packs and resolution otherwise equal; do not extrapolate co-located players to five clients exploring separate areas; use normal in-game settings, preserve data and restore original settings afterward. Different world/helper states are confounders. A stock-launch comparison is causal evidence only when workload/profile effects can be matched; otherwise label it descriptive.
3. **Control warmup and carryover.** Separate launch/world loading from steady-state measurements. For planning, use about 5 minutes warmup and 10 minutes per steady block, extending if memory is still changing; these are experimental starting durations, not acceptance guarantees. Repeat a promising comparison in reverse order (A-B-B-A across equivalent lifetimes). Match restarts only when normally saved/closed in an authorized session; do not force trims to manufacture a cold baseline.
4. **Align transitions and frame evidence.** Use a validated common timestamp basis for focus and native-present samples. Exercise host foreground, guest foreground with host background, and return; inspect time to resume smooth rendering and p95/p99 frame intervals. The earlier unaligned 30-second trace cannot supply these results. Preserve the single existing focus-control owner. SyncInterval 1 was observed previously; [DXGI Present](https://learn.microsoft.com/en-us/windows/win32/api/dxgi/nf-dxgi-idxgiswapchain-present) synchronizes to vertical blank, so display refresh/VSync remain confounders when assessing 144 displayed FPS.
5. **Select for both memory and playability.** Compare steady private commitment/residency and whole-system pressure, GPU dedicated/shared memory, CPU, frame tails and switch recovery. Observe continuing host world updates, saves and guest connectivity through ordinary gameplay. An option that saves memory but worsens host latency, disconnects players, loses persistence, or makes focus return stall is rejected. Keep results incomparable if required identity, workload or metric evidence is missing.

For later reporting, use `deltaMiB = baseline - candidate` and a percentage only for the same defined metric and matched workload. Separate working-set reductions from commitment reductions and whole-device VRAM from process attribution. Report median/peak and time trend, not only one low reading. Positive changes must repeat beyond natural run-to-run variation. A nonzero hard-fault counter alone is not a failure; correlate it with actual stalls, disk activity and memory pressure.

Do not run heap/memory dumps or retain account/token/world contents for this plan. Begin with metadata-only counters. Long play and repeated normal world transitions can reveal growth, but are a later authorized workload, not an implicit instruction to operate or edit worlds now.

## 11. Priority for discussion before clients 3–5

| Rank | Candidate | Expected impact, still unmeasured | Compatibility cost / next decision |
| --- | --- | --- | --- |
| 0 | Collect the matched two-client composition baseline with the existing opt-in manager/direct-child extension | Establishes what can actually be saved; no immediate resource reduction. | Diagnostic checks and one real baseline available; matched savings and in-process engine allocations remain unmeasured. |
| Optional settings track | Education's documented conservative graphics preset and render distance | Best-supported opportunity to reduce render work; possible RAM/VRAM benefit. Select the least costly preset that still meets the operator's visual needs. | Visual tradeoff; agree a stable usable preset before any setting change. |
| 2 | Validate existing 144/10 pacing and background host continuity | Potentially meaningful CPU/GPU work reduction; limited direct allocation effect. | Preserve host progression and fast focus return; avoid equating low FPS with low simulation rate. |
| Optional settings track | Verify supported render resolution | Potential GPU/render-target reduction. | Confirm actual Education behavior; keep a usable stable size. |
| 1 | Review emulator-owned lifecycle/resources after composition measurement | Removes only demonstrated emulator overhead. | Small bounded source work after a concrete retention/duplication finding; do not assume helpers are disposable. |
| 5 | Preserve common immutable mappings where compatible | Potential physical-page sharing; magnitude unknown and may already occur. | Isolation/launch risks can outweigh benefit; no profile/account sharing. |
| 6 | Targeted EcoQoS or memory-priority trial | Possible energy/scheduling or residency tradeoff; no guaranteed RAM gain. | Defer until measured contention; protect host and restoration behavior. |
| Reject as default | Periodic trim, hard caps, suspension, helper killing | Can lower a counter or stop work without reducing required game state. | Faults, starvation, allocation failures or broken sessions conflict with requirements. |

No numerical savings are claimed. The next useful evidence is a bounded, explicitly authorized comparison of the existing two-client workload, followed only later by additional clients. One baseline now supplies partial memory composition. Five-client capacity, in-process engine allocation ownership, Education's graphics-reload behavior and repeatable RAM/CPU/GPU gains remain **UNKNOWN / NOT_EXECUTED**. This note authorizes no game operations, settings changes, credentials/cache reads, CI or product implementation.

## Minecraft-internal efficiency feasibility: Sodium principles versus Education integration

Research date: 25 October 2026. This section extends the existing report;
current development intent remains in planning/development.md.
Project source reviewed at `506aef24b002204b6670d37280da0cadc24da6e9`.
The literature review accessed primary sources only; the subsequent authorized
UI intervention is identified separately below. Quantitative whole-game benefits
and a native internal-optimizer integration remain NOT_EXECUTED / UNKNOWN.

The operator now prioritizes Minecraft's own resource demand, permits reduced
visual effects/graphics, requires **render distance at least 12**, and requires
**simulation distance unchanged**. Keep resolution, independent profiles,
Education Host World and the existing 144/10 rendering policy. These preferences
supersede the earlier optional suggestion of render distance 6; do not apply 6.

### Decision

**Apply supported visual settings now; do not represent an Education Sodium port
as available.** There are real, transferable algorithms, but the current native
bootstrap/presentation wrapper has no identified ownership seam for Education's
terrain meshes, allocator or model graph. Direct installation of the Java mod
is unsupported; implementing equivalent native algorithms remains UNKNOWN until
that seam exists. This is an integration barrier, not proof that an independently
authored optimizer is mathematically impossible.

Education's [current official game-feature FAQ](https://edusupport.minecraft.net/hc/en-us/articles/360047117692-FAQ-Game-Features)
explicitly says community-created mods are not supported in Education. Its
[official Add-Ons collection](https://education.minecraft.net/en-us/resources/minecraft_education_add_ons_collection)
separately establishes support for packages that change world appearance, sound
and behavior. Both can be true: content-package support does not establish a
native renderer replacement interface. The FAQ also explains that Education
normally trails Bedrock's production version, so Bedrock API availability is not
an Education compatibility guarantee.

### Concrete source findings

Sodium source is pinned at
[`148151cf25c56f8b670986b348dcc589278c867e`](https://github.com/CaffeineMC/sodium/tree/148151cf25c56f8b670986b348dcc589278c867e).
For release context, its 0.9.2 Minecraft-26.1.2 tag resolves to
[`eb81e48d2646f76e7dbd1337d0f6ecb6a6dc5413`](https://github.com/CaffeineMC/sodium/releases/tag/mc26.1.2-0.9.2);
the release describes reducing terrain-buffer allocation/resizing. These are
two distinct source snapshots; the algorithm details below refer to 148151c.
No upstream code is copied.

| Mechanism | What the primary source actually owns/does | Education applicability |
| --- | --- | --- |
| Compact mesh representation | [CompactChunkVertex](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/vertex/format/impl/CompactChunkVertex.java) defines a 20-byte vertex stride and encodes positions/other attributes. | **UNKNOWN native integration.** Requires compatible mesh producers, consumers and shaders. Reducing an arbitrary D3D buffer size externally cannot preserve those contracts. The 20-byte figure is not evidence of Education's current format or a RAM saving. |
| Reuse and arena compaction | [ArenaAggregator](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/gpu/arena/ArenaAggregator.java) owns GpuBuffer allocation/reuse/closure; [DefragmentingBufferArena](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/gpu/arena/DefragmentingBufferArena.java) relocates segments under a defragmentation budget. | **UNKNOWN native integration.** Requires allocation ownership, segment references, command synchronization and destruction points. Primarily a terrain GPU-buffer mechanism; lower allocation churn does not itself prove lower private system RAM. |
| Culling and batching | [OcclusionCuller](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/occlusion/OcclusionCuller.java) traverses Java RenderSection/SectionStorage state with viewport and visibility information. | **UNKNOWN native integration.** A Present wrapper lacks the section graph and terrain draw semantics. Skipping a whole present is not equivalent to avoiding hidden geometry construction/drawing. No reason to assume Education currently lacks its own culling. |
| Bounded temporary world cache | [ClonedChunkSectionCache](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/world/cloned/ClonedChunkSectionCache.java) caps entries at 512 and expires old entries after five seconds. | **UNKNOWN native integration.** Useful ownership principle, but no Education chunk-copy cache or safe eviction seam is identified. Never free game-owned chunks or authentication caches from the wrapper. |
| Compact/deduplicated game state | FerriteCore [technical summary at `0cef1f2add1f1329aa6e690e8e292acd625c5c6d`](https://github.com/malte0811/FerriteCore/blob/0cef1f2add1f1329aa6e690e8e292acd625c5c6d/summary.md) changes Java block-state maps, model instances, predicates and shape/quad sharing. | **UNKNOWN native integration.** Requires replacing internal object representations and proving immutability. Historical Java modpack-specific memory numbers are not estimates for Education or permission to merge per-account private state. |

A useful caution beyond “reuse buffers”: Sodium's reviewed arena implementation
retains up to eight free GPU buffers, selects a reusable buffer within a bounded
oversize factor, and explicitly closes evicted/deleted resources. It also
overallocates some arenas to avoid growth churn. Therefore copying only the
word “pooling” can increase retained memory; full allocation/retirement and
performance semantics matter. Its Java [Fabric manifest](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/fabric/src/main/resources/fabric.mod.json)
declares loader dependencies, entrypoints and mixins. Those integration
contracts do not exist merely because our own DLL can load.

The project's [current DXGI owner](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/506aef24b002204b6670d37280da0cadc24da6e9/tooling/acceptance/windows/bootstrap-ordering/dxgi-pacing.cpp)
waits before forwarding Present/Present1 and records successful native
completions. It owns a pacer, HUD, wrapper references and chain lifetime, not
Education meshes or game allocations. **Inference from that source:** changing
this seam alone cannot supply Sodium's allocator, vertex layout or scene culler.
Generic hooking capability is not the missing semantic ownership contract.

### Supported content and optional features

- **Supported Education content lane:** resource/behavior Add-Ons are officially
  available. A lightweight *resource-only* content candidate is plausible when
  expensive custom textures/models/animations are present and the operator
  accepts the visual change. Its actual contents and version compatibility
  remain UNKNOWN here. The [Bedrock resource-pack documentation](https://learn.microsoft.com/en-us/minecraft/creator/documents/resourcepack?view=minecraft-bedrock-stable)
  describes content/manifest/stacking semantics, not a native heap/terrain
  allocator SDK. Do not infer Education support for every Bedrock pack key or
  claim that adding an override necessarily releases an original resource.
- **Not established:** an officially supported Education native-renderer plugin,
  model-cache ownership API, or mesh allocator replacement SDK. The reviewed
  [Bedrock server Script API](https://learn.microsoft.com/en-us/minecraft/creator/scriptapi/minecraft/server/minecraft-server?view=minecraft-bedrock-stable)
  manipulates world entities/blocks/dimensions; it is not proof of Education
  support or client GPU-buffer ownership. Native replacement feasibility stays
  UNKNOWN; direct Java-mod loading is unsupported.
- **Code Builder option:** [Education Classroom Settings](https://edusupport.minecraft.net/hc/en-us/articles/360061369132-Classroom-Settings)
  officially documents a Code Builder toggle, enabled by default, controlling
  access through C/the Agent icon. That establishes an allowed feature setting,
  **not** helper teardown, lazy initialization or memory release. Disabling it
  changes coding availability and is outside a graphics-only change. Do not
  toggle it without agreeing that tradeoff, or kill/merge its helper processes.
  Possible helper savings remain UNKNOWN until the game's own lifecycle proves
  them. Repeated helper names alone do not establish redundant work.

### Applied settings and remaining integration question

The operator-approved video candidate was applied through the actual Education
26.30.2 UI on both current clients: Graphics Mode Fancy → Simple; Fancy Leaves,
Fancy Bubbles, Render Clouds, Beautiful Skies and Smooth Lighting on → off;
Anti-Aliasing 2 → 1. Render distance was already 12 chunks on each and remained
12. Simulation/world settings, resolution, profiles and 144/10 policy were not
changed. The UI confirmed each changed value and returned normally to the main
menu. Disk retention has subsequently been observed in both original profiles after
normal exit; the restarted MC02 UI also confirms Simple / 12 chunks. An equivalent
loaded-world resource benefit has not been demonstrated. Verify the actual installed settings rather than importing Java or
Bedrock-only keys. The official [Education performance guidance](https://edusupport.minecraft.net/hc/en-us/articles/4404976988052-Troubleshoot-Performance-Issues)
supports lowering these visual workloads, but supplies no numerical RAM promise
and its render-distance-6 recommendation is superseded by the operator's 12
minimum. This is a settings result, clearly distinct from engine code savings.

After that concrete change, one bounded before/after check can establish whether
it helps; do not let further telemetry replace implementation. A native
Sodium-style branch should not start with generic detours or a demonstration
allocator: first it needs **one real Education mesh/resource ownership and
lifetime seam**. If none is verified, explicitly retain the unsupported/UNKNOWN
decision and pursue supported content/settings improvements instead.


## Current A/B/C goal and settings persistence on resume

The operator explicitly resumed work and established A (durable game settings),
B (real internal Minecraft optimization), and C (inactive-client efficiency,
prioritized) as the current goal. See the [canonical handoff](../../planning/development.md#primary-goal--a--b--c-minecraft-efficiency)
for ownership and one next action.

After both game processes disappeared, their existing manager logs reported
normal exit code 0 and saved startup journals. Read-only inspection of the
original per-slot game options files confirmed the approved graphics entries
still existed independently on disk: graphics_mode=0, gfx_viewdistance=192,
gfx_msaa=1, gfx_transparentleaves=0, gfx_smoothlighting=0, gfx_fancyskies=0,
gfx_toggleclouds=0 and gfx_bubble_particles=0. The value 192 corresponds to the
12-chunk value observed in the installed UI earlier; it is not a new cross-version
settings contract. No option file was rewritten and no account/world content
was accessed. This establishes disk retention after normal exit. A subsequent restart of the
same candidate/original profiles showed the original accounts in both main menus;
MC02's video UI also confirmed Simple / 12 chunks. The complete MC01 video UI
was not rechecked. No quantitative performance benefit is established.

For C, real native minimization/occlusion of an inactive guest is a concrete
candidate for avoiding game rendering work. Microsoft's [occlusion guidance](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/waiting-when-occluded)
explains the platform event/wait capability; it does not prove Minecraft's
loop, host simulation or memory retirement. Keep any declared host running,
use the existing presentation owner, and require normal recovery/connectivity.
Do not synthesize an occlusion HRESULT or suspend the whole client. Automatic
policy remains NOT_IMPLEMENTED. The bounded actual candidate below supersedes
its earlier NOT_EXECUTED observation status without establishing savings. The B internal
mesh/allocator integration boundary remains as documented above.


### Bounded intervention result — inactive guest and unused Code Builder

The guest joined the original Education host world. Normal minimization was
confirmed by the Computer Use window API's minimized-state response, and a normal
restore returned to its multiplayer pause surface showing two participants.
The host was observed across a night-to-day transition. This establishes only
short recovery/connection behavior, not a soak or host tick/network benchmark.

| Observation | Elapsed seconds | Guest CPU-seconds | Working set MiB | Private commit MiB |
| --- | ---: | ---: | ---: | ---: |
| Earlier background world surface | 10.015 | 1.547 | 1411.00 | 2433.99 |
| Confirmed minimized guest | 10.018 | 1.406 | 1412.87 | 2432.90 |

Working set is the process total, not private resident or unique physical RAM.
The guest was in its pause UI for the confirmed minimize interval, so this is
not a matched CPU/GPU comparison. A prior minimize request while mouse capture
was active was unconfirmed and excluded. **No meaningful RAM saving was
demonstrated; automatic minimization is rejected for rollout on this evidence.**
No new sampler, GPU trace or artificial occlusion result was added.

The operator confirmed Code Builder/Agent is unused and authorized disabling it.
After normal save/exit of guest and host, the existing host world's Edit World →
Coding & Scripting setting was switched from on to off; Save changes returned
the game's saved confirmation. Console commands, command blocks, simulation,
resolution and world contents were not edited. This is an actual world-scoped
feature setting, not a global per-profile/default promise or internal renderer
optimization. Helper teardown and RAM/CPU/GPU savings remain **UNKNOWN /
NOT_MEASURED**. The original games are left open in menus; other worlds are
untouched. Native mesh/allocator integration remains UNKNOWN.


### Driver-memory candidates: concrete ownership decision

Reviewed current [DXGI owner at d9af812](https://github.com/OpenITStudios/M-Minecraft-Emulator/blob/d9af812232f52fe299812b51a6ce4071c6e60b20/tooling/acceptance/windows/bootstrap-ordering/dxgi-pacing.cpp).
GetDevice/GetBuffer are forwarded; the adapter owns presentation pacing and HUD
lifetime, not terrain/texture generation, command recording, or the game's idle
boundary. Successful QueryInterface would establish API availability only.

| Candidate | Primary contract | Decision for the current integration |
| --- | --- | --- |
| [IDXGIDevice3::Trim](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_3/nf-dxgi1_3-idxgidevice3-trim) | Discards driver-internal buffers; intended before an extended render-idle period and incurs reallocation cost on rendering again. | Reject periodic/focus-triggered calls. Background 10-FPS rendering is not a verified idle transition. Do not ClearState/Flush a game-owned device to imitate suspension. Runtime NOT_EXECUTED. |
| [OfferResources](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/nf-dxgi1_2-idxgidevice2-offerresources) / [ReclaimResources](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/nf-dxgi1_2-idxgidevice2-reclaimresources) | Offering can unbind resources and discard contents. Reclaim must precede reuse; discarded contents must be written again. | Reject arbitrary game-resource offering: no Education mesh/texture regeneration contract exists in this adapter. Runtime NOT_EXECUTED. |
| [D3D11 rasterizer-state deduplication](https://learn.microsoft.com/en-us/windows/win32/api/d3d11/nf-d3d11-id3d11device-createrasterizerstate) | Equivalent state requests already return the same interface. | No evidence for another cache or reduced allocations here. Actual Education graphics device/version/use remains unmeasured; no game benefit is claimed. |

These decisions close specific proposals rather than claiming native optimization
is universally impossible. The operator confirms no native Education source/SDK
or integration reference is available. The B prerequisite is a verified internal
allocation, ownership and rebuild seam; the current seam is insufficient.
Version-specific reverse engineering would have to establish those semantics
before an original internal optimizer could be justified. No code, arbitrary
resource release, memory cap, trim loop, collector extension or game-file change
was introduced. B remains incomplete; A/C are not relabeled as satisfying it.


## Native efficiency implementation mechanisms and specific tools (25 October 2026)

The [implementation playbook](./native-efficiency-implementation-playbook.md) provides concrete Codex-ready mechanisms N01-N04 (registry/file hooks, expired startup tracing, event-led manager), R01-R03 (pacer timers, HUD, presentation), M01-M02 (owned immutable data), B01-B02 (Sodium/FerriteCore internal game feasibility) and W01 (conditional content tooling). Each has exact source owner, Windows/C++/Rust API, algorithm, safety/compatibility boundary and primary reference. This is **supporting technical design**, not new deployment authorization, an automatic optimization claim, or a second active TODO owner. The A/B/C current goal and **one next action** remain in [planning](../../planning/development.md).


## Deeper RAM-efficiency research — 25 October 2026 discussion

This is a research update, not authorization to change running games or enable
telemetry. The earlier absence of a native mesh SDK blocks that specific
integration, not every possible allocator or built-in-feature investigation.
No Minecraft RAM reduction has been measured by this research.

### Separate live data, allocator retention, residency and GPU memory

A process can retain live world/model/mesh data, unused allocator capacity, mapped
images/files and driver allocations. Their remedies differ. Working-set
reduction alone does not establish fewer live allocations or less private commit.
[VMMap](https://learn.microsoft.com/en-us/sysinternals/downloads/vmmap) can separate
committed-memory types and resident pages without proprietary game symbols.
[Microsoft's allocation-analysis guidance](https://learn.microsoft.com/en-us/troubleshoot/windows-server/performance/troubleshoot-application-service-memory-leaks)
also notes that stacks without third-party function names can still identify
modules. Thus missing PDBs are a limitation, not a universal diagnostic dead end.
No dump, heap trace or game memory contents were collected here.

### New allocator-level candidate: LFH cache decommit

[HeapSetInformation / HeapOptimizeResources](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapsetinformation)
can optimize LFH caches and decommit memory where possible; NULL selects the
calling process's LFH heaps. This differs from removing resident pages. Its
reach into Minecraft is UNKNOWN: live objects, custom arenas, direct VirtualAlloc
allocations and GPU resources are not automatically reclaimed. API success
does not prove reclaimed bytes. Heap coverage, private-commit change, refill
cost and focus/frame latency need evidence before any deployment. No periodic
loop, DllMain/Present maintenance, guessed page freeing or runtime call is proposed.

[Segment Heap manifest opt-in](https://learn.microsoft.com/en-us/windows/win32/sbscs/application-manifests#heaptype)
and [mimalloc Windows redirection](https://microsoft.github.io/mimalloc/overrides.html)
are different mechanisms. Neither establishes a safe late allocator replacement.
Changing our manager's heap does not optimize Minecraft's. Do not modify the
signed game manifest/imports or redirect allocation/free families without a
complete compatibility contract. This candidate is allocator footprint work,
not an implemented Sodium mesh optimizer.

### Real renderer structures, not just Present

Mojang's [GDC 2026 RenderDragon presentation](https://media.gdcvault.com/gdc2026/Slides/Fairfield_AJ_ModernizingTheRenderingOfMinecraft.pdf)
describes per-frame draw-data registries and opaque resource lifetime handles
(page 10), paged vertex pools with chunk/layer buffer ranges (page 15), and a
terrain atlas (page 21). Inference: reduced frame submission does not itself
release persistent mesh pools. Pool occupancy, retirement and rebuild ownership
are the relevant internal questions; this architecture description is not an
Education extension ABI.

### Built-in native mechanisms and version compatibility

The [Bedrock 26.30 changelog](https://www.minecraft.net/en-us/article/minecraft-26-30-bedrock-changelog)
documents PC/mobile texture streaming, matching voxel-shape deduplication at
world load, and texture-cache/streaming bug fixes. These are actual native
mechanisms worth checking before duplicating them. The [Education changelog](https://edusupport.minecraft.net/hc/en-us/articles/360047556451-Minecraft-Education-Change-Log/)
lists Chaos Cubed and performance fixes in 26.32; it does not enumerate equivalent
renderer capabilities. The installed options key or shared version ancestry
alone does not prove an active streaming implementation or its savings.
[Creator voxel-shape documentation](https://learn.microsoft.com/en-us/minecraft/creator/documents/voxelshapes?view=minecraft-bedrock-stable)
is a content contract, not access to all native block-state memory.

### Existing category diagnostics may help without proprietary symbols

Mojang's [memory-category documentation](https://mojang.github.io/bedrock-protocol-docs/1.26.60-preview.25/types/memory-memory-category/)
names terrain vertex/index buffers, tessellator, render registry, textures and
subchunk data. The [official debugger](https://github.com/Mojang/minecraft-debugger/blob/main/README.md)
also documents client diagnostics. This is a possible target-selection lane,
not an optimizer. Education compatibility, telemetry behavior and exact version
support remain UNKNOWN. Do not enable telemetry/privacy settings, attach a
network proxy or create another collector from this research.

### Historical Education-specific leak lead

A [2025 Education support discussion](https://edusupport.minecraft.net/hc/en-us/community/posts/38696872814484-Memory-Leaks?page=1)
contains first-hand reports that disabling Improved Input Response and restarting
resolved growth on affected builds, with a support-agent acknowledgement.
This is a historical regression lead, not an official current-build diagnosis
or evidence that our stable baseline is leaking. Establish feature availability
and version applicability before changing settings.

### Discussion recommendation

Keep A (durable settings), B (internal/allocator mechanisms) and C (inactive-client
behavior) explicit. First choose the largest actionable memory category rather
than collecting another time series of aggregate RAM. A focused existing-tool
snapshot or supported built-in diagnostic should answer a named decision:
unused LFH capacity, live/custom-arena state, textures, or terrain mesh. Close
the chosen candidate if its target category is absent. Implement only the
matching mechanism and verify reclaimed private commit/physical memory plus
focus/host health once. Five-client extrapolation and numerical savings remain
unproved. Render distance minimum 12, simulation, resolution, original profiles
and 144/10 policy remain unchanged.


## Executed LFH action, not an automatic RAM policy

A one-shot original action now exists in [heap-reclaim.cpp](../../tooling/acceptance/windows/heap-reclaim.cpp);
its opt-in build, identity guards, uncertain-thread handling and limitations are
documented in the [Windows tooling guide](../../tooling/acceptance/windows/README.md#explicit-one-shot-lfh-reclamation-experiment).
It requests documented heap-manager cache optimization, not arbitrary VirtualFree
or resource eviction. It is separate from native mesh optimization.

The original MC01 guest's menu-time private commit fell 38.754 MiB during an
8.903 ms API interval. The original MC02 Education host, running the saved world
with MC01 joined and foreground, fell 21.707 MiB during a 3.011 ms interval.
These are two different workload states, not a simultaneous durable total.
Both action DLLs unloaded and both games remained responsive; the guest's world
surface remained connected after the host action. No frame-latency, long-soak,
post-action save/restart or persistent in-world savings claim is made.

The initial guest VMMap category snapshot showed ~712.38 MiB heap commit and
~1229.58 MiB Private Data commit. Heap coverage/free space was not inferred from
that classification. Main process heap compatibility was LFH; the separate
snapshot's heap commit decreased ~36.51 MiB. Working-set reductions are excluded
from causal claims, especially because the untouched peer also lost residency.

This is the first bounded observation of an allocator-level private-commit
reduction in the current resumed effort. Its size is modest, not evidence for
a production cleanup loop, a Java optimizer port, or five-client readiness.
Canonical numbers, hashes, exact source baseline (8250ebf) and remaining proof
are in the current handoff. Next work should target the larger unattributed
Private Data allocation owner, not repeat LFH calls or add another collector.

A later running-host reading was 2864721920 private bytes, about 12.03 MiB above
its immediate post-action value. Ongoing game allocations confound attribution;
this reinforces that the instantaneous drop is not a durable RAM guarantee.


## Dominant private allocation class and GPU-upload eligibility

The already-local VMMap CSVs have parent allocations plus two-space-indented
leaf regions. Counting only 728 private-data leaves gives 1259088 KiB committed,
exactly equal to the summary. There are 57 write-combined leaves totaling
1098364 KiB (1072.62 MiB, 87.2%); that total is identical after the one-shot heap
action. These correct binary units replace earlier rough conversational GiB
figures. Parent+child summation would double-count the allocation.

The observed game modules include D3D12/D3D12Core and NVIDIA user-mode drivers.
[Microsoft's GPU-upload specification](https://microsoft.github.io/DirectX-Specs/d3d/D3D12GPUUploadHeaps.html)
maps ordinary discrete-GPU upload memory to write-combined L0 and GPU_UPLOAD to
write-combined L1. Therefore graphics upload memory is a plausible hypothesis,
not a proven attribution from page protection. CPU-visible mapped buffers are
not automatically unused memory that the LFH manager can release.

A temporary own-process D3D12 OPTIONS16 query returned S_OK and
GPUUploadHeapSupported=false. NVIDIA reported 8192 MiB framebuffer and 256 MiB
BAR1. [NVIDIA's documented GPU-upload requirements](https://developer.nvidia.com/blog/optimizing-dx12-resource-uploads-to-the-gpu-using-gpu-upload-heaps/)
require full CPU-visible VRAM on discrete GPUs. No game-device heap conversion,
BIOS/driver change or extra VRAM consumption was attempted. This closes automatic
GPU_UPLOAD substitution on the observed configuration, not every graphics-memory
optimization. It is a capability result rather than Minecraft RAM savings.

The planned [WPR VirtualAllocation](https://learn.microsoft.com/en-us/windows/apps/develop/performance/disk-memory)
capture needs administrator rights. Its UAC launch was canceled; no new ETL or
active default/named recording exists. A local Microsoft TraceEvent parser was
prepared and its API/ETL-read preflight succeeded, but the intended allocation
attribution remains NOT_EXECUTED. No new product collector or recurring service
was introduced. Guest restored to its existing host world after normal save/exit.

New-allocation traces cannot retrospectively identify stacks for an unchanged
pool allocated before recording. A single recorder-ready cold start of the
existing candidate/original profiles is the appropriate next observation if the
operator provides manual UAC consent; do not keep retrying the canceled request
or substitute another aggregate RAM chart. Until then, actual owner/lifetime
and safe buffer reduction remain UNKNOWN.


## Approved cold-start allocation attribution

Following explicit permission to retry UAC, one elevated existing WPR
VirtualAllocation file-mode capture completed around startup of the same
hash-checked candidate and original profiles. A ready handshake preceded launch;
the old processes exited with code 0 and no force termination was issued. The
guest's normal Save & Exit UI was observed; a separate host save UI sequence was
not observed. Both original accounts reappeared automatically, the existing
Education host world was reopened, and the guest rejoined it. This is bounded
recovery evidence, not an optimization or stability soak.

The capture requested 30 seconds before stop. The final ETL session metadata spans
25 October 2026 10:38:49.516–10:39:47.975 UTC, including stop/rundown.
TraceEvent 3.2.8 reports zero lost events; the named WPR session is no longer
recording and its elevated helper exited. No product collector/API, recurring
worker, game-file modification or additional heap action was added. Raw ETL,
addresses, process identity, symbol cache and analysis are kept local, outside Git.

A later menu-time VMMap snapshot (10:40:52.629 UTC) contains 50 write-combined
Private Data leaves totaling 1040.824 MiB. Counting indented leaves avoids
parent/child duplication. Offline matching uses each 4-KiB interval's last
captured commit/decommit/release and preserves distinct module-relative stack
frames; grouping only by module name would conceal different creation paths.
No zero-length free event occurred for the selected process.

All selected addresses match graphics allocation stacks through dxgmms2,
dxgkrnl, nvwgf2umx and D3D12. Public D3D12 symbols downloaded from Microsoft's
symbol server resolve the following candidate routes:

| Last captured creation route for current snapshot addresses | Matched MiB |
| --- | ---: |
| CreateHeap | 704.434 |
| CreateCommittedResource | 66.875 |
| Other/truncated graphics paths | 269.516 |
| Total | 1040.824 |

The largest group is 257.934 MiB, with CDevice::CreateHeap and game RVA
0xBC224F7 in its stack. These are **candidate owner correlations**: the snapshot
is roughly 65 seconds after the final ETL timestamp, and post-trace reuse or
resource changes are not recorded. Zero lost events does not eliminate that
gap. Heap type, terrain/texture/upload purpose, unused capacity and last GPU
use remain UNKNOWN. Do not label all write-combined allocations as upload heaps
solely from protection or these creation methods.

The trace's separate 1193.742 MiB sum is graphics-stack *commit requests during
the interval*, not retained bytes. Earlier 1072.62 MiB and current 1040.824 MiB
snapshots are different sessions/workloads; their difference is not a saving.
No additional Minecraft RAM reduction is established by this capture.

[CreateHeap's contract](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12device-createheap)
requires GPU reads/writes to have ended before the heap's final reference is
released; placed-resource references and reserved mappings also matter.
[Microsoft's fence-based reuse example](https://learn.microsoft.com/en-us/windows/win32/direct3d12/fence-based-resource-management)
ties reuse to allocation offsets and GPU completion. A creation stack alone
provides neither. Therefore ordinary LFH reclamation, arbitrary VirtualFree,
descriptor shrinking or releasing game-owned references is not a justified
optimization for this category. The concrete B integration target is now the
game-side owner of these D3D12 allocations; its occupancy/retirement contract
must be identified before implementing an original reuse/retirement change.

Local evidence SHA-256: ETL
`b9d7478705aab7eee8bb0febe3cabde0cf0129d38da02fe4f074201c6b2a2adb`;
menu VMMap CSV
`4e37fce8c5d97a662e8980155769ecafbfe1b2bce4e2637f14ebd71c2ec99319`.
The sole next action remains in the canonical handoff; five clients stay deferred.
