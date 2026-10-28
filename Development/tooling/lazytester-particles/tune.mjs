import {createHash} from 'node:crypto';
import {LOD, MANIFEST, PARTICLES} from './catalog.mjs';

export function gitBlobSha(raw) {
  const bytes = Buffer.from(raw);
  return createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
}

// A source mismatch is a feature/version mismatch, not an invitation to guess.
// Only a uniquely identified steady emitter gets a distance-dependent rate.
export function tuneParticle(raw, spec, lod = LOD) {
  if (gitBlobSha(raw) !== spec.blob) throw new Error(`Vanilla source fingerprint mismatch: ${spec.file}`);
  let original;
  try { original = JSON.parse(Buffer.from(raw).toString('utf8')); }
  catch { throw new Error(`Invalid vanilla particle JSON: ${spec.file}`); }
  const effect = original?.particle_effect;
  if (original?.format_version !== spec.format || effect?.description?.identifier !== spec.identifier
    || !effect.description.basic_render_parameters?.texture) {
    throw new Error(`Unsupported vanilla particle contract: ${spec.file}`);
  }
  const parts = effect.components;
  const steady = parts?.['minecraft:emitter_rate_steady'];
  if (!steady || 'minecraft:emitter_rate_instant' in parts || 'minecraft:emitter_rate_manual' in parts
    || !('max_particles' in steady)) throw new Error(`Not a solitary steady emitter: ${spec.file}`);
  const old = steady.spawn_rate;
  if (!Number.isInteger(lod.near) || !Number.isInteger(lod.far) || lod.near < 0 || lod.far <= lod.near
    || lod.far > 192 || !(lod.maximumReduction > 0 && lod.maximumReduction <= 0.20)) {
    throw new Error('Unsupported LazyTester particle LOD bounds');
  }
  const expression = typeof old === 'number' && Number.isFinite(old) && old > 0 ? String(old)
    : old && typeof old === 'object' && Object.keys(old).sort().join(',') === 'expression,version'
      && typeof old.expression === 'string' && old.expression.length > 0
      && Number.isInteger(old.version) && [0, 13].includes(old.version) ? old.expression : null;
  if (!expression || /[\r\n\0]/.test(expression)) throw new Error(`Unsupported spawn expression: ${spec.file}`);
  const multiplier = `1.0 - ${lod.maximumReduction.toFixed(2)} * query.camera_distance_range_lerp(${lod.near}, ${lod.far})`;
  const nextRate = `(${expression}) * (${multiplier})`;
  steady.spawn_rate = typeof old === 'number' ? nextRate : {...old, expression: nextRate};
  return original;
}

export function compileFiles(sources, catalog = PARTICLES) {
  const files = new Map([['manifest.json', Buffer.from(JSON.stringify(MANIFEST, null, 2) + '\n')]]);
  for (const spec of catalog) {
    const raw = sources.get(spec.file);
    if (!raw) throw new Error(`Missing pinned vanilla source: ${spec.file}`);
    const tuned = tuneParticle(raw, spec);
    files.set(`particles/${spec.file}`, Buffer.from(JSON.stringify(tuned, null, 2) + '\n'));
  }
  if (files.size !== catalog.length + 1) throw new Error('Duplicate particle output');
  return files;
}
