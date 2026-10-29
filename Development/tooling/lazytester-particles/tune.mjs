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

// The approved tiers are collinear: one clamped camera-distance query is
// equivalent to three lerps + max. Custom non-linear curves retain the
// original safe piecewise expression.
export function molangLod(lod = LOD) {
  const {stops, rates} = validatedLod(lod);
  const slope = rates[0] / (stops[3] - stops[0]);
  const affine = rates.every((v, i) =>
    Math.abs(v - (rates[0] - slope * (stops[i] - stops[0]))) < 1e-12);
  if (affine) {
    return `${rates[0].toFixed(2)} * (1.0 - query.camera_distance_range_lerp(${stops[0]}, ${stops[3]}))`;
  }
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
function scaledField(family, field, multiplier) {
  const original = family[field];
  const expression = `(${validOriginalRate(original, field)}) * (${multiplier})`;
  family[field] = typeof original === 'number' ? expression : {...original, expression};
}

// Steady: adjust both spawn rate and live count. Manual: live count only.
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
  if (!family || typeof family !== 'object' || Array.isArray(family)) {
    throw new Error(`Invalid vanilla emitter: ${spec.file}`);
  }
  const multiplier = molangLod(lod);
  if (spec.mode === 'steady') scaledField(family, 'spawn_rate', multiplier);
  scaledField(family, 'max_particles', multiplier);
  return original;
}

export function compileFiles(sources, catalog = PARTICLES) {
  if (!(sources instanceof Map) || !Array.isArray(catalog) || catalog.length === 0 || catalog.length > 31) {
    throw new Error('Unsupported particle catalog size');
  }
  const files = new Map([['manifest.json', Buffer.from(JSON.stringify(MANIFEST, null, 2) + '\n')]]);
  const names = new Set(), identifiers = new Set();
  for (const spec of catalog) {
    if (!spec || !/^[a-z0-9_]+\.json$/.test(spec.file) || typeof spec.identifier !== 'string' || names.has(spec.file)
      || identifiers.has(spec.identifier)) throw new Error('Duplicate or unsafe vanilla particle identity');
    names.add(spec.file); identifiers.add(spec.identifier);
    const raw = sources.get(spec.file);
    if (!raw) throw new Error(`Missing pinned vanilla source: ${spec.file}`);
    const tuned = tuneParticle(raw, spec);
    files.set(`particles/${spec.file}`, Buffer.from(JSON.stringify(tuned, null, 2) + '\n'));
  }
  return files;
}
