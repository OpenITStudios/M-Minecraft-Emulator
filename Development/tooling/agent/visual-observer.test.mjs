// Source authored only. Operator has not authorized running Node tests.
import test from 'node:test';
import assert from 'node:assert/strict';
import {createVisualObserver,summarizeTeamVisualState,
  verifyTeamObservationBasis} from './visual-observer.mjs';

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

test('finishing a failed/expired visual slot calls its capture-worker cleanup once',async()=>{
  const t=clockFixture();
  const finished=[];
  const observer=createVisualObserver({...t,capture:async slot=>frame(slot),
    onFinish:(slot,reason)=>finished.push([slot,reason])});
  observer.start([2],{durationMs:5000});
  await t.next();
  observer.stop([2]);
  observer.stop([2]);
  assert.deepEqual(finished,[[2,'STOPPED_BY_REQUEST']]);
  observer.shutdown();
});

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

test('known replaced native target stops observation immediately',async()=>{
  const t=clockFixture();
  const finished=[];
  let captures=0;
  const observer=createVisualObserver({...t,capture:async()=>{
    captures++;
    throw new Error('GPU_TARGET_IDENTITY_CHANGED');
  },onFinish:(slot,reason)=>finished.push([slot,reason])});
  observer.start([2],{durationMs:5000});
  await t.next();
  const result=observer.read(2);
  assert.equal(result.metadata.status,'TARGET_REPLACED_REFUSE_CAPTURE');
  assert.equal(result.metadata.active,false);
  assert.equal(result.metadata.errorCode,'WINDOW_IDENTITY_UNCONFIRMED');
  assert.equal(result.imageBase64,null);
  assert.equal(captures,1);
  observer.stop([2]);
  observer.shutdown();
  assert.deepEqual(finished,[[2,'TARGET_REPLACED_REFUSE_CAPTURE']]);
});


function visualMeta(slot,overrides={}) {
  return {schema:1,slot,status:'FRAME_BYTES_CHANGED_NOT_SEMANTIC_PROOF',
    active:true,freshFrameAvailable:true,sequence:2,successfulSamples:4,
    frameAgeMs:120,collectedAt:'2026-11-01T04:00:00.000Z',
    captureIdentity:{pid:300+slot,windowId:1000+slot,sessionId:1,
      createdFileTime:'134000000000000001'},
    ...overrides};
}

test('team summary reports four/five fresh cached frames without image bytes or gameplay conclusions',()=>{
  const slots=[2,3,4,5];
  const frames=slots.map(slot=>visualMeta(slot));
  const view=summarizeTeamVisualState('human-plus-four-ai',slots,frames);
  assert.equal(view.status,'RECENT_PER_SLOT_FRAMES_AVAILABLE');
  assert.equal(view.allSlotsHaveRecentFrames,true);
  assert.equal(view.imagesIncluded,false);
  assert.deepEqual(view.frames.map(x=>x.slot),slots);
  assert.equal(view.frames.every(x=>x.imageRead.tool==='read_visual_observation'),true);
  assert.equal(view.frames.every(x=>x.gameplayChange==='NOT_EVALUATED_BY_MINECRAFT_OR_MODEL'),true);
  const full=summarizeTeamVisualState('five-ai',[1,2,3,4,5],
    [1,2,3,4,5].map(slot=>visualMeta(slot)));
  assert.equal(full.frameCount,5);
  assert.throws(()=>summarizeTeamVisualState('human-plus-four-ai',[1,2],
    [1,2].map(slot=>visualMeta(slot))),/Human MC01/);
});

test('visual snapshots keep sample-count changes separate from PNG-byte changes',()=>{
  const slot=3;
  const first=summarizeTeamVisualState('five-ai',[slot],[visualMeta(slot)]);
  const basis=first.frames[0].baseline;
  const sampled=summarizeTeamVisualState('five-ai',[slot],[
    visualMeta(slot,{successfulSamples:5,frameAgeMs:100}),
  ],[basis]);
  assert.equal(sampled.frames[0].sampledSinceBaseline,true);
  assert.equal(sampled.frames[0].pngBytesChangedSinceBaseline,false);
  assert.equal(sampled.motionInference,'PNG_BYTES_AND_TIMESTAMPS_NOT_GAMEPLAY_PROOF');
  const replaced=summarizeTeamVisualState('five-ai',[slot],[
    visualMeta(slot,{captureIdentity:{...basis.captureIdentity,windowId:9999}}),
  ],[basis]);
  assert.equal(replaced.frames[0].observedSameProcessSinceBaseline,false);
  assert.equal(replaced.frames[0].sampledSinceBaseline,false);
});

test('observed action basis rejects incomplete, stale, changed and reordered client identities',()=>{
  const slots=[2,3,4,5],frames=slots.map(slot=>visualMeta(slot));
  const baselines=summarizeTeamVisualState('human-plus-four-ai',
    slots,frames).frames.map(v=>v.baseline);
  const good=verifyTeamObservationBasis('human-plus-four-ai',slots,baselines,frames);
  assert.equal(good.frameCount,4);
  assert.throws(()=>verifyTeamObservationBasis('human-plus-four-ai',
    slots,baselines.slice(1),frames),/STALE_OR_REPLACED/);
  assert.throws(()=>verifyTeamObservationBasis('human-plus-four-ai',
    slots,baselines.map(b=>b.slot===4?{...b,samples:b.samples+1}:b),
    frames),/STALE_OR_REPLACED/);
  assert.throws(()=>verifyTeamObservationBasis('human-plus-four-ai',
    slots,baselines,frames.map(f=>f.slot===5?{...f,
      captureIdentity:{...f.captureIdentity,pid:9999}}:f)),/STALE_OR_REPLACED/);
  assert.throws(()=>verifyTeamObservationBasis('human-plus-four-ai',
    slots,baselines,frames.map(f=>f.slot===3?{...f,
      frameAgeMs:5000,freshFrameAvailable:false}:f)),/INCOMPLETE/);
});

test('cached capture identifiers are available only while the underlying frame is fresh',async()=>{
  const t=clockFixture();
  const observer=createVisualObserver({...t,capture:async slot=>frame(slot)});
  observer.start([2],{intervalMs:800,durationMs:10000});
  await t.next();
  const fresh=observer.read(2,{includeImage:false}).metadata;
  assert.equal(fresh.freshFrameAvailable,true);
  assert.equal(fresh.captureIdentity.pid,212);
  assert.equal(fresh.hasNewImage,false);
  t.advance(4000);
  const old=observer.read(2,{includeImage:false}).metadata;
  assert.equal(old.freshFrameAvailable,false);
  assert.equal(old.captureIdentity,null);
  observer.stop([2]);
  assert.equal(observer.read(2,{includeImage:false}).metadata.captureIdentity,null);
  observer.shutdown();
});
