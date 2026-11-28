// Authored only. No source/local/Windows test execution permitted yet.
import test from 'node:test';
import assert from 'node:assert/strict';
import {EventEmitter} from 'node:events';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {
  createGpuPacketDecoder,createGpuFrameSource,validateGpuExecutable,inspectWebp,previewSize,
} from './gpu-frame-source.mjs';
const png=Buffer.from(
 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+j6YQAAAAASUVORK5CYII=',
 'base64');
function packet(seq,time=100000) {
  const header=Buffer.alloc(28);
  header.write('LTG1',0,'ascii');
  header.writeUInt32BE(png.length,4);
  header.writeUInt32BE(1,8);
  header.writeUInt32BE(1,12);
  header.writeUInt32BE(seq,16);
  header.writeBigUInt64BE(BigInt(time),20);
  return Buffer.concat([header,png]);
}
test('stream accepts split transport chunks and preserves frame order',()=>{
  const found=[];
  const decoder=createGpuPacketDecoder(frame=>found.push(frame),{clock:()=>100000});
  const first=packet(1),second=packet(2);
  decoder.push(first.subarray(0,10));
  decoder.push(first.subarray(10));
  decoder.push(second);
  assert.deepEqual(found.map(x=>x.sequence),[1,2]);
  assert.equal(found[1].width,1);
  assert.match(found[1].sha256,/^[a-f0-9]{64}$/);
  assert.equal(found[1].sha256,createHash('sha256').update(png).digest('hex'));
  // inspectPng already owns the one full-buffer digest; no second hash path.
  assert.throws(()=>decoder.push(packet(2)),/sequence/);
});
test('split WGC headers, payloads and joined packets keep frame order',()=>{
  const found=[];
  const decoder=createGpuPacketDecoder(frame=>found.push(frame),{clock:()=>100000});
  const first=packet(1),second=packet(2);
  const combined=Buffer.concat([first,second]);
  const cuts=[1,13,27,29,first.length-1,first.length+2,combined.length-1];
  let offset=0;
  for(const end of [...cuts,combined.length]) {
    decoder.push(combined.subarray(offset,end));
    offset=end;
  }
  assert.deepEqual(found.map(frame=>frame.sequence),[1,2]);
  assert.deepEqual(found.map(frame=>frame.bytes),[png,png]);
});

test('two complete WGC frames in one stdout chunk preserve sequence checks',()=>{
  const frames=[];
  const decoder=createGpuPacketDecoder(frame=>frames.push(frame),{clock:()=>100000});
  decoder.push(Buffer.concat([packet(1),packet(2)]));
  assert.deepEqual(frames.map(frame=>frame.sequence),[1,2]);
  assert.throws(()=>decoder.push(packet(2)),/sequence/);
});

test('bad frame length, unknown magic and stale timestamp are refused',()=>{
  const make=()=>createGpuPacketDecoder(()=>{}, {clock:()=>100000});
  assert.throws(()=>make().push(Buffer.from('NOT1'+Buffer.alloc(24).toString('binary'),'binary')),
    /protocol/);
  const corrupt=packet(1);corrupt.writeUInt32BE(4*1024*1024,4);
  assert.throws(()=>make().push(corrupt),/sequence\/size/);
  assert.throws(()=>make().push(packet(1,1)),/timestamp/);
});
test('invalid WGC timestamps are refused from headers before PNG assembly',()=>{
  const headerOnly=packet(1,1).subarray(0,28);
  const rejectStale=createGpuPacketDecoder(()=>{}, {clock:()=>100000});
  assert.throws(()=>rejectStale.push(headerOnly),/timestamp/);
  const future=packet(1,106000).subarray(0,28);
  const rejectFuture=createGpuPacketDecoder(()=>{}, {clock:()=>100000});
  assert.throws(()=>rejectFuture.push(future),/timestamp/);
});

test('WGC timestamp is rechecked after a fragmented frame transfer',()=>{
  let now=100000;
  const frames=[];
  const decoder=createGpuPacketDecoder(frame=>frames.push(frame),
    {clock:()=>now});
  const split=packet(1,100000);
  decoder.push(split.subarray(0,28)); // Valid header, no pixels yet.
  now=190001; // A delayed sender must not make this image fresh.
  assert.throws(()=>decoder.push(split.subarray(28)),/timestamp/);
  assert.deepEqual(frames,[]);
});

