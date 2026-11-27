// Authored contracts; do not run until the operator explicitly permits tests.
import test from 'node:test';
import assert from 'node:assert/strict';
import {slotsForMode,planPeerSession,planPlayerActions,expandActionGroups} from './control-plan.mjs';

const profile = slot => ({
  slot, state: slot === 4 ? 'STOPPED' : 'RUNNING',
  hasWindow: slot !== 4, responding: slot !== 4, needsAttention: false,
});
const snapshot = () => ({
  schema: 1, ok: true, operation: 'inspect',
  profiles: [1,2,3,4,5].map(profile),
});

test('peer plan keeps the human host and five slots without inventing joins',() => {
  const plan=planPeerSession('human-plus-four-ai',snapshot());
  assert.deepEqual(plan.humanSlots,[1]);
  assert.deepEqual(plan.aiSlots,[2,3,4,5]);
  assert.equal(plan.hostWorld,'NOT_OBSERVED');
  assert.equal(plan.profiles[0].next,'OBSERVE_WORLD_HOST_IN_GAME');
  assert.equal(plan.profiles[3].next,'REQUEST_LAUNCH_IF_NATIVE_ADMITS');
  assert.equal(plan.execution,'NOT_PERFORMED');
});

test('AI-only mode permits MC01 intent but not a fake gameplay receipt',() => {
  const plan=planPeerSession('five-ai',snapshot());
  assert.deepEqual(plan.humanSlots,[]);
  assert.deepEqual(plan.aiSlots,[1,2,3,4,5]);
  assert.equal(plan.independentBackgroundInput,'NOT_VERIFIED');
  assert.equal(slotsForMode('five-ai').hostSlot,1);
});

test('concurrent actions for different AI profiles are prepared, not dispatched',() => {
  const plan=planPlayerActions('human-plus-four-ai',[
    {slot:2,kind:'move_forward',atMs:0,durationMs:400},
    {slot:3,kind:'strafe_left',atMs:0,durationMs:400},
    {slot:4,kind:'jump',atMs:100,durationMs:100},
  ]);
  assert.equal(plan.maximumConcurrentIntents,3);
  assert.equal(plan.execution,'NOT_DISPATCHED');
  assert.match(plan.gate,/BLOCKED/);
});

test('MC01 protection and conflicting input channels still refuse overlap',()=>{
  assert.throws(()=>planPlayerActions('human-plus-four-ai',[
    {slot:1,kind:'move_forward',atMs:0,durationMs:80},
  ]),/human-owned/);
  assert.throws(()=>planPlayerActions('five-ai',[
    {slot:2,kind:'move_forward',atMs:0,durationMs:300},
    {slot:2,kind:'move_backward',atMs:200,durationMs:100},
  ]),/input channel/);
  assert.throws(()=>planPlayerActions('five-ai',[
    {slot:2,kind:'wait',atMs:0,durationMs:400},
    {slot:2,kind:'attack',atMs:200,durationMs:100},
  ]),/input channel/);
});
test('one AI may walk strafe jump attack and aim at the same time',()=>{
  const p=planPlayerActions('human-plus-four-ai',[
    {slot:2,kind:'move_forward',atMs:0,durationMs:400},
    {slot:2,kind:'strafe_left',atMs:0,durationMs:400},
    {slot:2,kind:'jump',atMs:0,durationMs:100},
    {slot:2,kind:'attack',atMs:0,durationMs:100},
    {slot:2,kind:'look_relative',atMs:0,durationMs:50,deltaX:90,deltaY:-20},
  ]);
  assert.equal(p.maximumConcurrentIntents,5);
  assert.equal(new Set(p.actions.map(x=>x.channel)).size,5);
  assert.equal(p.execution,'NOT_DISPATCHED');
  assert.match(p.inputBackend,/UNBOUND/);
});

