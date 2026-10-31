// Experimental MC02-only semantic action sequence; NEVER a production
// Minecraft input provider. Uses the existing per-process native opt-in pulse.
// No SendInput, foreground changes, controller duplication or background daemon.
import {performance} from 'node:perf_hooks';
import {setTimeout as delay} from 'node:timers/promises';
import {compilePlayerInputTimeline} from './input-timeline.mjs';

export const MC02_SEQUENCE_LIMITS=Object.freeze({
  maxActions:12,maxHorizonMs:3500,maxPulses:64,
  heartbeatMs:160,ttlMs:350,maxLatenessMs:500,
});
const ALLOWED=new Set([
  'move_forward','move_backward','strafe_left','strafe_right',
  'jump','sneak','attack','use','look_relative',
]);
const HELD=new Set([...ALLOWED].filter(control=>control!=='look_relative'));
const sleep=(ms,signal)=>delay(ms,undefined,{signal});

/**
 * Preserve the canonical action planner and its channel collision rules.
 * Rejections here are additional limitations of the MC02 experimental
 * backend, not a restriction on the five-client preview-only planner.
 */
export function prepareMc02ActionSequence(actions) {
  if(!Array.isArray(actions)||actions.length<1||
     actions.length>MC02_SEQUENCE_LIMITS.maxActions) {
    throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
  }
  let timeline;
  try {timeline=compilePlayerInputTimeline('human-plus-four-ai',actions);}
  catch {throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');}
  if(timeline.actions.some(a=>a.slot!==2||!ALLOWED.has(a.kind)) ||
     timeline.inputEvents.length===0 ||
     timeline.inputEvents.some(e=>e.atMs>MC02_SEQUENCE_LIMITS.maxHorizonMs ||
       (e.phase==='POINTER_DELTA'&&
        (Math.abs(e.deltaX)>127||Math.abs(e.deltaY)>127)))) {
    throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
  }
  return Object.freeze({
    events:timeline.inputEvents.map(event=>Object.freeze({...event})),
    eventCount:timeline.inputEvents.length,
    finalEventMs:timeline.inputEvents.at(-1).atMs,
  });
}

/**
 * Bounded, serialized source adapter for the already opted-in native MC02
 * pulse. A successful emit() means a shared-state write was acknowledged,
 * NOT that Minecraft processed WASD, mouse movement or an attack.
 *
 * Each held-key pulse expires in <=350ms in the native controller. An extra
 * neutral packet is attempted even on abort, timeout or partial failure.
 * If the MCP process crashes, the native TTL is the fail-safe; JavaScript
 * cannot guarantee an action-level key-up after process termination.
 */
export async function runExperimentalMc02Sequence(actions,{
  emit,signal,clock=()=>performance.now(),wait=sleep,
}={}) {
  if(typeof emit!=='function')throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
  const prepared=prepareMc02ActionSequence(actions);
  if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
  const held=new Set();
  const started=clock();
  let cursor=0,heartbeatAt=Infinity,acknowledged=0,attempted=false;
  let executionError=null,neutralAcknowledged=false;
  try {
    while(cursor<prepared.events.length || held.size>0) {
      if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
      const eventAt=cursor<prepared.events.length?
        prepared.events[cursor].atMs:Infinity;
      const due=Math.min(eventAt,heartbeatAt);
      if(!Number.isFinite(due)||acknowledged>=MC02_SEQUENCE_LIMITS.maxPulses) {
        throw new Error('MC02_SEQUENCE_DEADLINE');
      }
      const remaining=due-(clock()-started);
      if(remaining>0)await wait(remaining,signal);
      if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
      const lateness=clock()-started-due;
      if(!Number.isFinite(lateness)||lateness>MC02_SEQUENCE_LIMITS.maxLatenessMs) {
        throw new Error('MC02_SEQUENCE_DEADLINE');
      }
      let dx=0,dy=0;
      while(cursor<prepared.events.length &&
            prepared.events[cursor].atMs===due) {
        const event=prepared.events[cursor++];
        if(event.phase==='PRESS') {
          if(!HELD.has(event.control))throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
          held.add(event.control);
        } else if(event.phase==='RELEASE') {
          held.delete(event.control);
        } else if(event.phase==='POINTER_DELTA') {
          dx+=event.deltaX;dy+=event.deltaY;
        } else throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
      }
      if(Math.abs(dx)>127||Math.abs(dy)>127) {
        throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
      }
      // Last-writer-wins native key mask: send the COMPLETE held state,
      // not just the current PRESS/RELEASE edge. Pointer deltas occur
      // only once at the actual semantic event timestamp.
      attempted=true;
      const accepted=await emit({
        buttons:[...held],dx,dy,ttlMs:MC02_SEQUENCE_LIMITS.ttlMs,
      });
      if(accepted!==true)throw new Error('MC02_SEQUENCE_NATIVE_UNCONFIRMED');
      acknowledged++;
      if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
      heartbeatAt=held.size>0?
        due+MC02_SEQUENCE_LIMITS.heartbeatMs:Infinity;
    }
  } catch(error) {
    executionError=error;
  } finally {
    if(attempted) {
      try {
        // Independent of cancellation; a fresh request timestamp is
        // generated by the MCP bridge immediately before native send.
        neutralAcknowledged=await emit({
          buttons:[],dx:0,dy:0,ttlMs:1,
        })===true;
      } catch {neutralAcknowledged=false;}
    }
  }
  if(attempted&&!neutralAcknowledged) {
    throw new Error('MC02_RELEASE_UNCONFIRMED');
  }
  if(executionError)throw executionError;
  return {
    schema:1,slot:2,mode:'human-plus-four-ai',
    status:'EXPERIMENTAL_MC02_SEQUENCE_TRANSPORT_ACKNOWLEDGED',
    pulsesAcknowledged:acknowledged,
    neutralState:'NATIVE_NEUTRAL_WRITE_ACKNOWLEDGED_ONLY',
    inputEffect:'NOT_VERIFIED_IN_MINECRAFT_EDUCATION',
    gameplayObserved:false,humanInputUninterrupted:'NOT_PROVEN',
    note:'Native shared-state acknowledgements and TTL do not establish Minecraft gameplay or independent human-input safety.',
  };
}
