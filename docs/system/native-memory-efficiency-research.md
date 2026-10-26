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

## Active RAM goal: offline ownership and residency reconciliation — 25 October 2026

The operator resumed A/B/C implementation work with **no PC control while they
work**. This supersedes earlier full-access permissions for the current session:
no UI/input, game launch/closure, live process or memory inspection, new trace,
settings edit or runtime intervention. Work here used saved files only. The
goal remains actual Minecraft RAM efficiency, especially inactive clients;
144/10 pacing, Education host/network/save continuity, original profiles,
render distance at least12 and unchanged simulation/resolution remain boundaries.
Below1GB is aspirational, not a memory cap or acceptance prediction.

### Residual content/allocator review: integration impasse

Reviewed source parent `fe9dc1dfea8d541527e824b37473dedb30efc0c5`.
The final remaining content/allocator review found no additional intervention
whose Education waste owner, relevant bytes and compatible lifecycle are all
established. This is a boundary of the evidence and access currently available,
not proof that Education can never be optimized. The main goal is **BLOCKED,
not complete**; do not generate further research checkpoints, synthetic
allocators or repeated recordings for these same missing facts.

| Route reviewed | What the actual contract supports | What prevents a justified current implementation |
| --- | --- | --- |
| Windows LFH cache decommit | [HeapOptimizeResources](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapsetinformation) lets the heap manager optimize eligible LFH caches and decommit where possible. Existing heap-reclaim.cpp already makes that one-shot request. | Historical38.754/21.707MiB drops were private commitment at different workloads, not durable resident savings. Later growth and incomplete refill/latency/host evidence do not justify automatic scheduling. The saved direct graphics allocations did not decrease from that action. Reuse the recorded result; do not rerun it. |
| Allocator replacement/compaction | Production allocators can reclaim backing whose free/live state they own; Mesh has a different platform/aliasing contract. | The game allocation/free families, ownership transfer, free spans and Windows-compatible remapping are not established. Changing our manager's allocator would target the wrong process. |
| Supported content reduction | Microsoft's [resource guidance](https://learn.microsoft.com/en-us/minecraft/creator/documents/practices/improvingperformanceandresourceusage?view=minecraft-bedrock-stable) identifies model complexity, texture dimensions/counts, sound caches and entity work as relevant. | Reviewed allocation artifacts do not identify the active expensive/duplicate asset, its native copies or removable resident bytes. Compressed PNG/pack bytes are not a CPU-RAM allocation budget; formats, copies and mip/atlas overhead matter. No speculative replacement pack, world edits or asset deletion. |
| Resource-pack extension surface | The [Education collection](https://education.minecraft.net/en-us/resources/minecraft_education_add_ons_collection) offers content packages; [resource-pack stacking](https://learn.microsoft.com/en-us/minecraft/creator/documents/resourcepack?view=minecraft-bedrock-stable) changes selected content definitions. | Content overrides do not expose the game renderer's tables/mesh/cache lifetime. Bedrock documentation does not establish an Education native optimizer ABI; the [Education FAQ](https://edusupport.minecraft.net/hc/en-us/articles/360047117692-FAQ-Game-Features) separately excludes community mods. Official collection support must not be inflated into support for arbitrary native plugins. |
| Inactive-client reclamation | Existing focus/presentation owner provides144/10 pacing; host role remains independent. | Focus loss is not an unused-resource predicate. Render distance12, unchanged simulation and continuing host/network/save work rule out arbitrary world-state discard or suspension. A resource-owner retirement/rebuild contract is still missing. |

The LFH result is a legitimate owned-heap action, so a blanket claim that no
native memory mechanism exists would be wrong. What is missing is a justified,
durable RAM optimization policy for this workload. Likewise content reduction
is a valid option when its actual assets/compatibility/cost are known; adding
an unselected content pack now would not establish the requested engine result.

**Completion audit:** A has historical per-profile disk retention and partial
restart/UI evidence; no present-session profile inspection was made. B lacks
an integrated owner-compatible intervention and demonstrated durable
whole-Minecraft resident reduction. C retains historical pacing evidence but
has no new inactive RAM benefit or changed-optimizer host/gameplay proof.
Below1GB is unproven and aspirational. Accounts/worlds, simulation/resolution,
minimum distance and original-code boundaries remain preserved because no
runtime change was made. No product build or game acceptance executed here;
new savings remain NOT_EXECUTED.

The same missing integration/liveness evidence remained at the saved-range
reconciliation (f913d1c), BGFX comparison (fe9dc1d) and this residual review.
The remaining safe action needs **new compatible Education owner/access/release
evidence**, such as a documented integration or independently verified native
contract with relevant waste bytes. A new generic trace, broader PC permission,
or a lower Task Manager reading alone would not satisfy that requirement.
Keep the working payload unchanged and the prior count experiment OFF. Do not automatically
start a PC session, mutate settings or schedule a follow-up.

### Historical sample and conservative join

Source parent: `4b54a031eabf74c6881d1e2444b24e9bc531cf42`.
Existing cold-start ETL/ETLX allocation stacks were reconciled with the saved
menu VMMap CSV. Trace interval was 10:38:49.516–10:39:47.975 UTC; the VMMap
snapshot was about10:40:52.629 UTC, approximately65seconds later.
EventsLost=0 and zero-length-free count=0 in the offline reader do **not** cover
post-trace changes. This is a historical menu correlation, not current gameplay,
a synchronized liveness census or evidence that any page can be freed.

The existing offline reader tracks each4KiB page's last captured commit/free
stack. Its ignored local derivative changed only the final aggregation key to
prefix the existing range index; event handling was unchanged. Cached ETLX
reading and its in-memory analysis-helper compilation executed successfully.
No product build, new recorder, collector API, test fixture, symbol download,
new game-binary extraction or runtime experiment was introduced.

Only two-space-indented VMMap leaf rows were counted, avoiding parent/child
double-counting. A range must match one leaf by base and size; attributed page
bytes must equal the entire range. A leaf's Private WS is assigned to a route
only if **all** its captured page owners agree on that route. Mixed leaves
retain their entire WS as MIXED; no proportional owner-to-residency estimate
is used. A VMMap leaf is not asserted to equal one native D3D heap.

Historical menu total Private WS was1030.3671875MiB. Heap leaves contributed
296.7578125MiB, image private pages286.44140625MiB, WriteCombined private-data
leaves381.48828125MiB and other private-data leaves56.98828125MiB.
These are components, not a complete semantic ownership model; the remaining
categories still exist. Do not compare these menu values as savings against
earlier in-world samples.

### Graphics growth: resident relevance without free-capacity proof

All50 WriteCombined leaves reconcile to1040.82421875MiB committed and
381.48828125MiB Private WS. Thus the earlier approximately1GiB committed category
must not be advertised as1GiB of physical RAM available for reclamation.

| Captured game creation / caller return RVAs | Saved-code interpretation | Leaf count | Committed MiB | Private WS MiB |
| --- | --- | ---: | ---: | ---: |
| BC224F7 / BC2223E | Initializer requests heap creation |27|360|8|
| BC224F7 / BC23304 | Synchronous growth allocation route |5|208|144|
| BC224F7 / BC2270F | Requested growth worker |5|57|9|
| B03AFC7 / B032C2F | Separate game route; code unavailable in saved slice |1|64|0|
| MIXED | No uniform creation/caller pair for the whole leaf |12|351.82421875|220.48828125|

An independent review confirmed all50 joins, exact byte sums, the caller return
offsets and the single aggregation-key change. It did not establish named-pool
types, live/free occupancy or identity continuity over the65-second gap.
The144MiB synchronous-growth correlation is a research lead, **not144MiB of
spare/free memory or a savings forecast**. Initial capacity with360MiB commitment
but8MiB Private WS also illustrates why nominal capacity alone cannot rank a
RAM intervention.

Saved native slices establish a real growth chain: initializer BC22120 stores
size/alignment/count configuration, creates initial heaps via BC22490, appends
metadata and accounts capacity. Allocation paths BC22840/BC23200 query fit
through uncaptured BC23B20, and can synchronously create a heap sized for the
configured amount or aligned request. Worker BC22660 responds to a growth
request flag, creates a configured heap and appends it under synchronization.
The captured worker has no shrink branch.

The missing normal path is specific:
caller release → all live/recorded references retired → all actual GPU consumers
complete → wholly unused classification → native unlink/destruction/accounting.
BC23C50 is reachable constructor-failure cleanup, not demonstrated normal
retirement. BC23D80 and BC25CE0 initialize metadata/containers; short cleanup
slices do not supply the missing lifecycle. A ninth initializer argument is
stored at owner+15C but has no demonstrated consumer in these slices: its role
must remain UNKNOWN, not be renamed a reserve threshold and tuned.
BC23B20, BC23EB0, BC25960 and BC25FC0 are uncaptured boundaries. Re-reading the
same growth code cannot fill them.

### Competing CPU allocation lead

To avoid preselecting graphics, the same method covered the32 largest resident
fully committed heap leaves:324.6484375MiB committed and116.26171875MiB Private WS.
The entire3493-leaf heap category was581.71484375MiB committed and
296.7578125MiB Private WS; this32-leaf sample is **partial**.

| Uniform closest game return RVA | Leaf count | Committed MiB | Private WS MiB |
| --- | ---: | ---: | ---: |
| B05B30E |1|67.4375|58.45703125|
| 88900FA |1|198.84375|2.4375|
| 73C1F |1|2.51953125|2.51953125|
| MIXED |29|55.84765625|52.84765625|

Already-saved B05B2C0 code requests0x436F090 bytes
(70,709,392bytes;67.43373107910156MiB) through an allocator interface; its return
offset matches B05B30E. The request is slightly smaller than the rounded committed
range and is not an exact resident-byte count. It then calls uncaptured B024D10,
assigns a vtable and initializes fields at large offsets within the object.

**Follow-up source-only decision at parent f913d1cded8e166393b39f4314c10b5f29a83587:**
close the allocation-size patch / drop-in BGFX replacement / assumed configuration
toggle routes. Keep owner-side demand allocation as a concrete algorithmic
candidate, with Education integration UNKNOWN. This is a newly resolved
integration boundary, not implementation or Minecraft savings.

#### What the saved constructor actually covers

The captured function creates one object, calls an uncaptured initializer and
continues member initialization. Its first direct post-call store is at
object+0x3C56408. A later loop advances by0x168 between0x418E8B8 and0x436EF48,
performing direct object-relative stores. A following call receives
object+0x436EDF0, zero and length0x298, ending eight bytes before the requested
allocation end if interpreted as a zero-fill operation. That helper's body is
not captured: the argument shape alone is not independent proof of its behavior.
The preceding direct stores already establish use of high object offsets.
There is no demonstrated unused tail that can be removed by lowering malloc's
requested size. The uncaptured initializer prevents attribution of most earlier
object fields; do not label them textures, terrain, cache or duplication.

On initialization failure the saved factory invokes a virtual cleanup-like
method and returns the allocation through the same allocator interface with
size0, then clears its context pointers. That is a failure path; it does not
establish normal resource retirement or reconstruction. The caller slice
8881B20 includes the literal0x78666762 (ASCII bgfx) in its renderer selection
path. Combined with the stack route this supports a BGFX-family comparison,
**not** a dependency revision, matching upstream class layout or plugin ABI.

The separate saved symbol-stack aggregation has3804rows,3504containing game
RVA frames and zero matching named game frames in the reader's
`minecraft.windows.exe!symbol` format. This is a coverage result for that
artifact, not proof that no symbols exist anywhere. It supplies no content
identity/live-object census for declaring the large object or other heap
ranges redundant. The58.457MiB historical Private WS correlation remains
subject to the known trace/snapshot gap.

#### A closer native comparator: demand storage in BGFX

Mojang's A.J. Fairfield describes RenderDragon above BGFX, with gameplay
submissions copied into a per-frame registry and opaque resource handles whose
lifetimes/barriers belong to the rendering layer. The platform inventory
includes Education, but does not specify its binary layout or extension ABI.
This is relevant architectural authority; it does not identify the saved
object's members or unused capacity.
[Primary Mojang GDC2026 presentation, PDF pages7/10](https://media.gdcvault.com/gdc2026/Slides/Fairfield_AJ_ModernizingTheRenderingOfMinecraft.pdf#page=10).

Upstream BGFX has actual memory-demand changes, independently pinned rather
than inferred from the Education executable:

| Immutable upstream change | UTC author date | Source mechanism |
| --- | --- | --- |
| [86ad82c / #3790](https://github.com/bkaradzic/bgfx/commit/86ad82cfe2f5c2b8f56db2391a5ba5859251dad9) |2026-06-28|Dynamic draw/frame and matrix capacity; changes initialization, definitions/bindings and consumers. |
| [35a98dd / #3900](https://github.com/bkaradzic/bgfx/commit/35a98dd6453cf25dc75c68e233abb400836d5920) |2026-08-17|Per-frame storage grows on demand instead of reserving maximum representation. |
| [01e69a6 / #4024](https://github.com/bkaradzic/bgfx/commit/01e69a63074d4c103573ef8bc684ceec7e00b22e) |2026-10-04|HandleArenaT resource tables; alloc/access/release changes in core and six backends. |
| [e13b87c / #4025](https://github.com/bkaradzic/bgfx/commit/e13b87cdd9f169565fb2fefd0413890bcbafbb7d) |2026-10-05|Handle allocators, query rings and free queues scale with use. |

Inspected source pin:
`0c595f8e12c1f0db0653922c8c838ad74976af0f`. This is the upstream comparison
revision, **not the Education dependency version**. Commit metadata/patches
were read; neither upstream merge status nor a PR description is counted as a
passed test, measured benefit or Education rollout. No numerical saving is
extrapolated.

| Owner | Actual public mechanism | Integration implication |
| --- | --- | --- |
| [HandleArenaT](https://github.com/bkaradzic/bgfx/blob/0c595f8e12c1f0db0653922c8c838ad74976af0f/src/bgfx_p.h#L3319) | Allocates64-entry blocks on demand, tracks live entries/touched state, and releases eligible empty blocks through its own lifetime operations. | The consumers must index through the arena; a foreign allocator cannot redirect existing fixed-offset fields or manufacture live counts. |
| [Frame capacity owner](https://github.com/bkaradzic/bgfx/blob/0c595f8e12c1f0db0653922c8c838ad74976af0f/src/bgfx_p.h#L4283) | Observes peaks and adjusts capacity before frame counters reset; unused encoder uniform buffers are created lazily. | Requires native frame/use boundaries, not just Present pacing. Allocation/growth overhead and contiguous matrix growth can affect smoothness. |
| [D3D12 resource owner](https://github.com/bkaradzic/bgfx/blob/0c595f8e12c1f0db0653922c8c838ad74976af0f/src/renderer_d3d12.cpp#L9771) | Resource destroy/release bookkeeping precedes reclaim of unused CPU arena blocks. Context creation/deletion owns the whole renderer. | Empty CPU metadata blocks are distinguished from GPU resources; the existing game contract must be preserved. |
| [Compile-time configuration](https://github.com/bkaradzic/bgfx/blob/0c595f8e12c1f0db0653922c8c838ad74976af0f/src/config.h#L297) and [Init::Limits](https://github.com/bkaradzic/bgfx/blob/0c595f8e12c1f0db0653922c8c838ad74976af0f/include/bgfx/bgfx.h#L752) | Dynamic mode changes representation/accessors. Initialization controls reservations and peak windows; static-mode draw count becomes a hard cap that drops excess submissions. | A prebuilt Education binary cannot acquire this representation from an environment define. No compatible Education Init bridge or live trim setter is established. |

The native optimizer idea is now concrete: replace eagerly resident maximum
tables with demand representation **at their actual owner**, while maintaining
all access, resource release and reset paths. It is not a Sodium binary port,
a copied BGFX backend, a global purge or a smaller allocation constant. A valid
original implementation still needs the corresponding Education consumers and
lifetimes. The saved constructor and factory do not provide an independently
lifetimed table/accessor seam; the specific shortcut is closed rather than
converted into another generic fixture.

For this decision the delegated primary-source note has SHA-256
`a0d08d76fa3fdb0e9fbf41e58ab958d297e808731ff997fcfb6559eb0703c339`.
The additionally reviewed8881B20 saved object has SHA-256
`7df1dc83998bba855f5d2c43cdd2338ac9fcbbf335ac1f7f7c8e97ca923d75b1`.
Raw code remains ignored locally. No new PC/game access, capture, product build,
test fixture, ABI change or runtime intervention was performed.

### Candidate decision ledger and current implementation boundary

| Waste hypothesis | Education evidence now available | Missing fact / compatible intervention boundary | Decision |
| --- | --- | --- | --- |
| CPU representation or duplication, compared with FerriteCore and BGFX demand storage | Large B05B30E object correlates with58.457MiB resident; high-offset constructor accesses and a BGFX-family caller are saved | Object layout, actual redundant/live data, all readers/writers and compatible native owner remain missing; upstream demands representation/access/release changes | Direct size/config/drop-in replacement routes CLOSED; owner-side demand allocation remains unintegrated, no saving estimate |
| Allocator free retention / fragmentation, inspired by production allocators and Mesh | CRT ancestry appears in saved heap commit stacks; commitment can greatly exceed residency | Actual free slots/pages, allocation/free ownership, aliases, allocator contract and Windows-compatible reclaim/compaction | Prior one-shot LFH result is not durable resident saving; no additional free-byte proof or justified automatic purge/working-set trim |
| Content/model/cache demand, inspired by ModernFix | No content-level live/unused or duplicated-byte census in these captures | Asset identity, consumers, reference lifetime, reload capability and duplicated resident size | UNKNOWN; no cache deletion, resource sharing or unsupported mod |
| Graphics growth/retention, compared with Filament/Godot/O3DE/DXVK | Concrete create/fit/grow chain and144MiB resident correlation for one captured synchronous route | Normal free/reference/GPU completion/unlink/rebuild chain and genuinely unused resident capacity | Keep candidate unselected; no retirement hook or startup-count rerun |

A and C keep their existing owners; B does not get a second policy/governor,
collector, generic sandbox or speculative release hook. No winner is established
among these hypotheses. New game-side saving, inactive RAM benefit, workload
tradeoffs and unchanged multiplayer/gameplay on a changed implementation are
**NOT_EXECUTED**. Prior runtime evidence belongs to its old payload; this work
does not promote it into acceptance for an optimizer that has not been built.

### Local provenance and reproducibility boundary

Raw code fragments, trace/range addresses and captures remain ignored locally.
Only semantic conclusions, code RVAs, aggregate metrics and SHA-256 identifiers
are recorded here. No credentials, account/world contents or capability values
were accessed or published.

| Saved input or derived local artifact | SHA-256 |
| --- | --- |
| cold-start allocation.etl | b9d7478705aab7eee8bb0febe3cabde0cf0129d38da02fe4f074201c6b2a2adb |
| guest-menu.csv | 4e37fce8c5d97a662e8980155769ecafbfe1b2bce4e2637f14ebd71c2ec99319 |
| wc-ranges.json | 7d532483eb447d1d44687569ccb4052d24a38f5c574d0608d7ccea069085ee9f |
| offline per-range reader | 31e4cc71fac9fabb6f67118fccc98d05aab8199471e64c401c8c925439714c67 |
| wc-owners-by-range.json | 8dc54e87675a18534e60e8f36461691484aeb4b391b00d83ed075a6a5d3cf935 |
| wc-caller-ledger.json | 9edf8de1adda114bd5730528fb872db49b99c77e1fc672c131f4b5f6e3030a9d |
| heap-top-resident-ranges.json | e0ec0c25ee7c3c76cdf67fa7de8cc7c0a99e74c09b05b0723b9c3985fb0eead0 |
| heap-top-owners.json | 6e6b14dd1fca620e7ebb34e425d8f183ed388e812133f6093e6c50084591483b |
| heap-resident-ledger.json | e9b23e75f404bbf67bbdf0537cfc70a58094733fd51cffabc3f6358cd1c2a261 |
| B05B2C0 saved object | bc7af473e88d34afeaf6d805bdb7f16bc8845e2686005a5760c1795d73e83e5f |
| independent allocator-ownership review | 2bbf1d231253a00dde65eb396555017feb890072210b8bc1dfabc58cc80a2a63 |

## Critical comparative audit — 25 October 2026

**Decision authority:** this section supersedes the earlier staging-first priority
and conditional-design claims at 8095e337aef9a1277b0a6c5447c6752343afa0e8.
The prior versions remain in Git history. The operator requested a deeper,
self-critical comparison before further experiments. No game/process access,
new trace, UI action, build, benchmark, hook or runtime change occurred in this
audit. This is primary-source/code review, not new Minecraft acceptance.

### Audit of our own reasoning

| Prior statement or decision | Verdict | Correction and reason |
| --- | --- | --- |
| Unused CPU-visible staging retention is the main/best RAM solution | **WITHDRAWN** | A known safe pattern in another engine establishes a possible algorithm, not the size/existence of Education waste or its priority over CPU objects, fragmentation, models and driver allocations. No comparative reclaimable-byte evidence selected a winner. |
| The conditional lifecycle design is complete enough to suggest implementation progress | **OVERSTATED** | It describes safety requirements for one hypothesis. Education's producer, references, recorded commands, queue timelines, wholly unused state and native destruction path remain unproven. A checklist is not an integrated design. |
| The warm-pool comparison lowered commitment by174/142MiB | **OBSERVATION ONLY** | Those are differences in sequential readings with different join timing/allocator activity; no matched repeated intervention establishes causality. The effectively unchanged private working set justifies withholding rollout, but does not disprove every pressure-dependent benefit of the mechanism. |
| Named UploadBufferHeap has2x8MiB initial requests, so another startup-count canary is unjustified | **KEEP, NARROWLY** | The direct nominal count2->1 difference is8MiB. This neither caps later growth nor rejects all upload/graphics/allocator optimization. Actual properties and retained resident capacity at this site remain UNKNOWN. |
| Whole-unused-page destruction is the only safe native route | **NOT A UNIVERSAL CLAIM** | It is a conservative condition for our currently unverified foreign-resource integration. Mesh demonstrates physical compaction without changing live object virtual addresses under its own allocator/OS contract. That does not authorize remapping Minecraft memory. |
| Periodic/background cleanup should be rejected | **QUALIFY** | Owner-cooperative purge/decay is standard in production allocators. Reject blind working-set trim, raw decommit, foreign COM release and an unjustified new cleanup loop; do not conflate them with a proven allocator-owned reclaim mechanism. |
| Low background FPS should mean low RAM | **UNPROVEN** | Presentation frequency, simulation/upload jobs, retained caches and residency are distinct. Frame-count cleanup can even take longer in wall-clock time at10FPS. Education-specific behavior is not established. |
| An internal optimizer is possible because we can inject a DLL | **WITHDRAWN AS FEASIBILITY PROOF** | Loading/interception is a mechanism for integration, not proof of ownership, useful algorithm coverage, correctness or savings. Conversely closed source alone does not prove impossibility: mature translation layers can own an implemented API boundary. |
| More documentation and passing controls advance the RAM goal | **METHOD ERROR** | They can preserve knowledge and verify contracts, but are not the requested savings. The preceding work over-invested in one lead and repeatedly restated UNKNOWN instead of comparing competing mechanisms and their evidence. |

The main logical failure was promoting *algorithm plausibility* into *problem
diagnosis and priority*. We should have required a demonstrated waste class and
an applicable boundary before treating an implementation as the expected answer.
Version hashes and a correct call ABI cannot substitute for that missing causal
argument. Refusing unsafe frees was justified; suggesting that this left one
obvious main solution was not.

### Comparator sample: actual mechanisms, rather than a list of project names

The sample covers14 project/system families below. It is purposeful rather than
exhaustive, and does not benchmark any whole engine as globally superior.
Immutable code pins identify inspected implementations; release/blog/issue claims
are separately labeled empirical evidence. Their algorithms or published results
do not imply matching Education gains.

| Comparator and exact source | Mechanism in inspected code | What it teaches / integration boundary |
| --- | --- | --- |
| [Sodium148151c, CompactChunkVertex](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/render/chunk/vertex/format/impl/CompactChunkVertex.java) and [ArenaAggregator](https://github.com/CaffeineMC/sodium/blob/148151cf25c56f8b670986b348dcc589278c867e/common/src/main/java/net/caffeinemc/mods/sodium/client/gpu/arena/ArenaAggregator.java) |20-byte terrain vertices; arena reuse/relocation; deliberate spare capacity, growth-sensitive deallocation pause. Some sizing branches overallocate substantially; one resize-compaction branch is disabled because of churn. | Smaller representation, fewer redundant allocations and retained spare capacity can coexist. Requires the mesh producer and GPU decoding/relocation contract; allocator interception supplies neither. |
| [FerriteCore0cef1f2, FastMap](https://github.com/malte0811/FerriteCore/blob/0cef1f2add1f1329aa6e690e8e292acd625c5c6d/Common/src/main/java/malte0811/ferritecore/fastmap/FastMap.java), [BlockStateCacheImpl](https://github.com/malte0811/FerriteCore/blob/0cef1f2add1f1329aa6e690e8e292acd625c5c6d/Common/src/main/java/malte0811/ferritecore/impl/BlockStateCacheImpl.java) | Shared indexed state representation and canonical collision-shape/cache data replace repeated Java structures. | CPU live-data reduction, not generic heap purge. Needs actual duplicate immutable Education representations and all consumers. C++ Education may already avoid the Java overhead. |
| [ModernFix441fe62, DynamicBakedModelProvider](https://github.com/embeddedt/ModernFix/blob/441fe62210157ae62e925eaa2ea95c43652836ad/src/main/java/org/embeddedt/modernfix/dynamicresources/DynamicBakedModelProvider.java), [ModelBakeryMixin](https://github.com/embeddedt/ModernFix/blob/441fe62210157ae62e925eaa2ea95c43652836ad/src/main/java/org/embeddedt/modernfix/common/mixin/perf/dynamic_resources/ModelBakeryMixin.java) | Eager model loading becomes on-demand baking with dependency handling, bounded/expiring caches and permanent overrides. | Avoid loading dispensable data; requires registry, reconstruction and invalidation. Compatibility risk is real; not a drop-in Education extension. |
| [Godot c8256a8, RenderingDevice staging](https://github.com/godotengine/godot/blob/c8256a8ecec068340753869e423dd478c84ef167/servers/rendering/rendering_device.cpp#L963) | Persistently mapped CPU staging blocks, aligned suballocation, old-frame reuse, growth to a bound, then previous-frame/all-frame stall. | Deliberate retention trades footprint for upload latency. A mature implementation need not immediately shrink. Source establishes policy, not measured whole-process savings. |
| [Filament34388e7, VulkanStagePool](https://github.com/google/filament/blob/34388e76bec43a0e031b9722c46bf94fcb580214/filament/backend/src/vulkan/VulkanStagePool.cpp#L266) and [command completion](https://github.com/google/filament/blob/34388e76bec43a0e031b9722c46bf94fcb580214/filament/backend/src/vulkan/VulkanCommands.cpp#L299) | CPU_ONLY staging segments have owned references; completed commands release them. Cleanup destroys excess safe empty stages and retains one; staging images expire separately. | Concrete release/reuse design with actual lifecycle ownership. Retaining one stage is not a byte cap; it can be oversized. Vulkan ownership does not become an Education D3D12 seam automatically. |
| [O3DE2462b90, DX12 staging](https://github.com/o3de/o3de/blob/2462b90ee389939336931a14e234fc4e3b4cd534/Gems/Atom/RHI/DX12/Code/Source/RHI/StagingMemoryAllocator.cpp#L20) and [streaming images](https://github.com/o3de/o3de/blob/2462b90ee389939336931a14e234fc4e3b4cd534/Gems/Atom/RHI/DX12/Code/Source/RHI/StreamingImagePool.cpp#L405) | Ordinary medium/large staging pages set recycleOnCollect=true; oversized uploads have deferred release. Image streaming coordinates mip/tile trim with upload/frame dependencies. | Size classes can need different policies. Asset streaming is a separate lever, usually VRAM/quality related. GarbageCollect does not necessarily free all cached pages. |
| [DirectXTK12 f5b9284, LinearAllocator](https://github.com/microsoft/DirectXTK12/blob/f5b928460d91984b27029d064ee5a7dd4301513c/Src/LinearAllocator.cpp#L132) | Own references -> pending fence -> reusable pages; Shrink frees unused pages. | Establishes one cooperative lifetime model. No evidence Education embeds it or has callable equivalent maintenance. Own page RefCount must not be confused with external COM probing. |
| [mimalloc3.5.3/d4881d3, arena purge](https://github.com/microsoft/mimalloc/blob/d4881d338125e1cb7c47ba4cfb398d6f7c0c8d45/src/arena.c#L2277) | Local free-list sharding and delayed purge; eligible free slices are claimed before backing is purged, skipped if reused. | Allocation cost and owned-free retention can improve independently of live-data size. CRT/custom allocator/driver coverage and early compatible routing are unproven in Education. |
| [jemalloc5.3.0/54eaed1, pac decay](https://github.com/jemalloc/jemalloc/blob/54eaed1d8b56b1aa528be3bdd1877e59c56fa90c/src/pac.c#L340) and [tuning](https://github.com/jemalloc/jemalloc/blob/54eaed1d8b56b1aa528be3bdd1877e59c56fa90c/TUNING.md) | Dirty/muzzy extent caches, decay/purge, thread-cache/arena tradeoffs. | More aggressive release costs CPU; more arenas may reduce contention but increase fragmentation. No universal best preset and no safe foreign-heap tuning inferred. |
| [PartitionAlloc617efbf, MemoryReclaimer](https://github.com/chromium/chromium/blob/617efbff7213bad01531786b252a16e0e55a22ff/base/allocator/partition_allocator/src/partition_alloc/memory_reclaimer.cc#L63) | Owned span lifecycle, decommit and bounded reclaim, with aggressive thread-cache purge reserved for stronger pressure. | Cooperative periodic reclaim is legitimate. Chromium owns allocator metadata and embedding policy; the launcher cannot apply these lists to unknown pages. Current code is not the exact2021M89 payload. |
| [Mesh fa1542b, meshLocked](https://github.com/plasma-umass/Mesh/blob/fa1542b26468236b6c5589ceb46a8323bc11a9d2/src/global_heap_impl.h#L296) | Occupancy metadata and virtual-memory aliasing combine compatible partially occupied spans without moving live object virtual addresses. | Counterexample to a universal whole-empty-page restriction. Requires its own allocator, synchronization and OS mapping support; current [platform scope](https://github.com/plasma-umass/Mesh/blob/fa1542b26468236b6c5589ceb46a8323bc11a9d2/README.md) is Linux/macOS with Windows work in progress. Not game/D3D12 remapping permission. |
| [DXVK cbd2206, freeEmptyChunksInPool](https://github.com/doitsujin/dxvk/blob/cbd2206d72d3d9dd9941d5d868bb03a858e85f9b/src/dxvk/dxvk_memory.cpp#L1748) | Full graphics translation owner tracks allocated/used chunks, budgets, fragmentation and empty age. Mapped pools retain more spare capacity than device pools; cleanup is pressure/age dependent. | An external compatibility layer can optimize memory when it implements the relevant API/resource lifetime. This is substantially more ownership than our Present/creation wrapper and is not a ready D3D12 Education fix. |
| [vkd3d-proton ef9a02e, vkd3d_free_memory](https://github.com/HansKristian-Work/vkd3d-proton/blob/ef9a02e873715c9e2da0d6db8a236aca8380b303/libs/vkd3d/memory.c#L2087) | D3D12-to-Vulkan implementation merges suballocation free ranges, removes wholly free chunks under allocator lock, and respects relevant transfer dependencies. | Closed game source does not rule out API-layer optimization. But implementing a translation/resource owner, shader/descriptor compatibility and game support is a major separate undertaking; adopting it is not authorized by this audit. |
| [APEX Engine / Avalanche-AMD2022 case study](https://gpuopen.com/download/MemoryManagementInAPEX_v2.pdf) | Separates resource classes, controls allocation granularity/priorities, handles resize spikes and fragmentation with engine-owned resource references. | Production evidence concerns specific VRAM pathologies, not all game RAM. The lesson is diagnosis-specific policy, not one universally superior allocator. |

Pins are inspection snapshots, not assertions of latest versions or independent
performance superiority. Numeric constants above describe their sources; none
has been selected as an Education threshold.

### Published samples: distinguish data quality and metrics

All figures below are externally reported, not reproduced in this audit.
None uses our installed Education workload/hardware/settings.

| Sample and primary evidence | What is actually reported | Quality, scope and limitation |
| --- | --- | --- |
| [O3DE issue11446](https://github.com/o3de/o3de/issues/11446) and [inspected DX12 patch8650028](https://github.com/cyanide-studio/o3de/commit/8650028b183ff858a6917029d5ca8b5683460107) | Author reports scene VRAM roughly6300->5900MB DX12 and a2GB Vulkan reduction. Tiny unique allocations/alignment motivated changed suballocation/routing. | HWiNFO/patch author evidence, not independent replication/current RAM benchmark. Vulkan baselines are inconsistent:8700 total minus2300 pre-editor is6400, while the reported5900 subtracts2800 editor usage. Do not treat its2GB headline as a clean same-baseline delta. Driver crashes were also reported; no Education forecast. |
| [DXVK2.5 release](https://github.com/doitsujin/dxvk/releases/tag/v2.5) | Authors report peak video-memory reduction up to1GiB in extreme God of War cases after memory-management rewrite. | Release claim with workload context, no full protocol/variance in the note. VRAM, not private RAM. Retains spare capacity and notes driver exceptions; copying the peak figure would be misleading. |
| [ModernFix Dynamic Resources FAQ](https://github.com/embeddedt/ModernFix/wiki/Dynamic-Resources-FAQ) | BlanketCon23 example1.7GB->945MB, with FerriteCore already installed. | Historical author example; hardware/JVM/stage/repetition/metric not fully specified. Its stated55% reduction conflicts with endpoints: consistent decimal units imply about44% reduction/56% remaining. Feature is off by default for compatibility, including startup crashes/missing textures. |
| [FerriteCore pinned summary](https://github.com/malte0811/FerriteCore/blob/0cef1f2add1f1329aa6e690e8e292acd625c5c6d/summary.md) | One old modpack's repeated neighbour maps around600MB versus roughly7MB replacement FastMaps. | A specific Java representation, not whole-process resident RAM. Historical2020/MC1.16.4 examples, some difficult to reproduce; do not sum overlapping entries or transfer the ratio to native data. |
| [Chromium M89 official report](https://blog.chromium.org/2021/03/advanced-memory-management-and-more.html) | Windows savings up to22% browser,8% renderer,3% GPU process after allocator adoption; separate discardable-content change up to100MiB/tab. | Anonymous aggregate product field data; no full raw A/B dataset/variance provided. Per-process figures are not additive;2021 browser result does not establish a modern game allocator gain. |
| [Mesh PLDI2019](https://people.cs.umass.edu/~mcgregor/papers/19-pldi.pdf) and [original artifact322e727](https://github.com/plasma-umass/mesh-testsuite/blob/322e727877d25c8fbdb30eee8fae960b48c2ffa1/README.md#evaluating-paper-claims) | Paper reports Firefox16% and Redis39%; artifact's Firefox values530MB sigma22.4 vs632MB sigma25.3. | Paper plus evaluation scripts/environment is stronger reproducibility evidence. Historical external trials; architecture-dependent results and Docker/macOS slowdown caveats exist. Windows/Education/D3D12 integration remains absent. |
| [mimalloc benchmark documentation](https://github.com/microsoft/mimalloc/blob/d4881d338125e1cb7c47ba4cfb398d6f7c0c8d45/readme.md#performance) | Time/peak-RSS results with described Linux hardware/compiler/allocator versions and repeated timing runs. | Benchmark suite dates to2021; current README warns about age and speed-dependent RSS comparisons such as xmalloc-testN. Faster allocator does not imply less live data or lower game RAM. |
| [O3DE allocator discussion19539](https://github.com/o3de/o3de/discussions/19539) | Author reports mimalloc high-water12.5GB versus malloc10.1GB, while one frame-time comparison is10.7ms versus12.6ms. | Workload/allocator-combination report, not precisely specified private-resident metric or controlled repeated trial; author notes drift and crashes in a combination. Valuable counterexample to automatic speed-plus-RAM improvement, not a universal winner. |

FerriteCore's owner explicitly distinguishes Java GC baseline from working set
and warns that modern vanilla improvements and ModernFix overlap reduce or
overlap gains: [issue242](https://github.com/malte0811/FerriteCore/issues/242).
The ModernFix ultra-low-memory [demonstration](https://github.com/embeddedt/ModernFix/wiki/Ultra%E2%80%90Low-Memory-Footprint)
uses a constrained Java heap and much lower render distance; it does not meet
our12-chunk/native-process scope.

### What these comparisons change for Education

There are at least four distinct potential causes of avoidable footprint:
(1) unnecessarily large/duplicate live CPU data, (2) allocator fragmentation and
retained free backing, (3) dispensable assets/models/decoded copies, and
(4) graphics allocation granularity, upload lifetime and driver backing.
A process-total or VMMap category does not choose between them. Examples show
different solutions for each, not that Education actually has all these defects.

Existing baseline category readings (~601/754MiB Heap Private WS for guest/host,
~439/440MiB Private Data, ~293MiB Image Private WS each) are **not semantic
owner/liveness measurements**. Private working set covers pageable private
process residency; it is not a complete attribution of shared/system/driver RAM
or VRAM. GPU-stack address correlation does not label all those bytes upload or
unused. Live bytes, internal/external fragmentation, reusable spare capacity and
driver commitments must remain distinct.

| Candidate family | What must be true for it to help | Present Education evidence / decision |
| --- | --- | --- |
| CPU representation/dedup | Large repeated immutable structures exist; all readers/writers and identity rules are known | Heap is material, but representations/duplication are UNKNOWN. No selection or savings estimate. |
| Allocator retention/fragmentation | Meaningful free capacity retains resident backing; owning allocator can purge/compact/reacquire compatibly | LFH one-shot observations are modest/transient. Custom ownership, waste and replacement coverage UNKNOWN. No global malloc switch. |
| Demand loading/content cache | Reconstructible data is loaded/retained without current need; rebuild/invalidation is available at acceptable latency | No demonstrated Education redundant model/cache or valid eviction seam. Source analogy only. |
| Staging/post-growth retention | CPU-visible resident backing is materially unused after all actual consumers complete | Named initializer and generic growth code are known; type, occupancy, native retirement and reclaimable bytes UNKNOWN. One hypothesis, not main solution. |
| GPU granularity/streaming | Allocation alignment, duplication or mip demand wastes the relevant budget | L1 startup candidate showed no meaningful resident-RAM result. VRAM benefit, wider pathology and safe integration remain UNKNOWN. Quality effects must be attributed separately. |

No candidate currently combines demonstrated Education waste volume, compatible
ownership and predictable benefit. Therefore no evidence-based winner or
resident-RAM saving percentage is available. Below1GB remains aspirational,
not a proof-backed engineering estimate. This is an evidence limit, not proof
that an optimizer can never be developed.

For background clients, separate **presentation**, **simulation/network**,
**asset demand** and **retention maintenance**. A frame-age policy can take14.4x
as long in wall time at10FPS versus144FPS for the same frame count; this is
arithmetic about a hypothetical policy, not observed Education behavior.
Browser content discarding requires reconstructible content; freezing/discarding
an Education host would violate continuing gameplay/network requirements.
Release of legitimately owned free pages can be useful, but repeated churn and
recovery latency must be part of the objective. Do not optimize the RAM column
while moving cost to faults, loading, VRAM or stalls.

### Corrected decision and concrete research contract

1. Preserve A graphics persistence, existing144/10 focus behavior, original
   accounts/worlds, minimum12 chunks, unchanged simulation/resolution, Local only
   and CI disabled. Keep the L1 experiment default OFF and five clients deferred.
2. Withdraw staging-first priority. Keep the native retirement checklist as a
   conditional safety model for that candidate; it is not a complete Education
   design or a universal description of all valid memory optimizers.
3. Future selection must connect **observed waste -> semantic owner -> applicable
   mechanism -> supported lifetime/rebuild boundary -> relevant byte budget ->
   latency/correctness tradeoff**. A literature example supplies the mechanism,
   not the other links. Use existing captures/source first; do not start another
   canary simply because a familiar algorithm exists.
4. For each candidate distinguish code-derived capacity, author-reported results,
   reproducible published artifacts and actually observed Education evidence.
   No additive cross-project percentages, Java-heap/RSS conflation, nominal
   capacity-as-RAM claim or blanket no-native-optimization assertion.
5. No runtime/code experiment follows from this audit. Further source work should
   compare the four waste classes using available evidence before narrowing to
   one integration. If the necessary owner/live-free distinction is not in the
   captured evidence, state that exact missing fact and stop that selection.

The useful outcome of this audit is a corrected decision plus inspectable
counterexamples and evidence, not a new promise that one more hook will solve
the RAM goal. Raw private artifacts remain local; only this report and the
canonical handoff change.


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
compatible. See the [collector usage and proof boundary](../../Development/tooling/acceptance/windows/README.md#opt-in-two-client-memory-composition).

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

The [implementation playbook](native-efficiency-implementation-playbook.md) provides concrete Codex-ready mechanisms N01-N04 (registry/file hooks, expired startup tracing, event-led manager), R01-R03 (pacer timers, HUD, presentation), M01-M02 (owned immutable data), B01-B02 (Sodium/FerriteCore internal game feasibility) and W01 (conditional content tooling). Each has exact source owner, Windows/C++/Rust API, algorithm, safety/compatibility boundary and primary reference. This is **supporting technical design**, not new deployment authorization, an automatic optimization claim, or a second active TODO owner. The A/B/C current goal and **one next action** remain in [planning](../../planning/development.md).


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

A one-shot original action now exists in [heap-reclaim.cpp](../../Development/tooling/acceptance/windows/heap-reclaim.cpp);
its opt-in build, identity guards, uncertain-thread handling and limitations are
documented in the [Windows tooling guide](../../Development/tooling/acceptance/windows/README.md#explicit-one-shot-lfh-reclamation-experiment).
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


## Identified native preallocation seam and original candidate

Read-only inspection of selected loaded executable-image functions, validated
against the original process births and installed executable SHA-256, identifies
an internal pool initialization route. The on-disk image alone did not expose
the relevant decoded code/exception table; no game file was modified. Only
bounded MEM_IMAGE code and specifically referenced read-only literal data were
read; heap/account/token/world contents were excluded. Raw fragments stay local.

The loaded initializer names five pools: DefaultBufferHeap, TextureBufferHeap,
UploadBufferHeap, ReadBackBufferHeap and AccelerationStructureBufferHeap.
For DefaultBufferHeap the observed call passes a 16-MiB heap size, 64-KiB
allocation alignment and initial count 16. The initialization loop creates that
count. Separate inspected allocation paths grow by creating a heap when no
current heap fits, and a worker services growth requests. Four relevant loaded
functions match byte-for-byte across both original clients. These observations
identify a real startup-capacity/growth seam; they do not prove spare capacity
during a world or identify the purpose of every earlier graphics allocation.

The independently authored [warm-pool owner](../../Development/tooling/acceptance/windows/bootstrap-ordering/graphics-warm-pools.cpp)
and its header implement a build-opt-in candidate: reduce only the exact
DefaultBufferHeap initial tuple from sixteen heaps to four. It preserves native
growth and resource retirement, avoiding any need to declare a live GPU buffer
safe to free. Existing pacing-authorized factory enrollment is the only consumer;
there is no additional resource governor, recurring cleanup or public route.

Image metadata and four observed code hashes guard installation. The private
initializer wrapper also checks the observed caller address, literal/property
pointers, heap type and full parameter tuple. Mismatches preserve native
arguments. The DLL is pinned and failed thread/Detours transactions abort
installation. Unknown game versions remain unsupported; this is not a stable
Education extension API. The existing manager payload digest is unchanged.

The first source version's type-1 match did not apply in the actual game.
Both clients instead expose the exact properties [4,1,2,1,1]: CUSTOM heap,
CPU-page NOT_AVAILABLE, memory pool L1 and both node masks 1. The corrected
private policy checks every field and preserves all mismatches. Microsoft's
[L1 definition](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/ne-d3d12-d3d12_memory_pool)
describes the video-memory pool on discrete GPUs, while
[NOT_AVAILABLE](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/ne-d3d12-d3d12_cpu_page_property)
denotes CPU-inaccessible heap memory. Those properties do not identify the
driver's associated system-memory allocations as unused.

Both corrected enabled and disabled MSVC x64 Release /W4 /WX DLL builds passed.
The own-process controls passed exact/mismatched property tuples, own-image
rejection before private RVAs and handle balance. An enabled-only linker map
locates a private atomic action receipt, with no export/schema extension.
Each original client recorded one changed initialization returning native
success. This establishes actual private-ABI execution, not just API availability
or a loaded DLL; retained heap count and native growth under pressure were not
directly measured.

## Executed native startup-capacity comparison and decision

Source parent: `31e7527531c5d5d2ceb1952fbdbe8bed41435671`, plus the corrected
five-property policy/private receipt and ON-only map. Corrected DLL SHA-256
`bf6f5c1d189c84fcc4a4b36082ce4b73ed95433da084cb03012f87808bfbbdbd`.
Separate private manager SHA-256
`9436c0e85eba80faf5346dd47d0d737a229a94f72efb346f22dfa63e014aa1dc`.
Its only Rust source delta is the reviewed payload digest, not an override
argument or new production route. Canonical admission retains the prior working
payload. Original profile identity capability was reused; no account/world
directory or token was copied.

On 25 October 2026 both baseline and corrected canary automatically restored
their original accounts, opened the same existing MC02 Education Host World and
joined MC01. Existing Simple / 12-chunk settings and 144/10 ceilings were not
changed. Existing signed VMMap snapshots were used, with guest foreground and
host background; no WPR repetition, memory trim, extra action injection or
periodic sampler was introduced for this comparison.

| Metric, MiB | Guest baseline | Guest canary | Host baseline | Host canary |
| --- | ---: | ---: | ---: | ---: |
| VMMap Private WS | 1335.648 | 1330.410 | 1490.965 | 1495.809 |
| VMMap Total WS | 1421.523 | 1416.063 | 1576.852 | 1581.711 |
| VMMap Private commitment | 2404.125 | 2229.980 | 2555.324 | 2413.668 |

These metric columns have distinct meanings; commitment is not resident physical
RAM. Private commitment is lower by 174.145 / 141.656 MiB. Guest private physical
RAM is lower by 5.238 MiB while host is higher by 4.844 MiB; the combined
difference is only -0.395 MiB. **No meaningful resident-RAM reduction is
established.** These are bounded sequential observations, not a causal percentage,
a repeatability benchmark or a long-soak result. Elapsed time since confirmed
join is approximately 46 seconds for baseline and 13 seconds for the first
canary reading; natural allocation/chunk activity remains a confounder.
A later reading after host foreground activation gives Private WS
1330.934 / 1496.453 MiB; its changed focus condition excludes it from a matched
delta. Do not combine those snapshots or credit a working-set shift as freed
game resources.

Both clients remained in the shared world without an observed crash. The host
multiplayer panel showed two participants; after foreground resume its HUD
returned to approximately 135 FPS, while 10 FPS was observed in background.
Host Save & Exit completed through its UI, guest received the expected host-ended
disconnect, and both apps exited normally. This is bounded rendering/network/
save-path evidence, not pool-growth stress, measured frame latency, a persistent
FPS guarantee, post-canary restart proof or five-client acceptance.

Decision: preserve this original implementation as **default OFF, unpromoted for
resident-RAM optimization**. It reduces startup capacity and showed lower
commitment, but does not satisfy the requested physical-RAM outcome. The
theoretical 192-MiB capacity reduction concerns this one L1 pool; actual VRAM
benefit remains NOT_EXECUTED, and native demand may refill capacity. The operator
clarifies below 1 GB per game is aspirational, not a hard cap; it remains
NOT_ACHIEVED in this world. Preserve stability, accounts/worlds, minimum 12 chunks,
simulation/resolution and independent A/B/C ownership.

Local snapshot SHA-256 values:

| Snapshot | SHA-256 |
| --- | --- |
| Baseline guest | `0c708b1dc519da0b8e7087604fb291f29671b1f2800957f56204aec66219f41a` |
| Baseline host | `da87532ed164dc45fc0d4b42c0e79759ffef750e5b1fb17550d87abd165fc9b3` |
| Canary guest | `446b7fe04c904ce6b82f3a35fcc2056f7ecb4656afbcc694a6b208c0dc40f669` |
| Canary host | `c2abb828bc6103828b43bf5c41817d06cc635da9fd9e4b8eec6efdaca7aad5fb` |

Raw mappings, code fragments, profiles and captures remain local. The single
next intervention is owned by planning/development.md; the L1 result is not a
reason to repeat the same experiment or promise a Sodium-equivalent RAM floor.

## Native Upload empty-retention seam — authorized resume, 25 October 2026

This section supersedes the earlier no-live-access integration impasse for this
specific allocator route. The operator explicitly resumed the RAM goal with
full PC/runtime authorization. A/B/C remain separate; the latest working
144/10/HUD payload and original account/world profiles remain the baseline.
This is not a BGFX replacement or a predicted below-1GB result.

### Actual owner, retirement and completion evidence

The official Education executable SHA-256 remains
`04164c1c64458a5d984dbc1687d0e8023d1808c44ee7be03519cdf8355233eaa`,
timestamp `0x6a63e832`, image size `0x1255c000`. Selected bounded
loaded-code reads were necessary: relevant native ranges are not directly
file-backed in the installed executable. Raw instructions remain private.

Native initializer RVA BC22120 stores argument8 at owner+158 (initial count,
also the total-heap floor) and argument9 at +15C. Newly inspected normal
retirement BC24D10 resolves the latter as an empty-candidate count threshold:
after eligible deferred records retire, it collects heaps with metadata+E8=0,
requires candidate count above +15C and total count above +158, removes a
candidate, updates capacity/vector bookkeeping and invokes BC24570. That
release routine uses native locking and interface release calls. The sweep
requires newly processed pending records; an empty sample alone need not
cause immediate shrink. This corrects the earlier unresolved-argument finding.

A bounded observer actually ran on one original client: eight callback
invocations all returned to RVA B039C36. The callback caller B039BB0 queries
an interface at coordinator+18 via vtable+40, caches the returned value at
+30, treats UINT64_MAX specially, and dispatches registered native callbacks
with that value. The wait path B036C20 uses the same interface's vtable+48,
a requested value and event, waits, then queries again. The submit path
B02E8E0 passes that interface and the next value to a queue vtable+70 before
calling the dispatcher. These signatures/slots and the device-removed
sentinel match [ID3D12Fence::GetCompletedValue](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12fence-getcompletedvalue),
[SetEventOnCompletion](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12fence-seteventoncompletion)
and the installed Windows SDK vtable. Interface IID creation was not separately
verified. No completion value, pending record, resource counter or native
release argument was changed by the observer.

Private evidence:
- bounded native callback receipt SHA-256
  `8d9484ea1f6dd947e1fc5755da60e546e47835830b8788a685f9d32c5be3c818`;
- metadata sample SHA-256
  `0f5effd04e4cf5b13228d0db13c04da472283b0abb6b0aa827f8c773a9a2a95c`;
- observer source SHA-256
  `289ace1c9d4351a1e0bf0e90f78f67b10eefa60371c3ed1d6c80f96c73f82869`;
- observer DLL SHA-256
  `359be8345a272642909c0fce803697b058929ba237a5058703f5d168e2861a35`.

The observer preserves arguments and LastError, records at most eight calls,
and then passes through. Its hook/DLL is pinned until normal game exit; it
was not hot-unloaded. This is real native-callback evidence, not a savings
measurement or a generic allocator fixture.

The unsynchronized post-startup Upload sample showed five heaps, total
capacity168MiB, floor2, threshold4, with four zero-counter candidates totaling
88MiB capacity. Scalar/vector consistency was checked, but the sample does
not prove all candidates GPU-idle, physically resident or reclaimable.
The observed callbacks left this pool unchanged. No 88MiB saving is claimed.

### One independent implementation, default OFF

The existing graphics-warm-pools owner now has a separate
`BOOTSTRAP_UPLOAD_RETENTION_EXPERIMENT` build option. Exact native call return
B026D72, name pointer D0D5C64, properties pointer F544940 and the observed
UploadBufferHeap tuple (8MiB, alignment65536, flags0, initial2, reserve4) select
only initializer argument9, changed4→1. Initial2 remains untouched.
Existing image/code guards are extended with native retirement/release hashes:
BC24D10/2905bytes
`72b1c1c6202f7b39feab77f45b0114d38bf32a28f36a83e259782efa99d5091e`;
BC24570/172bytes
`4a004000f3dd483a8b7f7cce91eefe7241933703d2e8e4cfeda751aecfccca30`.

Startup initialization avoids a remote scalar write racing renderer teardown
or depending on a private mutex ABI. The native allocator still owns growth,
locks, deferred retirement and release. The old Default initial-count experiment
and new Upload retention experiment are mutually exclusive at CMake and C++
levels. Both default OFF. A private successful-initialization counter records
policy application only; it cannot certify native retirement or RAM benefit.
No public API, production admission, profile route, signed game file or CI
change is included.

MSVC19.44 /W4 /WX compiled and executed the exact policy/control sources in
Upload-enabled, legacy-enabled and both-disabled configurations. Controls
check selection/rejection and reject attachment to their non-game executable
without leaking handles. CMake with both experiments enabled was actually
rejected as intended. These are SOURCE_EXECUTION results; full compatibility
DLL/package construction and gameplay on the new policy are NOT_EXECUTED.

Risks remain allocation churn, refill stalls, no native sweep until pending
work retires, and a count threshold that is not a byte budget. Restoring the
baseline at normal restart restores future retention policy; already released
heaps are not instantly resurrected. Do not add a manual sweep, foreign free,
forced fence value, arbitrary decommit or working-set trim to make it appear
effective. Promotion requires comparable workload showing actual native
capacity reduction and durable resident-RAM improvement without frame/host/
save regressions. No B RAM saving is established yet.

## Upload retention live comparison — 25 October 2026

**Decision:** the independent4→1 Upload retention canary has now run on
official Education, and its intended native capacity reduction is observed.
A short same-world comparison also shows lower private working set and
commitment. Retain it as an opt-in candidate; this is not sustained RAM,
frame-time, multiplayer-host or production acceptance.

### Build and execution provenance

Compatibility sources were fetched at exact Local
`050c4fc9ff2d2371bd947b6188f89f6038ee204d`. Both complete compatibility DLLs
were built through the existing CMake/MSVC owner with /W4 /WX:
- Upload experiment ON, legacy Default experiment OFF:
  `72d004de09c64feded867f221a77465104826d9aef040a224c000b241761f1aa`;
- both experiments OFF, same source:
  `0473fc0b402d566318710caa0bf45b4e3047919eabac862cc57a372a0f4149c9`.

The configured Upload control actually built and ran. The unchanged reviewed
enroller SHA remains
`7b38a1c02f0d5169f74ebebc839d854d49e9a7e77b9933415b1ec8d5d539588a`.
Private preview managers were built from the existing cached manager base
506aef24b002204b6670d37280da0cadc24da6e9, with the exact current
independent_bootstrap.rs and only the matching private compatibility digest
substituted. These are not whole-repository exact-HEAD manager builds.
Candidate manager SHA:
`d6ae4e31de5807503216ddf209ae422114ebb194e41127efdf4ba4767cf0de59`;
baseline manager SHA:
`fa61102569112ca3805c8f80f8b8c8a81a693bcf28eead83b70a15bc80934e9b`.
Both offline locked release builds succeeded with104 existing compiler warnings;
do not report a warning-free Rust build. No canonical payload allowlist,
production admission, public API or CI setting changed.

Both original profiles were reopened through the unchanged216-byte identity
capability. No account/token/world content was copied. Automatic account
continuity and the HUD were visually observed in both candidate menus.
The prior observer DLL ended with the earlier normal process exit and was
not injected into either new candidate or comparison client.

### Observed effect and comparable-workload limit

The candidate's private counter reports one successful retention reduction
and zero initial-count reductions in each client. Metadata confirms initial
floor2 and empty threshold1. Both candidate clients had two Upload heaps
totaling88MiB after startup. The later MC01 world sample retained that same
2/88MiB state. The same-source baseline restored threshold4 and showed five
heaps/168MiB at startup and in the same world. The native allocator therefore
retained80MiB less Upload capacity on this candidate path. This is resource
capacity, not automatically80MiB less physical RAM.

The same original MC01 persisted Creative test world and gold marker were
opened normally. No movement, world edits, simulation/resolution/render-distance
change or new effects change was requested during the comparison. The world
was saved/exited normally between runs. Candidate ran first, baseline second;
time of day/entities/camera pitch and other allocator activity were not frozen.
The earlier long-lived old-payload sample is excluded from the numeric
comparison because process age/residency differed.

Three private-working-set/private-byte samples per variant were read through
Windows process performance metadata, about10.6–10.7seconds apart:

| Metric | Same-source baseline OFF | Upload candidate ON | Observed lower value |
| --- | ---: | ---: | ---: |
| Private working set, MiB, samples |1409.043 /1393.730 /1393.680|1364.770 /1333.770 /1333.777|—|
| Private working set median, MiB |1393.730|1333.777|59.953MiB (~4.30%)|
| Private commitment median, MiB |2556.914|2477.906|79.008MiB|
| Upload capacity, MiB |168|88|80MiB|
| Upload heap count / initial floor / threshold |5 /2 /4|2 /2 /1|native retention only|

This is one short sequential comparison, not randomized, repeated or durable
residency proof; warm-up durations were not identical. Other pools varied (e.g. Accel capacity); do not attribute
every total-process byte solely to Upload or extrapolate60MiB to all clients.
No CPU/GPU saving or frame-time improvement is established. HUD snapshots
around135–138FPS were observed on both foreground builds; they neither prove
144 sustained FPS nor absence of transient stalls. Background menu10FPS was
observed during a modal transition, not multiplayer-host-health acceptance.

The candidate opened the test world, continued rendering, saved/exited it,
and both candidate processes subsequently exited normally with journal
exitCode0. This is a bounded load/save/close result, not sustained gameplay,
world travel, allocation-churn or Education-host compatibility proof.
At the end of the comparison the nonexperimental same-source baseline remains
running: MC01 in its test world and MC02 at the menu. No experimental payload
is left active; accounts, worlds, working release candidate and private
rollback payloads are retained.

Private evidence SHA-256:
- candidate world pools:
  `d109352fca8cd742253c472834342e271cb1d3c0d6e8265f371b738f0a38b976`;
- candidate world memory:
  `ed0d04add9166212b8ecb12928d2e144444d782e417d59de02d0390a58da6c92`;
- baseline world pools:
  `6d30e9226243272e82df5d902e6615658da980d4b99441d3903156bd5030e55d`;
- baseline world memory:
  `65945b525197439a9976d2b8816291667aefeb33cf681604f6acf7b59eb30ce2`;
- derived comparison:
  `17256c8141c852fdae741c5346bdda793abd1733594b3ffd562103ec2cc7b524`.

**Next boundary:** one integrated candidate workload with Education host/join,
new-chunk demand, foreground/background changes and normal save/reopen must
decide refill-stall and host risk before any rollout. Do not rerun generic
traces or repeat this stationary comparison as a substitute. Below1GB and
additional background-specific RAM savings remain unproved.
