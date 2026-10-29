# Current Development Handoff — v0.1

## Current intent — 29 October 2026

**Goal (operator reprioritized 29 October 2026):** test **selective
Texture Lite 8×8** for **three mostly-idle profiles** while keeping **two
controlled profiles on normal vanilla textures**. The operator now explicitly
accepts lower 8×8 visual detail for those experimental idle profiles; this
supersedes the previous universal no-downscale rule **for this opt-in trial
only**, not the requirement to preserve gameplay/account/world data.
Retain optional Particle Tuning v0.3 separately. **STOP adding runtime
monitors, polling services, collectors, telemetry, or benchmark micro-systems.** Keep
existing product observation/admission necessary for safe operation; do not
remove functional owners merely because they inspect existing state. Native
heap interventions remain default OFF and are not the present work.

A smaller .mcpack/PNG compressed file is **not** proof of smaller in-game
RAM: meaningful texture-memory candidates require fewer decoded texels,
fewer genuinely distinct loaded texture handles, or removal of redundant
loaded asset variants with all references proven. Vanilla block textures are
already atlas-managed in Minecraft Bedrock; merging by filenames alone is
not a valid optimization. Texture effects primarily concern potential
texture/GPU residency, not proven Windows private resident savings.
Keep all five sessions, gameplay, multiplayer, world data, original 144/10
FPS policy, and profile isolation unchanged. No user-PC testing, CI,
automatic pack installation or background monitoring.

**State:** ACTIVE, source-only. Original MC01–MC05 CloseOne/CloseAll code
and native recovery guards remain intact. The last historical MC02 >35 s
normal-exit incident is **RUNTIME ACCEPTANCE DEFERRED**, not a new P0 source
gate. Reopen only for a confirmed new source defect or explicit operator
authorization to perform Windows/game acceptance.

## Optional follow-up — reference-verified Ghast skins (29 October 2026)

**Added to the existing offline Texture Lite builder, not the runtime app:**
`--with-ghast-entity` enables a pinned `entity_preview.py` owner.
Mojang `bedrock-samples/v1.26.30.5` provides two Ghast skin PNGs and three
related color/MERS TGA files at 128×64. The pinned original Ghast geometry
declares 64×32, with **60 UV faces within bounds**. Its pinned
`entity/ghast.entity.json`, `models/entity/ghast.geo.json`,
`render_controllers/ghast.render_controllers.json`, two texture-set
metadata files and five images are checked by **ten exact Git blob SHA
pins**. Fail closed if any content, selected skin mapping, PBR link,
geometry UV or image dimension differs.

With explicit selection, the builder reduces the five color/material
textures to 64×32 and includes them in the **existing block pack**
with the same UUID and an upgrade manifest version `[0,2,0]`. It
does not create a second always-on pack or a game process, does not
replace original game geometry or adjust its UVs, and does not touch
MC01–MC05, world save files or the installed game. The original
block-only invocation retains v0.1 byte semantics. The two controlled
clients still use unchanged vanilla textures; the three mostly idle
client roles require separate manual, **not yet verified** Education
Global Resources/import behavior.

**Evidence:** five new synthetic Python/Pillow tests plus the five
existing block tests actually executed in the isolated ChatGPT
environment, **10/10 passed**. Exact committed `build.py`,
`entity_preview.py` and test blobs are pinned to the locally
executed bytes. The real Mojang source was checked through pinned
GitHub metadata, client/geometry/controller/PBR content, PNG/TGA
dimensions and 60 UV faces, but a complete private Mojang-derived
`.mcpack` was **NOT built** in this turn, and no Minecraft
Education game or user PC was opened.

**Magnitude:** five 128×64 images to 64×32 each = 40,960 to 10,240
source texels, a hypothetical `122,880` RGBA8 bytes (`0.117 MiB`)
less across all five images if all are resident; three idle clients
would at most triple that hypothetical figure, not save gigabytes.
Loaded content, PBR/texture-set resolution behavior, runtime RAM,
VRAM and FPS remain `UNKNOWN`. Do not present a source-texel delta
as measured memory benefit.

## Active experiment — selective Idle Texture Lite 8×8 (29 October 2026)

**User-approved test case:** up to two actively controlled Minecraft
Education profiles keep ordinary original 16×16 vanilla textures, while
three predominantly idle profiles **may** use a low-fidelity 8×8 resource
pack. The MC01/MC02-main and MC03–MC05-idle examples in the new README
are **not hardcoded assignments**: roles can change, but no runtime pack
switch or automatic per-client injection is implemented.

