// Pure, stateless peer-to-peer planning for the existing five native game slots.
// Plans are instructions for an AI, NEVER game execution receipts or background input.
export const TEST_MODES = Object.freeze(['human-plus-four-ai', 'five-ai']);
export const PLAYER_ACTIONS = Object.freeze([
  'move_forward', 'move_backward', 'strafe_left', 'strafe_right',
  'jump', 'sneak', 'attack', 'use', 'look_left', 'look_right',
  'look_relative', 'wait',
]);
const ACTIONS = new Set(PLAYER_ACTIONS);
// Independent controls may overlap on the SAME client without sharing keys.
export const INPUT_CHANNELS=Object.freeze({
  move_forward:'move_longitudinal',move_backward:'move_longitudinal',
  strafe_left:'move_lateral',strafe_right:'move_lateral',
  look_left:'camera',look_right:'camera',look_relative:'camera',
  jump:'jump',sneak:'stance',attack:'attack',use:'use',wait:'exclusive',
});
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

// Compact authoring only: one group describes independent held controls and
// an optional relative camera path. Expand to the existing canonical actions;
// the existing planner remains the sole mode/channel/timing admission owner.
// No game presets, random jitter, input transport or provider capability.
export function expandActionGroups(groups) {
  if(!Array.isArray(groups)||groups.length<1||groups.length>MAX_ACTIONS) {
    throw new TypeError('Provide 1-32 bounded action groups');
  }
  const actions=[];
  const append=action=>{
    if(actions.length>=MAX_ACTIONS)throw new TypeError('Expanded action limit exceeded');
    actions.push(action);
  };
  for(const group of groups) {
    record(group,['slot','atMs','durationMs','hold','look']);
    const {slot,atMs,durationMs,hold=[],look}=group;
    validSlot(slot);
    if(!Number.isInteger(atMs)||atMs<0||atMs>MAX_OFFSET_MS||
       !Number.isInteger(durationMs)||durationMs<50||durationMs>MAX_DURATION_MS||
       !Array.isArray(hold)||hold.length>8||new Set(hold).size!==hold.length||
       (hold.length===0&&look===undefined)) {
      throw new TypeError('Invalid compact action group');
    }
    for(const kind of hold) {
      if(!ACTIONS.has(kind)||kind==='wait'||INPUT_CHANNELS[kind]==='camera') {
        throw new TypeError('Unsupported held control');
      }
      append({slot,kind,atMs,durationMs});
    }
    if(look===undefined)continue;
    record(look,['deltaX','deltaY','steps','curve']);
    const {deltaX,deltaY,curve='smooth'}=look;
    // Default to a small bounded set of samples, with user-selected density
    // when needed. These are authored samples, not a verified frame rate.
    const steps=look.steps===undefined?Math.ceil(durationMs/100):look.steps;
    if(!Number.isInteger(deltaX)||!Number.isInteger(deltaY)||
       Math.abs(deltaX)>512||Math.abs(deltaY)>512||(deltaX===0&&deltaY===0)||
       !Number.isInteger(steps)||steps<1||steps>MAX_ACTIONS||
       Math.floor(durationMs/steps)<50||Math.ceil(durationMs/steps)>250||
       !['linear','smooth'].includes(curve)) {
      throw new TypeError('Invalid compact camera path');
    }
    // Quantize cumulative positions instead of rounding each relative delta:
    // the final sum is exactly the authored integer displacement. An eased
    // curve is deterministic acceleration/deceleration, not gameplay AI.
    let x=0,y=0;
    for(let i=0;i<steps;i++) {
      const t=(i+1)/steps;
      const fraction=curve==='smooth'?t*t*(3-2*t):t;
      const nextX=Math.round(deltaX*fraction),nextY=Math.round(deltaY*fraction);
      const dx=nextX-x,dy=nextY-y;
      x=nextX;y=nextY;
      if(dx===0&&dy===0)continue; // no invented mouse movement
      const start=Math.floor(i*durationMs/steps);
      const end=Math.floor((i+1)*durationMs/steps);
      append({slot,kind:'look_relative',atMs:atMs+start,
        durationMs:end-start,deltaX:dx,deltaY:dy});
    }
  }
  return actions;
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
    record(action,['slot','kind','atMs','durationMs','deltaX','deltaY']);
    const slot = validSlot(action.slot);
    if (!ownership.aiSlots.includes(slot)) {
      throw new TypeError('The human-owned Minecraft profile cannot receive AI actions');
    }
    if (!ACTIONS.has(action.kind)) throw new TypeError('Unsupported player action');
    const channel=INPUT_CHANNELS[action.kind];
    if (action.kind==='look_relative') {
      if (!Number.isInteger(action.deltaX) || !Number.isInteger(action.deltaY) ||
          Math.abs(action.deltaX)>512 || Math.abs(action.deltaY)>512 ||
          (action.deltaX===0 && action.deltaY===0)) {
        throw new TypeError('Relative camera change requires nonzero integer deltas within ±512');
      }
    } else if (Object.hasOwn(action,'deltaX') || Object.hasOwn(action,'deltaY')) {
      throw new TypeError('Only look_relative accepts camera deltas');
    }
    if (!Number.isInteger(action.atMs) || action.atMs < 0 || action.atMs > MAX_OFFSET_MS) {
      throw new TypeError('Action offset must be 0-30000 ms');
    }
    const upperDuration = action.kind==='wait' ? 5000 :
      action.kind==='look_relative' ? 250 : MAX_DURATION_MS;
    if (!Number.isInteger(action.durationMs) || action.durationMs < 50 ||
        action.durationMs > upperDuration ||
        action.atMs + action.durationMs > MAX_OFFSET_MS + 5000) {
      throw new TypeError('Action duration is unbounded or invalid');
    }
    const endMs = action.atMs + action.durationMs;
    // Walk + aim + jump + attack may overlap. Conflicting controls on ONE
    // channel (or any action during an exclusive wait) are denied.
    if (scheduled.some(previous => previous.slot===slot &&
        action.atMs<previous.endMs && previous.startMs<endMs &&
        (channel==='exclusive' || previous.channel==='exclusive' ||
          channel===previous.channel))) {
      throw new TypeError('Overlapping actions on the same client input channel are refused');
    }
    scheduled.push({
      slot,kind:action.kind,channel,startMs:action.atMs,
      durationMs:action.durationMs,endMs,
      ...(action.kind==='look_relative'?
        {deltaX:action.deltaX,deltaY:action.deltaY}:{}),
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
    inputBackend:'UNBOUND_NO_VERIFIED_INDEPENDENT_WINDOWS_GAME_INPUT',
    requiresPerClientEmergencyRelease:true,
    note:'Concurrent-channel intents are validated but NOT sent to Minecraft; independent Windows input remains unverified.',
  };
}
