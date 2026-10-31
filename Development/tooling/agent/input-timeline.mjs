// Pure compiler: AI player intentions -> simultaneous semantic input events.
// No SendInput, WM_INPUT, game DLL hooks, physical inputs or Minecraft access.
import {planPlayerActions} from './control-plan.mjs';

export const INPUT_DELIVERY_GATE='BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND';
// The sole source mapping for planning and the opt-in MC02 native writer.
export const GAME_BUTTON_BITS=Object.freeze({
  move_forward:1,move_backward:2,strafe_left:4,strafe_right:8,
  jump:16,sneak:32,attack:64,use:128,
});
const BUTTON_ORDER=Object.keys(GAME_BUTTON_BITS);
const HELD=new Set([
  'move_forward','move_backward','strafe_left','strafe_right',
  'jump','sneak','attack','use','look_left','look_right'
]);
const MAX_OFFSET=35000;
function event(action,atMs,phase){
  return {
    slot:action.slot,atMs,phase,channel:action.channel,control:action.kind,
    identityGate:'REVALIDATE_NATIVE_PID_BIRTH_SESSION_HWND',
  };
}
function compare(a,b){
  if(a.atMs!==b.atMs)return a.atMs-b.atMs;
  const rank={RELEASE:0,POINTER_DELTA:1,PRESS:2};
  return rank[a.phase]-rank[b.phase] || a.slot-b.slot ||
    a.channel.localeCompare(b.channel);
}
export function compilePlayerInputTimeline(mode,actions){
  const plan=planPlayerActions(mode,actions);
  const inputEvents=[];
  for(const action of plan.actions){
    if(action.kind==='wait')continue;
    if(action.kind==='look_relative'){
      inputEvents.push({...event(action,action.startMs,'POINTER_DELTA'),
        deltaX:action.deltaX,deltaY:action.deltaY,
        units:'UNSCALED_RELATIVE_MOUSE_UNITS'});
      continue;
    }
    if(!HELD.has(action.kind))throw new TypeError('Unknown control channel');
    inputEvents.push(event(action,action.startMs,'PRESS'));
    inputEvents.push(event(action,action.endMs,'RELEASE'));
  }
  inputEvents.sort(compare);
  const timestampGroups=[];
  for(const item of inputEvents){
    const last=timestampGroups.at(-1);
    if(last?.atMs===item.atMs)last.events.push(item);
    else timestampGroups.push({atMs:item.atMs,events:[item]});
  }
  return {
    ...plan,inputEvents,timestampGroups,eventCount:inputEvents.length,
    simultaneousDispatch:'REQUIRES_INDEPENDENT_PER_CLIENT_BACKEND',
    execution:'NOT_DISPATCHED',deliveryGate:INPUT_DELIVERY_GATE,
    calibration:'GAME_KEYBINDS_AND_MOUSE_SENSITIVITY_UNVERIFIED',
    emergencyStop:'REQUIRES_REAL_PER_CLIENT_RELEASE_ALL_CONTROLS',
    note:'Only a semantic timeline; no Windows input or Minecraft gameplay is performed.',
  };
}
/**
 * Convert the existing five-slot semantic timeline into deterministic,
 * full held-button state frames. There is NO input dispatch here.
 * Every batch represents intentions at the same timestamp, never proof
 * that independent Windows game input was delivered simultaneously.
 */
export function compilePerClientInputFrames(mode,actions) {
  const timeline=compilePlayerInputTimeline(mode,actions);
  const clients=[...new Set(timeline.actions.map(a=>a.slot))].sort((a,b)=>a-b);
  if(timeline.actions.some(a=>a.kind!=='wait' &&
    a.kind!=='look_relative' && !Object.hasOwn(GAME_BUTTON_BITS,a.kind))) {
    throw new TypeError('Per-client frames require mapped buttons or look_relative');
  }
  const held=new Map(clients.map(slot=>[slot,new Set()]));
  const batches=[];
  let frameCount=0,maximumSameTimestampSlots=0;
  for(const group of timeline.timestampGroups) {
    const pending=new Map();
    for(const event of group.events) {
      if(!pending.has(event.slot)) {
        pending.set(event.slot,{lookDX:0,lookDY:0,released:new Set(),retriggers:new Set()});
      }
      const next=pending.get(event.slot);
      if(event.phase==='POINTER_DELTA') {
        next.lookDX+=event.deltaX;
        next.lookDY+=event.deltaY;
      } else if(event.phase==='RELEASE') {
        held.get(event.slot).delete(event.control);
        next.released.add(event.control);
      } else if(event.phase==='PRESS') {
        if(next.released.has(event.control))next.retriggers.add(event.control);
        held.get(event.slot).add(event.control);
      } else throw new TypeError('Unsupported per-client input event phase');
    }
    const frames=[...pending].sort(([a],[b])=>a-b).map(([slot,next])=>{
      const buttons=BUTTON_ORDER.filter(b=>held.get(slot).has(b));
      return {
        slot,atMs:group.atMs,buttons,
        keyMask:buttons.reduce((mask,b)=>mask|GAME_BUTTON_BITS[b],0),
        lookDX:next.lookDX,lookDY:next.lookDY,
        // A same-timestamp release/repress needs multiple future native
        // writes. A full-state packet alone must NOT claim that edge.
        retriggeredButtons:BUTTON_ORDER.filter(b=>next.retriggers.has(b)),
        state:'PLANNED_FULL_CLIENT_BUTTON_STATE',
        identityGate:'NATIVE_PID_BIRTH_SESSION_HWND_CHECK_REQUIRED_AT_SEND',
        provider:mode==='human-plus-four-ai'&&slot===2?
          'MC02_OPT_IN_CANDIDATE_NOT_GAMEPLAY_VERIFIED':
          'NO_VERIFIED_PER_CLIENT_INPUT_PROVIDER',
      };
    });
    frameCount+=frames.length;
    maximumSameTimestampSlots=Math.max(maximumSameTimestampSlots,frames.length);
    batches.push({atMs:group.atMs,frames});
  }
  return {
    schema:1,mode,hostSlot:1,humanSlots:timeline.humanSlots,
    aiSlots:timeline.aiSlots,clients,
    batches,frameCount,inputEventCount:timeline.eventCount,
    maximumSameTimestampSlots,releaseAllSlots:clients,
    execution:'NOT_DISPATCHED',deliveryGate:INPUT_DELIVERY_GATE,
    noGlobalKeyboardMouse:true,noCrossClientKeyState:true,
    simultaneousFrameGrouping:'PLANNED_ONLY_NOT_NATIVE_DISPATCH',
    note:'Per-slot input frames are only intentions, not Minecraft game actions or human-input isolation proof.',
  };
}

export function compileEmergencyRelease(mode,actions,atMs){
  if(!Number.isInteger(atMs)||atMs<0||atMs>MAX_OFFSET)
    throw new TypeError('Invalid emergency release offset');
  const plan=planPlayerActions(mode,actions);
  const releaseEvents=plan.actions.filter(a=>
    HELD.has(a.kind)&&a.startMs<=atMs&&atMs<a.endMs)
    .map(a=>event(a,atMs,'RELEASE')).sort(compare);
  return {
    schema:1,mode,atMs,releaseEvents,execution:'NOT_DISPATCHED',
    deliveryGate:INPUT_DELIVERY_GATE,mustReleaseAllAtProvider:true,
    note:'An authored release list is not proof of physical key or mouse release.',
  };
}
