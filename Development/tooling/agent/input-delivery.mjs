// The semantic event runner for a future VERIFIED native per-client game input
// provider. Not connected to MCP actions. There is NO production backend.
// It never imports SendInput, PostMessage, MakeCode, Raw Input, game DLL code,
// or any other keyboard/mouse injection implementation.
//
// The tests inject callbacks into this source boundary; tests DO NOT prove
// Windows input, Minecraft camera control, account identity, or gameplay.
import {performance} from 'node:perf_hooks';
import {setTimeout as delay} from 'node:timers/promises';
import {compilePlayerInputTimeline,INPUT_DELIVERY_GATE} from './input-timeline.mjs';
import {matchesNativeInputTarget} from './input-route.mjs';

const MAX_ROUTE_AGE_MS=1500;
const MAX_LATENESS_MS=500;
const MAX_EVENTS=64;
const MAX_EVENT_HORIZON_MS=35000;

export function inputExecutionStatus() {
  return {
    schema:1,
    provider:'NONE',
    execution:'DISABLED',
    gate:INPUT_DELIVERY_GATE,
    capabilities:{
      perClientKeyboard:'NOT_VERIFIED',
      perClientRelativeMouse:'NOT_VERIFIED',
      parallelInput:'NOT_VERIFIED',
      releaseAll:'NOT_IMPLEMENTED_IN_NATIVE_PROVIDER',
      minecraftGameResponse:'NOT_EXECUTED',
    },
    note:'Source event scheduling is not an independent Windows game-input provider.',
  };
}
const sleep=(ms,signal)=>delay(ms,undefined,{signal});

function validateRoute(route,actions,wallNow) {
  if(!route || route.schema!==1 ||
     route.execution!=='NOT_DISPATCHED' ||
     route.routeType!=='OBSERVATION_ONLY_NOT_GAME_INPUT' ||
     route.deliveryGate!==INPUT_DELIVERY_GATE ||
     !['human-plus-four-ai','five-ai'].includes(route.mode) ||
     typeof route.observedAt!=='string' ||
     !Number.isFinite(Date.parse(route.observedAt)) ||
     wallNow-Date.parse(route.observedAt)<0 ||
     wallNow-Date.parse(route.observedAt)>MAX_ROUTE_AGE_MS) {
    throw new Error('No recent native input route; never infer authorization');
  }
  const compiled=compilePlayerInputTimeline(route.mode,actions);
  if(compiled.eventCount>MAX_EVENTS ||
     compiled.timestampGroups.some(group=>group.atMs>MAX_EVENT_HORIZON_MS) ||
     JSON.stringify(compiled.inputEvents)!==
       JSON.stringify(route.actionTimeline?.inputEvents)) {
    throw new Error('Input event plan changed since native identity assessment');
  }
  const slots=[...new Set(compiled.actions.map(a=>a.slot))].sort((a,b)=>a-b);
  if(!Array.isArray(route.aiTargets) || route.aiTargets.length!==slots.length ||
     route.aiTargets.some((t,i)=>t?.slot!==slots[i] ||
       t.verification!=='NATIVE_PROCESS_WINDOW_AT_OBSERVATION_TIME_ONLY' ||
       !Number.isSafeInteger(t.pid) || t.pid<=0 ||
       !Number.isSafeInteger(t.windowId) || t.windowId<=0 ||
       !Number.isSafeInteger(t.sessionId) || t.sessionId<0 ||
       typeof t.createdFileTime!=='string' ||
       !/^\d{1,20}$/.test(t.createdFileTime))) {
    throw new Error('AI window targets cannot be used for input execution');
  }
  if(route.mode==='human-plus-four-ai' && slots.includes(1)) {
    throw new Error('MC01 is human-owned');
  }
  return {compiled,slots,targets:new Map(route.aiTargets.map(t=>[t.slot,t]))};
}

/**
 * This is a *port*, not a production backend. Its callbacks must eventually
 * be implemented by an independently verified, native, non-foreground game
 * input provider. This factory is not imported by the MCP execution path:
 * agents CANNOT register a backend, admit their own calls or enable input.
 *
 * Required future native capabilities:
 *  - atomic per-client input routing, no system-wide keyboard/mouse
 *  - independent client identity verification before every send/release
 *  - driver-level timeouts and release-all even if Node is killed
 *  - separate observed in-game movement/camera evidence
 */
