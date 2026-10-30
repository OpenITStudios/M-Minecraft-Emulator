// Authored only. No source/local/Windows test execution permitted yet.
import test from 'node:test';
import assert from 'node:assert/strict';
import {EventEmitter} from 'node:events';
import path from 'node:path';
import {
  createGpuPacketDecoder,createGpuFrameSource,validateGpuExecutable,
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
  assert.equal(source.latest(2,target).workerSequence,2);
  assert.throws(()=>source.latest(2),/Native-owned/);
  source.stop([2]);
  assert.equal(killed,1);
  assert.throws(()=>source.latest(2),/NOT_ACTIVE/);
  source.shutdown();
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
