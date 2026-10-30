// Pure gameplay-intent -> unassigned virtual Xbox pad REPORT candidate.
// No OS input. NO Minecraft PID/HWND binding or gamepad device binding.
// Never enable during human-plus-four-ai gameplay or call from production MCP.
import {planPlayerActions} from './control-plan.mjs';

const AXIS=32767,DIAGONAL=23170,CAMERA_AXIS=19000;
const MAX_LAB_MS=10000,HEARTBEAT_MS=200,MAX_REPORTS=150;
const INPUT_DELIVERY_GATE='BLOCKED_UNASSIGNED_GLOBAL_VIRTUAL_GAMEPAD';
const held=(a,t)=>a.startMs<=t && t<a.endMs;
const clamp=(n)=>Math.max(-AXIS,Math.min(AXIS,n));

/**
 * Converts ONE AI player's short action plan into X360 reports.
 * The right stick rotates the camera at an ANALOG RATE, which is not a
 * mouse pixel delta; the conversion must be calibrated in Minecraft.
 *
 * This does not connect the report to any game; Windows sends virtual
 * controller reports system-wide unless a separate proven routing
 * technology isolates them per process.
 */
export function compileGlobalGamepadCandidate(mode,actions) {
  if (mode!=='five-ai') {
    throw new Error('Global gamepad lab is refused while a human owns MC01');
  }
  const plan=planPlayerActions(mode,actions);
  const slots=[...new Set(plan.actions.map(a=>a.slot))];
  if(slots.length!==1) {
    throw new Error('One global virtual pad cannot target multiple Minecraft clients');
  }
  const slot=slots[0];
  const finishAt=Math.max(...plan.actions.map(a=>a.endMs));
  if(finishAt>MAX_LAB_MS) {
    throw new Error('Global virtual-pad experiment is capped at ten seconds');
  }
  const markers=new Set([0,finishAt]);
  for(const a of plan.actions) {
    markers.add(a.startMs);
    markers.add(a.endMs);
  }
  for(let time=HEARTBEAT_MS;time<finishAt;time+=HEARTBEAT_MS) {
    markers.add(time); // keep the native 350 ms fail-neutral watchdog fed
  }
  const times=[...markers].sort((a,b)=>a-b);
  if(times.length>MAX_REPORTS) {
    throw new Error('Too many bounded gamepad reports');
  }
  const frames=times.map((atMs,index)=>{
    const active=plan.actions.filter(a=>held(a,atMs));
    const has=kind=>active.some(a=>a.kind===kind);
    let lx=(has('strafe_right')?AXIS:0)-(has('strafe_left')?AXIS:0);
    let ly=(has('move_forward')?AXIS:0)-(has('move_backward')?AXIS:0);
    if(lx!==0 && ly!==0) {
      lx=Math.sign(lx)*DIAGONAL;
      ly=Math.sign(ly)*DIAGONAL;
    }
    let rx=(has('look_right')?CAMERA_AXIS:0)-
      (has('look_left')?CAMERA_AXIS:0);
    let ry=0;
    const relative=active.find(a=>a.kind==='look_relative');
    if(relative) {
      // Provisional analog RATE, not a mouse delta. Gain and Y polarity
      // require in-game calibration, so game-control quality is UNKNOWN.
      rx=clamp(relative.deltaX*128);
      ry=clamp(-relative.deltaY*128);
    }
    const buttons=[];
    if(has('jump')) buttons.push('a');
    if(has('sneak')) buttons.push('b');
    return {
      seq:index+1,atMs,lx,ly,rx,ry,
      lt:has('use')?255:0,rt:has('attack')?255:0,
      buttons,
    };
  });
  const last=frames.at(-1);
  if(last.lx!==0 || last.ly!==0 || last.rx!==0 || last.ry!==0 ||
     last.lt!==0 || last.rt!==0 || last.buttons.length!==0) {
    throw new Error('Source experiment did not end with a neutral report');
  }
  return {
    schema:1,mode,labSlotLabel:slot,
    gamepadRouting:'GLOBAL_DEVICE_NOT_BOUND_TO_MINECRAFT_PROCESS',
    executor:'SEPARATE_NATIVE_GAMEPAD_LAB_ONLY',
    execution:'NOT_DISPATCHED',
    deliveryGate:INPUT_DELIVERY_GATE,
    gamepadSignal:'SOURCE_ONLY_NOT_SENT',
    mouseCamera:'UNVERIFIED_XINPUT_RIGHT_STICK_ANALOG_RATE',
    minecraftEducationSupport:'NOT_TESTED',
    humanSafety:'GLOBAL_CONTROLLER_MAY_AFFECT_OTHER_GAMES',
    maxDurationMs:MAX_LAB_MS,
    frames,
    note:'A bounded report timeline is not a per-client game input backend or proof of player movement.',
  };
}