**Bounded implementation:** a new opt-in offline owner
`Development/tooling/lazytester-textures/build.py` uses only
unaltered Mojang `bedrock-samples` tag `v1.26.30.5` supplied explicitly
at build time. The exact `resource_pack/textures/blocks` Git tree SHA
is pinned to `6a811ec148ce4af37981f1055e8b678f7d11905e`
(3,818 files, 1,227 PNGs, ~4.96 MiB original subtree). Hash every
source object before conversion and reject missing/added/replaced files,
links, unsupported paths and size overflow. Apply Pillow BOX downsampling
to only single-frame non-interlaced static **16×16 PNGs**, preserving
alpha and relative texture paths in a standalone preview `.mcpack`.
Leave 16×N animation strips, UI, unrelated entity/PBR textures,
particles and installed Minecraft unchanged. Only the explicit
reference-pinned Ghast option above also reduces its two color PNG
skins and three related PBR TGA images; model/UV/control JSON stays
unchanged and is used strictly as read-only input proof.
Do not vendor Mojang or third-party image content into GitHub.

**Evidence:** exactly five synthetic local Python/Pillow unit tests
were executed successfully in the isolated ChatGPT environment, including
independent Git `write-tree` comparison, palette/RGBA 8×8 processing,
animated-strip skipping, deterministic ZIP/manifest and stale/symlink
refusal. Exact committed builder and tests use the same source bytes
as the executed fixtures (Git blob hashes pinned on commit).
**Full official source conversion and actual .mcpack installation were
NOT_EXECUTED**: the complete 3,818-file upstream source tree was verified
by GitHub metadata, not materialized into a build environment.

**Proof limit:** each original 16×16 tile contains 256 texels versus
64 texels in its 8×8 replacement, a theoretical 75% per-tile
source-texel reduction, **not 75% client RAM reduction**. Even a
theoretical 1,200 eligible static tiles would save just **~0.88 MiB**
of isolated RGBA8 pixel payload per client, before block atlasing,
mipmaps, PBR, caching, decoding and residency. Minecraft Education
26.32 custom packs remain officially unsupported; pack activation
and isolation to only three clients are NOT_VERIFIED. A world-scoped
host resource pack may propagate to other players; never edit the
host world or original world settings, and do not enable globally
as part of a launcher operation.

**Next:** supply or securely retrieve the complete pinned Mojang
source during an explicit offline build before publishing any private
preview pack, then independently verify resource-pack import on a
disposable copied Minecraft Education world if acceptance is authorized.
Do not add monitors, native hooks, background pack installers, new IPC,
asset downloads at ordinary game launch, CI or user-PC dependencies.

## Active direction — offline texture/particle .mcpack optimization (29 October 2026)

**Scope:** asset/build-time changes only. Reuse the existing
`Development/tooling/lazytester-particles/` source owner and its explicit
developer invocation; add a small sibling asset function only after a
version-matched source and genuinely memory-reducing transformation are
identified. No second game runtime, new IPC setting, service, profiler,
monitor, repeated benchmark loop or automatic MC01–MC05 profile changes.

- **Particle:** retain opt-in v0.3 as a strict 20-effect preview. Only three
  `steady` effects currently scale true spawn rate; 17 `manual` effects
  constrain active caps conditionally. Preserve the existing 75/50/25/0
  camera-distance curve and all vanilla pixels/UV/motion/lifetime; do not
  relabel a pack-size reduction as resident RAM saving.
- **Texture:** first examine exact *authorized/owned* asset references and
  dimensions. Promote a change only if it demonstrably lowers decoded texels,
  safely consolidates entity texture handles with all UV/geometry references
  updated, or removes truly unused content from an owned source pack.
  Minecraft already atlases ordinary block textures. Recompressing PNG/ZIP
  or blindly replacing vanilla assets does not establish resident RAM savings.
  **Exception newly approved for experimentation:** downscale validated
  *static* vanilla 16×16 block albedo to 8×8 on selected idle clients only.
  This deliberately loses pixel detail and is not quality-preserving. Other
  texture sizes, animated strips, UI, entity, PBR, mipmapping or UV changes
  still require their own review; no automatic quality switching.
- **Edition gate:** Bedrock creator documentation covers pack mechanisms, but
  Minecraft Education's official support continued to identify custom
  texture/resource packs as **unsupported** in 2026. A custom pack is thus
  an experimental optional copied-world artifact, not a supported Education
  deployment claim. Nothing is installed, enabled, injected, applied globally
  or imported into original host worlds on this source-only track.
