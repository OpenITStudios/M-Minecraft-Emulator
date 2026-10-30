// Pure, stateless peer-to-peer planning for the existing five native game slots.
// Plans are instructions for an AI, NEVER game execution receipts or background input.
export const TEST_MODES = Object.freeze(['human-plus-four-ai', 'five-ai']);
export const PLAYER_ACTIONS = Object.freeze([
  'move_forward', 'move_backward', 'strafe_left', 'strafe_right',
  'jump', 'sneak', 'attack', 'use', 'look_left', 'look_right', 'wait',
]);
const ACTIONS = new Set(PLAYER_ACTIONS);
const STATES = new Set(['STOPPED','STARTING','RUNNING','NOT_RESPONDING','ATTENTION']);
const MAX_ACTIONS = 32;
const MAX_OFFSET_MS = 30000;
const MAX_DURATION_MS = 2000;

export function slotsForMode(mode) {
  if (!TEST_MODES.includes(mode)) throw new TypeError('Unknown multiplayer test mode');
  return Object.freeze({
    hostSlot: 1,
    humanSlots: mode === 'human-plus-four-ai' ? [1] : [],
    aiSlots: mode === 'human-plus-four-ai' ? [2,3,4,5] : [1,2,3,4,5],
  });
}

function record(value,allowed) {
  if (!value || typeof value !== 'object' || Array.isArray(value) ||
      Object.keys(value).some(key => !allowed.includes(key))) {
    throw new TypeError('Invalid structured action');
  }
}
function validSlot(slot) {
  if (!Number.isInteger(slot) || slot < 1 || slot > 5) {
    throw new TypeError('Profile slot must be MC01 through MC05');
  }
  return slot;
}
function validNativeProfiles(snapshot) {
  if (!snapshot || snapshot.schema !== 1 || snapshot.ok !== true ||
      snapshot.operation !== 'inspect' || !Array.isArray(snapshot.profiles) ||
      snapshot.profiles.length !== 5) {
    throw new TypeError('A current native Inspect result is required');
  }
  snapshot.profiles.forEach((profile,index) => {
    if (!profile || profile.slot !== index + 1 || !STATES.has(profile.state) ||
        typeof profile.hasWindow !== 'boolean' ||
        typeof profile.responding !== 'boolean' ||
        typeof profile.needsAttention !== 'boolean') {
      throw new TypeError('Inconsistent native profile observation');
    }
  });
}

// Derived solely from the retained controller snapshot. It cannot determine
// whether MC01 hosts a world, peers joined it, or a player can receive keys.
export function planPeerSession(mode,snapshot) {
  const ownership = slotsForMode(mode);
  validNativeProfiles(snapshot);
  const profiles = snapshot.profiles.map(profile => {
    const role = ownership.humanSlots.includes(profile.slot) ? 'HUMAN' : 'AI';
    let next;
    if (profile.state === 'RUNNING' && profile.hasWindow && profile.responding) {
      next = profile.slot === 1 ? 'OBSERVE_WORLD_HOST_IN_GAME' : 'OBSERVE_WORLD_JOIN_IN_GAME';
    } else if (profile.state === 'STOPPED' || profile.state === 'ATTENTION') {
      next = role === 'HUMAN' ? 'HUMAN_LAUNCH_REQUIRED' : 'REQUEST_LAUNCH_IF_NATIVE_ADMITS';
    } else {
      next = 'WAIT_OR_INSPECT_NATIVE_SESSION';
    }
    return {
      slot: profile.slot, role, nativeState: profile.state,
      responsiveWindow: profile.hasWindow && profile.responding,
      next, worldPresence: 'NOT_OBSERVED', gameInput: 'NOT_VERIFIED',
    };
  });
  return {
    schema: 1, mode, hostSlot: 1,
    connection: 'MINECRAFT_EDUCATION_PEER_TO_PEER',
    nativeObservation: 'CURRENT_CONTROLLER_INSPECT_NOT_GAMEPLAY',
    hostWorld: 'NOT_OBSERVED',
    peersJoined: 'NOT_OBSERVED',
    independentBackgroundInput: 'NOT_VERIFIED',
    aiSlots: ownership.aiSlots, humanSlots: ownership.humanSlots, profiles,
    execution: 'NOT_PERFORMED',
    note: 'Launch and login are not host/join proof. World actions must be observed in Minecraft.',
  };
}

// This is a bounded multi-player INTENT contract, reusable when an actual
// per-client input transport is proven. The current bridge does not dispatch it.
export function planPlayerActions(mode,actions) {
  const ownership = slotsForMode(mode);
  if (!Array.isArray(actions) || actions.length === 0 || actions.length > MAX_ACTIONS) {
    throw new TypeError('Provide 1-32 bounded player actions');
  }
  const scheduled = [];
  for (const action of actions) {
    record(action,['slot','kind','atMs','durationMs']);
    const slot = validSlot(action.slot);
    if (!ownership.aiSlots.includes(slot)) {
      throw new TypeError('The human-owned Minecraft profile cannot receive AI actions');
    }
    if (!ACTIONS.has(action.kind)) throw new TypeError('Unsupported player action');
    if (!Number.isInteger(action.atMs) || action.atMs < 0 || action.atMs > MAX_OFFSET_MS) {
      throw new TypeError('Action offset must be 0-30000 ms');
    }
    const upperDuration = action.kind === 'wait' ? 5000 : MAX_DURATION_MS;
    if (!Number.isInteger(action.durationMs) || action.durationMs < 50 ||
        action.durationMs > upperDuration ||
        action.atMs + action.durationMs > MAX_OFFSET_MS + 5000) {
      throw new TypeError('Action duration is unbounded or invalid');
    }
    const endMs = action.atMs + action.durationMs;
    // No two overlapping actions can address one game slot: input providers
    // cannot safely release keys when actions compete on the same client.
    if (scheduled.some(previous => previous.slot === slot &&
        action.atMs < previous.endMs && previous.atMs < endMs)) {
      throw new TypeError('Overlapping actions for one Minecraft profile are refused');
    }
    scheduled.push({
      slot, kind: action.kind, startMs: action.atMs,
      durationMs: action.durationMs, endMs,
    });
  }
  scheduled.sort((a,b) => a.startMs - b.startMs || a.slot - b.slot);
  let peak = 0;
  for (const action of scheduled) {
    const active = scheduled.filter(other =>
      other.startMs <= action.startMs && other.endMs > action.startMs).length;
    peak = Math.max(peak,active);
  }
  return {
    schema: 1, mode, hostSlot: 1,
    humanSlots: ownership.humanSlots, aiSlots: ownership.aiSlots,
    actions: scheduled, maximumConcurrentIntents: peak,
    execution: 'NOT_DISPATCHED',
    gate: 'BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND',
    noGlobalKeyboardMouse: true,
    note: 'This is a validated intent schedule only. It cannot move a Minecraft client.',
  };
}
