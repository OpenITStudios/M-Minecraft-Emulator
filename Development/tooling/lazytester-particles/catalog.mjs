// Only ambient vanilla effects are included. Mojang source is not vendored;
// every file must match the exact Git blob recorded against the sample tag.
// The approved 75/50/25/0 curve is linear and can use one distance query.
export const SOURCE_TAG = 'v1.26.30.5';
export const SOURCE_URL = `https://raw.githubusercontent.com/Mojang/bedrock-samples/${SOURCE_TAG}/resource_pack/particles/`;

// Camera distance is measured from the particle emitter, not each particle.
// Near 75%, medium 50%, far 25%, very far 0%, with smooth interpolation.
export const LOD = Object.freeze({
  near: 16, medium: 32, far: 48, veryFar: 64,
  rates: Object.freeze([0.75, 0.50, 0.25, 0.00]),
});

// "steady" scales particles/second AND their active ceiling; "manual" adjusts
// only the per-emitter active ceiling, NOT event-driven emission frequency.
export const PARTICLES = Object.freeze([
  Object.freeze({file: 'portal_reverse.json', identifier: 'minecraft:portal_reverse_particle',
    blob: 'a9d69a82d65e8b8afdf58b408370b833328954ff', format: '1.26.10', mode: 'steady'}),
  Object.freeze({file: 'nectar_drip.json', identifier: 'minecraft:nectar_drip_particle',
    blob: '37965aff002e62307a27faa383f1823554e68e37', format: '1.26.10', mode: 'steady'}),
  Object.freeze({file: 'dolphin_move_particle.json', identifier: 'minecraft:dolphin_move_particle',
    blob: '28a82946fb4adc078cd9a9b861f23d3d083644e1', format: '1.26.10', mode: 'steady'}),
  Object.freeze({file: 'cherry_leaves_particle.json', identifier: 'minecraft:cherry_leaves_particle',
    blob: '7bcd07e1b98caec3c737d4ea06e44f0addb8589f', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'spore_blossom_shower.json', identifier: 'minecraft:spore_blossom_shower_particle',
    blob: '72c55723c60f7b4f4cc6056246b5a5accae49424', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'spore_blossom_ambient_block_actor.json', identifier: 'minecraft:spore_blossom_ambient_particle',
    blob: 'ec08d1147b4b11a63efadc12e58c6879d616c399', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'campfire_smoke.json', identifier: 'minecraft:campfire_smoke_particle',
    blob: '4cdc63b75a411fe9e8a24239520b234148a5a6d6', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'campfire_smoke_tall.json', identifier: 'minecraft:campfire_tall_smoke_particle',
    blob: 'ce0dd9c346c01a5a23384a6674e32c5cd9a26dba', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'rain_splash.json', identifier: 'minecraft:rain_splash_particle',
    blob: '268432249fef793e2dee9ffb3a33d68f43aae4a7', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'mycelium_dust.json', identifier: 'minecraft:mycelium_dust_particle',
    blob: 'e0ab0f8541c61ff7dd6fac702772634f7f726a1d', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'pale_oak_leaves_particle.json', identifier: 'minecraft:pale_oak_leaves_particle',
    blob: '1f70fa805d0852207575de7004231294b5dd11e2', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'biome_tinted_leaves_particle.json', identifier: 'minecraft:biome_tinted_leaves_particle',
    blob: '115de7836e6d2906f421a554c250894d73776864', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'water_drip.json', identifier: 'minecraft:water_drip_particle',
    blob: '7a53662c96a4f58604c577e58530c6a725b2dc52', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'lava_drip.json', identifier: 'minecraft:lava_drip_particle',
    blob: 'afa0b83897a0bcfa37aa6e12888815415e807112', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'dripstone_water_drip.json', identifier: 'minecraft:stalactite_water_drip_particle',
    blob: '9ea3d6598b11f2cd8d0c632c5347404e970e4169', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'dripstone_lava_drip.json', identifier: 'minecraft:stalactite_lava_drip_particle',
    blob: '751e078980b833bd09172255f9d9325ab9fa9d1d', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'honey_drip.json', identifier: 'minecraft:honey_drip_particle',
    blob: '7f4727aac744995563ccccc81685911e0749bdce', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'obsidian_tear.json', identifier: 'minecraft:obsidian_tear_particle',
    blob: 'b27941513b6c4645eadaad0ef0ae16618bfa72e3', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'snowflake.json', identifier: 'minecraft:snowflake_particle',
    blob: 'fa29c6597077b7d99726931076b1efec67da62f6', format: '1.26.10', mode: 'manual'}),
  Object.freeze({file: 'endrod.json', identifier: 'minecraft:endrod',
    blob: '2806b68c02f2e4dc9d385874f88fc50e77a8ae69', format: '1.26.10', mode: 'manual'}),
]);

export const MANIFEST = Object.freeze({
  format_version: 2,
  header: {
    name: 'LazyTester Particle Tuning (Preview)',
    description: 'Optional ambient LOD 75/50/25/0: 3 steady rates; 17 conditional manual caps; vanilla textures.',
    uuid: '9b0a78e3-5272-4fcb-9283-802d94ce1ddb',
    version: [0, 3, 0],
    min_engine_version: [1, 26, 10],
  },
  modules: [{type: 'resources', uuid: '48627918-8d1c-4505-a1c5-cfb5013d4331', version: [0, 3, 0]}],
});
