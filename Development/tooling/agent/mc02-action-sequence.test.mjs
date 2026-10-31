// AUTHORED ONLY: source fixtures; no Minecraft, native CLI or local run.
import test from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import {prepareMc02ActionSequence,runExperimentalMc02Sequence,
  MC02_SEQUENCE_LIMITS} from './mc02-action-sequence.mjs';
import {createHandler,parseConfiguration} from './mcp.mjs';
const move=(kind,atMs,durationMs,other={})=>({slot:2,kind,atMs,durationMs,...other});
function virtualClock(){
  let time=0;
  return {clock:()=>time,wait:async(ms,signal)=>{
    if(signal?.aborted)throw Error('ABORTED');
    time+=ms;
  }};
}
const optIn=()=>parseConfiguration([
  '--exe',path.resolve('lazy-tester.exe'),'--mc02-input-experimental','on',
],{LAZY_TESTER_MC02_VIRTUAL_INPUT:'I_ACCEPT_EXPERIMENTAL_MC02_INPUT'});
const receipt={schema:1,slot:2,status:'NATIVE_MC02_STATE_WRITTEN_GAMEPLAY_UNVERIFIED',gameplayObserved:false};

test('one bounded sequence combines movement and look; heartbeat and final neutral',async()=>{
  const pulses=[],timer=virtualClock();
  const actions=[move('move_forward',0,500),
    move('look_relative',0,50,{deltaX:20,deltaY:-10}),
    move('attack',100,150)];
  const prepared=prepareMc02ActionSequence(actions);
  assert.ok(prepared.eventCount>0);
  const result=await runExperimentalMc02Sequence(actions,{
    ...timer,emit:async p=>{pulses.push(p);return true;},
  });
  assert.equal(result.gameplayObserved,false);
  assert.equal(result.pulsesAcknowledged,pulses.length-1);
  assert.deepEqual(pulses[0].buttons,['move_forward']);
  assert.deepEqual([pulses[0].dx,pulses[0].dy],[20,-10]);
  assert.ok(pulses.some(p=>p.buttons.includes('attack')));
  assert.ok(pulses.filter(p=>p.buttons.includes('move_forward')).length>=3);
  assert.deepEqual(pulses.at(-1),{buttons:[],dx:0,dy:0,ttlMs:1});
  assert.equal(pulses.slice(1).every(p=>p.dx===0&&p.dy===0),true);
});
test('reject all other slots, unimplemented camera directions and oversized deltas',()=>{
  for(const actions of [
    [{slot:1,kind:'attack',atMs:0,durationMs:100}],
    [move('look_left',0,50)],
    [move('look_relative',0,50,{deltaX:128,deltaY:0})],
    [move('attack',0,100,{command:'give'})],
    [move('jump',3500,100)],
    Array.from({length:MC02_SEQUENCE_LIMITS.maxActions+1},(_,i)=>
      move('attack',i*100,50)),
  ])assert.throws(()=>prepareMc02ActionSequence(actions),/MC02_SEQUENCE_INVALID_ACTIONS/);
});
test('overdue native pulse fails closed before replaying stale controls',async()=>{
  const packets=[];
  let now=0;
  await assert.rejects(()=>runExperimentalMc02Sequence(
    [move('move_forward',0,500)],{
      clock:()=>now,
      wait:async(ms)=>{now+=ms+MC02_SEQUENCE_LIMITS.maxLatenessMs+1;},
      emit:async p=>{packets.push(p);return true;},
    }),/MC02_SEQUENCE_DEADLINE/);
  assert.equal(packets.length,2); // initial press, then neutral
  assert.deepEqual(packets.at(-1),{buttons:[],dx:0,dy:0,ttlMs:1});
});
test('native send failure still attempts neutral; never returns gameplay success',async()=>{
  const packets=[],timer=virtualClock();
  await assert.rejects(()=>runExperimentalMc02Sequence([move('attack',0,100)],{
    ...timer,emit:async p=>{packets.push(p);if(p.buttons.length)throw Error('NATIVE_REFUSED');return true;},
  }),/NATIVE_REFUSED/);
  assert.deepEqual(packets.at(-1),{buttons:[],dx:0,dy:0,ttlMs:1});
});
test('failed neutral is a hard failure, even after acknowledged action',async()=>{
  const timer=virtualClock();
  await assert.rejects(()=>runExperimentalMc02Sequence([move('jump',0,100)],{
    ...timer,emit:async p=>p.ttlMs!==1,
  }),/MC02_RELEASE_UNCONFIRMED/);
});
test('abort cancels further pulses, but still attempts independent neutral',async()=>{
  const controller=new AbortController(),timer=virtualClock(),packets=[];
  await assert.rejects(()=>runExperimentalMc02Sequence([move('move_forward',0,500)],{
    ...timer,signal:controller.signal,emit:async p=>{
      packets.push(p);if(p.buttons.length)controller.abort();return true;},
  }),/MC02_SEQUENCE_ABORTED/);
  assert.equal(packets.length,2);
  assert.deepEqual(packets.at(-1),{buttons:[],dx:0,dy:0,ttlMs:1});
});
test('experimental sequence tools are invisible in the normal MCP mode',async()=>{
  const normal=createHandler({mode:'human-plus-four-ai',executable:'x'});
  const list=(await normal.handle({jsonrpc:'2.0',id:1,method:'tools/list'})).result.tools;
  assert.equal(list.some(t=>t.name==='run_experimental_mc02_actions'),false);
  await assert.rejects(()=>normal.callTool('run_experimental_mc02_actions',{
    slot:2,actions:[move('attack',0,100)],
  }),/MC02_EXPERIMENT_DISABLED/);
});
test('MCP experimental sequence reuses the retained native MC02 input callback',async()=>{
  const packets=[],cfg=optIn();
  const handler=createHandler(cfg,{mc02Input:async(_exe,packet)=>{
    packets.push(packet);return receipt;
  }});
  const list=(await handler.handle({jsonrpc:'2.0',id:1,method:'tools/list'})).result.tools;
  assert.equal(list.some(t=>t.name==='run_experimental_mc02_actions'),true);
  assert.equal(list.some(t=>t.name==='stop_experimental_mc02_actions'),true);
  const output=await handler.callTool('run_experimental_mc02_actions',{
    slot:2,actions:[move('move_forward',0,100)],
  });
  assert.equal(output.gameplayObserved,false);
  assert.equal(output.pulsesAcknowledged,2);
  assert.equal(packets[0].keys,1);
  assert.equal(packets.at(-1).keys,0);
  assert.equal(packets.at(-1).ttlMs,1);
  assert.equal((await handler.callTool('stop_experimental_mc02_actions',{slot:2}))
    .status,'NO_ACTIVE_SEQUENCE');
  handler.shutdown();
});
