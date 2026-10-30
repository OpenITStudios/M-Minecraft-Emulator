// Source tests authored for future explicit operator approval.
import test from 'node:test';
import assert from 'node:assert/strict';
import {compileGlobalGamepadCandidate} from './global-gamepad-candidate.mjs';
const act=(slot,kind,atMs,durationMs,extra={})=>
  ({slot,kind,atMs,durationMs,...extra});

test('one AI gamepad maps genuine game controls but remains a global device',()=>{
  const out=compileGlobalGamepadCandidate('five-ai',[
    act(2,'move_forward',0,400),
    act(2,'strafe_left',0,400),
    act(2,'attack',0,150),
    act(2,'jump',0,100),
    act(2,'look_relative',0,50,{deltaX:40,deltaY:-12}),
  ]);
  const first=out.frames[0];
  assert.equal(first.lx,-23170);
  assert.equal(first.ly,23170);
  assert.equal(first.rt,255);
  assert.deepEqual(first.buttons,['a']);
  assert.equal(first.rx,5120);
  assert.equal(first.ry,1536);
  assert.equal(out.execution,'NOT_DISPATCHED');
  assert.equal(out.playbackPolicy,'LIVE_PACED_STDIN_REQUIRED_NO_BULK_REPLAY');
  assert.match(out.gamepadRouting,/GLOBAL_DEVICE/);
  assert.deepEqual(out.frames.map(f=>f.atMs).filter(t=>t===200),[200]);
  assert.equal(out.frames.at(-1).ly,0);
  assert.equal(out.frames.at(-1).rt,0);
});
test('human mode, several slots and unbounded trial lengths fail closed',()=>{
  assert.throws(()=>compileGlobalGamepadCandidate('human-plus-four-ai',[
    act(2,'jump',0,100)]),/human/);
  assert.throws(()=>compileGlobalGamepadCandidate('five-ai',[
    act(2,'jump',0,100),act(3,'attack',0,100)]),/multiple/);
  assert.throws(()=>compileGlobalGamepadCandidate('five-ai',[
    act(2,'move_forward',10000,150)]),/ten seconds/);
});
test('neutral-only wait emits no controls',()=>{
  const out=compileGlobalGamepadCandidate('five-ai',[
    act(2,'wait',0,300)]);
  assert.equal(out.frames.every(f=>
    f.lx===0 && f.ly===0 && f.lt===0 && f.rt===0 &&
    f.buttons.length===0),true);
});
