// Authored only. Do not execute without explicit operator authorization.
import test from 'node:test';
import assert from 'node:assert/strict';
import {compilePlayerInputTimeline,compileEmergencyRelease} from './input-timeline.mjs';
const a=(slot,kind,atMs,durationMs,more={})=>
  ({slot,kind,atMs,durationMs,...more});

test('same-client movement camera jump attack form parallel channels',()=>{
  const plan=compilePlayerInputTimeline('human-plus-four-ai',[
    a(2,'move_forward',0,400),a(2,'strafe_left',0,400),
    a(2,'jump',0,100),a(2,'attack',0,100),
    a(2,'look_relative',0,50,{deltaX:25,deltaY:-12}),
  ]);
  assert.equal(plan.maximumConcurrentIntents,5);
  assert.equal(plan.inputEvents.length,9);
  assert.equal(plan.timestampGroups[0].events.length,5);
  assert.equal(plan.inputEvents.find(x=>x.phase==='POINTER_DELTA').deltaY,-12);
  assert.equal(plan.execution,'NOT_DISPATCHED');
  assert.match(plan.emergencyStop,/REQUIRES_REAL_PER_CLIENT_RELEASE_ALL/);
});
test('releases precede new presses at the same timestamp',()=>{
  const plan=compilePlayerInputTimeline('five-ai',[
    a(2,'move_forward',0,200),a(2,'move_backward',200,200),
    a(3,'attack',0,100),
  ]);
  const boundary=plan.timestampGroups.find(g=>g.atMs===200).events;
  assert.deepEqual(boundary.map(e=>e.phase),['RELEASE','PRESS']);
  assert.equal(plan.maximumConcurrentIntents,2);
});
test('emergency release covers held keys but not expired or mouse deltas',()=>{
  const actions=[
    a(2,'move_forward',0,400),a(2,'attack',0,100),
    a(2,'look_relative',0,50,{deltaX:20,deltaY:2}),
    a(3,'jump',0,400),
  ];
  const stop=compileEmergencyRelease('human-plus-four-ai',actions,120);
  assert.deepEqual(stop.releaseEvents.map(e=>[e.slot,e.control]),
    [[2,'move_forward'],[3,'jump']]);
  assert.equal(stop.mustReleaseAllAtProvider,true);
  assert.equal(stop.execution,'NOT_DISPATCHED');
});
test('human target, unknown action or invalid emergency timing refused',()=>{
  assert.throws(()=>compilePlayerInputTimeline('human-plus-four-ai',[
    a(1,'attack',0,100)]),/human-owned/);
  assert.throws(()=>compilePlayerInputTimeline('five-ai',[
    a(2,'powershell',0,100)]),/Unsupported/);
  assert.throws(()=>compileEmergencyRelease('five-ai',[
    a(2,'attack',0,100)],-1),/offset/);
});