- **Evidence:** a byte size/estimated texel count is a *static resource
  budget*, not observed loaded allocations, Windows RAM, VRAM, FPS or
  multiplayer acceptance. Real saving is UNKNOWN without an independently
  authorized edition-matched runtime comparison. Do not build another
  tracking product to address that unknown.

Sources for this decision: official Minecraft Creator
[performance and texture guidance](https://learn.microsoft.com/en-us/minecraft/creator/documents/practices/improvingperformanceandresourceusage),
[texture budgets](https://learn.microsoft.com/en-us/minecraft/creator/documents/texturebudgets),
and [Minecraft Education support](https://edusupport.minecraft.net/hc/en-us/community/posts/27603741486228-How-to-get-a-transparent-picture-frame).

## Evidence — existing Memory Reduce .mcpack is NOT a safe lossless texture source (29 October 2026)

**Artifact reviewed:** prior user-uploaded `Memory Reduce [Ram].mcpack`,
SHA-256 `b3d4abd6a453171f85715bfa956303635a7dfb54f891c9fee1ff71cee92779db`
(1,556,587 compressed bytes; 1,463 ZIP entries). This is a private
metadata-only inspection; no third-party PNG/JSON was copied to GitHub,
rebuilt, distributed, installed, or applied to MC01–MC05.

One-off ChatGPT sandbox archive inspection found **1,109 PNGs** and 563
root block PNGs, including **310 at 1×1** and **234 at 8×8**. Root
particle files: **121** with disabled activation/zero active cap.
Root flipbook declarations: **69**, each set to the single frame
`[0]` (the two other subpack copies are identical). These directly
trade visual detail, atmospheric cues and animation for fewer source
pixels/particles, contrary to the quality-preserving goal; the
`Normal` subpack does not remove root overrides.

The archive contains **98 groups of byte-identical PNGs** (148 excess
filenames), including six pairs of 256×256 panorama images in distinct
GUI paths. Identical bytes do **not** prove both paths are loaded,
deduplicated by the renderer, or safe to alias: GUI paths, atlas
references, fixed filenames and unknown Education behavior have not
been fully demonstrated. Never delete/repoint them based solely on
SHA-256 collisions. **11 of the 20** current LazyTester particle
filenames overlap root overrides in this pack, and eight overlap its
subpacks. Activation order and winner are unverified; never stack
these packs and assert all 20 tuning effects are active.

For perspective only, summing source dimensions of all 1,109 PNGs
gives **4,053,196 texels**, or **15.46 MiB as hypothetical RGBA8
pixel buffers**. GUI images alone account for approximately 14.16 MiB;
block images approximately 0.20 MiB. These are not resident Minecraft
RAM, loaded GPU memory, eliminated textures, atlas residency, or
achievable savings, and may include files not used by Education.

**Disposition: `NO_SAFE_TEXTURE_TRANSFORM` for this candidate.**
There is no quality-preserving, Education-version-matched, fully
referenced texture transformation that can be implemented safely from
this archive alone. Do not import its root/subpack content, disable
ambient/gameplay effects, downscale vanilla pixels, or create a new
texture conversion/monitoring component to mask the missing proof.
Retain the separately source-pinned 20-effect LazyTester Particle
Tuning v0.3 offline preview unchanged.

**Next material source requirement:** an authorized original
texture/geometry/render-controller asset set with its complete
references and dimensions. Only then consider a bounded
reference-safe atlas, empty-space crop or exact reuse of a
truly duplicated loaded texture, and evaluate decoded texel/handle
budgets separately from any runtime RAM claim. No user PC, CI,
host-game changes or new monitoring are required for this audit.

## Active implementation — Preserve Visuals and texture memory configuration (29 October 2026)

**User-approved continuation after particle v0.3:** prefer supported game
configuration mechanisms before more native heap interventions. Do not infer
Education 26.32 support or claim measured MiB from Bedrock patch notes.

- For profiles without `graphics.json`, `GraphicsPreset::Preserve` is now
  the default. The original per-slot graphics owner skips all Minecraft
  options-directory reads/creation/writes for Preserve. Existing saved Low/High
  arrays retain their historical visual overrides and replay behavior;
  switching to Preserve **prevents future rewrites**, not restoration of
  previously reduced Minecraft settings. The typed UI exposes and explains it.
- The already existing confirmed `gfx_texture_streaming:0→1` setting now has
  a symmetric **explicit** `1→0` operation through the same retained session
  controller, slot/process admission, unique-key parser, verified backup and
  atomic replacement. Missing, malformed, duplicated, running or foreign
  settings fail closed. No graphics option is auto-created or automatically
  switched. It may affect GPU memory but Education 26.32 support/system RAM
  effects and visual texture-loading behavior remain UNKNOWN.
- The original Data Storage response retains the optional bounded pack
  folder inventory and now adds **optional world pack declarations** from
  each saved world's `world_resource_packs.json` only. The resource
  inventory is suppressed for the selected running/unadmitted profile.
  The bounded stopped-profile scan reads neither `db` nor `level.dat`,
  asset bytes, credentials, player data, pack textures or behavior packs;
  link/junction/oversize/invalid JSON and time limits fail closed. Counts
  declare per-world references, NOT confirmed active packs, the current
  world, global pack activation or loaded/reclaimable RAM/VRAM. Partial
  world metadata is independent from the valid on-disk pack folder count,
  and old wire responses without this optional field still parse. No
  world/pack mutation, CI or extra worker was added.
- First-run FPS consistency correction: when `performance.json` does not
  exist, the original manager now uses the same **144 active / 10 background**
  ceilings as the native pacer; the Svelte pre-observation placeholder agrees.
  Already saved FPS preferences (including older 120/5 or background 1..4)
  are preserved unchanged. The existing Rust settings-policy fixture now
  compares the fresh default with the native pacing header instead of
  duplicating literal values. This is a source-only consistency correction;
  no measured FPS or game-runtime proof is claimed.
- Code owner updates: `manager_settings.rs`, `game_preset.rs`,
  `native_clients_session.rs`, `session_controller.rs`, typed bridge/UI,
  and one small `resource_pack_inventory.rs` read-only owner. Native RAM
  canaries, executable admission, accounts/worlds, 144/10 FPS policy,
  render distance >=12 and original session architecture stay unchanged.
- **Evidence ceiling:** exact-GitHub-source contract and fixture review;
  Rust/Windows/Svelte/Vitest compilation, Minecraft Education runtime,
  Texture Streaming GPU/private-memory performance, five-client acceptance
  and resource pack activation are NOT_EXECUTED. No CI, package, main commit
  or user PC access.

## Active implementation — LazyTester Particle Tuning (29 October 2026)

**User-approved focus change:** priority has moved from Minecraft's native
UploadBufferHeap experiments to a small, separately built **LazyTester Particle
Tuning** resource-pack preview. Product-facing name is LazyTester, not MIVUBI;
existing on-disk/registry compatibility names are NOT renamed.

- Canonical source: `Development/tooling/lazytester-particles/` only.
  `catalog.mjs` pins **20 ambient Minecraft Bedrock sample effect identities**
  and exact Git blob hashes from Mojang `v1.26.30.5`. `tune.mjs`
  preserves full vanilla definitions, tuning **three steady emitters'**
  `spawn_rate` **and** `max_particles`, plus **17 event-driven manual
  emitters'** `max_particles` only. Manual cap reductions do not directly
  control per-event spawn rate and may not save anything below saturation.
  `archive.mjs` emits a deterministic compressed ZIP; `build.mjs`
  requires explicit source/output, rejects stale sources as encountered and
  bounds streamed official downloads. Compression saves pack bytes, not RAM.
- **Operator-approved aggressive visual budget:** 0–16 blocks **75%**;
  32 blocks **50%**; 48 blocks **25%**; >=64 blocks **0%**, interpolating
  smoothly between thresholds. Camera distance refers to emitter distance,
  not every individual particle. Only the two reviewed rate/cap fields may
  change; no new texture files, altered pixels, UV/animation, color,
  lifetime, physics, combat feedback or Education particle definitions.
  The selected four thresholds are collinear: the default generated
  MoLang uses **one clamped camera-distance query per edited field**, down
  from three for the prior piecewise expression, with exactly the same
  mathematical 75/50/25/0 curve. Cap reevaluation timing on Education and
  actual live particle counts remain UNKNOWN. Decorative effects can be less
  dense even near the player; no pixel-for-pixel equivalence or measured
  FPS/RAM/VRAM benefit is claimed.
- Source-only effectiveness disclosure: the generated manifest and optional
  builder success receipt now distinguish the three direct `spawn_rate`
  controls from 17 conditional `max_particles` ceilings. No new emission
  interception, event suppression, game hook, world import or runtime claim
  was introduced. An unchanged manual emitter below its cap may show no
  reduction at any distance. The reviewed original rate/cap transform is
  otherwise untouched; exact-head Node/Minecraft execution is NOT_EXECUTED.
- Preview pack version advanced to `[0,3,0]` retaining pack UUIDs
  for explicit version upgrades. **All 20** official sample blob hashes,
  effect identifiers, formats and emitter classes were independently
  checked through GitHub. The isolated Node 22.16.0 candidate source passed
  **14 staged regression checks**; an independently parsed **synthetic-only**
  21-entry ZIP passed Python ZIP/CRC checks. The final committed suite has
  12 cases; exact-HEAD Node/Windows/Education execution was NOT_EXECUTED.
  No source-retrieved official `.mcpack` has been published or installed.
  Minecraft Education runtime, multiplayer behavior and real RAM/FPS impact
  remain NOT_EXECUTED.
- Mojang's sample files are **not vendored** because their LICENSE is EULA
  governed, not permissive. Output is generated privately only on an explicit
  developer invocation; never staged, automatically enabled in profiles,
  loaded into an original multiplayer world, or packaged as a game DLL.
  Education 26.32 support for these Bedrock sample particle definitions and
  the MoLang camera-distance query remains NOT_EXECUTED/UNKNOWN.
- This is independent of the previously uploaded `Memory Reduce [Ram].mcpack`
  (still DEFERRED) and existing default-OFF native heap experiments.
  MC01–MC05 accounts, world files, game executable, 144/10 FPS and render
  distance >=12 remain untouched. Synthetic Node source/ZIP tests were
  executed separately in ChatGPT's Node 22.16.0 sandbox; Windows/Minecraft
  runtime remains NOT_EXECUTED. No CI or local user PC.

## Latest source-integrity and blocked-profile inspection correction (29 October 2026)

- The existing source verifier had an obsolete Improved Input Response parser
  literal and could falsely refuse today's shared `observed_binary_option`.
  Its existing ownership check now covers the current unique 0/1 parser for
  Improved Input Response and Texture Streaming, both typed receipts and
  mounted UI. It also guards current admission order rather than adding a
  parallel verification system.
- The settings inspector now refuses unadmitted active/foreign/uncertain
  profiles **before reading** `options.txt` or its backup. The refusal remains
  visible and key values stay UNKNOWN; admitted stopped profiles retain their
  original options validation and recovery semantics.
- Full-clear status still checks the journal for pending recovery even for
  blocked profiles but postpones full catalog/hive token computation until
  successful admission. A blocked slot receives no confirmation token;
  actual deletion still needs its original fresh independent gate.
- Focused Rust/TypeScript regression fixtures and the existing source guard
  were authored and reviewed by exact GitHub source only, NOT_EXECUTED.
  No Windows, Minecraft, installer, FPS or RAM claim. The Settings page's
  DataLocation read remains active because the UI displays/opens that path.

## Latest five-profile resource-metadata deadline correction (29 October 2026)

- Repaired the original read-only `resource_pack_inventory.rs` owner's
  deadline semantics. Previously a shared two-second budget was checked
  only while enumerating entries; when another profile used the budget,
  later roots and missing/empty inventories could still be opened or
  reported as complete zero. The owner now refuses before extra opens,
  after ancestor probes, at each leaf scanner's completion and after
  a missing-world directory probe. The response remains `partial=true`
  with an issue, not false absence or proven reclaimable memory.
- The existing `session_controller.rs` profile-size scanner marks a
  traversal incomplete when its final filesystem operation consumes the
  budget, even if no next loop iteration exists. Original account/world,
  gameplay, profile/registry, cached-data delete and session mutation
  admissions, five-profile budget, and wire contracts stay unchanged.
- One focused Rust fixture for missing and populated roots verifies
  fail-closed zero-budget responses and the admitted complete path.
  It was AUTHORED, NOT_EXECUTED; GitHub source checks alone do not
  establish a Windows speed, RAM, FPS or game-runtime improvement.
  CI remains disabled; the user's computer was not accessed.

## Active owners

- Mounted Tauri 2/Svelte UI: `Development/apps/desktop/src/LazyTesterApp.svelte`
  and typed bridges; the WindowsNative capability gates unsupported OS.
- Original game/session/profile storage and normal close:
  `src-tauri/src/engine/independent_bootstrap/session_controller.rs`.
  Exactly one authoritative controller and no new account manager.
- Presentation facade: `src-tauri/src/engine/workspace_surface.rs`.
  Win32 adapter: `workspace_surface/native.rs`. Retained recovery controller:
  `workspace_surface/controller.rs`; tests are extracted into submodules.
- DWM facade: `workspace_surface/previews.rs`. Sole capacity-one Win32
  worker: `workspace_surface/previews/windows.rs`; fixtures are separate.
- Existing native compatibility/FPS/HUD DLLs:
  `Development/tooling/acceptance/windows/bootstrap-ordering/`.
- RAM observation only: `Development/tooling/acceptance/windows/collect-native-performance.ps1`
  with its opt-in `native-profile-baseline.ps1` helper, reviewed by existing
  `Development/tooling/acceptance/evaluate-native-performance.mjs`.

## Completed bounded source work

- Separated the original Win32 preview/recovery API, implementations and
  tests mechanically. Retained DWM, Win32 handle checks, original lifecycle,
  one worker, IPC and unsupported-platform refusal without runtime claims.
- Existing read-only memory sampler supports zero to five current native
  clients, explicit sparse MC slot labels, manager and whole-system metrics,
  with optional bounded direct-child attribution. Schema 7 retains exact
  process birth pins and UNKNOWN on identity loss. Schemas 1–6 remain
  historical diagnostic compatibility only.
- Existing evaluator accepts schema 7, rejects false identity/attribution
  and compares matched A/B private commitment, private working set and system
  memory descriptively. Three schema-7 pure logic fixtures were run in a
  sandboxed JS review environment; repository Node/Rust/PowerShell tests
  were NOT_EXECUTED. Updated source verifier owns the new file boundaries.
- Exact Win32 process-image reads now share one bounded Windows-only owner
  (`src-tauri/src/engine/windows_process_image.rs`) in five existing
  callsites: native observation, authenticated IPC, signer verification,
  workspace parent validation and launch bootstrap. Typical image paths
  now start with a 512 UTF-16-character buffer (1 KiB) instead of 32,768
  characters (64 KiB), expanding only when Win32 reports insufficient
  space. This removes repeated temporary heap allocations and one large
  bootstrap stack buffer; it is **not** a measured Minecraft RAM reduction.
  Windows-only fixture authored, NOT_EXECUTED.
- Settings/Storage with no running Minecraft now skip four of five automatic
  native refresh opportunities (10 s instead of 2 s); live games, workspace,
  launcher views and recovery keep the existing 2 s cadence. Uses the single
  existing timer. Only source-level scheduling logic/fixtures were authored.
- The existing native FPS HUD worker now retains one DIB/font across redraws
  at identical DPI/size and releases it at worker exit, eliminating repeated
  GDI create/delete churn without introducing an extra cache, worker or
  process owner. C++/Windows test NOT_EXECUTED.
- The native schema-7 collector now includes opt-in cumulative process
  PageFaultCount from its existing process-memory API call; historical
  schema-5/6 shape and older schema-7 reports remain supported. Page fault
  count is not a hard-fault/disk-read indicator. Five isolated evaluator
  fixture cases passed; PowerShell/Windows NOT_EXECUTED.
- Read-only `InspectSettings` now uses the original exact image path
  pinned during controller preparation for process discovery, avoiding a
  repeated registered-app PowerShell resolution while showing idle status.
  `ApplyPreset`, `ApplyGraphicsPreset`, data-clear/reset and other writes
  still use their existing fresh registration/identity checks. The passive
  `canApplyPreset` flag is only a UI hint, never write authority.
  This reduces temporary Windows process/registry work; memory savings and
  build/runtime behavior are NOT_EXECUTED.
- Native Win32 process observations now feed the existing strict duplicate
  PID/HWND/schema validator directly, rather than allocating temporary JSON
  and parsing it back on every scan. Raw JSON inputs still deserialize and
  pass through that same validator; the five-profile cap and identity checks
  are unchanged. Rust fixture authored, NOT_EXECUTED.
- The existing offline native schema-7/8 RAM reviewer now returns a
  **per-selected-profile aspirational <1 GiB distance** using the complete
  private-working-set median. It reports a signed median gap and descriptive
  percentage only with full process/private-residency sample coverage; any
  lost identity or unavailable reading returns null. It never converts a
  threshold crossing into game/world/host readiness or measured savings.
  This extends reviewer output only, not collector schema, Minecraft,
  profiles, the launcher lifecycle or resource policy. Focused Node
  regression fixtures are AUTHORED, NOT_EXECUTED.
- The existing schema-7/8 offline evaluator now derives **synchronous
  selected-root memory totals** (private commitment and private working set),
  separately for selected MC roots and the optional manager. It never sums
  per-profile maxima across timestamps or adds direct children/shared pages.
  Incomplete rows, missing EX2 residency, identity loss and zero selected
  clients cannot produce misleading numeric total/aggregate A/B differences.
  Numeric overflow in otherwise finite observations also fails closed.
  The retained comparer remains descriptive and explicitly lacks workload,
  gameplay and system-memory proof. Focused regression fixtures are authored;
  isolated JS logic and full Node/Windows execution are separately reported.
  Isolated source review: the preceding aggregate commit message recorded
  15 checks by mistake; its tool output actually recorded 14. The final
  numerical-hardening revision passes 17 additional isolated V8
  checks with synthetic schema-7/8 data. Full Node, Rust, Windows and
  Minecraft tests remain NOT_EXECUTED; this is not a RAM saving.
- The current graphics owner audit confirms that owned DWM preview
  registrations are unregistered through Thumbnail::drop, and native
  workspace suspend/release clears the preview roster. The HUD worker
  releases its GDI bitmap/font and thread when retired. This is bounded
  overlay resource ownership, not hundreds of MiB of game heap reclamation.
  Both game heap experiments remain default OFF and mutually exclusive.
  The existing repository source verifier now enforces the default-OFF
  product build, version-pinned PE/code hashes, and no-op disabled mode.
  Focused negative regression fixtures are authored; no Windows/game
  runtime or release payload was changed.
- Verified a redundant startup resource path in the mounted Play
  initializer: \`profile_binding::run(false)\` calls
  \`product_profiles::open()\`, which validates all five profile
  directories/registry hives. The immediately following startup
  \`loadLocation()\` called \`product_profiles::location(Inspect)\`
  and \`open()\` again before Play displayed any location data.
  Removed only that second read-only location request from successful
  Play initialization; Settings/Storage navigation, explicit Refresh,
  Locate/Choose, and all fresh mutation/launch checks retain their
  original owners and failure handling. This saves one repeated
  five-profile open/RegLoadAppKey cycle on the successful startup path
  in source, not proven MiB of resident RAM, faster startup, or game
  allocator savings. Focused Svelte source regression fixture
  AUTHORED, NOT_EXECUTED; no native binary or profile mutation.
- **Material code-first mitigation prepared:** the existing native session
  controller accepts an explicit per-slot `DisableImprovedInputResponse`
  operation through the original serialized authenticated transport.
  It rechecks fresh game/process admission, requires that selected profile
  is stopped and a single exact saved `ctrl_improvedInputResponse:1`
  exists, edits that existing key to 0 only, keeps BOM/line endings and
  unrelated bytes, and publishes the existing verified backup before
  the atomic options replacement. It refuses missing, disabled, duplicated,
  malformed or linked settings, never seeds a new options file, and can
  be undone using the existing Restore Settings workflow. The mounted
  Storage UI requires an explicit confirmation and discloses input-latency
  tradeoff and unverified Education 26.32 RAM effect. Rust/TS fixtures
  AUTHORED, NOT_EXECUTED; no product, game, profile or Windows changes
  were applied in the GitHub-only context.
- Native **Texture Streaming opt-in** is now available for a stopped
  launcher with an existing uniquely disabled `gfx_texture_streaming:0`
  option. Mojang's Bedrock 26.30 notes identify texture streaming as a
  built-in GPU texture-memory reduction on supported PC devices; Education
  26.32 carries some 26.30 features but exact feature support in its
  installed executable is NOT verified. The original settings owner now
  reports existing 0/1 status and, only after explicit UI confirmation
  and original controller process/slot admission, updates 0 to 1 using
  the existing verified backup and restore path. It refuses missing,
  duplicate, malformed or ambiguous settings; never adds a game option,
  changes profiles/worlds/accounts or enables unsupported graphics hacks.
  Texture residency and total RAM reductions are NOT_EXECUTED/UNKNOWN.
  Technical references:
  https://www.minecraft.net/en-us/article/minecraft-26-30-bedrock-changelog
  https://minecraft.wiki/w/Options.txt_(Bedrock_Edition)
- CI stays OFF. No install, release, Windows game process or user PC touched.

## Deferred candidate — visual resource pack (not active)

- User-supplied `Memory Reduce [Ram].mcpack`, retained in the originating
  conversation; **metadata only** is recorded here, not redistributed as
  third-party asset content or included in the application. File SHA-256:
  `b3d4abd6a453171f85715bfa956303635a7dfb54f891c9fee1ff71cee92779db`;
  1,556,587 bytes, 1,463 ZIP entries. This fingerprint allows an exact
  later match if the original is supplied again.
- Static archive inspection: 317 images at 1x1 pixels, 285 at 8x8,
  121 disabled main particle definitions, and 69 static flipbook entries.
  The pack trades away block texture detail, animations and gameplay/
  Education particle cues; its "2chunks" fog presentation is not proof
  that the game actually loads fewer chunks. Its "Normal" subpack does
  not undo root-level visual overrides.
- **DEFERRED / NOT_APPROVED for implementation.** Possible benefits are
  reduced texture/particle workload and GPU memory, not verified private
  working-set reclamation. No pack installation, repository import, game
  changes, benchmark, or addon rollout is authorized by this note.
  Revisit only on explicit user reprioritization, with selective
  gameplay-safe treatment rather than blind full-pack adoption.

## Evidence and preservation

Only REMOTE_GITHUB source changes and isolated evaluator logic checks are
supported. Rust/Windows compilation, Minecraft preview and normal exit,
CPU/RAM benchmarks, concurrent five-client gameplay, account/multiplayer
continuity and macOS support are NOT_EXECUTED or unsupported.
Historical ~2.49/2.68 GiB client private commit is not a current
revision's measurement. No verified RAM/CPU improvement percentage exists.

Preserve all original MC01–MC05 Roaming/Local/HKCU data, accounts, worlds,
process identity, normal exit/recovery, DLL admission, 144/10 FPS limits,
12-chunk minimum render distance, host simulation/network/save continuity.
No force-close, blind working-set trims, foreign heap releases, speculative
game hooks, new monitoring service, test bypass, CI, installer or main branch
publication. The Mac build remains a diagnostic shell without game backend.

## Next bounded source action

Target stays Minecraft Education stable 26.32 Windows (release family,
not a verified current EXE fingerprint). Existing launcher image/DLL
identity gates, five isolated account/world profiles and default-OFF
native GPU memory canaries remain unchanged. P0/localtest/CI remain off.

**New material lead:** 2025 Minecraft Education users reported very
large RAM growth with Improved Input Response enabled and relief after
disabling/restarting. Related Windows Bedrock issues continued in
later versions; no causal RAM effect is proven for Education 26.32.
Source-only implementation initially reads the existing bounded options.txt
during already-authorized storage inspection and optionally exposes
a single boolean for the explicit `ctrl_improvedInputResponse` 0/1
entry. Missing, malformed, contradictory or unauthorized observations
stay UNKNOWN; no automatic toggle, unauthorized game I/O, account/world changes or
modified graphics preset. The existing strict typed UI shows this as a
non-authoritative diagnostic, with no promise of savings. Optional wire
field preserves older session response compatibility. Rust/Vite fixtures
are AUTHORED, NOT_EXECUTED; only exact-source guards were reviewed.

**New reviewer-only progress metric:** schema-7/8 now reports each selected
process's descriptive median private-residency gap against the historical
<1024 MiB aspirational target, without assuming that a PID is a verified
Minecraft 26.32 world, that its helper tree was measured, or that FPS/host
continuity passed. Missing or identity-lost readings never receive a numeric
gap. No RAM saving is claimed; Node fixtures AUTHORED, NOT_EXECUTED.

**New source boundary:** no authoritative Education-26.32 graphics/CPU
allocation ownership, synchronized 220.488/144 MiB residency/free-state
mapping or 67 MiB object-release/reconstruction contract is present in
this revision. Microsoft PIX/DxTimingCaptureLibrary can interpret sufficiently
covered DirectX ETW, but raw provider-complete target ETW and comparable
private-residency evidence are absent from this GitHub-only source context.
No speculative renderer patch, working-set trim or experimental hook
promotion is justified. The reviewer improvement concerns *measurement
honesty*, not reclaimed game RAM.

**Primary next source action:** use the original offline Texture
Lite builder and its opt-in `--with-ghast-entity` preflight for the
three idle-profile experiment, but do not install or activate it
automatically. Official Ghast original source and all UV references
are pinned, with **10/10 synthetic source tests passed**, but the
full source-derived pack and Education 26.32 gameplay remain
`NOT_EXECUTED`. This Ghast candidate saves only `0.117 MiB`
of hypothetical RGBA8 image payload per client, not meaningful
measured system RAM. Larger texture candidates must earn their
place through a complete model + render-controller + PBR dependency
audit and real usage relevance. No monitoring, hooks, extra managers,
CI, world edits or user-PC testing.
