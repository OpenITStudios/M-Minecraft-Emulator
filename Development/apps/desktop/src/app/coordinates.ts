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

// Read-only candidate screening for a future per-client coordinate source.
// A matching synthetic hint is NEVER verified player/world evidence.
import {parseObservedProcess} from './bridge/observedProcess.js';
import type {ProfileSession, ProfileSlot} from './bridge/lazyTester.js';

export type CoordinateCandidateStatus =
  | 'NO_CANDIDATE'
  | 'MALFORMED_CANDIDATE'
  | 'NO_RESPONDING_CLIENT'
  | 'SLOT_MISMATCH'
  | 'PROCESS_MISMATCH'
  | 'STALE_OR_UNTIMED'
  | 'PLAYER_WORLD_ORIGIN_UNVERIFIED';

export interface CoordinateCandidateAssessment {
  readonly status: CoordinateCandidateStatus;
  readonly copyAllowed: false;
}

export interface CoordinateCandidateContext {
  readonly selectedSlot: ProfileSlot;
  readonly profile: ProfileSession | null;
  // Must be assigned by a future receiving owner, NOT by source payload.
  // Clock screening does not independently authenticate a game world.
  readonly receivedAtMonotonicMs: number | null;
  readonly nowMonotonicMs: number;
}

const MAX_CANDIDATE_AGE_MS = 2000;

function parseCoordinateProcessHint(value: unknown) {
  if (!record(value) ||
      !exact(value, ['pid', 'createdFileTime', 'sessionId', 'windowHandle'])) {
    throw new TypeError('Invalid coordinate process hint');
  }
  return parseObservedProcess({...value, responding: false});
}

const denyCandidate = (status: CoordinateCandidateStatus): CoordinateCandidateAssessment =>
  ({status, copyAllowed: false});

export function assessUnverifiedCoordinateCandidate(
  candidate: unknown,
  context: CoordinateCandidateContext,
): CoordinateCandidateAssessment {
  if (candidate === null || candidate === undefined) return denyCandidate('NO_CANDIDATE');
  if (!record(candidate) ||
      !exact(candidate, ['schema', 'slot', 'process', 'sample']) ||
      candidate.schema !== 1 ||
      typeof candidate.slot !== 'number' ||
      !Number.isInteger(candidate.slot) ||
      candidate.slot < 1 || candidate.slot > 5) {
    return denyCandidate('MALFORMED_CANDIDATE');
  }

  let hintedProcess: ReturnType<typeof parseCoordinateProcessHint>;
  try {
    hintedProcess = parseCoordinateProcessHint(candidate.process);
    parseUnboundCoordinateSample(candidate.sample);
  } catch {
    return denyCandidate('MALFORMED_CANDIDATE');
  }

  const {profile, selectedSlot, receivedAtMonotonicMs, nowMonotonicMs} = context;
  if (!profile || profile.slot !== selectedSlot ||
      profile.state !== 'RUNNING' || !profile.process ||
      profile.process.responding !== true) {
    return denyCandidate('NO_RESPONDING_CLIENT');
  }
  if (candidate.slot !== selectedSlot) return denyCandidate('SLOT_MISMATCH');

  let current: ReturnType<typeof parseObservedProcess>;
  try {
    current = parseObservedProcess(profile.process);
  } catch {
    return denyCandidate('NO_RESPONDING_CLIENT');
  }

  if (hintedProcess.pid !== current.pid ||
      hintedProcess.createdFileTime !== current.createdFileTime ||
      hintedProcess.sessionId !== current.sessionId ||
      hintedProcess.windowHandle !== current.windowHandle) {
    return denyCandidate('PROCESS_MISMATCH');
  }
  if (receivedAtMonotonicMs === null ||
      !Number.isFinite(receivedAtMonotonicMs) ||
      !Number.isFinite(nowMonotonicMs) ||
      receivedAtMonotonicMs < 0 ||
      nowMonotonicMs < receivedAtMonotonicMs ||
      nowMonotonicMs - receivedAtMonotonicMs > MAX_CANDIDATE_AGE_MS) {
    return denyCandidate('STALE_OR_UNTIMED');
  }

  // Process hints, local freshness and Script API output cannot prove which
  // world and player produced the sample. The native issuer does not exist.
  return denyCandidate('PLAYER_WORLD_ORIGIN_UNVERIFIED');
}
