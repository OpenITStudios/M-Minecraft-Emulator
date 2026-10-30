// Authored regression fixtures. Do not run without operator approval.
import test from 'node:test';
import assert from 'node:assert/strict';
import {assessInputRoute,requiredInputSlots,matchesNativeInputTarget} from './input-route.mjs';

const action=(slot,kind,atMs=0,durationMs=150)=>
  ({slot,kind,atMs,durationMs});
const target=(slot,patch={})=>({
  schema:1,ok:true,operation:'target',slot,
  pid:1000+slot,windowId:0x2000+slot,
  createdFileTime:String(134000000000000000n+BigInt(slot)),
  sessionId:1,foreground:slot===1,
  visible:true,minimized:false,
  verification:'SAME_SESSION_PID_BIRTH_ROOT_HWND',gameplayInput:'NOT_VERIFIED',
  ...patch,
});

test('runtime target comparison rejects replaced native PID, HWND or birth time',()=>{
  const actions=[action(2,'move_forward')];
  const plan=assessInputRoute('human-plus-four-ai',actions,
    [target(1),target(2)],{observedAtMs:100000,elapsedMs:10});
  const expected=plan.aiTargets[0];
  assert.equal(matchesNativeInputTarget(expected,target(2)),true);
  assert.equal(matchesNativeInputTarget(expected,target(2,{pid:900})),false);
  assert.equal(matchesNativeInputTarget(expected,target(2,{windowId:900})),false);
  assert.equal(matchesNativeInputTarget(expected,
    target(2,{createdFileTime:'134000000000000500'})),false);
  assert.throws(()=>matchesNativeInputTarget(expected,
    target(2,{verification:'PID_ONLY'})),/not verified/);
});

test('hybrid route pins human window but excludes MC01 from AI actions',()=>{
  const actions=[action(2,'move_forward'),action(3,'attack')];
  const requested=requiredInputSlots('human-plus-four-ai',actions);
  assert.deepEqual(requested.aiSlots,[2,3]);
  assert.deepEqual(requested.observedSlots,[1,2,3]);
  const result=assessInputRoute('human-plus-four-ai',actions,
    [target(1),target(2),target(3)],{observedAtMs:100000,elapsedMs:100});
  assert.equal(result.execution,'NOT_DISPATCHED');
  assert.equal(result.humanForegroundObserved,true);
  assert.deepEqual(result.aiTargets.map(t=>t.slot),[2,3]);
  assert.equal(result.aiTargets.some(t=>t.slot===1),false);
  assert.match(result.deliveryGate,/BLOCKED/);
  assert.equal(result.gameplayInputAcceptance,'NOT_EXECUTED');
});

test('five-AI mode may bind MC01 as an AI window without human assumptions',()=>{
  const result=assessInputRoute('five-ai',[action(1,'jump')],
    [target(1)],{observedAtMs:100000,elapsedMs:1});
  assert.equal(result.aiTargets[0].slot,1);
  assert.equal(result.humanForegroundObserved,null);
  assert.equal(result.releaseAllBackend,'NOT_IMPLEMENTED');
});

test('alias, missing, or stale targets are never accepted',()=>{
  const acts=[action(2,'move_forward'),action(3,'attack')];
  const available=[target(1),target(2),target(3)];
  assert.throws(()=>assessInputRoute('human-plus-four-ai',acts,
    [target(1),target(2),target(2)]),/Duplicate/);
  assert.throws(()=>assessInputRoute('human-plus-four-ai',acts,
    available.slice(0,2)),/Incomplete/);
  assert.throws(()=>assessInputRoute('human-plus-four-ai',acts,
    [target(1),target(2),target(3,{pid:1002})]),/share one native/);
  assert.throws(()=>assessInputRoute('human-plus-four-ai',acts,
    [target(1),target(2),target(3,{windowId:0x2002})]),/share one native/);
  assert.throws(()=>assessInputRoute('human-plus-four-ai',acts,
    available,{elapsedMs:6000}),/stale/);
  assert.throws(()=>assessInputRoute('human-plus-four-ai',acts,
    [target(1),target(2),target(3,{verification:'PID_ONLY'})]),/not verified/);
  assert.throws(()=>assessInputRoute('human-plus-four-ai',acts,
    [target(1),target(2),target(3,{sessionId:2})]),/Cross-session/);
});

test('no game-input authorization can be inferred from observed foreground',()=>{
  const result=assessInputRoute('human-plus-four-ai',
    [action(2,'move_forward')],[target(1,{foreground:false}),target(2)],
    {observedAtMs:100000,elapsedMs:10});
  assert.equal(result.humanForegroundObserved,false);
  assert.equal(result.humanInputPreservation,
    'NOT_PROVEN_BY_NATIVE_WINDOW_INSPECTION');
  assert.match(result.routeType,/NOT_GAME_INPUT/);
  assert.equal(result.actionTimeline.execution,'NOT_DISPATCHED');
});
