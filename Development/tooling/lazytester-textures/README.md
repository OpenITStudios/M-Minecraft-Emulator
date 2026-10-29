# LazyTester Idle Texture Lite 8×8 — optional offline preview v0.1 / v0.2

**For testing only.** This is a build-time texture override pack, not a game
plugin, automatic installation, background service, monitor, memory cleaner or
per-focus runtime quality switch. It does not access MC01–MC05 storage, world
files, credentials or Minecraft. Particle Tuning v0.3 remains separate.

## Proposed two-tier, five-client test

| Intended role | Example launcher assignment | Textures |
| --- | --- | --- |
| Up to two actively controlled clients | MC01–MC02 | Native/default textures, unchanged |
| Three mostly idle clients | MC03–MC05 | Optional Lite 8×8 **if manually enabled per client** |

These are **example roles**, not hardcoded slots. A running game does not switch
packs automatically when focus changes. A **world resource pack can affect all
joining clients**; therefore, do not attach Lite to the host world, use forced
multiplayer packs, or edit a live original world. Global Resources are usually
client-local in Bedrock, but their behavior and compatibility are **not
established for Minecraft Education 26.32**, which does not officially support
custom third-party texture packs. The possibility of isolating it to three
Education profiles is thus **NOT_VERIFIED**. A separate, disposable copied-world
trial and explicit per-client selection would be required to validate it.

## Transform

- Source: **unaltered official Mojang Bedrock Samples tag `v1.26.30.5`**,
  `resource_pack/textures/blocks` Git tree
  `6a811ec148ce4af37981f1055e8b678f7d11905e`.
- Validate the **entire** directory Git tree by hashing its original files;
  abort on missing/added/modified sources, linked/junction paths, oversize
  entries or invalid PNG data. The original assets are never committed here.
- By default, only **single-frame, noninterlaced, 16×16 PNG block albedo**
  entries become 8×8 PNGs at the same relative paths. Preserve alpha with
  Pillow's 2×2 BOX area downsampling. Animated strips such as water/lava,
  item animations, UI and particles remain untouched. The only additional
  entity/PBR exception is an explicitly requested, pinned Ghast trial below.
- The default archive retains the original v0.1 manifest. The explicit
  Ghast extension uses the **same pack UUID, version v0.2**, and one combined
  ZIP/manifest (not two overlapping texture packs). Do not publish or
  automatically install Mojang-derived textures. Respect Mojang's Minecraft
  EULA; previews are private and require authorized source material.

A 16×16 RGBA image occupies **1,024 decoded pixel bytes** versus **256 bytes**
for 8×8, before atlases, mipmaps, GPU layout, streaming and other engine effects.
That's **75% fewer source texels per selected image**, not 75% lower Minecraft
RAM. Even 1,200 such images would save only about **0.88 MiB** of theoretical
RGBA8 image bytes **per client** if the originals were all loaded distinctly;
Minecraft normally atlases block images, so actual RAM or VRAM savings may be
smaller, zero or otherwise different.

## Optional v0.2 Ghast texture/PBR experiment

The **only** optional entity candidate is based on Mojang Bedrock Samples
`v1.26.30.5`. Its pinned `entity/ghast.entity.json` defines two skin paths,
the render controller selects `Texture.default` or `Texture.shooting`,
and the original `geometry.ghast` declares an atlas of **64×32 texels**.
All **60 face UVs** in the pinned original geometry fit that declared atlas.
The two pinned PNG skins and three associated pinned TGA color/MERS images
are **128×64**. This trial halves each image to **64×32**, matching the
geometric texture dimensions, **without editing geometry, UV values, entity
behavior, animations, or render-controller JSON**. It also reads and verifies
both official `*.texture_set.json` references before accepting the images.

The 10 required source files (entity, geometry, render controller, two
texture_set JSONs, and five PNG/TGA images) must match their exact pinned
Mojang Git blob SHA. Nonregular files, links, changed IDs, unexpected
skin references, unsupported model UVs and wrong image dimensions fail
closed. PNG color uses BOX downsampling; TGA material channels use nearest
sampling so metalness/emissive/roughness control values are not blended into
invented intermediate categories. **No source images or JSON are included in
this repository.**

For these five images, 40,960 original texels become 10,240 output texels,
a **75% source-texel reduction**. At most that corresponds to **122,880
hypothetical RGBA8 decoded bytes (0.117 MiB) per client** across all five
textures. Three idle clients could reduce only ~0.35 MiB of such hypothetical
image payload, and only if all those originals would otherwise be loaded
and the game uses the overriding images. Actual texture-set precedence,
PBR loading, Minecraft Education 26.32 resource-pack support, GPU residency,
private RAM and FPS remain UNKNOWN. It is not a meaningful whole-game RAM
saving until a runtime comparison demonstrates one.

Do not use this option in an original hosted world, force it on joining
players, or assume Global Resources isolates three Education clients.

## Explicit build (not installed)

Requires **Python 3.12+** and **Pillow 10+** in an independent development
environment. The source folder must come from an original pinned Mojang
Bedrock Samples checkout or an unmodified official source archive. This
builder does not download, copy, or redistribute the source itself.

```sh
python Development/tooling/lazytester-textures/build.py \
  --source /path/to/bedrock-samples/resource_pack \
  --output /chosen/path/LazyTester-Idle-Texture-8x-preview.mcpack
```

With explicit Ghast/PBR extension (same pack UUID, manifest v0.2):

```sh
python Development/tooling/lazytester-textures/build.py \
  --source /path/to/bedrock-samples/resource_pack \
  --output /chosen/path/LazyTester-Idle-Texture-8x-Ghast-v0.2-preview.mcpack \
  --with-ghast-entity
```

A complete build prints JSON including converted image count, original/preview
texels, theoretical 75% reduction, source tree fingerprint and `NOT_MEASURED`
for runtime RAM. It never overwrites an existing file. Local Python tests use
only synthetic PNGs and can be run independently:

```sh
python -m unittest discover -s Development/tooling/lazytester-textures -p 'test_*.py'
```

**Do not apply automatically to MC03–MC05.** If individually authorized for
a real edition-matched test, first export/copy the world, preserve original
launcher data and test pack import/Global Resources/pack priority on a throwaway
client before selecting any more clients. Disable/remove the pack manually to
revert; do not alter a running game, change pack order for the host, or stack
with the third-party `Memory Reduce [Ram].mcpack`.

## Evidence and limitations

The block builder and opt-in Ghast/PBR source checks passed **10 synthetic
Python/Pillow unit tests** in the isolated ChatGPT environment. This includes
preservation of original v0.1 archive behavior, a deterministic combined v0.2
ZIP, strict model/skin/PBR references, rejection of altered hashes and invalid
UVs, and symlink refusal. Official full-source conversion, Minecraft
Education loading/Global Resource isolation, texture atlas allocation, private
RAM, VRAM, frame time, FPS, multiplayer and five-client behavior are
**NOT_EXECUTED**. Source-only 75% texel math is not a measured memory saving.

References: [Mojang source](https://github.com/Mojang/bedrock-samples/tree/v1.26.30.5/resource_pack/textures/blocks),
[Mojang license](https://github.com/Mojang/bedrock-samples/blob/v1.26.30.5/LICENSE.md),
[vanilla path overrides](https://learn.microsoft.com/en-us/minecraft/creator/documents/overwritingassets),
[texture memory best practices](https://learn.microsoft.com/en-us/minecraft/creator/documents/practices/improvingperformanceandresourceusage),
[Minecraft Education custom pack support](https://edusupport.minecraft.net/hc/en-us/community/posts/4413216364308-Minecraft-Education-Edition).