test('GPU binary cannot use arbitrary or relative executable names',()=>{
  assert.throws(()=>validateGpuExecutable('./gpu.exe'));
  assert.throws(()=>validateGpuExecutable(path.resolve('powershell.exe')));
  assert.equal(validateGpuExecutable(path.resolve('lazy-tester-gpu-capture.exe')),
    path.resolve('lazy-tester-gpu-capture.exe'));
});
test('one capture worker preserves only latest frame and stops without game controls',async()=>{
  const pipe=new EventEmitter();
  const child=new EventEmitter();child.stdout=pipe;
  let killed=0;child.kill=()=>{killed++;return true;};
  const source=createGpuFrameSource({
    executable:path.resolve('lazy-tester-gpu-capture.exe'),
    platform:'win32',clock:()=>100000,
    statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
    spawnProcess:()=>child,
    schedule:()=>({unref(){}}),cancel:()=>{},
  });
  const target={pid:42,windowId:123,createdFileTime:'134000000000000001',sessionId:1};
  await source.start(2,target);
  pipe.emit('data',packet(1));
  pipe.emit('data',packet(2));
  assert.equal(source.status(2).frames,2);
  const firstRead=source.latest(2,target);
  assert.equal(firstRead.workerSequence,2);
  assert.equal(firstRead.imageBase64,png.toString('base64'));
  assert.equal(source.latest(2,target).imageBase64,firstRead.imageBase64);
  assert.throws(()=>source.latest(2),/Native-owned/);
  source.stop([2]);
  assert.equal(killed,1);
  assert.throws(()=>source.latest(2),/NOT_ACTIVE/);
  source.shutdown();
});

test('stopped WGC worker retains counters but not authority over a restarted slot',async()=>{
  const children=[];
  const target={pid:42,windowId:123,
    createdFileTime:'134000000000000001',sessionId:1};
  const source=createGpuFrameSource({
    executable:path.resolve('lazy-tester-gpu-capture.exe'),
    platform:'win32',clock:()=>100000,
    statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
    spawnProcess:()=>{
      const child=new EventEmitter();
      child.stdout=new EventEmitter();
      child.kills=0;
      child.kill=()=>{child.kills++;return true;};
      children.push(child);
      return child;
    },
    schedule:()=>({unref(){}}),cancel:()=>{},
  });
  try {
    await source.start(2,target);
    children[0].stdout.emit('data',packet(1));
    assert.equal(source.status(2).frames,1);
    source.stop([2]);
    assert.deepEqual(source.status(2),{
      slot:2,status:'STOPPED_BY_REQUEST',frames:1,errors:0,active:false,
    });
    assert.equal(children[0].kills,1);
    assert.throws(()=>source.latest(2,target),/GPU_SOURCE_NOT_ACTIVE/);

    await assert.rejects(source.start(2,target),/GPU_WORKER_STOP_PENDING/);
    assert.equal(children.length,1);
    children[0].emit('close');
    await source.start(2,target);
    // A late old-worker callback must not corrupt the new active worker.
    children[0].stdout.emit('data',packet(2));
    children[0].emit('close');
    assert.equal(source.status(2).status,'STARTING');
    children[1].stdout.emit('data',packet(1));
    assert.equal(source.latest(2,target).workerSequence,1);
    assert.equal(source.status(2).frames,1);
    assert.equal(children[1].kills,0);
  } finally {source.shutdown();}
  assert.equal(children[1].kills,1);
});

test('identical new WGC frame retains base64 without losing sequence and time',async()=>{
  const pipe=new EventEmitter();
  const child=new EventEmitter();child.stdout=pipe;
  child.kill=()=>true;
  let now=100000;
  const source=createGpuFrameSource({
    executable:path.resolve('lazy-tester-gpu-capture.exe'),
    platform:'win32',clock:()=>now,
    statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
    spawnProcess:()=>child,
    schedule:()=>({unref(){}}),cancel:()=>{},
  });
  const target={pid:42,windowId:123,
    createdFileTime:'134000000000000001',sessionId:1};
  try {
    await source.start(2,target);
    pipe.emit('data',packet(1,100000));
    const first=source.latest(2,target);
    assert.equal(first.imageBase64,png.toString('base64'));
    now=100001;
    pipe.emit('data',packet(2,100001));
    const second=source.latest(2,target);
    assert.equal(second.workerSequence,2);
    assert.equal(second.capturedAtMs,100001);
    assert.equal(second.imageBase64,first.imageBase64);
    assert.equal(source.status(2).frames,2);
  } finally {source.shutdown();}
});

