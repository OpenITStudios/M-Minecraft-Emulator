// Source authored only. Operator has not authorized running Node tests.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createVisualObserver} from './visual-observer.mjs';

function clockFixture() {
  let now=1000;
  const queue=[];
  const schedule=(fn,ms)=>{
    const job={fn,ms,cancelled:false,unref(){}};
    queue.push(job);return job;
  };
  const cancel=job=>{job.cancelled=true;};
  const next=async()=>{
    const job=queue.find(x=>!x.cancelled);
    if(!job) throw new Error('Expected pending source fixture task');
    job.cancelled=true;now+=job.ms;
    await job.fn();
  };
  return {clock:()=>now,schedule,cancel,next,advance:ms=>{now+=ms;}};
}
function frame(slot,sha='a'.repeat(64),windowId=102) {
  return {metadata:{slot,provider:'CUA_DRIVER_WINDOW_CAPTURE',sha256:sha,
    width:800,height:600,screenshotStatus:'PNG_RETURNED_NOT_GAMEPLAY_PROOF',
    processIdentity:{pid:210+slot,windowId,createdFileTime:'134000000000000001',
      sessionId:1},background:true},
    imageBase64:'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwC' };
}

test('one local sampler retains a frame but only delivers it upon a read',async()=>{
  const t=clockFixture();
  const calls=[];
  const observer=createVisualObserver({...t,capture:async slot=>{
    calls.push(slot);return frame(slot);
  }});
  const started=observer.start([2],{intervalMs:800,durationMs:10000});
  assert.equal(started.liveVideo,false);
  assert.equal(observer.read(2).metadata.status,'WAITING_FOR_FIRST_FRAME');
  await t.next();
  const first=observer.read(2,{afterSequence:0,includeImage:true});
  assert.equal(first.metadata.hasNewImage,true);
  assert.equal(first.metadata.sequence,1);
  assert.deepEqual(calls,[2]);
  const noRedelivery=observer.read(2,{afterSequence:1});
  assert.equal(noRedelivery.imageBase64,null);
  await t.next();
  assert.equal(observer.read(2,{afterSequence:1}).metadata.hasNewImage,false);
  observer.shutdown();
});

test('an older cached image is refused until a new local sample exists',async()=>{
  const t=clockFixture();
  const observer=createVisualObserver({...t,capture:async slot=>frame(slot)});
  observer.start([2],{intervalMs:1250,durationMs:10000});
  await t.next();
  assert.equal(observer.read(2,{afterSequence:0}).metadata.hasNewImage,true);
  t.advance(4000);
  const old=observer.read(2,{afterSequence:0});
  assert.equal(old.metadata.status,'CACHED_FRAME_AGED_OUT');
  assert.equal(old.metadata.hasNewImage,false);
  assert.equal(old.imageBase64,null);
  observer.shutdown();
});

test('static PNG bytes are not reported as proof that Minecraft froze',async()=>{
  const t=clockFixture();
  const observer=createVisualObserver({...t,capture:async slot=>frame(slot)});
  observer.start([2],{intervalMs:800,durationMs:30000});
  await t.next();await t.next();await t.next();await t.next();
  assert.equal(observer.read(2).metadata.status,'STATIC_PNG_BYTES_GAME_STATE_UNKNOWN');
  assert.equal(observer.read(2).metadata.repeatedPngBytes,3);
  observer.shutdown();
});

test('changed native identity discards cached frame and stops sampling',async()=>{
  const t=clockFixture();
  let count=0;
  const observer=createVisualObserver({...t,capture:async slot=>{
    count++;return frame(slot,count===1?'a'.repeat(64):'b'.repeat(64),
      count===1?102:999);
  }});
  observer.start([2],{intervalMs:900,durationMs:10000});
  await t.next();await t.next();
  const result=observer.read(2);
  assert.equal(result.metadata.status,'TARGET_REPLACED_REFUSE_CAPTURE');
  assert.equal(result.imageBase64,null);
  observer.shutdown();
});

test('explicit stop and expiry never retain pixels indefinitely',async()=>{
  const t=clockFixture();
  const observer=createVisualObserver({...t,capture:async slot=>frame(slot)});
  observer.start([2],{intervalMs:900,durationMs:5000});
  await t.next();
  observer.stop([2]);
  assert.equal(observer.read(2).imageBase64,null);
  observer.start([2],{intervalMs:900,durationMs:5000});
  t.advance(5500);
  assert.equal(observer.read(2).metadata.status,'EXPIRED');
  assert.equal(observer.read(2).imageBase64,null);
  observer.shutdown();
});

test('rejects duplicate slots, unlimited durations and second observer session',()=>{
  const t=clockFixture();
  const observer=createVisualObserver({...t,capture:async slot=>frame(slot)});
  assert.throws(()=>observer.start([2,2]),/distinct/);
  assert.throws(()=>observer.start([0]),/slots/);
  assert.throws(()=>observer.start([2],{durationMs:999999}),/duration/);
  observer.start([2]);
  assert.throws(()=>observer.start([2]),/already/);
  observer.shutdown();
});
