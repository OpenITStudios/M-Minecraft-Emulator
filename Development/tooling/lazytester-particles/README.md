# LazyTester Particle Tuning — optional preview

This is a **resource-pack builder**, not a game mod, installer, runtime hook or
new Minecraft session owner. It produces an optional Bedrock-format
`.mcpack` only when explicitly invoked. Nothing is installed or activated
in any MC01–MC05 profile or original world.

## Source, rights, compatibility

- The only source is Mojang's
  [Bedrock sample v1.26.30.5](https://github.com/Mojang/bedrock-samples/tree/v1.26.30.5/resource_pack/particles),
  selected by three **exact Git blob hashes** in `catalog.mjs`.
- Mojang's [license](https://github.com/Mojang/bedrock-samples/blob/v1.26.30.5/LICENSE.md)
  reserves rights and refers to the Minecraft EULA. No Mojang samples or
  third-party particle/texture files are committed to this repository; the
  builder transforms source bytes provided or explicitly fetched at build time.
- **Bedrock samples are not authoritative Minecraft Education 26.32 assets.**
  The Education implementation of these three identifiers and the
  camera-distance query has not been verified at runtime.
- No part of the previously uploaded `Memory Reduce [Ram].mcpack` is imported.

## Exactly what is modified

| Vanilla identifier | Source operation |
| --- | --- |
| `minecraft:portal_reverse_particle` | Distance-adjusted steady `spawn_rate` |
| `minecraft:nectar_drip_particle` | Distance-adjusted steady `spawn_rate` |
| `minecraft:dolphin_move_particle` | Distance-adjusted steady `spawn_rate` |

- **0–16 blocks:** exactly the source emission rate.
- **16–48 blocks:** smoothly interpolate to 80% of the source emission rate.
- **48+ blocks:** maximum 20% reduction in particle spawning, not opacity.
- Original effect identifier, material, texture reference, UV, particle size,
  motion, lifetime, color, animation and `max_particles` remain unchanged.
- No new textures, atlases, script APIs, Java mods, entity behaviors, render
  distance changes, global particle removals or original game file edits.
  Visual particle density far away can change; pixels are not guaranteed
  identical in every frame. FPS and RAM benefits are not assumed.

## Explicit offline build or official-source retrieval

Using a separately obtained official `resource_pack` directory:

```sh
node Development/tooling/lazytester-particles/build.mjs \
  --source /path/to/bedrock-samples/resource_pack \
  --output /chosen/path/LazyTester-Particle-Tuning-preview.mcpack
```

Alternatively, only on explicit command, fetch the three official sample
files from Mojang (network required):

```sh
node Development/tooling/lazytester-particles/build.mjs \
  --official yes \
  --output /chosen/path/LazyTester-Particle-Tuning-preview.mcpack
```

Every exact source is verified **before** output. A missing/changed source,
invalid identifier, unsupported emitter or existing output file fails closed.
The tiny `.mcpack` is a ZIP with only `manifest.json` and three
particle JSON definitions. The script never reads profile directories.

## Safe activation, rollback, evidence

**Preview only; never auto-install.** When separate testing is authorized,
import the pack into Minecraft Education and activate it as a Resource Pack
in a **disposable copy** of a world, not the original Education host world.
A world pack may affect multiplayer clients: do not enable it across five
accounts without host/join compatibility checks. To revert, deactivate
`LazyTester Particle Tuning (Preview)` in that copied world's Resource Packs.

Synthetic Node builder/ZIP tests can validate input contracts and unchanged
JSON components; they are **not** proof of game loading, visual equivalence,
FPS, private RAM, VRAM, multiplayer or long-session reliability.
Minecraft Education runtime and installed-client validation are NOT_EXECUTED.

References: [Molang camera-distance query](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/molangreference/examples/molangconcepts/queryfunctions/query_camera_distance_range_lerp),
[steady particle emitter](https://learn.microsoft.com/en-us/minecraft/creator/reference/content/particlesreference/particlecomponents/minecraftemitter_rate_steady),
[vanilla asset overrides](https://learn.microsoft.com/en-us/minecraft/creator/documents/overwritingassets),
[Minecraft Education Add-Ons](https://education.minecraft.net/en-us/resources/minecraft_education_add_ons_collection).
