// Experimental MC02-only semantic action sequence; NEVER a production
// Minecraft input provider. Uses the existing per-process native opt-in pulse.
// No SendInput, foreground changes, controller duplication or background daemon.
import {performance} from 'node:perf_hooks';
import {setTimeout as delay} from 'node:timers/promises';
import {compilePerClientInputFrames} from './input-timeline.mjs';

export const MC02_SEQUENCE_LIMITS=Object.freeze({
  maxActions:12,maxHorizonMs:3500,maxPulses:64,
  heartbeatMs:160,ttlMs:350,maxLatenessMs:150,
});
const ALLOWED=new Set([
  'move_forward','move_backward','strafe_left','strafe_right',
  'jump','sneak','attack','use','look_relative',
]);
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
  try {timeline=compilePerClientInputFrames('human-plus-four-ai',actions);}
  catch {throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');}
  const frames=timeline.batches.flatMap(batch=>batch.frames);
  if(actions.some(a=>a.slot!==2||!ALLOWED.has(a.kind)) ||
     frames.length===0 ||
     frames.some(frame=>frame.slot!==2 ||
       frame.atMs>MC02_SEQUENCE_LIMITS.maxHorizonMs ||
       Math.abs(frame.lookDX)>127||Math.abs(frame.lookDY)>127 ||
       frame.retriggeredButtons.length>0)) {
    throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
  }
  return Object.freeze({
    frames:frames.map(frame=>Object.freeze({
      ...frame,buttons:Object.freeze([...frame.buttons]),
    })),
    eventCount:timeline.inputEventCount,
    finalEventMs:frames.at(-1).atMs,
  });
}

/**
 * One bounded deterministic schedule for BOTH the native single-process
 * batch transport and the source-only injected per-pulse fixture runner.
 * The native controller still validates every pulse independently.
 */
export function prepareMc02PulseBatch(actions) {
  const prepared=prepareMc02ActionSequence(actions);
  const pulses=[];
  let cursor=0,buttons=[],keyMask=0,heartbeatAt=Infinity;
  while(cursor<prepared.frames.length || buttons.length>0) {
    const eventAt=cursor<prepared.frames.length?
      prepared.frames[cursor].atMs:Infinity;
    const due=Math.min(eventAt,heartbeatAt);
    if(!Number.isFinite(due)||due>MC02_SEQUENCE_LIMITS.maxHorizonMs||
       pulses.length>=MC02_SEQUENCE_LIMITS.maxPulses) {
      throw new Error('MC02_SEQUENCE_DEADLINE');
    }
    let dx=0,dy=0;
    if(cursor<prepared.frames.length&&
       prepared.frames[cursor].atMs===due) {
      const frame=prepared.frames[cursor++];
      buttons=frame.buttons;keyMask=frame.keyMask;
      dx=frame.lookDX;dy=frame.lookDY;
    }
    pulses.push(Object.freeze({
      atMs:due,keys:keyMask,buttons:Object.freeze([...buttons]),
      dx,dy,ttlMs:MC02_SEQUENCE_LIMITS.ttlMs,
    }));
    heartbeatAt=buttons.length>0?
      due+MC02_SEQUENCE_LIMITS.heartbeatMs:Infinity;
  }
  return Object.freeze({
    pulses:Object.freeze(pulses),
    count:pulses.length,
    finalEventMs:prepared.finalEventMs,
  });
}

/**
 * Bounded, serialized source adapter for the already opted-in native MC02
 * pulse. A successful emit() means a shared-state write was acknowledged,
 * NOT that Minecraft processed WASD, mouse movement or an attack.
 *
 * Each held-key pulse expires in <=350ms in the native controller.
 * Heartbeats are scheduled at 160ms with at most 150ms allowed lag, so
 * late native CLI operations fail closed instead of claiming uninterrupted
 * input after the previous pulse may already have expired.
 * An extra neutral packet is attempted even on abort, timeout or partial failure.
 * If the MCP process crashes, the native TTL is the fail-safe; JavaScript
 * cannot guarantee an action-level key-up after process termination.
 */
export async function runExperimentalMc02Sequence(actions,{
  emit,signal,clock=()=>performance.now(),wait=sleep,
}={}) {
  if(typeof emit!=='function')throw new Error('MC02_SEQUENCE_INVALID_ACTIONS');
  const batch=prepareMc02PulseBatch(actions);
  if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
  const started=clock();
  let acknowledged=0,attempted=false;
  let executionError=null,neutralAcknowledged=false;
  try {
    for(const pulse of batch.pulses) {
      if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
      const remaining=pulse.atMs-(clock()-started);
      if(remaining>0)await wait(remaining,signal);
      if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
      const lateness=clock()-started-pulse.atMs;
      if(!Number.isFinite(lateness)||
         lateness>MC02_SEQUENCE_LIMITS.maxLatenessMs) {
        throw new Error('MC02_SEQUENCE_DEADLINE');
      }
      attempted=true;
      const accepted=await emit({
        buttons:[...pulse.buttons],dx:pulse.dx,dy:pulse.dy,
        ttlMs:pulse.ttlMs,
      });
      if(accepted!==true)throw new Error('MC02_SEQUENCE_NATIVE_UNCONFIRMED');
      acknowledged++;
      if(signal?.aborted)throw new Error('MC02_SEQUENCE_ABORTED');
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
