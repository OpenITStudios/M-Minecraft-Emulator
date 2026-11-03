// Pure F3-style coordinate data and copy-text logic.
// This module does not read Minecraft, identify a player/world, authenticate
// a client slot, access the clipboard, dispatch a command, or run a HUD.
// A valid sample is still UNBOUND and must never be treated as live evidence.

export interface CoordinateVector3 {
  readonly x: number;
  readonly y: number;
  readonly z: number;
}

// Minecraft Script API Vector2: x is pitch, y is yaw.
// /tp accepts yaw first, pitch second.
export interface CoordinateRotation {
  readonly x: number;
  readonly y: number;
}

export interface UnboundCoordinateSample {
  readonly schema: 1;
  readonly position: CoordinateVector3;
  readonly rotation: CoordinateRotation;
  readonly dimensionId: string;
  readonly gameTick: number;
}

export type CoordinateCopyMode = 'XYZ' | 'BLOCK' | 'TELEPORT';

const MAX_ABSOLUTE_VALUE = 1_000_000_000;
const DIMENSION_ID = /^[a-z0-9_.-]+:[a-z0-9_.-]+(?:\/[a-z0-9_.-]+)*$/;

function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}

function exact(value: Record<string, unknown>, keys: readonly string[]): boolean {
  return Object.keys(value).length === keys.length &&
    keys.every(key => Object.hasOwn(value, key));
}

function boundedNumber(value: unknown): value is number {
  return typeof value === 'number' &&
    Number.isFinite(value) &&
    Math.abs(value) <= MAX_ABSOLUTE_VALUE;
}

function vector3(value: unknown): CoordinateVector3 {
  if (!record(value) || !exact(value, ['x', 'y', 'z']) ||
    !boundedNumber(value.x) || !boundedNumber(value.y) ||
    !boundedNumber(value.z)) {
    throw new TypeError('Invalid coordinate position');
  }
  return {x: value.x, y: value.y, z: value.z};
}

function rotation2(value: unknown): CoordinateRotation {
  if (!record(value) || !exact(value, ['x', 'y']) ||
    !boundedNumber(value.x) || !boundedNumber(value.y)) {
    throw new TypeError('Invalid coordinate rotation');
  }
  return {x: value.x, y: value.y};
}

// Converts a bounded, UNTRUSTED data object to an allowlisted representation.
// No source, slot, player, world, timestamp, process or proof assertion is
// accepted from the payload. Binding must be performed by an independent owner.
export function parseUnboundCoordinateSample(value: unknown): UnboundCoordinateSample {
  if (!record(value) ||
    !exact(value, ['schema', 'position', 'rotation', 'dimensionId', 'gameTick']) ||
    value.schema !== 1 ||
    typeof value.dimensionId !== 'string' ||
    value.dimensionId.length > 96 ||
    !DIMENSION_ID.test(value.dimensionId) ||
    typeof value.gameTick !== 'number' ||
    !Number.isSafeInteger(value.gameTick) || value.gameTick < 0) {
    throw new TypeError('Invalid unbound coordinate sample');
  }

  return {
    schema: 1,
    position: vector3(value.position),
    rotation: rotation2(value.rotation),
    dimensionId: value.dimensionId,
    gameTick: value.gameTick,
  };
}

// Keep copy values unrounded, invariant-culture and in plain decimal notation.
// Minecraft commands must not receive locale commas or exponential numbers.
// Display text is intentionally rounded separately below.
function decimal(value: number): string {
  if (!boundedNumber(value)) throw new TypeError('Invalid coordinate number');
  if (value === 0) return '0'; // includes negative zero
  const raw = String(value);
  const exponential = /^(-?)(\d+)(?:\.(\d+))?e([+-]?\d+)$/.exec(raw);
  if (!exponential) return raw;

  const sign = exponential[1]!;
  const whole = exponential[2]!;
  const fraction = exponential[3] ?? '';
  const digits = whole + fraction;
  const point = whole.length + Number(exponential[4]!);
  if (point <= 0) return sign + '0.' + '0'.repeat(-point) + digits;
  if (point >= digits.length) return sign + digits + '0'.repeat(point - digits.length);
  return sign + digits.slice(0, point) + '.' + digits.slice(point);
}

