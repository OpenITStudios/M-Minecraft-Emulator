// Pure, unauthenticated candidate-screening layer. No I/O or proof issuer.
import {parseUnboundCoordinateSample, type UnboundCoordinateSample} from './coordinates.js';

function record(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value);
}
function exact(value: Record<string, unknown>, keys: readonly string[]): boolean {
  return Object.keys(value).length === keys.length &&
    keys.every(key => Object.hasOwn(value, key));
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

export type CoordinateSequenceStatus =
  | 'NO_PREVIOUS_CANDIDATE'
  | 'MALFORMED_SEQUENCE'
  | 'SLOT_CHANGED'
  | 'PROCESS_REPLACED'
  | 'DIMENSION_CHANGED'
  | 'TICK_REPLAY_OR_RESET'
  | 'RECEIPT_SEQUENCE_INVALID'
  | 'CONTINUOUS_BUT_UNVERIFIED';

export interface CoordinateSequenceAssessment {
  readonly status: CoordinateSequenceStatus;
  readonly copyAllowed: false;
}
const sequenceDeny = (status: CoordinateSequenceStatus): CoordinateSequenceAssessment =>
  ({status, copyAllowed: false});

// Only parses an untrusted hint. There is no player/world authentication.
function parseSequenceFrame(value: unknown): {
  slot: number;
  process: ReturnType<typeof parseCoordinateProcessHint>;
  sample: UnboundCoordinateSample;
} {
  if (!record(value) ||
      !exact(value, ['schema', 'slot', 'process', 'sample']) ||
      value.schema !== 1 ||
      typeof value.slot !== 'number' ||
      !Number.isInteger(value.slot) ||
      value.slot < 1 || value.slot > 5) {
    throw new TypeError('Invalid coordinate sequence frame');
  }
  return {
    slot: value.slot,
    process: parseCoordinateProcessHint(value.process),
    sample: parseUnboundCoordinateSample(value.sample),
  };
}

/**
 * Source-only continuity screening: never authorizes clipboard.
 * Receipt times, if available, must be from one receiver-side monotonic
 * clock, NOT from data reported by Minecraft or an unauthenticated log.
 */
export function assessUnverifiedCoordinateSequence(
  previous: unknown,
  current: unknown,
  previousReceivedAtMonotonicMs: number | null,
  currentReceivedAtMonotonicMs: number | null,
): CoordinateSequenceAssessment {
  if (previous === null || previous === undefined) {
    return sequenceDeny('NO_PREVIOUS_CANDIDATE');
  }
  let a: ReturnType<typeof parseSequenceFrame>;
  let b: ReturnType<typeof parseSequenceFrame>;
  try {
    a = parseSequenceFrame(previous);
    b = parseSequenceFrame(current);
  } catch {
    return sequenceDeny('MALFORMED_SEQUENCE');
  }
  if (a.slot !== b.slot) return sequenceDeny('SLOT_CHANGED');
  if (a.process.pid !== b.process.pid ||
      a.process.createdFileTime !== b.process.createdFileTime ||
      a.process.sessionId !== b.process.sessionId ||
      a.process.windowHandle !== b.process.windowHandle) {
    return sequenceDeny('PROCESS_REPLACED');
  }
  if (a.sample.dimensionId !== b.sample.dimensionId) {
    return sequenceDeny('DIMENSION_CHANGED');
  }
  if (b.sample.gameTick <= a.sample.gameTick) {
    return sequenceDeny('TICK_REPLAY_OR_RESET');
  }
  if (previousReceivedAtMonotonicMs === null ||
      currentReceivedAtMonotonicMs === null ||
      !Number.isFinite(previousReceivedAtMonotonicMs) ||
      !Number.isFinite(currentReceivedAtMonotonicMs) ||
      previousReceivedAtMonotonicMs < 0 ||
      currentReceivedAtMonotonicMs < previousReceivedAtMonotonicMs ||
      currentReceivedAtMonotonicMs - previousReceivedAtMonotonicMs > 2000) {
    return sequenceDeny('RECEIPT_SEQUENCE_INVALID');
  }
  return sequenceDeny('CONTINUOUS_BUT_UNVERIFIED');
}
