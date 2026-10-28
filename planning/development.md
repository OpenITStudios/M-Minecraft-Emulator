# Current Development Handoff — v0.1

## Current intent — 29 October 2026

**Goal:** Optimize real private-resident RAM and committed memory for 0–5
Minecraft Education Windows clients as far as safely possible. The previous
<1024 MiB per-client figure is an **aspirational reference, not a required
success threshold**; smaller confirmed savings are useful if FPS, gameplay,
multiplayer hosting, world saves and profile isolation remain intact.
The operator explicitly requested CODE-FIRST development without Windows
localtest and no repeated P0/checker/documentation loops.

**State:** ACTIVE, source-only. Original MC01–MC05 CloseOne/CloseAll code
and native recovery guards remain intact. The last historical MC02 >35 s
normal-exit incident is **RUNTIME ACCEPTANCE DEFERRED**, not a new P0 source
gate. Reopen only for a confirmed new source defect or explicit operator
authorization to perform Windows/game acceptance.

## Active implementation — LazyTester Particle Tuning (29 October 2026)

**User-approved focus change:** priority has moved from Minecraft's native
UploadBufferHeap experiments to a small, separately built **LazyTester Particle
Tuning** resource-pack preview. Product-facing name is LazyTester, not MIVUBI;
existing on-disk/registry compatibility names are NOT renamed.

- Canonical source: `Development/tooling/lazytester-particles/` only.
  `catalog.mjs` pins **10 ambient Minecraft Bedrock sample effect identities**
  and Git blob hashes from Mojang `v1.26.30.5`. `tune.mjs` preserves full
  source definitions while tuning three steady emitters' `spawn_rate` and
  seven event-driven manual emitters' `max_particles` using the same
  distance-relative curve. These represent **different** native semantics:
  manual cap reductions do not guarantee fewer emissions below the cap.
  `archive.mjs` emits a deterministic tiny resource ZIP; `build.mjs`
  requires an explicit source/output and rejects changed/oversized sources.
- **Operator-approved aggressive visual budget:** 0–16 blocks **75%**;
  32 blocks **50%**; 48 blocks **25%**; >=64 blocks **0%**, interpolating
  smoothly between thresholds. Camera distance refers to emitter distance,
  not every individual particle. Only the two reviewed rate/cap fields may
  change; no new texture files, altered pixels, UV/animation, color,
  lifetime, physics, combat feedback or Education particle definitions.
  Decorative effects are genuinely less dense even near the player;
  no visual equivalence or measured FPS/RAM/VRAM benefit is claimed.
- Preview pack version advanced to `[0,2,0]` with original pack UUIDs,
  so versions remain distinct. Ten official sample metadata hashes and
  emitter classes were independently verified through GitHub; the Node
  22.16.0 sandbox executed seven focused regression cases and created one
  **synthetic-only** 11-entry ZIP independently CRC-checked by Python.
  Official-source full-pack creation, Education runtime, multiplayer
  behavior and sustained resource benefits remain NOT_EXECUTED.
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

**Primary next source action:** retain optional LazyTester Particle
Tuning v0.2 as a **source-built-only** preview with pinned Mojang samples,
aggressive 75/50/25/0 distance policy and no installed world/profile changes.
Before activating it, establish Minecraft Education 26.32 support for the
referenced vanilla effects and distance/cap Molang evaluation. Do not equate
synthetic tests or Mojang's Bedrock source with Education runtime acceptance;
a copied-world host/join trial requires separate operator authorization.
Native `UploadBufferHeap` remains default OFF and outside this workstream.
