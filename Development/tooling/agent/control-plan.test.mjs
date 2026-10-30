// Authored contracts; do not run until the operator explicitly permits tests.
import test from 'node:test';
import assert from 'node:assert/strict';
import {slotsForMode,planPeerSession,planPlayerActions} from './control-plan.mjs';

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

test('human keyboard and overlapping per-slot inputs are never admitted',() => {
  assert.throws(()=>planPlayerActions('human-plus-four-ai',[
    {slot:1,kind:'move_forward',atMs:0,durationMs:80},
  ]),/human-owned/);
  assert.throws(()=>planPlayerActions('five-ai',[
    {slot:2,kind:'move_forward',atMs:0,durationMs:300},
    {slot:2,kind:'jump',atMs:200,durationMs:100},
  ]),/Overlapping/);
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
  ]) assert.throws(()=>planPlayerActions('five-ai',[action]));
  assert.throws(()=>planPeerSession('five-ai',{...snapshot(),operation:'launch'}));
});
