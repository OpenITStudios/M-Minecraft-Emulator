import {createHash} from 'node:crypto';
import {LOD, MANIFEST, PARTICLES} from './catalog.mjs';

export function gitBlobSha(raw) {
  const bytes = Buffer.from(raw);
  return createHash('sha1').update(`blob ${bytes.length}\0`).update(bytes).digest('hex');
}

function validatedLod(lod) {
  const stops = [lod?.near, lod?.medium, lod?.far, lod?.veryFar];
  const rates = lod?.rates;
  if (stops.some((n) => !Number.isInteger(n) || n < 0 || n > 192)
    || stops.some((n, i) => i > 0 && n <= stops[i - 1])
    || !Array.isArray(rates) || rates.length !== 4
    || rates.some((n) => !Number.isFinite(n) || n < 0 || n > 1)
    || rates.some((n, i) => i > 0 && n >= rates[i - 1])
    || rates[3] !== 0) throw new Error('Unsupported LazyTester particle LOD profile');
  return {stops, rates};
}

export function emissionAtDistance(distance, lod = LOD) {
  if (!Number.isFinite(distance) || distance < 0) throw new Error('Invalid camera distance');
  const {stops, rates} = validatedLod(lod);
  let result = rates[0];
  for (let i = 1; i < stops.length; i++) {
    const fraction = Math.max(0, Math.min(1,
      (distance - stops[i - 1]) / (stops[i] - stops[i - 1])));
    result -= (rates[i - 1] - rates[i]) * fraction;
  }
  return Math.max(0, result);
}

function molangLod(lod) {
  const {stops, rates} = validatedLod(lod);
  let expression = rates[0].toFixed(2);
  for (let i = 1; i < stops.length; i++) {
    const delta = (rates[i - 1] - rates[i]).toFixed(2);
    expression += ` - ${delta} * query.camera_distance_range_lerp(${stops[i - 1]}, ${stops[i]})`;
  }
  return `math.max(0.0, ${expression})`;
}

function validOriginalRate(value, field) {
  if (typeof value === 'number' && Number.isFinite(value) && value > 0
    && (field !== 'max_particles' || Number.isSafeInteger(value))) return String(value);
  if (value && typeof value === 'object' && !Array.isArray(value)
    && Object.keys(value).sort().join(',') === 'expression,version'
    && typeof value.expression === 'string' && value.expression.trim().length > 0
    && value.expression.length <= 4096 && !/[\r\n\0]/.test(value.expression)
    && Number.isInteger(value.version) && [0, 13].includes(value.version)) return value.expression;
  throw new Error(`Unsupported source ${field} expression`);
}

// Fail closed on any changed file/hash, identifier, format or emission family.
// Preserve every vanilla appearance, lifetime, motion and texture property.
export function tuneParticle(raw, spec, lod = LOD) {
  if (gitBlobSha(raw) !== spec?.blob) throw new Error(`Vanilla source fingerprint mismatch: ${spec?.file}`);
  let original;
  try { original = JSON.parse(Buffer.from(raw).toString('utf8')); }
  catch { throw new Error(`Invalid vanilla particle JSON: ${spec.file}`); }
  const effect = original?.particle_effect;
  if (original?.format_version !== spec.format
    || effect?.description?.identifier !== spec.identifier
    || !effect.description.basic_render_parameters?.texture) {
    throw new Error(`Unsupported vanilla particle contract: ${spec.file}`);
  }
  const parts = effect.components;
  if (!parts || typeof parts !== 'object' || Array.isArray(parts)) {
    throw new Error(`Missing vanilla emitter components: ${spec.file}`);
  }
  const keys = Object.keys(parts).filter(key => key.startsWith('minecraft:emitter_rate_'));
  if (!['steady', 'manual'].includes(spec.mode) || keys.length !== 1
    || keys[0] !== `minecraft:emitter_rate_${spec.mode}`) {
    throw new Error(`Unsupported vanilla emitter family: ${spec.file}`);
  }
  const family = parts[keys[0]];
  const field = spec.mode === 'steady' ? 'spawn_rate' : 'max_particles';
  const originalRate = validOriginalRate(family?.[field], field);
  if (spec.mode === 'steady' && !Object.hasOwn(family, 'max_particles')) {
    throw new Error(`Missing steady emitter cap: ${spec.file}`);
  }
  const updated = `(${originalRate}) * (${molangLod(lod)})`;
  family[field] = typeof family[field] === 'number'
    ? updated : {...family[field], expression: updated};
  return original;
}

export function compileFiles(sources, catalog = PARTICLES) {
  if (!Array.isArray(catalog) || catalog.length === 0 || catalog.length > 15) {
    throw new Error('Unsupported particle catalog size');
  }
  const files = new Map([['manifest.json', Buffer.from(JSON.stringify(MANIFEST, null, 2) + '\n')]]);
  const names = new Set(), identifiers = new Set();
  for (const spec of catalog) {
    if (!/^[a-z0-9_]+\.json$/.test(spec.file) || names.has(spec.file)
      || identifiers.has(spec.identifier)) throw new Error('Duplicate or unsafe vanilla particle identity');
    names.add(spec.file); identifiers.add(spec.identifier);
    const raw = sources.get(spec.file);
    if (!raw) throw new Error(`Missing pinned vanilla source: ${spec.file}`);
    const tuned = tuneParticle(raw, spec);
    files.set(`particles/${spec.file}`, Buffer.from(JSON.stringify(tuned, null, 2) + '\n'));
  }
  return files;
}
