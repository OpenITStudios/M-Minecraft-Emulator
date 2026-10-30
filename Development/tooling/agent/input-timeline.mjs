// Pure compiler: AI player intentions -> simultaneous semantic input events.
// No SendInput, WM_INPUT, game DLL hooks, physical inputs or Minecraft access.
import {planPlayerActions} from './control-plan.mjs';

export const INPUT_DELIVERY_GATE='BLOCKED_NO_VERIFIED_PER_CLIENT_INPUT_BACKEND';
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
