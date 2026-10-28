// Only these ambient vanilla effects may be overridden in the first preview.
// The Mojang samples are not vendored; build-time bytes must match each Git blob.
export const SOURCE_TAG = 'v1.26.30.5';
export const SOURCE_URL = `https://raw.githubusercontent.com/Mojang/bedrock-samples/${SOURCE_TAG}/resource_pack/particles/`;
export const LOD = Object.freeze({near: 16, far: 48, maximumReduction: 0.20});
export const PARTICLES = Object.freeze([
  Object.freeze({file: 'portal_reverse.json', identifier: 'minecraft:portal_reverse_particle',
    blob: 'a9d69a82d65e8b8afdf58b408370b833328954ff', format: '1.26.10'}),
  Object.freeze({file: 'nectar_drip.json', identifier: 'minecraft:nectar_drip_particle',
    blob: '37965aff002e62307a27faa383f1823554e68e37', format: '1.26.10'}),
  Object.freeze({file: 'dolphin_move_particle.json', identifier: 'minecraft:dolphin_move_particle',
    blob: '28a82946fb4adc078cd9a9b861f23d3d083644e1', format: '1.26.10'}),
]);
export const MANIFEST = Object.freeze({
  format_version: 2,
  header: {
    name: 'LazyTester Particle Tuning (Preview)',
    description: 'Optional distant ambient particle LOD; original Minecraft textures and gameplay.',
    uuid: '9b0a78e3-5272-4fcb-9283-802d94ce1ddb',
    version: [0, 1, 0],
    min_engine_version: [1, 26, 10],
  },
  modules: [{type: 'resources', uuid: '48627918-8d1c-4505-a1c5-cfb5013d4331', version: [0, 1, 0]}],
});