test('worker never attributes a cached old-client PNG to a replacement',async()=>{
  const target={pid:42,windowId:123,
    createdFileTime:'134000000000000001',sessionId:1};
  const changes=[
    {pid:43},
    {windowId:124},
    {createdFileTime:'134000000000000002'},
    {sessionId:2},
  ];
  for(const changed of changes) {
    const pipe=new EventEmitter();
    const child=new EventEmitter();child.stdout=pipe;
    let killed=0;child.kill=()=>{killed++;return true;};
    const source=createGpuFrameSource({
      executable:path.resolve('lazy-tester-gpu-capture.exe'),
      platform:'win32',clock:()=>100000,
      statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
      spawnProcess:()=>child,
      schedule:()=>({unref(){}}),cancel:()=>{},
    });
    await source.start(2,target);
    pipe.emit('data',packet(1));
    assert.throws(()=>source.latest(2,{...target,...changed}),
      /GPU_TARGET_IDENTITY_CHANGED/);
    assert.equal(source.status(2).status,'TARGET_IDENTITY_CHANGED');
    assert.equal(source.status(2).active,false);
    assert.equal(killed,1);
    assert.throws(()=>source.latest(2,target),/GPU_SOURCE_NOT_ACTIVE/);
    source.shutdown();
  }
});

test('future-dated native frames are not delivered as recent pixels',async()=>{
  let now=100000;
  const pipe=new EventEmitter();
  const child=new EventEmitter();child.stdout=pipe;
  child.kill=()=>true;
  const source=createGpuFrameSource({
    executable:path.resolve('lazy-tester-gpu-capture.exe'),
    platform:'win32',clock:()=>now,
    statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
    spawnProcess:()=>child,
    schedule:()=>({unref(){}}),cancel:()=>{},
  });
  const target={pid:42,windowId:123,
    createdFileTime:'134000000000000001',sessionId:1};
  await source.start(2,target);
  pipe.emit('data',packet(1,100100));
  assert.throws(()=>source.latest(2,target),/GPU_FRAME_NOT_READY/);
  now=100100;
  assert.equal(source.latest(2,target).workerSequence,1);
  source.shutdown();
});

test('pending GPU source reservation cancels before spawn on stop and shutdown',async()=>{
  const target={pid:42,windowId:123,
    createdFileTime:'134000000000000001',sessionId:1};
  const file={isFile:()=>true,isSymbolicLink:()=>false};
  for(const cancelVia of ['stop','shutdown']) {
    let resolveStat;
    const pendingFile=new Promise(resolve=>{resolveStat=resolve;});
    let spawned=0;
    const source=createGpuFrameSource({
      executable:path.resolve('lazy-tester-gpu-capture.exe'),
      platform:'win32',clock:()=>100000,
      statFile:()=>pendingFile,
      spawnProcess:()=>{spawned++;throw Error('unintended worker');},
      schedule:()=>({unref(){}}),cancel:()=>{},
    });
    const pending=source.start(2,target);
    await assert.rejects(()=>source.start(2,target),/already active or starting/);
    if(cancelVia==='stop') source.stop([2]);
    else source.shutdown();
    resolveStat(file);
    await assert.rejects(pending,/GPU_SOURCE_START_CANCELLED/);
    assert.equal(spawned,0);
    source.shutdown();
  }
});

test('startup refuses and kills an unsupervisable GPU worker',async()=>{
  let killed=0;
  const source=createGpuFrameSource({
    executable:path.resolve('lazy-tester-gpu-capture.exe'),
    platform:'win32',clock:()=>100000,
    statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
    spawnProcess:()=>({stdout:null,kill(){killed++;}}),
    schedule:()=>({unref(){}}),cancel:()=>{},
  });
  const target={pid:42,windowId:123,
    createdFileTime:'134000000000000001',sessionId:1};
  await assert.rejects(()=>source.start(2,target),/bounded stdout pipe/);
  assert.equal(killed,1);
  source.shutdown();
});

 // Structural packet fixtures only; Rust decoder fixture covers actual encoding.