export function createInputDeliveryHarness({
  backend=null,
  admit=null,
  verify=null,
  protectHuman=null,
  wallClock=()=>Date.now(),
  clock=()=>performance.now(),
  wait=sleep,
}={}) {
  let active=null;
  const status=()=>({...inputExecutionStatus(),
    activeTransaction:active!==null,
    testPortWired:backend!==null,
    note:backend===null?
      'No production game-input backend exists; all dispatch is blocked.':
      'Internal callback port only; no Minecraft compatibility or runtime authority inferred.',
  });
  async function run(route,actions,{signal}={}) {
    if(backend===null || typeof admit!=='function' ||
       typeof verify!=='function' ||
       typeof backend.send!=='function' ||
       typeof backend.releaseAll!=='function' ||
       typeof protectHuman!=='function') {
      throw new Error(INPUT_DELIVERY_GATE);
    }
    if(active!==null) throw new Error('A bounded input transaction is already active');
    const {compiled,slots,targets}=validateRoute(route,actions,wallClock());
    const controller=new AbortController();
    const abort=()=>controller.abort();
    if(signal?.aborted) throw new Error('Input transaction cancelled before start');
    signal?.addEventListener('abort',abort,{once:true});
    const transaction={controller,slots,targets,acknowledged:0};
    active=transaction;
    let deliveryError=null;
    let releaseResults=[];
    const t0=clock();
    try {
      for(const group of compiled.timestampGroups) {
        if(controller.signal.aborted) throw new Error('INPUT_TRANSACTION_ABORTED');
        const remaining=group.atMs-(clock()-t0);
        if(remaining>0) await wait(remaining,controller.signal);
        if(controller.signal.aborted) throw new Error('INPUT_TRANSACTION_ABORTED');
        const late=clock()-t0-group.atMs;
        if(late>MAX_LATENESS_MS) {
          throw new Error('INPUT_EVENT_DEADLINE_EXCEEDED');
        }
        if(route.mode==='human-plus-four-ai' &&
           await protectHuman({signal:controller.signal})!==true) {
          throw new Error('PROTECTED_HUMAN_DESKTOP_NOT_CONFIRMED');
        }
        // Each underlying target is rechecked before the timestamp group,
        // including the human-protection check for the default hybrid mode.
        const grouped=new Map();
        for(const event of group.events) {
          if(!grouped.has(event.slot)) grouped.set(event.slot,[]);
          grouped.get(event.slot).push(event);
        }
        for(const slot of grouped.keys()) {
          const expected=targets.get(slot);
          const observed=await verify(slot,{signal:controller.signal});
          if(!matchesNativeInputTarget(expected,observed)) {
            throw new Error('INPUT_TARGET_IDENTITY_CHANGED');
          }
          // This additional callback is a native policy gate, NOT a textual
          // "supported=true" claim from MCP/AI. No such native callback
          // exists in the installed product.
          if(await admit(expected,{signal:controller.signal})!==true) {
            throw new Error(INPUT_DELIVERY_GATE);
          }
        }
        // Concurrent across separate clients; ordered within each client
        // to guarantee RELEASE precedes its replacement PRESS.
        const delivered=await Promise.allSettled(
          [...grouped].map(async ([slot,events])=>{
            const target=targets.get(slot);
            let accepted=0;
            for(const event of events) {
              if(controller.signal.aborted) throw new Error('INPUT_TRANSACTION_ABORTED');
              const receipt=await backend.send(target,event,{signal:controller.signal});
              if(receipt?.accepted!==true) {
                throw new Error('NATIVE_INPUT_TRANSPORT_DID_NOT_ACKNOWLEDGE');
              }
              accepted++;
            }
            return accepted;
          })
        );
        transaction.acknowledged+=delivered.filter(r=>r.status==='fulfilled')
          .reduce((n,r)=>n+r.value,0);
        if(delivered.some(r=>r.status==='rejected')) {
          throw new Error('INPUT_EVENT_GROUP_PARTIAL_OR_UNCONFIRMED');
        }
      }
    } catch(error) {
      deliveryError=error;
    } finally {
      controller.abort();
      // Always ask every backend slot to release ALL controls, even after
      // a partially accepted group. Native watchdog remains mandatory;
      // JavaScript callbacks cannot guarantee physical key-up on crash.
      releaseResults=await Promise.allSettled(
        slots.map(slot=>backend.releaseAll(targets.get(slot)))
      );
      active=null;
      signal?.removeEventListener('abort',abort);
    }
    if(releaseResults.some(r=>r.status==='rejected' ||
        r.value?.released!==true)) {
      throw new Error('RELEASE_ALL_NOT_CONFIRMED_BY_TRANSPORT');
    }
    if(deliveryError) throw deliveryError;
    return {
      schema:1,mode:route.mode,
      transportEventsAcknowledged:transaction.acknowledged,
      inputTransport:'CALLBACKS_ACKNOWLEDGED_NOT_GAMEPLAY_VERIFIED',
      releaseAll:'TRANSPORT_CALLBACKS_ACKNOWLEDGED_ONLY',
      minecraftGameplay:'NOT_OBSERVED',
      humanInputUninterrupted:'NOT_PROVEN_BY_CALLBACK_RECEIPTS',
    };
  }
  function stop() {
    if(active===null) return false;
    active.controller.abort();
    // Provider release-all occurs in run() finally. An actual native
    // backend MUST also enforce an independent crash/deadline watchdog.
    return true;
  }
  return {status,run,stop};
}