function positionText(position: CoordinateVector3): string {
  return [position.x, position.y, position.z].map(decimal).join(' ');
}

export function blockCoordinates(position: CoordinateVector3): CoordinateVector3 {
  return {
    x: Math.floor(position.x),
    y: Math.floor(position.y),
    z: Math.floor(position.z),
  };
}

export function chunkCoordinates(position: CoordinateVector3): Readonly<{x: number; z: number}> {
  return {
    x: Math.floor(position.x / 16),
    z: Math.floor(position.z / 16),
  };
}

export function coordinateDisplayXYZ(value: unknown): string {
  const {position} = parseUnboundCoordinateSample(value);
  return [position.x, position.y, position.z].map(number => {
    const rounded = number.toFixed(3);
    return rounded === '-0.000' ? '0.000' : rounded;
  }).join(' / ');
}

// Returns text only. Callers must independently verify client/world identity,
// freshness, and active selection before exposing a Copy button or clipboard.
export function coordinateCopyText(value: unknown, mode: CoordinateCopyMode): string {
  const sample = parseUnboundCoordinateSample(value);
  switch (mode) {
    case 'XYZ':
      return positionText(sample.position);
    case 'BLOCK':
      return positionText(blockCoordinates(sample.position));
    case 'TELEPORT':
      // Same-dimension self teleport only; never executes the command.
      return '/tp @s ' + positionText(sample.position) + ' ' +
        decimal(sample.rotation.y) + ' ' + decimal(sample.rotation.x);
    default:
      throw new TypeError('Unsupported coordinate copy mode');
  }
}

/**
 * Explicitly pasted text, NOT a live Minecraft or authenticated player feed.
 * Bedrock's native Copy Coordinate UI may produce XYZ; Education 26.33
 * support has not been verified. This has NO clipboard or game side effects.
 */
export interface UserPastedCoordinates {
  readonly origin: 'USER_PASTED_UNVERIFIED';
  readonly position: CoordinateVector3;
}
export type PastedCoordinateOutput = 'XYZ' | 'BLOCK' | 'TELEPORT_POSITION_ONLY';

// Exactly three signed ASCII decimals on a single line; no command text,
// player names, floating exponents, locale ambiguity or extra metadata.
const PASTED_XYZ = /^([+-]?(?:\d+(?:\.\d+)?|\.\d+))(?:[ \t]+|[ \t]*,[ \t]*)([+-]?(?:\d+(?:\.\d+)?|\.\d+))(?:[ \t]+|[ \t]*,[ \t]*)([+-]?(?:\d+(?:\.\d+)?|\.\d+))$/;

export function parseUserPastedCoordinates(raw: unknown): UserPastedCoordinates {
  if (typeof raw !== 'string' || raw.length > 128 || /[\r\n]/.test(raw)) {
    throw new TypeError('Paste one XYZ coordinate line');
  }
  const matches = PASTED_XYZ.exec(raw.trim());
  if (!matches) throw new TypeError('Invalid pasted XYZ coordinates');
  const x = Number(matches[1]);
  const y = Number(matches[2]);
  const z = Number(matches[3]);
  if (![x, y, z].every(boundedNumber)) {
    throw new TypeError('Pasted XYZ coordinate outside supported bounds');
  }
  return {origin: 'USER_PASTED_UNVERIFIED', position: {x, y, z}};
}

export function formatUserPastedCoordinates(raw: unknown, output: PastedCoordinateOutput): string {
  const {position} = parseUserPastedCoordinates(raw);
  switch (output) {
    case 'XYZ': return positionText(position);
    case 'BLOCK': return positionText(blockCoordinates(position));
    case 'TELEPORT_POSITION_ONLY':
      // No guessed yaw/pitch, world or dimension. TEXT ONLY, not executed.
      return '/tp @s ' + positionText(position);
    default: throw new TypeError('Unsupported pasted coordinate format');
  }
}

export function displayUserPastedCoordinates(raw: unknown): string {
  const {position} = parseUserPastedCoordinates(raw);
  return [position.x, position.y, position.z].map(value => {
    const s = value.toFixed(3);
    return s === '-0.000' ? '0.000' : s;
  }).join(' / ');
}
