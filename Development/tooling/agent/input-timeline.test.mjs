// Authored only. Do not execute without explicit operator authorization.
import test from 'node:test';
import assert from 'node:assert/strict';
import {compilePlayerInputTimeline,compileEmergencyRelease,compilePerClientInputFrames,GAME_BUTTON_BITS} from './input-timeline.mjs';
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

test('per-client input frames preserve independent slot masks and synchronized batches',()=>{
  const result=compilePerClientInputFrames('human-plus-four-ai',[
    a(2,'move_forward',0,300),a(2,'attack',0,100),
    a(2,'look_relative',0,50,{deltaX:25,deltaY:-10}),
    a(3,'strafe_right',0,200),a(4,'jump',100,150),
  ]);
  assert.equal(result.execution,'NOT_DISPATCHED');
  assert.deepEqual(result.clients,[2,3,4]);
  assert.deepEqual(result.humanSlots,[1]);
  assert.equal(result.maximumSameTimestampSlots,2);
  assert.deepEqual(result.batches[0].frames.map(f=>f.slot),[2,3]);
  assert.deepEqual(result.batches[0].frames[0].buttons,['move_forward','attack']);
  assert.equal(result.batches[0].frames[0].keyMask,
    GAME_BUTTON_BITS.move_forward|GAME_BUTTON_BITS.attack);
  assert.deepEqual([result.batches[0].frames[0].lookDX,
    result.batches[0].frames[0].lookDY],[25,-10]);
  assert.deepEqual(result.batches[0].frames[1].buttons,['strafe_right']);
  assert.equal(result.batches[1].atMs,100);
  assert.equal(result.batches[1].frames.find(f=>f.slot===2).keyMask,1);
  assert.equal(result.batches[1].frames.find(f=>f.slot===4).keyMask,16);
  assert.deepEqual(result.releaseAllSlots,[2,3,4]);
});
test('release-before-press swap and same-key retrigger remain explicit',()=>{
  const swap=compilePerClientInputFrames('five-ai',[
    a(1,'move_forward',0,200),a(1,'move_backward',200,100),
    a(5,'use',200,100),
  ]);
  assert.deepEqual(swap.humanSlots,[]);
  assert.deepEqual(swap.batches[1].frames.map(f=>[f.slot,f.keyMask]),
    [[1,GAME_BUTTON_BITS.move_backward],[5,GAME_BUTTON_BITS.use]]);
  const repeated=compilePerClientInputFrames('five-ai',[
    a(2,'attack',0,100),a(2,'attack',100,100),
  ]);
  assert.deepEqual(repeated.batches[1].frames[0].retriggeredButtons,['attack']);
});

test('protected MC01 and unmapped camera macros cannot become input frames',()=>{
  assert.throws(()=>compilePerClientInputFrames('human-plus-four-ai',[
    a(1,'attack',0,100)]),/human-owned/);
  assert.throws(()=>compilePerClientInputFrames('five-ai',[
    a(2,'look_left',0,100)]),/mapped buttons/);
  const wait=compilePerClientInputFrames('human-plus-four-ai',[
    a(4,'wait',0,150)]);
  assert.deepEqual(wait.clients,[4]);
  assert.equal(wait.frameCount,0);
});