function webpPacket(width=640,height=360,sequence=1,time=100000) {
  const bytes=Buffer.alloc(26);
  bytes.write('RIFF',0);bytes.writeUInt32LE(18,4);bytes.write('WEBPVP8L',8);
  bytes.writeUInt32LE(6,16);bytes[20]=0x2f;
  bytes.writeUInt32LE((width-1)|((height-1)<<14),21);
  const header=Buffer.alloc(28);header.write('LTG2',0);
  header.writeUInt32BE(bytes.length,4);header.writeUInt32BE(width,8);
  header.writeUInt32BE(height,12);header.writeUInt32BE(sequence,16);
  header.writeBigUInt64BE(BigInt(time),20);
  return Buffer.concat([header,bytes]);
}
test('explicit WebP decoder checks exact negotiated size and format, with no PNG downgrade',()=>{
  const frames=[],size=previewSize(320,180);
  const decoder=createGpuPacketDecoder(f=>frames.push(f),{clock:()=>100000,webpSize:size});
  const bytes=webpPacket(320,180);
  decoder.push(bytes.subarray(0,29));decoder.push(bytes.subarray(29));decoder.finish();
  assert.equal(frames.length,1);assert.equal(frames[0].mimeType,'image/webp');
  assert.equal(inspectWebp(frames[0].bytes).width,320);
  const make=()=>createGpuPacketDecoder(()=>{}, {clock:()=>100000,webpSize:size});
  assert.throws(()=>make().push(packet(1)),/protocol/);
  assert.throws(()=>make().push(webpPacket()),/sequence\/size/);
  const partial=make();partial.push(bytes.subarray(0,bytes.length-1));
  assert.throws(()=>partial.finish(),/Incomplete/);
  const mismatch=Buffer.from(bytes);mismatch.writeUInt32LE(0,28+21);
  assert.throws(()=>make().push(mismatch),/dimensions/);
  assert.throws(()=>previewSize(4096,360),/Invalid tool arguments/);
});
test('one-shot preview uses the existing executable, exact target, bounded exit and no stream',async()=>{
  const target={pid:42,windowId:123,createdFileTime:'134000000000000001',sessionId:1};
  let calls=0,output=webpPacket(320,180);
  const source=createGpuFrameSource({
    executable:path.resolve('lazy-tester-gpu-capture.exe'),platform:'win32',clock:()=>100000,
    statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
    spawnProcess:()=>{throw Error('One shot must not create a persistent stream');},
    executePreview:async(_exe,args,options)=>{
      calls++;assert.deepEqual(args,['--preview','--pid','42','--window-id','123',
        '--birth',target.createdFileTime,'--session','1','--width','320','--height','180']);
      assert.equal(options.shell,false);assert.equal(options.timeout,8000);
      assert.equal(options.maxBuffer,3*1024*1024+28);
      return {stdout:output};
    },
  });
  try {
    const image=await source.captureOnce(2,target,{width:320,height:180});
    assert.equal(image.mimeType,'image/webp');assert.equal(image.width,320);
    assert.ok(Buffer.isBuffer(image.bytes));
    assert.equal(Object.hasOwn(image,'imageBase64'),false);
    assert.equal(source.status(2).active,false);
    output=Buffer.concat([webpPacket(320,180),webpPacket(320,180,2)]);
    await assert.rejects(source.captureOnce(2,target,{width:320,height:180}),/GPU_PREVIEW_UNAVAILABLE/);
    output=packet(1);
    await assert.rejects(source.captureOnce(2,target,{width:320,height:180}),/GPU_PREVIEW_UNAVAILABLE/);
    assert.equal(calls,3);
  }finally{source.shutdown();}
});
test('WebP stream and one-shot share slot admission; cached image is not re-encoded',async()=>{
  const target={pid:42,windowId:123,createdFileTime:'134000000000000001',sessionId:1};
  const child=new EventEmitter();child.stdout=new EventEmitter();child.kill=()=>true;
  let calls=0;
  const source=createGpuFrameSource({
    executable:path.resolve('lazy-tester-gpu-capture.exe'),platform:'win32',clock:()=>100000,
    statFile:async()=>({isFile:()=>true,isSymbolicLink:()=>false}),
    spawnProcess:(_exe,args)=>{assert.equal(args[0],'--webp');assert.deepEqual(args.slice(-4),
      ['--width','320','--height','180']);return child;},
    executePreview:async()=>{calls++;return {stdout:webpPacket(320,180)};},
    schedule:()=>({unref(){}}),cancel:()=>{},
  });
  try {
    await source.start(2,target,{webpSize:{width:320,height:180}});
    child.stdout.emit('data',webpPacket(320,180));
    const deferred=source.latest(2,target,{deferImage:true});
    assert.equal(deferred.mimeType,'image/webp');
    assert.equal(Object.hasOwn(deferred,'imageBase64'),false);
    assert.equal(Object.hasOwn(deferred,'bytes'),false);
    child.stdout.emit('data',webpPacket(320,180,2));
    const next=source.latest(2,target,{deferImage:true});
    assert.equal(next.workerSequence,2);
    assert.equal(next.imageRef,deferred.imageRef);
    const frame=source.latest(2,target);
    assert.equal(frame.imageBase64,deferred.imageRef.read());
    assert.equal(source.latest(2,target).imageBase64,frame.imageBase64);
    await assert.rejects(source.captureOnce(2,target,{width:320,height:180}),/USE_EXISTING_SAMPLER/);
    assert.equal(calls,0);
    source.stop([2]);
    await assert.rejects(source.captureOnce(2,target,{width:320,height:180}),/GPU_WORKER_STOP_PENDING/);
    child.emit('close');
    assert.equal((await source.captureOnce(2,target,{width:320,height:180})).mimeType,'image/webp');
  }finally{source.shutdown();}
});
