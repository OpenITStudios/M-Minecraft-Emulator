# LazyTester Idle Texture Lite 8×8 — optional offline preview v0.1

**For testing only.** This is a build-time texture override pack, not a game
plugin, automatic installation, background service, monitor, memory cleaner or
per-focus runtime quality switch. It does not access MC01–MC05 storage, world
files, credentials or Minecraft. Particle Tuning v0.3 remains separate.

**Final temporary scope (29 October 2026): block textures only.** Vanilla
entity textures, geometry, UVs, animation and PBR remain untouched. The former
opt-in Ghast extension was removed from active code; Git history retains its
experimental provenance. This source preview never changes a profile by itself.

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
- Only **single-frame, noninterlaced, 16×16 PNG block albedo** entries become
  8×8 PNGs at the **same relative texture paths**. Preserve alpha and color
  by using Pillow's 2×2 BOX area downsampling. Animated strips (such as
  `water_still` and `lava_still` at 16×512), TGA PBR textures, UI, entities,
  item animations, particle images and source metadata are not transformed.
- Bundle one resource manifest and the selected PNGs only. Do not publish or
  automatically install the generated Mojang-derived textures. Respect Mojang's
  Minecraft EULA and make only private, properly authorized previews.

A 16×16 RGBA image occupies **1,024 decoded pixel bytes** versus **256 bytes**
for 8×8, before atlases, mipmaps, GPU layout, streaming and other engine effects.
That's **75% fewer source texels per selected image**, not 75% lower Minecraft
RAM. Even 1,200 such images would save only about **0.88 MiB** of theoretical
RGBA8 image bytes **per client** if the originals were all loaded distinctly;
Minecraft normally atlases block images, so actual RAM or VRAM savings may be
smaller, zero or otherwise different. **No entity, UI or item texture
optimizations are planned in this temporary scope.**

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

The included builder/PNG tests ran in an isolated ChatGPT Python 3/Pillow
sandbox against synthetic assets. Official full-source conversion, Minecraft
Education loading/Global Resource isolation, texture atlas allocation, private
RAM, VRAM, frame time, FPS, multiplayer and five-client behavior are
**NOT_EXECUTED**. Source-only 75% texel math is not a measured memory saving.

References: [Mojang source](https://github.com/Mojang/bedrock-samples/tree/v1.26.30.5/resource_pack/textures/blocks),
[Mojang license](https://github.com/Mojang/bedrock-samples/blob/v1.26.30.5/LICENSE.md),
[vanilla path overrides](https://learn.microsoft.com/en-us/minecraft/creator/documents/overwritingassets),
[texture memory best practices](https://learn.microsoft.com/en-us/minecraft/creator/documents/practices/improvingperformanceandresourceusage),
[Minecraft Education custom pack support](https://edusupport.minecraft.net/hc/en-us/community/posts/4413216364308-Minecraft-Education-Edition).
