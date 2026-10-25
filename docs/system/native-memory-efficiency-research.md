---
id: document.system.native-memory-efficiency-research
class: DOCUMENT
domain: system
role: REPORT
authority: WORKING
lifecycle: ACTIVE
---

# Native client memory and rendering efficiency research

Status: DISCUSSION / RESEARCH ONLY. Researched 2026-10-25 against product source authority `OpenITStudios/M-Minecraft-Emulator`, branch `Local`, exact HEAD `5eaa49ed41c5c455f1b783d3a7c659bbba40e2d5`. The performance proposals do not implement game changes or live setting changes; the later diagnostic-only increment is identified below. No new Windows/game measurement or optimization experiment ran for this research. Gains from the proposed changes and five-client capacity remain **UNKNOWN / NOT_EXECUTED**; prior two-client observations retain their own evidence boundary below.

## Decision context

The operator reports two original Minecraft Education clients working on an i7-14700, 32 GB RAM, RTX 3070 with 8 GB VRAM, and a 512 GB SSD, with no usable iGPU. The intended next stage is one Education client hosting a world and four guest clients joining that world. Foreground rendering should have a ceiling of 144 FPS; background rendering should have a ceiling of 10 FPS, while host simulation/networking continue and switching stays quick. These are requirements. Prior operator-confirmed two-client behavior is partial evidence, not five-client acceptance or a controlled efficiency benchmark.

The two screenshot readings, 1797.3 MB and 1724.2 MB, are observations at one moment. Their column semantics and process-group coverage are not established. They do not demonstrate a leak, private commitment, unique physical RAM, or a predictable per-client increment. Neither a savings estimate nor a capacity guarantee follows from them.

The primary direction is now original-system efficiency at unchanged game settings and 144/10 policy, starting with memory composition and attributable engine overhead. Education's documented settings remain a separate optional tradeoff, not the primary engine optimization. Keep independent accounts/profiles and original game behavior. Treat external memory trimming, suspension and hard caps as poor default candidates because they can move cost into faults, stalls or failed allocations. Every resource effect remains a hypothesis until measured.

## Attribution rule: settings savings versus original-system efficiency

Operator clarification on 2026-10-25: distinguish changes to game settings
from improvements in the system we build. The primary engineering objective
is lower overhead and unnecessary work at the SAME game settings, visual
output, 144/10 ceilings and host role. A lower render distance/resolution is
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
There is no E1 implementation yet. Keep foreground/background policy identical
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
the handoff, with no new Minecraft measurement or demonstrated savings.
The earlier exact-source audit above retains its original pinned snapshot;
it is not a claim that the collector still lacks these new opt-in fields.

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
| 0 | Collect the matched two-client composition baseline with the existing opt-in manager/direct-child extension | Establishes what can actually be saved; no immediate resource reduction. | Diagnostic source/own-data checks implemented; Minecraft composition and savings remain unmeasured. |
| Optional settings track | Education's documented conservative graphics preset and render distance | Best-supported opportunity to reduce render work; possible RAM/VRAM benefit. Select the least costly preset that still meets the operator's visual needs. | Visual tradeoff; agree a stable usable preset before any setting change. |
| 2 | Validate existing 144/10 pacing and background host continuity | Potentially meaningful CPU/GPU work reduction; limited direct allocation effect. | Preserve host progression and fast focus return; avoid equating low FPS with low simulation rate. |
| Optional settings track | Verify supported render resolution | Potential GPU/render-target reduction. | Confirm actual Education behavior; keep a usable stable size. |
| 1 | Review emulator-owned lifecycle/resources after composition measurement | Removes only demonstrated emulator overhead. | Small bounded source work after a concrete retention/duplication finding; do not assume helpers are disposable. |
| 5 | Preserve common immutable mappings where compatible | Potential physical-page sharing; magnitude unknown and may already occur. | Isolation/launch risks can outweigh benefit; no profile/account sharing. |
| 6 | Targeted EcoQoS or memory-priority trial | Possible energy/scheduling or residency tradeoff; no guaranteed RAM gain. | Defer until measured contention; protect host and restoration behavior. |
| Reject as default | Periodic trim, hard caps, suspension, helper killing | Can lower a counter or stop work without reducing required game state. | Faults, starvation, allocation failures or broken sessions conflict with requirements. |

No numerical savings are claimed. The next useful evidence is a bounded, explicitly authorized comparison of the existing two-client workload, followed only later by additional clients. Until then, five-client capacity, current memory composition, Education's graphics-reload behavior and the practical RAM/CPU/GPU gains remain **UNKNOWN / NOT_EXECUTED**. This note authorizes no game operations, settings changes, credentials/cache reads, CI or product implementation.
