// Read-only native binding for a future independent Minecraft input backend.
// This is NOT an input dispatcher, transport authorization, gameplay receipt
// or a way to drive a window via SendInput/PostMessage/MakeCode.
import {compilePlayerInputTimeline} from './input-timeline.mjs';
import {slotsForMode} from './control-plan.mjs';

const MAX_SNAPSHOT_AGE_MS=5000;
const DECIMAL_FILETIME=/^\d{1,20}$/;
const SAFE_WINDOW_VERIFICATION='SAME_SESSION_PID_BIRTH_ROOT_HWND';

export function requiredInputSlots(mode,actions) {
  const ownership=slotsForMode(mode);
  const timeline=compilePlayerInputTimeline(mode,actions);
  const aiSlots=[...new Set(timeline.actions.map(a=>a.slot))].sort((a,b)=>a-b);
  // Inspect MC01 read-only in hybrid mode to distinguish the human-owned
  // desktop from potential AI targets. Never return it as an action binding.
  const observedSlots=[...new Set([
    ...aiSlots,...ownership.humanSlots
  ])].sort((a,b)=>a-b);
  return {timeline,aiSlots,observedSlots};
}

function trustedWindow(target,slot) {
  if (!target || target.schema!==1 || target.ok!==true ||
      target.operation!=='target' || target.slot!==slot ||
      !Number.isSafeInteger(target.pid) || target.pid<1 ||
      !Number.isSafeInteger(target.windowId) || target.windowId<1 ||
      !Number.isSafeInteger(target.sessionId) || target.sessionId<0 ||
      typeof target.createdFileTime!=='string' ||
      !DECIMAL_FILETIME.test(target.createdFileTime) ||
      target.verification!==SAFE_WINDOW_VERIFICATION ||
      target.visible!==true || target.minimized!==false ||
      typeof target.foreground!=='boolean' ||
      target.gameplayInput!=='NOT_VERIFIED') {
    throw new TypeError('Native window identity is not verified for MC0'+slot);
  }
  return {
    slot,pid:target.pid,windowId:target.windowId,
    createdFileTime:target.createdFileTime,
    sessionId:target.sessionId,
    foreground:target.foreground,
    verification:'NATIVE_PROCESS_WINDOW_AT_OBSERVATION_TIME_ONLY',
  };
}

/**
 * Binds an already-validated multi-client semantic timeline to live owned
 * Windows window identities. The resulting mapping is diagnostic only and
 * cannot authorize a future game-input backend to send events.
 *
 * Neither a matching PID nor a claimed provider capability proves a game
 * receives independent background controls. Game acceptance is a separate
 * real-Windows/Minecraft requirement.
 */
export function assessInputRoute(mode,actions,observations,{
  observedAtMs=Date.now(),elapsedMs=0,
}={}) {
  const {timeline,aiSlots,observedSlots}=requiredInputSlots(mode,actions);
  if (!Array.isArray(observations) ||
      observations.length!==observedSlots.length ||
      !Number.isSafeInteger(observedAtMs) || observedAtMs<0 ||
      !Number.isSafeInteger(elapsedMs) || elapsedMs<0 ||
      elapsedMs>MAX_SNAPSHOT_AGE_MS) {
    throw new TypeError('Incomplete or stale native window observation');
  }
  const bySlot=new Map();
  for(const observation of observations) {
    const slot=observation?.slot;
    if (!observedSlots.includes(slot) || bySlot.has(slot)) {
      throw new TypeError('Duplicate, missing or unexpected native window target');
    }
    bySlot.set(slot,trustedWindow(observation,slot));
  }
  if (observedSlots.some(slot=>!bySlot.has(slot))) {
    throw new TypeError('Missing native window for requested AI/human slot');
  }
  // One client must never be silently aliased onto a second account, HWND
  // or PID, including the protected human MC01.
  const processes=new Set(),windows=new Set();
  for(const target of bySlot.values()){
    if(processes.has(target.pid) || windows.has(target.windowId)) {
      throw new TypeError('Multiple Minecraft slots share one native process/window');
    }
    processes.add(target.pid);
    windows.add(target.windowId);
  }
  const human=mode==='human-plus-four-ai'?bySlot.get(1):null;
  const windowsForAI=aiSlots.map(slot=>bySlot.get(slot));
  if (human && windowsForAI.some(target=>
      target.sessionId!==human.sessionId)) {
    // Existing controller is same-session only. Do not pretend a future
    // multi-session adapter can reuse this transport without redesign.
    throw new TypeError('Cross-session input routing is not supported');
  }
  return {
    schema:1,mode,
    execution:'NOT_DISPATCHED',
    inputBackend:'UNBOUND_NO_VERIFIED_GAME_INPUT',
    deliveryGate:'BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND',
    observedAt:new Date(observedAtMs).toISOString(),
    maximumNativeSnapshotAgeMs:MAX_SNAPSHOT_AGE_MS,
    nativeObservationElapsedMs:elapsedMs,
    humanForegroundObserved:human?.foreground??null,
    humanInputPreservation:'NOT_PROVEN_BY_NATIVE_WINDOW_INSPECTION',
    aiTargets:windowsForAI,
    humanWindowBinding:'EXCLUDED_FROM_AI_ACTIONS',
    routeType:'OBSERVATION_ONLY_NOT_GAME_INPUT',
    releaseAllBackend:'NOT_IMPLEMENTED',
    gameplayInputAcceptance:'NOT_EXECUTED',
    requiredIndependentProperties:[
      'PER_CLIENT_KEYBOARD_WITHOUT_FOREGROUND',
      'PER_CLIENT_RELATIVE_MOUSE_WITHOUT_FOREGROUND',
      'SIMULTANEOUS_DIFFERENT_GAME_CLIENTS',
      'GUARANTEED_RELEASE_ALL_ON_STOP_OR_FAILURE',
      'HUMAN_MC01_UNINTERRUPTED',
      'MINECRAFT_EDUCATION_ACTUAL_MOVEMENT_EVIDENCE'
    ],
    actionTimeline:timeline,
    note:'Verified native windows do not prove a game can receive background input. This response never authorizes input execution.',
  };
}
