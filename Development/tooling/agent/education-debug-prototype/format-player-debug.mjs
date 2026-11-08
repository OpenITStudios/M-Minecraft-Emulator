// Source-only, side-effect-free in-game debug-data projection.
// Intended for review by the canonical team coords/display/form owners.
// This does not subscribe to Minecraft input, render UI, handle F3,
// access Windows clipboard, or select a player by name/slot.
const DIMENSIONS = Object.freeze({
  'minecraft:overworld': 'Overworld',
  'minecraft:nether': 'Nether',
  'minecraft:the_end': 'The End',
});
function finite(value, label) {
  if (typeof value !== 'number' || !Number.isFinite(value) ||
      Math.abs(value) > 30_000_000) {
    throw new TypeError('Invalid in-game ' + label);
  }
  return value;
}
function block(value) { return Math.floor(value); }
function chunk(value) { return Math.floor(block(value) / 16); }
function insideChunk(value) { return block(value) - chunk(value) * 16; }
function formatted(value) { return value.toFixed(2); }
function facing(yaw) {
  const normalized = ((yaw % 360) + 360) % 360;
  if (normalized < 45 || normalized >= 315) return 'South (+Z)';
  if (normalized < 135) return 'West (-X)';
  if (normalized < 225) return 'North (-Z)';
  return 'East (+X)';
}

// The caller must supply location, rotation and dimension from the SAME
// original Minecraft Player. Never accept desktop MCxx labels or another
// player's telemetry. Returns text only; NOT a clipboard action or HUD.
export function formatPlayerDebugSnapshot({location, rotation, dimensionId}) {
  if (!location || typeof location !== 'object' ||
      !rotation || typeof rotation !== 'object' ||
      !Object.hasOwn(DIMENSIONS, dimensionId)) {
    throw new TypeError('Unverified in-game player debug context');
  }
  const x = finite(location.x, 'X'), y = finite(location.y, 'Y'),
    z = finite(location.z, 'Z');
  // Bedrock getRotation(): x = pitch, y = yaw.
  const pitch = finite(rotation.x, 'pitch');
  const yaw = finite(rotation.y, 'yaw');
  if (pitch < -90 || pitch > 90) {
    throw new TypeError('Invalid in-game player pitch');
  }
  const bx = block(x), by = block(y), bz = block(z);
  const copyCoordinatesText = bx + ' ' + by + ' ' + bz;
  const lines = Object.freeze([
    'Minecraft Education | Debug',
    'XYZ: ' + formatted(x) + ' / ' + formatted(y) + ' / ' + formatted(z),
    'Block: ' + bx + ' / ' + by + ' / ' + bz,
    'Chunk: ' + chunk(x) + ' / ' + chunk(z),
    'Within chunk: ' + insideChunk(x) + ' / ' + insideChunk(z),
    'Facing: ' + facing(yaw),
    'Rotation: yaw ' + formatted(yaw) + ' / pitch ' + formatted(pitch),
    'Dimension: ' + DIMENSIONS[dimensionId],
  ]);
  return Object.freeze({lines, copyCoordinatesText});
}