test('unknown, oversized and malformed actions are refused',() => {
  assert.throws(()=>planPlayerActions('five-ai',[]));
  assert.throws(()=>planPlayerActions('five-ai',Array(33).fill({
    slot:2,kind:'wait',atMs:0,durationMs:100,
  })));
  for(const action of [
    {slot:2,kind:'execute_shell',atMs:0,durationMs:100},
    {slot:6,kind:'jump',atMs:0,durationMs:100},
    {slot:2,kind:'jump',atMs:0,durationMs:100,command:'/op @a'},
    {slot:2,kind:'move_forward',atMs:-1,durationMs:100},
    {slot:2,kind:'attack',atMs:0,durationMs:99999},
    {slot:2,kind:'look_relative',atMs:0,durationMs:50,deltaX:0,deltaY:0},
    {slot:2,kind:'look_relative',atMs:0,durationMs:50,deltaX:513,deltaY:1},
    {slot:2,kind:'look_relative',atMs:0,durationMs:50},
    {slot:2,kind:'move_forward',atMs:0,durationMs:80,deltaX:1},
  ]) assert.throws(()=>planPlayerActions('five-ai',[action]));
  assert.throws(()=>planPeerSession('five-ai',{...snapshot(),operation:'launch'}));
});

test('compact groups reuse canonical simultaneous controls and preserve exact eased mouse displacement',()=>{
  const actions=expandActionGroups([
    {slot:2,atMs:0,durationMs:800,hold:['move_forward','strafe_right']},
    {slot:2,atMs:100,durationMs:400,look:{deltaX:97,deltaY:-31,steps:4}},
    {slot:2,atMs:200,durationMs:100,hold:['jump','attack']},
    {slot:3,atMs:200,durationMs:300,hold:['use']},
  ]);
  const plan=planPlayerActions('human-plus-four-ai',actions);
  const look=actions.filter(a=>a.kind==='look_relative');
  assert.equal(actions.length,9);
  assert.deepEqual(look.map(a=>a.atMs),[100,200,300,400]);
  assert.equal(look.reduce((n,a)=>n+a.deltaX,0),97);
  assert.equal(look.reduce((n,a)=>n+a.deltaY,0),-31);
  assert.ok(Math.abs(look[0].deltaX)<Math.abs(look[1].deltaX));
  assert.equal(plan.maximumConcurrentIntents,6);
  assert.equal(plan.execution,'NOT_DISPATCHED');
});
test('compact camera timing partitions non-divisible durations, preserves exact signed totals',()=>{
  const actions=expandActionGroups([
    {slot:2,atMs:13,durationMs:203,look:{deltaX:-101,deltaY:17,steps:3,curve:'linear'}},
  ]);
  assert.deepEqual(actions.map(a=>[a.atMs,a.durationMs]),[[13,67],[80,68],[148,68]]);
  assert.equal(actions.reduce((n,a)=>n+a.deltaX,0),-101);
  assert.equal(actions.reduce((n,a)=>n+a.deltaY,0),17);
  assert.equal(actions.at(-1).atMs+actions.at(-1).durationMs,216);
});
test('compact authoring refuses expansion overflow and unsupported controls, canonical mode/conflicts remain authoritative',()=>{
  for(const groups of [
    [], [{slot:2,atMs:0,durationMs:100,hold:['sprint']}],
    [{slot:2,atMs:0,durationMs:100,hold:['attack','attack']}],
    [{slot:2,atMs:0,durationMs:100,hold:['look_relative']}],
    [{slot:2,atMs:0,durationMs:100,look:{deltaX:1,deltaY:0,steps:3}}],
    [{slot:2,atMs:0,durationMs:100,look:{deltaX:1,deltaY:0,steps:null}}],
    [{slot:2,atMs:0,durationMs:100,look:{deltaX:1,deltaY:0,curve:'random'}}],
    [{slot:2,atMs:0,durationMs:100,hold:['attack'],command:'unsafe'}],
    Array.from({length:5},(_,i)=>({slot:2,atMs:i*100,durationMs:100,
      hold:['move_forward','move_backward','strafe_left','strafe_right','jump','sneak','attack','use']})),
  ])assert.throws(()=>expandActionGroups(groups));
  assert.throws(()=>planPlayerActions('human-plus-four-ai',expandActionGroups([
    {slot:1,atMs:0,durationMs:100,hold:['attack']},
  ])),/human-owned/);
  assert.throws(()=>planPlayerActions('five-ai',expandActionGroups([
    {slot:2,atMs:0,durationMs:100,hold:['move_forward','move_backward']},
  ])),/input channel/);
});
