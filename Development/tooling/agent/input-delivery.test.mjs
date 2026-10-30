// Authored-only callback fixtures. DO NOT EXECUTE until operator approval.
// These are NOT Minecraft game simulations or working Windows input.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createInputDeliveryHarness} from './input-delivery.mjs';
import {assessInputRoute} from './input-route.mjs';
const rawTarget=(slot)=>({
  schema:1,ok:true,operation:'target',slot,
  pid:1000+slot,windowId:3000+slot,
  createdFileTime:'134000000000000001',sessionId:1,
  foreground:slot===1,visible:true,minimized:false,
  verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED',
});
const action=(slot,kind,atMs,durationMs,other={})=>({
  slot,kind,atMs,durationMs,...other,
});
function route(actions,mode='human-plus-four-ai') {
  const slots=[...new Set([...(mode==='human-plus-four-ai'?[1]:[]),
    ...actions.map(a=>a.slot)])].sort((a,b)=>a-b);
  return assessInputRoute(mode,actions,slots.map(rawTarget),{
    observedAtMs:100000,elapsedMs:10,
  });
}
function port({human=true,identityChanged=false,releaseFails=false,
  cancelOnFirstSend=null}={}) {
  const sent=[],released=[];
  let current=0;
  return {
    sent,released,
    admission:async()=>true,
    human:async()=>human,
    verify:async slot=>identityChanged?
      {...rawTarget(slot),windowId:99999}:rawTarget(slot),
    backend:{
      send:async (target,event)=>{
        sent.push({slot:target.slot,phase:event.phase,control:event.control});
        if(++current===1)cancelOnFirstSend?.();
        return {accepted:true};
      },
      releaseAll:async target=>{
        released.push(target.slot);
        return {released:!releaseFails};
      },
    },
  };
}
test('no native backend means NEVER dispatch anything',async()=>{
  const runner=createInputDeliveryHarness();
  assert.equal(runner.status().provider,'NONE');
  assert.equal(runner.status().execution,'DISABLED');
  await assert.rejects(()=>runner.run(null,[]),/BLOCKED_NO_VERIFIED/);
});
test('independent slots and simultaneous camera/walk are ordered by channel',async()=>{
  const actions=[
    action(2,'move_forward',0,200),
    action(2,'look_relative',0,50,{deltaX:10,deltaY:-2}),
    action(3,'attack',0,100),
  ];
  const bound=route(actions);
  const p=port();
  let tick=0;
  const runner=createInputDeliveryHarness({
    backend:p.backend,admit:p.admission,verify:p.verify,protectHuman:p.human,
    wallClock:()=>100000,clock:()=>tick,
    wait:async ms=>{tick+=ms;},
  });
  const result=await runner.run(bound,actions);
  assert.equal(result.minecraftGameplay,'NOT_OBSERVED');
  assert.equal(p.sent.length,5);
  assert.equal(p.sent.some(e=>e.phase==='POINTER_DELTA'),true);
  assert.deepEqual([...p.released].sort(),[2,3]);
  assert.equal(runner.status().activeTransaction,false);
});
test('target changed: reject before callbacks; release-all remains mandatory',async()=>{
  const actions=[action(2,'move_forward',0,100)];
  const p=port({identityChanged:true});
  const runner=createInputDeliveryHarness({
    backend:p.backend,admit:p.admission,verify:p.verify,protectHuman:p.human,
    wallClock:()=>100000,clock:()=>0,
  });
  await assert.rejects(()=>runner.run(route(actions),actions),/IDENTITY_CHANGED/);
  assert.equal(p.sent.length,0);
  assert.deepEqual(p.released,[2]);
});
test('human focus protection refuses delivery to all AI clients',async()=>{
  const actions=[action(2,'attack',0,100)];
  const p=port({human:false});
  const runner=createInputDeliveryHarness({
    backend:p.backend,admit:p.admission,verify:p.verify,protectHuman:p.human,
    wallClock:()=>100000,clock:()=>0,
  });
  await assert.rejects(()=>runner.run(route(actions),actions),
    /HUMAN_DESKTOP_NOT_CONFIRMED/);
  assert.equal(p.sent.length,0);
  assert.deepEqual(p.released,[2]);
});
test('abort and failed native release are never reported as successful gameplay',async()=>{
  const actions=[action(2,'move_forward',0,200)];
  const controller=new AbortController();
  const p=port({releaseFails:true,cancelOnFirstSend:()=>controller.abort()});
  const runner=createInputDeliveryHarness({
    backend:p.backend,admit:p.admission,verify:p.verify,protectHuman:p.human,
    wallClock:()=>100000,clock:()=>0,
    wait:async()=>{},
  });
  await assert.rejects(()=>runner.run(route(actions),actions,
    {signal:controller.signal}),/RELEASE_ALL_NOT_CONFIRMED/);
  assert.equal(p.released.length,1);
});
test('stale route cannot authorize callbacks',async()=>{
  const actions=[action(2,'jump',0,100)];
  const p=port();
  const runner=createInputDeliveryHarness({
    backend:p.backend,admit:p.admission,verify:p.verify,protectHuman:p.human,
    wallClock:()=>105000,clock:()=>0,
  });
  await assert.rejects(()=>runner.run(route(actions),actions),/recent native/);
  assert.equal(p.sent.length,0);
});
