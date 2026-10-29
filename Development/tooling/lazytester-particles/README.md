# LazyTester Particle Tuning — optional aggressive preview v0.3

An **optional resource-pack builder** for a small allowlist of ambient vanilla
particle effects. It does not install a Minecraft add-on, access original
MC01–MC05 profiles, modify worlds, patch executable/renderer memory or spawn
another background service. The builder is not part of the released app.

## Source and compatibility

Mojang's [Bedrock sample v1.26.30.5](https://github.com/Mojang/bedrock-samples/tree/v1.26.30.5/resource_pack/particles)
provides the original particle definitions, fetched only by explicit request
or supplied as a locally obtained `resource_pack` directory. Every complete
source must match its **exact Git blob SHA** before an override is created.
Mojang's [license](https://github.com/Mojang/bedrock-samples/blob/v1.26.30.5/LICENSE.md)
reserves rights under the Minecraft EULA; those sample JSON files are **not
vendored in LazyTester Git history**. The old third-party `Memory Reduce
[Ram].mcpack` is not included.

**Bedrock is not Minecraft Education.** Exact effect support, distance-query
behavior and resource pack precedence on Education 26.32 remain unverified.
These changes are therefore an offline-build preview, **not enabled by default**.

## Four distance tiers

| Camera distance to emitter | Relative budget |
| --- | ---: |
| 0–16 blocks (near) | **75%** |
| 32 blocks (medium) | **50%** |
| 48 blocks (far) | **25%** |
| 64+ blocks (very far) | **0%** |

Within 16–32, 32–48, and 48–64 blocks, the budget transitions **smoothly**.
For example, 24 blocks = 62.5%; 40 = 37.5%; 56 = 12.5%. This is a
per-effect **relative emitter** policy, not a global GPU particle budget.

- **Three `steady` emitters:** scale both `spawn_rate` (particles/sec)
  and `max_particles` (the simultaneous per-emitter cap). This limits
  particle creation and the upper bound of live particles; existing
  particles still expire via their unmodified original lifetimes.
- **Seventeen `manual` emitters:** scale only `max_particles`. Manual
  emissions remain **event-driven**. A cap has no effect if the actual
  particle population never reaches it; this is not a guaranteed 75/50/25/0
  drop in observed manual spawn counts.
- The approved four thresholds are exactly **collinear**. LazyTester
  generates `0.75 * (1.0 - query.camera_distance_range_lerp(16, 64))`
  instead of three camera-distance queries and `math.max`. The curve
  is algebraically identical at every distance under the Bedrock query
  semantics. This is a **source-level calculation reduction**, not a
  measured FPS improvement.
- Whether live cap changes reflect camera movement immediately depends
  on when the Education engine reevaluates them. Do not claim an immediate
  hard culling guarantee for already-existing particles at 64+ blocks.

### Selected vanilla effects

| Steady rate tuning | Manual active-cap tuning |
| --- | --- |
| `portal_reverse` | `cherry_leaves_particle` |
| `nectar_drip` | `spore_blossom_shower` |
| `dolphin_move_particle` | `spore_blossom_ambient_block_actor` |
| | `campfire_smoke` |
| | `campfire_smoke_tall` |
| | `rain_splash` |
| | `mycelium_dust` |
| | `pale_oak_leaves_particle` |
| | `biome_tinted_leaves_particle` |
| | `water_drip` |
| | `lava_drip` |
| | `dripstone_water_drip` |
| | `dripstone_lava_drip` |
| | `honey_drip` |
| | `obsidian_tear` |
| | `snowflake` |
| | `endrod` |

**Preserved:** original particle identifiers, UV, material and texture paths,
all pixels, colors, billboard sizes, lighting, lifetime, movement, collision,
texture animations, sound, scripts, world simulation and render distance.
No combat, critical hit, explosion, damage, potion, redstone, navigational,
Education lab or warning effects are overridden.

**Visual trade-off:** Ambient particle *density* really does change, even at
close range, and distant selected ambient effects can disappear. This is
not pixel-identical vanilla and may reduce visible smoke/rain/leaf cues. It
cannot be advertised as lossless visual equivalence or measured RAM savings.

## Build explicitly; never auto-install

Using the pinned official Bedrock sample `resource_pack` folder:

```sh
node Development/tooling/lazytester-particles/build.mjs \
  --source /path/to/bedrock-samples/resource_pack \
  --output /chosen/path/LazyTester-Particle-Tuning-v0.3-preview.mcpack
```

Or allow the builder to fetch the **20 specific Mojang sample JSON files**:

```sh
node Development/tooling/lazytester-particles/build.mjs \
  --official yes \
  --output /chosen/path/LazyTester-Particle-Tuning-v0.3-preview.mcpack
```

The builder refuses modified, missing, oversized, symlinked or unsupported
sources and preexisting outputs. The resulting `.mcpack` archive contains
**one manifest plus 20 particle JSON files**, and **no new textures**.
The ZIP entries use deterministic DEFLATE compression where it reduces file
size; this only reduces the build artifact's disk/transfer bytes, not
the decoded texture or game RAM. A mismatched source is rejected as soon
as encountered, before later sources are fetched or the output is written.
Remote responses are read with a strict 128 KiB per-source byte limit.
On successful output, the CLI identifies **3 steady-rate controls** and
**17 conditional manual-population caps** separately. The generated pack
manifest also identifies this distinction. These are *source transformations*,
not a measurement that 20 effects reduced their live spawn frequency or that
Minecraft reclaimed CPU/GPU memory.
It is not an installer or in-app toggle. A source's Git SHA is a version
pin, not proof of correct Education behavior or runtime performance.

## Rollback and acceptance

When Minecraft Education testing is separately authorized, enable this pack
**only in a disposable copied world**, evaluate visual quality and multiplayer
host/join behavior, then deactivate it in that copied world's Resource Packs
to revert. Never apply to an original active host world or every MC01–MC05
client automatically. Gameplay, Education compatibility, actual FPS/frame time,
private working set, commit, VRAM and sustained RAM reduction are **NOT_EXECUTED**.
The preview has been validated using synthetic JSON, Node tests and a Python
ZIP parser, **not the actual Education renderer**. Avoid assuming the
Bedrock particle list is identical to the Education client binary.

References: [Camera distance interpolation](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/molangreference/examples/molangconcepts/queryfunctions/query_camera_distance_range_lerp),
[steady emitter rate and cap](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/particlesreference/particlecomponents/minecraftemitter_rate_steady),
[manual emitter cap](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/particlesreference/examples/particlecomponents/emitter_rate_manual),
[vanilla resource overrides](https://learn.microsoft.com/en-us/minecraft/creator/documents/overwritingassets).
